use dsl_interpreter::{Interpreter};
use dsl_ir::Value;
use indexmap::IndexMap;

#[tokio::test]
async fn test_sql_variable_auto_registration() {
    // Create interpreter
    let mut interpreter = Interpreter::new().unwrap();
    
    // Create test data
    let mut map1 = IndexMap::new();
    map1.insert("name".to_string(), Value::String("Alice".to_string()));
    map1.insert("age".to_string(), Value::Int(25));
    
    let mut map2 = IndexMap::new();
    map2.insert("name".to_string(), Value::String("Bob".to_string()));
    map2.insert("age".to_string(), Value::Int(30));
    
    let mut map3 = IndexMap::new();
    map3.insert("name".to_string(), Value::String("Charlie".to_string()));
    map3.insert("age".to_string(), Value::Int(35));
    
    let users = Value::List(vec![
        Value::Map(map1),
        Value::Map(map2),
        Value::Map(map3),
    ]);
    
    // Set the variable in scope
    interpreter.runtime.set_var("users".to_string(), users);
    
    // Test SQL with $variable syntax
    let query = Value::String("SELECT * FROM $users WHERE age > 28".to_string());
    let result = interpreter.builtins.call_intrinsic_with_values(
        "__sql",
        &[query],
        None,
        None,
        |var_name| interpreter.runtime.get_var(var_name).ok(),
    ).await;
    
    assert!(result.is_ok(), "SQL query failed: {:?}", result);
    
    let result_value = result.unwrap();
    if let Value::List(rows) = result_value {
        println!("Result rows: {:?}", rows);
        assert_eq!(rows.len(), 2, "Expected 2 rows (Bob and Charlie)");
    } else {
        panic!("Expected List result, got: {:?}", result_value);
    }
}

#[tokio::test]
async fn test_sql_variable_not_found() {
    let mut interpreter = Interpreter::new().unwrap();
    
    // Try to query without setting the variable
    let query = Value::String("SELECT * FROM $users".to_string());
    let result = interpreter.builtins.call_intrinsic_with_values(
        "__sql",
        &[query],
        None,
        None,
        |var_name| interpreter.runtime.get_var(var_name).ok(),
    ).await;
    
    assert!(result.is_err(), "Expected error when variable not found");
    let err = result.unwrap_err();
    assert!(format!("{:?}", err).contains("not found"), "Error should mention variable not found: {:?}", err);
}

#[tokio::test]
async fn test_sql_invalid_schema() {
    let mut interpreter = Interpreter::new().unwrap();
    
    // Create data with inconsistent schema
    let mut map1 = IndexMap::new();
    map1.insert("name".to_string(), Value::String("Alice".to_string()));
    map1.insert("age".to_string(), Value::Int(25));
    
    let mut map2 = IndexMap::new();
    map2.insert("name".to_string(), Value::String("Bob".to_string()));
    // Missing "age" field
    
    let invalid_data = Value::List(vec![
        Value::Map(map1),
        Value::Map(map2),
    ]);
    
    interpreter.runtime.set_var("data".to_string(), invalid_data);
    
    let query = Value::String("SELECT * FROM $data".to_string());
    let result = interpreter.builtins.call_intrinsic_with_values(
        "__sql",
        &[query],
        None,
        None,
        |var_name| interpreter.runtime.get_var(var_name).ok(),
    ).await;
    
    assert!(result.is_err(), "Expected error with inconsistent schema");
    let err = result.unwrap_err();
    assert!(format!("{:?}", err).contains("Inconsistent schema"), "Error should mention inconsistent schema: {:?}", err);
}

#[tokio::test]
async fn test_sql_nested_structures() {
    let mut interpreter = Interpreter::new().unwrap();
    
    // Create data with nested structures
    let mut inner_map = IndexMap::new();
    inner_map.insert("street".to_string(), Value::String("Main St".to_string()));
    
    let mut map1 = IndexMap::new();
    map1.insert("name".to_string(), Value::String("Alice".to_string()));
    map1.insert("address".to_string(), Value::Map(inner_map));  // Nested map
    
    let nested_data = Value::List(vec![Value::Map(map1)]);
    
    interpreter.runtime.set_var("nested".to_string(), nested_data);
    
    let query = Value::String("SELECT * FROM $nested".to_string());
    let result = interpreter.builtins.call_intrinsic_with_values(
        "__sql",
        &[query],
        None,
        None,
        |var_name| interpreter.runtime.get_var(var_name).ok(),
    ).await;
    
    assert!(result.is_err(), "Expected error with nested structures");
    let err = result.unwrap_err();
    assert!(format!("{:?}", err).contains("nested"), "Error should mention nested structure: {:?}", err);
}

#[tokio::test]
async fn test_sql_array_literal() {
    let mut interpreter = Interpreter::new().unwrap();
    
    // Use $variable with a list of primitives
    let numbers = Value::List(vec![
        Value::Int(1),
        Value::Int(2),
        Value::Int(3),
    ]);
    interpreter.runtime.set_var("numbers".to_string(), numbers);
    
    // Query using array literal
    let query = Value::String("SELECT unnest($numbers) as value".to_string());
    let result = interpreter.builtins.call_intrinsic_with_values(
        "__sql",
        &[query],
        None,
        None,
        |var_name| interpreter.runtime.get_var(var_name).ok(),
    ).await;
    
    assert!(result.is_ok(), "Array literal query failed: {:?}", result);
    
    if let Value::List(rows) = result.unwrap() {
        assert_eq!(rows.len(), 3, "Should return 3 rows");
    }
}

#[tokio::test]
async fn test_sql_primitive_values() {
    let mut interpreter = Interpreter::new().unwrap();
    
    // Set various primitive values
    interpreter.runtime.set_var("x".to_string(), Value::Int(42));
    interpreter.runtime.set_var("name".to_string(), Value::String("Alice".to_string()));
    interpreter.runtime.set_var("pi".to_string(), Value::Float(3.14));
    
    // Query using primitive literals - cast to appropriate types
    let query = Value::String("SELECT CAST($x AS INTEGER) as num, CAST($name AS VARCHAR) as text, CAST($pi AS DOUBLE) as f".to_string());
    let result = interpreter.builtins.call_intrinsic_with_values(
        "__sql",
        &[query],
        None,
        None,
        |var_name| interpreter.runtime.get_var(var_name).ok(),
    ).await;
    
    assert!(result.is_ok(), "Primitive literal query failed: {:?}", result);
    
    if let Value::List(rows) = result.unwrap() {
        assert_eq!(rows.len(), 1, "Should return 1 row");
        if let Value::Map(row) = &rows[0] {
            assert_eq!(row.get("num"), Some(&Value::Int(42)));
            assert_eq!(row.get("text"), Some(&Value::String("Alice".to_string())));
        }
    }
}

#[tokio::test]
async fn test_sql_struct_literal() {
    use dsl_interpreter::SQLExecutor;
    
    let mut executor = SQLExecutor::new().unwrap();
    
    // Create a struct/map value
    let mut person = IndexMap::new();
    person.insert("name".to_string(), Value::String("Alice".to_string()));
    person.insert("age".to_string(), Value::Int(25));
    
    let person_val = Value::Map(person);
    
    // Use it in a query
    let query = executor.prepare_query_with_variables(
        "SELECT $person.name as name, $person.age as age",
        |var_name| {
            if var_name == "person" {
                Some(person_val.clone())
            } else {
                None
            }
        }
    ).unwrap();
    
    // Should interpolate as struct literal
    assert!(query.contains("{'name':"), "Query should contain struct literal: {}", query);
}

#[tokio::test]
async fn test_sql_nested_array() {
    let mut interpreter = Interpreter::new().unwrap();
    
    // Create nested list
    let nested = Value::List(vec![
        Value::List(vec![Value::Int(1), Value::Int(2)]),
        Value::List(vec![Value::Int(3), Value::Int(4)]),
    ]);
    interpreter.runtime.set_var("nested".to_string(), nested);
    
    // Query using nested array
    let query = Value::String("SELECT $nested as arr".to_string());
    let result = interpreter.builtins.call_intrinsic_with_values(
        "__sql",
        &[query],
        None,
        None,
        |var_name| interpreter.runtime.get_var(var_name).ok(),
    ).await;
    
    assert!(result.is_ok(), "Nested array query failed: {:?}", result);
}

#[tokio::test]
async fn test_sql_table_caching() {
    let mut interpreter = Interpreter::new().unwrap();
    
    // Create test data
    let mut map1 = IndexMap::new();
    map1.insert("name".to_string(), Value::String("Alice".to_string()));
    map1.insert("age".to_string(), Value::Int(25));
    
    let users = Value::List(vec![Value::Map(map1)]);
    interpreter.runtime.set_var("users".to_string(), users);
    
    // First query - should register table
    let query1 = Value::String("SELECT COUNT(*) as count FROM $users".to_string());
    let result1 = interpreter.builtins.call_intrinsic_with_values(
        "__sql",
        &[query1],
        None,
        None,
        |var_name| interpreter.runtime.get_var(var_name).ok(),
    ).await;
    assert!(result1.is_ok());
    
    // Update the users variable with new data
    let mut map2 = IndexMap::new();
    map2.insert("name".to_string(), Value::String("Bob".to_string()));
    map2.insert("age".to_string(), Value::Int(30));
    
    let new_users = Value::List(vec![Value::Map(map2)]);
    interpreter.runtime.set_var("users".to_string(), new_users);
    
    // Second query - should use cached table (still sees old data with 1 row)
    let query2 = Value::String("SELECT COUNT(*) as count FROM $users".to_string());
    let result2 = interpreter.builtins.call_intrinsic_with_values(
        "__sql",
        &[query2],
        None,
        None,
        |var_name| interpreter.runtime.get_var(var_name).ok(),
    ).await;
    assert!(result2.is_ok());
    
    // Verify it's using cached data (should still be 1 row, not the new data)
    if let Value::List(rows) = result2.unwrap() {
        assert_eq!(rows.len(), 1, "Should return 1 result row");
        if let Value::Map(row) = &rows[0] {
            if let Some(Value::Int(count)) = row.get("count") {
                assert_eq!(*count, 1, "Cached table should still have 1 row");
            }
        }
    }
}

#[tokio::test]
async fn test_sql_clear_table() {
    use dsl_interpreter::SQLExecutor;
    
    let mut executor = SQLExecutor::new().unwrap();
    
    // Create and register a table
    let mut map1 = IndexMap::new();
    map1.insert("name".to_string(), Value::String("Alice".to_string()));
    map1.insert("age".to_string(), Value::Int(25));
    
    let users = Value::List(vec![Value::Map(map1)]);
    
    // Register using prepare_query_with_variables
    let _query = executor.prepare_query_with_variables(
        "SELECT * FROM $users",
        |_| Some(users.clone())
    ).unwrap();
    
    // Clear the table
    let clear_result = executor.clear_table("users");
    assert!(clear_result.is_ok(), "Should clear registered table");
    
    // Try to clear again - should fail since it's no longer registered
    let clear_again = executor.clear_table("users");
    assert!(clear_again.is_err(), "Should fail to clear non-registered table");
}

#[tokio::test]
async fn test_sql_clear_all_tables() {
    use dsl_interpreter::SQLExecutor;
    
    let mut executor = SQLExecutor::new().unwrap();
    
    // Create data
    let mut map1 = IndexMap::new();
    map1.insert("id".to_string(), Value::Int(1));
    let data = Value::List(vec![Value::Map(map1)]);
    
    // Register multiple tables
    executor.prepare_query_with_variables(
        "SELECT * FROM $users, $orders, $products",
        |name| match name {
            "users" | "orders" | "products" => Some(data.clone()),
            _ => None,
        }
    ).unwrap();
    
    // Clear all tables
    let clear_result = executor.clear_all_tables();
    assert!(clear_result.is_ok(), "Should clear all tables");
    
    // Verify tables need re-registration
    let requery = executor.prepare_query_with_variables(
        "SELECT * FROM $users",
        |_| Some(data.clone())
    );
    assert!(requery.is_ok(), "Should re-register after clear");
}
