//! SQL table reference extraction using AST parsing
//!
//! This module provides robust table name extraction from SQL queries using
//! sqlparser-rs. This is used to identify which Lattice variables need to be
//! registered as virtual tables before query execution.

use sqlparser::ast::{
    Expr, FunctionArg, FunctionArgExpr, Query, Select, SelectItem, SetExpr, Statement,
    TableFactor, TableWithJoins,
};
use sqlparser::dialect::DuckDbDialect;
use sqlparser::parser::Parser;

use crate::runtime::providers::SqlError;

/// Extract table references from a SQL query using AST parsing
///
/// Returns a list of table names referenced in the query. Only extracts
/// simple table references - table functions, schema-qualified names, and
/// other complex constructs are ignored.
///
/// # Errors
///
/// Returns an error for:
/// - SQL syntax errors
/// - Unsupported constructs (CTEs, subqueries, derived tables)
///
/// # Example
///
/// ```ignore
/// let tables = extract_table_references("SELECT * FROM users JOIN orders ON users.id = orders.user_id")?;
/// assert_eq!(tables, vec!["users", "orders"]);
/// ```
pub fn extract_table_references(sql: &str) -> Result<Vec<String>, SqlError> {
    let dialect = DuckDbDialect {};
    let ast = Parser::parse_sql(&dialect, sql)
        .map_err(|e| SqlError::PrepareError(format!("SQL parse error: {}", e)))?;

    let mut tables = Vec::new();
    for statement in ast {
        extract_tables_from_statement(&statement, &mut tables)?;
    }

    // Remove duplicates while preserving order
    let mut seen = std::collections::HashSet::new();
    tables.retain(|t| seen.insert(t.clone()));

    Ok(tables)
}

/// Extract tables from a single SQL statement
fn extract_tables_from_statement(
    stmt: &Statement,
    tables: &mut Vec<String>,
) -> Result<(), SqlError> {
    match stmt {
        Statement::Query(query) => {
            extract_tables_from_query(query, tables)?;
        }
        Statement::Insert {
            table_name, source, ..
        } => {
            // Extract table name from INSERT
            if let Some(name) = simple_table_name(table_name) {
                tables.push(name);
            }
            // Also check the source query if present
            if let Some(src) = source {
                extract_tables_from_query(src, tables)?;
            }
        }
        Statement::Update { table, from, .. } => {
            // Extract from UPDATE target
            extract_tables_from_table_with_joins(table, tables)?;
            // And FROM clause if present
            if let Some(from_clause) = from {
                extract_tables_from_table_with_joins(from_clause, tables)?;
            }
        }
        Statement::Delete { from, .. } => {
            // Extract from DELETE target
            for twj in from {
                extract_tables_from_table_with_joins(twj, tables)?;
            }
        }
        // Other statements don't typically reference Lattice variables
        _ => {}
    }
    Ok(())
}

/// Extract tables from a query (SELECT, UNION, etc.)
fn extract_tables_from_query(query: &Query, tables: &mut Vec<String>) -> Result<(), SqlError> {
    // Reject CTEs (WITH clauses)
    if query.with.is_some() {
        return Err(SqlError::PrepareError(
            "SQL on Lattice data does not support CTEs (WITH clauses). \
             Please rewrite the query without CTEs."
                .to_string(),
        ));
    }

    extract_tables_from_set_expr(&query.body, tables)?;
    Ok(())
}

/// Extract tables from a set expression (SELECT, UNION, etc.)
fn extract_tables_from_set_expr(
    set_expr: &SetExpr,
    tables: &mut Vec<String>,
) -> Result<(), SqlError> {
    match set_expr {
        SetExpr::Select(select) => {
            extract_tables_from_select(select, tables)?;
        }
        SetExpr::Query(query) => {
            extract_tables_from_query(query, tables)?;
        }
        SetExpr::SetOperation { left, right, .. } => {
            // UNION, INTERSECT, EXCEPT
            extract_tables_from_set_expr(left, tables)?;
            extract_tables_from_set_expr(right, tables)?;
        }
        SetExpr::Values(_) => {
            // VALUES clause - no tables
        }
        SetExpr::Insert(stmt) => {
            extract_tables_from_statement(stmt, tables)?;
        }
        SetExpr::Update(stmt) => {
            extract_tables_from_statement(stmt, tables)?;
        }
        SetExpr::Table(table) => {
            // TABLE <table_name> syntax
            if let Some(ref table_name) = table.table_name {
                if let Some(name) = simple_object_name(table_name) {
                    tables.push(name);
                }
            }
        }
    }
    Ok(())
}

/// Extract tables from a SELECT statement
fn extract_tables_from_select(select: &Select, tables: &mut Vec<String>) -> Result<(), SqlError> {
    // Check for subqueries in SELECT items
    for item in &select.projection {
        if let SelectItem::ExprWithAlias { expr, .. } | SelectItem::UnnamedExpr(expr) = item {
            check_expr_for_subquery(expr)?;
        }
    }

    // Extract from FROM clause
    for table_with_joins in &select.from {
        extract_tables_from_table_with_joins(table_with_joins, tables)?;
    }

    // Check WHERE clause for subqueries
    if let Some(where_expr) = &select.selection {
        check_expr_for_subquery(where_expr)?;
    }

    // Check HAVING clause for subqueries
    if let Some(having_expr) = &select.having {
        check_expr_for_subquery(having_expr)?;
    }

    Ok(())
}

/// Extract tables from a TableWithJoins (main table + joins)
fn extract_tables_from_table_with_joins(
    twj: &TableWithJoins,
    tables: &mut Vec<String>,
) -> Result<(), SqlError> {
    extract_tables_from_table_factor(&twj.relation, tables)?;

    for join in &twj.joins {
        extract_tables_from_table_factor(&join.relation, tables)?;
    }

    Ok(())
}

/// Extract table name from a TableFactor
fn extract_tables_from_table_factor(
    factor: &TableFactor,
    tables: &mut Vec<String>,
) -> Result<(), SqlError> {
    match factor {
        TableFactor::Table { name, args, .. } => {
            // Ignore table functions (tables with arguments like read_csv('file.csv'))
            if args.is_some() {
                return Ok(());
            }
            // Only extract simple table names (not schema-qualified)
            if let Some(table_name) = simple_table_name(name) {
                tables.push(table_name);
            }
        }
        TableFactor::Derived { .. } => {
            // Reject derived tables (subqueries in FROM)
            return Err(SqlError::PrepareError(
                "SQL on Lattice data does not support subqueries in FROM clause. \
                 Please rewrite the query without derived tables."
                    .to_string(),
            ));
        }
        TableFactor::NestedJoin { table_with_joins, .. } => {
            // Parenthesized join - recurse into it
            extract_tables_from_table_with_joins(table_with_joins, tables)?;
        }
        TableFactor::TableFunction { .. } => {
            // Table functions (like read_csv()) - ignore, not Lattice variables
        }
        TableFactor::UNNEST { .. } => {
            // UNNEST - ignore
        }
        TableFactor::Pivot { table, .. } | TableFactor::Unpivot { table, .. } => {
            // PIVOT/UNPIVOT - recurse into the source table
            extract_tables_from_table_factor(table, tables)?;
        }
        TableFactor::JsonTable { .. } => {
            // JSON_TABLE - ignore
        }
        // Catch-all for any other variants we don't need to handle
        _ => {}
    }
    Ok(())
}

/// Check an expression for subqueries and reject if found
fn check_expr_for_subquery(expr: &Expr) -> Result<(), SqlError> {
    match expr {
        Expr::Subquery(_) => {
            return Err(SqlError::PrepareError(
                "SQL on Lattice data does not support subqueries. \
                 Please rewrite the query without subqueries."
                    .to_string(),
            ));
        }
        Expr::InSubquery { .. } => {
            return Err(SqlError::PrepareError(
                "SQL on Lattice data does not support IN (SELECT ...) subqueries. \
                 Please rewrite the query without subqueries."
                    .to_string(),
            ));
        }
        Expr::Exists { .. } => {
            return Err(SqlError::PrepareError(
                "SQL on Lattice data does not support EXISTS subqueries. \
                 Please rewrite the query without subqueries."
                    .to_string(),
            ));
        }
        // For other expression types, we need to recursively check nested expressions
        Expr::BinaryOp { left, right, .. } => {
            check_expr_for_subquery(left)?;
            check_expr_for_subquery(right)?;
        }
        Expr::UnaryOp { expr, .. } => {
            check_expr_for_subquery(expr)?;
        }
        Expr::Between {
            expr, low, high, ..
        } => {
            check_expr_for_subquery(expr)?;
            check_expr_for_subquery(low)?;
            check_expr_for_subquery(high)?;
        }
        Expr::InList { expr, list, .. } => {
            check_expr_for_subquery(expr)?;
            for item in list {
                check_expr_for_subquery(item)?;
            }
        }
        Expr::Case {
            operand,
            conditions,
            results,
            else_result,
        } => {
            if let Some(op) = operand {
                check_expr_for_subquery(op)?;
            }
            for cond in conditions {
                check_expr_for_subquery(cond)?;
            }
            for res in results {
                check_expr_for_subquery(res)?;
            }
            if let Some(else_res) = else_result {
                check_expr_for_subquery(else_res)?;
            }
        }
        Expr::Function(func) => {
            for arg in &func.args {
                if let FunctionArg::Unnamed(FunctionArgExpr::Expr(e)) = arg {
                    check_expr_for_subquery(e)?;
                }
            }
        }
        Expr::Nested(inner) => {
            check_expr_for_subquery(inner)?;
        }
        // Other expression types don't contain subqueries
        _ => {}
    }
    Ok(())
}

/// Extract simple table name from ObjectName
///
/// Returns None for schema-qualified names (e.g., "schema.table")
/// Returns the table name for simple names (e.g., "users")
fn simple_table_name(name: &sqlparser::ast::ObjectName) -> Option<String> {
    if name.0.len() == 1 {
        // Simple table name without schema qualification
        Some(name.0[0].value.clone())
    } else {
        // Schema-qualified or more complex name - ignore
        // These are likely external database tables, not Lattice variables
        None
    }
}

/// Extract simple table name from a string (for TABLE syntax)
fn simple_object_name(name: &str) -> Option<String> {
    // Check if it's schema-qualified
    if name.contains('.') {
        None
    } else {
        Some(name.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_select() {
        let tables = extract_table_references("SELECT * FROM users").unwrap();
        assert_eq!(tables, vec!["users"]);
    }

    #[test]
    fn test_select_with_alias() {
        let tables = extract_table_references("SELECT * FROM users u").unwrap();
        assert_eq!(tables, vec!["users"]);
    }

    #[test]
    fn test_select_with_multiple_columns() {
        let tables =
            extract_table_references("SELECT id, name, age FROM employees WHERE age > 21").unwrap();
        assert_eq!(tables, vec!["employees"]);
    }

    #[test]
    fn test_join() {
        let tables = extract_table_references(
            "SELECT * FROM users JOIN orders ON users.id = orders.user_id",
        )
        .unwrap();
        assert_eq!(tables, vec!["users", "orders"]);
    }

    #[test]
    fn test_multiple_joins() {
        let tables = extract_table_references(
            "SELECT * FROM users u \
             JOIN orders o ON u.id = o.user_id \
             JOIN products p ON o.product_id = p.id",
        )
        .unwrap();
        assert_eq!(tables, vec!["users", "orders", "products"]);
    }

    #[test]
    fn test_left_join() {
        let tables = extract_table_references(
            "SELECT * FROM users LEFT JOIN orders ON users.id = orders.user_id",
        )
        .unwrap();
        assert_eq!(tables, vec!["users", "orders"]);
    }

    #[test]
    fn test_cross_join() {
        let tables = extract_table_references("SELECT * FROM users CROSS JOIN products").unwrap();
        assert_eq!(tables, vec!["users", "products"]);
    }

    #[test]
    fn test_union() {
        let tables =
            extract_table_references("SELECT id FROM users UNION SELECT id FROM admins").unwrap();
        assert_eq!(tables, vec!["users", "admins"]);
    }

    #[test]
    fn test_duplicate_tables_removed() {
        let tables = extract_table_references(
            "SELECT * FROM users WHERE id IN (SELECT user_id FROM users)",
        );
        // This should error due to subquery, but if it didn't, duplicates would be removed
        assert!(tables.is_err());
    }

    #[test]
    fn test_case_sensitivity() {
        let tables = extract_table_references("SELECT * FROM Users").unwrap();
        assert_eq!(tables, vec!["Users"]);
    }

    #[test]
    fn test_quoted_identifier() {
        let tables = extract_table_references("SELECT * FROM \"user-data\"").unwrap();
        assert_eq!(tables, vec!["user-data"]);
    }

    #[test]
    fn test_cte_rejected() {
        let result =
            extract_table_references("WITH cte AS (SELECT * FROM users) SELECT * FROM cte");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("CTE"));
    }

    #[test]
    fn test_subquery_in_where_rejected() {
        let result = extract_table_references(
            "SELECT * FROM users WHERE id IN (SELECT user_id FROM orders)",
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("subquer"));
    }

    #[test]
    fn test_subquery_in_from_rejected() {
        let result = extract_table_references("SELECT * FROM (SELECT * FROM users) AS subq");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("subquer"));
    }

    #[test]
    fn test_exists_subquery_rejected() {
        let result = extract_table_references(
            "SELECT * FROM users WHERE EXISTS (SELECT 1 FROM orders WHERE orders.user_id = users.id)",
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("EXISTS"));
    }

    #[test]
    fn test_schema_qualified_ignored() {
        // Schema-qualified names should be ignored (they're likely external tables)
        let tables = extract_table_references(
            "SELECT * FROM users JOIN schema.external_table ON users.id = schema.external_table.user_id",
        ).unwrap();
        assert_eq!(tables, vec!["users"]);
    }

    #[test]
    fn test_table_function_ignored() {
        // Table functions like read_csv should be ignored
        let tables = extract_table_references("SELECT * FROM read_csv('file.csv')").unwrap();
        assert!(tables.is_empty());
    }

    #[test]
    fn test_empty_query() {
        let tables = extract_table_references("SELECT 1").unwrap();
        assert!(tables.is_empty());
    }

    #[test]
    fn test_syntax_error() {
        let result = extract_table_references("SELECT * FORM users");
        assert!(result.is_err());
    }

    #[test]
    fn test_insert_statement() {
        let tables = extract_table_references(
            "INSERT INTO users (id, name) SELECT id, name FROM temp_users",
        )
        .unwrap();
        assert_eq!(tables, vec!["users", "temp_users"]);
    }

    #[test]
    fn test_update_statement() {
        let tables = extract_table_references(
            "UPDATE users SET name = 'test' FROM orders WHERE users.id = orders.user_id",
        )
        .unwrap();
        assert_eq!(tables, vec!["users", "orders"]);
    }

    #[test]
    fn test_delete_statement() {
        let tables = extract_table_references("DELETE FROM users WHERE id = 1").unwrap();
        assert_eq!(tables, vec!["users"]);
    }

    #[test]
    fn test_complex_where_clause() {
        let tables = extract_table_references(
            "SELECT * FROM users WHERE age > 21 AND name LIKE '%test%' OR active = true",
        )
        .unwrap();
        assert_eq!(tables, vec!["users"]);
    }

    #[test]
    fn test_group_by_having() {
        let tables = extract_table_references(
            "SELECT department, COUNT(*) FROM employees GROUP BY department HAVING COUNT(*) > 5",
        )
        .unwrap();
        assert_eq!(tables, vec!["employees"]);
    }

    #[test]
    fn test_order_by_limit() {
        let tables =
            extract_table_references("SELECT * FROM users ORDER BY name LIMIT 10 OFFSET 5")
                .unwrap();
        assert_eq!(tables, vec!["users"]);
    }
}
