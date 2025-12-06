//! JSON parser with type coercion
//!
//! This module implements lenient JSON parsing and type coercion.
//! It attempts to parse LLM outputs that may not be perfectly formatted JSON,
//! and coerces the values to match the expected types.

use anyhow::{Context, Result};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

use crate::types::ir::{FieldType, Value, IR};

pub struct Parser<'a> {
    ir: &'a IR,
}

impl<'a> Parser<'a> {
    pub fn new(ir: &'a IR) -> Self {
        Self { ir }
    }

    /// Parse and coerce a raw LLM response string to a Value
    ///
    /// # Arguments
    /// * `raw_response` - The raw string from the LLM
    /// * `target_type` - The expected output type
    ///
    /// # Returns
    /// The parsed and coerced Value
    pub fn parse(&self, raw_response: &str, target_type: &FieldType) -> Result<Value> {
        // Step 1: Extract JSON from the response (lenient extraction)
        let json_str = self.extract_json(raw_response)?;

        // Step 2: Parse JSON
        let json_value: JsonValue =
            serde_json::from_str(&json_str).context("Failed to parse JSON from LLM response")?;

        // Step 3: Coerce to target type
        self.coerce(&json_value, target_type)
    }

    /// Extract JSON from a response that may contain markdown code blocks or extra text
    fn extract_json(&self, response: &str) -> Result<String> {
        let response = response.trim();

        // Check for markdown code blocks
        if let Some(start) = response.find("```json") {
            let json_start = start + 7; // len("```json")
            // Find the closing ``` after the opening one
            if let Some(end_offset) = response[json_start..].find("```") {
                let json_end = json_start + end_offset;
                return Ok(response[json_start..json_end].trim().to_string());
            }
        }

        // Check for code block without language specifier
        if let Some(start) = response.find("```") {
            if let Some(end) = response[start + 3..].find("```") {
                let json_start = start + 3;
                let json_end = start + 3 + end;
                let content = response[json_start..json_end].trim();
                // Only use this if it looks like JSON
                if content.starts_with('{') || content.starts_with('[') {
                    return Ok(content.to_string());
                }
            }
        }

        // Try to find JSON object or array boundaries
        // Check which comes first: [ or {
        let array_start = response.find('[');
        let object_start = response.find('{');

        // Prioritize arrays over objects if array comes first
        if let (Some(arr_start), Some(obj_start)) = (array_start, object_start) {
            if arr_start < obj_start {
                // Array comes first
                if let Some(end) = response.rfind(']') {
                    if end > arr_start {
                        return Ok(response[arr_start..=end].to_string());
                    }
                }
            } else {
                // Object comes first
                if let Some(end) = response.rfind('}') {
                    if end > obj_start {
                        return Ok(response[obj_start..=end].to_string());
                    }
                }
            }
        } else if let Some(start) = array_start {
            // Only array found
            if let Some(end) = response.rfind(']') {
                if end > start {
                    return Ok(response[start..=end].to_string());
                }
            }
        } else if let Some(start) = object_start {
            // Only object found
            if let Some(end) = response.rfind('}') {
                if end > start {
                    return Ok(response[start..=end].to_string());
                }
            }
        }

        // If nothing found, assume the whole response is JSON
        Ok(response.to_string())
    }

    /// Coerce a JSON value to match the target type
    fn coerce(&self, value: &JsonValue, target_type: &FieldType) -> Result<Value> {
        match target_type {
            FieldType::String => Self::coerce_string(value),
            FieldType::Int => Self::coerce_int(value),
            FieldType::Float => Self::coerce_float(value),
            FieldType::Bool => Self::coerce_bool(value),
            FieldType::Enum(enum_name) => self.coerce_enum(value, enum_name),
            FieldType::Class(class_name) => {
                // Try as class first, then fall back to enum
                // This handles the case where macros generate Class for all custom types
                if self.ir.find_class(class_name).is_some() {
                    self.coerce_class(value, class_name)
                } else if self.ir.find_enum(class_name).is_some() {
                    self.coerce_enum(value, class_name)
                } else {
                    anyhow::bail!("Type '{}' not found (neither class nor enum)", class_name)
                }
            }
            FieldType::List(inner) => self.coerce_list(value, inner),
            FieldType::Map(k, v) => self.coerce_map(value, k, v),
            FieldType::Union(types) => self.coerce_union(value, types),
            FieldType::Path => Self::coerce_path(value),
        }
    }

    fn coerce_path(value: &JsonValue) -> Result<Value> {
        match value {
            JsonValue::String(s) => Ok(Value::path(std::path::PathBuf::from(s))),
            _ => anyhow::bail!("Cannot convert {:?} to Path", value),
        }
    }

    fn coerce_string(value: &JsonValue) -> Result<Value> {
        match value {
            JsonValue::String(s) => Ok(Value::string(s.as_str())),
            JsonValue::Number(n) => Ok(Value::string(n.to_string())),
            JsonValue::Bool(b) => Ok(Value::string(b.to_string())),
            JsonValue::Null => Ok(Value::string("")),
            JsonValue::Object(obj) => {
                // LLM may wrap the value in an object like { "value": "text" } or { "string": "text" }
                // Try common field names
                for field_name in [
                    "value", "Value", "string", "String", "text", "Text", "result", "Result",
                ] {
                    if let Some(inner) = obj.get(field_name) {
                        return Self::coerce_string(inner);
                    }
                }
                // If object has only one field, use that
                if obj.len() == 1 {
                    if let Some((_, inner)) = obj.iter().next() {
                        return Self::coerce_string(inner);
                    }
                }
                anyhow::bail!("Cannot coerce object to string: {:?}", value)
            }
            _ => anyhow::bail!("Cannot coerce {:?} to string", value),
        }
    }

    fn coerce_int(value: &JsonValue) -> Result<Value> {
        match value {
            JsonValue::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(Value::Int(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(Value::Int(f as i64))
                } else {
                    anyhow::bail!("Cannot coerce number to int")
                }
            }
            JsonValue::String(s) => {
                let i = s.parse::<i64>().context("Cannot parse string as int")?;
                Ok(Value::Int(i))
            }
            JsonValue::Object(obj) => {
                // LLM may wrap the value in an object like { "value": 42 } or { "int": 42 }
                // Try common field names
                for field_name in [
                    "value", "Value", "int", "Int", "number", "Number", "result", "Result",
                ] {
                    if let Some(inner) = obj.get(field_name) {
                        return Self::coerce_int(inner);
                    }
                }
                // If object has only one field, use that
                if obj.len() == 1 {
                    if let Some((_, inner)) = obj.iter().next() {
                        return Self::coerce_int(inner);
                    }
                }
                anyhow::bail!("Cannot coerce object to int: {:?}", value)
            }
            _ => anyhow::bail!("Cannot coerce {:?} to int", value),
        }
    }

    fn coerce_float(value: &JsonValue) -> Result<Value> {
        match value {
            JsonValue::Number(n) => {
                if let Some(f) = n.as_f64() {
                    Ok(Value::Float(f))
                } else {
                    anyhow::bail!("Cannot coerce number to float")
                }
            }
            JsonValue::String(s) => {
                let f = s.parse::<f64>().context("Cannot parse string as float")?;
                Ok(Value::Float(f))
            }
            JsonValue::Object(obj) => {
                // LLM may wrap the value in an object like { "value": 3.14 } or { "float": 3.14 }
                // Try common field names
                for field_name in [
                    "value", "Value", "float", "Float", "number", "Number", "result", "Result",
                ] {
                    if let Some(inner) = obj.get(field_name) {
                        return Self::coerce_float(inner);
                    }
                }
                // If object has only one field, use that
                if obj.len() == 1 {
                    if let Some((_, inner)) = obj.iter().next() {
                        return Self::coerce_float(inner);
                    }
                }
                anyhow::bail!("Cannot coerce object to float: {:?}", value)
            }
            _ => anyhow::bail!("Cannot coerce {:?} to float", value),
        }
    }

    fn coerce_bool(value: &JsonValue) -> Result<Value> {
        match value {
            JsonValue::Bool(b) => Ok(Value::Bool(*b)),
            JsonValue::String(s) => {
                let s_lower = s.to_lowercase();
                if s_lower == "true" || s_lower == "yes" || s_lower == "1" {
                    Ok(Value::Bool(true))
                } else if s_lower == "false" || s_lower == "no" || s_lower == "0" {
                    Ok(Value::Bool(false))
                } else {
                    anyhow::bail!("Cannot parse '{}' as bool", s)
                }
            }
            JsonValue::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(Value::Bool(i != 0))
                } else {
                    anyhow::bail!("Cannot coerce number to bool")
                }
            }
            JsonValue::Object(obj) => {
                // LLM may wrap the value in an object like { "value": true } or { "bool": true }
                // Try common field names
                for field_name in ["value", "Value", "bool", "Bool", "result", "Result"] {
                    if let Some(inner) = obj.get(field_name) {
                        return Self::coerce_bool(inner);
                    }
                }
                // If object has only one field, use that
                if obj.len() == 1 {
                    if let Some((_, inner)) = obj.iter().next() {
                        return Self::coerce_bool(inner);
                    }
                }
                anyhow::bail!("Cannot coerce object to bool: {:?}", value)
            }
            _ => anyhow::bail!("Cannot coerce {:?} to bool", value),
        }
    }

    fn coerce_enum(&self, value: &JsonValue, enum_name: &str) -> Result<Value> {
        let e = self
            .ir
            .find_enum(enum_name)
            .ok_or_else(|| anyhow::anyhow!("Enum '{}' not found", enum_name))?;

        // Handle case where LLM wraps enum in an object like {"Sentiment": "Positive"}
        let str_value = match value {
            JsonValue::String(s) => s.clone(),
            JsonValue::Object(obj) => {
                // Try common field names first
                for field_name in ["value", "Value", "result", "Result"] {
                    if let Some(JsonValue::String(s)) = obj.get(field_name) {
                        return self.coerce_enum(&JsonValue::String(s.clone()), enum_name);
                    }
                }
                // Try the enum name itself as a field (e.g., {"Sentiment": "Positive"})
                if let Some(JsonValue::String(s)) = obj.get(enum_name) {
                    return self.coerce_enum(&JsonValue::String(s.clone()), enum_name);
                }
                // Try case-insensitive enum name match
                let enum_name_lower = enum_name.to_lowercase();
                for (key, inner) in obj.iter() {
                    if key.to_lowercase() == enum_name_lower {
                        if let JsonValue::String(s) = inner {
                            return self.coerce_enum(&JsonValue::String(s.clone()), enum_name);
                        }
                    }
                }
                // If object has only one field, use that
                if obj.len() == 1 {
                    if let Some((_, JsonValue::String(s))) = obj.iter().next() {
                        return self.coerce_enum(&JsonValue::String(s.clone()), enum_name);
                    }
                }
                anyhow::bail!("Cannot coerce object {:?} to enum '{}'", value, enum_name)
            }
            _ => value.to_string().trim_matches('"').to_string(),
        };

        // Check if the value is a valid enum variant
        if e.values.contains(&str_value) {
            Ok(Value::string(str_value))
        } else {
            // Try case-insensitive match
            let lower = str_value.to_lowercase();
            for variant in &e.values {
                if variant.to_lowercase() == lower {
                    return Ok(Value::string(variant.as_str()));
                }
            }
            anyhow::bail!(
                "'{}' is not a valid variant of enum '{}'",
                str_value,
                enum_name
            )
        }
    }

    fn coerce_class(&self, value: &JsonValue, class_name: &str) -> Result<Value> {
        let class = self
            .ir
            .find_class(class_name)
            .ok_or_else(|| anyhow::anyhow!("Class '{}' not found", class_name))?;

        let obj = value
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("Expected object for class '{}'", class_name))?;

        let mut result = HashMap::new();

        for field in &class.fields {
            if let Some(field_value) = obj.get(&field.name) {
                let coerced = self.coerce(field_value, &field.field_type)?;
                result.insert(field.name.clone(), coerced);
            } else if !field.optional {
                anyhow::bail!(
                    "Missing required field '{}' in class '{}'",
                    field.name,
                    class_name
                );
            }
        }

        Ok(Value::map(result))
    }

    fn coerce_list(&self, value: &JsonValue, inner_type: &FieldType) -> Result<Value> {
        let arr = value
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Expected array"))?;

        let coerced: Result<Vec<Value>> = arr
            .iter()
            .map(|item| self.coerce(item, inner_type))
            .collect();

        Ok(Value::list(coerced?))
    }

    fn coerce_map(
        &self,
        value: &JsonValue,
        _key_type: &FieldType,
        value_type: &FieldType,
    ) -> Result<Value> {
        let obj = value
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("Expected object for map"))?;

        let coerced: Result<HashMap<String, Value>> = obj
            .iter()
            .map(|(k, v)| {
                self.coerce(v, value_type)
                    .map(|coerced_v| (k.clone(), coerced_v))
            })
            .collect();

        Ok(Value::map(coerced?))
    }

    fn coerce_union(&self, value: &JsonValue, types: &[FieldType]) -> Result<Value> {
        // Try each type in order until one succeeds
        for t in types {
            if let Ok(coerced) = self.coerce(value, t) {
                return Ok(coerced);
            }
        }
        anyhow::bail!("Cannot coerce {:?} to any of the union types", value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ir::*;

    #[test]
    fn test_extract_json_from_markdown() {
        let ir = IR::new();
        let parser = Parser::new(&ir);

        let response = r#"
        Here's the result:
        ```json
        {"name": "John", "age": 30}
        ```
        "#;

        let json = parser.extract_json(response).unwrap();
        assert_eq!(json.trim(), r#"{"name": "John", "age": 30}"#);
    }

    #[test]
    fn test_coerce_int_from_string() {
        let value = JsonValue::String("42".to_string());
        let result = Parser::coerce_int(&value).unwrap();

        assert_eq!(result.as_int(), Some(42));
    }

    #[test]
    fn test_parse_class() {
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

        let parser = Parser::new(&ir);
        let response = r#"{"name": "John", "age": 30}"#;

        let result = parser
            .parse(response, &FieldType::Class("Person".to_string()))
            .unwrap();

        if let Value::Map(map) = result {
            assert_eq!(map.get("name").and_then(|v| v.as_string()), Some("John"));
            assert_eq!(map.get("age").and_then(|v| v.as_int()), Some(30));
        } else {
            panic!("Expected Map");
        }
    }
}
