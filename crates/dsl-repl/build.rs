use std::fs;
use std::path::PathBuf;

fn main() {
    // Read VERSION file from workspace root
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("Failed to find workspace root");

    let version_file = workspace_root.join("VERSION");

    if version_file.exists() {
        let version = fs::read_to_string(&version_file)
            .expect("Failed to read VERSION file")
            .trim()
            .to_string();

        println!("cargo:rustc-env=DSL_VERSION={}", version);
        println!("cargo:rerun-if-changed={}", version_file.display());
    } else {
        // Fallback to Cargo.toml version if VERSION file doesn't exist
        println!("cargo:rustc-env=DSL_VERSION={}", env!("CARGO_PKG_VERSION"));
    }
}
