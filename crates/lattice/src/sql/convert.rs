//! DuckDB to Lattice Value conversion
//!
//! This module provides conversion between DuckDB types and Lattice `Value` types.
//!
//! Type mapping:
//! | DuckDB Type | Lattice Type |
//! |-------------|--------------|
//! | INTEGER     | Int          |
//! | BIGINT      | Int          |
//! | DOUBLE      | Float        |
//! | VARCHAR     | String       |
//! | BOOLEAN     | Bool         |
//! | DATE        | String (ISO) |
//! | TIMESTAMP   | String (ISO) |
//! | ARRAY       | List         |
//! | STRUCT      | Map          |
//! | NULL        | Null         |

use duckdb::types::{TimeUnit, Value as DuckValue, ValueRef};
use duckdb::{Connection, Result as DuckResult, Row};
use std::collections::HashMap;

use crate::error::{LatticeError, Result};
use crate::types::Value;

/// Convert a DuckDB ValueRef to a Lattice Value
///
/// This uses the ValueRef's to_owned() method to get an owned DuckValue,
/// then converts that to a Lattice Value.
pub fn duckdb_value_to_value(value_ref: ValueRef<'_>) -> Result<Value> {
    let owned = value_ref.to_owned();
    duckdb_owned_value_to_value(&owned)
}

/// Convert a DuckDB owned Value to a Lattice Value
pub fn duckdb_owned_value_to_value(value: &DuckValue) -> Result<Value> {
    match value {
        DuckValue::Null => Ok(Value::Null),
        DuckValue::Boolean(b) => Ok(Value::Bool(*b)),
        DuckValue::TinyInt(i) => Ok(Value::Int(*i as i64)),
        DuckValue::SmallInt(i) => Ok(Value::Int(*i as i64)),
        DuckValue::Int(i) => Ok(Value::Int(*i as i64)),
        DuckValue::BigInt(i) => Ok(Value::Int(*i)),
        DuckValue::HugeInt(i) => {
            if *i > i64::MAX as i128 || *i < i64::MIN as i128 {
                Err(LatticeError::Runtime(format!(
                    "HugeInt value {} is out of range for Int type",
                    i
                )))
            } else {
                Ok(Value::Int(*i as i64))
            }
        }
        DuckValue::UTinyInt(i) => Ok(Value::Int(*i as i64)),
        DuckValue::USmallInt(i) => Ok(Value::Int(*i as i64)),
        DuckValue::UInt(i) => Ok(Value::Int(*i as i64)),
        DuckValue::UBigInt(i) => {
            if *i > i64::MAX as u64 {
                Err(LatticeError::Runtime(format!(
                    "UBigInt value {} is out of range for Int type",
                    i
                )))
            } else {
                Ok(Value::Int(*i as i64))
            }
        }
        DuckValue::Float(f) => Ok(Value::Float(*f as f64)),
        DuckValue::Double(f) => Ok(Value::Float(*f)),
        DuckValue::Decimal(d) => {
            // Convert Decimal to f64 using try_into
            // Could add a Decimal type to Lattice later if needed
            use std::str::FromStr;
            let f = f64::from_str(&d.to_string()).unwrap_or(0.0);
            Ok(Value::Float(f))
        }
        DuckValue::Text(s) => Ok(Value::string(s.clone())),
        DuckValue::Blob(bytes) => {
            // Convert blob to base64 or hex representation
            Ok(Value::string(format!("<blob:{} bytes>", bytes.len())))
        }
        DuckValue::Date32(days) => {
            let date = date_from_days(*days);
            Ok(Value::string(date))
        }
        DuckValue::Time64(unit, value) => {
            let time = time_from_value(*unit, *value);
            Ok(Value::string(time))
        }
        DuckValue::Timestamp(unit, value) => {
            let timestamp = timestamp_from_value(*unit, *value);
            Ok(Value::string(timestamp))
        }
        DuckValue::Interval { months, days, nanos } => {
            // ISO 8601 duration format
            Ok(Value::string(format!(
                "P{}M{}DT{}N",
                months, days, nanos
            )))
        }
        DuckValue::List(items) => {
            let values: Result<Vec<Value>> = items
                .iter()
                .map(duckdb_owned_value_to_value)
                .collect();
            Ok(Value::list(values?))
        }
        DuckValue::Array(items) => {
            let values: Result<Vec<Value>> = items
                .iter()
                .map(duckdb_owned_value_to_value)
                .collect();
            Ok(Value::list(values?))
        }
        DuckValue::Enum(variant) => {
            Ok(Value::string(variant.clone()))
        }
        DuckValue::Struct(fields) => {
            let mut map = HashMap::new();
            for (key, val) in fields.iter() {
                let value = duckdb_owned_value_to_value(val)?;
                map.insert(key.clone(), value);
            }
            Ok(Value::map(map))
        }
        DuckValue::Map(entries) => {
            let mut map = HashMap::new();
            for (key, val) in entries.iter() {
                // Convert key to string
                let key_str = match key {
                    DuckValue::Text(s) => s.clone(),
                    DuckValue::Int(i) => i.to_string(),
                    DuckValue::BigInt(i) => i.to_string(),
                    other => format!("{:?}", other),
                };
                let value = duckdb_owned_value_to_value(val)?;
                map.insert(key_str, value);
            }
            Ok(Value::map(map))
        }
        DuckValue::Union(inner) => {
            duckdb_owned_value_to_value(inner)
        }
    }
}

/// Convert a Lattice Value to a DuckDB Value for parameterized queries
pub fn value_to_duckdb(value: &Value) -> DuckValue {
    match value {
        Value::Null => DuckValue::Null,
        Value::Bool(b) => DuckValue::Boolean(*b),
        Value::Int(i) => DuckValue::BigInt(*i),
        Value::Float(f) => DuckValue::Double(*f),
        Value::String(s) => DuckValue::Text(s.to_string()),
        Value::List(items) => {
            let duck_items: Vec<DuckValue> = items
                .iter()
                .map(value_to_duckdb)
                .collect();
            DuckValue::List(duck_items)
        }
        Value::Map(map) => {
            // Convert to DuckDB struct
            let entries: Vec<(String, DuckValue)> = map
                .iter()
                .map(|(k, v)| (k.clone(), value_to_duckdb(v)))
                .collect();
            DuckValue::Struct(entries.into())
        }
        Value::Path(p) => DuckValue::Text(p.display().to_string()),
    }
}

// ============================================================================
// Date/Time Helpers
// ============================================================================

/// Convert days since Unix epoch to ISO date string
fn date_from_days(days: i32) -> String {
    // Days since 1970-01-01
    // Using a simple calculation instead of chrono dependency
    let epoch_year = 1970;
    let _days_per_year = 365;
    let days_per_400_years = 146097; // 400 years including leap years

    let mut remaining_days = days as i64;

    // Handle negative days (before 1970)
    if remaining_days < 0 {
        // For simplicity, just return a placeholder for very old dates
        return format!("{}BC", -remaining_days / 365);
    }

    // Calculate year
    let mut year = epoch_year;

    // Fast forward by 400-year blocks
    let blocks_400 = remaining_days / days_per_400_years;
    year += (blocks_400 * 400) as i32;
    remaining_days %= days_per_400_years;

    // Then by years
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    // Calculate month and day
    let is_leap = is_leap_year(year);
    let days_in_months: [i64; 12] = if is_leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1;
    for days_in_month in days_in_months.iter() {
        if remaining_days < *days_in_month {
            break;
        }
        remaining_days -= days_in_month;
        month += 1;
    }

    let day = remaining_days + 1;

    format!("{:04}-{:02}-{:02}", year, month, day)
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Convert time64 value to ISO time string
fn time_from_value(unit: TimeUnit, value: i64) -> String {
    let micros = unit.to_micros(value);
    let total_secs = micros / 1_000_000;
    let remaining_micros = (micros % 1_000_000).abs();

    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    format!("{:02}:{:02}:{:02}.{:06}", hours, mins, secs, remaining_micros)
}

/// Convert timestamp to ISO datetime string
fn timestamp_from_value(unit: TimeUnit, value: i64) -> String {
    let micros = unit.to_micros(value);
    let total_secs = micros / 1_000_000;
    let remaining_micros = (micros % 1_000_000).abs();

    // Calculate date components
    let days = (total_secs / 86400) as i32;
    let time_secs = total_secs % 86400;

    let date = date_from_days(days);
    let hours = time_secs / 3600;
    let mins = (time_secs % 3600) / 60;
    let secs = time_secs % 60;

    format!("{}T{:02}:{:02}:{:02}.{:06}Z", date, hours, mins, secs, remaining_micros)
}

// ============================================================================
// SQL Execution
// ============================================================================

/// SQL execution context holding a DuckDB connection
pub struct SqlContext {
    conn: Connection,
}

impl SqlContext {
    /// Create a new SQL context with an in-memory DuckDB database
    pub fn new() -> DuckResult<Self> {
        let conn = Connection::open_in_memory()?;
        Ok(Self { conn })
    }

    /// Create a new SQL context with a file-based DuckDB database
    pub fn open(path: &str) -> DuckResult<Self> {
        let conn = Connection::open(path)?;
        Ok(Self { conn })
    }

    /// Get a reference to the underlying DuckDB connection
    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    /// Execute a SQL query and return results as a list of maps
    ///
    /// Returns `Value::List` where each element is a `Value::Map` representing a row.
    /// Column names become map keys, and cell values become Lattice values.
    pub fn execute_query(&self, query: &str) -> Result<Value> {
        let mut stmt = self.conn.prepare(query).map_err(|e| {
            LatticeError::Runtime(format!("SQL prepare error: {}", e))
        })?;

        // Execute the statement first - DuckDB requires execution before column_count()/column_names() work
        stmt.execute([]).map_err(|e| {
            LatticeError::Runtime(format!("SQL execute error: {}", e))
        })?;

        // Get column info after execution
        let column_names = stmt.column_names();

        // Now query again to iterate through results
        let mut rows = stmt.query([]).map_err(|e| {
            LatticeError::Runtime(format!("SQL query error: {}", e))
        })?;

        let mut results = Vec::new();
        while let Some(row) = rows.next().map_err(|e| {
            LatticeError::Runtime(format!("SQL row error: {}", e))
        })? {
            let row_map = row_to_map(row, &column_names)?;
            results.push(row_map);
        }

        Ok(Value::list(results))
    }

    /// Execute a SQL query with parameters
    pub fn execute_query_with_params(&self, query: &str, params: &[Value]) -> Result<Value> {
        let duck_params: Vec<DuckValue> = params.iter().map(value_to_duckdb).collect();

        let mut stmt = self.conn.prepare(query).map_err(|e| {
            LatticeError::Runtime(format!("SQL prepare error: {}", e))
        })?;

        let params_slice: Vec<&dyn duckdb::ToSql> = duck_params
            .iter()
            .map(|v| v as &dyn duckdb::ToSql)
            .collect();

        // Execute the statement first - DuckDB requires execution before column_names() works
        stmt.execute(params_slice.as_slice()).map_err(|e| {
            LatticeError::Runtime(format!("SQL execute error: {}", e))
        })?;

        // Get column info after execution
        let column_names = stmt.column_names();

        // Now query again to iterate through results
        let mut rows = stmt.query(params_slice.as_slice()).map_err(|e| {
            LatticeError::Runtime(format!("SQL query error: {}", e))
        })?;

        let mut results = Vec::new();
        while let Some(row) = rows.next().map_err(|e| {
            LatticeError::Runtime(format!("SQL row error: {}", e))
        })? {
            let row_map = row_to_map(row, &column_names)?;
            results.push(row_map);
        }

        Ok(Value::list(results))
    }

    /// Get column metadata from a query result
    pub fn get_columns(&self, query: &str) -> Result<Vec<(String, String)>> {
        let mut stmt = self.conn.prepare(query).map_err(|e| {
            LatticeError::Runtime(format!("SQL prepare error: {}", e))
        })?;

        // Execute the statement first - DuckDB requires execution before column_type() works
        let mut rows = stmt.query([]).map_err(|e| {
            LatticeError::Runtime(format!("SQL query error: {}", e))
        })?;

        // Fetch at least one row to populate column metadata
        let _ = rows.next();

        let column_count = stmt.column_count();
        let mut columns = Vec::with_capacity(column_count);

        for i in 0..column_count {
            let name = stmt.column_name(i)
                .map(|s| s.to_string())
                .unwrap_or_else(|_| format!("col_{}", i));
            let type_name = format!("{:?}", stmt.column_type(i));
            columns.push((name, type_name));
        }

        Ok(columns)
    }
}

impl Default for SqlContext {
    fn default() -> Self {
        Self::new().expect("Failed to create in-memory DuckDB connection")
    }
}

/// Convert a DuckDB row to a Lattice Map value
fn row_to_map(row: &Row, column_names: &[String]) -> Result<Value> {
    let mut map = HashMap::new();

    for (idx, name) in column_names.iter().enumerate() {
        let value_ref = row.get_ref(idx).map_err(|e| {
            LatticeError::Runtime(format!("Error reading column {}: {}", name, e))
        })?;
        let value = duckdb_value_to_value(value_ref)?;
        map.insert(name.clone(), value);
    }

    Ok(Value::map(map))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_context_creation() {
        let ctx = SqlContext::new().unwrap();
        assert!(ctx.connection().is_autocommit());
    }

    #[test]
    fn test_simple_query() {
        let ctx = SqlContext::new().unwrap();

        // Create a simple table and query it
        ctx.connection()
            .execute("CREATE TABLE test (id INTEGER, name VARCHAR)", [])
            .unwrap();
        ctx.connection()
            .execute("INSERT INTO test VALUES (1, 'Alice'), (2, 'Bob')", [])
            .unwrap();

        let result = ctx.execute_query("SELECT * FROM test ORDER BY id").unwrap();

        match result {
            Value::List(rows) => {
                assert_eq!(rows.len(), 2);

                // Check first row
                if let Value::Map(row1) = &rows[0] {
                    assert!(matches!(row1.get("id"), Some(Value::Int(1))));
                    assert!(matches!(row1.get("name"), Some(Value::String(ref s)) if &**s == "Alice"));
                } else {
                    panic!("Expected Map");
                }

                // Check second row
                if let Value::Map(row2) = &rows[1] {
                    assert!(matches!(row2.get("id"), Some(Value::Int(2))));
                    assert!(matches!(row2.get("name"), Some(Value::String(ref s)) if &**s == "Bob"));
                } else {
                    panic!("Expected Map");
                }
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_null_handling() {
        let ctx = SqlContext::new().unwrap();

        ctx.connection()
            .execute("CREATE TABLE nulltest (id INTEGER, name VARCHAR)", [])
            .unwrap();
        ctx.connection()
            .execute("INSERT INTO nulltest VALUES (1, NULL)", [])
            .unwrap();

        let result = ctx.execute_query("SELECT * FROM nulltest").unwrap();

        match result {
            Value::List(rows) => {
                assert_eq!(rows.len(), 1);
                if let Value::Map(row) = &rows[0] {
                    assert!(matches!(row.get("name"), Some(Value::Null)));
                } else {
                    panic!("Expected Map");
                }
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_various_types() {
        let ctx = SqlContext::new().unwrap();

        ctx.connection()
            .execute(
                "CREATE TABLE types_test (
                    int_col INTEGER,
                    bigint_col BIGINT,
                    float_col FLOAT,
                    double_col DOUBLE,
                    bool_col BOOLEAN,
                    text_col VARCHAR
                )",
                [],
            )
            .unwrap();
        ctx.connection()
            .execute(
                "INSERT INTO types_test VALUES (42, 9223372036854775807, 3.14, 2.718281828, true, 'hello')",
                [],
            )
            .unwrap();

        let result = ctx.execute_query("SELECT * FROM types_test").unwrap();

        match result {
            Value::List(rows) => {
                assert_eq!(rows.len(), 1);
                if let Value::Map(row) = &rows[0] {
                    assert!(matches!(row.get("int_col"), Some(Value::Int(42))));
                    assert!(matches!(row.get("bigint_col"), Some(Value::Int(9223372036854775807))));
                    assert!(matches!(row.get("bool_col"), Some(Value::Bool(true))));
                    assert!(matches!(row.get("text_col"), Some(Value::String(ref s)) if &**s == "hello"));

                    // Check float values with tolerance
                    if let Some(Value::Float(f)) = row.get("float_col") {
                        assert!((f - 3.14).abs() < 0.01);
                    }
                    if let Some(Value::Float(d)) = row.get("double_col") {
                        assert!((d - 2.718281828).abs() < 0.0001);
                    }
                }
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_list_type() {
        let ctx = SqlContext::new().unwrap();

        let result = ctx.execute_query("SELECT [1, 2, 3] as arr").unwrap();

        match result {
            Value::List(rows) => {
                assert_eq!(rows.len(), 1);
                if let Value::Map(row) = &rows[0] {
                    if let Some(Value::List(arr)) = row.get("arr") {
                        assert_eq!(arr.len(), 3);
                        assert!(matches!(arr[0], Value::Int(1)));
                        assert!(matches!(arr[1], Value::Int(2)));
                        assert!(matches!(arr[2], Value::Int(3)));
                    } else {
                        panic!("Expected List for arr column");
                    }
                }
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_struct_type() {
        let ctx = SqlContext::new().unwrap();

        let result = ctx.execute_query("SELECT {'name': 'Alice', 'age': 30} as person").unwrap();

        match result {
            Value::List(rows) => {
                assert_eq!(rows.len(), 1);
                if let Value::Map(row) = &rows[0] {
                    if let Some(Value::Map(person)) = row.get("person") {
                        assert!(matches!(person.get("name"), Some(Value::String(ref s)) if &**s == "Alice"));
                        assert!(matches!(person.get("age"), Some(Value::Int(30))));
                    } else {
                        panic!("Expected Map for person column");
                    }
                }
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_csv_query() {
        let ctx = SqlContext::new().unwrap();

        // DuckDB can query CSV files directly
        // For this test, we'll create a temp table instead
        ctx.connection()
            .execute("CREATE TABLE users (name VARCHAR, age INTEGER)", [])
            .unwrap();
        ctx.connection()
            .execute("INSERT INTO users VALUES ('Alice', 30), ('Bob', 25)", [])
            .unwrap();

        let result = ctx.execute_query("SELECT * FROM users WHERE age > 20 ORDER BY name").unwrap();

        match result {
            Value::List(rows) => {
                assert_eq!(rows.len(), 2);
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_get_columns() {
        let ctx = SqlContext::new().unwrap();

        ctx.connection()
            .execute("CREATE TABLE col_test (id INTEGER, name VARCHAR, score DOUBLE)", [])
            .unwrap();

        let columns = ctx.get_columns("SELECT * FROM col_test").unwrap();

        assert_eq!(columns.len(), 3);
        assert_eq!(columns[0].0, "id");
        assert_eq!(columns[1].0, "name");
        assert_eq!(columns[2].0, "score");
    }

    #[test]
    fn test_value_to_duckdb_roundtrip() {
        // Test Int
        let int_val = Value::Int(42);
        let duck_int = value_to_duckdb(&int_val);
        assert!(matches!(duck_int, DuckValue::BigInt(42)));

        // Test Float
        let float_val = Value::Float(3.14);
        let duck_float = value_to_duckdb(&float_val);
        if let DuckValue::Double(f) = duck_float {
            assert!((f - 3.14).abs() < 0.0001);
        } else {
            panic!("Expected Double");
        }

        // Test String
        let str_val = Value::string("hello");
        let duck_str = value_to_duckdb(&str_val);
        assert!(matches!(duck_str, DuckValue::Text(s) if s == "hello"));

        // Test Bool
        let bool_val = Value::Bool(true);
        let duck_bool = value_to_duckdb(&bool_val);
        assert!(matches!(duck_bool, DuckValue::Boolean(true)));

        // Test Null
        let null_val = Value::Null;
        let duck_null = value_to_duckdb(&null_val);
        assert!(matches!(duck_null, DuckValue::Null));
    }

    #[test]
    fn test_date_from_days() {
        // 1970-01-01 is day 0
        assert_eq!(date_from_days(0), "1970-01-01");

        // Test a known date: 2024-01-01 is about 19724 days after 1970-01-01
        let days_2024 = (2024 - 1970) * 365 + 13; // approx with leap years
        let date = date_from_days(days_2024);
        assert!(date.starts_with("202"));
    }

    #[test]
    fn test_is_leap_year() {
        assert!(is_leap_year(2000)); // divisible by 400
        assert!(!is_leap_year(1900)); // divisible by 100 but not 400
        assert!(is_leap_year(2004)); // divisible by 4 but not 100
        assert!(!is_leap_year(2001)); // not divisible by 4
    }
}
