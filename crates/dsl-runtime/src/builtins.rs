//! Builtin Functions for DSL Runtime
//!
//! This module re-exports builtin functions from dsl-interpreter
//! and provides convenient wrappers for generated code.

pub use dsl_interpreter::BuiltinFunctions;
pub use dsl_ir::Value;

/// Simple string operations

pub fn upper(s: &str) -> String {
    s.to_uppercase()
}

pub fn lower(s: &str) -> String {
    s.to_lowercase()
}

pub fn length(s: &str) -> i64 {
    s.len() as i64
}

pub fn not(b: bool) -> bool {
    !b
}

pub fn join(items: Vec<String>, separator: &str) -> String {
    items.join(separator)
}

pub fn render_markdown(s: String) -> Value {
    Value::Markdown(s)
}

/// Async builtin functions that require runtime state
/// These are re-exported from dsl-interpreter and should be called
/// through a BuiltinFunctions instance in generated code.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upper() {
        assert_eq!(upper("hello"), "HELLO");
    }

    #[test]
    fn test_lower() {
        assert_eq!(lower("WORLD"), "world");
    }

    #[test]
    fn test_length() {
        assert_eq!(length("hello"), 5);
    }

    #[test]
    fn test_not() {
        assert_eq!(not(true), false);
        assert_eq!(not(false), true);
    }

    #[test]
    fn test_join() {
        assert_eq!(join(vec!["a".to_string(), "b".to_string()], ", "), "a, b");
    }

    #[test]
    fn test_render_markdown() {
        let result = render_markdown("# Hello".to_string());
        assert_eq!(result, Value::Markdown("# Hello".to_string()));
    }
}
