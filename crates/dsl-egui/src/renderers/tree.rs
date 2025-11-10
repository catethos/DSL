// Tree rendering utilities with expand/collapse functionality

use eframe::egui;
use std::collections::HashMap;

use crate::output_item::{TreeNode, TreeNodeValue};

/// Render a tree structure with expand/collapse functionality
pub fn render_tree(ui: &mut egui::Ui, root: &TreeNode, expanded_paths: &mut HashMap<String, bool>) {
    render_tree_node(ui, root, expanded_paths, 0);
}

/// Render a single tree node with indentation and expand/collapse controls
fn render_tree_node(
    ui: &mut egui::Ui,
    node: &TreeNode,
    expanded_paths: &mut HashMap<String, bool>,
    indent_level: usize,
) {
    let has_children = !node.children.is_empty();

    ui.horizontal(|ui| {
        // Indentation
        ui.add_space(indent_level as f32 * 16.0);

        // Expand/collapse arrow (only if has children)
        if has_children {
            // Default to expanded (true) if not explicitly set
            let is_expanded = expanded_paths.get(&node.path).copied().unwrap_or(true);

            let arrow = if is_expanded { "▼" } else { "▶" };
            let response = ui.selectable_label(false, arrow);

            if response.clicked() {
                // Toggle expanded state
                expanded_paths.insert(node.path.clone(), !is_expanded);
            }
        } else {
            // No arrow for leaf nodes, but maintain spacing
            ui.add_space(16.0);
        }

        // Render the key (if present)
        if let Some(key) = &node.key {
            ui.label(egui::RichText::new(key)
                .family(egui::FontFamily::Monospace)
                .color(egui::Color32::from_rgb(100, 150, 200)));
            ui.label(egui::RichText::new(":")
                .family(egui::FontFamily::Monospace));
        }

        // Render the value
        match &node.value {
            TreeNodeValue::Leaf(value) => {
                ui.monospace(value);
            }
            TreeNodeValue::Map => {
                ui.label(egui::RichText::new("{...}")
                    .family(egui::FontFamily::Monospace)
                    .color(egui::Color32::from_rgb(150, 150, 150)));
                if !has_children {
                    ui.label(egui::RichText::new("{}")
                        .family(egui::FontFamily::Monospace)
                        .color(egui::Color32::from_rgb(150, 150, 150)));
                } else {
                    let count = node.children.len();
                    ui.label(
                        egui::RichText::new(format!("({} field{})", count, if count == 1 { "" } else { "s" }))
                            .family(egui::FontFamily::Monospace)
                            .color(egui::Color32::from_rgb(150, 150, 150)),
                    );
                }
            }
            TreeNodeValue::List => {
                ui.label(egui::RichText::new("[...]")
                    .family(egui::FontFamily::Monospace)
                    .color(egui::Color32::from_rgb(150, 150, 150)));
                if !has_children {
                    ui.label(egui::RichText::new("[]")
                        .family(egui::FontFamily::Monospace)
                        .color(egui::Color32::from_rgb(150, 150, 150)));
                } else {
                    let count = node.children.len();
                    ui.label(
                        egui::RichText::new(format!("({} item{})", count, if count == 1 { "" } else { "s" }))
                            .family(egui::FontFamily::Monospace)
                            .color(egui::Color32::from_rgb(150, 150, 150)),
                    );
                }
            }
        }
    });

    // Render children if expanded
    if has_children {
        // Default to expanded (true) if not explicitly set
        let is_expanded = expanded_paths.get(&node.path).copied().unwrap_or(true);
        if is_expanded {
            for child in &node.children {
                render_tree_node(ui, child, expanded_paths, indent_level + 1);
            }
        }
    }
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
