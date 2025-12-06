//! Cell output formatting for notebook display
//!
//! This module provides types and functions for converting runtime Values
//! to different output formats suitable for notebook display, including:
//! - Text (simple string representation)
//! - Tables (for SQL results and lists of maps)
//! - Structured data (for typed structs)
//! - Errors (for runtime errors with source location)

use crate::syntax::ast::Span;
use crate::types::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Output from executing a notebook cell
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CellOutput {
    /// Plain text output
    Text { content: String },
    /// Raw JSON output (for LLM responses that return JSON)
    Json { content: serde_json::Value },
    /// Tabular data (for SQL results)
    Table {
        columns: Vec<ColumnInfo>,
        rows: Vec<Vec<CellValue>>,
    },
    /// Structured data (for typed objects)
    Struct {
        type_name: String,
        fields: HashMap<String, CellValue>,
    },
    /// Error output
    Error {
        message: String,
        span: Option<SerializableSpan>,
    },
    /// No output (for statements)
    None,
}

/// Information about a table column
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_name: Option<String>,
}

/// A serializable span for error reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableSpan {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl From<Span> for SerializableSpan {
    fn from(span: Span) -> Self {
        Self {
            start: span.start,
            end: span.end,
            line: span.line,
            column: span.column,
        }
    }
}

/// A cell value that can be serialized for display
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum CellValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
    List(Vec<CellValue>),
    Map(HashMap<String, CellValue>),
}

impl From<&Value> for CellValue {
    fn from(value: &Value) -> Self {
        match value {
            Value::String(s) => CellValue::String(s.to_string()),
            Value::Int(i) => CellValue::Int(*i),
            Value::Float(f) => CellValue::Float(*f),
            Value::Bool(b) => CellValue::Bool(*b),
            Value::Null => CellValue::Null,
            Value::Path(p) => CellValue::String(p.display().to_string()),
            Value::List(items) => CellValue::List(items.iter().map(CellValue::from).collect()),
            Value::Map(map) => {
                CellValue::Map(map.iter().map(|(k, v)| (k.clone(), CellValue::from(v))).collect())
            }
        }
    }
}

impl std::fmt::Display for CellValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CellValue::String(s) => write!(f, "{}", s),
            CellValue::Int(i) => write!(f, "{}", i),
            CellValue::Float(fl) => write!(f, "{}", fl),
            CellValue::Bool(b) => write!(f, "{}", b),
            CellValue::Null => write!(f, "null"),
            CellValue::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            CellValue::Map(map) => {
                write!(f, "{{")?;
                for (i, (k, v)) in map.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            }
        }
    }
}

/// Convert a runtime Value directly to a serde_json::Value
/// This is useful for clean JSON output without CellOutput wrapper
pub fn value_to_json(value: &Value) -> serde_json::Value {
    match value {
        Value::String(s) => {
            // If the string looks like JSON, parse and return it directly
            let trimmed = s.trim();
            if (trimmed.starts_with('{') && trimmed.ends_with('}'))
                || (trimmed.starts_with('[') && trimmed.ends_with(']'))
            {
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(trimmed) {
                    return json_value;
                }
            }
            // Otherwise return as a string
            serde_json::Value::String(s.to_string())
        }
        Value::Int(i) => serde_json::Value::Number((*i).into()),
        Value::Float(f) => {
            serde_json::Number::from_f64(*f)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null)
        }
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::Null => serde_json::Value::Null,
        Value::List(items) => {
            serde_json::Value::Array(items.iter().map(value_to_json).collect())
        }
        Value::Map(map) => {
            let obj: serde_json::Map<String, serde_json::Value> = map
                .iter()
                .filter(|(k, _)| *k != "__type") // Exclude internal type marker
                .map(|(k, v)| (k.clone(), value_to_json(v)))
                .collect();
            serde_json::Value::Object(obj)
        }
        Value::Path(p) => serde_json::Value::String(p.display().to_string()),
    }
}

/// Convert a runtime Value to a CellOutput
pub fn value_to_output(value: &Value) -> CellOutput {
    // Check if it's a table-like structure (list of maps)
    if let Value::List(rows) = value {
        if !rows.is_empty() {
            // Check if all items are maps
            let all_maps = rows.iter().all(|r| matches!(r, Value::Map(_)));
            if all_maps {
                return value_to_table(value);
            }
        }
    }

    // Check if it's a typed struct (has __type field)
    if let Value::Map(map) = value {
        if let Some(Value::String(type_name)) = map.get("__type") {
            let mut fields = HashMap::new();
            for (k, v) in map.iter() {
                if k != "__type" {
                    fields.insert(k.clone(), CellValue::from(v));
                }
            }
            return CellOutput::Struct {
                type_name: type_name.to_string(),
                fields,
            };
        }
        // Map without __type - convert to JSON for cleaner output
        return CellOutput::Json {
            content: value_to_json(value),
        };
    }

    // Check if it's a string containing JSON - unwrap it for cleaner output
    if let Value::String(s) = value {
        let trimmed = s.trim();
        // Try to parse as JSON if it looks like JSON (starts with { or [)
        if (trimmed.starts_with('{') && trimmed.ends_with('}'))
            || (trimmed.starts_with('[') && trimmed.ends_with(']'))
        {
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(trimmed) {
                return CellOutput::Json { content: json_value };
            }
        }
    }

    // Default to text
    CellOutput::Text {
        content: format!("{}", value),
    }
}

/// Convert a list of maps to a table output
fn value_to_table(value: &Value) -> CellOutput {
    let rows = match value {
        Value::List(rows) => rows,
        _ => return CellOutput::Text { content: format!("{}", value) },
    };

    if rows.is_empty() {
        return CellOutput::Table {
            columns: vec![],
            rows: vec![],
        };
    }

    // Collect all unique column names from all rows
    let mut column_names: Vec<String> = Vec::new();
    for row in rows.iter() {
        if let Value::Map(map) = row {
            for key in map.keys() {
                if !column_names.contains(key) && key != "__type" {
                    column_names.push(key.clone());
                }
            }
        }
    }

    // Sort columns alphabetically for consistent display
    column_names.sort();

    // Build column info
    let columns: Vec<ColumnInfo> = column_names
        .iter()
        .map(|name| ColumnInfo {
            name: name.clone(),
            type_name: None, // Could infer from first non-null value
        })
        .collect();

    // Build row data
    let mut table_rows = Vec::with_capacity(rows.len());
    for row in rows.iter() {
        if let Value::Map(map) = row {
            let mut row_values = Vec::with_capacity(column_names.len());
            for col_name in &column_names {
                let cell = map
                    .get(col_name)
                    .map(CellValue::from)
                    .unwrap_or(CellValue::Null);
                row_values.push(cell);
            }
            table_rows.push(row_values);
        }
    }

    CellOutput::Table {
        columns,
        rows: table_rows,
    }
}

/// Format a table as plain text (ASCII table)
pub fn format_table_as_text(output: &CellOutput) -> String {
    match output {
        CellOutput::Table { columns, rows } => {
            if columns.is_empty() {
                return "(empty table)".to_string();
            }

            // Calculate column widths
            let mut widths: Vec<usize> = columns.iter().map(|c| c.name.len()).collect();

            for row in rows {
                for (i, cell) in row.iter().enumerate() {
                    if i < widths.len() {
                        let cell_str = format!("{}", cell);
                        widths[i] = widths[i].max(cell_str.len());
                    }
                }
            }

            // Build output
            let mut output = String::new();

            // Header separator
            let separator: String = widths
                .iter()
                .map(|w| "-".repeat(*w + 2))
                .collect::<Vec<_>>()
                .join("+");
            let separator = format!("+{}+\n", separator);

            output.push_str(&separator);

            // Header row
            let header: String = columns
                .iter()
                .enumerate()
                .map(|(i, col)| format!(" {:width$} ", col.name, width = widths[i]))
                .collect::<Vec<_>>()
                .join("|");
            output.push_str(&format!("|{}|\n", header));

            output.push_str(&separator);

            // Data rows
            for row in rows {
                let row_str: String = row
                    .iter()
                    .enumerate()
                    .map(|(i, cell)| {
                        let cell_str = format!("{}", cell);
                        format!(" {:width$} ", cell_str, width = widths.get(i).copied().unwrap_or(0))
                    })
                    .collect::<Vec<_>>()
                    .join("|");
                output.push_str(&format!("|{}|\n", row_str));
            }

            output.push_str(&separator);

            // Row count
            output.push_str(&format!("({} row{})\n", rows.len(), if rows.len() == 1 { "" } else { "s" }));

            output
        }
        _ => format!("{:?}", output),
    }
}

/// Format a table as HTML for rich notebook display
pub fn format_table_as_html(output: &CellOutput) -> String {
    match output {
        CellOutput::Table { columns, rows } => {
            if columns.is_empty() {
                return "<p><em>(empty table)</em></p>".to_string();
            }

            let mut html = String::new();
            html.push_str("<table class=\"lattice-table\">\n");

            // Header
            html.push_str("  <thead>\n    <tr>\n");
            for col in columns {
                html.push_str(&format!("      <th>{}</th>\n", escape_html(&col.name)));
            }
            html.push_str("    </tr>\n  </thead>\n");

            // Body
            html.push_str("  <tbody>\n");
            for row in rows {
                html.push_str("    <tr>\n");
                for cell in row {
                    let class = match cell {
                        CellValue::Null => " class=\"null\"",
                        CellValue::Int(_) | CellValue::Float(_) => " class=\"number\"",
                        CellValue::Bool(_) => " class=\"bool\"",
                        _ => "",
                    };
                    html.push_str(&format!(
                        "      <td{}>{}</td>\n",
                        class,
                        escape_html(&format!("{}", cell))
                    ));
                }
                html.push_str("    </tr>\n");
            }
            html.push_str("  </tbody>\n");

            html.push_str("</table>\n");
            html.push_str(&format!(
                "<p class=\"row-count\">({} row{})</p>",
                rows.len(),
                if rows.len() == 1 { "" } else { "s" }
            ));

            html
        }
        _ => format!("<pre>{:?}</pre>", output),
    }
}

/// Escape HTML special characters
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// CSS styles for the table (can be included in notebook app)
pub const TABLE_CSS: &str = r#"
.lattice-table {
    border-collapse: collapse;
    margin: 8px 0;
    font-family: monospace;
    font-size: 13px;
}

.lattice-table th,
.lattice-table td {
    border: 1px solid #ddd;
    padding: 6px 12px;
    text-align: left;
}

.lattice-table th {
    background-color: #f5f5f5;
    font-weight: bold;
}

.lattice-table tr:nth-child(even) {
    background-color: #fafafa;
}

.lattice-table tr:hover {
    background-color: #f0f0f0;
}

.lattice-table td.null {
    color: #999;
    font-style: italic;
}

.lattice-table td.number {
    text-align: right;
}

.lattice-table td.bool {
    color: #0066cc;
}

.row-count {
    margin: 4px 0;
    color: #666;
    font-size: 12px;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_to_table() {
        let value = Value::list(vec![
            Value::map({
                let mut m = HashMap::new();
                m.insert("id".to_string(), Value::Int(1));
                m.insert("name".to_string(), Value::string("Alice"));
                m
            }),
            Value::map({
                let mut m = HashMap::new();
                m.insert("id".to_string(), Value::Int(2));
                m.insert("name".to_string(), Value::string("Bob"));
                m
            }),
        ]);

        let output = value_to_output(&value);

        match output {
            CellOutput::Table { columns, rows } => {
                assert_eq!(columns.len(), 2);
                assert_eq!(rows.len(), 2);
            }
            _ => panic!("Expected Table output"),
        }
    }

    #[test]
    fn test_format_table_as_text() {
        let output = CellOutput::Table {
            columns: vec![
                ColumnInfo { name: "id".to_string(), type_name: None },
                ColumnInfo { name: "name".to_string(), type_name: None },
            ],
            rows: vec![
                vec![CellValue::Int(1), CellValue::String("Alice".to_string())],
                vec![CellValue::Int(2), CellValue::String("Bob".to_string())],
            ],
        };

        let text = format_table_as_text(&output);
        assert!(text.contains("id"));
        assert!(text.contains("name"));
        assert!(text.contains("Alice"));
        assert!(text.contains("Bob"));
        assert!(text.contains("(2 rows)"));
    }

    #[test]
    fn test_format_table_as_html() {
        let output = CellOutput::Table {
            columns: vec![
                ColumnInfo { name: "id".to_string(), type_name: None },
                ColumnInfo { name: "name".to_string(), type_name: None },
            ],
            rows: vec![
                vec![CellValue::Int(1), CellValue::String("Alice".to_string())],
            ],
        };

        let html = format_table_as_html(&output);
        assert!(html.contains("<table"));
        assert!(html.contains("</table>"));
        assert!(html.contains("<th>id</th>"));
        assert!(html.contains("<th>name</th>"));
        assert!(html.contains("Alice"));
    }

    #[test]
    fn test_struct_output() {
        let value = Value::map({
            let mut m = HashMap::new();
            m.insert("__type".to_string(), Value::string("Person"));
            m.insert("name".to_string(), Value::string("Alice"));
            m.insert("age".to_string(), Value::Int(30));
            m
        });

        let output = value_to_output(&value);

        match output {
            CellOutput::Struct { type_name, fields } => {
                assert_eq!(type_name, "Person");
                assert_eq!(fields.len(), 2); // name and age, not __type
                assert!(!fields.contains_key("__type"));
            }
            _ => panic!("Expected Struct output"),
        }
    }

    #[test]
    fn test_null_in_table() {
        let output = CellOutput::Table {
            columns: vec![ColumnInfo { name: "value".to_string(), type_name: None }],
            rows: vec![vec![CellValue::Null]],
        };

        let html = format_table_as_html(&output);
        assert!(html.contains("class=\"null\""));
        assert!(html.contains("null"));
    }
}
