import {invoke} from "@tauri-apps/api/core";
import {set} from "immutable";

export interface UserSettings {
    debugger: {
        enabled: boolean,
        suspend: boolean,
        port: string
    }
}

export const loadSettings = async () => {
    return await invoke<UserSettings>("get_settings")
}

export const saveSettings = async (settings: UserSettings) => {
    return await invoke("save_settings", {
        settings: settings
    })
}