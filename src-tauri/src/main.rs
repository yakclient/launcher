// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{env, io};
use std::path::PathBuf;
use std::sync::Arc;

use crate::extensions::{get_extension_state, set_extension_state};
use crate::launch::{end_launch_process, launch_minecraft};
use crate::mods::{get_mod_state, set_mod_state};
use crate::oauth::{get_mc_profile, microsoft_login, use_no_auth};
use crate::persist::PersistedData;
use crate::state::{Extension, LaunchInstance, MinecraftAuthentication, OAuthConfig};
use tauri::Manager;
use tokio::sync::{Mutex, MutexGuard};
use crate::open_url::open_url;

mod oauth;
mod state;
mod launch;
mod extensions;
mod persist;
mod mods;
mod open_url;

pub fn minecraft_dir() -> PathBuf {
    let path = if cfg!(target_os = "windows") {
        env::var("APPDATA")
            .map(|it| PathBuf::from(it))
            .unwrap_or_else(|_| {
                let home = home::home_dir().unwrap();
                home.join("AppData").join("Roaming")
            }).join(".minecraft")
    } else if cfg!(target_os = "macos") {
        home::home_dir().unwrap()
            .join("Library")
            .join("Application Support")
            .join("minecraft")
    } else {
        home::home_dir().unwrap().join(".minecraft")
    };

    path
}

pub fn extframework_dir() -> PathBuf {
    let path = minecraft_dir();

    path.join(".extframework")
}

pub fn yakclient_dir() -> PathBuf {
    extframework_dir().join("yakclient")
}

fn main() {
    tauri::Builder::default()
        .manage(OAuthConfig {
            client_id: "d64e5a9a-514f-482a-a8b4-967918739d9c".to_string(),
            response_type: "code".to_string(),
            scope: "XboxLive.signin%20offline_access".to_string(),
            tenant: "consumers".to_string(),
        })
        // .manage(Arc::new(Mutex::new(None::<MinecraftAuthentication>)))
        .manage(Arc::new(Mutex::new(None::<LaunchInstance>)))
        // .manage(Arc::new(Mutex::new(Vec::new() as Vec<Extension>)))
        .manage(PersistedData::read_from(yakclient_dir().join("config.json")).expect("Unable to load config"))
        .invoke_handler(tauri::generate_handler![
            microsoft_login,
            launch_minecraft,
            end_launch_process,
            set_extension_state,
            get_extension_state,
            set_mod_state,
            get_mod_state,
            use_no_auth,
            open_url,
            get_mc_profile
        ])
        .on_window_event(|app_handle, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let state = app_handle.state::<PersistedData>();
                state.persist_to(yakclient_dir().join("config.json")).expect("Failed to persist config");
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}