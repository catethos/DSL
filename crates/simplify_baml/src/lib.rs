/// Simplified BAML Runtime
///
/// A minimal implementation of BAML (Basically A Markup Language) runtime
/// that demonstrates the core concepts:
/// 1. IR (Intermediate Representation) - Type definitions
/// 2. Schema Formatter - Convert types to human-readable schemas
/// 3. Template Renderer - Jinja2 template rendering with schema injection
/// 4. HTTP Client - Simple LLM API wrapper
/// 5. Parser - Lenient JSON parsing with type coercion
/// 6. Runtime - Orchestration of the above components
///
/// This is a simplified version that reduces the ~50K line BAML codebase
/// to ~5K lines by focusing only on the essential functionality.
pub mod ir;
pub mod schema;
pub mod renderer;
pub mod client;
pub mod parser;
pub mod runtime;
pub mod registry;
pub mod partial_parser;
pub mod streaming_value;

// Re-export commonly used types
pub use ir::{BamlSchema, BamlValue, Class, Enum, Field, FieldType, Function, IR};
pub use client::LLMClient;
pub use runtime::{BamlRuntime, RuntimeBuilder, generate_prompt_from_ir, parse_llm_response_with_ir, try_parse_partial_response, update_streaming_response};
pub use registry::BamlSchemaRegistry;
pub use partial_parser::try_parse_partial_json;
pub use streaming_value::{StreamingBamlValue, CompletionState};

// Re-export the derive macros and attribute macros
pub use simplify_baml_macros::{BamlSchema as DeriveBamlSchema, BamlClient as DeriveBamlClient, baml_function};

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_simple_ir_construction() {
        let mut ir = IR::new();

        ir.classes.push(Class {
            name: "Person".to_string(),
            description: None,
            fields: vec![
                Field {
                    name: "name".to_string(),
                    field_type: FieldType::String,
                    optional: false,
                    description: None,
                },
            ],
        });

        assert!(ir.find_class("Person").is_some());
    }

    #[test]
    fn test_baml_value_conversions() {
        let value = BamlValue::String("test".to_string());
        assert_eq!(value.as_string(), Some("test"));

        let value = BamlValue::Int(42);
        assert_eq!(value.as_int(), Some(42));

        let mut map = HashMap::new();
        map.insert("key".to_string(), BamlValue::String("value".to_string()));
        let value = BamlValue::Map(map);
        assert!(value.as_map().is_some());
    }
}
