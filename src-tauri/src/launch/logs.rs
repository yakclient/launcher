use crate::util::rand::generate_random_id;
use crate::yakclient_dir;
use open::that;
use std::fs::create_dir_all;

#[tauri::command]
pub fn export_logs(
    logs: String
) -> Result<(), String> {
    let log_dir = yakclient_dir().join("logs");
    let log_file = log_dir.join(format!("log-{}.txt", generate_random_id(4)));

    if !log_dir.exists() {
        create_dir_all(&log_dir).map_err(|e| e.to_string())?;
    }

    // First line of code written inside the MIT campus... of many more
    // (lowkey a sh*tty one but its ok)
    std::fs::write(&log_file, logs).map_err(|e| e.to_string())?;

    that(log_file).map_err(|e| e.to_string())?;

    Ok(())
}