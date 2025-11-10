//! Language keywords and constructs
//!
//! This module provides a central definition of all DSL language keywords,
//! types, and other language constructs. These are used by the parser and
//! can be referenced by tools like autocomplete.

/// Declaration keywords (def, type, enum, workflow)
pub const DECLARATION_KEYWORDS: &[&str] = &["def", "type", "enum", "workflow"];

/// Control flow and binding keywords
pub const CONTROL_KEYWORDS: &[&str] = &["let", "as", "match", "if"];

/// Function execution block prefixes (prompt:, sql:, http:)
pub const BLOCK_KEYWORDS: &[&str] = &["prompt:", "sql:", "http:"];

/// Primitive type names
pub const PRIMITIVE_TYPES: &[&str] = &["String", "Int", "Float", "Bool", "Table", "Any"];

/// Boolean literals
pub const BOOLEAN_LITERALS: &[&str] = &["true", "false"];

/// Get all keywords (declaration + control + blocks)
pub fn all_keywords() -> Vec<&'static str> {
    DECLARATION_KEYWORDS
        .iter()
        .chain(CONTROL_KEYWORDS.iter())
        .chain(BLOCK_KEYWORDS.iter())
        .copied()
        .collect()
}

/// Get all types (primitive types)
pub fn all_types() -> Vec<&'static str> {
    PRIMITIVE_TYPES.to_vec()
}

/// Get all boolean literals
pub fn all_boolean_literals() -> Vec<&'static str> {
    BOOLEAN_LITERALS.to_vec()
}

/// Keyword metadata for autocomplete
#[derive(Debug, Clone)]
pub struct KeywordInfo {
    pub keyword: &'static str,
    pub description: &'static str,
}

/// Get keyword information with descriptions
pub fn keyword_info() -> Vec<KeywordInfo> {
    vec![
        // Declaration keywords
        KeywordInfo {
            keyword: "def",
            description: "Function definition",
        },
        KeywordInfo {
            keyword: "type",
            description: "Type definition",
        },
        KeywordInfo {
            keyword: "enum",
            description: "Enum definition",
        },
        KeywordInfo {
            keyword: "workflow",
            description: "Workflow definition",
        },
        // Control flow and binding
        KeywordInfo {
            keyword: "let",
            description: "Variable binding (let x = expr)",
        },
        KeywordInfo {
            keyword: "as",
            description: "Variable binding",
        },
        KeywordInfo {
            keyword: "match",
            description: "Pattern matching",
        },
        KeywordInfo {
            keyword: "if",
            description: "Conditional guard",
        },
        // Function execution blocks
        KeywordInfo {
            keyword: "prompt:",
            description: "LLM prompt block",
        },
        KeywordInfo {
            keyword: "sql:",
            description: "SQL query block",
        },
        KeywordInfo {
            keyword: "http:",
            description: "HTTP request block",
        },
    ]
}

/// Type metadata for autocomplete
#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub type_name: &'static str,
    pub description: &'static str,
}

/// Get type information with descriptions
pub fn type_info() -> Vec<TypeInfo> {
    vec![
        TypeInfo {
            type_name: "String",
            description: "String type",
        },
        TypeInfo {
            type_name: "Int",
            description: "Integer type",
        },
        TypeInfo {
            type_name: "Float",
            description: "Float type",
        },
        TypeInfo {
            type_name: "Bool",
            description: "Boolean type",
        },
        TypeInfo {
            type_name: "Table",
            description: "Table type",
        },
        TypeInfo {
            type_name: "Any",
            description: "Any type",
        },
    ]
}

/// Boolean literal metadata
#[derive(Debug, Clone)]
pub struct BooleanInfo {
    pub literal: &'static str,
    pub description: &'static str,
}

/// Get boolean literal information
pub fn boolean_info() -> Vec<BooleanInfo> {
    vec![
        BooleanInfo {
            literal: "true",
            description: "Boolean true",
        },
        BooleanInfo {
            literal: "false",
            description: "Boolean false",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_keywords() {
        let keywords = all_keywords();
        assert!(keywords.contains(&"def"));
        assert!(keywords.contains(&"type"));
        assert!(keywords.contains(&"let"));
        assert!(keywords.contains(&"prompt:"));
    }

    #[test]
    fn test_all_types() {
        let types = all_types();
        assert!(types.contains(&"String"));
        assert!(types.contains(&"Int"));
        assert!(types.contains(&"Bool"));
    }

    #[test]
    fn test_keyword_info() {
        let info = keyword_info();
        assert!(info.iter().any(|k| k.keyword == "def"));
        assert!(info.iter().any(|k| k.keyword == "let"));
    }
}
