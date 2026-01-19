//! Arrow conversion for Lattice values
//!
//! This module provides conversion from Lattice data structures (List<Map>)
//! to Apache Arrow's columnar format for efficient SQL querying.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use arrow::array::{
    ArrayRef, BooleanArray, Float64Array, Int64Array, NullArray, StringBuilder,
};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;

use crate::runtime::providers::SqlError;
use crate::types::Value;

/// Get the type name of a Value for error messages
fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::String(_) => "String",
        Value::Int(_) => "Int",
        Value::Float(_) => "Float",
        Value::Bool(_) => "Bool",
        Value::Path(_) => "Path",
        Value::List(_) => "List",
        Value::Map(_) => "Map",
        Value::Null => "Null",
    }
}

/// Default number of rows to sample for schema inference
const DEFAULT_SCHEMA_SAMPLE_SIZE: usize = 100;

/// Convert a List<Map> to an Arrow RecordBatch
///
/// This validates that all elements are Maps and converts them to a columnar
/// Arrow format suitable for SQL querying.
///
/// # Type Mapping
///
/// | Lattice Type | Arrow Type |
/// |--------------|------------|
/// | Int          | Int64      |
/// | Float        | Float64    |
/// | String       | Utf8       |
/// | Bool         | Boolean    |
/// | Null         | Null       |
///
/// # Type Promotion
///
/// When a column has mixed types:
/// - Null + X → X (with nulls)
/// - Int + Float → Float64
/// - String + Int → Error
/// - Bool + Int → Error
///
/// # Errors
///
/// Returns an error if:
/// - Any element is not a Map
/// - A column has incompatible types (e.g., String + Int)
/// - A column appears after the schema inference window
pub fn lattice_list_to_recordbatch(list: &[Value]) -> Result<RecordBatch, SqlError> {
    if list.is_empty() {
        // Return empty RecordBatch with no columns
        return Ok(RecordBatch::new_empty(Arc::new(Schema::empty())));
    }

    // Validate all elements are Maps
    for (i, elem) in list.iter().enumerate() {
        if !matches!(elem, Value::Map(_)) {
            return Err(SqlError::ConversionError(format!(
                "Row {} is not a Map (found {}). SQL queries require List<Map> data.",
                i,
                value_type_name(elem)
            )));
        }
    }

    // Infer schema from first N rows
    let schema = infer_schema_from_list(list, DEFAULT_SCHEMA_SAMPLE_SIZE)?;

    // Validate no new columns appear after inference window
    if list.len() > DEFAULT_SCHEMA_SAMPLE_SIZE {
        let schema_columns: HashSet<&str> =
            schema.fields().iter().map(|f| f.name().as_str()).collect();

        for (i, elem) in list.iter().enumerate().skip(DEFAULT_SCHEMA_SAMPLE_SIZE) {
            if let Value::Map(map) = elem {
                for key in map.keys() {
                    if !schema_columns.contains(key.as_str()) {
                        return Err(SqlError::ConversionError(format!(
                            "Column '{}' appears at row {} after schema inference window (first {} rows). \
                             All columns must appear within the first {} rows.",
                            key, i, DEFAULT_SCHEMA_SAMPLE_SIZE, DEFAULT_SCHEMA_SAMPLE_SIZE
                        )));
                    }
                }
            }
        }
    }

    // Build arrays for each column
    let mut columns: Vec<ArrayRef> = Vec::with_capacity(schema.fields().len());

    for field in schema.fields() {
        let array = build_arrow_array(list, field)?;
        columns.push(array);
    }

    RecordBatch::try_new(Arc::new(schema), columns)
        .map_err(|e| SqlError::ConversionError(format!("Failed to create RecordBatch: {}", e)))
}

/// Infer Arrow schema from the first N rows of a list
fn infer_schema_from_list(list: &[Value], sample_size: usize) -> Result<Schema, SqlError> {
    let sample = &list[..list.len().min(sample_size)];

    // Collect all column names and their observed types
    let mut column_types: HashMap<String, Vec<LatticeType>> = HashMap::new();

    for elem in sample {
        if let Value::Map(map) = elem {
            for (key, value) in map.iter() {
                let lattice_type = LatticeType::from_value(value);
                column_types
                    .entry(key.clone())
                    .or_default()
                    .push(lattice_type);
            }
        }
    }

    // Determine the final Arrow type for each column
    let mut fields: Vec<Field> = Vec::new();

    // Sort column names for deterministic ordering
    let mut column_names: Vec<_> = column_types.keys().cloned().collect();
    column_names.sort();

    for name in column_names {
        let types = &column_types[&name];
        let arrow_type = infer_arrow_type(types, &name)?;
        fields.push(Field::new(&name, arrow_type, true)); // nullable=true for all columns
    }

    Ok(Schema::new(fields))
}

/// Build an Arrow array for a single column
fn build_arrow_array(list: &[Value], field: &Field) -> Result<ArrayRef, SqlError> {
    let column_name = field.name();

    match field.data_type() {
        DataType::Int64 => {
            let values: Vec<Option<i64>> = list
                .iter()
                .map(|row| extract_int64(row, column_name))
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Arc::new(Int64Array::from(values)))
        }
        DataType::Float64 => {
            let values: Vec<Option<f64>> = list
                .iter()
                .map(|row| extract_float64(row, column_name))
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Arc::new(Float64Array::from(values)))
        }
        DataType::Utf8 => {
            let values: Vec<Option<String>> = list
                .iter()
                .map(|row| extract_string(row, column_name))
                .collect::<Result<Vec<_>, _>>()?;

            let mut builder = StringBuilder::new();
            for value in &values {
                match value {
                    Some(s) => builder.append_value(s),
                    None => builder.append_null(),
                }
            }
            Ok(Arc::new(builder.finish()))
        }
        DataType::Boolean => {
            let values: Vec<Option<bool>> = list
                .iter()
                .map(|row| extract_bool(row, column_name))
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Arc::new(BooleanArray::from(values)))
        }
        DataType::Null => {
            // All values are null
            Ok(Arc::new(NullArray::new(list.len())))
        }
        other => Err(SqlError::ConversionError(format!(
            "Unsupported Arrow type: {:?}",
            other
        ))),
    }
}

/// Extract an i64 value from a row, with type promotion from Float
fn extract_int64(row: &Value, column: &str) -> Result<Option<i64>, SqlError> {
    let value = get_column_value(row, column);

    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Int(i)) => Ok(Some(*i)),
        Some(other) => Err(SqlError::ConversionError(format!(
            "Cannot convert {} to Int64 for column '{}'",
            value_type_name(other),
            column
        ))),
    }
}

/// Extract a f64 value from a row, with type promotion from Int
fn extract_float64(row: &Value, column: &str) -> Result<Option<f64>, SqlError> {
    let value = get_column_value(row, column);

    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Float(f)) => Ok(Some(*f)),
        Some(Value::Int(i)) => Ok(Some(*i as f64)), // Int → Float promotion
        Some(other) => Err(SqlError::ConversionError(format!(
            "Cannot convert {} to Float64 for column '{}'",
            value_type_name(other),
            column
        ))),
    }
}

/// Extract a string value from a row
fn extract_string(row: &Value, column: &str) -> Result<Option<String>, SqlError> {
    let value = get_column_value(row, column);

    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s.to_string())),
        Some(other) => Err(SqlError::ConversionError(format!(
            "Cannot convert {} to String for column '{}'",
            value_type_name(other),
            column
        ))),
    }
}

/// Extract a bool value from a row
fn extract_bool(row: &Value, column: &str) -> Result<Option<bool>, SqlError> {
    let value = get_column_value(row, column);

    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Bool(b)) => Ok(Some(*b)),
        Some(other) => Err(SqlError::ConversionError(format!(
            "Cannot convert {} to Boolean for column '{}'",
            value_type_name(other),
            column
        ))),
    }
}

/// Get a column value from a row
fn get_column_value<'a>(row: &'a Value, column: &str) -> Option<&'a Value> {
    if let Value::Map(map) = row {
        map.get(column)
    } else {
        None
    }
}

/// Lattice type categories for type inference
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LatticeType {
    Int,
    Float,
    String,
    Bool,
    Null,
    Other,
}

impl LatticeType {
    fn from_value(value: &Value) -> Self {
        match value {
            Value::Int(_) => LatticeType::Int,
            Value::Float(_) => LatticeType::Float,
            Value::String(_) => LatticeType::String,
            Value::Bool(_) => LatticeType::Bool,
            Value::Null => LatticeType::Null,
            _ => LatticeType::Other,
        }
    }
}

/// Infer the Arrow type from a set of observed Lattice types
fn infer_arrow_type(types: &[LatticeType], column: &str) -> Result<DataType, SqlError> {
    // Filter out Null types
    let non_null_types: Vec<_> = types.iter().filter(|t| **t != LatticeType::Null).collect();

    if non_null_types.is_empty() {
        // All nulls → Null type
        return Ok(DataType::Null);
    }

    // Check for consistent types
    let first_type = non_null_types[0];
    let mut has_int = false;
    let mut has_float = false;

    for t in &non_null_types {
        match **t {
            LatticeType::Int => has_int = true,
            LatticeType::Float => has_float = true,
            _ => {}
        }

        // Check for incompatible types
        if **t != *first_type && !is_promotable(**t, *first_type) {
            return Err(SqlError::ConversionError(format!(
                "Column '{}' has incompatible types: {:?} and {:?}. \
                 Only Int/Float promotion is supported.",
                column, first_type, **t
            )));
        }
    }

    // Apply type promotion
    if has_int && has_float {
        return Ok(DataType::Float64);
    }

    // Return the appropriate type
    match *first_type {
        LatticeType::Int => Ok(DataType::Int64),
        LatticeType::Float => Ok(DataType::Float64),
        LatticeType::String => Ok(DataType::Utf8),
        LatticeType::Bool => Ok(DataType::Boolean),
        LatticeType::Other => Err(SqlError::ConversionError(format!(
            "Column '{}' contains unsupported type (List, Map, or Path)",
            column
        ))),
        LatticeType::Null => unreachable!(), // Filtered out above
    }
}

/// Check if two types can be promoted to a common type
fn is_promotable(a: LatticeType, b: LatticeType) -> bool {
    matches!(
        (a, b),
        (LatticeType::Int, LatticeType::Float) | (LatticeType::Float, LatticeType::Int)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create a Map value
    fn map(pairs: Vec<(&str, Value)>) -> Value {
        let m: HashMap<String, Value> = pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect();
        Value::Map(Arc::new(m))
    }

    #[test]
    fn test_empty_list() {
        let list: Vec<Value> = vec![];
        let batch = lattice_list_to_recordbatch(&list).unwrap();
        assert_eq!(batch.num_rows(), 0);
        assert_eq!(batch.num_columns(), 0);
    }

    #[test]
    fn test_single_row_int_column() {
        let list = vec![map(vec![("id", Value::Int(42))])];
        let batch = lattice_list_to_recordbatch(&list).unwrap();

        assert_eq!(batch.num_rows(), 1);
        assert_eq!(batch.num_columns(), 1);
        assert_eq!(batch.schema().field(0).name(), "id");
        assert_eq!(batch.schema().field(0).data_type(), &DataType::Int64);
    }

    #[test]
    fn test_multiple_rows_multiple_columns() {
        let list = vec![
            map(vec![
                ("id", Value::Int(1)),
                ("name", Value::string("Alice")),
                ("active", Value::Bool(true)),
            ]),
            map(vec![
                ("id", Value::Int(2)),
                ("name", Value::string("Bob")),
                ("active", Value::Bool(false)),
            ]),
        ];
        let batch = lattice_list_to_recordbatch(&list).unwrap();

        assert_eq!(batch.num_rows(), 2);
        assert_eq!(batch.num_columns(), 3);
    }

    #[test]
    fn test_int_float_promotion() {
        let list = vec![
            map(vec![("value", Value::Int(42))]),
            map(vec![("value", Value::Float(3.14))]),
        ];
        let batch = lattice_list_to_recordbatch(&list).unwrap();

        assert_eq!(batch.schema().field(0).data_type(), &DataType::Float64);
    }

    #[test]
    fn test_null_handling() {
        let list = vec![
            map(vec![("id", Value::Int(1))]),
            map(vec![("id", Value::Null)]),
            map(vec![("id", Value::Int(3))]),
        ];
        let batch = lattice_list_to_recordbatch(&list).unwrap();

        assert_eq!(batch.num_rows(), 3);
        assert_eq!(batch.schema().field(0).data_type(), &DataType::Int64);
    }

    #[test]
    fn test_all_nulls() {
        let list = vec![
            map(vec![("value", Value::Null)]),
            map(vec![("value", Value::Null)]),
        ];
        let batch = lattice_list_to_recordbatch(&list).unwrap();

        assert_eq!(batch.schema().field(0).data_type(), &DataType::Null);
    }

    #[test]
    fn test_null_plus_type() {
        let list = vec![
            map(vec![("value", Value::Null)]),
            map(vec![("value", Value::Int(42))]),
        ];
        let batch = lattice_list_to_recordbatch(&list).unwrap();

        // Should infer Int64, not Null
        assert_eq!(batch.schema().field(0).data_type(), &DataType::Int64);
    }

    #[test]
    fn test_missing_columns() {
        let list = vec![
            map(vec![("id", Value::Int(1)), ("name", Value::string("Alice"))]),
            map(vec![("id", Value::Int(2))]), // missing "name"
        ];
        let batch = lattice_list_to_recordbatch(&list).unwrap();

        assert_eq!(batch.num_rows(), 2);
        assert_eq!(batch.num_columns(), 2);
    }

    #[test]
    fn test_non_map_element_error() {
        let list = vec![
            map(vec![("id", Value::Int(1))]),
            Value::Int(42), // Not a Map
        ];
        let result = lattice_list_to_recordbatch(&list);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not a Map"));
    }

    #[test]
    fn test_incompatible_types_string_int() {
        let list = vec![
            map(vec![("value", Value::Int(42))]),
            map(vec![("value", Value::string("hello"))]),
        ];
        let result = lattice_list_to_recordbatch(&list);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("incompatible types"));
    }

    #[test]
    fn test_incompatible_types_bool_int() {
        let list = vec![
            map(vec![("value", Value::Int(42))]),
            map(vec![("value", Value::Bool(true))]),
        ];
        let result = lattice_list_to_recordbatch(&list);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("incompatible types"));
    }

    #[test]
    fn test_deterministic_column_order() {
        // Columns should be sorted alphabetically
        let list = vec![map(vec![
            ("zebra", Value::Int(1)),
            ("alpha", Value::Int(2)),
            ("middle", Value::Int(3)),
        ])];
        let batch = lattice_list_to_recordbatch(&list).unwrap();

        assert_eq!(batch.schema().field(0).name(), "alpha");
        assert_eq!(batch.schema().field(1).name(), "middle");
        assert_eq!(batch.schema().field(2).name(), "zebra");
    }

    #[test]
    fn test_float_column() {
        let list = vec![
            map(vec![("value", Value::Float(3.14))]),
            map(vec![("value", Value::Float(2.71))]),
        ];
        let batch = lattice_list_to_recordbatch(&list).unwrap();

        assert_eq!(batch.schema().field(0).data_type(), &DataType::Float64);
    }

    #[test]
    fn test_bool_column() {
        let list = vec![
            map(vec![("flag", Value::Bool(true))]),
            map(vec![("flag", Value::Bool(false))]),
        ];
        let batch = lattice_list_to_recordbatch(&list).unwrap();

        assert_eq!(batch.schema().field(0).data_type(), &DataType::Boolean);
    }

    #[test]
    fn test_string_column() {
        let list = vec![
            map(vec![("name", Value::string("Alice"))]),
            map(vec![("name", Value::string("Bob"))]),
        ];
        let batch = lattice_list_to_recordbatch(&list).unwrap();

        assert_eq!(batch.schema().field(0).data_type(), &DataType::Utf8);
    }
}
