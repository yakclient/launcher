use std::fmt::{Display, Formatter};
use std::io;
use std::io::{Read, Write};
use std::ops::Deref;
use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard};

use serde::{Serialize, Serializer};
use tauri::{AppHandle, State};
use tokio::io::AsyncReadExt;

use crate::launch::client::get_client;
use crate::launch::ClientError::{ClientNotRunning, ClientProcessError, IoError, NetworkError, Unauthenticated};
use crate::launch::process::{capture_child, launch_process};
use crate::state::{LaunchInstance, MinecraftAuthentication};

mod client;
mod output;
mod process;

const CLIENT_VERSION: &'static str = "1.0-SNAPSHOT";

#[derive(Debug)]
pub enum ClientError {
    NetworkError(reqwest::Error),
    IoError(io::Error),
    ClientProcessError(String),
    Unauthenticated,
    ClientNotRunning,
    ClientAlreadyRunning,
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
        };
        write!(f, "{}", str)
    }
}

#[tauri::command]
pub async fn launch_minecraft(
    version: String,
    mc_creds: State<'_, Arc<RwLock<Option<MinecraftAuthentication>>>>,
    app_handle: AppHandle,
    process: State<'_, RwLock<Option<LaunchInstance>>>,
) -> Result<(), ClientError> {
    if process.read().unwrap().is_some() { return Err(ClientError::ClientAlreadyRunning); }

    let client_path = get_client(CLIENT_VERSION.to_string()).await.map_err(|e| NetworkError(e))?;

    println!("Launching Minecraft");
    // Fucked up rust syntax
    let cred_lock = mc_creds.read().unwrap();
    let result = if let Some(credentials) = cred_lock.deref() {
        credentials
    } else {
        return Err(Unauthenticated);
    };

    let child = launch_process(version, client_path, result)?;

    let child = capture_child(child, app_handle);

    let instance = LaunchInstance {
        child,
    };

    *process.write().unwrap() = Some(instance);

    Ok(())
}

#[tauri::command]
pub async fn end_launch_process(
    process: State<'_, RwLock<Option<LaunchInstance>>>,
) -> Result<(), ClientError> {
    println!("PROCESS IS: {}", process.read().unwrap().is_some());
    if let Some(process) = process.read().unwrap().deref() {
        process.shutdown()
    } else {
        return Err(ClientNotRunning)
    }

    *process.write().unwrap() = None;

    Ok(())
}