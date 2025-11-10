/// Simplified Intermediate Representation (IR) for BAML
///
/// This module defines the core types that represent a BAML program:
/// - Classes (structured types with fields)
/// - Enums (enumerated types with variants)
/// - Functions (LLM functions with inputs and outputs)
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export type definitions from dsl-types
pub use dsl_types::{Class, Enum, Field, FieldType};

/// The root IR containing all type definitions and functions
#[derive(Debug, Clone)]
pub struct IR {
    pub classes: Vec<Class>,
    pub enums: Vec<Enum>,
    pub functions: Vec<Function>,
}

impl Default for IR {
    fn default() -> Self {
        Self::new()
    }
}

impl IR {
    pub fn new() -> Self {
        Self {
            classes: Vec::new(),
            enums: Vec::new(),
            functions: Vec::new(),
        }
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

// Class, Enum, Field, and FieldType are now re-exported from dsl-types above

/// A function represents an LLM function call
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub inputs: Vec<Field>,
    pub output: FieldType,
    pub prompt_template: String,
    pub client: String,
}

/// Runtime value that can be passed to functions or returned from LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BamlValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    List(Vec<BamlValue>),
    Map(HashMap<String, BamlValue>),
    Null,
}

impl BamlValue {
    pub fn as_string(&self) -> Option<&str> {
        match self {
            BamlValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            BamlValue::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            BamlValue::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            BamlValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&Vec<BamlValue>> {
        match self {
            BamlValue::List(l) => Some(l),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&HashMap<String, BamlValue>> {
        match self {
            BamlValue::Map(m) => Some(m),
            _ => None,
        }
    }
}

/// Trait for types that can be converted to BAML IR
///
/// This trait is automatically implemented by the `#[derive(BamlSchema)]` macro.
/// It allows automatic generation of IR from Rust type definitions.
///
/// # Example
/// ```ignore
/// use simplify_baml::*;
///
/// #[derive(BamlSchema)]
/// #[baml(description = "A person entity")]
/// struct Person {
///     name: String,
///     age: i64,
/// }
///
/// // Automatically register into IR
/// let ir = BamlSchemaRegistry::new()
///     .register::<Person>()
///     .build();
/// ```
pub trait BamlSchema {
    /// Returns the name of this schema type
    fn schema_name() -> &'static str;

    /// Registers this schema and all its dependencies into the registry
    fn register_schemas(registry: &mut crate::registry::BamlSchemaRegistry);
}
