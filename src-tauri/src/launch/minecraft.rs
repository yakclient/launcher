use crate::launch::minecraft::Error::{
    InvalidInfo, Network, Serde, UnknownVersion, ZipExtract, IO,
};
use crate::task::copy::{copy_stream_tracking, copy_stream_tracking_async};
use crate::task::{ChildProgressTracker, ProgressTracker, ProgressUpdateAsync, Task, TaskManager};
use discord_rich_presence::new_client;
use futures::future::join_all;
use futures::stream::{FuturesUnordered, StreamExt};
use futures::TryFutureExt;
use serde::{Deserialize, Serialize};
use serde_urlencoded::{from_bytes, from_reader};
use std::collections::HashMap;
use std::fmt::{format, Display, Formatter};
use std::fs::{create_dir, File};
use std::future::Future;
use std::io::{copy, Cursor};
use std::ops::DerefMut;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::create_dir_all;
use tokio::io::{self, AsyncWriteExt};
use uuid::serde::urn::deserialize;
use zip_extract::{extract, ZipExtractError};

#[derive(Debug)]
pub enum Error {
    Network(reqwest::Error),
    Serde(serde_json::error::Error),
    UnknownVersion(String),
    IO(std::io::Error),
    InvalidInfo(&'static str),
    ZipExtract(ZipExtractError),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        IO(value)
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Network(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Network(e) => e.to_string(),
            Serde(e) => e.to_string(),
            UnknownVersion(e) => e.to_string(),
            IO(e) => e.to_string(),
            InvalidInfo(e) => e.to_string(),
            ZipExtract(e) => e.to_string(),
        };

        write!(f, "{}", str)
    }
}

#[derive(Debug)]
pub struct MinecraftEnvironment {
    client_jar: PathBuf,
    natives: PathBuf,
    libraries: Vec<PathBuf>,
}

const MINECRAFT_RESOURCES: &'static str = "https://resources.download.minecraft.net";

impl MinecraftEnvironment {
    async fn download_version_info(version: &str, path: &PathBuf) -> Result<(), Error> {
        let response = reqwest::get(VERSION_MANIFEST).await.map_err(Network)?;
        let bytes = response.bytes().await.map_err(Network)?;
        let bytes = bytes.as_ref();
        let manifest: VersionManifest = serde_json::from_slice(bytes).map_err(Serde)?;

        let entry = manifest
            .versions
            .iter()
            .find(|v| v.id == version)
            .ok_or(UnknownVersion(version.to_string()))?;

        let url = entry.url;

        let response = reqwest::get(url).await.map_err(Network)?;
        let bytes = response.bytes().await.map_err(Network)?;

        copy(&mut Cursor::new(bytes), &mut File::create(path)?)?;

        // let info: VersionInfo = serde_json::from_reader(Cursor::new(bytes)).map_err(Serde)?;

        Ok(())
    }

    fn current_os() -> OsRule {
        let name = if cfg!(target_os = "windows") {
            Some("windows".to_string())
        } else if cfg!(target_os = "macos") {
            Some("osx".to_string())
        } else if cfg!(target_os = "linux") {
            Some("linux".to_string())
        } else if cfg!(target_os = "freebsd") {
            Some("freebsd".to_string())
        } else if cfg!(target_os = "dragonfly") {
            Some("dragonfly".to_string())
        } else if cfg!(target_os = "openbsd") {
            Some("openbsd".to_string())
        } else if cfg!(target_os = "netbsd") {
            Some("netbsd".to_string())
        } else if cfg!(target_os = "android") {
            Some("android".to_string())
        } else {
            None
        };

        let arch = if cfg!(target_arch = "x86_64") {
            Some("x86_64".to_string())
        } else if cfg!(target_arch = "x86") {
            Some("x86".to_string())
        } else if cfg!(target_arch = "arm") {
            Some("arm".to_string())
        } else if cfg!(target_arch = "aarch64") {
            Some("aarch64".to_string())
        } else if cfg!(target_arch = "mips") {
            Some("mips".to_string())
        } else if cfg!(target_arch = "mips64") {
            Some("mips64".to_string())
        } else {
            None // Unsupported or unknown architecture
        };

        OsRule { name, arch }
    }

    fn apply_rules<'a>(os: &OsRule, libraries: &'a [Library]) -> Vec<&'a Library> {
        libraries
            .iter()
            .filter(|lib| Self::filter_library(os, lib))
            .collect()
    }

    fn filter_library(os: &OsRule, lib: &Library) -> bool {
        if let Some(rules) = &lib.rules {
            rules.iter().any(|rule| {
                if rule.action == "allow" {
                    match &rule.os {
                        Some(os_rule) => {
                            (os_rule.name == os.name)
                                && (os_rule.arch.is_none() || os_rule.arch == os.arch)
                        }
                        None => true,
                    }
                } else {
                    false
                }
            })
        } else {
            true
        }
    }

    async fn environment(
        path: PathBuf,
        version: &str,
        tasks: &mut TaskManager,
    ) -> Result<MinecraftEnvironment, Error> {
        let version_path = path.join("versions").join(version);
        create_dir_all(&version_path).await?;

        let client_json_path = version_path.join(format!("{}.json", version));
        if !client_json_path.exists() {
            Self::download_version_info(version, &client_json_path).await?;
        }

        let info: VersionInfo =
            serde_json::from_reader(File::open(&client_json_path)?).map_err(Serde)?;

        let client_info = (&info.downloads.get("client"))
            .ok_or(InvalidInfo("No client available to download"))?;

        let client_path = version_path.join(format!("{}.jar", &info.id));

        let client_jar_fut = if !client_path.exists() {
            let mut client_response = reqwest::get(&client_info.url)
                .await
                .map_err(Network)?
                .bytes_stream();
            let client_size = client_info.size;

            let fut = tasks.submit("Download Minecraft", |mut task| async move {
                let result: Result<(), Error> = copy_stream_tracking(
                    &mut client_response,
                    &mut File::create(client_path).map_err(IO)?,
                    client_size,
                    task.progress.deref_mut(),
                )
                .await;

                result?;

                task.progress.update(1.0);

                Ok::<(), Error>(())
            });

            Some(fut)
        } else {
            None
        };

        enum LibraryProcessRequest {
            DownloadArtifact(DownloadInfo),
            NativeExtract(DownloadInfo),
        }

        impl LibraryProcessRequest {
            async fn do_action(
                self,
                path: &PathBuf,
                mut tracker: ChildProgressTracker,
            ) -> Result<Option<PathBuf>, Error> {
                match self {
                    LibraryProcessRequest::DownloadArtifact(info) => {
                        let path = path
                            .join("libraries")
                            .join(&info.path.expect("Path should be included on library"));

                        if path.exists() {
                            return Ok(Some(path));
                        }

                        create_dir_all(path.parent().unwrap()).await?;

                        let mut response = reqwest::get(&info.url)
                            .await
                            .map_err(Network)?
                            .bytes_stream();

                        let size = info.size;

                        let result: Result<(), Error> = copy_stream_tracking_async(
                            &mut response,
                            &mut File::create(&path).map_err(IO)?,
                            size,
                            &mut tracker,
                        )
                        .await;

                        result?;

                        tracker.update(1.0).await;

                        Ok(Some(path))
                    }
                    LibraryProcessRequest::NativeExtract(info) => {
                        let path = path.join("bin");

                        create_dir_all(&path).await?;

                        let response = reqwest::get(&info.url)
                            .await
                            .map_err(Network)?
                            .bytes()
                            .await?;

                        extract(Cursor::new(response), Path::new(&path), false)
                            .map_err(ZipExtract)?;

                        tracker.update(1.0).await;

                        Ok(None)
                    }
                }
            }
        }

        let client_libraries = tasks.submit("Download Minecraft libraries", |task: Task| {
            let arc = task.to_arc();

            let libraries = info.libraries.clone();
            let iter = libraries
                .into_iter()
                .filter(|lib| Self::filter_library(&Self::current_os(), lib))
                .flat_map(|library| {
                    let mut vec = Vec::<(LibraryProcessRequest, u64)>::new();

                    if let Some(lib) = library.downloads.artifact {
                        vec.push((
                            LibraryProcessRequest::DownloadArtifact(lib.clone()),
                            lib.size,
                        ))
                    }

                    if let Some(natives) = library.natives {
                        let os = Self::current_os();

                        let classifier = natives.get(&os.name.unwrap());

                        if let Some(classifier) = classifier {
                            if let Some(classifiers) = library.downloads.classifiers {
                                if let Some(classifier_download) = classifiers.get(classifier) {
                                    vec.push((
                                        LibraryProcessRequest::NativeExtract(
                                            classifier_download.clone(),
                                        ),
                                        classifier_download.size,
                                    ));
                                }
                            }
                        }
                    }

                    vec
                });

            let vec = iter.collect::<Vec<(LibraryProcessRequest, u64)>>();
            let total_size = vec.iter().map(|(_, size)| *size as f64).sum::<f64>();
            let iter = vec.into_iter();

            let futures = iter.map(|(request, size)| {
                request.do_action(&path, Task::child(&arc, (size as f64) / total_size))
            });

            join_all(futures)
        });

        let assets_path = path.join("assets");
        let indexes_path = assets_path.join("indexes");
        create_dir_all(&indexes_path).await?;
        let objects_path = assets_path.join("objects");
        create_dir_all(&objects_path).await?;

        let indexes_path = indexes_path.join(format!("{}.json", info.id));
        let asset_index = reqwest::get(&info.asset_index.url).await?.bytes().await?;
        copy(
            &mut Cursor::new(asset_index),
            &mut File::create(&indexes_path)?,
        )?;
        let asset_index: AssetObjects =
            serde_json::from_reader(File::open(&indexes_path)?).map_err(Serde)?;

        let (asset_task, asset_fut) = tasks.submit("Download Minecraft Assets", |task: Task| {
            let objects = asset_index.objects;

            let task = Task::to_arc(task);

            let borrowable_task = Arc::clone(&task);

            let iter = objects
                .into_iter()
                .map(move |entry: (String, AssetContent)| {
                    let checksum = &entry.1.hash;

                    let parent_path = (&objects_path).join(&checksum[0..2].to_string());
                    let path = parent_path.join(&checksum);

                    (path, entry)
                })
                .filter(|entry| !entry.0.exists());

            let vec = iter.collect::<Vec<(PathBuf, (String, AssetContent))>>();
            let total_size = vec
                .iter()
                .fold(0u64, |acc, e: &(PathBuf, (String, AssetContent))| {
                    acc + e.1 .1.size
                });

            let iter = vec
                .into_iter()
                .map(|entry: (PathBuf, (String, AssetContent))| {
                    let checksum = entry.1 .1.hash;
                    let size = entry.1 .1.size;

                    let path = entry.0;

                    let mut tracker =
                        Task::child(&borrowable_task, (size as f64) / (total_size as f64));

                    async move {
                        let mut tries = 0;
                        for _ in 0..2 {
                            tries = tries + 1;
                            let r = async {
                                let mut asset_response = reqwest::get(
                                    format!(
                                        "{}/{}/{}",
                                        MINECRAFT_RESOURCES,
                                        checksum[0..2].to_string(),
                                        checksum.to_string()
                                    )
                                    .as_str(),
                                )
                                .await?
                                .bytes_stream();

                                create_dir_all(&path.parent().unwrap()).await?;

                                let r: Result<(), Error> = copy_stream_tracking_async(
                                    &mut asset_response,
                                    &mut File::create(&path).map_err(IO)?,
                                    size,
                                    &mut tracker,
                                )
                                .await;
                                r?;

                                Ok::<(), Error>(())
                            }
                            .await;

                            if r.is_ok() || (r.is_err() && tries >= 3) {
                                return r;
                            }
                        }
                        Ok(())
                    }
                });

            (task, join_all(iter))
        });

        if let Some(fut) = client_jar_fut {
            fut.await?;
        }

        let library_paths = client_libraries
            .await
            .into_iter()
            .collect::<Result<Vec<Option<PathBuf>>, Error>>()?
            .into_iter()
            .filter_map(|x| x)
            .collect::<Vec<_>>();

        asset_fut
            .await
            .into_iter()
            .collect::<Result<Vec<()>, Error>>()?;
        asset_task.lock().await.update(1.0);

        Result::<MinecraftEnvironment, Error>::Ok(MinecraftEnvironment {
            client_jar: version_path.join(format!("{}.jar", &info.id)),
            natives: path.join("bin"),
            libraries: library_paths,
        })
    }
}

// VERSION MANIFEST

pub const VERSION_MANIFEST: &'static str =
    "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json";

#[derive(Deserialize)]
pub struct VersionManifest<'a> {
    #[serde(borrow)]
    pub versions: Vec<VersionEntry<'a>>,
}

#[derive(Deserialize)]
pub struct VersionEntry<'a> {
    pub id: &'a str,
    pub url: &'a str,
    pub sha1: &'a str,
}

// VERSION INFO

#[derive(Serialize, Deserialize, Debug)]
struct VersionInfo {
    arguments: Option<Arguments>,
    #[serde(rename = "assetIndex")]
    asset_index: AssetIndex,
    assets: String,
    #[serde(rename = "complianceLevel")]
    compliance_level: i32,
    downloads: HashMap<String, DownloadInfo>,
    id: String,
    #[serde(rename = "javaVersion")]
    java_version: JavaVersion,
    libraries: Vec<Library>,
    logging: Logging,
    #[serde(rename = "mainClass")]
    main_class: String,
    #[serde(rename = "minimumLauncherVersion")]
    minimum_launcher_version: i32,
    #[serde(rename = "releaseTime")]
    release_time: String,
    time: String,
    #[serde(rename = "type")]
    type_field: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Arguments {
    game: Vec<Argument>,
    jvm: Vec<Argument>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum Argument {
    String(String),
    ArgumentWithRules { rules: Vec<Rule>, value: ValueType },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum ValueType {
    String(String),
    Array(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Rule {
    action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    os: Option<OsRule>,
    #[serde(skip_serializing_if = "Option::is_none")]
    features: Option<Features>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct OsRule {
    name: Option<String>,
    arch: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Features {
    #[serde(rename = "is_demo_user")]
    is_demo_user: Option<bool>,
    #[serde(rename = "has_custom_resolution")]
    has_custom_resolution: Option<bool>,
    #[serde(rename = "has_quick_plays_support")]
    has_quick_plays_support: Option<bool>,
    #[serde(rename = "is_quick_play_singleplayer")]
    is_quick_play_singleplayer: Option<bool>,
    #[serde(rename = "is_quick_play_multiplayer")]
    is_quick_play_multiplayer: Option<bool>,
    #[serde(rename = "is_quick_play_realms")]
    is_quick_play_realms: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
struct AssetIndex {
    id: String,
    sha1: String,
    size: u64,
    #[serde(rename = "totalSize")]
    total_size: u64,
    url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct DownloadInfo {
    sha1: String,
    size: u64,
    url: String,
    path: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JavaVersion {
    component: String,
    #[serde(rename = "majorVersion")]
    major_version: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Library {
    downloads: LibraryDownloads,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    rules: Option<Vec<Rule>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    natives: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    extract: Option<LibraryExtract>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct LibraryExtract {
    exclude: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct LibraryDownloads {
    artifact: Option<DownloadInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    classifiers: Option<HashMap<String, DownloadInfo>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Logging {
    client: LoggingClient,
}

#[derive(Serialize, Deserialize, Debug)]
struct LoggingClient {
    argument: String,
    file: LoggingFile,
    #[serde(rename = "type")]
    type_field: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct LoggingFile {
    id: String,
    sha1: String,
    size: u64,
    url: String,
}

// ASSET INDEX
#[derive(Deserialize, Debug)]
struct AssetObjects {
    pub objects: HashMap<String, AssetContent>,
}

#[derive(Deserialize, Debug)]
struct AssetContent {
    hash: String,
    size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::tests::PrintingTrackerBuilder;
    use crate::task::TrackerBuilder;
    use tokio::fs::create_dir_all;

    #[tokio::test]
    async fn test_download_minecraft() {
        let logs_buf = PathBuf::from("tests/logs");
        create_dir_all(&logs_buf).await.unwrap();
        let mut tasks = TaskManager::new(Box::new(PrintingTrackerBuilder { path: logs_buf }));

        let mc_buf = PathBuf::from("tests/mc");
        create_dir_all(&mc_buf).await.unwrap();
        let env = MinecraftEnvironment::environment(mc_buf, "1.8.9", &mut tasks)
            .await
            .unwrap();

        println!("{:#?}", env);
    }
}