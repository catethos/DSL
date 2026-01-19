//! Integration tests for SQL queries on Lattice data structures
//!
//! These tests verify that SQL queries can reference Lattice variables
//! and query them as if they were database tables.
//!
//! Requires the `sql-arrow` feature to be enabled.

#![cfg(feature = "sql-arrow")]

use lattice::runtime::{LatticeRuntime, LatticeValue, RuntimeBuilder};

// ============================================================
// Helper functions
// ============================================================

/// Create a runtime with SQL support enabled
fn create_sql_runtime() -> LatticeRuntime {
    LatticeRuntime::from_built(
        RuntimeBuilder::new()
            .without_llm()
            .with_default_sql_provider()
            .expect("Failed to create SQL provider")
            .build()
            .expect("Failed to build runtime"),
    )
}

/// Helper to get a value from a LatticeValue::Map by key
fn map_get<'a>(map: &'a [(String, LatticeValue)], key: &str) -> Option<&'a LatticeValue> {
    map.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

/// Helper to check if a map contains a key
fn map_has_key(map: &[(String, LatticeValue)], key: &str) -> bool {
    map.iter().any(|(k, _)| k == key)
}

// ============================================================
// Basic SQL on Lattice Data Tests
// ============================================================

#[test]
fn test_sql_on_lattice_list() {
    let mut runtime = create_sql_runtime();

    // Create a Lattice variable with list of maps
    runtime
        .eval(
            r#"
        let users = [
            {id: 1, name: "Alice", age: 30},
            {id: 2, name: "Bob", age: 18},
            {id: 3, name: "Charlie", age: 25}
        ]
    "#,
        )
        .expect("Failed to create users variable");

    // Query the Lattice variable using SQL
    let result = runtime
        .eval(r#"SQL("SELECT * FROM users WHERE age > 21 ORDER BY id")"#)
        .expect("SQL query should succeed");

    // Should return Alice (30) and Charlie (25)
    if let LatticeValue::List(rows) = result {
        assert_eq!(rows.len(), 2, "Expected 2 rows (Alice and Charlie)");

        // Check first row is Alice
        if let LatticeValue::Map(row) = &rows[0] {
            assert_eq!(map_get(row,"name"), Some(&LatticeValue::String("Alice".into())));
        } else {
            panic!("Expected Map row");
        }

        // Check second row is Charlie
        if let LatticeValue::Map(row) = &rows[1] {
            assert_eq!(
                map_get(row,"name"),
                Some(&LatticeValue::String("Charlie".into()))
            );
        } else {
            panic!("Expected Map row");
        }
    } else {
        panic!("Expected List result, got {:?}", result);
    }
}

#[test]
fn test_sql_select_specific_columns() {
    let mut runtime = create_sql_runtime();

    runtime
        .eval(
            r#"
        let products = [
            {id: 1, name: "Widget", price: 9.99, stock: 100},
            {id: 2, name: "Gadget", price: 19.99, stock: 50}
        ]
    "#,
        )
        .expect("Failed to create products");

    let result = runtime
        .eval(r#"SQL("SELECT name, price FROM products ORDER BY id")"#)
        .expect("SQL query should succeed");

    if let LatticeValue::List(rows) = result {
        assert_eq!(rows.len(), 2);
        if let LatticeValue::Map(row) = &rows[0] {
            assert!(map_has_key(row,"name"));
            assert!(map_has_key(row,"price"));
            // stock should not be included
            assert!(!map_has_key(row,"stock"));
        }
    } else {
        panic!("Expected List result");
    }
}

#[test]
fn test_sql_aggregate_functions() {
    let mut runtime = create_sql_runtime();

    runtime
        .eval(
            r#"
        let sales = [
            {product: "A", amount: 100},
            {product: "A", amount: 150},
            {product: "B", amount: 200}
        ]
    "#,
        )
        .expect("Failed to create sales");

    let result = runtime
        .eval(r#"SQL("SELECT SUM(amount) as total FROM sales")"#)
        .expect("SQL query should succeed");

    if let LatticeValue::List(rows) = result {
        assert_eq!(rows.len(), 1);
        if let LatticeValue::Map(row) = &rows[0] {
            // Total should be 450
            let total = map_get(row,"total");
            assert!(
                matches!(total, Some(LatticeValue::Int(450))),
                "Expected total=450, got {:?}",
                total
            );
        }
    } else {
        panic!("Expected List result");
    }
}

#[test]
fn test_sql_group_by() {
    let mut runtime = create_sql_runtime();

    runtime
        .eval(
            r#"
        let orders = [
            {customer: "Alice", amount: 100},
            {customer: "Bob", amount: 200},
            {customer: "Alice", amount: 150}
        ]
    "#,
        )
        .expect("Failed to create orders");

    let result = runtime
        .eval(
            r#"SQL("SELECT customer, SUM(amount) as total FROM orders GROUP BY customer ORDER BY customer")"#,
        )
        .expect("SQL query should succeed");

    if let LatticeValue::List(rows) = result {
        assert_eq!(rows.len(), 2);

        // Alice's total should be 250
        if let LatticeValue::Map(row) = &rows[0] {
            assert_eq!(
                map_get(row,"customer"),
                Some(&LatticeValue::String("Alice".into()))
            );
            assert_eq!(map_get(row,"total"), Some(&LatticeValue::Int(250)));
        }

        // Bob's total should be 200
        if let LatticeValue::Map(row) = &rows[1] {
            assert_eq!(
                map_get(row,"customer"),
                Some(&LatticeValue::String("Bob".into()))
            );
            assert_eq!(map_get(row,"total"), Some(&LatticeValue::Int(200)));
        }
    } else {
        panic!("Expected List result");
    }
}

// ============================================================
// Error Handling Tests
// ============================================================

#[test]
fn test_sql_rejects_non_list() {
    let mut runtime = create_sql_runtime();

    // Create a non-list variable
    runtime
        .eval(r#"let not_a_list = "hello""#)
        .expect("Failed to create variable");

    // SQL query should fail because variable is not a List
    let result = runtime.eval(r#"SQL("SELECT * FROM not_a_list")"#);

    assert!(result.is_err(), "Should reject non-List variable");
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("wrong type")
            || err.to_string().contains("Wrong")
            || err.to_string().contains("List"),
        "Error should mention type mismatch: {}",
        err
    );
}

#[test]
fn test_sql_table_not_found() {
    let mut runtime = create_sql_runtime();

    // Query a non-existent table
    let result = runtime.eval(r#"SQL("SELECT * FROM nonexistent_table")"#);

    assert!(result.is_err(), "Should fail for non-existent table");
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("not found")
            || err.to_string().contains("Not found")
            || err.to_string().contains("nonexistent_table"),
        "Error should mention table not found: {}",
        err
    );
}

#[test]
fn test_sql_syntax_error() {
    let mut runtime = create_sql_runtime();

    // Invalid SQL syntax
    let result = runtime.eval(r#"SQL("SELEKT * FORM users")"#);

    assert!(result.is_err(), "Should fail for invalid SQL");
}

// ============================================================
// Empty Data Tests
// ============================================================

#[test]
fn test_sql_empty_list() {
    let mut runtime = create_sql_runtime();

    runtime
        .eval(r#"let empty_list = []"#)
        .expect("Failed to create empty list");

    // Empty list has no schema to infer - DuckDB requires at least one column
    // This should fail with an appropriate error
    let result = runtime.eval(r#"SQL("SELECT * FROM empty_list")"#);

    assert!(
        result.is_err(),
        "SQL on empty list should fail (no schema to infer)"
    );
}

// ============================================================
// Multiple Table Tests
// ============================================================

#[test]
fn test_sql_join_lattice_tables() {
    let mut runtime = create_sql_runtime();

    runtime
        .eval(
            r#"
        let customers = [
            {id: 1, name: "Alice"},
            {id: 2, name: "Bob"}
        ]
        let orders = [
            {customer_id: 1, product: "Widget", amount: 100},
            {customer_id: 1, product: "Gadget", amount: 50},
            {customer_id: 2, product: "Widget", amount: 200}
        ]
    "#,
        )
        .expect("Failed to create tables");

    let result = runtime
        .eval(
            r#"SQL("SELECT c.name, SUM(o.amount) as total FROM customers c JOIN orders o ON c.id = o.customer_id GROUP BY c.name ORDER BY c.name")"#,
        )
        .expect("JOIN query should succeed");

    if let LatticeValue::List(rows) = result {
        assert_eq!(rows.len(), 2);

        // Alice: 100 + 50 = 150
        if let LatticeValue::Map(row) = &rows[0] {
            assert_eq!(map_get(row,"name"), Some(&LatticeValue::String("Alice".into())));
            assert_eq!(map_get(row,"total"), Some(&LatticeValue::Int(150)));
        }

        // Bob: 200
        if let LatticeValue::Map(row) = &rows[1] {
            assert_eq!(map_get(row,"name"), Some(&LatticeValue::String("Bob".into())));
            assert_eq!(map_get(row,"total"), Some(&LatticeValue::Int(200)));
        }
    } else {
        panic!("Expected List result");
    }
}

// ============================================================
// Type Handling Tests
// ============================================================

#[test]
fn test_sql_handles_null_values() {
    let mut runtime = create_sql_runtime();

    runtime
        .eval(
            r#"
        let data = [
            {id: 1, value: 100},
            {id: 2, value: null},
            {id: 3, value: 300}
        ]
    "#,
        )
        .expect("Failed to create data with nulls");

    let result = runtime
        .eval(r#"SQL("SELECT * FROM data WHERE value IS NOT NULL ORDER BY id")"#)
        .expect("SQL query should handle nulls");

    if let LatticeValue::List(rows) = result {
        assert_eq!(rows.len(), 2, "Should filter out null row");
    } else {
        panic!("Expected List result");
    }
}

#[test]
fn test_sql_handles_mixed_types_in_column() {
    let mut runtime = create_sql_runtime();

    // Create data with int and float in same column
    runtime
        .eval(
            r#"
        let mixed = [
            {id: 1, value: 100},
            {id: 2, value: 3.14}
        ]
    "#,
        )
        .expect("Failed to create mixed type data");

    // This should work - int gets promoted to float
    let result = runtime.eval(r#"SQL("SELECT * FROM mixed ORDER BY id")"#);

    // The query should succeed (types get promoted)
    assert!(
        result.is_ok(),
        "Should handle int/float promotion: {:?}",
        result.err()
    );
}

// ============================================================
// Cleanup Tests
// ============================================================

#[test]
fn test_sql_cleanup_after_query() {
    let mut runtime = create_sql_runtime();

    runtime
        .eval(
            r#"
        let temp_data = [
            {id: 1, name: "test"}
        ]
    "#,
        )
        .expect("Failed to create temp_data");

    // First query should work
    runtime
        .eval(r#"SQL("SELECT * FROM temp_data")"#)
        .expect("First query should succeed");

    // Modify the variable
    runtime
        .eval(
            r#"
        let temp_data = [
            {id: 2, name: "modified"}
        ]
    "#,
        )
        .expect("Failed to modify temp_data");

    // Second query should see the new data (table was cleaned up)
    let result = runtime
        .eval(r#"SQL("SELECT * FROM temp_data")"#)
        .expect("Second query should succeed");

    if let LatticeValue::List(rows) = result {
        assert_eq!(rows.len(), 1);
        if let LatticeValue::Map(row) = &rows[0] {
            assert_eq!(map_get(row,"id"), Some(&LatticeValue::Int(2)));
            assert_eq!(
                map_get(row,"name"),
                Some(&LatticeValue::String("modified".into()))
            );
        }
    } else {
        panic!("Expected List result");
    }
}
