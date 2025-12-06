//! SQL Provider trait for injectable SQL capabilities
//!
//! This module provides the `SqlProvider` trait that allows the Lattice runtime
//! to execute SQL queries through an injectable implementation. This enables:
//!
//! - Default DuckDB-based execution (via `DuckDbProvider`)
//! - Host-language callbacks (e.g., using Elixir's Ecto or Python's SQLAlchemy)
//! - Disabled SQL support (via `NoSqlProvider`)
//! - Testing with mock responses

use std::fmt;
use std::sync::Arc;

use crate::runtime::LatticeValue;

/// Error type for SQL operations
#[derive(Debug, Clone)]
pub enum SqlError {
    /// SQL provider is not configured
    NotConfigured(String),
    /// Connection error
    ConnectionError(String),
    /// Query preparation error
    PrepareError(String),
    /// Query execution error
    ExecutionError(String),
    /// Failed to convert result values
    ConversionError(String),
    /// Other errors
    Other(String),
}

impl fmt::Display for SqlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SqlError::NotConfigured(msg) => write!(f, "SQL not configured: {}", msg),
            SqlError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            SqlError::PrepareError(msg) => write!(f, "Prepare error: {}", msg),
            SqlError::ExecutionError(msg) => write!(f, "Execution error: {}", msg),
            SqlError::ConversionError(msg) => write!(f, "Conversion error: {}", msg),
            SqlError::Other(msg) => write!(f, "SQL error: {}", msg),
        }
    }
}

impl std::error::Error for SqlError {}

/// A row from a SQL query result
pub type SqlRow = Vec<(String, LatticeValue)>;

/// Result from a SQL query
#[derive(Clone, Debug)]
pub struct SqlResult {
    /// Column names in order
    pub columns: Vec<String>,
    /// Rows of data, each row is a list of (column_name, value) pairs
    pub rows: Vec<SqlRow>,
}

impl SqlResult {
    /// Create an empty result with no columns or rows
    pub fn empty() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
        }
    }

    /// Create a new result with specified columns and rows
    pub fn new(columns: Vec<String>, rows: Vec<SqlRow>) -> Self {
        Self { columns, rows }
    }

    /// Get the number of rows
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Get the number of columns
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Check if the result is empty
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Convert to a LatticeValue (list of maps)
    pub fn to_lattice_value(&self) -> LatticeValue {
        let rows: Vec<LatticeValue> = self
            .rows
            .iter()
            .map(|row| LatticeValue::Map(row.clone()))
            .collect();
        LatticeValue::List(rows)
    }
}

/// Trait for SQL providers
///
/// Implement this trait to provide custom SQL handling. The trait is designed
/// to be object-safe so it can be used as `Arc<dyn SqlProvider>`.
///
/// Unlike LlmProvider, SqlProvider uses synchronous methods because most
/// SQL databases (including DuckDB) have synchronous APIs.
///
/// # Example
///
/// ```ignore
/// use lattice::runtime::providers::{SqlProvider, SqlResult, SqlError};
///
/// struct MyCustomProvider;
///
/// impl SqlProvider for MyCustomProvider {
///     fn query(&self, sql: &str) -> Result<SqlResult, SqlError> {
///         // Custom implementation...
///         Ok(SqlResult::empty())
///     }
/// }
/// ```
pub trait SqlProvider: Send + Sync {
    /// Execute a SQL query and return results
    ///
    /// The query should be a valid SQL statement. Results are returned as
    /// a `SqlResult` containing column names and row data.
    fn query(&self, sql: &str) -> Result<SqlResult, SqlError>;

    /// Execute a SQL query with parameters
    ///
    /// Default implementation ignores parameters and just calls `query`.
    /// Override this for providers that support parameterized queries.
    fn query_with_params(
        &self,
        sql: &str,
        _params: &[LatticeValue],
    ) -> Result<SqlResult, SqlError> {
        // Default: ignore params
        self.query(sql)
    }

    /// Execute a SQL statement that doesn't return results (INSERT, UPDATE, DELETE, etc.)
    ///
    /// Returns the number of affected rows if available.
    fn execute(&self, sql: &str) -> Result<usize, SqlError> {
        // Default implementation: run as query, return 0
        self.query(sql)?;
        Ok(0)
    }

    /// Get column metadata for a query without executing it
    ///
    /// Returns a list of (column_name, type_name) pairs.
    /// Default implementation returns an empty list.
    fn get_columns(&self, _sql: &str) -> Result<Vec<(String, String)>, SqlError> {
        Ok(Vec::new())
    }
}

/// DuckDB-based SQL provider (default)
///
/// This provider uses an in-memory DuckDB database. It's suitable for
/// ad-hoc SQL queries on CSV/Parquet files and in-memory data processing.
///
/// The connection is wrapped in a Mutex to ensure thread-safety, as
/// DuckDB's Connection type is not Sync.
#[cfg(feature = "sql")]
pub struct DuckDbProvider {
    connection: std::sync::Mutex<duckdb::Connection>,
}

#[cfg(feature = "sql")]
impl DuckDbProvider {
    /// Create a new provider with an in-memory database
    pub fn new() -> Result<Self, SqlError> {
        let connection = duckdb::Connection::open_in_memory()
            .map_err(|e| SqlError::ConnectionError(e.to_string()))?;
        Ok(Self {
            connection: std::sync::Mutex::new(connection),
        })
    }

    /// Create a provider with a file-based database
    pub fn with_path(path: &str) -> Result<Self, SqlError> {
        let connection = duckdb::Connection::open(path)
            .map_err(|e| SqlError::ConnectionError(e.to_string()))?;
        Ok(Self {
            connection: std::sync::Mutex::new(connection),
        })
    }

    /// Execute a function with access to the underlying DuckDB connection
    ///
    /// This locks the connection for the duration of the function call.
    pub fn with_connection<F, R>(&self, f: F) -> Result<R, SqlError>
    where
        F: FnOnce(&duckdb::Connection) -> Result<R, SqlError>,
    {
        let conn = self
            .connection
            .lock()
            .map_err(|e| SqlError::Other(format!("Lock poisoned: {}", e)))?;
        f(&conn)
    }

    /// Convert a DuckDB row to a SqlRow
    fn row_to_sql_row(
        row: &duckdb::Row,
        column_names: &[String],
    ) -> Result<SqlRow, SqlError> {
        use crate::sql::convert::duckdb_value_to_value;

        let mut sql_row = Vec::with_capacity(column_names.len());

        for (idx, name) in column_names.iter().enumerate() {
            let value_ref = row.get_ref(idx).map_err(|e| {
                SqlError::ConversionError(format!("Error reading column {}: {}", name, e))
            })?;

            // Convert DuckDB value to internal Value, then to LatticeValue
            let internal_value = duckdb_value_to_value(value_ref)
                .map_err(|e| SqlError::ConversionError(e.to_string()))?;

            let lattice_value = LatticeValue::from_internal(&internal_value)
                .map_err(|e| SqlError::ConversionError(e.to_string()))?;

            sql_row.push((name.clone(), lattice_value));
        }

        Ok(sql_row)
    }
}

#[cfg(feature = "sql")]
impl Default for DuckDbProvider {
    fn default() -> Self {
        Self::new().expect("Failed to create in-memory DuckDB connection")
    }
}

#[cfg(feature = "sql")]
impl SqlProvider for DuckDbProvider {
    fn query(&self, sql: &str) -> Result<SqlResult, SqlError> {
        let conn = self
            .connection
            .lock()
            .map_err(|e| SqlError::Other(format!("Lock poisoned: {}", e)))?;

        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| SqlError::PrepareError(e.to_string()))?;

        // Execute the statement first - DuckDB requires execution before column_names() works
        stmt.execute([])
            .map_err(|e| SqlError::ExecutionError(e.to_string()))?;

        // Get column info after execution
        let column_names = stmt.column_names();

        // Now query again to iterate through results
        let mut rows_iter = stmt
            .query([])
            .map_err(|e| SqlError::ExecutionError(e.to_string()))?;

        let mut rows = Vec::new();
        while let Some(row) = rows_iter
            .next()
            .map_err(|e| SqlError::ExecutionError(e.to_string()))?
        {
            let sql_row = Self::row_to_sql_row(row, &column_names)?;
            rows.push(sql_row);
        }

        Ok(SqlResult::new(column_names, rows))
    }

    fn query_with_params(
        &self,
        sql: &str,
        params: &[LatticeValue],
    ) -> Result<SqlResult, SqlError> {
        use crate::sql::convert::value_to_duckdb;
        use duckdb::types::Value as DuckValue;

        let conn = self
            .connection
            .lock()
            .map_err(|e| SqlError::Other(format!("Lock poisoned: {}", e)))?;

        // Convert LatticeValue params to DuckDB values
        let internal_params: Vec<_> = params.iter().map(|v| v.to_internal()).collect();
        let duck_params: Vec<DuckValue> = internal_params.iter().map(value_to_duckdb).collect();

        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| SqlError::PrepareError(e.to_string()))?;

        let params_slice: Vec<&dyn duckdb::ToSql> =
            duck_params.iter().map(|v| v as &dyn duckdb::ToSql).collect();

        // Execute the statement first
        stmt.execute(params_slice.as_slice())
            .map_err(|e| SqlError::ExecutionError(e.to_string()))?;

        // Get column info after execution
        let column_names = stmt.column_names();

        // Now query again to iterate through results
        let mut rows_iter = stmt
            .query(params_slice.as_slice())
            .map_err(|e| SqlError::ExecutionError(e.to_string()))?;

        let mut rows = Vec::new();
        while let Some(row) = rows_iter
            .next()
            .map_err(|e| SqlError::ExecutionError(e.to_string()))?
        {
            let sql_row = Self::row_to_sql_row(row, &column_names)?;
            rows.push(sql_row);
        }

        Ok(SqlResult::new(column_names, rows))
    }

    fn execute(&self, sql: &str) -> Result<usize, SqlError> {
        let conn = self
            .connection
            .lock()
            .map_err(|e| SqlError::Other(format!("Lock poisoned: {}", e)))?;

        let affected = conn
            .execute(sql, [])
            .map_err(|e| SqlError::ExecutionError(e.to_string()))?;
        Ok(affected)
    }

    fn get_columns(&self, sql: &str) -> Result<Vec<(String, String)>, SqlError> {
        let conn = self
            .connection
            .lock()
            .map_err(|e| SqlError::Other(format!("Lock poisoned: {}", e)))?;

        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| SqlError::PrepareError(e.to_string()))?;

        // Execute the statement first - DuckDB requires execution before column_type() works
        let mut rows = stmt
            .query([])
            .map_err(|e| SqlError::ExecutionError(e.to_string()))?;

        // Fetch at least one row to populate column metadata
        let _ = rows.next();

        let column_count = stmt.column_count();
        let mut columns = Vec::with_capacity(column_count);

        for i in 0..column_count {
            let name = stmt
                .column_name(i)
                .map(|s| s.to_string())
                .unwrap_or_else(|_| format!("col_{}", i));
            let type_name = format!("{:?}", stmt.column_type(i));
            columns.push((name, type_name));
        }

        Ok(columns)
    }
}

/// No-op provider for runtimes without SQL support
///
/// This provider always returns an error. Use it when embedding Lattice
/// in contexts where SQL queries should be disabled.
pub struct NoSqlProvider;

impl SqlProvider for NoSqlProvider {
    fn query(&self, _sql: &str) -> Result<SqlResult, SqlError> {
        Err(SqlError::NotConfigured(
            "SQL provider not configured. Use RuntimeBuilder::with_sql_provider() to enable SQL support.".to_string(),
        ))
    }

    fn execute(&self, _sql: &str) -> Result<usize, SqlError> {
        Err(SqlError::NotConfigured(
            "SQL provider not configured. Use RuntimeBuilder::with_sql_provider() to enable SQL support.".to_string(),
        ))
    }
}

/// Type alias for boxed SQL provider
pub type BoxedSqlProvider = Arc<dyn SqlProvider>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_result_empty() {
        let result = SqlResult::empty();
        assert!(result.is_empty());
        assert_eq!(result.row_count(), 0);
        assert_eq!(result.column_count(), 0);
    }

    #[test]
    fn test_sql_result_new() {
        let columns = vec!["id".to_string(), "name".to_string()];
        let rows = vec![
            vec![
                ("id".to_string(), LatticeValue::Int(1)),
                ("name".to_string(), LatticeValue::String("Alice".to_string())),
            ],
            vec![
                ("id".to_string(), LatticeValue::Int(2)),
                ("name".to_string(), LatticeValue::String("Bob".to_string())),
            ],
        ];

        let result = SqlResult::new(columns, rows);
        assert!(!result.is_empty());
        assert_eq!(result.row_count(), 2);
        assert_eq!(result.column_count(), 2);
    }

    #[test]
    fn test_sql_result_to_lattice_value() {
        let columns = vec!["id".to_string(), "name".to_string()];
        let rows = vec![vec![
            ("id".to_string(), LatticeValue::Int(1)),
            ("name".to_string(), LatticeValue::String("Alice".to_string())),
        ]];

        let result = SqlResult::new(columns, rows);
        let value = result.to_lattice_value();

        if let LatticeValue::List(list) = value {
            assert_eq!(list.len(), 1);
            if let LatticeValue::Map(row) = &list[0] {
                assert_eq!(row.len(), 2);
                // Check that the row contains expected values
                let id = row.iter().find(|(k, _)| k == "id").map(|(_, v)| v);
                assert_eq!(id, Some(&LatticeValue::Int(1)));
            } else {
                panic!("Expected Map");
            }
        } else {
            panic!("Expected List");
        }
    }

    #[test]
    fn test_no_sql_provider_returns_error() {
        let provider = NoSqlProvider;

        let result = provider.query("SELECT 1");
        assert!(result.is_err());
        match result {
            Err(SqlError::NotConfigured(_)) => (),
            _ => panic!("Expected NotConfigured error"),
        }

        let exec_result = provider.execute("INSERT INTO test VALUES (1)");
        assert!(exec_result.is_err());
        match exec_result {
            Err(SqlError::NotConfigured(_)) => (),
            _ => panic!("Expected NotConfigured error"),
        }
    }

    #[test]
    fn test_sql_error_display() {
        let errors = vec![
            (
                SqlError::NotConfigured("test".to_string()),
                "SQL not configured: test",
            ),
            (
                SqlError::ConnectionError("timeout".to_string()),
                "Connection error: timeout",
            ),
            (
                SqlError::PrepareError("syntax error".to_string()),
                "Prepare error: syntax error",
            ),
            (
                SqlError::ExecutionError("constraint violation".to_string()),
                "Execution error: constraint violation",
            ),
            (
                SqlError::ConversionError("invalid type".to_string()),
                "Conversion error: invalid type",
            ),
            (SqlError::Other("misc".to_string()), "SQL error: misc"),
        ];

        for (error, expected) in errors {
            assert_eq!(format!("{}", error), expected);
        }
    }

    #[cfg(feature = "sql")]
    mod duckdb_tests {
        use super::*;

        #[test]
        fn test_duckdb_provider_creation() {
            let provider = DuckDbProvider::new().unwrap();
            provider
                .with_connection(|conn| {
                    assert!(conn.is_autocommit());
                    Ok(())
                })
                .unwrap();
        }

        #[test]
        fn test_duckdb_simple_query() {
            let provider = DuckDbProvider::new().unwrap();

            // Create a simple table and query it
            provider
                .with_connection(|conn| {
                    conn.execute("CREATE TABLE test (id INTEGER, name VARCHAR)", [])
                        .map_err(|e| SqlError::ExecutionError(e.to_string()))?;
                    conn.execute("INSERT INTO test VALUES (1, 'Alice'), (2, 'Bob')", [])
                        .map_err(|e| SqlError::ExecutionError(e.to_string()))?;
                    Ok(())
                })
                .unwrap();

            let result = provider
                .query("SELECT * FROM test ORDER BY id")
                .unwrap();

            assert_eq!(result.row_count(), 2);
            assert_eq!(result.column_count(), 2);
            assert!(result.columns.contains(&"id".to_string()));
            assert!(result.columns.contains(&"name".to_string()));

            // Check first row values
            let first_row = &result.rows[0];
            let id_val = first_row.iter().find(|(k, _)| k == "id").map(|(_, v)| v);
            assert_eq!(id_val, Some(&LatticeValue::Int(1)));

            let name_val = first_row.iter().find(|(k, _)| k == "name").map(|(_, v)| v);
            assert_eq!(
                name_val,
                Some(&LatticeValue::String("Alice".to_string()))
            );
        }

        #[test]
        fn test_duckdb_execute() {
            let provider = DuckDbProvider::new().unwrap();

            provider
                .with_connection(|conn| {
                    conn.execute("CREATE TABLE test (id INTEGER)", [])
                        .map_err(|e| SqlError::ExecutionError(e.to_string()))?;
                    Ok(())
                })
                .unwrap();

            let affected = provider
                .execute("INSERT INTO test VALUES (1), (2), (3)")
                .unwrap();

            // DuckDB returns the number of inserted rows
            assert_eq!(affected, 3);
        }

        #[test]
        fn test_duckdb_null_handling() {
            let provider = DuckDbProvider::new().unwrap();

            provider
                .with_connection(|conn| {
                    conn.execute("CREATE TABLE nulltest (id INTEGER, name VARCHAR)", [])
                        .map_err(|e| SqlError::ExecutionError(e.to_string()))?;
                    conn.execute("INSERT INTO nulltest VALUES (1, NULL)", [])
                        .map_err(|e| SqlError::ExecutionError(e.to_string()))?;
                    Ok(())
                })
                .unwrap();

            let result = provider.query("SELECT * FROM nulltest").unwrap();

            assert_eq!(result.row_count(), 1);
            let row = &result.rows[0];
            let name_val = row.iter().find(|(k, _)| k == "name").map(|(_, v)| v);
            assert_eq!(name_val, Some(&LatticeValue::Null));
        }

        #[test]
        fn test_duckdb_various_types() {
            let provider = DuckDbProvider::new().unwrap();

            provider
                .with_connection(|conn| {
                    conn.execute(
                        "CREATE TABLE types_test (
                            int_col INTEGER,
                            float_col DOUBLE,
                            bool_col BOOLEAN,
                            text_col VARCHAR
                        )",
                        [],
                    )
                    .map_err(|e| SqlError::ExecutionError(e.to_string()))?;
                    conn.execute(
                        "INSERT INTO types_test VALUES (42, 3.14, true, 'hello')",
                        [],
                    )
                    .map_err(|e| SqlError::ExecutionError(e.to_string()))?;
                    Ok(())
                })
                .unwrap();

            let result = provider.query("SELECT * FROM types_test").unwrap();

            assert_eq!(result.row_count(), 1);
            let row = &result.rows[0];

            let int_val = row.iter().find(|(k, _)| k == "int_col").map(|(_, v)| v);
            assert_eq!(int_val, Some(&LatticeValue::Int(42)));

            let bool_val = row.iter().find(|(k, _)| k == "bool_col").map(|(_, v)| v);
            assert_eq!(bool_val, Some(&LatticeValue::Bool(true)));

            let text_val = row.iter().find(|(k, _)| k == "text_col").map(|(_, v)| v);
            assert_eq!(
                text_val,
                Some(&LatticeValue::String("hello".to_string()))
            );

            // Float comparison with tolerance
            if let Some(LatticeValue::Float(f)) =
                row.iter().find(|(k, _)| k == "float_col").map(|(_, v)| v)
            {
                assert!((f - 3.14).abs() < 0.0001);
            } else {
                panic!("Expected Float");
            }
        }

        #[test]
        fn test_duckdb_get_columns() {
            let provider = DuckDbProvider::new().unwrap();

            provider
                .with_connection(|conn| {
                    conn.execute(
                        "CREATE TABLE col_test (id INTEGER, name VARCHAR, score DOUBLE)",
                        [],
                    )
                    .map_err(|e| SqlError::ExecutionError(e.to_string()))?;
                    Ok(())
                })
                .unwrap();

            let columns = provider.get_columns("SELECT * FROM col_test").unwrap();

            assert_eq!(columns.len(), 3);
            assert_eq!(columns[0].0, "id");
            assert_eq!(columns[1].0, "name");
            assert_eq!(columns[2].0, "score");
        }

        #[test]
        fn test_duckdb_query_with_params() {
            let provider = DuckDbProvider::new().unwrap();

            provider
                .with_connection(|conn| {
                    conn.execute("CREATE TABLE param_test (id INTEGER, name VARCHAR)", [])
                        .map_err(|e| SqlError::ExecutionError(e.to_string()))?;
                    conn.execute(
                        "INSERT INTO param_test VALUES (1, 'Alice'), (2, 'Bob'), (3, 'Charlie')",
                        [],
                    )
                    .map_err(|e| SqlError::ExecutionError(e.to_string()))?;
                    Ok(())
                })
                .unwrap();

            // Query where id > 1 should return 2 rows (id=2 and id=3)
            let params = vec![LatticeValue::Int(1)];
            let result = provider
                .query_with_params("SELECT * FROM param_test WHERE id > ?", &params)
                .unwrap();

            assert_eq!(result.row_count(), 2);
        }
    }
}
