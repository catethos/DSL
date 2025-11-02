use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

/// Render a table as lines for display
pub fn table_to_lines(
    columns: &[String],
    rows: &[Vec<String>],
    selected: Option<usize>,
    _width: usize,
) -> Vec<Line<'static>> {
    if columns.is_empty() {
        return vec![Line::from("(empty table)")];
    }

    let mut lines = Vec::new();

    // Calculate column widths
    let mut col_widths: Vec<usize> = columns.iter().map(|c| c.len()).collect();

    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < col_widths.len() {
                col_widths[i] = col_widths[i].max(cell.len());
            }
        }
    }

    // Limit column width to 30 characters
    for width in col_widths.iter_mut() {
        *width = (*width).min(30);
    }

    // Top border
    let mut top_border = String::from("┌");
    for (i, width) in col_widths.iter().enumerate() {
        top_border.push_str(&"─".repeat(width + 2));
        if i < col_widths.len() - 1 {
            top_border.push('┬');
        }
    }
    top_border.push('┐');
    lines.push(Line::from(Span::styled(
        top_border,
        Style::default().fg(Color::Cyan),
    )));

    // Header row
    let mut header_spans = vec![Span::styled("│", Style::default().fg(Color::Cyan))];
    for (i, col) in columns.iter().enumerate() {
        let width = col_widths[i];
        header_spans.push(Span::styled(
            format!(" {:<width$} ", col, width = width),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
        header_spans.push(Span::styled("│", Style::default().fg(Color::Cyan)));
    }
    lines.push(Line::from(header_spans));

    // Header separator
    let mut header_sep = String::from("├");
    for (i, width) in col_widths.iter().enumerate() {
        header_sep.push_str(&"─".repeat(width + 2));
        if i < col_widths.len() - 1 {
            header_sep.push('┼');
        }
    }
    header_sep.push('┤');
    lines.push(Line::from(Span::styled(
        header_sep,
        Style::default().fg(Color::Cyan),
    )));

    // Data rows (show first 20 rows)
    let display_rows = rows.iter().take(20).enumerate();
    for (row_idx, row) in display_rows {
        let is_selected = selected == Some(row_idx);
        let mut row_spans = vec![Span::styled("│", Style::default().fg(Color::Cyan))];

        for (i, cell) in row.iter().enumerate() {
            if i < col_widths.len() {
                let width = col_widths[i];
                let cell_str = if cell.len() > 30 {
                    format!("{}...", &cell.chars().take(27).collect::<String>())
                } else {
                    cell.clone()
                };

                let style = if is_selected {
                    Style::default()
                        .bg(Color::Blue)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                row_spans.push(Span::styled(
                    format!(" {:<width$} ", cell_str, width = width),
                    style,
                ));
                row_spans.push(Span::styled("│", Style::default().fg(Color::Cyan)));
            }
        }

        lines.push(Line::from(row_spans));
    }

    // Bottom border
    let mut bottom_border = String::from("└");
    for (i, width) in col_widths.iter().enumerate() {
        bottom_border.push_str(&"─".repeat(width + 2));
        if i < col_widths.len() - 1 {
            bottom_border.push('┴');
        }
    }
    bottom_border.push('┘');
    lines.push(Line::from(Span::styled(
        bottom_border,
        Style::default().fg(Color::Cyan),
    )));

    // Row count
    let count_text = if rows.len() > 20 {
        format!("({} rows shown, {} total)", 20, rows.len())
    } else {
        format!("({} rows)", rows.len())
    };
    lines.push(Line::from(Span::styled(
        count_text,
        Style::default().fg(Color::DarkGray),
    )));

    lines
}
