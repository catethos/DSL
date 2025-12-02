//! Schema formatter - converts IR types to human-readable schema strings
//!
//! This generates schema descriptions that help LLMs understand the expected output format.
//!
//! Example output:
//! ```text
//! Month
//! ----
//! - January
//! - February
//! - March
//!
//! Answer in JSON using this schema:
//! {
//!   name: string,
//!   age: int,
//!   birthMonth: Month,
//! }
//! ```

use crate::types::ir::{Class, Enum, FieldType, IR};
use std::collections::HashSet;

pub struct SchemaFormatter<'a> {
    ir: &'a IR,
}

impl<'a> SchemaFormatter<'a> {
    pub fn new(ir: &'a IR) -> Self {
        Self { ir }
    }

    /// Render the complete schema for a given type
    pub fn render(&mut self, output_type: &FieldType) -> String {
        let mut result = String::new();

        // First, collect all referenced enums and classes
        let (enums, _classes) = self.collect_dependencies(output_type);

        // Render enums first
        for enum_name in enums {
            if let Some(e) = self.ir.find_enum(&enum_name) {
                result.push_str(&self.render_enum(e));
                result.push_str("\n\n");
            }
        }

        // Then render the main schema
        // For primitive types, be explicit that we want just the value, not wrapped in an object
        match output_type {
            FieldType::Bool => {
                result.push_str(
                    "Answer with ONLY a JSON boolean value (true or false), nothing else. Do not wrap in an object.",
                );
            }
            FieldType::String => {
                result.push_str(
                    "Answer with ONLY a JSON string value, nothing else. Do not wrap in an object.",
                );
            }
            FieldType::Int => {
                result.push_str(
                    "Answer with ONLY a JSON integer value, nothing else. Do not wrap in an object.",
                );
            }
            FieldType::Float => {
                result.push_str(
                    "Answer with ONLY a JSON float value, nothing else. Do not wrap in an object.",
                );
            }
            _ => {
                result.push_str("Answer in JSON using this schema:\n");
                result.push_str(&self.render_type(output_type, 0));
            }
        }

        result
    }

    /// Collect all enums and classes referenced by a type
    fn collect_dependencies(&self, field_type: &FieldType) -> (Vec<String>, Vec<String>) {
        let mut enums = Vec::new();
        let mut classes = Vec::new();
        let mut visited = HashSet::new();

        self.collect_deps_recursive(field_type, &mut enums, &mut classes, &mut visited);

        (enums, classes)
    }

    fn collect_deps_recursive(
        &self,
        field_type: &FieldType,
        enums: &mut Vec<String>,
        classes: &mut Vec<String>,
        visited: &mut HashSet<String>,
    ) {
        match field_type {
            FieldType::Enum(name) => {
                if !visited.contains(name) {
                    visited.insert(name.clone());
                    enums.push(name.clone());
                }
            }
            FieldType::Class(name) => {
                if !visited.contains(name) {
                    visited.insert(name.clone());

                    // Check if this is actually an enum (compiler uses Class for all named types)
                    if self.ir.find_enum(name).is_some() {
                        enums.push(name.clone());
                    } else {
                        classes.push(name.clone());

                        // Recursively collect dependencies from class fields
                        if let Some(class) = self.ir.find_class(name) {
                            for field in &class.fields {
                                self.collect_deps_recursive(&field.field_type, enums, classes, visited);
                            }
                        }
                    }
                }
            }
            FieldType::List(inner) => {
                self.collect_deps_recursive(inner, enums, classes, visited);
            }
            FieldType::Map(k, v) => {
                self.collect_deps_recursive(k, enums, classes, visited);
                self.collect_deps_recursive(v, enums, classes, visited);
            }
            FieldType::Union(types) => {
                for t in types {
                    self.collect_deps_recursive(t, enums, classes, visited);
                }
            }
            _ => {}
        }
    }

    /// Render an enum definition
    fn render_enum(&self, e: &Enum) -> String {
        let mut result = String::new();

        // Render enum name with optional description
        if let Some(desc) = &e.description {
            result.push_str(&format!("{} ({})\n", e.name, desc));
        } else {
            result.push_str(&format!("{}\n", e.name));
        }

        result.push_str(&"-".repeat(e.name.len()));
        result.push('\n');

        for value in &e.values {
            result.push_str(&format!("- {}\n", value));
        }

        result.trim_end().to_string()
    }

    /// Render a type recursively
    fn render_type(&self, field_type: &FieldType, indent: usize) -> String {
        let indent_str = "  ".repeat(indent);

        match field_type {
            FieldType::String => "string".to_string(),
            FieldType::Int => "int".to_string(),
            FieldType::Float => "float".to_string(),
            FieldType::Bool => "bool".to_string(),
            FieldType::Enum(name) => name.clone(),
            FieldType::Class(name) => {
                if let Some(class) = self.ir.find_class(name) {
                    self.render_class(class, indent)
                } else {
                    name.clone()
                }
            }
            FieldType::List(inner) => {
                // Add a comment to encourage the LLM to populate the array
                format!(
                    "[ // array with one or more items\n{}{}\n{}]",
                    "  ".repeat(indent + 1),
                    self.render_type(inner, indent + 1),
                    indent_str
                )
            }
            FieldType::Map(k, v) => {
                format!(
                    "map<{}, {}>",
                    self.render_type(k, indent),
                    self.render_type(v, indent)
                )
            }
            FieldType::Union(types) => types
                .iter()
                .map(|t| self.render_type(t, indent))
                .collect::<Vec<_>>()
                .join(" or "),
            FieldType::Path => "path".to_string(),
        }
    }

    /// Render a class definition
    fn render_class(&self, class: &Class, indent: usize) -> String {
        let indent_str = "  ".repeat(indent);
        let field_indent = "  ".repeat(indent + 1);

        let mut result = String::from("{\n");

        // Add class description as a comment if present
        if let Some(desc) = &class.description {
            result.push_str(&format!("{}// {}\n", field_indent, desc));
        }

        for field in &class.fields {
            let field_line = if let Some(desc) = &field.description {
                format!(
                    "{}{}: {}, // {}\n",
                    field_indent,
                    field.name,
                    self.render_type(&field.field_type, indent + 1),
                    desc
                )
            } else {
                format!(
                    "{}{}: {},\n",
                    field_indent,
                    field.name,
                    self.render_type(&field.field_type, indent + 1)
                )
            };
            result.push_str(&field_line);
        }

        result.push_str(&format!("{}}}", indent_str));
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ir::*;

    #[test]
    fn test_simple_schema() {
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

        let mut formatter = SchemaFormatter::new(&ir);
        let output = formatter.render(&FieldType::Class("Person".to_string()));

        assert!(output.contains("Answer in JSON using this schema:"));
        assert!(output.contains("name: string"));
        assert!(output.contains("age: int"));
    }

    #[test]
    fn test_enum_schema() {
        let mut ir = IR::new();
        ir.enums.push(Enum {
            name: "Month".to_string(),
            description: None,
            values: vec![
                "January".to_string(),
                "February".to_string(),
                "March".to_string(),
            ],
        });
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
                    name: "birthMonth".to_string(),
                    field_type: FieldType::Enum("Month".to_string()),
                    optional: false,
                    description: None,
                },
            ],
        });

        let mut formatter = SchemaFormatter::new(&ir);
        let output = formatter.render(&FieldType::Class("Person".to_string()));

        assert!(output.contains("Month\n----"));
        assert!(output.contains("- January"));
        assert!(output.contains("birthMonth: Month"));
    }
}
