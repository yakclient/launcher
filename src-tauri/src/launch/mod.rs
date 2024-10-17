use std::fmt::{Display, Formatter};
use std::io;
use std::io::{Read, Write};
use std::ops::Deref;
use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard};

use serde::{Serialize, Serializer};
use tauri::{AppHandle, State};
use tauri::ipc::Channel;
use tokio::io::AsyncReadExt;
use zip_extract::ZipExtractError;

use crate::launch::client::get_client;
use crate::launch::ClientError::{ClientNotRunning, ClientProcessError, IoError, NetworkError, Unauthenticated};
use crate::launch::process::{capture_child, launch_process, ProcessStdoutEvent};
use crate::state::{ExtensionState, LaunchInstance, MinecraftAuthentication};

mod client;
mod process;
mod java;

const CLIENT_VERSION: &'static str = "1.0.1-BETA";

#[derive(Debug)]
pub enum ClientError {
    NetworkError(reqwest::Error),
    IoError(io::Error),
    ClientProcessError(String),
    Unauthenticated,
    ClientNotRunning,
    ClientAlreadyRunning,
    ZipExtractError(ZipExtractError),
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
            ClientError::ClientProcessError(s) => { s.clone() }
            Unauthenticated => { "You are not authenticated! Please login first.".into() }
            ClientNotRunning => "The client is not currently running".into(),
            ClientError::ClientAlreadyRunning => "The client is already running".into(),
            ClientError::ZipExtractError(t) => t.to_string()
        };
        write!(f, "{}", str)
    }
}

#[tauri::command]
pub async fn launch_minecraft(
    version: String,
    mc_creds: State<'_, Arc<Mutex<Option<MinecraftAuthentication>>>>,
    process: State<'_, Arc<Mutex<Option<LaunchInstance>>>>,
    extensions: State<'_, ExtensionState>,
    console_channel: Channel<ProcessStdoutEvent>,
) -> Result<(), ClientError> {
    if process.lock().unwrap().is_some() { return Err(ClientError::ClientAlreadyRunning); }

    let client_path = get_client(CLIENT_VERSION.to_string()).await.map_err(|e| NetworkError(e))?;

    println!("Launching Minecraft");
    let cred_lock = mc_creds.lock().unwrap();
    let result = cred_lock.deref();

    let child = launch_process(
        version,
        client::extframework_dir().join("yakclient"),
        client_path,
        result,
        &*extensions.lock().unwrap(),
    )?;

    let child = capture_child(
        child,
        console_channel,
    );

    let instance = LaunchInstance {
        child,
    };

    *process.lock().unwrap() = Some(instance);

    Ok(())
}

#[tauri::command]
pub async fn end_launch_process(
    process: State<'_, Arc<Mutex<Option<LaunchInstance>>>>,
) -> Result<(), ClientError> {
    let mut guard = process.lock().unwrap();

    println!("PROCESS IS: {}", guard.is_some());
    if let Some(process) = guard.deref() {
        process.shutdown()
    } else {
        return Err(ClientNotRunning);
    }

    *guard = None;

    Ok(())
}
