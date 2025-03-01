use crate::extensions::types::{
    ExtensionParent, ExtensionRepository, ExtensionRuntimeModel,
    PartitionRuntimeModel,
};
use crate::persist::PersistedData;
use crate::state::{Extension, Mod, RepositoryType};
use futures::stream::iter;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt::{format, Display, Formatter};
use std::fs::{copy, create_dir_all, File};
use std::io::{Cursor, Write};
use std::path::PathBuf;
use home::home_dir;
use tauri::State;
use uuid::Uuid;
use rand::{random, thread_rng, Rng};
use crate::util::rand::generate_random_id;

#[tauri::command]
pub async fn set_mod_state(
    updated: Vec<Mod>,
    persisted_data: State<'_, PersistedData>,
) -> Result<(), ()> {
    persisted_data.put_value("mods", updated);

    Ok(())
}

#[tauri::command]
pub async fn get_mod_state(persisted_data: State<'_, PersistedData>) -> Result<Vec<Mod>, ()> {
    Ok(persisted_data.read_value("mods").unwrap_or(Vec::new()))
}

#[derive(Debug)]
pub enum ModExtGenerationError {
    NetworkError(reqwest::Error),
    SerdeError(serde_json::Error),
    IOError(std::io::Error),
}

impl Display for ModExtGenerationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let err = match self {
            ModExtGenerationError::NetworkError(e) => e.to_string(),
            ModExtGenerationError::SerdeError(e) => e.to_string(),
            ModExtGenerationError::IOError(e) => e.to_string(),
        };

        write!(f, "{}", err)
    }
}

#[derive(Deserialize, Clone)]
struct ModVersionInfo {
    game_versions: Option<Vec<String>>,
    loaders: Option<HashSet<String>>,
    id: String,
    project_id: String,
}

#[derive(Deserialize, Serialize, Clone)]
struct ModsStore {
    lookup: HashMap<String, Extension>,
}

impl ModsStore {
    fn lookup(
        &self,
        mods: &Vec<Mod>,
    ) -> Option<&Extension> {
        let key = mods.iter().map(|it|
            format!("{}:{}", it.loader, it.project_id)
        ).collect::<Vec<String>>().join(",");

        self.lookup.get(&key)
    }

    fn put(
        &mut self,
        mods: &Vec<Mod>,
        extension: Extension,
    ) {
        let key = mods.iter().map(|it|
            format!("{}:{}", it.loader, it.project_id)
        ).collect::<Vec<String>>().join(",");

        self.lookup.insert(key, extension);
    }
}

pub async fn get_mod_extension(
    mods: &Vec<Mod>,
    path: PathBuf,
) -> Result<Extension, ModExtGenerationError> {
    let mods_lookup_path = path.join("mods.json");

    let mut store: ModsStore = if !mods_lookup_path.exists() {
        create_dir_all(mods_lookup_path.parent().unwrap())
            .map_err(ModExtGenerationError::IOError)?;

        ModsStore {
            lookup: HashMap::new(),
        }
    } else {
        let file = File::open(&mods_lookup_path)
            .map_err(ModExtGenerationError::IOError)?;

        serde_json::from_reader(
            file
        ).map_err(ModExtGenerationError::SerdeError)?
    };

    if let Some(extension) = store.lookup(&mods) {
        Ok(extension.clone())
    } else {
        let generated: Extension = generate_mod_extension(
            &mods,
            path,
        ).await?;

        store.put(&mods, generated.clone());

        let file = File::create(mods_lookup_path)
            .map_err(ModExtGenerationError::IOError)?;

        serde_json::to_writer(file, &store)
            .map_err(ModExtGenerationError::SerdeError)?;

        Ok(generated)
    }
}

pub async fn generate_mod_extension(
    mods: &Vec<Mod>,
    path: PathBuf,
) -> Result<Extension, ModExtGenerationError> {
    let client = reqwest::Client::new();
    let requested_loaders = mods.iter().map(|t| t.loader.clone()).collect();

    let mods = mods.iter().map(|it| async {
        let response = client
            .get(format!(
                "https://api.modrinth.com/v2/project/{}/version",
                it.project_id
            ))
            .send()
            .await
            .map_err(|e| ModExtGenerationError::NetworkError(e))?;
        let bytes = response
            .bytes()
            .await
            .map_err(|e| ModExtGenerationError::NetworkError(e))?;

        let info: Vec<ModVersionInfo> = serde_json::from_reader(Cursor::new(bytes))
            .map_err(|e| ModExtGenerationError::SerdeError(e))?;
        Ok(info)
    });
    let mods = futures::future::join_all(mods).await;
    let mods: Vec<ModVersionInfo> = mods
        .into_iter()
        .collect::<Result<Vec<Vec<ModVersionInfo>>, ModExtGenerationError>>()?
        .into_iter()
        .flat_map(|t| t)
        .collect();

    let mut target_partitions: HashMap<String, Vec<ModVersionInfo>> = HashMap::new();
    mods.into_iter().for_each(|it| {
        if let Some(versions) = &it.game_versions {
            versions.iter().for_each(|version| {
                if !target_partitions.contains_key(version) {
                    target_partitions.insert(version.clone(), Vec::new());
                }

                target_partitions.get_mut(version).unwrap().push(it.clone());
            });
        }
    });

    let target_partitions = target_partitions
        .iter()
        .map(|(version, info)| {
            let mut projects: HashSet<String> = HashSet::new();
            let info = info.iter().filter(|it| {
                if let Some(loaders) = &it.loaders {
                    loaders.intersection(&requested_loaders).count() != 0
                        && projects.insert(it.project_id.clone())
                } else {
                    false
                }
            });

            PartitionRuntimeModel {
                r#type: "minecraft".to_string(),
                name: version.clone(),
                repositories: vec![ExtensionRepository {
                    r#type: "fabric-mod:modrinth".to_string(),
                    settings: Default::default(),
                }],
                dependencies: info
                    .map(|mod_info| {
                        HashMap::from([
                            ("projectId".to_string(), mod_info.project_id.clone()),
                            ("versionId".to_string(), mod_info.id.clone()),
                        ])
                    })
                    .collect(),
                options: HashMap::from([("versions".to_string(), version.clone())]),
            }
        })
        .collect::<Vec<PartitionRuntimeModel>>();

    let runtime_model = ExtensionRuntimeModel {
        api_version: 2,
        group_id: "dev.extframework.generated".to_string(),
        name: format!("mods-{}", generate_random_id(8)),
        version: "1".to_string(),
        repositories: vec![HashMap::from([(
            "location".to_string(),
            "https://repo.extframework.dev/registry".to_string(),
        )]), HashMap::from([(
            "location".to_string(),
            home_dir().unwrap().join(".m2").join("repository").to_str().unwrap().to_string(),
        ), (
            "type".to_string(), "local".to_string()
        )])],
        parents: vec![ExtensionParent {
            group: "dev.extframework.integrations".to_string(),
            extension: "fabric-ext".to_string(),
            version: "1.0.2-BETA".to_string(),
        }],
        partitions: target_partitions,
        // .iter()
        // .filter(|it| it.name == version)
        // .map(|it| PartitionModelReference {
        //     r#type: "target".to_string(),
        //     name: it.name.clone(),
        // })
        // .collect(),
    };

    let version_path = path
        .join(
            runtime_model
                .group_id
                .replace(".", std::path::MAIN_SEPARATOR_STR),
        )
        .join(&runtime_model.name)
        .join(&runtime_model.version);

    create_dir_all(&version_path).map_err(|e| ModExtGenerationError::IOError(e))?;

    let erm_path = version_path.join(format!(
        "{}-{}-erm.json",
        runtime_model.name, runtime_model.version
    ));
    let erm_path = File::create(erm_path).map_err(|e| ModExtGenerationError::IOError(e))?;

    serde_json::to_writer(erm_path, &runtime_model)
        .map_err(|e| ModExtGenerationError::SerdeError(e))?;
    //
    // for prm in target_partitions {
    //     let prm_path = version_path.join(format!(
    //         "{}-{}-{}.json",
    //         runtime_model.name, runtime_model.version, prm.name
    //     ));
    //     let prm_path = File::create(prm_path).map_err(|e| ModExtGenerationError::IOError(e))?;
    //
    //     serde_json::to_writer(prm_path, &prm).map_err(|e| ModExtGenerationError::SerdeError(e))?;
    // }

    Ok(Extension {
        descriptor: format!(
            "{}:{}:{}",
            runtime_model.group_id, runtime_model.name, runtime_model.version
        ),
        repository: path.to_str().unwrap().to_string(),
        repository_type: RepositoryType::LOCAL,
    })
}




#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_empty_mod_ext_creation() {
        generate_mod_extension(
            &vec![
                Mod {
                    project_id: "u6dRKJwZ".to_string(),
                    loader: "fabric".to_string(),
                },
                Mod {
                    project_id: "51VWX4KM".to_string(),
                    loader: "forge".to_string(),
                },
            ],
            PathBuf::from("tests/repo"),
        )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_generate_mod_store() {
        get_mod_extension(
            &vec![

            ],
            PathBuf::from("tests/repo"),
        )
            .await
            .unwrap();
    }

}
