//! Import resolution for Lattice
//!
//! This module implements a preprocessing approach to imports. Import statements
//! are resolved before parsing by replacing them with the contents of the imported files.
//!
//! This approach requires NO changes to the AST, compiler, or VM.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::error::LatticeError;
use regex::Regex;

/// Resolve imports in source code.
///
/// This function finds all `import "path"` statements and replaces them with
/// the contents of the imported files. It handles:
/// - Relative paths (resolved relative to the importing file)
/// - Circular import detection
/// - Recursive imports (imported files can have their own imports)
/// - Deduplication (same file imported from multiple places is included only once)
///
/// # Arguments
///
/// * `source` - The source code to process
/// * `base_path` - The directory to resolve relative imports from
///
/// # Returns
///
/// The source code with all imports resolved (import statements replaced with file contents).
///
/// # Errors
///
/// Returns an error if:
/// - An imported file cannot be read
/// - A circular import is detected (A imports B, B imports A)
/// - The import path is invalid
pub fn resolve_imports(source: &str, base_path: &Path) -> Result<String, LatticeError> {
    let mut imported = HashSet::new();  // Files fully processed
    let mut stack = HashSet::new();     // Files currently being processed (for circular detection)
    resolve_imports_inner(source, base_path, &mut imported, &mut stack)
}

/// Internal recursive import resolution.
///
/// Two sets are used:
/// - `imported`: Files that have been fully processed. Used to skip duplicate imports.
/// - `stack`: Files currently being processed in the call stack. Used to detect circular imports.
///
/// A file in `stack` but not in `imported` indicates we're in the middle of processing it,
/// so importing it again would be circular.
fn resolve_imports_inner(
    source: &str,
    base_path: &Path,
    imported: &mut HashSet<PathBuf>,
    stack: &mut HashSet<PathBuf>,
) -> Result<String, LatticeError> {
    // Regex to match import statements: import "path/to/file.lat"
    // Matches: import followed by a string literal (with optional whitespace)
    let import_re = Regex::new(r#"import\s+"([^"]+)""#)
        .map_err(|e| LatticeError::Runtime(format!("Regex error: {}", e)))?;

    let mut result = String::new();
    let mut last_end = 0;

    for cap in import_re.captures_iter(source) {
        let full_match = cap.get(0).unwrap();
        let import_path = cap.get(1).unwrap().as_str();

        // Add text before this import
        result.push_str(&source[last_end..full_match.start()]);

        // Resolve the import path
        let resolved_path = base_path.join(import_path);
        let canonical_path = resolved_path.canonicalize().map_err(|e| {
            LatticeError::Parse(format!(
                "Cannot resolve import '{}': {}",
                import_path, e
            ))
        })?;

        // Check for circular imports (file is in the current call stack)
        if stack.contains(&canonical_path) {
            return Err(LatticeError::Parse(format!(
                "Circular import detected: {}",
                canonical_path.display()
            )));
        }

        // Skip files that have already been fully imported (prevents duplicate definitions)
        // This allows the same file to be imported from multiple places
        if imported.contains(&canonical_path) {
            // Add a comment noting this was skipped
            result.push_str(&format!("// import \"{}\" (already imported)\n", import_path));
            last_end = full_match.end();
            continue;
        }

        // Add to stack BEFORE processing (for circular detection)
        stack.insert(canonical_path.clone());

        // Read the imported file
        let imported_source = std::fs::read_to_string(&canonical_path).map_err(|e| {
            LatticeError::Parse(format!(
                "Cannot read imported file '{}': {}",
                canonical_path.display(),
                e
            ))
        })?;

        // Recursively resolve imports in the imported file
        let imported_base = canonical_path.parent().unwrap_or(Path::new("."));
        let resolved_import = resolve_imports_inner(&imported_source, imported_base, imported, stack)?;

        // Remove from stack and add to imported (fully processed)
        stack.remove(&canonical_path);
        imported.insert(canonical_path.clone());

        // Add a comment marking the import source for debugging
        result.push_str(&format!("// BEGIN import \"{}\"\n", import_path));
        result.push_str(&resolved_import);
        result.push_str(&format!("\n// END import \"{}\"\n", import_path));

        last_end = full_match.end();
    }

    // Add remaining text after last import
    result.push_str(&source[last_end..]);

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn test_no_imports() {
        let source = "let x = 1\nlet y = 2";
        let result = resolve_imports(source, Path::new(".")).unwrap();
        assert_eq!(result, source);
    }

    #[test]
    fn test_simple_import() {
        let dir = TempDir::new().unwrap();
        let dir_path = dir.path();

        // Create a file to import
        create_test_file(dir_path, "types.lat", "type Foo { x: Int }");

        // Source with import
        let source = r#"import "types.lat"
let foo = Foo { x: 1 }"#;

        let result = resolve_imports(source, dir_path).unwrap();

        assert!(result.contains("type Foo { x: Int }"));
        assert!(result.contains("let foo = Foo { x: 1 }"));
        assert!(result.contains("// BEGIN import \"types.lat\""));
        assert!(result.contains("// END import \"types.lat\""));
    }

    #[test]
    fn test_nested_imports() {
        let dir = TempDir::new().unwrap();
        let dir_path = dir.path();

        // Create files
        create_test_file(dir_path, "base.lat", "type Base { id: Int }");
        create_test_file(
            dir_path,
            "derived.lat",
            r#"import "base.lat"
type Derived { base: Base }"#,
        );

        // Main source
        let source = r#"import "derived.lat"
let d = Derived { base: Base { id: 1 } }"#;

        let result = resolve_imports(source, dir_path).unwrap();

        assert!(result.contains("type Base { id: Int }"));
        assert!(result.contains("type Derived { base: Base }"));
        assert!(result.contains("let d = Derived"));
    }

    #[test]
    fn test_circular_import_detection() {
        let dir = TempDir::new().unwrap();
        let dir_path = dir.path();

        // Create circular imports
        create_test_file(dir_path, "a.lat", r#"import "b.lat""#);
        create_test_file(dir_path, "b.lat", r#"import "a.lat""#);

        let source = r#"import "a.lat""#;
        let result = resolve_imports(source, dir_path);

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Circular import"));
    }

    #[test]
    fn test_multiple_imports() {
        let dir = TempDir::new().unwrap();
        let dir_path = dir.path();

        create_test_file(dir_path, "types.lat", "type Person { name: String }");
        create_test_file(dir_path, "funcs.lat", "def greet(p: Person) -> String { p.name }");

        let source = r#"import "types.lat"
import "funcs.lat"
greet(Person { name: "Alice" })"#;

        let result = resolve_imports(source, dir_path).unwrap();

        assert!(result.contains("type Person"));
        assert!(result.contains("def greet"));
        assert!(result.contains("greet(Person"));
    }

    #[test]
    fn test_import_in_subdirectory() {
        let dir = TempDir::new().unwrap();
        let dir_path = dir.path();

        // Create subdirectory
        let subdir = dir_path.join("lib");
        fs::create_dir(&subdir).unwrap();

        create_test_file(&subdir, "utils.lat", "def helper() -> Int { 42 }");

        let source = r#"import "lib/utils.lat"
helper()"#;

        let result = resolve_imports(source, dir_path).unwrap();

        assert!(result.contains("def helper"));
        assert!(result.contains("helper()"));
    }

    #[test]
    fn test_import_file_not_found() {
        let dir = TempDir::new().unwrap();
        let source = r#"import "nonexistent.lat""#;

        let result = resolve_imports(source, dir.path());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Cannot resolve import"));
    }

    #[test]
    fn test_same_file_imported_twice_from_different_files() {
        let dir = TempDir::new().unwrap();
        let dir_path = dir.path();

        // Create shared types
        create_test_file(dir_path, "shared.lat", "type Shared { x: Int }");

        // Two files that both import shared
        create_test_file(
            dir_path,
            "a.lat",
            r#"import "shared.lat"
type A { s: Shared }"#,
        );
        create_test_file(
            dir_path,
            "b.lat",
            r#"import "shared.lat"
type B { s: Shared }"#,
        );

        // Main imports both a and b
        let source = r#"import "a.lat"
import "b.lat""#;

        let result = resolve_imports(source, dir_path).unwrap();

        // shared.lat should only be included once (the first time, through a.lat)
        // The second import from b.lat should be skipped
        assert!(result.contains("type Shared { x: Int }"));
        assert!(result.contains("type A { s: Shared }"));
        assert!(result.contains("type B { s: Shared }"));

        // Count occurrences of the shared type - should appear exactly once
        let shared_count = result.matches("type Shared { x: Int }").count();
        assert_eq!(shared_count, 1, "Shared should be included exactly once");

        // Should have a comment indicating b.lat's import was skipped
        assert!(result.contains("already imported"));
    }

    #[test]
    fn test_import_preserves_surrounding_code() {
        let dir = TempDir::new().unwrap();
        let dir_path = dir.path();

        create_test_file(dir_path, "types.lat", "type T { x: Int }");

        let source = r#"// Header comment
let before = 1
import "types.lat"
let after = 2
// Footer"#;

        let result = resolve_imports(source, dir_path).unwrap();

        assert!(result.contains("// Header comment"));
        assert!(result.contains("let before = 1"));
        assert!(result.contains("type T { x: Int }"));
        assert!(result.contains("let after = 2"));
        assert!(result.contains("// Footer"));
    }
}
