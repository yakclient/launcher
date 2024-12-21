use crate::launch::client::{get_client, get_client_version};
use crate::launch::java::JreSetupError;
use crate::launch::minecraft::MinecraftEnvironment;
use crate::launch::process::{capture_child, launch_process, ProcessStdoutEvent};
use crate::launch::ClientError::{ClientNotRunning, ClientProcessError, IoError, MinecraftSetupErr, ModExtError, NetworkError, Unauthenticated};
use crate::mods::{generate_mod_extension, ModExtGenerationError};
use crate::persist::PersistedData;
use crate::state::{Extension, LaunchInstance, MinecraftAuthentication, Mod};
use crate::task::TaskManager;
use crate::{launcher_status, minecraft_dir, yakclient_dir};
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use futures::TryFutureExt;
use serde::{Serialize, Serializer};
use std::fmt::{Display, Formatter};
use std::fs::create_dir_all;
use std::io;
use std::io::{Error, Read, Write};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use tauri::ipc::Channel;
use tauri::{AppHandle, Emitter, State};
use tokio::io::AsyncReadExt;
use tokio::sync::Mutex;

mod client;
mod java;
mod minecraft;
mod process;
// const CLIENT_VERSION: &'static str = "1.0.11-BETA";

#[derive(Debug)]
pub enum ClientError {
    NetworkError(reqwest::Error),
    IoError(io::Error),
    ClientProcessError(String),
    Unauthenticated,
    ClientNotRunning,
    ClientAlreadyRunning,
    JreInstallError(JreSetupError),
    ModExtError(ModExtGenerationError),
    MinecraftSetupErr(minecraft::Error)
}

impl From<Error> for ClientError {
    fn from(value: Error) -> Self {
        IoError(value)
    }
}


impl From<reqwest::Error> for ClientError {
    fn from(value: reqwest::Error) -> Self {
        NetworkError(value)
    }
}
impl Serialize for ClientError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

impl Display for ClientError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            NetworkError(e) => e.to_string(),
            IoError(e) => e.to_string(),
            ClientProcessError(s) => s.clone(),
            Unauthenticated => "You are not authenticated! Please login first.".into(),
            ClientNotRunning => "The client is not currently running".into(),
            ClientError::ClientAlreadyRunning => "The client is already running".into(),
            ClientError::JreInstallError(t) => t.to_string(),
            ClientError::ModExtError(t) => t.to_string(),
            MinecraftSetupErr(t) => {t.to_string()}
        };
        write!(f, "{}", str)
    }
}

#[tauri::command]
pub async fn launch_minecraft(
    version: String,
    process: State<'_, Arc<Mutex<Option<LaunchInstance>>>>,
    persisted_data: State<'_, PersistedData>,
    console_channel: Channel<ProcessStdoutEvent>,
    discord_client: State<'_, std::sync::Mutex<Option<DiscordIpcClient>>>,
    app: AppHandle,
    tasks: State<'_, Mutex<TaskManager>>
) -> Result<(), ClientError> {
    if process.lock().await.is_some() {
        return Err(ClientError::ClientAlreadyRunning);
    }

    let yakclient_dir = yakclient_dir();

    let mut tasks = tasks.lock().await;

    let client_path = get_client(get_client_version().await?, &mut *tasks)
        .await?;

    println!("Launching Minecraft");
    let ms_auth: Option<MinecraftAuthentication> = persisted_data.read_value("ms_auth");

    let mut extensions: Vec<Extension> = persisted_data
        .read_value("extensions")
        .unwrap_or(Vec::new());
    let java_dir = yakclient_dir.join("runtime");
    create_dir_all(&java_dir).map_err(IoError)?;

    let mods: Vec<Mod> = persisted_data.read_value("mods").unwrap_or(Vec::new());
    if !mods.is_empty() {
        let mod_ext = generate_mod_extension(mods, yakclient_dir.join("repo"), version.clone())
            .await
            .map_err(|e| ModExtError(e))?;

        extensions.push(mod_ext);
    }

    let env = MinecraftEnvironment::environment(
        minecraft_dir(),
        version.as_str(),
        &mut *tasks,
    ).await.map_err(MinecraftSetupErr)?;

    let child = launch_process(
        version.clone(),
        java_dir,
        client_path,
        &ms_auth,
        &extensions,
        &env
    )
    .await?;

    let child = capture_child(child, console_channel);

    let instance = LaunchInstance { child };

    *process.lock().await = Some(instance);

    if let Some(ref mut discord_client) = discord_client.lock().unwrap().deref_mut() {
        let _ = discord_client.set_activity(
            activity::Activity::new()
                .state("Playing Minecraft")
                .details(format!("Extframework {}", version).as_str()),
        );
    };

    Ok(())
}

#[tauri::command]
pub async fn end_launch_process(
    process: State<'_, Arc<Mutex<Option<LaunchInstance>>>>,
    discord_client: State<'_, std::sync::Mutex<Option<DiscordIpcClient>>>,
) -> Result<(), ClientError> {
    let mut guard = process.lock().await;

    println!("PROCESS IS: {}", guard.is_some());
    if let Some(process) = guard.deref() {
        process.shutdown().await
    } else {
        return Err(ClientNotRunning);
    }

    *guard = None;

    if let Some(ref mut discord_client) = discord_client.lock().unwrap().deref_mut() {
        let _ = launcher_status(discord_client);
    };

    Ok(())
}
