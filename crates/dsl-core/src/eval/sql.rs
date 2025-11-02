use crate::types::Value;
use duckdb::Connection;
use indexmap::IndexMap;
use std::collections::HashMap;

pub struct SQLExecutor {
    conn: Connection,
}

impl SQLExecutor {
    pub fn new() -> Result<Self, String> {
        let conn = Connection::open_in_memory()
            .map_err(|e| format!("Failed to create DuckDB connection: {}", e))?;

        Ok(Self { conn })
    }

    pub fn execute(&mut self, sql: &str, params: &HashMap<String, Value>) -> Result<Value, String> {
        // Simple template replacement for {{variable}} syntax
        let mut query = sql.to_string();
        for (key, value) in params {
            let placeholder = format!("{{{{{}}}}}", key);
            let replacement = self.value_to_sql_string(value);
            query = query.replace(&placeholder, &replacement);
        }

        // Execute query and get results
        let mut stmt = self
            .conn
            .prepare(&query)
            .map_err(|e| format!("SQL error: {}", e))?;

        // Execute the query first to get access to column metadata
        let mut rows = stmt
            .query([])
            .map_err(|e| format!("Query execution error: {}", e))?;

        // Get column count and names from the first row's metadata
        let column_count = rows.as_ref().map(|r| r.column_count()).unwrap_or(0);
        if column_count == 0 {
            // No results (e.g., CREATE TABLE, INSERT)
            return Ok(Value::Null);
        }

        let column_names: Vec<String> = (0..column_count)
            .map(|i| {
                rows.as_ref()
                    .and_then(|r| r.column_name(i).ok())
                    .map(|s| s.to_string())
                    .unwrap_or_else(String::new)
            })
            .collect();

        // Collect all rows
        let mut result_rows = Vec::new();
        while let Some(row) = rows.next().map_err(|e| format!("Row fetch error: {}", e))? {
            let mut map = IndexMap::new();
            for (i, col_name) in column_names.iter().enumerate() {
                // Try different types
                if let Ok(v) = row.get::<_, i64>(i) {
                    map.insert(col_name.clone(), Value::Int(v));
                } else if let Ok(v) = row.get::<_, f64>(i) {
                    map.insert(col_name.clone(), Value::Float(v));
                } else if let Ok(v) = row.get::<_, String>(i) {
                    map.insert(col_name.clone(), Value::String(v));
                } else if let Ok(v) = row.get::<_, bool>(i) {
                    map.insert(col_name.clone(), Value::Bool(v));
                }
            }
            result_rows.push(map);
        }

        // Convert to list of maps
        let list: Vec<Value> = result_rows.into_iter().map(Value::Map).collect();

        Ok(Value::List(list))
    }

    pub fn register_table(&mut self, name: &str, data: &Value) -> Result<(), String> {
        match data {
            Value::List(items) => {
                if items.is_empty() {
                    return Ok(());
                }

                // Build CREATE TABLE and INSERT statements from the data
                // First, infer schema from the first item
                let first_item = &items[0];
                let fields = match first_item {
                    Value::Map(m) => m,
                    _ => return Err("Table data must be a list of maps".to_string()),
                };

                // Create table
                let mut create_parts = vec![];
                for (field_name, field_value) in fields.iter() {
                    let sql_type = match field_value {
                        Value::Int(_) => "BIGINT",
                        Value::Float(_) => "DOUBLE",
                        Value::Bool(_) => "BOOLEAN",
                        _ => "VARCHAR",
                    };
                    create_parts.push(format!("{} {}", field_name, sql_type));
                }

                let create_query = format!(
                    "CREATE OR REPLACE TABLE {} ({})",
                    name,
                    create_parts.join(", ")
                );

                self.conn
                    .execute(&create_query, [])
                    .map_err(|e| format!("Failed to create table: {}", e))?;

                // Insert data
                let field_names: Vec<String> = fields.keys().cloned().collect();
                for item in items {
                    if let Value::Map(row) = item {
                        // DuckDB params - we'll use a simpler approach with string interpolation
                        // for now since the DuckDB API can be tricky
                        let values: Vec<String> = field_names
                            .iter()
                            .map(|field| {
                                row.get(field)
                                    .map(|v| self.value_to_sql_string(v))
                                    .unwrap_or_else(|| "NULL".to_string())
                            })
                            .collect();

                        let insert_direct = format!(
                            "INSERT INTO {} ({}) VALUES ({})",
                            name,
                            field_names.join(", "),
                            values.join(", ")
                        );

                        self.conn
                            .execute(&insert_direct, [])
                            .map_err(|e| format!("Failed to insert row: {}", e))?;
                    }
                }

                Ok(())
            }
            _ => Err("Can only register lists as tables".to_string()),
        }
    }

    fn value_to_sql_string(&self, value: &Value) -> String {
        match value {
            Value::String(s) => format!("'{}'", s.replace("'", "''")),
            Value::Int(n) => n.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "NULL".to_string(),
            _ => "NULL".to_string(),
        }
    }
}
