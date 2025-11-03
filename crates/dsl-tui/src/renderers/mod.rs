pub mod markdown;
pub mod table;
pub mod text;
pub mod tree;

use crate::output_item::OutputItem;
use ratatui::text::Line;

/// Trait for rendering different output types
pub trait OutputRenderer {
    /// Convert the output item to lines for display in a Paragraph widget
    /// This allows all output to be rendered together in a scrollable view
    fn to_lines(&self, width: usize) -> Vec<Line<'static>>;
}

impl OutputRenderer for OutputItem {
    fn to_lines(&self, width: usize) -> Vec<Line<'static>> {
        match self {
            OutputItem::Text(s) => text::text_to_lines(s, width),
            OutputItem::Table {
                columns,
                rows,
                selected,
            } => table::table_to_lines(columns, rows, *selected, width),
            OutputItem::Tree {
                root,
                expanded_paths,
            } => tree::tree_to_lines(root, expanded_paths, width),
            OutputItem::Error(s) => text::error_to_lines(s, width),
            OutputItem::Markdown(s) => markdown::markdown_to_lines(s, width),
            OutputItem::Chart { .. } => {
                vec![Line::from("[Chart rendering not yet implemented]")]
            }
        }
    }
}
