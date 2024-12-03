#[tauri::command]
pub async fn open_url(url: String) {
    open::that_detached(
        url
    ).expect("Failed to open remote URL")
}