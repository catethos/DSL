//! DuckDB SQL integration module
//!
//! Provides SQL execution and type conversion between DuckDB and Lattice types.

pub mod convert;

pub use convert::{
    duckdb_value_to_value, duckdb_owned_value_to_value, value_to_duckdb, SqlContext,
};
