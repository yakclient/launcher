pub mod types;

use crate::persist::PersistedData;
use crate::state::{Extension, Mod};
use std::path::PathBuf;
use tauri::State;

#[tauri::command]
pub async fn set_extension_state(
    updated: Vec<Extension>,
    persisted_data: State<'_, PersistedData>,
) -> Result<(), ()> {
    println!("set extension state, {:?}", updated);
    persisted_data.put_value("extensions", updated);

    Ok(())
}

#[tauri::command]
pub async fn get_extension_state(
    persisted_data: State<'_, PersistedData>,
) -> Result<Vec<Extension>, ()> {
    println!("Getting extensions");
    Ok(persisted_data
        .read_value("extensions")
        .unwrap_or(Vec::new()))
}

#[tauri::command]
pub async fn get_maven_local() -> String {
    home::home_dir()
        .unwrap()
        .join(".m2")
        .join("repository")
        .to_str()
        .unwrap()
        .to_string()
}
