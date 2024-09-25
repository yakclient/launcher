// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};

use tauri::Manager;

use crate::extension_state::{get_extension_state, set_extension_state};
use crate::launch::{end_launch_process, launch_minecraft};
use crate::oauth::{microsoft_login, use_no_auth};
use crate::state::{Extension, LaunchInstance, MinecraftAuthentication, OAuthConfig};

mod oauth;
mod state;
mod launch;
mod extension_state;

fn main() {
    tauri::Builder::default()
        .manage(OAuthConfig {
            client_id: "d64e5a9a-514f-482a-a8b4-967918739d9c",
            response_type: "code",
            scope: "XboxLive.signin%20offline_access",
            tenant: "consumers",
        })
        .manage(Arc::new(Mutex::new(None::<MinecraftAuthentication>)))
        .manage(Arc::new(Mutex::new(None::<LaunchInstance>)))
        .manage(Arc::new(Mutex::new(Vec::new() as Vec<Extension>)))
        .invoke_handler(tauri::generate_handler![
            microsoft_login,
            launch_minecraft,
            end_launch_process,
            set_extension_state,
            get_extension_state,
            use_no_auth
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}