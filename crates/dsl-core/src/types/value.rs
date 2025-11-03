use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Value {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    List(Vec<Value>),
    // Using IndexMap instead of HashMap to preserve insertion order.
    // This ensures that when parallel operations (||) collect results, the order
    // displayed in tables matches the order when accessing by index (e.g., answers[0]).
    // Without this, HashMap's unordered iteration caused visual display to differ
    // from actual data structure order, leading to confusing inconsistencies.
    Map(IndexMap<String, Value>),
    Null,
    /// Markdown-formatted string for rich text rendering
    Markdown(String),
}

impl Value {
    pub fn type_name(&self) -> &str {
        match self {
            Value::String(_) => "String",
            Value::Int(_) => "Int",
            Value::Float(_) => "Float",
            Value::Bool(_) => "Bool",
            Value::List(_) => "List",
            Value::Map(_) => "Map",
            Value::Null => "Null",
            Value::Markdown(_) => "Markdown",
        }
    }

    pub fn display(&self) -> String {
        match self {
            Value::String(s) => format!("\"{}\"", s),
            Value::Int(n) => n.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::List(items) => {
                // Check if this is a table (list of maps with consistent keys)
                if self.is_table() {
                    self.display_as_table()
                } else {
                    let items_str: Vec<_> = items.iter().take(5).map(|v| v.display()).collect();
                    if items.len() > 5 {
                        format!("[{}, ... {} more]", items_str.join(", "), items.len() - 5)
                    } else {
                        format!("[{}]", items_str.join(", "))
                    }
                }
            }
            Value::Map(m) => {
                if m.is_empty() {
                    "{}".to_string()
                } else {
                    let fields: Vec<String> = m
                        .iter()
                        .take(3)
                        .map(|(k, v)| format!("{}: {}", k, v.display()))
                        .collect();
                    if m.len() > 3 {
                        format!("{{ {}, ... {} more }}", fields.join(", "), m.len() - 3)
                    } else {
                        format!("{{ {} }}", fields.join(", "))
                    }
                }
            }
            Value::Null => "null".to_string(),
            Value::Markdown(s) => {
                format!("[Markdown: {}...]", s.chars().take(50).collect::<String>())
            }
        }
    }

    /// Check if this value is a table (list of maps)
    pub fn is_table(&self) -> bool {
        match self {
            Value::List(items) => {
                !items.is_empty() && items.iter().all(|item| matches!(item, Value::Map(_)))
            }
            _ => false,
        }
    }

    /// Display as a formatted table
    fn display_as_table(&self) -> String {
        let items = match self {
            Value::List(items) => items,
            _ => return self.display(),
        };

        if items.is_empty() {
            return "[]".to_string();
        }

        // Get all unique column names
        let mut columns = Vec::new();
        for item in items {
            if let Value::Map(map) = item {
                for key in map.keys() {
                    if !columns.contains(key) {
                        columns.push(key.clone());
                    }
                }
            }
        }
        columns.sort();

        // Calculate column widths
        let mut col_widths: IndexMap<String, usize> = IndexMap::new();
        for col in &columns {
            col_widths.insert(col.clone(), col.len());
        }

        for item in items {
            if let Value::Map(map) = item {
                for col in &columns {
                    if let Some(val) = map.get(col) {
                        let val_str = match val {
                            Value::String(s) => s.clone(),
                            Value::Int(n) => n.to_string(),
                            Value::Float(f) => format!("{:.2}", f),
                            Value::Bool(b) => b.to_string(),
                            Value::Null => "null".to_string(),
                            _ => val.display(),
                        };
                        let width = col_widths.get(col).unwrap_or(&0);
                        col_widths.insert(col.clone(), (*width).max(val_str.len()));
                    }
                }
            }
        }

        // Limit column width to 30 characters
        for width in col_widths.values_mut() {
            *width = (*width).min(30);
        }

        let mut result = String::new();

        // Header row
        result.push('┌');
        for (i, col) in columns.iter().enumerate() {
            let width = col_widths.get(col).unwrap_or(&10);
            result.push_str(&"─".repeat(width + 2));
            if i < columns.len() - 1 {
                result.push('┬');
            }
        }
        result.push_str("┐\n");

        // Column names
        result.push('│');
        for col in &columns {
            let width = col_widths.get(col).unwrap_or(&10);
            result.push_str(&format!(" {:<width$} │", col, width = width));
        }
        result.push('\n');

        // Separator
        result.push('├');
        for (i, col) in columns.iter().enumerate() {
            let width = col_widths.get(col).unwrap_or(&10);
            result.push_str(&"─".repeat(width + 2));
            if i < columns.len() - 1 {
                result.push('┼');
            }
        }
        result.push_str("┤\n");

        // Data rows (show first 20 rows)
        let display_rows = items.iter().take(20);
        for item in display_rows {
            if let Value::Map(map) = item {
                result.push('│');
                for col in &columns {
                    let width = col_widths.get(col).unwrap_or(&10);
                    let val_str = if let Some(val) = map.get(col) {
                        match val {
                            Value::String(s) => {
                                if s.len() > 30 {
                                    format!("{}...", &s.chars().take(27).collect::<String>())
                                } else {
                                    s.clone()
                                }
                            }
                            Value::Int(n) => n.to_string(),
                            Value::Float(f) => format!("{:.2}", f),
                            Value::Bool(b) => b.to_string(),
                            Value::Null => "null".to_string(),
                            _ => val.display(),
                        }
                    } else {
                        "".to_string()
                    };
                    result.push_str(&format!(" {:<width$} │", val_str, width = width));
                }
                result.push('\n');
            }
        }

        // Bottom border
        result.push('└');
        for (i, col) in columns.iter().enumerate() {
            let width = col_widths.get(col).unwrap_or(&10);
            result.push_str(&"─".repeat(width + 2));
            if i < columns.len() - 1 {
                result.push('┴');
            }
        }
        result.push('┘');

        if items.len() > 20 {
            result.push_str(&format!("\n({} rows shown, {} total)", 20, items.len()));
        } else {
            result.push_str(&format!("\n({} rows)", items.len()));
        }

        result
    }

    /// Convert value to a string suitable for LLM prompts (full data, no truncation)
    pub fn to_prompt_string(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            Value::Int(n) => n.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::List(_items) => {
                // For LLM prompts, serialize as JSON
                serde_json::to_string_pretty(self).unwrap_or_else(|_| "[]".to_string())
            }
            Value::Map(_m) => {
                // For LLM prompts, serialize as JSON
                serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
            }
            Value::Null => "null".to_string(),
            Value::Markdown(s) => s.clone(),
        }
    }

    /// Convert from serde_json::Value to our Value type
    pub fn from_json(json: serde_json::Value) -> Self {
        match json {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(b) => Value::Bool(b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Value::Int(i)
                } else if let Some(f) = n.as_f64() {
                    Value::Float(f)
                } else {
                    Value::Null
                }
            }
            serde_json::Value::String(s) => Value::String(s),
            serde_json::Value::Array(arr) => {
                Value::List(arr.into_iter().map(Value::from_json).collect())
            }
            serde_json::Value::Object(obj) => {
                let mut map = IndexMap::new();
                for (k, v) in obj {
                    map.insert(k, Value::from_json(v));
                }
                Value::Map(map)
            }
        }
    }
}
