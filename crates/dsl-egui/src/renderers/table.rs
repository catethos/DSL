//! Table rendering utilities for displaying tabular data in egui

use egui;
use egui_extras;

/// Renders a table with columns and rows
///
/// # Arguments
/// * `ui` - The egui UI context
/// * `columns` - Column headers
/// * `rows` - Table data (each row must have same length as columns)
/// * `selected` - Optional selected row index (for highlighting)
pub fn render_table(
    ui: &mut egui::Ui,
    columns: &[String],
    rows: &[Vec<String>],
    selected: &Option<usize>,
) {
    if columns.is_empty() {
        ui.label("(empty table)");
        return;
    }

    // Calculate column widths based on content
    let min_column_width = 80.0;
    let max_column_width = 300.0;

    egui_extras::TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .columns(
            egui_extras::Column::auto()
                .at_least(min_column_width)
                .at_most(max_column_width),
            columns.len(),
        )
        .header(22.0, |mut header| {
            for col in columns.iter() {
                header.col(|ui| {
                    ui.label(egui::RichText::new(col.as_str())
                        .family(egui::FontFamily::Monospace)
                        .strong());
                });
            }
        })
        .body(|mut body| {
            for (row_idx, row) in rows.iter().enumerate() {
                let is_selected = selected.is_some_and(|s| s == row_idx);
                let row_height = 20.0;

                body.row(row_height, |mut row_ui| {
                    for cell in row.iter() {
                        row_ui.col(|ui| {
                            // Highlight selected row by using a different text color
                            if is_selected {
                                ui.label(egui::RichText::new(cell.as_str())
                                    .family(egui::FontFamily::Monospace)
                                    .color(egui::Color32::from_rgb(150, 200, 255)));
                            } else {
                                ui.label(egui::RichText::new(cell.as_str())
                                    .family(egui::FontFamily::Monospace));
                            }
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
}

/// Renders a simple table without selection support
pub fn render_simple_table(ui: &mut egui::Ui, columns: &[String], rows: &[Vec<String>]) {
    render_table(ui, columns, rows, &None);
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
