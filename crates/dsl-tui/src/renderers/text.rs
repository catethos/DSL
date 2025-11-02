use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

/// Wrap text to fit within the given width
pub fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![text.to_string()];
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;

    for word in text.split_whitespace() {
        let word_len = word.chars().count();

        // If adding this word would exceed width
        if current_width + word_len + 1 > width && !current_line.is_empty() {
            lines.push(current_line.clone());
            current_line.clear();
            current_width = 0;
        }

        // If the word itself is longer than width, break it
        if word_len > width {
            if !current_line.is_empty() {
                lines.push(current_line.clone());
                current_line.clear();
                current_width = 0;
            }

            // Break the word into chunks
            let chars: Vec<char> = word.chars().collect();
            for chunk in chars.chunks(width) {
                lines.push(chunk.iter().collect());
            }
        } else {
            if !current_line.is_empty() {
                current_line.push(' ');
                current_width += 1;
            }
            current_line.push_str(word);
            current_width += word_len;
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

/// Convert plain text to lines with word wrapping
pub fn text_to_lines(text: &str, width: usize) -> Vec<Line<'static>> {
    if text.is_empty() {
        return vec![Line::from("")];
    }

    let wrapped = wrap_text(text, width);
    wrapped
        .into_iter()
        .map(|line| Line::from(line))
        .collect()
}

/// Convert error text to styled lines
pub fn error_to_lines(text: &str, width: usize) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    // Add top border
    lines.push(Line::from(vec![Span::styled(
        format!("┌{}┐", "─".repeat(width.saturating_sub(2))),
        Style::default().fg(Color::Red),
    )]));

    // Add error text
    let wrapped = wrap_text(text, width.saturating_sub(4));
    for line in wrapped {
        lines.push(Line::from(vec![
            Span::styled("│ ", Style::default().fg(Color::Red)),
            Span::styled(
                format!("{:<width$}", line, width = width.saturating_sub(4)),
                Style::default().fg(Color::Red),
            ),
            Span::styled(" │", Style::default().fg(Color::Red)),
        ]));
    }

    // Add bottom border
    lines.push(Line::from(vec![Span::styled(
        format!("└{}┘", "─".repeat(width.saturating_sub(2))),
        Style::default().fg(Color::Red),
    )]));

    lines
}
