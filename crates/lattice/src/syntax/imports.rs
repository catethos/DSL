//! Import resolution for Lattice
//!
//! This module implements a preprocessing approach to imports. Import statements
//! are resolved before parsing by replacing them with the contents of the imported files.
//!
//! This approach requires NO changes to the AST, compiler, or VM.
//!
//! ## Supported File Types
//!
//! - `.lat` files: Standard Lattice source files (recursively resolved)
//! - `.md` files: Markdown LLM function definitions (transpiled to Lattice source)
//!
//! ## Import Variants
//!
//! ### Standard Imports
//! ```lattice
//! import "path/to/file.lat"  // Import all definitions
//! ```
//!
//! ### Namespaced Imports
//! ```lattice
//! import "math.lat" as math
//! let result = math.add(1, 2)  // Calls math_add internally
//! ```
//! Implemented via source-level transformation - all definitions prefixed with alias.
//!
//! ### Selective Imports
//! ```lattice
//! from "math.lat" import add, Vector
//! let result = add(1, 2)  // Direct usage without prefix
//! ```
//! Only the specified definitions are imported.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::error::LatticeError;
use crate::syntax::markdown::parse_markdown_llm;
use regex::Regex;

/// Represents the kind of import statement
#[derive(Debug, Clone)]
enum ImportKind {
    /// import "path" - import all definitions
    All,
    /// import "path" as alias - import all with namespace prefix
    Namespaced(String),
    /// from "path" import name1, name2 - import only specific names
    Selective(Vec<String>),
}

/// Resolve imports in source code.
///
/// This function finds all `import "path"` statements and replaces them with
/// the contents of the imported files. It handles:
/// - Relative paths (resolved relative to the importing file)
/// - Circular import detection
/// - Recursive imports (imported files can have their own imports)
/// - Deduplication (same file imported from multiple places is included only once)
/// - Markdown LLM files (.md) are transpiled to Lattice source
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
/// - A markdown file has invalid frontmatter
pub fn resolve_imports(source: &str, base_path: &Path) -> Result<String, LatticeError> {
    let mut imported = HashSet::new();  // Files fully processed
    let mut stack = HashSet::new();     // Files currently being processed (for circular detection)
    resolve_imports_inner(source, base_path, &mut imported, &mut stack)
}

/// Parsed import statement with all relevant information
struct ParsedImport {
    start: usize,
    end: usize,
    path: String,
    kind: ImportKind,
}

/// Parse all import statements from source code.
/// Returns a list of parsed imports in order of appearance.
fn parse_imports(source: &str) -> Result<Vec<ParsedImport>, LatticeError> {
    // Regex for "from" style imports: from "path" import name1, name2, ...
    let from_re = Regex::new(r#"from\s+"([^"]+)"\s+import\s+([a-zA-Z_][a-zA-Z0-9_]*(?:\s*,\s*[a-zA-Z_][a-zA-Z0-9_]*)*)"#)
        .map_err(|e| LatticeError::Runtime(format!("Regex error: {}", e)))?;

    // Regex for regular imports: import "path" or import "path" as alias
    let import_re = Regex::new(r#"import\s+"([^"]+)"(?:\s+as\s+([a-zA-Z_][a-zA-Z0-9_]*))?"#)
        .map_err(|e| LatticeError::Runtime(format!("Regex error: {}", e)))?;

    let mut imports = Vec::new();

    // First, find all "from" imports
    for cap in from_re.captures_iter(source) {
        let full_match = cap.get(0).unwrap();
        let path = cap.get(1).unwrap().as_str().to_string();
        let names_str = cap.get(2).unwrap().as_str();

        // Parse the comma-separated names
        let names: Vec<String> = names_str
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        imports.push(ParsedImport {
            start: full_match.start(),
            end: full_match.end(),
            path,
            kind: ImportKind::Selective(names),
        });
    }

    // Then, find regular imports (but skip any that overlap with "from" imports)
    for cap in import_re.captures_iter(source) {
        let full_match = cap.get(0).unwrap();
        let start = full_match.start();
        let end = full_match.end();

        // Skip if this overlaps with an existing import (shouldn't happen, but be safe)
        if imports.iter().any(|i| start < i.end && end > i.start) {
            continue;
        }

        let path = cap.get(1).unwrap().as_str().to_string();
        let alias = cap.get(2).map(|m| m.as_str().to_string());

        let kind = match alias {
            Some(a) => ImportKind::Namespaced(a),
            None => ImportKind::All,
        };

        imports.push(ParsedImport {
            start,
            end,
            path,
            kind,
        });
    }

    // Sort by position in source
    imports.sort_by_key(|i| i.start);

    Ok(imports)
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
    let imports = parse_imports(source)?;

    let mut result = String::new();
    let mut last_end = 0;
    let mut aliases: Vec<String> = Vec::new(); // Track aliases for qualified name resolution

    for import in imports {
        // Add text before this import, transforming any qualified names
        let text_before = &source[last_end..import.start];
        let transformed_before = transform_qualified_names(text_before, &aliases)?;
        result.push_str(&transformed_before);

        // Resolve the import path
        let resolved_path = base_path.join(&import.path);
        let canonical_path = resolved_path.canonicalize().map_err(|e| {
            LatticeError::Parse(format!(
                "Cannot resolve import '{}': {}",
                import.path, e
            ))
        })?;

        // Check for circular imports (file is in the current call stack)
        if stack.contains(&canonical_path) {
            return Err(LatticeError::Parse(format!(
                "Circular import detected: {}",
                canonical_path.display()
            )));
        }

        // Generate import description for comments
        let import_desc = match &import.kind {
            ImportKind::All => format!("import \"{}\"", import.path),
            ImportKind::Namespaced(a) => format!("import \"{}\" as {}", import.path, a),
            ImportKind::Selective(names) => format!("from \"{}\" import {}", import.path, names.join(", ")),
        };

        // For selective imports, we can import the same file multiple times with different symbols
        // But for All/Namespaced, skip if already imported
        let skip_if_imported = !matches!(import.kind, ImportKind::Selective(_));

        if skip_if_imported && imported.contains(&canonical_path) {
            // Add a comment noting this was skipped
            result.push_str(&format!("// {} (already imported)\n", import_desc));
            last_end = import.end;
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

        // Handle based on file extension
        let mut resolved_import = if import.path.ends_with(".md") {
            // Markdown LLM file: transpile to Lattice source
            let md_def = parse_markdown_llm(&imported_source).map_err(|e| {
                LatticeError::Parse(format!(
                    "Error parsing markdown file '{}': {}",
                    import.path, e
                ))
            })?;
            md_def.to_lattice_source()
        } else {
            // Regular .lat file: recursively resolve imports
            let imported_base = canonical_path.parent().unwrap_or(Path::new("."));
            resolve_imports_inner(&imported_source, imported_base, imported, stack)?
        };

        // Apply transformations based on import kind
        resolved_import = match &import.kind {
            ImportKind::All => resolved_import,
            ImportKind::Namespaced(alias) => {
                aliases.push(alias.clone());
                prefix_definitions(&resolved_import, alias)?
            }
            ImportKind::Selective(names) => {
                extract_definitions(&resolved_import, names, &import.path)?
            }
        };

        // Remove from stack and add to imported (fully processed) for non-selective
        stack.remove(&canonical_path);
        if !matches!(import.kind, ImportKind::Selective(_)) {
            imported.insert(canonical_path.clone());
        }

        // Add a comment marking the import source for debugging
        result.push_str(&format!("// BEGIN {}\n", import_desc));
        result.push_str(&resolved_import);
        result.push_str(&format!("\n// END {}\n", import_desc));

        last_end = import.end;
    }

    // Add remaining text after last import
    let remaining = &source[last_end..];

    // Transform qualified name references (alias.name -> alias_name) in the remaining source
    let transformed_remaining = transform_qualified_names(remaining, &aliases)?;
    result.push_str(&transformed_remaining);

    Ok(result)
}

/// Prefix all top-level definitions in the source with the given alias.
///
/// This transforms:
/// - `def foo(...)` → `def alias_foo(...)`
/// - `type Foo { ... }` → `type alias_Foo { ... }`
/// - `enum Foo { ... }` → `enum alias_Foo { ... }`
/// - `llm_config foo { ... }` → `llm_config alias_foo { ... }`
fn prefix_definitions(source: &str, alias: &str) -> Result<String, LatticeError> {
    let mut result = source.to_string();

    // Prefix function definitions: def name( -> def alias_name(
    let def_re = Regex::new(r"\bdef\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(")
        .map_err(|e| LatticeError::Runtime(format!("Regex error: {}", e)))?;
    result = def_re.replace_all(&result, |caps: &regex::Captures| {
        format!("def {}_{}(", alias, &caps[1])
    }).to_string();

    // Prefix type definitions: type Name { -> type alias_Name {
    let type_re = Regex::new(r"\btype\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\{")
        .map_err(|e| LatticeError::Runtime(format!("Regex error: {}", e)))?;
    result = type_re.replace_all(&result, |caps: &regex::Captures| {
        format!("type {}_{} {{", alias, &caps[1])
    }).to_string();

    // Prefix enum definitions: enum Name { -> enum alias_Name {
    let enum_re = Regex::new(r"\benum\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\{")
        .map_err(|e| LatticeError::Runtime(format!("Regex error: {}", e)))?;
    result = enum_re.replace_all(&result, |caps: &regex::Captures| {
        format!("enum {}_{} {{", alias, &caps[1])
    }).to_string();

    // Prefix llm_config definitions: llm_config name { -> llm_config alias_name {
    let config_re = Regex::new(r"\bllm_config\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\{")
        .map_err(|e| LatticeError::Runtime(format!("Regex error: {}", e)))?;
    result = config_re.replace_all(&result, |caps: &regex::Captures| {
        format!("llm_config {}_{} {{", alias, &caps[1])
    }).to_string();

    Ok(result)
}

/// Extract only the specified definitions from the source.
///
/// This is used for selective imports: `from "file.lat" import name1, name2`
/// Only the definitions matching the requested names are included in the output.
///
/// Supports:
/// - `def name(...)` - function definitions
/// - `type Name { ... }` - type definitions
/// - `enum Name { ... }` - enum definitions
/// - `llm_config name { ... }` - LLM config definitions
fn extract_definitions(source: &str, names: &[String], import_path: &str) -> Result<String, LatticeError> {
    let mut extracted = Vec::new();
    let mut found_names = HashSet::new();

    // Regex patterns for each definition type
    // These capture the entire definition including the body
    // We use a simple approach: find the start, then match braces/parens to find the end

    for name in names {
        // Try each definition type
        if let Some(def) = extract_function(source, name)? {
            extracted.push(def);
            found_names.insert(name.clone());
        } else if let Some(def) = extract_type(source, name)? {
            extracted.push(def);
            found_names.insert(name.clone());
        } else if let Some(def) = extract_enum(source, name)? {
            extracted.push(def);
            found_names.insert(name.clone());
        } else if let Some(def) = extract_llm_config(source, name)? {
            extracted.push(def);
            found_names.insert(name.clone());
        }
    }

    // Check for any names that weren't found
    let missing: Vec<_> = names.iter()
        .filter(|n| !found_names.contains(*n))
        .collect();

    if !missing.is_empty() {
        return Err(LatticeError::Parse(format!(
            "Cannot find definition(s) {} in '{}'. Available definitions may not match.",
            missing.iter().map(|s| format!("'{}'", s)).collect::<Vec<_>>().join(", "),
            import_path
        )));
    }

    Ok(extracted.join("\n\n"))
}

/// Extract a function definition by name from source
fn extract_function(source: &str, name: &str) -> Result<Option<String>, LatticeError> {
    let pattern = format!(r"\bdef\s+{}\s*\(", regex::escape(name));
    let re = Regex::new(&pattern)
        .map_err(|e| LatticeError::Runtime(format!("Regex error: {}", e)))?;

    if let Some(m) = re.find(source) {
        // Found the function start, now find the function body's opening brace
        let start = m.start();
        // Search for the opening brace after the parameters
        if let Some(brace_pos) = source[m.end()..].find('{') {
            let brace_start = m.end() + brace_pos;
            if let Some(end) = find_matching_brace(source, brace_start) {
                return Ok(Some(source[start..=end].to_string()));
            }
        }
    }
    Ok(None)
}

/// Extract a type definition by name from source
fn extract_type(source: &str, name: &str) -> Result<Option<String>, LatticeError> {
    let pattern = format!(r"\btype\s+{}\s*\{{", regex::escape(name));
    let re = Regex::new(&pattern)
        .map_err(|e| LatticeError::Runtime(format!("Regex error: {}", e)))?;

    if let Some(m) = re.find(source) {
        let start = m.start();
        if let Some(end) = find_matching_brace(source, m.end() - 1) {
            return Ok(Some(source[start..=end].to_string()));
        }
    }
    Ok(None)
}

/// Extract an enum definition by name from source
fn extract_enum(source: &str, name: &str) -> Result<Option<String>, LatticeError> {
    let pattern = format!(r"\benum\s+{}\s*\{{", regex::escape(name));
    let re = Regex::new(&pattern)
        .map_err(|e| LatticeError::Runtime(format!("Regex error: {}", e)))?;

    if let Some(m) = re.find(source) {
        let start = m.start();
        if let Some(end) = find_matching_brace(source, m.end() - 1) {
            return Ok(Some(source[start..=end].to_string()));
        }
    }
    Ok(None)
}

/// Extract an llm_config definition by name from source
fn extract_llm_config(source: &str, name: &str) -> Result<Option<String>, LatticeError> {
    let pattern = format!(r"\bllm_config\s+{}\s*\{{", regex::escape(name));
    let re = Regex::new(&pattern)
        .map_err(|e| LatticeError::Runtime(format!("Regex error: {}", e)))?;

    if let Some(m) = re.find(source) {
        let start = m.start();
        if let Some(end) = find_matching_brace(source, m.end() - 1) {
            return Ok(Some(source[start..=end].to_string()));
        }
    }
    Ok(None)
}

/// Find the position of the closing brace that matches the opening brace at `start`
/// Returns the byte offset of the closing brace, or None if not found
fn find_matching_brace(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    if start >= bytes.len() || bytes[start] != b'{' {
        return None;
    }

    let mut depth = 0;
    let mut in_string = false;
    let mut in_triple_string = false;
    let mut escape_next = false;
    let mut i = start;

    while i < bytes.len() {
        let c = bytes[i];

        if escape_next {
            escape_next = false;
            i += 1;
            continue;
        }

        if c == b'\\' && in_string {
            escape_next = true;
            i += 1;
            continue;
        }

        // Check for triple-quoted strings
        if i + 2 < bytes.len() && &bytes[i..i+3] == b"\"\"\"" {
            if in_triple_string {
                in_triple_string = false;
                i += 3;
                continue;
            } else if !in_string {
                in_triple_string = true;
                i += 3;
                continue;
            }
        }

        // Check for regular strings (only if not in triple string)
        if c == b'"' && !in_triple_string {
            in_string = !in_string;
            i += 1;
            continue;
        }

        // Only count braces outside strings
        if !in_string && !in_triple_string {
            if c == b'{' {
                depth += 1;
            } else if c == b'}' {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
        }

        i += 1;
    }

    None
}

/// Transform qualified name references (alias.name) to prefixed names (alias_name).
///
/// This handles:
/// - Function calls: `alias.func(args)` → `alias_func(args)`
/// - Type references: `alias.Type` → `alias_Type`
/// - Field access is NOT transformed (e.g., `obj.field` stays as is)
/// - Text inside regular string literals is preserved unchanged
/// - F-string interpolations `{expr}` are processed to transform qualified names
fn transform_qualified_names(source: &str, aliases: &[String]) -> Result<String, LatticeError> {
    if aliases.is_empty() {
        return Ok(source.to_string());
    }

    // Build regex for qualified names
    let alias_pattern = aliases
        .iter()
        .map(|a| regex::escape(a))
        .collect::<Vec<_>>()
        .join("|");

    let qualified_re = Regex::new(&format!(r"\b({})\s*\.\s*([a-zA-Z_][a-zA-Z0-9_]*)", alias_pattern))
        .map_err(|e| LatticeError::Runtime(format!("Regex error: {}", e)))?;

    // Process the source character by character, tracking string contexts
    let mut result = String::new();
    let chars: Vec<char> = source.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Check for triple-quoted string start (""" or f""")
        if i + 2 < chars.len() && source[i..].starts_with("\"\"\"") {
            let is_fstring = i > 0 && chars[i - 1] == 'f';
            let start = if is_fstring { i - 1 } else { i };
            if is_fstring && !result.is_empty() && result.ends_with('f') {
                result.pop(); // Remove the 'f' we already added
            }
            // Find the end of the triple-quoted string
            if let Some(end) = find_triple_quote_end(source, i + 3) {
                let string_content = &source[start..=end + 2];
                if is_fstring {
                    result.push_str(&transform_fstring(string_content, &qualified_re, aliases)?);
                } else {
                    result.push_str(string_content);
                }
                i = end + 3;
                continue;
            }
        }

        // Check for f-string start (f")
        if i + 1 < chars.len() && chars[i] == 'f' && chars[i + 1] == '"' {
            // Find the end of the f-string
            if let Some(end) = find_string_end(source, i + 2) {
                let fstring_content = &source[i..=end];
                result.push_str(&transform_fstring(fstring_content, &qualified_re, aliases)?);
                i = end + 1;
                continue;
            }
        }

        // Check for regular string start (")
        if chars[i] == '"' {
            // Find the end of the string
            if let Some(end) = find_string_end(source, i + 1) {
                // Copy the string as-is (no transformation)
                result.push_str(&source[i..=end]);
                i = end + 1;
                continue;
            }
        }

        // Check for qualified name outside strings
        if let Some(m) = qualified_re.find(&source[i..]) {
            if m.start() == 0 {
                // We have a match at current position
                let caps = qualified_re.captures(&source[i..]).unwrap();
                let alias = caps.get(1).unwrap().as_str();
                let name = caps.get(2).unwrap().as_str();
                result.push_str(&format!("{}_{}", alias, name));
                i += m.end();
                continue;
            }
        }

        // Regular character
        result.push(chars[i]);
        i += 1;
    }

    Ok(result)
}

/// Find the end of a regular string starting at the given position (after opening quote)
fn find_string_end(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut i = start;
    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 1 < bytes.len() {
            i += 2; // Skip escaped character
        } else if bytes[i] == b'"' {
            return Some(i);
        } else {
            i += 1;
        }
    }
    None
}

/// Find the end of a triple-quoted string starting at the given position (after opening """)
fn find_triple_quote_end(source: &str, start: usize) -> Option<usize> {
    if let Some(pos) = source[start..].find("\"\"\"") {
        return Some(start + pos);
    }
    None
}

/// Transform qualified names inside f-string interpolations
fn transform_fstring(fstring: &str, qualified_re: &Regex, _aliases: &[String]) -> Result<String, LatticeError> {
    // Parse the f-string and transform expressions inside {}, preserving text outside
    let mut result = String::new();
    let chars: Vec<char> = fstring.chars().collect();
    let mut i = 0;
    let mut in_interp = false;
    let mut brace_depth = 0;
    let mut current_expr = String::new();

    while i < chars.len() {
        let c = chars[i];

        if in_interp {
            if c == '{' {
                brace_depth += 1;
                current_expr.push(c);
            } else if c == '}' {
                brace_depth -= 1;
                if brace_depth == 0 {
                    // End of interpolation - transform the expression
                    let transformed = qualified_re.replace_all(&current_expr, |caps: &regex::Captures| {
                        let alias = caps.get(1).unwrap().as_str();
                        let name = caps.get(2).unwrap().as_str();
                        format!("{}_{}", alias, name)
                    });
                    result.push('{');
                    result.push_str(&transformed);
                    result.push('}');
                    in_interp = false;
                    current_expr.clear();
                } else {
                    current_expr.push(c);
                }
            } else {
                current_expr.push(c);
            }
        } else {
            // Not in interpolation
            if c == '{' && i + 1 < chars.len() && chars[i + 1] == '{' {
                // Escaped brace
                result.push_str("{{");
                i += 1;
            } else if c == '{' {
                // Start of interpolation
                in_interp = true;
                brace_depth = 1;
            } else if c == '}' && i + 1 < chars.len() && chars[i + 1] == '}' {
                // Escaped brace
                result.push_str("}}");
                i += 1;
            } else {
                result.push(c);
            }
        }
        i += 1;
    }

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

    // =========================================================================
    // Markdown LLM file import tests
    // =========================================================================

    #[test]
    fn test_import_markdown_llm_file() {
        let dir = TempDir::new().unwrap();
        let dir_path = dir.path();

        // Create a markdown LLM function file
        create_test_file(
            dir_path,
            "greet.md",
            r#"---
name: greet
model: gpt-4
input:
  name: String
output: String
---
Say hello to {name}!"#,
        );

        // Main file imports the markdown
        let source = r#"import "greet.md"
let result = greet("World")"#;

        let result = resolve_imports(source, dir_path).unwrap();

        // Should contain the transpiled function
        assert!(result.contains("def greet(name: String) -> String {"));
        assert!(result.contains("model: \"gpt-4\""));
        assert!(result.contains("${name}"));
        assert!(result.contains("prompt: \"\"\""));
        // Should also contain the usage
        assert!(result.contains("let result = greet(\"World\")"));
    }

    #[test]
    fn test_import_markdown_with_inline_type() {
        let dir = TempDir::new().unwrap();
        let dir_path = dir.path();

        create_test_file(
            dir_path,
            "analyze.md",
            r#"---
name: analyze_sentiment
model: gpt-4
input:
  text: String
output:
  sentiment: String
  confidence: Float
---
Analyze: {text}"#,
        );

        let source = r#"import "analyze.md""#;
        let result = resolve_imports(source, dir_path).unwrap();

        // Should generate the inline type
        assert!(result.contains("type AnalyzeSentimentOutput {"));
        assert!(result.contains("confidence: Float"));
        assert!(result.contains("sentiment: String"));
        // Function should use the generated type
        assert!(result.contains("-> AnalyzeSentimentOutput {"));
    }

    #[test]
    fn test_import_markdown_and_lat_together() {
        let dir = TempDir::new().unwrap();
        let dir_path = dir.path();

        // Create a type definition in a .lat file
        create_test_file(
            dir_path,
            "types.lat",
            "type Sentiment { label: String, score: Float }",
        );

        // Create a markdown LLM function that uses that type
        create_test_file(
            dir_path,
            "analyze.md",
            r#"---
name: analyze
model: gpt-4
input:
  text: String
output: Sentiment
---
Analyze: {text}"#,
        );

        // Main imports both
        let source = r#"import "types.lat"
import "analyze.md"
let result = analyze("Great!")"#;

        let result = resolve_imports(source, dir_path).unwrap();

        // Should contain both the type and the function
        assert!(result.contains("type Sentiment { label: String, score: Float }"));
        assert!(result.contains("def analyze(text: String) -> Sentiment {"));
        assert!(result.contains("let result = analyze"));
    }

    #[test]
    fn test_import_markdown_invalid_frontmatter() {
        let dir = TempDir::new().unwrap();
        let dir_path = dir.path();

        // Create an invalid markdown file (missing required 'model' field)
        create_test_file(
            dir_path,
            "invalid.md",
            r#"---
name: broken
input:
  x: String
output: String
---
prompt"#,
        );

        let source = r#"import "invalid.md""#;
        let result = resolve_imports(source, dir_path);

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Error parsing markdown file"));
    }

    #[test]
    fn test_import_markdown_deduplication() {
        let dir = TempDir::new().unwrap();
        let dir_path = dir.path();

        // Create a markdown file
        create_test_file(
            dir_path,
            "shared.md",
            r#"---
name: shared_fn
model: gpt-4
input:
  x: String
output: String
---
{x}"#,
        );

        // Import the same markdown file twice
        let source = r#"import "shared.md"
import "shared.md""#;

        let result = resolve_imports(source, dir_path).unwrap();

        // Should only include the function once
        let fn_count = result.matches("def shared_fn").count();
        assert_eq!(fn_count, 1, "Function should be included exactly once");
        // Second import should be marked as already imported
        assert!(result.contains("already imported"));
    }

    // =========================================================================
    // Namespaced import tests
    // =========================================================================

    #[test]
    fn test_namespaced_import_function() {
        let dir = TempDir::new().unwrap();
        let dir_path = dir.path();

        // Create a math library
        create_test_file(
            dir_path,
            "math.lat",
            r#"def add(a: Int, b: Int) -> Int { a + b }
def multiply(a: Int, b: Int) -> Int { a * b }"#,
        );

        // Import with alias
        let source = r#"import "math.lat" as math
let result = math.add(1, 2)"#;

        let result = resolve_imports(source, dir_path).unwrap();

        // Functions should be prefixed
        assert!(result.contains("def math_add(a: Int, b: Int)"));
        assert!(result.contains("def math_multiply(a: Int, b: Int)"));
        // Qualified call should be transformed
        assert!(result.contains("let result = math_add(1, 2)"));
        // Comment should show alias
        assert!(result.contains("import \"math.lat\" as math"));
    }

    #[test]
    fn test_namespaced_import_type() {
        let dir = TempDir::new().unwrap();
        let dir_path = dir.path();

        // Create a types library
        create_test_file(
            dir_path,
            "types.lat",
            "type Point { x: Int, y: Int }",
        );

        // Import with alias
        let source = r#"import "types.lat" as geo
let p = geo.Point { x: 1, y: 2 }"#;

        let result = resolve_imports(source, dir_path).unwrap();

        // Type should be prefixed
        assert!(result.contains("type geo_Point {"));
        // Qualified type reference should be transformed
        assert!(result.contains("let p = geo_Point { x: 1, y: 2 }"));
    }

    #[test]
    fn test_namespaced_import_enum() {
        let dir = TempDir::new().unwrap();
        let dir_path = dir.path();

        // Create an enum library
        create_test_file(
            dir_path,
            "colors.lat",
            "enum Color { Red, Green, Blue }",
        );

        // Import with alias
        let source = r#"import "colors.lat" as colors
let c = colors.Color"#;

        let result = resolve_imports(source, dir_path).unwrap();

        // Enum should be prefixed
        assert!(result.contains("enum colors_Color {"));
        // Qualified reference should be transformed
        assert!(result.contains("let c = colors_Color"));
    }

    #[test]
    fn test_namespaced_import_llm_config() {
        let dir = TempDir::new().unwrap();
        let dir_path = dir.path();

        // Create an llm_config library
        create_test_file(
            dir_path,
            "configs.lat",
            r#"llm_config fast_model {
    model: "gpt-4o-mini"
}"#,
        );

        // Import with alias
        let source = r#"import "configs.lat" as cfg"#;

        let result = resolve_imports(source, dir_path).unwrap();

        // llm_config should be prefixed
        assert!(result.contains("llm_config cfg_fast_model {"));
    }

    #[test]
    fn test_namespaced_import_multiple_aliases() {
        let dir = TempDir::new().unwrap();
        let dir_path = dir.path();

        // Create two libraries
        create_test_file(
            dir_path,
            "math.lat",
            "def add(a: Int, b: Int) -> Int { a + b }",
        );
        create_test_file(
            dir_path,
            "string.lat",
            "def concat(a: String, b: String) -> String { a + b }",
        );

        // Import both with aliases
        let source = r#"import "math.lat" as math
import "string.lat" as str
let x = math.add(1, 2)
let y = str.concat("a", "b")"#;

        let result = resolve_imports(source, dir_path).unwrap();

        // Both should be prefixed
        assert!(result.contains("def math_add"));
        assert!(result.contains("def str_concat"));
        // Both qualified calls should be transformed
        assert!(result.contains("let x = math_add(1, 2)"));
        assert!(result.contains("let y = str_concat(\"a\", \"b\")"));
    }

    #[test]
    fn test_namespaced_import_mixed_with_regular() {
        let dir = TempDir::new().unwrap();
        let dir_path = dir.path();

        // Create two libraries
        create_test_file(
            dir_path,
            "math.lat",
            "def add(a: Int, b: Int) -> Int { a + b }",
        );
        create_test_file(
            dir_path,
            "helpers.lat",
            "def helper() -> Int { 42 }",
        );

        // Mix aliased and non-aliased imports
        let source = r#"import "math.lat" as math
import "helpers.lat"
let x = math.add(1, 2)
let y = helper()"#;

        let result = resolve_imports(source, dir_path).unwrap();

        // math should be prefixed
        assert!(result.contains("def math_add"));
        // helpers should NOT be prefixed
        assert!(result.contains("def helper()"));
        // Qualified call should be transformed
        assert!(result.contains("let x = math_add(1, 2)"));
        // Regular call should remain unchanged
        assert!(result.contains("let y = helper()"));
    }

    #[test]
    fn test_namespaced_import_with_markdown() {
        let dir = TempDir::new().unwrap();
        let dir_path = dir.path();

        // Create a markdown LLM function
        create_test_file(
            dir_path,
            "sentiment.md",
            r#"---
name: analyze
model: gpt-4
input:
  text: String
output: String
---
Analyze: {text}"#,
        );

        // Import with alias
        let source = r#"import "sentiment.md" as ai
let result = ai.analyze("Hello")"#;

        let result = resolve_imports(source, dir_path).unwrap();

        // Function should be prefixed
        assert!(result.contains("def ai_analyze(text: String)"));
        // Qualified call should be transformed
        assert!(result.contains("let result = ai_analyze(\"Hello\")"));
    }

    #[test]
    fn test_prefix_definitions_function() {
        let source = "def foo(x: Int) -> Int { x + 1 }";
        let result = prefix_definitions(source, "math").unwrap();
        assert_eq!(result, "def math_foo(x: Int) -> Int { x + 1 }");
    }

    #[test]
    fn test_prefix_definitions_type() {
        let source = "type Point { x: Int, y: Int }";
        let result = prefix_definitions(source, "geo").unwrap();
        assert_eq!(result, "type geo_Point { x: Int, y: Int }");
    }

    #[test]
    fn test_prefix_definitions_enum() {
        let source = "enum Status { Active, Inactive }";
        let result = prefix_definitions(source, "state").unwrap();
        assert_eq!(result, "enum state_Status { Active, Inactive }");
    }

    #[test]
    fn test_transform_qualified_names_simple() {
        let source = "let x = math.add(1, 2)";
        let aliases = vec!["math".to_string()];
        let result = transform_qualified_names(source, &aliases).unwrap();
        assert_eq!(result, "let x = math_add(1, 2)");
    }

    #[test]
    fn test_transform_qualified_names_type() {
        let source = "let p = geo.Point { x: 1 }";
        let aliases = vec!["geo".to_string()];
        let result = transform_qualified_names(source, &aliases).unwrap();
        assert_eq!(result, "let p = geo_Point { x: 1 }");
    }

    #[test]
    fn test_transform_qualified_names_preserves_field_access() {
        // obj.field should NOT be transformed when obj is not an alias
        let source = "let x = obj.field";
        let aliases = vec!["math".to_string()];
        let result = transform_qualified_names(source, &aliases).unwrap();
        assert_eq!(result, "let x = obj.field");
    }

    #[test]
    fn test_transform_qualified_names_multiple_aliases() {
        let source = "let x = math.add(1, str.len(\"hi\"))";
        let aliases = vec!["math".to_string(), "str".to_string()];
        let result = transform_qualified_names(source, &aliases).unwrap();
        assert_eq!(result, "let x = math_add(1, str_len(\"hi\"))");
    }

    #[test]
    fn test_transform_qualified_names_preserves_strings() {
        // Qualified names inside strings should NOT be transformed
        let source = r#"let x = math.add(1, 2)
let msg = "Using math.add function""#;
        let aliases = vec!["math".to_string()];
        let result = transform_qualified_names(source, &aliases).unwrap();
        assert!(result.contains("math_add(1, 2)"), "Code should be transformed");
        assert!(result.contains("\"Using math.add function\""), "String should be preserved");
    }

    #[test]
    fn test_transform_qualified_names_preserves_fstrings() {
        // Qualified names in f-string text should NOT be transformed
        // But actual code expressions would be (which happens before import resolution)
        let source = r#"let x = math.add(1, 2)
print(f"Called math.add: {x}")"#;
        let aliases = vec!["math".to_string()];
        let result = transform_qualified_names(source, &aliases).unwrap();
        assert!(result.contains("math_add(1, 2)"), "Code should be transformed");
        assert!(result.contains("f\"Called math.add: {x}\""), "F-string literal should be preserved");
    }

    // =========================================================================
    // Selective import tests
    // =========================================================================

    #[test]
    fn test_selective_import_single_function() {
        let dir = TempDir::new().unwrap();
        let dir_path = dir.path();

        // Create a library with multiple functions
        create_test_file(
            dir_path,
            "math.lat",
            r#"def add(a: Int, b: Int) -> Int { a + b }
def subtract(a: Int, b: Int) -> Int { a - b }
def multiply(a: Int, b: Int) -> Int { a * b }"#,
        );

        // Import only one function
        let source = r#"from "math.lat" import add
let result = add(1, 2)"#;

        let result = resolve_imports(source, dir_path).unwrap();

        // Only add should be included
        assert!(result.contains("def add(a: Int, b: Int)"));
        assert!(!result.contains("def subtract"));
        assert!(!result.contains("def multiply"));
        assert!(result.contains("let result = add(1, 2)"));
    }

    #[test]
    fn test_selective_import_multiple_functions() {
        let dir = TempDir::new().unwrap();
        let dir_path = dir.path();

        create_test_file(
            dir_path,
            "math.lat",
            r#"def add(a: Int, b: Int) -> Int { a + b }
def subtract(a: Int, b: Int) -> Int { a - b }
def multiply(a: Int, b: Int) -> Int { a * b }"#,
        );

        // Import two functions
        let source = r#"from "math.lat" import add, multiply
let sum = add(1, 2)
let product = multiply(3, 4)"#;

        let result = resolve_imports(source, dir_path).unwrap();

        // add and multiply should be included, but not subtract
        assert!(result.contains("def add(a: Int, b: Int)"));
        assert!(result.contains("def multiply(a: Int, b: Int)"));
        assert!(!result.contains("def subtract"));
    }

    #[test]
    fn test_selective_import_type() {
        let dir = TempDir::new().unwrap();
        let dir_path = dir.path();

        create_test_file(
            dir_path,
            "types.lat",
            r#"type Point { x: Int, y: Int }
type Rectangle { width: Int, height: Int }
type Circle { radius: Float }"#,
        );

        // Import only Point
        let source = r#"from "types.lat" import Point
let p = Point { x: 1, y: 2 }"#;

        let result = resolve_imports(source, dir_path).unwrap();

        assert!(result.contains("type Point { x: Int, y: Int }"));
        assert!(!result.contains("type Rectangle"));
        assert!(!result.contains("type Circle"));
    }

    #[test]
    fn test_selective_import_enum() {
        let dir = TempDir::new().unwrap();
        let dir_path = dir.path();

        create_test_file(
            dir_path,
            "enums.lat",
            r#"enum Color { Red, Green, Blue }
enum Status { Active, Inactive }
enum Size { Small, Medium, Large }"#,
        );

        // Import only Color
        let source = r#"from "enums.lat" import Color"#;

        let result = resolve_imports(source, dir_path).unwrap();

        assert!(result.contains("enum Color { Red, Green, Blue }"));
        assert!(!result.contains("enum Status"));
        assert!(!result.contains("enum Size"));
    }

    #[test]
    fn test_selective_import_mixed_types() {
        let dir = TempDir::new().unwrap();
        let dir_path = dir.path();

        create_test_file(
            dir_path,
            "lib.lat",
            r#"type Point { x: Int, y: Int }
enum Color { Red, Green, Blue }
def distance(p: Point) -> Int { p.x * p.x + p.y * p.y }"#,
        );

        // Import a type and a function
        let source = r#"from "lib.lat" import Point, distance
let p = Point { x: 3, y: 4 }
let d = distance(p)"#;

        let result = resolve_imports(source, dir_path).unwrap();

        assert!(result.contains("type Point { x: Int, y: Int }"));
        assert!(result.contains("def distance(p: Point)"));
        assert!(!result.contains("enum Color"));
    }

    #[test]
    fn test_selective_import_not_found() {
        let dir = TempDir::new().unwrap();
        let dir_path = dir.path();

        create_test_file(
            dir_path,
            "math.lat",
            "def add(a: Int, b: Int) -> Int { a + b }",
        );

        // Try to import a function that doesn't exist
        let source = r#"from "math.lat" import nonexistent"#;

        let result = resolve_imports(source, dir_path);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Cannot find definition"));
        assert!(err.contains("nonexistent"));
    }

    #[test]
    fn test_selective_import_with_regular_import() {
        let dir = TempDir::new().unwrap();
        let dir_path = dir.path();

        create_test_file(
            dir_path,
            "math.lat",
            r#"def add(a: Int, b: Int) -> Int { a + b }
def multiply(a: Int, b: Int) -> Int { a * b }"#,
        );
        create_test_file(
            dir_path,
            "helpers.lat",
            "def helper() -> Int { 42 }",
        );

        // Mix selective and regular imports
        let source = r#"from "math.lat" import add
import "helpers.lat"
let x = add(1, 2)
let y = helper()"#;

        let result = resolve_imports(source, dir_path).unwrap();

        // Selective: only add from math
        assert!(result.contains("def add(a: Int, b: Int)"));
        assert!(!result.contains("def multiply"));
        // Regular: all of helpers
        assert!(result.contains("def helper()"));
    }

    #[test]
    fn test_extract_function_simple() {
        let source = r#"def foo(x: Int) -> Int { x + 1 }
def bar() -> String { "hello" }"#;
        let result = extract_function(source, "foo").unwrap();
        assert!(result.is_some());
        let extracted = result.unwrap();
        assert!(extracted.starts_with("def foo"));
        assert!(extracted.contains("x + 1"));
        assert!(!extracted.contains("bar"));
    }

    #[test]
    fn test_extract_function_with_nested_braces() {
        let source = r#"def foo(x: Int) -> Int {
    if x > 0 {
        x + 1
    } else {
        0
    }
}
def bar() -> Int { 1 }"#;
        let result = extract_function(source, "foo").unwrap();
        assert!(result.is_some());
        let extracted = result.unwrap();
        assert!(extracted.contains("if x > 0"));
        assert!(extracted.contains("else"));
        assert!(extracted.ends_with("}"));
    }

    #[test]
    fn test_extract_type_simple() {
        let source = r#"type Point { x: Int, y: Int }
type Other { a: String }"#;
        let result = extract_type(source, "Point").unwrap();
        assert!(result.is_some());
        let extracted = result.unwrap();
        assert_eq!(extracted, "type Point { x: Int, y: Int }");
    }

    #[test]
    fn test_extract_enum_simple() {
        let source = r#"enum Color { Red, Green, Blue }
enum Size { Small, Large }"#;
        let result = extract_enum(source, "Color").unwrap();
        assert!(result.is_some());
        let extracted = result.unwrap();
        assert_eq!(extracted, "enum Color { Red, Green, Blue }");
    }

    #[test]
    fn test_find_matching_brace_simple() {
        let source = "{ }";
        let end = find_matching_brace(source, 0);
        assert_eq!(end, Some(2));
    }

    #[test]
    fn test_find_matching_brace_nested() {
        let source = "{ { } }";
        let end = find_matching_brace(source, 0);
        assert_eq!(end, Some(6));
    }

    #[test]
    fn test_find_matching_brace_with_string() {
        let source = r#"{ "}" }"#;
        let end = find_matching_brace(source, 0);
        assert_eq!(end, Some(6));
    }

    #[test]
    fn test_find_matching_brace_with_triple_string() {
        // Triple-quoted strings containing braces should be handled
        let source = r#"{ """}}""" }"#;
        let end = find_matching_brace(source, 0);
        // The string is: { """}}""" }
        //               01234567890123
        // The closing brace is at position 11
        assert_eq!(end, Some(11));
    }
}
