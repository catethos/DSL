// Tree rendering utilities with expand/collapse functionality

use eframe::egui;
use std::collections::HashMap;

use crate::output_item::{TreeNode, TreeNodeValue};
use crate::theme::Theme;

/// Render a tree structure with expand/collapse functionality
pub fn render_tree(ui: &mut egui::Ui, root: &TreeNode, expanded_paths: &mut HashMap<String, bool>, theme: &Theme) {
    render_tree_node(ui, root, expanded_paths, 0, theme);
}

/// Render a single tree node with indentation and expand/collapse controls
fn render_tree_node(
    ui: &mut egui::Ui,
    node: &TreeNode,
    expanded_paths: &mut HashMap<String, bool>,
    indent_level: usize,
    theme: &Theme,
) {
    let has_children = !node.children.is_empty();

    // Wrap entire row in a frame for hover effects
    let row_response = ui.horizontal(|ui| {
        // Indentation with subtle guide lines
        if indent_level > 0 {
            for _ in 0..indent_level {
                ui.add_space(4.0);
                ui.label(egui::RichText::new("│")
                    .color(theme.ui.tree_guide)
                    .family(egui::FontFamily::Monospace));
                ui.add_space(11.0);
            }
        }

        // Expand/collapse arrow (only if has children)
        if has_children {
            let is_expanded = expanded_paths.get(&node.path).copied().unwrap_or(true);
            let arrow = if is_expanded { "▼" } else { "▶" };
            
            let arrow_response = ui.add(
                egui::Label::new(
                    egui::RichText::new(arrow)
                        .color(theme.ui.tree_arrow)
                        .size(13.0)
                        .family(egui::FontFamily::Monospace)
                )
                .sense(egui::Sense::click())
            );

            if arrow_response.clicked() {
                expanded_paths.insert(node.path.clone(), !is_expanded);
            }
            
            ui.add_space(4.0);
        } else {
            ui.add_space(18.0);
        }

        // Render the key with color based on node type
        if let Some(key) = &node.key {
            let key_color = if key.starts_with('[') && key.ends_with(']') {
                theme.data_types.array_index
            } else {
                theme.data_types.object_key
            };
            
            ui.label(egui::RichText::new(key)
                .family(egui::FontFamily::Monospace)
                .color(key_color)
                .strong());
            ui.label(egui::RichText::new(": ")
                .family(egui::FontFamily::Monospace)
                .color(theme.syntax.punctuation));
        }

        // Render the value with rich coloring
        match &node.value {
            TreeNodeValue::Leaf(value) => {
                render_colored_value(ui, value, theme);
            }
            TreeNodeValue::Map => {
                ui.label(egui::RichText::new("{")
                    .family(egui::FontFamily::Monospace)
                    .color(theme.data_types.object_punctuation)
                    .strong());
                
                if !has_children {
                    ui.label(egui::RichText::new("}")
                        .family(egui::FontFamily::Monospace)
                        .color(theme.data_types.object_punctuation)
                        .strong());
                } else {
                    let count = node.children.len();
                    ui.label(egui::RichText::new("...")
                        .family(egui::FontFamily::Monospace)
                        .color(theme.data_types.object_punctuation));
                    ui.label(egui::RichText::new("}")
                        .family(egui::FontFamily::Monospace)
                        .color(theme.data_types.object_punctuation)
                        .strong());
                    ui.label(
                        egui::RichText::new(format!("  // {} field{}", count, if count == 1 { "" } else { "s" }))
                            .family(egui::FontFamily::Monospace)
                            .color(theme.syntax.comment)
                            .italics(),
                    );
                }
            }
            TreeNodeValue::List => {
                ui.label(egui::RichText::new("[")
                    .family(egui::FontFamily::Monospace)
                    .color(theme.data_types.array_punctuation)
                    .strong());
                
                if !has_children {
                    ui.label(egui::RichText::new("]")
                        .family(egui::FontFamily::Monospace)
                        .color(theme.data_types.array_punctuation)
                        .strong());
                } else {
                    let count = node.children.len();
                    ui.label(egui::RichText::new("...")
                        .family(egui::FontFamily::Monospace)
                        .color(theme.data_types.array_punctuation));
                    ui.label(egui::RichText::new("]")
                        .family(egui::FontFamily::Monospace)
                        .color(theme.data_types.array_punctuation)
                        .strong());
                    ui.label(
                        egui::RichText::new(format!("  // {} item{}", count, if count == 1 { "" } else { "s" }))
                            .family(egui::FontFamily::Monospace)
                            .color(theme.syntax.comment)
                            .italics(),
                    );
                }
            }
        }
    }).response;
    
    // Add subtle hover effect
    if row_response.hovered() {
        let hover_rect = row_response.rect;
        ui.painter().rect_filled(
            hover_rect,
            2.0,
            egui::Color32::from_rgba_unmultiplied(139, 233, 253, 10),
        );
    }

    // Render children if expanded
    if has_children {
        let is_expanded = expanded_paths.get(&node.path).copied().unwrap_or(true);
        if is_expanded {
            for child in &node.children {
                render_tree_node(ui, child, expanded_paths, indent_level + 1, theme);
            }
        }
    }
}

/// Render a value with syntax-aware coloring
fn render_colored_value(ui: &mut egui::Ui, value: &str, theme: &Theme) {
    let (text, color) = if value == "true" {
        (value, theme.data_types.bool_true)
    } else if value == "false" {
        (value, theme.data_types.bool_false)
    } else if value == "null" {
        (value, theme.data_types.null)
    } else if value.starts_with('"') && value.ends_with('"') {
        (value, theme.data_types.string)
    } else if value.parse::<f64>().is_ok() {
        (value, theme.data_types.number)
    } else if value.starts_with("<image:") {
        ("🖼️ ", theme.data_types.special_type)
    } else if value.starts_with("<markdown:") {
        ("📄 ", theme.data_types.special_type)
    } else {
        (value, theme.syntax.default)
    };
    
    ui.label(egui::RichText::new(text)
        .family(egui::FontFamily::Monospace)
        .color(color));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output_item::TreeNode;

    #[test]
    fn test_tree_node_path() {
        let node = TreeNode {
            key: Some("test".to_string()),
            value: TreeNodeValue::Leaf("value".to_string()),
            children: Vec::new(),
            path: "root.test".to_string(),
        };
        assert_eq!(node.path, "root.test");
    }
}
