/// Jinja2 template renderer
///
/// This module handles rendering BAML prompt templates using Jinja2.
/// It injects the schema into the template context automatically.
use anyhow::Result;
use minijinja::Environment;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

use crate::ir::IR;
use crate::ir::{BamlValue, FieldType};
use crate::schema::SchemaFormatter;

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
    /// * `template` - The Jinja2 template string
    /// * `params` - Input parameters as BamlValue::Map
    /// * `output_type` - The expected output type for schema generation
    ///
    /// # Returns
    /// The rendered prompt string with schema appended
    pub fn render(
        &self,
        template: &str,
        params: &HashMap<String, BamlValue>,
        output_type: &FieldType,
    ) -> Result<String> {
        // Generate the schema
        let mut formatter = SchemaFormatter::new(self.ir);
        let schema = formatter.render(output_type);

        // Convert BamlValue params to JSON for minijinja
        let json_params = params_to_json(params);

        // Set up minijinja environment
        let mut env = Environment::new();

        // Add the template
        env.add_template("prompt", template)?;

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
            && !rendered.contains("Answer with ONLY a JSON") {
            Ok(format!("{}\n\n{}", rendered, schema))
        } else {
            Ok(rendered)
        }
    }
}

/// Convert BamlValue HashMap to JSON Value for minijinja
fn params_to_json(params: &HashMap<String, BamlValue>) -> HashMap<String, JsonValue> {
    params
        .iter()
        .map(|(k, v)| (k.clone(), baml_value_to_json(v)))
        .collect()
}

/// Convert a single BamlValue to JSON Value
fn baml_value_to_json(value: &BamlValue) -> JsonValue {
    match value {
        BamlValue::String(s) => JsonValue::String(s.clone()),
        BamlValue::Int(i) => JsonValue::Number((*i).into()),
        BamlValue::Float(f) => {
            JsonValue::Number(serde_json::Number::from_f64(*f).unwrap_or(0.into()))
        }
        BamlValue::Bool(b) => JsonValue::Bool(*b),
        BamlValue::List(items) => JsonValue::Array(items.iter().map(baml_value_to_json).collect()),
        BamlValue::Map(map) => JsonValue::Object(
            map.iter()
                .map(|(k, v)| (k.clone(), baml_value_to_json(v)))
                .collect(),
        ),
        BamlValue::Null => JsonValue::Null,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::*;

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
            BamlValue::String("John is 30 years old".to_string()),
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
}
