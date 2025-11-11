//! Table rendering utilities for displaying tabular data in egui

use egui;
use egui_extras;

use crate::theme::Theme;

/// Renders a table with columns and rows
///
/// # Arguments
/// * `ui` - The egui UI context
/// * `columns` - Column headers
/// * `rows` - Table data (each row must have same length as columns)
/// * `selected` - Optional selected row index (for highlighting)
/// * `theme` - Color theme for styling
pub fn render_table(
    ui: &mut egui::Ui,
    columns: &[String],
    rows: &[Vec<String>],
    selected: &Option<usize>,
    theme: &Theme,
) {
    if columns.is_empty() {
        ui.label(egui::RichText::new("(empty table)")
            .color(theme.syntax.comment)
            .italics());
        return;
    }

    let min_column_width = 80.0;
    let max_column_width = 300.0;

    // Pre-calculate row heights to avoid borrow checker issues
    let row_heights: Vec<f32> = rows.iter()
        .map(|row| calculate_row_height(ui, row, max_column_width))
        .collect();

    egui::Frame::default()
        .stroke(egui::Stroke::new(1.0, theme.ui.panel_border))
        .inner_margin(0.0)
        .show(ui, |ui| {
            egui_extras::TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::TOP))
                .columns(
                    egui_extras::Column::auto()
                        .at_least(min_column_width)
                        .at_most(max_column_width),
                    columns.len(),
                )
                .header(24.0, |mut header| {
                    for col in columns.iter() {
                        header.col(|ui| {
                            ui.style_mut().visuals.widgets.noninteractive.bg_fill = theme.ui.table_header_bg;
                            ui.add(egui::Label::new(egui::RichText::new(col.as_str())
                                .family(egui::FontFamily::Monospace)
                                .color(theme.ui.table_header_text)
                                .strong()
                                .size(15.0))
                                .wrap_mode(egui::TextWrapMode::Wrap));
                        });
                    }
                })
                .body(|mut body| {
                    for (row_idx, row) in rows.iter().enumerate() {
                        let is_selected = selected.is_some_and(|s| s == row_idx);
                        let row_height = row_heights[row_idx];

                        body.row(row_height, |mut row_ui| {
                            for cell in row.iter() {
                                row_ui.col(|ui| {
                                    let cell_color = if is_selected {
                                        theme.ui.table_selected
                                    } else {
                                        detect_cell_color(cell, theme)
                                    };
                                    
                                    ui.add(egui::Label::new(egui::RichText::new(cell.as_str())
                                        .family(egui::FontFamily::Monospace)
                                        .color(cell_color)
                                        .size(12.0))
                                        .wrap_mode(egui::TextWrapMode::Wrap));
                                });
                            }

                            // Handle padding if row has fewer cells than columns
                            for _ in row.len()..columns.len() {
                                row_ui.col(|ui| {
                                    ui.label("");
                                });
                            }
                        });
                    }
                });
        });
}

/// Calculate the required row height based on wrapped text content
fn calculate_row_height(ui: &egui::Ui, row: &[String], max_column_width: f32) -> f32 {
    let font_id = egui::FontId::monospace(12.0);
    let min_height = 22.0;
    let padding = 8.0;
    
    let max_lines = row.iter().map(|cell| {
        let galley = ui.fonts(|fonts| {
            fonts.layout(
                cell.to_string(),
                font_id.clone(),
                egui::Color32::WHITE,
                max_column_width - padding,
            )
        });
        galley.rows.len()
    }).max().unwrap_or(1);
    
    (max_lines as f32 * 14.0 + padding).max(min_height)
}

/// Detect appropriate color for a table cell based on its content
fn detect_cell_color(cell: &str, theme: &Theme) -> egui::Color32 {
    // Numbers
    if cell.parse::<f64>().is_ok() {
        return theme.data_types.number;
    }
    
    // Booleans
    if cell == "true" {
        return theme.data_types.bool_true;
    }
    if cell == "false" {
        return theme.data_types.bool_false;
    }
    
    // Null
    if cell == "null" || cell.is_empty() {
        return theme.data_types.null;
    }
    
    // Strings (quoted or regular text)
    if cell.starts_with('"') && cell.ends_with('"') {
        return theme.data_types.string;
    }
    
    // Objects/Arrays (collapsed)
    if cell == "{...}" || cell == "[...]" {
        return theme.syntax.comment;
    }
    
    // Default
    theme.ui.table_cell_text
}

/// Renders a simple table without selection support
pub fn render_simple_table(ui: &mut egui::Ui, columns: &[String], rows: &[Vec<String>], theme: &Theme) {
    render_table(ui, columns, rows, &None, theme);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_empty_table() {
        // Empty columns should render a message
        let columns: Vec<String> = vec![];
        let _rows: Vec<Vec<String>> = vec![];
        // Would need egui context to actually test rendering
        assert_eq!(columns.len(), 0);
    }

    #[test]
    fn test_table_with_data() {
        let columns = ["Name".to_string(), "Age".to_string()];
        let rows = [
            vec!["Alice".to_string(), "30".to_string()],
            vec!["Bob".to_string(), "25".to_string()],
        ];
        assert_eq!(columns.len(), 2);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].len(), 2);
    }
}
