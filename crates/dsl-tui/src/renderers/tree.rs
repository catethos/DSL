use crate::output_item::{TreeNode, TreeValue};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use std::collections::HashMap;

/// Render a tree structure as lines for display
pub fn tree_to_lines(
    root: &TreeNode,
    expanded_paths: &HashMap<String, bool>,
    _width: usize,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    // Add a hint that the tree is interactive (only if it has expandable nodes)
    if matches!(root.value, TreeValue::Map { .. } | TreeValue::List { .. }) {
        lines.push(Line::from(vec![Span::styled(
            "💡 Click on ▶/▼ to expand/collapse".to_string(),
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )]));
    }

    // Render the tree recursively
    render_tree_node(root, expanded_paths, "", true, &mut lines);

    lines
}

fn render_tree_node(
    node: &TreeNode,
    expanded_paths: &HashMap<String, bool>,
    prefix: &str,
    is_last: bool,
    lines: &mut Vec<Line<'static>>,
) {
    let is_expanded = expanded_paths.get(&node.path).copied().unwrap_or(false);

    // Build the line for this node
    let mut spans = Vec::new();

    // Add tree structure prefix (convert to owned string)
    if !prefix.is_empty() {
        spans.push(Span::styled(
            prefix.to_string(),
            Style::default().fg(Color::DarkGray),
        ));
    }

    // Add branch character (convert to owned string)
    let branch = if is_last { "└─ " } else { "├─ " };
    spans.push(Span::styled(
        branch.to_string(),
        Style::default().fg(Color::DarkGray),
    ));

    // Add expand/collapse indicator for containers
    match &node.value {
        TreeValue::Map { .. } | TreeValue::List { .. } => {
            let indicator = if is_expanded { "▼ " } else { "▶ " };
            spans.push(Span::styled(
                indicator.to_string(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD), // Make it bold and more prominent
            ));
        }
        TreeValue::Primitive(_) => {
            spans.push(Span::raw("  ".to_string()));
        }
    }

    // Add key (if present)
    if let Some(key) = &node.key {
        spans.push(Span::styled(
            key.clone(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(": ".to_string()));
    }

    // Add value
    match &node.value {
        TreeValue::Primitive(val) => {
            spans.push(Span::styled(val.clone(), Style::default().fg(Color::Green)));
        }
        TreeValue::Map { size } => {
            spans.push(Span::styled(
                format!("{{ {} fields }}", size),
                Style::default().fg(Color::Magenta),
            ));
        }
        TreeValue::List { size } => {
            spans.push(Span::styled(
                format!("[ {} items ]", size),
                Style::default().fg(Color::Magenta),
            ));
        }
    }

    lines.push(Line::from(spans));

    // Render children if expanded
    if is_expanded && !node.children.is_empty() {
        let child_prefix = if prefix.is_empty() {
            String::new()
        } else {
            format!("{}{}", prefix, if is_last { "   " } else { "│  " })
        };

        for (i, child) in node.children.iter().enumerate() {
            let is_last_child = i == node.children.len() - 1;
            render_tree_node(child, expanded_paths, &child_prefix, is_last_child, lines);
        }
    }
}

/// Find the tree node at a given line index for interaction
pub fn find_node_at_line<'a>(
    root: &'a TreeNode,
    expanded_paths: &HashMap<String, bool>,
    target_line: usize,
) -> Option<&'a TreeNode> {
    let mut current_line = 0;
    find_node_at_line_recursive(root, expanded_paths, target_line, &mut current_line)
}

fn find_node_at_line_recursive<'a>(
    node: &'a TreeNode,
    expanded_paths: &HashMap<String, bool>,
    target_line: usize,
    current_line: &mut usize,
) -> Option<&'a TreeNode> {
    if *current_line == target_line {
        return Some(node);
    }

    *current_line += 1;

    let is_expanded = expanded_paths.get(&node.path).copied().unwrap_or(false);
    if is_expanded {
        for child in &node.children {
            if let Some(found) =
                find_node_at_line_recursive(child, expanded_paths, target_line, current_line)
            {
                return Some(found);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output_item::TreeNode;

    #[test]
    fn test_tree_rendering() {
        let root = TreeNode {
            key: None,
            value: TreeValue::Map { size: 2 },
            children: vec![
                TreeNode {
                    key: Some("name".to_string()),
                    value: TreeValue::Primitive("\"Alice\"".to_string()),
                    children: vec![],
                    path: "name".to_string(),
                },
                TreeNode {
                    key: Some("age".to_string()),
                    value: TreeValue::Primitive("30".to_string()),
                    children: vec![],
                    path: "age".to_string(),
                },
            ],
            path: "root".to_string(),
        };

        let mut expanded = HashMap::new();
        expanded.insert("root".to_string(), true);

        let lines = tree_to_lines(&root, &expanded, 80);
        assert_eq!(lines.len(), 4); // hint line + root + 2 children
    }
}
