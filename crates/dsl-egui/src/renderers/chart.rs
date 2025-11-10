//! Chart rendering utilities using egui_plot

use crate::output_item::ChartType;
use dsl_ir::Value;
use egui;
use egui::ecolor::Color32;
use egui_plot::{Bar, BarChart, Legend, Line, Plot, PlotPoints};

/// Renders a chart using egui_plot
///
/// # Arguments
/// * `ui` - The egui UI context
/// * `chart_type` - The type of chart to render
/// * `data` - The data to display in the chart
pub fn render_chart(ui: &mut egui::Ui, chart_type: &ChartType, data: &Value) {
    // Generate a unique ID for this chart instance
    let chart_id = ui.id().with(format!("{:?}", data as *const _));
    match chart_type {
        ChartType::Bar => render_bar_chart(ui, data, chart_id),
        ChartType::Line => render_line_chart(ui, data, chart_id),
        ChartType::Scatter => render_scatter_plot(ui, data, chart_id),
        ChartType::Pie => {
            // Pie charts are not directly supported by egui_plot
            // Show a message for now
            ui.label(egui::RichText::new("[Pie chart rendering not yet implemented]")
                .family(egui::FontFamily::Monospace)
                .color(Color32::from_rgb(150, 150, 150)));
            ui.label(egui::RichText::new(format!("Data: {}", format_chart_data(data)))
                .family(egui::FontFamily::Monospace));
        }
    }
}

/// Render a bar chart
fn render_bar_chart(ui: &mut egui::Ui, data: &Value, chart_id: egui::Id) {
    let bars = extract_bars_from_value(data);

    if bars.is_empty() {
        ui.label(egui::RichText::new("(no data for bar chart)")
            .family(egui::FontFamily::Monospace));
        return;
    }

    Plot::new(chart_id)
        .legend(Legend::default())
        .show_axes([true, true])
        .height(300.0)
        .allow_zoom(false)
        .allow_drag(false)
        .allow_scroll(false)
        .allow_boxed_zoom(false)
        .show(ui, |plot_ui| {
            let chart = BarChart::new(bars).color(Color32::from_rgb(100, 150, 250));
            plot_ui.bar_chart(chart);
        });
}

/// Render a line chart
fn render_line_chart(ui: &mut egui::Ui, data: &Value, chart_id: egui::Id) {
    let points = extract_points_from_value(data);

    if points.is_empty() {
        ui.label(egui::RichText::new("(no data for line chart)")
            .family(egui::FontFamily::Monospace));
        return;
    }

    Plot::new(chart_id)
        .legend(Legend::default())
        .show_axes([true, true])
        .height(300.0)
        .allow_zoom(false)
        .allow_drag(false)
        .allow_scroll(false)
        .allow_boxed_zoom(false)
        .show(ui, |plot_ui| {
            let line = Line::new(PlotPoints::from(points))
                .color(Color32::from_rgb(100, 200, 100))
                .width(2.0);
            plot_ui.line(line);
        });
}

/// Render a scatter plot
fn render_scatter_plot(ui: &mut egui::Ui, data: &Value, chart_id: egui::Id) {
    let points = extract_points_from_value(data);

    if points.is_empty() {
        ui.label(egui::RichText::new("(no data for scatter plot)")
            .family(egui::FontFamily::Monospace));
        return;
    }

    Plot::new(chart_id)
        .legend(Legend::default())
        .show_axes([true, true])
        .height(300.0)
        .allow_zoom(false)
        .allow_drag(false)
        .allow_scroll(false)
        .allow_boxed_zoom(false)
        .show(ui, |plot_ui| {
            let scatter = egui_plot::Points::new(PlotPoints::from(points))
                .color(Color32::from_rgb(200, 100, 100))
                .radius(5.0);
            plot_ui.points(scatter);
        });
}

/// Extract bar data from a Value
///
/// Expected format:
/// - List of numbers: each becomes a bar at position [index]
/// - List of maps with "x" and "y" keys
/// - Map with "values" key containing a list
fn extract_bars_from_value(value: &Value) -> Vec<Bar> {
    match value {
        Value::List(items) => {
            // Try to extract as list of numbers (y values, x = index)
            let mut bars = Vec::new();
            for (i, item) in items.iter().enumerate() {
                if let Some(y) = extract_number(item) {
                    bars.push(Bar::new(i as f64, y));
                } else if let Value::Map(map) = item {
                    // Try to extract x, y from map
                    if let (Some(x_val), Some(y_val)) = (map.get("x"), map.get("y")) {
                        if let (Some(x), Some(y)) = (extract_number(x_val), extract_number(y_val)) {
                            bars.push(Bar::new(x, y));
                        }
                    }
                }
            }
            bars
        }
        Value::Map(map) => {
            // Try to extract from a "values" key
            if let Some(values) = map.get("values") {
                extract_bars_from_value(values)
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(),
    }
}

/// Extract point data from a Value
///
/// Expected format:
/// - List of numbers: each becomes a point at [index, value]
/// - List of lists with 2 elements: [[x1, y1], [x2, y2], ...]
/// - List of maps with "x" and "y" keys
fn extract_points_from_value(value: &Value) -> Vec<[f64; 2]> {
    match value {
        Value::List(items) => {
            let mut points = Vec::new();
            for (i, item) in items.iter().enumerate() {
                match item {
                    // Simple number: use index as x
                    _ if extract_number(item).is_some() => {
                        points.push([i as f64, extract_number(item).unwrap()]);
                    }
                    // List with 2 elements: [x, y]
                    Value::List(pair) if pair.len() == 2 => {
                        if let (Some(x), Some(y)) =
                            (extract_number(&pair[0]), extract_number(&pair[1]))
                        {
                            points.push([x, y]);
                        }
                    }
                    // Map with x, y keys
                    Value::Map(map) => {
                        if let (Some(x_val), Some(y_val)) = (map.get("x"), map.get("y")) {
                            if let (Some(x), Some(y)) =
                                (extract_number(x_val), extract_number(y_val))
                            {
                                points.push([x, y]);
                            }
                        }
                    }
                    _ => {}
                }
            }
            points
        }
        Value::Map(map) => {
            // Try to extract from a "points" or "data" key
            if let Some(data) = map.get("points").or_else(|| map.get("data")) {
                extract_points_from_value(data)
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(),
    }
}

/// Extract a numeric value from a Value
fn extract_number(value: &Value) -> Option<f64> {
    match value {
        Value::Int(i) => Some(*i as f64),
        Value::Float(f) => Some(*f),
        _ => None,
    }
}

/// Format chart data for display
fn format_chart_data(value: &Value) -> String {
    match value {
        Value::List(items) => format!("[{} items]", items.len()),
        Value::Map(_) => "{...}".to_string(),
        _ => format!("{:?}", value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;

    #[test]
    fn test_extract_bars_from_list() {
        let values = vec![Value::Int(10), Value::Int(20), Value::Int(30)];
        let data = Value::List(values);
        let bars = extract_bars_from_value(&data);
        assert_eq!(bars.len(), 3);
    }

    #[test]
    fn test_extract_points_from_list() {
        let values = vec![Value::Int(5), Value::Int(10), Value::Int(15)];
        let data = Value::List(values);
        let points = extract_points_from_value(&data);
        assert_eq!(points.len(), 3);
        assert_eq!(points[0], [0.0, 5.0]);
        assert_eq!(points[1], [1.0, 10.0]);
    }

    #[test]
    fn test_extract_number() {
        assert_eq!(extract_number(&Value::Int(42)), Some(42.0));
        assert_eq!(extract_number(&Value::Float(3.15)), Some(3.15));
        assert_eq!(extract_number(&Value::String("42".to_string())), None);
    }

    #[test]
    fn test_extract_points_from_pairs() {
        let values = vec![
            Value::List(vec![Value::Int(1), Value::Int(10)]),
            Value::List(vec![Value::Int(2), Value::Int(20)]),
            Value::List(vec![Value::Int(3), Value::Int(30)]),
        ];
        let data = Value::List(values);
        let points = extract_points_from_value(&data);
        assert_eq!(points.len(), 3);
        assert_eq!(points[0], [1.0, 10.0]);
        assert_eq!(points[2], [3.0, 30.0]);
    }

    #[test]
    fn test_extract_points_from_maps() {
        let mut map1 = IndexMap::new();
        map1.insert("x".to_string(), Value::Int(1));
        map1.insert("y".to_string(), Value::Int(100));

        let mut map2 = IndexMap::new();
        map2.insert("x".to_string(), Value::Int(2));
        map2.insert("y".to_string(), Value::Int(200));

        let values = vec![Value::Map(map1), Value::Map(map2)];
        let data = Value::List(values);
        let points = extract_points_from_value(&data);
        assert_eq!(points.len(), 2);
        assert_eq!(points[0], [1.0, 100.0]);
        assert_eq!(points[1], [2.0, 200.0]);
    }
}
