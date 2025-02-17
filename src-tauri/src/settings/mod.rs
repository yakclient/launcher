use std::sync::Mutex;
use serde::{Deserialize, Serialize};
use tauri::State;
use crate::persist::PersistedData;

#[derive(Serialize,Deserialize,Clone)]
pub struct UserSettings {
    pub debugger: DebuggerSettings
}

#[derive(Serialize,Deserialize, Clone)]
pub struct DebuggerSettings {
    pub enabled: bool,
    pub suspend: bool,
    pub port: String
}

#[tauri::command]
pub fn get_settings(
    persisted_data: State<'_, PersistedData>
) -> UserSettings {
    return persisted_data.read_value("settings").unwrap()
}

#[tauri::command]
pub fn save_settings(
    settings: UserSettings,
    persisted_data: State<'_, PersistedData>
) {
    persisted_data.put_value("settings", settings);
}