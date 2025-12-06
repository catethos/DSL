//! Streaming-aware Value that always returns the full schema shape
//!
//! This module provides a way to create "skeleton" values from IR and progressively
//! fill them as streaming data arrives. The UI always gets a consistent structure.

use crate::types::ir::{Class, FieldType, Value, IR};
use serde::{Serialize, Serializer};
use std::collections::HashMap;

/// A Value with completion state tracking
#[derive(Debug, Clone)]
pub struct StreamingValue {
    pub value: Value,
    pub completion_state: CompletionState,
}

/// Tracks which parts of the value are complete
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CompletionState {
    /// Fully received and parsed
    Complete,
    /// Partially received, may update
    Partial,
    /// Not yet received
    Pending,
}

impl Serialize for StreamingValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("StreamingValue", 2)?;
        state.serialize_field("value", &self.value)?;
        state.serialize_field("state", &self.completion_state)?;
        state.end()
    }
}

impl StreamingValue {
    /// Create a new streaming value with initial state
    pub fn new(value: Value, state: CompletionState) -> Self {
        Self {
            value,
            completion_state: state,
        }
    }

    /// Create a skeleton value from IR with all fields marked as Pending
    pub fn from_ir_skeleton(ir: &IR, field_type: &FieldType) -> Self {
        let value = create_skeleton_value(ir, field_type);
        Self::new(value, CompletionState::Pending)
    }

    /// Update this value with new partial data
    pub fn update_from_partial(&mut self, ir: &IR, partial_value: Value, field_type: &FieldType) {
        merge_values(&mut self.value, partial_value, ir, field_type);
        self.completion_state = CompletionState::Partial;
    }

    /// Mark this value as complete
    pub fn mark_complete(&mut self) {
        self.completion_state = CompletionState::Complete;
    }
}

/// Create a skeleton Value filled with defaults based on IR
fn create_skeleton_value(ir: &IR, field_type: &FieldType) -> Value {
    match field_type {
        FieldType::String => Value::Null,
        FieldType::Int => Value::Null,
        FieldType::Float => Value::Null,
        FieldType::Bool => Value::Null,
        FieldType::Enum(_) => Value::Null,

        FieldType::Class(class_name) => {
            if let Some(class) = ir.find_class(class_name) {
                create_skeleton_class(ir, class)
            } else {
                Value::Null
            }
        }

        FieldType::List(_) => Value::list(vec![]),

        FieldType::Map(_, _) => Value::map(HashMap::new()),

        FieldType::Union(types) => {
            // For unions, create skeleton of the first type
            if let Some(first_type) = types.first() {
                create_skeleton_value(ir, first_type)
            } else {
                Value::Null
            }
        }

        FieldType::Path => Value::Null,
    }
}

/// Create a skeleton for a class with all fields initialized
fn create_skeleton_class(ir: &IR, class: &Class) -> Value {
    let mut fields = HashMap::new();

    for field in &class.fields {
        let field_value = if field.optional {
            Value::Null
        } else {
            create_skeleton_value(ir, &field.field_type)
        };
        fields.insert(field.name.clone(), field_value);
    }

    Value::map(fields)
}

/// Merge partial data into existing skeleton, keeping structure intact
fn merge_values(target: &mut Value, source: Value, ir: &IR, field_type: &FieldType) {
    match (target, source) {
        // Primitives: direct replacement if not null
        (target @ Value::Null, source @ Value::String(_)) => *target = source,
        (target @ Value::Null, source @ Value::Int(_)) => *target = source,
        (target @ Value::Null, source @ Value::Float(_)) => *target = source,
        (target @ Value::Null, source @ Value::Bool(_)) => *target = source,

        // Replace existing primitives with new values
        (target @ Value::String(_), source @ Value::String(_)) => *target = source,
        (target @ Value::Int(_), source @ Value::Int(_)) => *target = source,
        (target @ Value::Float(_), source @ Value::Float(_)) => *target = source,
        (target @ Value::Bool(_), source @ Value::Bool(_)) => *target = source,

        // Maps (including classes): merge field by field
        (Value::Map(target_map), Value::Map(source_map)) => {
            // Get the class definition to know field types
            if let FieldType::Class(class_name) = field_type {
                if let Some(class) = ir.find_class(class_name) {
                    for (key, source_value) in source_map.iter() {
                        if let Some(field) = class.fields.iter().find(|f| &f.name == key) {
                            let target_map_mut = std::sync::Arc::make_mut(target_map);
                            if let Some(target_value) = target_map_mut.get_mut(key) {
                                merge_values(target_value, source_value.clone(), ir, &field.field_type);
                            } else {
                                // Field doesn't exist in target, add it
                                target_map_mut.insert(key.clone(), source_value.clone());
                            }
                        }
                    }
                    return;
                }
            }

            // Fallback: just merge keys
            let target_map_mut = std::sync::Arc::make_mut(target_map);
            for (key, value) in source_map.iter() {
                target_map_mut.insert(key.clone(), value.clone());
            }
        }

        // Lists: replace entire list
        (target @ Value::List(_), source @ Value::List(_)) => *target = source,

        // Source is null: keep target as-is
        (_, Value::Null) => {}

        // Any other combination: replace
        (target, source) => *target = source,
    }
}

/// Trait for creating streaming-aware values
pub trait StreamingCapable {
    /// Create a skeleton value from IR
    fn create_skeleton(ir: &IR, field_type: &FieldType) -> StreamingValue {
        StreamingValue::from_ir_skeleton(ir, field_type)
    }
}

impl StreamingCapable for StreamingValue {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ir::{Field, FieldType};

    #[test]
    fn test_create_skeleton_primitives() {
        let ir = IR::new();

        let skeleton = create_skeleton_value(&ir, &FieldType::String);
        assert!(matches!(skeleton, Value::Null));

        let skeleton = create_skeleton_value(&ir, &FieldType::Int);
        assert!(matches!(skeleton, Value::Null));
    }

    #[test]
    fn test_create_skeleton_class() {
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
                Field {
                    name: "age".to_string(),
                    field_type: FieldType::Int,
                    optional: false,
                    description: None,
                },
            ],
        });

        let skeleton =
            StreamingValue::from_ir_skeleton(&ir, &FieldType::Class("Person".to_string()));

        if let Value::Map(map) = &skeleton.value {
            assert!(map.contains_key("name"));
            assert!(map.contains_key("age"));
            assert!(matches!(map.get("name"), Some(Value::Null)));
            assert!(matches!(map.get("age"), Some(Value::Null)));
        } else {
            panic!("Expected Map");
        }
    }

    #[test]
    fn test_merge_values() {
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
                Field {
                    name: "age".to_string(),
                    field_type: FieldType::Int,
                    optional: false,
                    description: None,
                },
            ],
        });

        let mut skeleton =
            StreamingValue::from_ir_skeleton(&ir, &FieldType::Class("Person".to_string()));

        // First update: partial data with just name
        let mut partial1 = HashMap::new();
        partial1.insert("name".to_string(), Value::string("John"));

        skeleton.update_from_partial(
            &ir,
            Value::map(partial1),
            &FieldType::Class("Person".to_string()),
        );

        if let Value::Map(map) = &skeleton.value {
            assert_eq!(map.get("name").and_then(|v| v.as_string()), Some("John"));
            assert!(matches!(map.get("age"), Some(Value::Null))); // Still null
        } else {
            panic!("Expected Map");
        }

        assert_eq!(skeleton.completion_state, CompletionState::Partial);

        // Second update: add age
        let mut partial2 = HashMap::new();
        partial2.insert("name".to_string(), Value::string("John"));
        partial2.insert("age".to_string(), Value::Int(30));

        skeleton.update_from_partial(
            &ir,
            Value::map(partial2),
            &FieldType::Class("Person".to_string()),
        );

        if let Value::Map(map) = &skeleton.value {
            assert_eq!(map.get("name").and_then(|v| v.as_string()), Some("John"));
            assert_eq!(map.get("age").and_then(|v| v.as_int()), Some(30));
        } else {
            panic!("Expected Map");
        }

        // Mark complete
        skeleton.mark_complete();
        assert_eq!(skeleton.completion_state, CompletionState::Complete);
    }

    #[test]
    fn test_serialization() {
        let value = StreamingValue::new(
            Value::string("test"),
            CompletionState::Partial,
        );

        let json = serde_json::to_string(&value).unwrap();
        assert!(json.contains("\"state\":\"partial\""));
        assert!(json.contains("\"value\""));
    }
}
