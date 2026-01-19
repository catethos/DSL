//! DuckDB SQL integration module
//!
//! Provides SQL execution and type conversion between DuckDB and Lattice types.

#[cfg(feature = "sql-arrow")]
pub mod arrow;
pub mod convert;
pub mod ident;
pub mod refs;

#[cfg(feature = "sql-arrow")]
pub use arrow::lattice_list_to_recordbatch;
pub use convert::{
    duckdb_value_to_value, duckdb_owned_value_to_value, value_to_duckdb, SqlContext,
};
pub use ident::{is_valid_unquoted_ident, normalize_ident, quote_ident};
pub use refs::extract_table_references;
