use tauri::State;
use crate::state::{Extension, ExtensionState};

#[tauri::command]
pub async fn set_extension_state(
    updated: Vec<Extension>,
    previous: State<'_, ExtensionState>
) -> Result<(), ()> {

    *previous.lock().unwrap() = updated;

    Ok(())
}

#[tauri::command]
pub async fn get_extension_state(
    extensions: State<'_, ExtensionState>
) -> Result<Vec<Extension>, ()> {
    Ok(extensions.lock().unwrap().clone())
}