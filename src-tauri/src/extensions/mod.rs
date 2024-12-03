pub mod types;

use tauri::State;
use crate::persist::PersistedData;
use crate::state::{Extension, Mod};

#[tauri::command]
pub async fn set_extension_state(
    updated: Vec<Extension>,
    persisted_data: State<'_, PersistedData>,

) -> Result<(), ()> {
    persisted_data.put_value("extensions", updated);

    Ok(())
}

#[tauri::command]
pub async fn get_extension_state(
    persisted_data: State<'_, PersistedData>,
) -> Result<Vec<Extension>, ()> {
    println!("Getting extensions");
    Ok(persisted_data.read_value("extensions").unwrap_or(Vec::new()))
}

