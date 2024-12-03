use crate::launch::client::get_client;
use crate::launch::java::JreSetupError;
use crate::launch::process::{capture_child, launch_process, ProcessStdoutEvent};
use crate::launch::ClientError::{ClientNotRunning, ClientProcessError, IoError, ModExtError, NetworkError, Unauthenticated};
use crate::persist::PersistedData;
use crate::state::{Extension, LaunchInstance, MinecraftAuthentication, Mod};
use crate::yakclient_dir;
use serde::{Serialize, Serializer};
use std::fmt::{Display, Formatter};
use std::fs::create_dir_all;
use std::io;
use std::io::{Read, Write};
use std::ops::Deref;
use std::sync::Arc;
use tauri::ipc::Channel;
use tauri::State;
use tokio::io::AsyncReadExt;
use tokio::sync::Mutex;
use crate::mods::{generate_mod_extension, ModExtGenerationError};

mod client;
mod process;
mod java;

const CLIENT_VERSION: &'static str = "1.0.11-BETA";

#[derive(Debug)]
pub enum ClientError {
    NetworkError(reqwest::Error),
    IoError(io::Error),
    ClientProcessError(String),
    Unauthenticated,
    ClientNotRunning,
    ClientAlreadyRunning,
    JreInstallError(JreSetupError),
    ModExtError(ModExtGenerationError)
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
            NetworkError(e) => { e.to_string() }
            IoError(e) => { e.to_string() }
            ClientProcessError(s) => { s.clone() }
            Unauthenticated => { "You are not authenticated! Please login first.".into() }
            ClientNotRunning => "The client is not currently running".into(),
            ClientError::ClientAlreadyRunning => "The client is already running".into(),
            ClientError::JreInstallError(t) => t.to_string(),
            ClientError::ModExtError(t) => t.to_string()
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
) -> Result<(), ClientError> {
    if process.lock().await.is_some() { return Err(ClientError::ClientAlreadyRunning); }

    let client_path = get_client(CLIENT_VERSION.to_string()).await.map_err(|e| NetworkError(e))?;

    println!("Launching Minecraft");
    let ms_auth: Option<MinecraftAuthentication> = persisted_data.read_value("ms_auth");

    let mut extensions: Vec<Extension> = persisted_data.read_value("extensions").unwrap_or(Vec::new());
    let java_dir = yakclient_dir().join("runtime");
    create_dir_all(&java_dir).map_err(IoError)?;

    let mods: Vec<Mod> = persisted_data.read_value("mods").unwrap_or(Vec::new());
    if !mods.is_empty() {
        let mod_ext = generate_mod_extension(
            mods,
            yakclient_dir().join("repo"),
            version.clone(),
        ).await.map_err(|e| ModExtError(e))?;

        extensions.push(mod_ext);
    }

    let child = launch_process(
        version,
        java_dir,
        client_path,
        &ms_auth,
        &extensions,
    ).await?;

    let child = capture_child(
        child,
        console_channel,
    );

    let instance = LaunchInstance {
        child,
    };

    *process.lock().await = Some(instance);

    Ok(())
}

#[tauri::command]
pub async fn end_launch_process(
    process: State<'_, Arc<Mutex<Option<LaunchInstance>>>>,
) -> Result<(), ClientError> {
    let mut guard = process.lock().await;

    println!("PROCESS IS: {}", guard.is_some());
    if let Some(process) = guard.deref() {
        process.shutdown().await
    } else {
        return Err(ClientNotRunning);
    }

    *guard = None;

    Ok(())
}
