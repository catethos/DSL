use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

const UPDATE_SERVER: &str = "http://72.61.149.67:8000";
const BINARY_NAME: &str = "dsl";

/// Compare semantic versions (simple implementation)
fn is_newer_version(current: &str, remote: &str) -> bool {
    let current_parts: Vec<u32> = current.split('.').filter_map(|s| s.parse().ok()).collect();
    let remote_parts: Vec<u32> = remote.split('.').filter_map(|s| s.parse().ok()).collect();

    for i in 0..3 {
        let curr = current_parts.get(i).unwrap_or(&0);
        let rem = remote_parts.get(i).unwrap_or(&0);
        if rem > curr {
            return true;
        } else if rem < curr {
            return false;
        }
    }
    false
}

/// Check if a newer version is available
pub async fn check_for_update(
    current_version: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let url = format!("{}/VERSION", UPDATE_SERVER);
    let response = reqwest::get(&url).await?;

    if !response.status().is_success() {
        return Err("Failed to fetch version information".into());
    }

    let remote_version = response.text().await?.trim().to_string();

    if is_newer_version(current_version, &remote_version) {
        Ok(Some(remote_version))
    } else {
        Ok(None)
    }
}

/// Perform self-update
pub async fn self_update(current_version: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Checking for updates...");

    let new_version = match check_for_update(current_version).await? {
        Some(v) => v,
        None => {
            println!(
                "You are already running the latest version: {}",
                current_version
            );
            return Ok(());
        }
    };

    println!(
        "New version available: {} -> {}",
        current_version, new_version
    );
    println!("Downloading update...");

    // Detect architecture
    let arch = std::env::consts::ARCH;
    let arch_name = match arch {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        _ => return Err(format!("Unsupported architecture: {}", arch).into()),
    };

    let archive_name = format!(
        "{}-v{}-{}-macos.tar.gz",
        BINARY_NAME, new_version, arch_name
    );
    let download_url = format!("{}/{}", UPDATE_SERVER, archive_name);

    // Download the new binary
    let response = reqwest::get(&download_url).await?;
    if !response.status().is_success() {
        return Err(format!("Failed to download update: HTTP {}", response.status()).into());
    }

    let archive_data = response.bytes().await?;

    // Create temporary directory
    let temp_dir = tempfile::tempdir()?;
    let archive_path = temp_dir.path().join(&archive_name);
    fs::write(&archive_path, archive_data)?;

    // Extract the archive
    println!("Extracting...");
    let status = Command::new("tar")
        .args(&["xzf", &archive_name])
        .current_dir(temp_dir.path())
        .status()?;

    if !status.success() {
        return Err("Failed to extract archive".into());
    }

    // Get the path of the current executable
    let current_exe = std::env::current_exe()?;
    let new_binary = temp_dir.path().join(BINARY_NAME);

    // Check if we need sudo
    let parent_dir = current_exe
        .parent()
        .ok_or("Failed to get parent directory")?;
    let needs_sudo = !is_writable(parent_dir);

    println!("Installing update...");

    if needs_sudo {
        println!("This operation requires administrator privileges.");
        let status = Command::new("sudo")
            .args(&["mv", "-f"])
            .arg(&new_binary)
            .arg(&current_exe)
            .status()?;

        if !status.success() {
            return Err("Failed to install update (permission denied)".into());
        }
    } else {
        fs::rename(&new_binary, &current_exe)?;
    }

    // Ensure executable permissions
    let metadata = fs::metadata(&current_exe)?;
    let mut permissions = metadata.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&current_exe, permissions)?;

    println!("✓ Successfully updated to version {}", new_version);
    println!("Please restart the application.");

    Ok(())
}

/// Check if a directory is writable
fn is_writable(path: &std::path::Path) -> bool {
    // Try to create a temporary file
    let test_file = path.join(format!(".write_test_{}", std::process::id()));
    match fs::write(&test_file, b"test") {
        Ok(_) => {
            let _ = fs::remove_file(&test_file);
            true
        }
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(is_newer_version("0.1.0", "0.2.0"));
        assert!(is_newer_version("0.1.0", "1.0.0"));
        assert!(is_newer_version("1.2.3", "1.2.4"));
        assert!(!is_newer_version("1.0.0", "0.9.0"));
        assert!(!is_newer_version("1.0.0", "1.0.0"));
    }
}
