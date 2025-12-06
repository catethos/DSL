//! FFI-safe value types for cross-language marshaling
//!
//! This module defines `LatticeValue`, an FFI-friendly representation of
//! Lattice values that can be easily converted to/from host language types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;

use crate::types::Value;

/// Error type for value conversion failures
#[derive(Debug, Clone, PartialEq)]
pub enum ConversionError {
    /// The value type cannot be exported to FFI
    UnsupportedType(String),
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConversionError::UnsupportedType(msg) => {
                write!(f, "unsupported type for FFI export: {}", msg)
            }
        }
    }
}

impl std::error::Error for ConversionError {}

/// FFI-safe value type for embedding Lattice in other languages.
///
/// Designed for easy conversion to/from host language types:
/// - Uses `String` instead of `Arc<str>` for simplicity
/// - Uses `Vec<(String, LatticeValue)>` instead of `HashMap` for deterministic ordering
/// - Excludes internal-only types (Closure, Future) that can't cross FFI boundaries
///
/// All variants are designed to be easily representable in common FFI targets:
/// - JSON (via Serde)
/// - Elixir terms (via Rustler)
/// - Python objects (via PyO3)
/// - JavaScript values (via Neon)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum LatticeValue {
    /// Null/nil value
    Null,
    /// Boolean value
    Bool(bool),
    /// 64-bit signed integer
    Int(i64),
    /// 64-bit floating point
    Float(f64),
    /// UTF-8 string (owned, not Arc)
    String(String),
    /// Filesystem path
    Path(String),
    /// Ordered list of values
    List(Vec<LatticeValue>),
    /// Map as ordered key-value pairs (deterministic iteration order)
    Map(Vec<(String, LatticeValue)>),
}

impl LatticeValue {
    /// Convert from internal VM `Value` to FFI-safe `LatticeValue`.
    ///
    /// This conversion is infallible for all current Value variants since the
    /// internal Value type doesn't have Closure/Future variants yet.
    ///
    /// # Examples
    ///
    /// ```
    /// use lattice::types::Value;
    /// use lattice::runtime::LatticeValue;
    ///
    /// let internal = Value::Int(42);
    /// let ffi = LatticeValue::from_internal(&internal).unwrap();
    /// assert_eq!(ffi, LatticeValue::Int(42));
    /// ```
    pub fn from_internal(value: &Value) -> Result<Self, ConversionError> {
        match value {
            Value::Null => Ok(LatticeValue::Null),
            Value::Bool(b) => Ok(LatticeValue::Bool(*b)),
            Value::Int(i) => Ok(LatticeValue::Int(*i)),
            Value::Float(f) => Ok(LatticeValue::Float(*f)),
            Value::String(s) => Ok(LatticeValue::String(s.to_string())),
            Value::Path(p) => Ok(LatticeValue::Path(p.to_string_lossy().into_owned())),
            Value::List(items) => {
                let converted: Result<Vec<_>, _> =
                    items.iter().map(Self::from_internal).collect();
                Ok(LatticeValue::List(converted?))
            }
            Value::Map(map) => {
                // Convert HashMap to Vec<(K, V)> with sorted keys for deterministic order
                let mut pairs: Vec<_> = map
                    .iter()
                    .map(|(k, v)| Self::from_internal(v).map(|converted| (k.clone(), converted)))
                    .collect::<Result<Vec<_>, _>>()?;
                // Sort by key for deterministic output
                pairs.sort_by(|a, b| a.0.cmp(&b.0));
                Ok(LatticeValue::Map(pairs))
            }
        }
    }

    /// Convert FFI-safe `LatticeValue` to internal VM `Value`.
    ///
    /// This conversion is infallible - all LatticeValue variants have
    /// corresponding Value representations.
    ///
    /// # Examples
    ///
    /// ```
    /// use lattice::types::Value;
    /// use lattice::runtime::LatticeValue;
    ///
    /// let ffi = LatticeValue::Int(42);
    /// let internal = ffi.to_internal();
    /// assert_eq!(internal.as_int(), Some(42));
    /// ```
    pub fn to_internal(&self) -> Value {
        match self {
            LatticeValue::Null => Value::Null,
            LatticeValue::Bool(b) => Value::Bool(*b),
            LatticeValue::Int(i) => Value::Int(*i),
            LatticeValue::Float(f) => Value::Float(*f),
            LatticeValue::String(s) => Value::String(Arc::from(s.as_str())),
            LatticeValue::Path(p) => Value::Path(Arc::new(PathBuf::from(p))),
            LatticeValue::List(items) => {
                let converted: Vec<Value> = items.iter().map(|v| v.to_internal()).collect();
                Value::List(Arc::new(converted))
            }
            LatticeValue::Map(pairs) => {
                let map: HashMap<String, Value> = pairs
                    .iter()
                    .map(|(k, v)| (k.clone(), v.to_internal()))
                    .collect();
                Value::Map(Arc::new(map))
            }
        }
    }

    /// Check if this value is null
    pub fn is_null(&self) -> bool {
        matches!(self, LatticeValue::Null)
    }

    /// Try to get this value as a bool
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            LatticeValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Try to get this value as an i64
    pub fn as_int(&self) -> Option<i64> {
        match self {
            LatticeValue::Int(i) => Some(*i),
            _ => None,
        }
    }

    /// Try to get this value as an f64
    pub fn as_float(&self) -> Option<f64> {
        match self {
            LatticeValue::Float(f) => Some(*f),
            _ => None,
        }
    }

    /// Try to get this value as a string slice
    pub fn as_str(&self) -> Option<&str> {
        match self {
            LatticeValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get this value as a list
    pub fn as_list(&self) -> Option<&[LatticeValue]> {
        match self {
            LatticeValue::List(l) => Some(l),
            _ => None,
        }
    }

    /// Try to get this value as a map (ordered key-value pairs)
    pub fn as_map(&self) -> Option<&[(String, LatticeValue)]> {
        match self {
            LatticeValue::Map(m) => Some(m),
            _ => None,
        }
    }

    /// Get a field from a map by key
    pub fn get(&self, key: &str) -> Option<&LatticeValue> {
        match self {
            LatticeValue::Map(pairs) => pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }
}

impl fmt::Display for LatticeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LatticeValue::Null => write!(f, "null"),
            LatticeValue::Bool(b) => write!(f, "{}", b),
            LatticeValue::Int(i) => write!(f, "{}", i),
            LatticeValue::Float(fl) => write!(f, "{}", fl),
            LatticeValue::String(s) => write!(f, "\"{}\"", s),
            LatticeValue::Path(p) => write!(f, "Path(\"{}\")", p),
            LatticeValue::List(l) => {
                write!(f, "[")?;
                for (i, v) in l.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            LatticeValue::Map(pairs) => {
                write!(f, "{{")?;
                for (i, (k, v)) in pairs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {}", k, v)?;
                }
                write!(f, "}}")
            }
        }
    }
}

// Convenient From implementations for creating LatticeValue

impl From<bool> for LatticeValue {
    fn from(b: bool) -> Self {
        LatticeValue::Bool(b)
    }
}

impl From<i64> for LatticeValue {
    fn from(i: i64) -> Self {
        LatticeValue::Int(i)
    }
}

impl From<i32> for LatticeValue {
    fn from(i: i32) -> Self {
        LatticeValue::Int(i64::from(i))
    }
}

impl From<f64> for LatticeValue {
    fn from(f: f64) -> Self {
        LatticeValue::Float(f)
    }
}

impl From<String> for LatticeValue {
    fn from(s: String) -> Self {
        LatticeValue::String(s)
    }
}

impl From<&str> for LatticeValue {
    fn from(s: &str) -> Self {
        LatticeValue::String(s.to_string())
    }
}

impl<T: Into<LatticeValue>> From<Vec<T>> for LatticeValue {
    fn from(v: Vec<T>) -> Self {
        LatticeValue::List(v.into_iter().map(Into::into).collect())
    }
}

impl From<()> for LatticeValue {
    fn from(_: ()) -> Self {
        LatticeValue::Null
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_roundtrip() {
        let internal = Value::Null;
        let ffi = LatticeValue::from_internal(&internal).unwrap();
        assert_eq!(ffi, LatticeValue::Null);
        let back = ffi.to_internal();
        assert!(back.is_null());
    }

    #[test]
    fn test_bool_roundtrip() {
        for b in [true, false] {
            let internal = Value::Bool(b);
            let ffi = LatticeValue::from_internal(&internal).unwrap();
            assert_eq!(ffi, LatticeValue::Bool(b));
            let back = ffi.to_internal();
            assert_eq!(back.as_bool(), Some(b));
        }
    }

    #[test]
    fn test_int_roundtrip() {
        for i in [0i64, 42, -100, i64::MAX, i64::MIN] {
            let internal = Value::Int(i);
            let ffi = LatticeValue::from_internal(&internal).unwrap();
            assert_eq!(ffi, LatticeValue::Int(i));
            let back = ffi.to_internal();
            assert_eq!(back.as_int(), Some(i));
        }
    }

    #[test]
    fn test_float_roundtrip() {
        for f in [0.0f64, 3.14, -2.718, f64::MAX, f64::MIN] {
            let internal = Value::Float(f);
            let ffi = LatticeValue::from_internal(&internal).unwrap();
            assert_eq!(ffi, LatticeValue::Float(f));
            let back = ffi.to_internal();
            assert_eq!(back.as_float(), Some(f));
        }
    }

    #[test]
    fn test_string_roundtrip() {
        let test_strings = ["", "hello", "Hello, 世界!", "with\nnewline"];
        for s in test_strings {
            let internal = Value::string(s);
            let ffi = LatticeValue::from_internal(&internal).unwrap();
            assert_eq!(ffi, LatticeValue::String(s.to_string()));
            let back = ffi.to_internal();
            assert_eq!(back.as_string(), Some(s));
        }
    }

    #[test]
    fn test_path_roundtrip() {
        let path = "/home/user/file.txt";
        let internal = Value::path(PathBuf::from(path));
        let ffi = LatticeValue::from_internal(&internal).unwrap();
        assert_eq!(ffi, LatticeValue::Path(path.to_string()));
        let back = ffi.to_internal();
        assert_eq!(back.as_path().map(|p| p.to_string_lossy().into_owned()), Some(path.to_string()));
    }

    #[test]
    fn test_list_roundtrip() {
        let internal = Value::list(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
        ]);
        let ffi = LatticeValue::from_internal(&internal).unwrap();
        assert_eq!(
            ffi,
            LatticeValue::List(vec![
                LatticeValue::Int(1),
                LatticeValue::Int(2),
                LatticeValue::Int(3),
            ])
        );
        let back = ffi.to_internal();
        let list = back.as_list().unwrap();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].as_int(), Some(1));
        assert_eq!(list[1].as_int(), Some(2));
        assert_eq!(list[2].as_int(), Some(3));
    }

    #[test]
    fn test_nested_list_roundtrip() {
        let internal = Value::list(vec![
            Value::list(vec![Value::Int(1), Value::Int(2)]),
            Value::list(vec![Value::Int(3), Value::Int(4)]),
        ]);
        let ffi = LatticeValue::from_internal(&internal).unwrap();
        assert_eq!(
            ffi,
            LatticeValue::List(vec![
                LatticeValue::List(vec![LatticeValue::Int(1), LatticeValue::Int(2)]),
                LatticeValue::List(vec![LatticeValue::Int(3), LatticeValue::Int(4)]),
            ])
        );
        let back = ffi.to_internal();
        let outer = back.as_list().unwrap();
        assert_eq!(outer.len(), 2);
        let inner1 = outer[0].as_list().unwrap();
        assert_eq!(inner1.len(), 2);
    }

    #[test]
    fn test_map_roundtrip() {
        let mut map = HashMap::new();
        map.insert("a".to_string(), Value::Int(1));
        map.insert("b".to_string(), Value::string("hello"));
        let internal = Value::map(map);

        let ffi = LatticeValue::from_internal(&internal).unwrap();

        // Map should be sorted by key
        assert_eq!(
            ffi,
            LatticeValue::Map(vec![
                ("a".to_string(), LatticeValue::Int(1)),
                ("b".to_string(), LatticeValue::String("hello".to_string())),
            ])
        );

        let back = ffi.to_internal();
        let back_map = back.as_map().unwrap();
        assert_eq!(back_map.get("a").and_then(|v| v.as_int()), Some(1));
        assert_eq!(back_map.get("b").and_then(|v| v.as_string()), Some("hello"));
    }

    #[test]
    fn test_map_deterministic_order() {
        // Create map with keys that would hash differently
        let mut map = HashMap::new();
        map.insert("zebra".to_string(), Value::Int(1));
        map.insert("apple".to_string(), Value::Int(2));
        map.insert("mango".to_string(), Value::Int(3));
        let internal = Value::map(map);

        let ffi = LatticeValue::from_internal(&internal).unwrap();

        // Should be sorted alphabetically
        if let LatticeValue::Map(pairs) = &ffi {
            assert_eq!(pairs[0].0, "apple");
            assert_eq!(pairs[1].0, "mango");
            assert_eq!(pairs[2].0, "zebra");
        } else {
            panic!("Expected Map");
        }
    }

    #[test]
    fn test_complex_nested_structure() {
        // Test a complex nested structure: map containing lists and nested maps
        let mut inner_map = HashMap::new();
        inner_map.insert("nested".to_string(), Value::Bool(true));

        let mut outer_map = HashMap::new();
        outer_map.insert("numbers".to_string(), Value::list(vec![
            Value::Int(1),
            Value::Int(2),
        ]));
        outer_map.insert("inner".to_string(), Value::map(inner_map));
        outer_map.insert("name".to_string(), Value::string("test"));

        let internal = Value::map(outer_map);
        let ffi = LatticeValue::from_internal(&internal).unwrap();

        // Verify structure preserved
        assert!(ffi.get("numbers").is_some());
        assert!(ffi.get("inner").is_some());
        assert!(ffi.get("name").is_some());

        // Verify nested access
        let inner = ffi.get("inner").unwrap();
        assert_eq!(inner.get("nested"), Some(&LatticeValue::Bool(true)));

        // Roundtrip
        let back = ffi.to_internal();
        let back_map = back.as_map().unwrap();
        assert_eq!(back_map.len(), 3);
    }

    #[test]
    fn test_from_impls() {
        assert_eq!(LatticeValue::from(true), LatticeValue::Bool(true));
        assert_eq!(LatticeValue::from(42i64), LatticeValue::Int(42));
        assert_eq!(LatticeValue::from(42i32), LatticeValue::Int(42));
        assert_eq!(LatticeValue::from(3.14f64), LatticeValue::Float(3.14));
        assert_eq!(LatticeValue::from("hello"), LatticeValue::String("hello".to_string()));
        assert_eq!(LatticeValue::from("hello".to_string()), LatticeValue::String("hello".to_string()));
        assert_eq!(LatticeValue::from(()), LatticeValue::Null);

        let list: LatticeValue = vec![1i64, 2i64, 3i64].into();
        assert_eq!(list, LatticeValue::List(vec![
            LatticeValue::Int(1),
            LatticeValue::Int(2),
            LatticeValue::Int(3),
        ]));
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", LatticeValue::Null), "null");
        assert_eq!(format!("{}", LatticeValue::Bool(true)), "true");
        assert_eq!(format!("{}", LatticeValue::Int(42)), "42");
        assert_eq!(format!("{}", LatticeValue::Float(3.14)), "3.14");
        assert_eq!(format!("{}", LatticeValue::String("hello".to_string())), "\"hello\"");
        assert_eq!(format!("{}", LatticeValue::Path("/tmp".to_string())), "Path(\"/tmp\")");

        let list = LatticeValue::List(vec![LatticeValue::Int(1), LatticeValue::Int(2)]);
        assert_eq!(format!("{}", list), "[1, 2]");

        let map = LatticeValue::Map(vec![
            ("a".to_string(), LatticeValue::Int(1)),
            ("b".to_string(), LatticeValue::Int(2)),
        ]);
        assert_eq!(format!("{}", map), "{\"a\": 1, \"b\": 2}");
    }

    #[test]
    fn test_accessors() {
        let null = LatticeValue::Null;
        assert!(null.is_null());
        assert!(null.as_bool().is_none());

        let b = LatticeValue::Bool(true);
        assert!(!b.is_null());
        assert_eq!(b.as_bool(), Some(true));
        assert!(b.as_int().is_none());

        let i = LatticeValue::Int(42);
        assert_eq!(i.as_int(), Some(42));
        assert!(i.as_float().is_none());

        let f = LatticeValue::Float(3.14);
        assert_eq!(f.as_float(), Some(3.14));
        assert!(f.as_str().is_none());

        let s = LatticeValue::String("test".to_string());
        assert_eq!(s.as_str(), Some("test"));
        assert!(s.as_list().is_none());

        let list = LatticeValue::List(vec![LatticeValue::Int(1)]);
        assert_eq!(list.as_list().map(|l| l.len()), Some(1));
        assert!(list.as_map().is_none());

        let map = LatticeValue::Map(vec![("key".to_string(), LatticeValue::Int(42))]);
        assert_eq!(map.as_map().map(|m| m.len()), Some(1));
        assert_eq!(map.get("key"), Some(&LatticeValue::Int(42)));
        assert_eq!(map.get("missing"), None);
    }

    #[test]
    fn test_serde_json_roundtrip() {
        let values = vec![
            LatticeValue::Null,
            LatticeValue::Bool(true),
            LatticeValue::Int(42),
            LatticeValue::Float(3.14),
            LatticeValue::String("hello".to_string()),
            LatticeValue::Path("/tmp/test".to_string()),
            LatticeValue::List(vec![LatticeValue::Int(1), LatticeValue::Int(2)]),
            LatticeValue::Map(vec![
                ("key".to_string(), LatticeValue::String("value".to_string())),
            ]),
        ];

        for value in values {
            let json = serde_json::to_string(&value).unwrap();
            let back: LatticeValue = serde_json::from_str(&json).unwrap();
            assert_eq!(value, back, "Failed for: {:?}", value);
        }
    }
}
