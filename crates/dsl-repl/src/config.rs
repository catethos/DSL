use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Get the configuration directory
fn get_config_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config").join("dsl")
}

/// Get the path to the last update check file
fn get_last_check_file() -> PathBuf {
    get_config_dir().join(".last_update_check")
}

/// Check if we should check for updates (once per day)
pub fn should_check_for_updates() -> bool {
    let check_file = get_last_check_file();

    if !check_file.exists() {
        return true;
    }

    let Ok(contents) = fs::read_to_string(&check_file) else {
        return true;
    };

    let Ok(last_check) = contents.trim().parse::<u64>() else {
        return true;
    };

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Check once per day (86400 seconds)
    now - last_check > 86400
}

/// Record that we've checked for updates
pub fn record_update_check() {
    let check_file = get_last_check_file();

    // Create config directory if it doesn't exist
    if let Some(parent) = check_file.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let _ = fs::write(&check_file, now.to_string());
}
