//! FFI-safe type schema for exporting Lattice types to host languages.
//!
//! This module provides `TypeSchema`, an FFI-friendly representation of
//! Lattice types that can be serialized to JSON and used to generate
//! typed structures in host languages (Elixir, Python, etc.).

use serde::{Deserialize, Serialize};

use crate::types::{Class, Enum, Field, FieldType};

/// FFI-safe representation of a Lattice type.
///
/// Designed for easy serialization and use in host languages:
/// - Generates typed structs/classes from Lattice types
/// - Validates values before passing to Lattice
/// - Displays type information in IDEs
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data")]
pub enum TypeSchema {
    // Primitives
    Null,
    Bool,
    Int,
    Float,
    String,
    Path,

    // Composite types
    List(Box<TypeSchema>),
    Map {
        key: Box<TypeSchema>,
        value: Box<TypeSchema>,
    },
    Optional(Box<TypeSchema>),

    // User-defined types
    Struct(StructSchema),
    Enum(EnumSchema),

    // Reference to a named type (not yet resolved)
    Named(String),

    // Special
    Any,
}

/// Schema for a struct type
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StructSchema {
    pub name: String,
    pub fields: Vec<FieldSchema>,
    pub description: Option<String>,
}

/// Schema for a field in a struct
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FieldSchema {
    pub name: String,
    pub type_schema: TypeSchema,
    pub optional: bool,
    pub description: Option<String>,
}

/// Schema for an enum type
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EnumSchema {
    pub name: String,
    pub variants: Vec<String>,
    pub description: Option<String>,
}

impl TypeSchema {
    /// Convert from internal IR FieldType to TypeSchema
    pub fn from_field_type(ft: &FieldType) -> Self {
        match ft {
            FieldType::String => TypeSchema::String,
            FieldType::Int => TypeSchema::Int,
            FieldType::Float => TypeSchema::Float,
            FieldType::Bool => TypeSchema::Bool,
            FieldType::Path => TypeSchema::Path,
            FieldType::Class(name) => TypeSchema::Named(name.clone()),
            FieldType::Enum(name) => TypeSchema::Named(name.clone()),
            FieldType::List(inner) => {
                TypeSchema::List(Box::new(Self::from_field_type(inner)))
            }
            FieldType::Map(k, v) => TypeSchema::Map {
                key: Box::new(Self::from_field_type(k)),
                value: Box::new(Self::from_field_type(v)),
            },
            FieldType::Union(_) => {
                // Union types are represented as Any for now
                // A full implementation would create a Union variant
                TypeSchema::Any
            }
        }
    }

    /// Convert from internal IR Class to TypeSchema
    pub fn from_class(class: &Class) -> Self {
        TypeSchema::Struct(StructSchema {
            name: class.name.clone(),
            fields: class
                .fields
                .iter()
                .map(|f| FieldSchema {
                    name: f.name.clone(),
                    type_schema: Self::from_field_type(&f.field_type),
                    optional: f.optional,
                    description: f.description.clone(),
                })
                .collect(),
            description: class.description.clone(),
        })
    }

    /// Convert from internal IR Enum to TypeSchema
    pub fn from_enum(e: &Enum) -> Self {
        TypeSchema::Enum(EnumSchema {
            name: e.name.clone(),
            variants: e.values.clone(),
            description: e.description.clone(),
        })
    }

    /// Convert TypeSchema back to internal IR Class
    ///
    /// Returns None if this is not a Struct schema
    pub fn to_class(&self) -> Option<Class> {
        match self {
            TypeSchema::Struct(schema) => Some(Class {
                name: schema.name.clone(),
                description: schema.description.clone(),
                fields: schema
                    .fields
                    .iter()
                    .map(|f| Field {
                        name: f.name.clone(),
                        field_type: Self::to_field_type(&f.type_schema),
                        optional: f.optional,
                        description: f.description.clone(),
                    })
                    .collect(),
            }),
            _ => None,
        }
    }

    /// Convert TypeSchema back to internal IR Enum
    ///
    /// Returns None if this is not an Enum schema
    pub fn to_enum(&self) -> Option<Enum> {
        match self {
            TypeSchema::Enum(schema) => Some(Enum {
                name: schema.name.clone(),
                description: schema.description.clone(),
                values: schema.variants.clone(),
            }),
            _ => None,
        }
    }

    /// Convert TypeSchema to internal IR FieldType
    fn to_field_type(schema: &TypeSchema) -> FieldType {
        match schema {
            TypeSchema::Null => FieldType::String, // No direct null type in FieldType
            TypeSchema::Bool => FieldType::Bool,
            TypeSchema::Int => FieldType::Int,
            TypeSchema::Float => FieldType::Float,
            TypeSchema::String => FieldType::String,
            TypeSchema::Path => FieldType::Path,
            TypeSchema::List(inner) => {
                FieldType::List(Box::new(Self::to_field_type(inner)))
            }
            TypeSchema::Map { key, value } => FieldType::Map(
                Box::new(Self::to_field_type(key)),
                Box::new(Self::to_field_type(value)),
            ),
            TypeSchema::Optional(inner) => {
                // Optional becomes Union with null in FieldType
                FieldType::Union(vec![
                    Self::to_field_type(inner),
                    FieldType::String, // Using String as proxy for null
                ])
            }
            TypeSchema::Struct(s) => FieldType::Class(s.name.clone()),
            TypeSchema::Enum(e) => FieldType::Enum(e.name.clone()),
            TypeSchema::Named(name) => FieldType::Class(name.clone()),
            TypeSchema::Any => FieldType::String, // Fallback
        }
    }

    /// Check if this schema represents a primitive type
    pub fn is_primitive(&self) -> bool {
        matches!(
            self,
            TypeSchema::Null
                | TypeSchema::Bool
                | TypeSchema::Int
                | TypeSchema::Float
                | TypeSchema::String
                | TypeSchema::Path
        )
    }

    /// Check if this schema represents a composite type
    pub fn is_composite(&self) -> bool {
        matches!(
            self,
            TypeSchema::List(_)
                | TypeSchema::Map { .. }
                | TypeSchema::Optional(_)
                | TypeSchema::Struct(_)
                | TypeSchema::Enum(_)
        )
    }

    /// Get the name if this is a named/struct/enum type
    pub fn type_name(&self) -> Option<&str> {
        match self {
            TypeSchema::Struct(s) => Some(&s.name),
            TypeSchema::Enum(e) => Some(&e.name),
            TypeSchema::Named(n) => Some(n),
            _ => None,
        }
    }
}

impl std::fmt::Display for TypeSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeSchema::Null => write!(f, "Null"),
            TypeSchema::Bool => write!(f, "Bool"),
            TypeSchema::Int => write!(f, "Int"),
            TypeSchema::Float => write!(f, "Float"),
            TypeSchema::String => write!(f, "String"),
            TypeSchema::Path => write!(f, "Path"),
            TypeSchema::List(inner) => write!(f, "[{}]", inner),
            TypeSchema::Map { key, value } => write!(f, "Map<{}, {}>", key, value),
            TypeSchema::Optional(inner) => write!(f, "{}?", inner),
            TypeSchema::Struct(s) => write!(f, "{}", s.name),
            TypeSchema::Enum(e) => write!(f, "{}", e.name),
            TypeSchema::Named(n) => write!(f, "{}", n),
            TypeSchema::Any => write!(f, "Any"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_schema() {
        assert_eq!(TypeSchema::from_field_type(&FieldType::String), TypeSchema::String);
        assert_eq!(TypeSchema::from_field_type(&FieldType::Int), TypeSchema::Int);
        assert_eq!(TypeSchema::from_field_type(&FieldType::Float), TypeSchema::Float);
        assert_eq!(TypeSchema::from_field_type(&FieldType::Bool), TypeSchema::Bool);
        assert_eq!(TypeSchema::from_field_type(&FieldType::Path), TypeSchema::Path);
    }

    #[test]
    fn test_list_schema() {
        let list_type = FieldType::List(Box::new(FieldType::Int));
        let schema = TypeSchema::from_field_type(&list_type);
        assert_eq!(schema, TypeSchema::List(Box::new(TypeSchema::Int)));
    }

    #[test]
    fn test_map_schema() {
        let map_type = FieldType::Map(
            Box::new(FieldType::String),
            Box::new(FieldType::Int),
        );
        let schema = TypeSchema::from_field_type(&map_type);
        assert_eq!(
            schema,
            TypeSchema::Map {
                key: Box::new(TypeSchema::String),
                value: Box::new(TypeSchema::Int),
            }
        );
    }

    #[test]
    fn test_class_to_struct_schema() {
        let class = Class {
            name: "Person".to_string(),
            description: Some("A person".to_string()),
            fields: vec![
                Field {
                    name: "name".to_string(),
                    field_type: FieldType::String,
                    optional: false,
                    description: None,
                },
                Field {
                    name: "age".to_string(),
                    field_type: FieldType::Int,
                    optional: false,
                    description: None,
                },
            ],
        };

        let schema = TypeSchema::from_class(&class);
        match schema {
            TypeSchema::Struct(s) => {
                assert_eq!(s.name, "Person");
                assert_eq!(s.fields.len(), 2);
                assert_eq!(s.fields[0].name, "name");
                assert_eq!(s.fields[0].type_schema, TypeSchema::String);
                assert_eq!(s.fields[1].name, "age");
                assert_eq!(s.fields[1].type_schema, TypeSchema::Int);
            }
            _ => panic!("Expected Struct schema"),
        }
    }

    #[test]
    fn test_enum_to_schema() {
        let enum_def = Enum {
            name: "Color".to_string(),
            description: None,
            values: vec!["Red".to_string(), "Green".to_string(), "Blue".to_string()],
        };

        let schema = TypeSchema::from_enum(&enum_def);
        match schema {
            TypeSchema::Enum(e) => {
                assert_eq!(e.name, "Color");
                assert_eq!(e.variants, vec!["Red", "Green", "Blue"]);
            }
            _ => panic!("Expected Enum schema"),
        }
    }

    #[test]
    fn test_struct_schema_roundtrip() {
        let original = Class {
            name: "Test".to_string(),
            description: Some("Test class".to_string()),
            fields: vec![Field {
                name: "value".to_string(),
                field_type: FieldType::Int,
                optional: false,
                description: Some("The value".to_string()),
            }],
        };

        let schema = TypeSchema::from_class(&original);
        let back = schema.to_class().expect("Should convert back to class");

        assert_eq!(back.name, original.name);
        assert_eq!(back.description, original.description);
        assert_eq!(back.fields.len(), original.fields.len());
        assert_eq!(back.fields[0].name, original.fields[0].name);
    }

    #[test]
    fn test_json_serialization() {
        let schema = TypeSchema::Struct(StructSchema {
            name: "Person".to_string(),
            fields: vec![
                FieldSchema {
                    name: "name".to_string(),
                    type_schema: TypeSchema::String,
                    optional: false,
                    description: None,
                },
            ],
            description: None,
        });

        let json = serde_json::to_string(&schema).expect("Should serialize");
        let back: TypeSchema = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(schema, back);
    }

    #[test]
    fn test_display() {
        assert_eq!(TypeSchema::String.to_string(), "String");
        assert_eq!(TypeSchema::List(Box::new(TypeSchema::Int)).to_string(), "[Int]");
        assert_eq!(
            TypeSchema::Map {
                key: Box::new(TypeSchema::String),
                value: Box::new(TypeSchema::Int),
            }
            .to_string(),
            "Map<String, Int>"
        );
    }

    #[test]
    fn test_is_primitive() {
        assert!(TypeSchema::String.is_primitive());
        assert!(TypeSchema::Int.is_primitive());
        assert!(!TypeSchema::List(Box::new(TypeSchema::Int)).is_primitive());
    }

    #[test]
    fn test_is_composite() {
        assert!(!TypeSchema::String.is_composite());
        assert!(TypeSchema::List(Box::new(TypeSchema::Int)).is_composite());
        assert!(TypeSchema::Struct(StructSchema {
            name: "Test".to_string(),
            fields: vec![],
            description: None,
        }).is_composite());
    }
}
