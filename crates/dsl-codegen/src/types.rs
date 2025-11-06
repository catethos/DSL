use crate::rust_ast::*;
use dsl_ir::{Class, Enum, FieldType};

pub fn generate_struct(class: &Class) -> RustStruct {
    RustStruct {
        name: class.name.clone(),
        derives: vec![
            "Debug".to_string(),
            "Clone".to_string(),
            "serde::Serialize".to_string(),
            "serde::Deserialize".to_string(),
        ],
        fields: class
            .fields
            .iter()
            .map(|f| RustField {
                name: f.name.clone(),
                ty: field_type_to_rust(&f.field_type, f.optional),
                is_public: true,
            })
            .collect(),
        is_public: true,
    }
}

pub fn generate_enum(enum_def: &Enum) -> RustEnum {
    RustEnum {
        name: enum_def.name.clone(),
        derives: vec![
            "Debug".to_string(),
            "Clone".to_string(),
            "serde::Serialize".to_string(),
            "serde::Deserialize".to_string(),
        ],
        variants: enum_def
            .values
            .iter()
            .map(|v| RustVariant {
                name: v.clone(),
                fields: None,
            })
            .collect(),
        is_public: true,
    }
}

pub fn field_type_to_rust(field_type: &FieldType, optional: bool) -> RustType {
    let base = match field_type {
        FieldType::String => RustType::Named("String".to_string()),
        FieldType::Int => RustType::Named("i64".to_string()),
        FieldType::Float => RustType::Named("f64".to_string()),
        FieldType::Bool => RustType::Named("bool".to_string()),
        FieldType::List(inner) => {
            RustType::Generic("Vec".to_string(), vec![field_type_to_rust(inner, false)])
        }
        FieldType::Class(name) => RustType::Named(name.clone()),
        FieldType::Enum(name) => RustType::Named(name.clone()),
        FieldType::Map(key, value) => RustType::Generic(
            "indexmap::IndexMap".to_string(),
            vec![
                field_type_to_rust(key, false),
                field_type_to_rust(value, false),
            ],
        ),
        FieldType::Union(_types) => {
            RustType::Named("Value".to_string())
        }
    };

    if optional {
        RustType::Generic("Option".to_string(), vec![base])
    } else {
        base
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dsl_ir::Field;

    #[test]
    fn test_generate_simple_struct() {
        let class = Class {
            name: "Person".to_string(),
            description: None,
            fields: vec![
                Field {
                    name: "name".to_string(),
                    description: None,
                    field_type: FieldType::String,
                    optional: false,
                },
                Field {
                    name: "age".to_string(),
                    description: None,
                    field_type: FieldType::Int,
                    optional: false,
                },
            ],
        };

        let rust_struct = generate_struct(&class);
        assert_eq!(rust_struct.name, "Person");
        assert_eq!(rust_struct.fields.len(), 2);
        assert_eq!(rust_struct.fields[0].name, "name");
        assert_eq!(rust_struct.fields[1].name, "age");
    }

    #[test]
    fn test_generate_simple_enum() {
        let enum_def = Enum {
            name: "Status".to_string(),
            description: None,
            values: vec!["Active".to_string(), "Inactive".to_string()],
        };

        let rust_enum = generate_enum(&enum_def);
        assert_eq!(rust_enum.name, "Status");
        assert_eq!(rust_enum.variants.len(), 2);
        assert_eq!(rust_enum.variants[0].name, "Active");
        assert_eq!(rust_enum.variants[1].name, "Inactive");
    }

    #[test]
    fn test_optional_fields() {
        let field_type = FieldType::String;
        let rust_type = field_type_to_rust(&field_type, true);

        match rust_type {
            RustType::Generic(name, args) => {
                assert_eq!(name, "Option");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected Option type"),
        }
    }
}
