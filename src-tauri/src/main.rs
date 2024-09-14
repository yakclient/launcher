// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod oauth;
mod state;
mod launch;

use std::sync::{Arc, RwLock};
use tauri::Manager;
use crate::launch::{end_launch_process, launch_minecraft};
use crate::oauth::microsoft_login;
use crate::state::{LaunchInstance, MinecraftAuthentication, OAuthConfig};

fn main() {
    tauri::Builder::default()
        .manage(OAuthConfig {
            client_id: "d64e5a9a-514f-482a-a8b4-967918739d9c",
            response_type: "code",
            scope: "XboxLive.signin%20offline_access",
            tenant: "consumers",
        })
        .manage(Arc::new(RwLock::new(None::<MinecraftAuthentication>)))
        .manage(RwLock::new(None::<LaunchInstance>))
        .invoke_handler(tauri::generate_handler![microsoft_login, launch_minecraft, end_launch_process])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}