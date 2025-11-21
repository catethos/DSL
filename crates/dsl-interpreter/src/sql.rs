use dsl_ir::Value;
use duckdb::Connection;
use indexmap::IndexMap;
use regex::Regex;
use std::collections::{HashMap, HashSet};

pub struct SQLExecutor {
    conn: Connection,
    registered_tables: HashSet<String>,
}

impl SQLExecutor {
    pub fn new() -> Result<Self, String> {
        let conn = Connection::open_in_memory()
            .map_err(|e| format!("Failed to create DuckDB connection: {}", e))?;

        Ok(Self {
            conn,
            registered_tables: HashSet::new(),
        })
    }

    /// Extract variable names from SQL query with $variable syntax
    pub fn extract_variable_names(sql: &str) -> Vec<String> {
        let re = Regex::new(r"\$([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
        re.captures_iter(sql)
            .map(|cap| cap[1].to_string())
            .collect()
    }

    /// Execute SQL query with auto-registration of $variables
    /// - List<Map> → Registers as table (with caching)
    /// - Other types → Interpolates as DuckDB literal
    pub fn prepare_query_with_variables(
        &mut self,
        sql: &str,
        scope_lookup: impl Fn(&str) -> Option<Value>,
    ) -> Result<String, String> {
        let var_names = Self::extract_variable_names(sql);
        let mut query = sql.to_string();

        for var_name in var_names {
            // Look up variable in scope
            let value = scope_lookup(&var_name).ok_or_else(|| {
                format!("Variable '{}' not found in scope", var_name)
            })?;

            let placeholder = format!("${}", var_name);
            
            // Check if it's a List<Map> that should be registered as table
            if Self::is_table_data(&value) {
                // Only register if not already registered (optimization)
                if !self.registered_tables.contains(&var_name) {
                    self.register_table(&var_name, &value)?;
                    self.registered_tables.insert(var_name.clone());
                }
                // Replace $variable with table name
                query = query.replace(&placeholder, &var_name);
            } else {
                // For other types, interpolate as DuckDB literal
                let literal = self.value_to_duckdb_literal(&value);
                query = query.replace(&placeholder, &literal);
            }
        }

        Ok(query)
    }
    
    /// Check if a value should be registered as a table (List of Maps)
    fn is_table_data(value: &Value) -> bool {
        match value {
            Value::List(items) if !items.is_empty() => {
                matches!(items[0], Value::Map(_))
            }
            _ => false,
        }
    }
    
    /// Convert a Value to DuckDB literal syntax
    fn value_to_duckdb_literal(&self, value: &Value) -> String {
        match value {
            // Primitives
            Value::String(s) => format!("'{}'", s.replace("'", "''")),
            Value::Int(n) => n.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "NULL".to_string(),
            
            // List → DuckDB array literal
            Value::List(items) => {
                let elements: Vec<String> = items
                    .iter()
                    .map(|v| self.value_to_duckdb_literal(v))
                    .collect();
                format!("[{}]", elements.join(", "))
            }
            
            // Map → DuckDB struct literal
            Value::Map(map) => {
                let fields: Vec<String> = map
                    .iter()
                    .map(|(k, v)| {
                        format!("'{}': {}", k, self.value_to_duckdb_literal(v))
                    })
                    .collect();
                format!("{{{}}}", fields.join(", "))
            }
            
            // Special types - treat as strings
            Value::Markdown(s) | Value::Image(s) => {
                format!("'{}'", s.replace("'", "''"))
            }
        }
    }

    pub fn execute(&mut self, sql: &str, params: &HashMap<String, Value>) -> Result<Value, String> {
        let mut query = sql.to_string();
        
        // Simple template replacement for {{variable}} syntax
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
                // NOTE: Try f64 BEFORE i64 to avoid truncation of decimal values
                // DuckDB's get::<_, i64>() can silently truncate floats like 0.2667 to 0
                if let Ok(v) = row.get::<_, f64>(i) {
                    // Check if it's actually a whole number (integer)
                    if v.fract() == 0.0 && v >= i64::MIN as f64 && v <= i64::MAX as f64 {
                        map.insert(col_name.clone(), Value::Int(v as i64));
                    } else {
                        map.insert(col_name.clone(), Value::Float(v));
                    }
                } else if let Ok(v) = row.get::<_, i64>(i) {
                    map.insert(col_name.clone(), Value::Int(v));
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
        // 1. Validate it's a List
        let items = match data {
            Value::List(items) => items,
            _ => return Err("Can only register List as table".to_string()),
        };

        if items.is_empty() {
            return Err("Cannot register empty list as table".to_string());
        }

        // 2. Validate all items are Maps
        for item in items {
            if !matches!(item, Value::Map(_)) {
                return Err("All list items must be Maps (records)".to_string());
            }
        }

        // 3. Get first item's schema
        let first_item = &items[0];
        let fields = match first_item {
            Value::Map(m) => m,
            _ => unreachable!(), // Already validated above
        };

        // 4. Validate schema consistency (all maps have same keys)
        let expected_keys: HashSet<_> = fields.keys().collect();
        for (idx, item) in items.iter().enumerate().skip(1) {
            let map = match item {
                Value::Map(m) => m,
                _ => unreachable!(), // Already validated
            };
            let keys: HashSet<_> = map.keys().collect();
            if keys != expected_keys {
                return Err(format!(
                    "Inconsistent schema at row {}: all maps must have same keys. Expected {:?}, got {:?}",
                    idx,
                    expected_keys.iter().collect::<Vec<_>>(),
                    keys.iter().collect::<Vec<_>>()
                ));
            }
        }

        // 5. Validate no nested structures
        for (key, val) in fields.iter() {
            if matches!(val, Value::Map(_) | Value::List(_)) {
                return Err(format!(
                    "Field '{}' contains nested structure - DuckDB only supports flat tables",
                    key
                ));
            }
        }

        // 6. Create table
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

        // 7. Insert data
        let field_names: Vec<String> = fields.keys().cloned().collect();
        for item in items {
            if let Value::Map(row) = item {
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

    /// Clear a registered table, forcing it to be re-registered on next use
    pub fn clear_table(&mut self, name: &str) -> Result<(), String> {
        if self.registered_tables.remove(name) {
            // Drop the table from DuckDB
            let drop_query = format!("DROP TABLE IF EXISTS {}", name);
            self.conn
                .execute(&drop_query, [])
                .map_err(|e| format!("Failed to drop table: {}", e))?;
            Ok(())
        } else {
            Err(format!("Table '{}' not registered", name))
        }
    }

    /// Clear all registered tables
    pub fn clear_all_tables(&mut self) -> Result<(), String> {
        for table_name in self.registered_tables.clone() {
            let drop_query = format!("DROP TABLE IF EXISTS {}", table_name);
            self.conn
                .execute(&drop_query, [])
                .map_err(|e| format!("Failed to drop table {}: {}", table_name, e))?;
        }
        self.registered_tables.clear();
        Ok(())
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
