//! Intermediate Representation (IR) for Lattice
//!
//! This module defines the core types that represent a Lattice program:
//! - Classes (structured types with fields)
//! - Enums (enumerated types with variants)
//! - Functions (LLM functions with inputs and outputs)
//! - Values (runtime values)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

// ============================================================================
// Type Definitions (inlined from dsl-types)
// ============================================================================

/// A class represents a structured type (like a struct)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Class {
    pub name: String,
    pub description: Option<String>,
    pub fields: Vec<Field>,
}

/// A field in a class
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub field_type: FieldType,
    pub optional: bool,
    pub description: Option<String>,
}

/// Field type representation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FieldType {
    String,
    Int,
    Float,
    Bool,
    Path,
    Class(String),
    Enum(String),
    List(Box<FieldType>),
    Map(Box<FieldType>, Box<FieldType>),
    Union(Vec<FieldType>),
}

impl std::fmt::Display for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldType::String => write!(f, "String"),
            FieldType::Int => write!(f, "Int"),
            FieldType::Float => write!(f, "Float"),
            FieldType::Bool => write!(f, "Bool"),
            FieldType::Path => write!(f, "Path"),
            FieldType::Class(name) => write!(f, "{}", name),
            FieldType::Enum(name) => write!(f, "{}", name),
            FieldType::List(inner) => write!(f, "[{}]", inner),
            FieldType::Map(k, v) => write!(f, "Map<{}, {}>", k, v),
            FieldType::Union(types) => {
                let type_strings: Vec<String> = types.iter().map(|t| t.to_string()).collect();
                write!(f, "{}", type_strings.join(" | "))
            }
        }
    }
}

/// An enum represents a type with a fixed set of variants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enum {
    pub name: String,
    pub description: Option<String>,
    pub values: Vec<String>,
}

// ============================================================================
// IR (Intermediate Representation)
// ============================================================================

/// The root IR containing all type definitions and functions
#[derive(Debug, Clone, Default)]
pub struct IR {
    pub classes: Vec<Class>,
    pub enums: Vec<Enum>,
    pub functions: Vec<Function>,
}

impl IR {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn find_class(&self, name: &str) -> Option<&Class> {
        self.classes.iter().find(|c| c.name == name)
    }

    pub fn find_enum(&self, name: &str) -> Option<&Enum> {
        self.enums.iter().find(|e| e.name == name)
    }

    pub fn find_function(&self, name: &str) -> Option<&Function> {
        self.functions.iter().find(|f| f.name == name)
    }
}

/// A function represents an LLM function call
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub inputs: Vec<Field>,
    pub output: FieldType,
    pub prompt_template: String,
    pub client: String,
}

// ============================================================================
// Runtime Values
// ============================================================================

/// Runtime value that can be passed to functions or returned from LLM
///
/// Uses Arc for heap-allocated variants to make cloning O(1) instead of O(n).
/// This significantly improves VM performance for stack operations.
/// Arc is used instead of Rc to ensure thread safety (Send + Sync).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    String(Arc<str>),
    Int(i64),
    Float(f64),
    Bool(bool),
    Path(Arc<PathBuf>),
    List(Arc<Vec<Value>>),
    Map(Arc<HashMap<String, Value>>),
    Null,
}

impl Value {
    // Constructors for convenient Value creation

    /// Create a String value from anything that can be converted to a string
    pub fn string(s: impl Into<Arc<str>>) -> Self {
        Value::String(s.into())
    }

    /// Create a List value from a Vec
    pub fn list(v: Vec<Value>) -> Self {
        Value::List(Arc::new(v))
    }

    /// Create a Map value from a HashMap
    pub fn map(m: HashMap<String, Value>) -> Self {
        Value::Map(Arc::new(m))
    }

    /// Create a Path value from a PathBuf
    pub fn path(p: PathBuf) -> Self {
        Value::Path(Arc::new(p))
    }

    // Accessors

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&Vec<Value>> {
        match self {
            Value::List(l) => Some(l),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Value::Map(m) => Some(m),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    pub fn as_path(&self) -> Option<&PathBuf> {
        match self {
            Value::Path(p) => Some(p),
            _ => None,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Path(p) => write!(f, "Path(\"{}\")", p.display()),
            Value::List(l) => {
                write!(f, "[")?;
                for (i, v) in l.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Value::Map(m) => {
                write!(f, "{{")?;
                for (i, (k, v)) in m.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {}", k, v)?;
                }
                write!(f, "}}")
            }
            Value::Null => write!(f, "null"),
        }
    }
}

// From implementations for convenient Value construction

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s.into())
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.into())
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Value::Int(i)
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::Float(f)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<PathBuf> for Value {
    fn from(p: PathBuf) -> Self {
        Value::Path(Arc::new(p))
    }
}

impl From<Vec<Value>> for Value {
    fn from(v: Vec<Value>) -> Self {
        Value::List(Arc::new(v))
    }
}

impl From<HashMap<String, Value>> for Value {
    fn from(m: HashMap<String, Value>) -> Self {
        Value::Map(Arc::new(m))
    }
}
