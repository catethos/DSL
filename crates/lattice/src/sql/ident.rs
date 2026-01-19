//! SQL identifier utilities
//!
//! This module provides utilities for safe SQL identifier handling,
//! including quoting identifiers to prevent SQL injection and validating
//! that identifiers can be used without quoting.

/// Quote an identifier for safe use in SQL
///
/// This function properly escapes identifiers by:
/// 1. Wrapping in double quotes
/// 2. Escaping any internal double quotes by doubling them
///
/// This prevents SQL injection and handles special characters.
///
/// # Examples
///
/// ```ignore
/// assert_eq!(quote_ident("users"), "\"users\"");
/// assert_eq!(quote_ident("user-data"), "\"user-data\"");
/// assert_eq!(quote_ident("table\"name"), "\"table\"\"name\"");
/// ```
pub fn quote_ident(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

/// Check if a name is a valid unquoted SQL identifier
///
/// A valid unquoted identifier:
/// - Starts with a letter (a-z, A-Z) or underscore
/// - Contains only letters, digits, and underscores
/// - Is not a SQL reserved word
///
/// # Examples
///
/// ```ignore
/// assert!(is_valid_unquoted_ident("users"));
/// assert!(is_valid_unquoted_ident("user_data"));
/// assert!(!is_valid_unquoted_ident("user-data"));  // contains hyphen
/// assert!(!is_valid_unquoted_ident("123abc"));     // starts with digit
/// assert!(!is_valid_unquoted_ident("select"));     // reserved word
/// ```
pub fn is_valid_unquoted_ident(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let mut chars = name.chars();

    // First character must be letter or underscore
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }

    // Remaining characters must be letter, digit, or underscore
    for c in chars {
        if !c.is_ascii_alphanumeric() && c != '_' {
            return false;
        }
    }

    // Check against reserved words (case-insensitive)
    !is_reserved_word(name)
}

/// Check if a name is a SQL reserved word
///
/// This checks against common SQL reserved words that cannot be used
/// as unquoted identifiers in most SQL dialects.
fn is_reserved_word(name: &str) -> bool {
    // Convert to uppercase for case-insensitive comparison
    let upper = name.to_ascii_uppercase();

    // Common SQL reserved words
    // This is not exhaustive but covers the most common cases
    matches!(
        upper.as_str(),
        "ADD"
            | "ALL"
            | "ALTER"
            | "AND"
            | "ANY"
            | "AS"
            | "ASC"
            | "BETWEEN"
            | "BY"
            | "CASE"
            | "CHECK"
            | "COLUMN"
            | "CONSTRAINT"
            | "CREATE"
            | "CROSS"
            | "CURRENT"
            | "CURRENT_DATE"
            | "CURRENT_TIME"
            | "CURRENT_TIMESTAMP"
            | "DATABASE"
            | "DEFAULT"
            | "DELETE"
            | "DESC"
            | "DISTINCT"
            | "DROP"
            | "ELSE"
            | "END"
            | "ESCAPE"
            | "EXCEPT"
            | "EXISTS"
            | "FALSE"
            | "FETCH"
            | "FOR"
            | "FOREIGN"
            | "FROM"
            | "FULL"
            | "GROUP"
            | "HAVING"
            | "IF"
            | "IN"
            | "INDEX"
            | "INNER"
            | "INSERT"
            | "INTERSECT"
            | "INTO"
            | "IS"
            | "JOIN"
            | "KEY"
            | "LEFT"
            | "LIKE"
            | "LIMIT"
            | "NOT"
            | "NULL"
            | "OFFSET"
            | "ON"
            | "OR"
            | "ORDER"
            | "OUTER"
            | "PRIMARY"
            | "REFERENCES"
            | "RIGHT"
            | "SELECT"
            | "SET"
            | "TABLE"
            | "THEN"
            | "TRUE"
            | "UNION"
            | "UNIQUE"
            | "UPDATE"
            | "USING"
            | "VALUES"
            | "VIEW"
            | "WHEN"
            | "WHERE"
            | "WITH"
    )
}

/// Normalize an identifier for case-insensitive lookup
///
/// SQL identifiers are typically case-insensitive unless quoted.
/// This function converts to lowercase for consistent lookups.
pub fn normalize_ident(name: &str) -> String {
    name.to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==========================================================================
    // quote_ident tests
    // ==========================================================================

    #[test]
    fn test_quote_simple_identifier() {
        assert_eq!(quote_ident("users"), "\"users\"");
        assert_eq!(quote_ident("my_table"), "\"my_table\"");
    }

    #[test]
    fn test_quote_identifier_with_special_chars() {
        assert_eq!(quote_ident("user-data"), "\"user-data\"");
        assert_eq!(quote_ident("table.name"), "\"table.name\"");
        assert_eq!(quote_ident("space name"), "\"space name\"");
    }

    #[test]
    fn test_quote_identifier_with_quotes() {
        // Double quotes in name should be escaped by doubling
        assert_eq!(quote_ident("table\"name"), "\"table\"\"name\"");
        assert_eq!(quote_ident("a\"b\"c"), "\"a\"\"b\"\"c\"");
    }

    #[test]
    fn test_quote_empty_identifier() {
        assert_eq!(quote_ident(""), "\"\"");
    }

    #[test]
    fn test_quote_sql_injection_attempt() {
        // Attempt to break out of quotes should be escaped
        assert_eq!(
            quote_ident("users\"; DROP TABLE users; --"),
            "\"users\"\"; DROP TABLE users; --\""
        );
    }

    // ==========================================================================
    // is_valid_unquoted_ident tests
    // ==========================================================================

    #[test]
    fn test_valid_identifiers() {
        assert!(is_valid_unquoted_ident("users"));
        assert!(is_valid_unquoted_ident("Users"));
        assert!(is_valid_unquoted_ident("user_data"));
        assert!(is_valid_unquoted_ident("_private"));
        assert!(is_valid_unquoted_ident("a1"));
        assert!(is_valid_unquoted_ident("table123"));
    }

    #[test]
    fn test_invalid_identifiers_special_chars() {
        assert!(!is_valid_unquoted_ident("user-data"));
        assert!(!is_valid_unquoted_ident("table.name"));
        assert!(!is_valid_unquoted_ident("space name"));
        assert!(!is_valid_unquoted_ident("tab\tname"));
    }

    #[test]
    fn test_invalid_identifiers_starts_with_digit() {
        assert!(!is_valid_unquoted_ident("123abc"));
        assert!(!is_valid_unquoted_ident("1table"));
        assert!(!is_valid_unquoted_ident("0"));
    }

    #[test]
    fn test_invalid_identifiers_empty() {
        assert!(!is_valid_unquoted_ident(""));
    }

    #[test]
    fn test_reserved_words_rejected() {
        assert!(!is_valid_unquoted_ident("select"));
        assert!(!is_valid_unquoted_ident("SELECT"));
        assert!(!is_valid_unquoted_ident("Select"));
        assert!(!is_valid_unquoted_ident("from"));
        assert!(!is_valid_unquoted_ident("where"));
        assert!(!is_valid_unquoted_ident("table"));
        assert!(!is_valid_unquoted_ident("join"));
        assert!(!is_valid_unquoted_ident("null"));
        assert!(!is_valid_unquoted_ident("true"));
        assert!(!is_valid_unquoted_ident("false"));
    }

    #[test]
    fn test_non_reserved_words_accepted() {
        // Words that look like reserved words but aren't
        assert!(is_valid_unquoted_ident("selecting"));
        assert!(is_valid_unquoted_ident("from_date"));
        assert!(is_valid_unquoted_ident("table_name"));
        assert!(is_valid_unquoted_ident("users"));
        assert!(is_valid_unquoted_ident("orders"));
    }

    // ==========================================================================
    // normalize_ident tests
    // ==========================================================================

    #[test]
    fn test_normalize_lowercase() {
        assert_eq!(normalize_ident("users"), "users");
    }

    #[test]
    fn test_normalize_uppercase() {
        assert_eq!(normalize_ident("USERS"), "users");
    }

    #[test]
    fn test_normalize_mixed_case() {
        assert_eq!(normalize_ident("UserData"), "userdata");
        assert_eq!(normalize_ident("MyTable"), "mytable");
    }

    #[test]
    fn test_normalize_preserves_underscores() {
        assert_eq!(normalize_ident("USER_DATA"), "user_data");
    }

    // ==========================================================================
    // Integration-style tests
    // ==========================================================================

    #[test]
    fn test_quote_then_use_in_sql() {
        // Simulate building a SQL statement with quoted identifier
        let table_name = "user-data";
        let quoted = quote_ident(table_name);
        let sql = format!("SELECT * FROM {}", quoted);
        assert_eq!(sql, "SELECT * FROM \"user-data\"");
    }

    #[test]
    fn test_check_before_quote() {
        // If valid unquoted, can use directly; otherwise quote
        let name1 = "users";
        let name2 = "user-data";

        let sql1 = if is_valid_unquoted_ident(name1) {
            format!("SELECT * FROM {}", name1)
        } else {
            format!("SELECT * FROM {}", quote_ident(name1))
        };

        let sql2 = if is_valid_unquoted_ident(name2) {
            format!("SELECT * FROM {}", name2)
        } else {
            format!("SELECT * FROM {}", quote_ident(name2))
        };

        assert_eq!(sql1, "SELECT * FROM users");
        assert_eq!(sql2, "SELECT * FROM \"user-data\"");
    }
}
