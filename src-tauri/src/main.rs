// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::extensions::{get_extension_state, get_maven_local, set_extension_state};
use crate::launch::{end_launch_process, launch_minecraft};
use crate::mods::{get_mod_state, set_mod_state};
use crate::oauth::{get_mc_profile, microsoft_login, use_no_auth};
use crate::open_url::open_url;
use crate::persist::PersistedData;
use crate::state::{Extension, LaunchInstance, MinecraftAuthentication, OAuthConfig};
use crate::task::channel_progress::{register_task_channel, ChannelProgressBuilder, ChannelProgressManager};
use crate::task::TaskManager;
use discord_rich_presence::activity::Timestamps;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use log::debug;
use std::collections::HashMap;
use std::error::Error;
use std::ops::DerefMut;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use std::{env, io};
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager};
use tokio::sync::{Mutex, MutexGuard};

mod extensions;
mod launch;
mod mods;
mod oauth;
mod open_url;
mod persist;
mod state;
mod task;
mod util;

pub fn minecraft_dir() -> PathBuf {
    let path = if cfg!(target_os = "windows") {
        env::var("APPDATA")
            .map(|it| PathBuf::from(it))
            .unwrap_or_else(|_| {
                let home = home::home_dir().unwrap();
                home.join("AppData").join("Roaming")
            })
            .join(".minecraft")
    } else if cfg!(target_os = "macos") {
        home::home_dir()
            .unwrap()
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
    let discord_client = setup_discord_client();

    if let Err(ref e) = discord_client {
        debug!("Discord client failed to connect: {}", e.to_string());
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(OAuthConfig {
            client_id: "d64e5a9a-514f-482a-a8b4-967918739d9c".to_string(),
            response_type: "code".to_string(),
            scope: "XboxLive.signin%20offline_access".to_string(),
            tenant: "consumers".to_string(),
        })
        .manage(Arc::new(Mutex::new(None::<LaunchInstance>)))
        .manage(std::sync::Mutex::new(discord_client.ok()))
        .manage(
            PersistedData::read_from(yakclient_dir().join("config.json"))
                .expect("Unable to load config"),
        )
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
            get_mc_profile,
            get_maven_local,
            leave_splashscreen,
            register_task_channel
        ])
        .setup(|app| {
            let handle = app.handle().clone();

            let manager = ChannelProgressManager {
                channels: Mutex::new(HashMap::new()),
            };

            let manager = Arc::new(manager);
            let tasks = TaskManager {
                progress_builder: Box::new(ChannelProgressBuilder {
                    id: 0,
                    manager: Arc::clone(&manager),
                    handle,
                })
            };

            app.manage(Arc::clone(&manager));
            app.manage(Mutex::new(tasks));

            Ok(())
        })
        .on_window_event(|app_handle, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let persisted_data = app_handle.state::<PersistedData>();
                persisted_data
                    .persist_to(yakclient_dir().join("config.json"))
                    .expect("Failed to persist config");

                let mut discord_client =
                    app_handle.state::<std::sync::Mutex<Option<DiscordIpcClient>>>();

                if let Some(ref mut discord_client) = discord_client.lock().unwrap().deref_mut() {
                    discord_client.close().unwrap();
                };
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_discord_client() -> Result<DiscordIpcClient, Box<dyn Error>> {
    let mut discord_client = DiscordIpcClient::new("823623307567038534")?;

    discord_client.connect()?;
    launcher_status(&mut discord_client)?;

    Ok(discord_client)
}

pub fn launcher_status(discord_client: &mut DiscordIpcClient) -> Result<(), Box<dyn Error>> {
    discord_client.set_activity(
        activity::Activity::new()
            .state("In launcher")
            .details("Using YakClient Beta"),
    )
}

#[tauri::command]
fn leave_splashscreen(
    app: AppHandle
) {
    let splash_window = app.get_webview_window("splashscreen").unwrap();
    let main_window = app.get_webview_window("main").unwrap();
    splash_window.close().unwrap();
    main_window.show().unwrap();
}