use ratatui::{
    layout::Rect,
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::highlight;

pub fn render_editor(f: &mut Frame, area: Rect, app: &mut App) {
    let title = if let Some(path) = &app.editor_file_path {
        format!("Editor - {}", path)
    } else {
        "Editor - <unsaved>".to_string()
    };
    render_editor_with_title(f, area, app, &title);
}

pub fn render_editor_with_title(f: &mut Frame, area: Rect, app: &mut App, title: &str) {
    // Get the full text from the editor
    let editor_text = app.editor.lines().join("\n");

    // Apply Tree-sitter syntax highlighting to get styled lines
    let highlighted_lines = if editor_text.is_empty() {
        vec![Line::from("")]
    } else {
        highlight::highlight_text(&editor_text)
    };

    // Get cursor position from TextArea
    let (cursor_row, cursor_col) = app.editor.cursor();

    // Create paragraph with syntax highlighting
    let paragraph = Paragraph::new(highlighted_lines)
        .block(Block::default().borders(Borders::ALL).title(title));

    f.render_widget(paragraph, area);

    // Set cursor position (accounting for borders and line numbers if any)
    let cursor_x = area.x + 1 + cursor_col as u16;
    let cursor_y = area.y + 1 + cursor_row as u16;

    // Only set cursor if it's within bounds
    if cursor_y < area.y + area.height.saturating_sub(1) {
        f.set_cursor_position((cursor_x, cursor_y));
    }
}
