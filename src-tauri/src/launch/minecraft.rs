use crate::launch::minecraft::Error::{
    InvalidInfo, Network, Serde, UnknownVersion, ZipExtract, IO,
};
use crate::task::copy::copy_stream_tracking;
use crate::task::{Progress, Task, TaskManager};
use crate::util::{map_async, Compress};
use bytes::Bytes;
use discord_rich_presence::new_client;
use futures::future::join_all;
use futures::stream::{FuturesUnordered, StreamExt};
use futures::{FutureExt, TryFutureExt};
use serde::{Deserialize, Serialize};
use serde_urlencoded::{from_bytes, from_reader};
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::fmt::{format, Display, Formatter};
use std::fs::{create_dir, File};
use std::future::Future;
use std::io::{copy, Cursor};
use std::ops::DerefMut;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use reqwest::Client;
use tokio::fs::create_dir_all;
use tokio::io::{self, AsyncWriteExt};
use uuid::serde::urn::deserialize;
use zip_extract::{extract, ZipExtractError};
use crate::launch::lib_patch::{fetch_library_patches, patch_library};

#[derive(Debug)]
pub enum Error {
    Network(reqwest::Error),
    Serde(serde_json::error::Error),
    UnknownVersion(String),
    IO(io::Error),
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
    pub client_jar: PathBuf,
    pub libraries: Vec<PathBuf>,
    pub asset_path: PathBuf,
    pub asset_index_name: String,
    pub natives_path: PathBuf,
    pub arguments: Arguments,
    pub main_class: String,
    pub java_version: JavaVersion,
}

pub trait FormatForCommand {
    fn format(
        &self,
        values: &HashMap<&str, String>,
    ) -> Vec<String>;

    fn apply(
        &self,
        command: &mut Command,
        values: &HashMap<&str, String>,
    ) {
        let format = self.format(values);

        for arg in format {
            command.arg(arg);
        }
    }
}

fn replace_option_variable(
    str: &String,
    values: &HashMap<&str, String>,
) -> Option<String> {
    let mut str = str.to_string();

    loop {
        if let Some(mut index) = str.find("${") {
            // Find finds the beginning of the occurrence, we want the end (hence add 2)
            let closing_brace = str.find('}').unwrap_or(str.len());
            let name = &str[index + 2..closing_brace];
            let value = values.get(name);

            if let Some(value) = value {
                str.replace_range(index..closing_brace + 1, value)
            } else {
                return None;
            }
        } else {
            return Some(str);
        }
    }
}

// An argument chunk where if not all are substitutable none are returned
impl FormatForCommand for Vec<&[Argument]> {
    fn format(&self, values: &HashMap<&str, String>) -> Vec<String> {
        self.iter().flat_map(|it| {
            it.iter().map(|arg| {
                format_arg(&values, arg)
            }).collect::<Option<Vec<Vec<String>>>>().unwrap_or(vec![])
        }).flatten().collect()
    }
}

fn format_arg(values: &HashMap<&str, String>, arg: &Argument) -> Option<Vec<String>> {
    match arg {
        Argument::Value(s) => {
            match s {
                ValueType::String(str) => {
                    replace_option_variable(str, values)
                        .map(|t| vec![t])
                }
                ValueType::Array(vec) => {
                    vec.iter().map(|s| {
                        replace_option_variable(s, values)
                    }).collect::<Option<Vec<String>>>()
                }
            }
        }
        Argument::ArgumentWithRules {
            rules, value
        } => {
            let os_rule = MinecraftEnvironment::current_os();

            let apply = rules.iter().all(|it| {
                MinecraftEnvironment::apply_rule(
                    &os_rule,
                    it,
                )
            });

            if apply {
                match value {
                    ValueType::String(str) => {
                        replace_option_variable(str, &values).map(|t| vec![t])
                    }
                    ValueType::Array(vec) => {
                        vec.iter().map(|it| {
                            replace_option_variable(it, &values)
                        }).collect::<Option<Vec<String>>>()
                    }
                }
            } else {
                None
            }
        }
    }
}

impl FormatForCommand for Vec<Argument> {
    fn format(&self, values: &HashMap<&str, String>) -> Vec<String> {
        self.iter().flat_map(|arg| {
            let arg = format_arg(values, arg);

            arg.unwrap_or(vec![])
        }).collect()
    }
}

const MINECRAFT_RESOURCES: &'static str = "https://resources.download.minecraft.net";

impl MinecraftEnvironment {
    async fn download_version_info(version: &str, client: &Client, path: &PathBuf) -> Result<(), Error> {
        let response = client.get(VERSION_MANIFEST).send().await.map_err(Network)?;
        let bytes = response.bytes().await.map_err(Network)?;
        let bytes = bytes.as_ref();
        let manifest: VersionManifest = serde_json::from_slice(bytes).map_err(Serde)?;

        let entry = manifest
            .versions
            .iter()
            .find(|v| v.id == version)
            .ok_or(UnknownVersion(version.to_string()))?;

        let url = entry.url;

        let response = client.get(url).send().await.map_err(Network)?;
        let bytes = response.bytes().await.map_err(Network)?;

        copy(&mut Cursor::new(bytes), &mut File::create(path)?)?;

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

    fn apply_rule(os: &OsRule, rule: &Rule) -> bool {
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
    }

    fn filter_library(os: &OsRule, lib: &Library) -> bool {
        if let Some(rules) = &lib.rules {
            rules.iter().any(|rule| {
                Self::apply_rule(os, rule)
            })
        } else {
            true
        }
    }

    pub async fn environment(
        path: PathBuf,
        version: &str,
        tasks: &mut TaskManager,
    ) -> Result<MinecraftEnvironment, Error> {
        let version_path = path.join("versions").join(version);
        if !version_path.exists() {
            create_dir_all(&version_path).await?;
        }

        let client = Client::new();

        let client_json_path = version_path.join(format!("{}.json", version));
        if !client_json_path.exists() {
            Self::download_version_info(version, &client, &client_json_path).await?;
        }

        let mut info: VersionInfo =
            serde_json::from_reader(File::open(&client_json_path)?).map_err(Serde)?;

        // Again, thank you so much Modrinth! You actually saved me like weeks
        let patches = fetch_library_patches()?;
        info.libraries = info.libraries.iter().flat_map(|lib| {
            patch_library(&patches, lib.clone())
        }).collect();

        let client_info = (&info.downloads.get("client"))
            .ok_or(InvalidInfo("No client available to download"))?;

        let client_path = version_path.join(format!("{}.jar", &info.id));

        let client_jar_fut = if !client_path.exists() {
            let mut client_response = client.get(&client_info.url).send()
                .await
                .map_err(Network)?
                .bytes_stream();
            let client_size = client_info.size;

            let fut = tasks.submit("Download Minecraft", |mut task| async move {
                let result: Result<(), Error> = copy_stream_tracking(
                    &mut client_response,
                    &mut File::create(client_path).map_err(IO)?,
                    client_size,
                    &mut task.progress,
                )
                    .await;

                result?;

                task.progress.update(1.0).await;

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
                client: &Client,
                mut tracker: Progress,
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

                        let mut response = client.get(&info.url).send()
                            .await
                            .map_err(Network)?
                            .bytes_stream();

                        let size = info.size;

                        let result: Result<(), Error> = copy_stream_tracking(
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

                        let response = map_async(
                            client.get(&info.url).send().await,
                            |r| r.bytes(),
                        ).await.compress();

                        match response {
                            Ok(bytes) => {
                                extract(Cursor::new(bytes), Path::new(&path), false)
                                    .map_err(ZipExtract)?;

                                tracker.update(1.0).await;

                                Ok(None)
                            }
                            Err(e) => {
                                tracker.erroneously_complete(&e).await;
                                Ok(None)
                            }
                        }
                    }
                }
            }
        }

        let bin_path = path.join("bin");
        if bin_path.exists() {
            std::fs::remove_dir_all(bin_path)?;
        }
        let (client_libraries, libraries_task) = tasks.submit("Download Minecraft libraries", |task: Task| {
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
                request.do_action(&path, &client, Task::child(&arc, (size as f64) / total_size))
            });

            (join_all(futures), arc)
        });

        let assets_path = path.join("assets");
        let indexes_path = assets_path.join("indexes");
        if !indexes_path.exists() {
            create_dir_all(&indexes_path).await?;
        }
        let objects_path = assets_path.join("objects");
        if !objects_path.exists() {
            create_dir_all(&objects_path).await?;
        }

        let indexes_json_path = indexes_path.join(format!("{}.json", info.assets));

        if !indexes_json_path.exists() {
            let asset_index = reqwest::get(&info.asset_index.url).await?.bytes().await?;
            copy(
                &mut Cursor::new(asset_index),
                &mut File::create(&indexes_json_path)?,
            )?;
        }

        let asset_index: AssetObjects =
            serde_json::from_reader(File::open(&indexes_json_path)?).map_err(Serde)?;

        let (asset_fut, assets_task) = tasks.submit("Download Minecraft Assets", |task: Task| {
            let objects = &asset_index.objects;

            let task = Task::to_arc(task);

            let borrowable_task = Arc::clone(&task);

            let total_size = objects
                .iter()
                .map(|o| o.1.size).sum::<u64>();

            let iter = objects
                .iter()
                .map(move |entry| {
                    let checksum = &entry.1.hash;

                    let parent_path = (&objects_path).join(&checksum[0..2].to_string());
                    let path = parent_path.join(&checksum);

                    (path, entry)
                })
                .filter(|entry| !entry.0.exists())
                .map(|entry| {
                    let checksum = &entry.1.1.hash;
                    let size = &entry.1.1.size;

                    let path = entry.0;

                    let mut tracker =
                        Task::child(&borrowable_task, (*size as f64) / (total_size as f64));

                    let client = Arc::new(&client);

                    async move {
                        let mut tries = 0;
                        create_dir_all(&path.parent().unwrap()).await?;

                        let client = Arc::clone(&client);

                        for _ in 0..2 {
                            let mut asset_response = client.get(
                                format!(
                                    "{}/{}/{}",
                                    MINECRAFT_RESOURCES,
                                    checksum[0..2].to_string(),
                                    checksum.to_string()
                                ).as_str(),
                            ).send();

                            tries = tries + 1;
                            let r = async {
                                let mut asset_response = asset_response
                                    .await?
                                    .bytes_stream();


                                let r: Result<(), Error> = copy_stream_tracking(
                                    &mut asset_response,
                                    &mut File::create(&path).map_err(IO)?,
                                    size.clone(),
                                    &mut tracker,
                                )
                                    .await;
                                r?;

                                Ok::<(), Error>(())
                            }.await;

                            if r.is_ok() || (r.is_err() && tries >= 3) {
                                return r;
                            }
                        }
                        Ok(())
                    }
                });

            (join_all(iter), task)
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
        libraries_task.lock().await.update(1.0).await;

        asset_fut
            .await
            .into_iter()
            .collect::<Result<Vec<()>, Error>>()?;
        assets_task.lock().await.update(1.0).await;

        Result::<MinecraftEnvironment, Error>::Ok(MinecraftEnvironment {
            client_jar: version_path.join(format!("{}.jar", &info.id)),
            libraries: library_paths,
            asset_path: assets_path,
            asset_index_name: info.asset_index.id,
            natives_path: path.join("bin"),
            arguments: info.arguments.clone().unwrap_or_else(|| {
                let default_jvm_args = vec![ // This is an option we always want even if MC doesnt say it needs it
                                             Argument::Value(ValueType::String("-Djava.library.path=${natives_directory}".to_string())),
                                             Argument::Value(ValueType::Array(vec!["-cp".to_string(), "${classpath}".to_string()]))
                ];
                if let Some(args) = info.minecraft_arguments {
                    let args = args.split(" ")
                        .map(|str| {
                            Argument::Value(ValueType::String(str.to_string()))
                        })
                        .collect::<Vec<Argument>>();

                    Arguments {
                        game: args,
                        jvm: default_jvm_args,
                    }
                } else {
                    Arguments {
                        game: vec![],
                        jvm: default_jvm_args,
                    }
                }
            }),
            main_class: info.main_class.clone(),
            java_version: info.java_version,
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
    #[serde(rename = "minecraftArguments")]
    minecraft_arguments: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Arguments {
    pub game: Vec<Argument>,
    pub jvm: Vec<Argument>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Argument {
    Value(ValueType),
    ArgumentWithRules { rules: Vec<Rule>, value: ValueType },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ValueType {
    String(String),
    Array(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Rule {
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os: Option<OsRule>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<Features>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OsRule {
    pub name: Option<String>,
    pub arch: Option<String>,
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
pub struct JavaVersion {
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Library {
    pub downloads: LibraryDownloads,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<Rule>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub natives: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extract: Option<LibraryExtract>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LibraryExtract {
    pub exclude: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LibraryDownloads {
    pub artifact: Option<DownloadInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classifiers: Option<HashMap<String, DownloadInfo>>,
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

    #[tokio::test]
    async fn test_apply_args() {
        let args = vec![
            Argument::Value(ValueType::String("--test=${var1}".to_string())),
            Argument::Value(ValueType::String("--test2=${var2}".to_string())),
            Argument::Value(ValueType::String("--test3=${var1}".to_string())),
        ];

        let values = HashMap::from([
            ("var1", "First test".to_string()),
            ("var2", "Second test".to_string()),
        ]);


        let result = args.format(
            &values
        );

        println!("{:#?}", result);

        assert_eq!(result.get(0).unwrap(), "--test=First test");
        assert_eq!(result.get(2).unwrap(), "--test3=First test");
        assert_eq!(result.get(1).unwrap(), "--test2=Second test");
    }
}
