use serde::{Deserialize, Serialize};

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
    Class(String),
    Enum(String),
    List(Box<FieldType>),
    Map(Box<FieldType>, Box<FieldType>),
    Union(Vec<FieldType>),
}

impl std::fmt::Display for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldType::String => write!(f, "string"),
            FieldType::Int => write!(f, "int"),
            FieldType::Float => write!(f, "float"),
            FieldType::Bool => write!(f, "bool"),
            FieldType::Class(name) => write!(f, "{}", name),
            FieldType::Enum(name) => write!(f, "{}", name),
            FieldType::List(inner) => write!(f, "[{}]", inner),
            FieldType::Map(k, v) => write!(f, "map<{}, {}>", k, v),
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
