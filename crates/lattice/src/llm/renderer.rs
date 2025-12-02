//! Jinja2 template renderer
//!
//! This module handles rendering prompt templates using Jinja2.
//! It injects the schema into the template context automatically.

use anyhow::Result;
use minijinja::Environment;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

use crate::llm::schema::SchemaFormatter;
use crate::types::ir::{FieldType, Value, IR};

pub struct PromptRenderer<'a> {
    ir: &'a IR,
}

impl<'a> PromptRenderer<'a> {
    pub fn new(ir: &'a IR) -> Self {
        Self { ir }
    }

    /// Render a prompt template with the given parameters
    ///
    /// # Arguments
    /// * `template` - The Jinja2 template string (supports both `{{ var }}` and `${var}` syntax)
    /// * `params` - Input parameters as Value::Map
    /// * `output_type` - The expected output type for schema generation
    ///
    /// # Returns
    /// The rendered prompt string with schema appended
    pub fn render(
        &self,
        template: &str,
        params: &HashMap<String, Value>,
        output_type: &FieldType,
    ) -> Result<String> {
        // Generate the schema
        let mut formatter = SchemaFormatter::new(self.ir);
        let schema = formatter.render(output_type);

        // Convert ${var} syntax to {{ var }} for minijinja compatibility
        let template = convert_dollar_syntax(template);

        // Convert Value params to JSON for minijinja
        let json_params = params_to_json(params);

        // Set up minijinja environment
        let mut env = Environment::new();

        // Add the template
        env.add_template("prompt", &template)?;

        // Get the template
        let tmpl = env.get_template("prompt")?;

        // Render with parameters
        // Build context with all params plus the schema
        let mut ctx = json_params;
        ctx.insert(
            "output_schema".to_string(),
            JsonValue::String(schema.clone()),
        );

        let rendered = tmpl.render(&ctx)?;

        // If the template doesn't already include the schema, append it
        // Check for both primitive type schemas ("Answer with ONLY") and complex type schemas ("Answer in JSON")
        if !rendered.contains("Answer in JSON using this schema:")
            && !rendered.contains("Answer with ONLY a JSON")
        {
            Ok(format!("{}\n\n{}", rendered, schema))
        } else {
            Ok(rendered)
        }
    }
}

/// Convert ${var} syntax to {{ var }} for minijinja compatibility
/// This allows Lattice prompts to use the more familiar ${var} interpolation
fn convert_dollar_syntax(template: &str) -> String {
    let mut result = String::with_capacity(template.len());
    let mut chars = template.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' && chars.peek() == Some(&'{') {
            // Skip the '{'
            chars.next();
            // Collect the variable name
            let mut var_name = String::new();
            while let Some(&ch) = chars.peek() {
                if ch == '}' {
                    chars.next(); // consume the '}'
                    break;
                }
                var_name.push(chars.next().unwrap());
            }
            // Convert to jinja syntax
            result.push_str("{{ ");
            result.push_str(&var_name);
            result.push_str(" }}");
        } else {
            result.push(c);
        }
    }

    result
}

/// Convert Value HashMap to JSON Value for minijinja
fn params_to_json(params: &HashMap<String, Value>) -> HashMap<String, JsonValue> {
    params
        .iter()
        .map(|(k, v)| (k.clone(), value_to_json(v)))
        .collect()
}

/// Convert a single Value to JSON Value
fn value_to_json(value: &Value) -> JsonValue {
    match value {
        Value::String(s) => JsonValue::String(s.clone()),
        Value::Int(i) => JsonValue::Number((*i).into()),
        Value::Float(f) => JsonValue::Number(serde_json::Number::from_f64(*f).unwrap_or(0.into())),
        Value::Bool(b) => JsonValue::Bool(*b),
        Value::Path(p) => JsonValue::String(p.display().to_string()),
        Value::List(items) => JsonValue::Array(items.iter().map(value_to_json).collect()),
        Value::Map(map) => {
            JsonValue::Object(map.iter().map(|(k, v)| (k.clone(), value_to_json(v))).collect())
        }
        Value::Null => JsonValue::Null,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ir::*;

    #[test]
    fn test_simple_render() {
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

        let renderer = PromptRenderer::new(&ir);

        let template = "Extract person info from: {{ text }}";
        let mut params = HashMap::new();
        params.insert(
            "text".to_string(),
            Value::String("John is 30 years old".to_string()),
        );

        let result = renderer
            .render(template, &params, &FieldType::Class("Person".to_string()))
            .unwrap();

        assert!(result.contains("Extract person info from: John is 30 years old"));
        assert!(result.contains("Answer in JSON using this schema:"));
        assert!(result.contains("name: string"));
        assert!(result.contains("age: int"));
    }

    #[test]
    fn test_render_with_explicit_schema() {
        let ir = IR::new();
        let renderer = PromptRenderer::new(&ir);

        let template = "Extract text\n\n{{ output_schema }}";
        let params = HashMap::new();

        let result = renderer
            .render(template, &params, &FieldType::String)
            .unwrap();

        // Should only contain schema once
        assert_eq!(
            result
                .matches("Answer with ONLY a JSON string value")
                .count(),
            1
        );
    }

    #[test]
    fn test_render_custom_class_type() {
        // Test that custom class types get proper schema in prompt
        let mut ir = IR::new();
        ir.classes.push(Class {
            name: "Explanation".to_string(),
            description: None,
            fields: vec![
                Field {
                    name: "entity".to_string(),
                    field_type: FieldType::String,
                    optional: false,
                    description: None,
                },
                Field {
                    name: "explanation".to_string(),
                    field_type: FieldType::String,
                    optional: false,
                    description: None,
                },
            ],
        });

        let renderer = PromptRenderer::new(&ir);

        let template = "explain this like i am 5: {{ x }}";
        let mut params = HashMap::new();
        params.insert("x".to_string(), Value::String("love".to_string()));

        let result = renderer
            .render(template, &params, &FieldType::Class("Explanation".to_string()))
            .unwrap();

        assert!(result.contains("explain this like i am 5: love"));
        assert!(result.contains("Answer in JSON using this schema:"));
        assert!(result.contains("entity: string"));
        assert!(result.contains("explanation: string"));
    }
}
