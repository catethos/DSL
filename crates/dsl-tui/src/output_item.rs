use dsl_core::Value;
use std::collections::HashMap;

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

    /// Error messages with special formatting
    Error(String),

    /// Chart/visualization placeholder (for future)
    Chart {
        chart_type: ChartType,
        data: Value,
    },
}

#[derive(Clone, Debug)]
pub struct TreeNode {
    /// The key for this node (None for root)
    pub key: Option<String>,
    /// The type and display value
    pub value: TreeValue,
    /// Child nodes
    pub children: Vec<TreeNode>,
    /// Full path to this node (for tracking expansion state)
    pub path: String,
}

#[derive(Clone, Debug)]
pub enum TreeValue {
    /// A primitive value that can be displayed directly
    Primitive(String),
    /// A map container (shows "{...}")
    Map { size: usize },
    /// A list container (shows "[...]")
    List { size: usize },
}

#[derive(Clone, Debug)]
pub enum ChartType {
    BarChart,
    LineChart,
}

impl OutputItem {
    /// Create from a Value, automatically choosing the best display type
    pub fn from_value(value: &Value) -> Self {
        if value.is_table() {
            Self::from_table_value(value)
        } else if matches!(value, Value::Map(_)) {
            Self::from_tree_value(value, "root", true)
        } else {
            Self::Text(value.display())
        }
    }

    /// Create a text output item
    pub fn text(s: impl Into<String>) -> Self {
        Self::Text(s.into())
    }

    /// Create an error output item
    pub fn error(s: impl Into<String>) -> Self {
        Self::Error(s.into())
    }

    /// Convert a tabular value to a Table output item
    fn from_table_value(value: &Value) -> Self {
        let items = match value {
            Value::List(items) => items,
            _ => return Self::Text(value.display()),
        };

        if items.is_empty() {
            return Self::Text("[]".to_string());
        }

        // Get all unique column names
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

        // Convert to rows
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

    /// Convert any value to a tree structure
    fn from_tree_value(value: &Value, path: &str, expanded_by_default: bool) -> Self {
        let mut expanded_paths = HashMap::new();

        fn build_tree_node(
            value: &Value,
            key: Option<String>,
            path: String,
            expanded_paths: &mut HashMap<String, bool>,
            expanded_by_default: bool,
        ) -> TreeNode {
            // Mark this node as expanded by default
            if expanded_by_default {
                expanded_paths.insert(path.clone(), true);
            }

            match value {
                Value::Map(map) => {
                    let children = map
                        .iter()
                        .map(|(k, v)| {
                            let child_path = if path.is_empty() || path == "root" {
                                k.clone()
                            } else {
                                format!("{}.{}", path, k)
                            };
                            build_tree_node(
                                v,
                                Some(k.clone()),
                                child_path,
                                expanded_paths,
                                expanded_by_default,
                            )
                        })
                        .collect();

                    TreeNode {
                        key,
                        value: TreeValue::Map { size: map.len() },
                        children,
                        path,
                    }
                }
                Value::List(items) => {
                    let children = items
                        .iter()
                        .enumerate()
                        .map(|(i, v)| {
                            let child_path = format!("{}[{}]", path, i);
                            build_tree_node(
                                v,
                                Some(format!("[{}]", i)),
                                child_path,
                                expanded_paths,
                                expanded_by_default,
                            )
                        })
                        .collect();

                    TreeNode {
                        key,
                        value: TreeValue::List { size: items.len() },
                        children,
                        path,
                    }
                }
                _ => TreeNode {
                    key,
                    value: TreeValue::Primitive(match value {
                        Value::String(s) => format!("\"{}\"", s),
                        Value::Int(n) => n.to_string(),
                        Value::Float(f) => format!("{:.2}", f),
                        Value::Bool(b) => b.to_string(),
                        Value::Null => "null".to_string(),
                        _ => value.display(),
                    }),
                    children: vec![],
                    path,
                },
            }
        }

        let root = build_tree_node(value, None, path.to_string(), &mut expanded_paths, expanded_by_default);

        Self::Tree {
            root,
            expanded_paths,
        }
    }

    /// Toggle expansion state for a tree node at the given path
    pub fn toggle_expansion(&mut self, path: &str) -> bool {
        if let OutputItem::Tree { expanded_paths, .. } = self {
            let current = expanded_paths.get(path).copied().unwrap_or(false);
            expanded_paths.insert(path.to_string(), !current);
            !current
        } else {
            false
        }
    }

    /// Check if a tree node is expanded
    pub fn is_expanded(&self, path: &str) -> bool {
        if let OutputItem::Tree { expanded_paths, .. } = self {
            expanded_paths.get(path).copied().unwrap_or(false)
        } else {
            false
        }
    }
}
