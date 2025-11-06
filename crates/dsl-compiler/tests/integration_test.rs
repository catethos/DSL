use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn get_binary_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.push("target");
    path.push("debug");
    path.push("dsl-compiler");
    path
}

fn create_temp_dsl_file(content: &str) -> tempfile::NamedTempFile {
    let mut file = tempfile::NamedTempFile::new().unwrap();
    use std::io::Write;
    file.write_all(content.as_bytes()).unwrap();
    file.flush().unwrap();
    file
}

#[test]
fn test_check_valid_program() {
    let temp_file = create_temp_dsl_file("40 + 2");
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .args(&["check", temp_file.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("✓ No errors found"));
}

#[test]
fn test_check_invalid_program() {
    // Use syntax that's definitely invalid
    let temp_file = create_temp_dsl_file("@@@invalid");
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .args(&["check", temp_file.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success(), "Expected check to fail for invalid syntax");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Compilation failed") || stderr.contains("Parse error"));
}

#[test]
fn test_ir_json_output() {
    let temp_file = create_temp_dsl_file("40 + 2");
    let output_file = tempfile::NamedTempFile::new().unwrap();
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .args(&[
            "ir",
            temp_file.path().to_str().unwrap(),
            "-o",
            output_file.path().to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // Verify JSON file exists and is valid
    let ir_content = fs::read_to_string(output_file.path()).unwrap();
    let ir_json: serde_json::Value = serde_json::from_str(&ir_content).unwrap();

    assert_eq!(ir_json["version"], "0.1.0");
    assert!(ir_json["entry_expr"].is_object());
}

#[test]
fn test_ir_msgpack_output() {
    let temp_file = create_temp_dsl_file("1 + 1");
    let output_file = tempfile::NamedTempFile::new().unwrap();
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .args(&[
            "ir",
            temp_file.path().to_str().unwrap(),
            "-o",
            output_file.path().to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // Verify MessagePack file exists and has content
    let ir_bytes = fs::read(output_file.path()).unwrap();
    assert!(!ir_bytes.is_empty());
}

#[test]
fn test_help_command() {
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .arg("--help")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Compiler for DSL programs"));
    assert!(stdout.contains("build"));
    assert!(stdout.contains("check"));
    assert!(stdout.contains("ir"));
}
