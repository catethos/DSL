use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render_editor(f: &mut Frame, area: Rect, app: &App) {
    let title = if let Some(path) = &app.editor_file_path {
        format!("Editor - {}", path)
    } else {
        "Editor - <unsaved>".to_string()
    };
    render_editor_with_title(f, area, app, &title);
}

pub fn render_editor_with_title(f: &mut Frame, area: Rect, app: &App, title: &str) {
    let visible_height = area.height.saturating_sub(2) as usize; // Account for borders
    let visible_start = app.editor_scroll_offset;
    let visible_end = (visible_start + visible_height).min(app.editor_lines.len());

    let lines: Vec<Line> = app.editor_lines[visible_start..visible_end]
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let line_num = visible_start + i + 1;
            let highlighted = highlight_syntax(line);

            let mut spans = vec![Span::styled(
                format!("{:4} │ ", line_num),
                Style::default().fg(Color::DarkGray),
            )];
            spans.extend(highlighted);

            Line::from(spans)
        })
        .collect();

    let editor = Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(title));

    f.render_widget(editor, area);

    // Set cursor position (only if we're in Workspace mode and Editor pane is active)
    // Note: In full-screen editor mode (which doesn't exist anymore), this would always show cursor
    // In workspace mode, cursor is controlled by the active_pane check in ui.rs
}

pub fn highlight_syntax_public(line: &str) -> Vec<Span<'_>> {
    highlight_syntax(line)
}

fn highlight_syntax(line: &str) -> Vec<Span<'_>> {
    let keywords = [
        "type", "enum", "workflow", "as", "sql", "prompt", "http", "def", "let",
    ];
    let operators = [">>", "||", "?:", "->"];
    let types = ["String", "Int", "Float", "Bool", "List", "Map"];

    let mut spans = Vec::new();
    let mut current_word = String::new();
    let mut in_string = false;
    let mut string_content = String::new();
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        // Handle strings
        if ch == '"' && !in_string {
            // Flush current word
            if !current_word.is_empty() {
                spans.push(highlight_word(&current_word, &keywords, &types));
                current_word.clear();
            }

            in_string = true;
            string_content.push(ch);
        } else if ch == '"' && in_string {
            string_content.push(ch);
            spans.push(Span::styled(
                string_content.clone(),
                Style::default().fg(Color::Green),
            ));
            string_content.clear();
            in_string = false;
        } else if in_string {
            string_content.push(ch);
        } else if ch.is_whitespace() {
            // Flush current word
            if !current_word.is_empty() {
                spans.push(highlight_word(&current_word, &keywords, &types));
                current_word.clear();
            }
            spans.push(Span::raw(ch.to_string()));
        } else if ch.is_alphanumeric() || ch == '_' {
            current_word.push(ch);
        } else {
            // Flush current word
            if !current_word.is_empty() {
                spans.push(highlight_word(&current_word, &keywords, &types));
                current_word.clear();
            }

            // Check for multi-char operators
            let mut op = ch.to_string();
            if let Some(&next_ch) = chars.peek() {
                let two_char = format!("{}{}", ch, next_ch);
                if operators.contains(&two_char.as_str()) {
                    op = two_char;
                    chars.next(); // Consume the next character
                }
            }

            // Highlight operators
            let style = if operators.contains(&op.as_str())
                || ch == '{'
                || ch == '}'
                || ch == '['
                || ch == ']'
                || ch == '('
                || ch == ')'
            {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if ch == ':' {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            spans.push(Span::styled(op, style));
        }
    }

    // Flush remaining content
    if in_string {
        spans.push(Span::styled(
            string_content,
            Style::default().fg(Color::Green),
        ));
    } else if !current_word.is_empty() {
        spans.push(highlight_word(&current_word, &keywords, &types));
    }

    if spans.is_empty() {
        spans.push(Span::raw(""));
    }

    spans
}

fn highlight_word(word: &str, keywords: &[&str], types: &[&str]) -> Span<'static> {
    let style = if keywords.contains(&word) {
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD)
    } else if types.contains(&word) {
        Style::default()
            .fg(Color::Blue)
            .add_modifier(Modifier::BOLD)
    } else if word.parse::<i64>().is_ok() || word.parse::<f64>().is_ok() {
        Style::default().fg(Color::Yellow)
    } else if word == "true" || word == "false" {
        Style::default().fg(Color::LightYellow)
    } else {
        Style::default()
    };

    Span::styled(word.to_string(), style)
}
