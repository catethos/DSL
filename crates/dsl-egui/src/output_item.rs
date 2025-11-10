use dsl_ir::{Value, Span};
use dsl_interpreter::InterpreterError;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Represents different types of output that can be displayed in the REPL
#[derive(Clone, Debug)]
pub enum OutputItem {
    /// Plain text output
    Text(String),

    /// Tabular data with columns and rows
    Table {
        columns: Vec<String>,
        rows: Vec<Vec<String>>,
        /// Track which row is selected (for future interaction)
        selected: Option<usize>,
    },

    /// Hierarchical data (maps, nested structures)
    Tree {
        root: TreeNode,
        /// Track expanded/collapsed state by node path (path is dot-separated keys)
        expanded_paths: HashMap<String, bool>,
    },

    /// Error messages with rich formatting and context
    Error(ErrorDetail),

    /// Markdown formatted text with syntax highlighting
    Markdown(String),

    /// Image file path with loaded image data for rendering
    Image {
        path: String,
        data: Option<Arc<image::DynamicImage>>,
    },

    /// Chart/visualization placeholder (for future)
    Chart { chart_type: ChartType, data: Value },
}

/// Rich error details for enhanced error display
#[derive(Clone, Debug)]
pub struct ErrorDetail {
    /// Error category: "Type Error", "Runtime Error", "LLM Error", etc.
    pub error_type: String,

    /// Main error message
    pub message: String,

    /// Where error occurred (file:line:column)
    pub source_span: Option<Span>,

    /// Function context
    pub function_context: Option<String>,

    /// Error-specific details
    pub details: ErrorDetails,

    /// Suggestions for fixing the error
    pub suggestions: Vec<String>,

    /// For expandable UI sections
    pub expanded_sections: HashSet<String>,
}

/// Error-specific details based on error type
#[derive(Clone, Debug)]
pub enum ErrorDetails {
    Type {
        expected: String,
        got: String,
    },
    LLM {
        prompt: Option<String>,
        response: Option<String>,
    },
    HTTP {
        method: Option<String>,
        url: Option<String>,
    },
    SQL {
        query: Option<String>,
    },
    UnknownVariable {
        name: String,
    },
    UnknownFunction {
        name: String,
    },
    Runtime,
    Other(String),
}

#[derive(Clone, Debug)]
pub struct TreeNode {
    /// The key for this node (None for root)
    pub key: Option<String>,

    /// The value at this node
    pub value: TreeNodeValue,

    /// Children of this node
    pub children: Vec<TreeNode>,

    /// Path to this node (dot-separated keys from root, e.g., "foo.bar.baz")
    pub path: String,
}

#[derive(Clone, Debug)]
pub enum TreeNodeValue {
    /// A leaf value (string, number, bool, null, etc.)
    Leaf(String),

    /// A map node (shows "{...}" as the value)
    Map,

    /// A list node (shows "[...]" as the value)
    List,
}

#[derive(Clone, Debug)]
pub enum ChartType {
    Bar,
    Line,
    Scatter,
    Pie,
}

impl OutputItem {
    /// Create an OutputItem from a DSL Value
    pub fn from_value(value: &Value) -> Self {
        // Check for special value types first
        if let Value::Markdown(md) = value {
            return OutputItem::Markdown(md.clone());
        }

        if let Value::Image(path) = value {
            // Don't load the image yet - use lazy loading for performance
            return OutputItem::Image {
                path: path.clone(),
                data: None, // Lazy load when rendered
            };
        }

        // Check if this is a table (list of maps with consistent keys)
        if value.is_table() {
            return Self::from_table_value(value);
        }

        // Convert complex values to Tree
        match value {
            Value::Map(_) | Value::List(_) => {
                let root = TreeNode::from_value(value, None, String::new());
                OutputItem::Tree {
                    root,
                    expanded_paths: HashMap::new(),
                }
            }
            _ => {
                // Convert simple values to Text
                OutputItem::Text(format_value(value))
            }
        }
    }

    /// Convert a tabular value (list of maps) to a Table output item
    fn from_table_value(value: &Value) -> Self {
        let items = match value {
            Value::List(items) => items,
            _ => return Self::Text(format_value(value)),
        };

        if items.is_empty() {
            return Self::Text("[]".to_string());
        }

        // Get all unique column names from all maps
        let mut columns: Vec<String> = Vec::new();
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

        // Convert to rows - each row corresponds to one map in the list
        let rows: Vec<Vec<String>> = items
            .iter()
            .map(|item| {
                if let Value::Map(map) = item {
                    columns
                        .iter()
                        .map(|col| {
                            map.get(col)
                                .map(|val| match val {
                                    Value::String(s) => {
                                        // Truncate long strings for table display
                                        if s.len() > 50 {
                                            format!(
                                                "{}...",
                                                &s.chars().take(47).collect::<String>()
                                            )
                                        } else {
                                            s.clone()
                                        }
                                    }
                                    Value::Int(n) => n.to_string(),
                                    Value::Float(f) => format!("{:.2}", f),
                                    Value::Bool(b) => b.to_string(),
                                    Value::Null => "null".to_string(),
                                    // For complex values, show a simplified representation
                                    Value::List(_) => "[...]".to_string(),
                                    Value::Map(_) => "{...}".to_string(),
                                    _ => format_value(val),
                                })
                                .unwrap_or_default()
                        })
                        .collect()
                } else {
                    vec![]
                }
            })
            .collect();

        Self::Table {
            columns,
            rows,
            selected: None,
        }
    }
}

impl TreeNode {
    /// Create a TreeNode from a DSL Value
    pub fn from_value(value: &Value, key: Option<String>, path: String) -> Self {
        match value {
            Value::Map(map) => {
                let children = map
                    .iter()
                    .map(|(k, v)| {
                        let child_path = if path.is_empty() {
                            k.clone()
                        } else {
                            format!("{}.{}", path, k)
                        };
                        TreeNode::from_value(v, Some(k.clone()), child_path)
                    })
                    .collect();

                TreeNode {
                    key,
                    value: TreeNodeValue::Map,
                    children,
                    path,
                }
            }
            Value::List(items) => {
                let children = items
                    .iter()
                    .enumerate()
                    .map(|(i, v)| {
                        let child_path = if path.is_empty() {
                            format!("[{}]", i)
                        } else {
                            format!("{}[{}]", path, i)
                        };
                        TreeNode::from_value(v, Some(format!("[{}]", i)), child_path)
                    })
                    .collect();

                TreeNode {
                    key,
                    value: TreeNodeValue::List,
                    children,
                    path,
                }
            }
            _ => TreeNode {
                key,
                value: TreeNodeValue::Leaf(format_value(value)),
                children: Vec::new(),
                path,
            },
        }
    }
}

/// Format a Value as a string
fn format_value(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::String(s) => format!("\"{}\"", s),
        Value::List(items) => {
            let items_str: Vec<String> = items.iter().map(format_value).collect();
            format!("[{}]", items_str.join(", "))
        }
        Value::Map(map) => {
            let pairs: Vec<String> = map
                .iter()
                .map(|(k, v)| format!("{}: {}", k, format_value(v)))
                .collect();
            format!("{{{}}}", pairs.join(", "))
        }
        Value::Markdown(md) => md.clone(),
        Value::Image(path) => format!("<image: {}>", path),
    }
}

impl ErrorDetail {
    /// Create ErrorDetail from a string error message (fallback for simple errors)
    pub fn from_string(message: String) -> Self {
        // Try to parse common error patterns and add helpful context
        let (error_type, details, suggestions) = if message.contains("Variable") && message.contains("not found") {
            // Extract variable name if possible
            let var_name = message
                .split('\'')
                .nth(1)
                .unwrap_or("")
                .to_string();

            (
                "Unknown Variable".to_string(),
                if !var_name.is_empty() {
                    ErrorDetails::UnknownVariable { name: var_name }
                } else {
                    ErrorDetails::Runtime
                },
                vec![
                    "Define the variable with 'let' before using it".to_string(),
                    "Check for typos in the variable name".to_string(),
                    "Ensure the variable is in scope".to_string(),
                ],
            )
        } else if message.contains("Function") && message.contains("not found") {
            // Extract function name if possible
            let func_name = message
                .split('\'')
                .nth(1)
                .unwrap_or("")
                .to_string();

            (
                "Unknown Function".to_string(),
                if !func_name.is_empty() {
                    ErrorDetails::UnknownFunction { name: func_name }
                } else {
                    ErrorDetails::Runtime
                },
                vec![
                    "Check the function name spelling".to_string(),
                    "Ensure the function is defined before calling it".to_string(),
                    "Use 'function name() { ... }' to define functions".to_string(),
                ],
            )
        } else if message.contains("Parse error") || message.contains("parse") {
            (
                "Parse Error".to_string(),
                ErrorDetails::Runtime,
                vec![
                    "Check for missing or extra brackets/braces".to_string(),
                    "Ensure all statements are complete".to_string(),
                    "Verify syntax matches DSL language rules".to_string(),
                ],
            )
        } else if message.contains("Type") || message.contains("type") {
            (
                "Type Error".to_string(),
                ErrorDetails::Runtime,
                vec![
                    "Check that values match expected types".to_string(),
                    "Use type conversion functions if needed".to_string(),
                ],
            )
        } else if message.contains("Compile error") {
            (
                "Compile Error".to_string(),
                ErrorDetails::Runtime,
                vec![
                    "Check for syntax errors in your code".to_string(),
                    "Ensure all variables and functions are defined".to_string(),
                ],
            )
        } else if message.contains("Runtime error") {
            (
                "Runtime Error".to_string(),
                ErrorDetails::Runtime,
                vec![
                    "Check for division by zero".to_string(),
                    "Verify all operations are valid for the data types".to_string(),
                    "Ensure all required arguments are provided".to_string(),
                ],
            )
        } else {
            (
                "Error".to_string(),
                ErrorDetails::Other(String::new()),
                vec![],
            )
        };

        Self {
            error_type,
            message,
            source_span: None,
            function_context: None,
            details,
            suggestions,
            expanded_sections: HashSet::new(),
        }
    }

    /// Create ErrorDetail from InterpreterError
    pub fn from_interpreter_error(error: InterpreterError) -> Self {
        match error {
            InterpreterError::TypeError {
                message,
                expected,
                got,
                source_span,
            } => Self {
                error_type: "Type Error".to_string(),
                message,
                source_span,
                function_context: None,
                details: ErrorDetails::Type { expected, got },
                suggestions: vec![
                    "Check the variable type using typeof()".to_string(),
                    "Use explicit type conversion if needed".to_string(),
                ],
                expanded_sections: HashSet::new(),
            },

            InterpreterError::LLMError {
                message,
                function_name,
                source_span,
                prompt,
                response,
            } => Self {
                error_type: "LLM Error".to_string(),
                message,
                source_span,
                function_context: function_name,
                details: ErrorDetails::LLM { prompt, response },
                suggestions: vec![
                    "Check API credentials and rate limits".to_string(),
                    "Verify prompt format matches API requirements".to_string(),
                ],
                expanded_sections: HashSet::new(),
            },

            InterpreterError::HTTPError {
                message,
                function_name,
                source_span,
                method,
                url,
            } => Self {
                error_type: "HTTP Error".to_string(),
                message,
                source_span,
                function_context: function_name,
                details: ErrorDetails::HTTP { method, url },
                suggestions: vec![
                    "Check URL and network connectivity".to_string(),
                    "Verify HTTP method (GET, POST, etc.) is correct".to_string(),
                ],
                expanded_sections: HashSet::new(),
            },

            InterpreterError::SQLError {
                message,
                function_name,
                source_span,
                query,
            } => Self {
                error_type: "SQL Error".to_string(),
                message,
                source_span,
                function_context: function_name,
                details: ErrorDetails::SQL { query },
                suggestions: vec![
                    "Check SQL syntax".to_string(),
                    "Verify table and column names exist".to_string(),
                ],
                expanded_sections: HashSet::new(),
            },

            InterpreterError::UnknownVariable { name, source_span } => Self {
                error_type: "Unknown Variable".to_string(),
                message: format!("Variable '{}' is not defined", name),
                source_span,
                function_context: None,
                details: ErrorDetails::UnknownVariable { name },
                suggestions: vec![
                    "Define the variable with 'let'".to_string(),
                    "Check for typos in the variable name".to_string(),
                ],
                expanded_sections: HashSet::new(),
            },

            InterpreterError::UnknownFunction { name, source_span } => Self {
                error_type: "Unknown Function".to_string(),
                message: format!("Function '{}' is not defined", name),
                source_span,
                function_context: None,
                details: ErrorDetails::UnknownFunction { name },
                suggestions: vec![
                    "Check function name spelling".to_string(),
                    "Define function with 'function' keyword".to_string(),
                ],
                expanded_sections: HashSet::new(),
            },

            InterpreterError::RuntimeError {
                message,
                source_span,
            } => Self {
                error_type: "Runtime Error".to_string(),
                message,
                source_span,
                function_context: None,
                details: ErrorDetails::Runtime,
                suggestions: vec![],
                expanded_sections: HashSet::new(),
            },

            InterpreterError::UnknownIntrinsic { name, source_span } => Self {
                error_type: "Unknown Intrinsic".to_string(),
                message: format!("Intrinsic function '{}' is not defined", name),
                source_span,
                function_context: None,
                details: ErrorDetails::Other(format!("Unknown intrinsic: {}", name)),
                suggestions: vec![
                    "This is likely an internal compiler error".to_string(),
                ],
                expanded_sections: HashSet::new(),
            },

            InterpreterError::InvalidArguments {
                message,
                source_span,
            } => Self {
                error_type: "Invalid Arguments".to_string(),
                message,
                source_span,
                function_context: None,
                details: ErrorDetails::Runtime,
                suggestions: vec![
                    "Check the function signature".to_string(),
                    "Verify argument count and types".to_string(),
                ],
                expanded_sections: HashSet::new(),
            },
        }
    }
}
