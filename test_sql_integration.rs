use dsl_ir::Value;
use dsl_interpreter::Interpreter;
use indexmap::IndexMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create interpreter
    let mut interpreter = Interpreter::new()?;
    
    // Create test data
    let mut map1 = IndexMap::new();
    map1.insert("name".to_string(), Value::String("Alice".to_string()));
    map1.insert("age".to_string(), Value::Int(25));
    map1.insert("city".to_string(), Value::String("NYC".to_string()));
    
    let mut map2 = IndexMap::new();
    map2.insert("name".to_string(), Value::String("Bob".to_string()));
    map2.insert("age".to_string(), Value::Int(30));
    map2.insert("city".to_string(), Value::String("LA".to_string()));
    
    let mut map3 = IndexMap::new();
    map3.insert("name".to_string(), Value::String("Charlie".to_string()));
    map3.insert("age".to_string(), Value::Int(35));
    map3.insert("city".to_string(), Value::String("NYC".to_string()));
    
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
    ).await?;
    
    println!("Result: {:?}", result);
    
    Ok(())
}
