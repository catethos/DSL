use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

use crate::app::{App, Mode, WorkspacePane};
use crate::banner;
use crate::preview;

/// Wrap text to fit within the given width
fn wrap_text(text: &str, width: usize) -> Vec<String> {
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

pub fn draw(f: &mut Frame, app: &mut App) -> Rect {
    match app.mode {
        Mode::Repl => draw_repl(f, app),
        Mode::Workspace => draw_workspace(f, app),
        Mode::TypeExplorer => draw_type_explorer_mode(f, app),
    }
}

fn draw_repl(f: &mut Frame, app: &App) -> Rect {
    // Single pane for all REPL content (history + current input)
    let area = f.size();

    // Build the complete REPL view: banner + history + prompt + current input
    let mut repl_text: Vec<Line> = Vec::new();

    // Show banner if this is the first screen
    if app.show_banner {
        repl_text.extend(banner::get_banner());
    }

    // Add history output with word wrapping
    let repl_width = area.width.saturating_sub(2) as usize; // Account for borders
    for line in &app.output {
        if line.is_empty() {
            repl_text.push(Line::from(""));
        } else if line.starts_with('┌')
            || line.starts_with('├')
            || line.starts_with('│')
            || line.starts_with('└')
        {
            // Don't wrap table lines - they use box-drawing characters
            repl_text.push(Line::from(line.as_str()));
        } else {
            // Wrap long lines
            let wrapped = wrap_text(line, repl_width);
            for wrapped_line in wrapped {
                repl_text.push(Line::from(wrapped_line));
            }
        }
    }

    // Add current prompt and input
    if app.input.is_empty() {
        // Show prompt only
        repl_text.push(Line::from(vec![Span::styled(
            "flow> ",
            Style::default().fg(Color::Green),
        )]));
    } else {
        // Show prompt with first line of input
        let input_lines: Vec<&str> = app.input.lines().collect();
        if let Some(first_line) = input_lines.first() {
            repl_text.push(Line::from(vec![
                Span::styled("flow> ", Style::default().fg(Color::Green)),
                Span::styled(*first_line, Style::default().fg(Color::Yellow)),
            ]));
        }
        // Show continuation lines with "...> " prefix
        for line in input_lines.iter().skip(1) {
            repl_text.push(Line::from(vec![
                Span::styled("...> ", Style::default().fg(Color::Green)),
                Span::styled(*line, Style::default().fg(Color::Yellow)),
            ]));
        }
    }

    // Calculate scroll position to keep cursor visible
    let total_lines = repl_text.len();
    let visible_lines = area.height.saturating_sub(2) as usize; // Account for borders

    // Calculate which line the cursor is on (in the complete repl_text)
    let text_before_cursor = &app.input[..app.cursor_position];
    let cursor_input_line = text_before_cursor.matches('\n').count();

    // Auto-scroll to keep cursor visible
    let scroll_offset = if app.auto_scroll_output {
        // Keep the bottom visible (where we're typing)
        total_lines.saturating_sub(visible_lines) as u16
    } else {
        // Manual scroll position
        app.output_scroll_offset
            .min(total_lines.saturating_sub(visible_lines) as u16)
    };

    let title = if app.is_loading {
        "DSL REPL (⏳ Loading...)"
    } else if app.multiline_mode || app.has_unclosed_delimiters() {
        "DSL REPL (Multi-line: Shift+Enter for newline, Enter when done)"
    } else {
        "DSL REPL (Shift+Enter for newline, Enter to submit)"
    };

    let paragraph = Paragraph::new(repl_text)
        .block(Block::default().borders(Borders::ALL).title(title))
        .scroll((scroll_offset, 0));

    f.render_widget(paragraph, area);

    // Add scrollbar if needed
    if total_lines > visible_lines {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .style(Style::default().fg(Color::Cyan));

        let mut scrollbar_state = ScrollbarState::new(total_lines.saturating_sub(visible_lines))
            .position(scroll_offset as usize);

        let scrollbar_area = Rect {
            x: area.x + area.width.saturating_sub(1),
            y: area.y + 1,
            width: 1,
            height: area.height.saturating_sub(2),
        };

        f.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }

    // Set cursor position
    // The cursor is on the line: (total_lines - input_line_count + current_input_line)
    let input_line_count = app.input.lines().count().max(1);
    let cursor_global_line = total_lines.saturating_sub(input_line_count) + cursor_input_line;
    let cursor_visible_line = cursor_global_line.saturating_sub(scroll_offset as usize);

    // Calculate column position (including "flow> " or "...> " prefix)
    let line_start = text_before_cursor
        .rfind('\n')
        .map(|pos| pos + 1)
        .unwrap_or(0);
    let cursor_col_in_input = app.cursor_position - line_start;
    let prompt_width = if cursor_input_line == 0 { 6 } else { 5 }; // "flow> " or "...> "
    let cursor_x = area.x + 1 + prompt_width + cursor_col_in_input as u16;
    let cursor_y = area.y + 1 + cursor_visible_line as u16;

    // Only set cursor if it's visible
    if cursor_visible_line < visible_lines && cursor_y < area.y + area.height.saturating_sub(1) {
        f.set_cursor(cursor_x, cursor_y);
    }

    area
}

fn draw_workspace(f: &mut Frame, app: &mut App) -> Rect {
    // Create main layout with keybindings at bottom
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(f.size());

    // Split workspace into two vertical panes (50/50)
    let workspace_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // REPL
            Constraint::Percentage(50), // Editor
        ])
        .split(main_chunks[0]);

    // Draw REPL pane (left side)
    let repl_title = if app.active_pane == WorkspacePane::Repl {
        "[ REPL ]"
    } else {
        "REPL"
    };
    draw_repl_pane(f, workspace_chunks[0], app, repl_title);

    // Draw editor pane (right side)
    let editor_title = if app.active_pane == WorkspacePane::Editor {
        "[ Editor ] (Ctrl+E: Send line, Ctrl+R: Run all)"
    } else {
        "Editor"
    };
    draw_editor_pane(f, workspace_chunks[1], app, editor_title);

    // Render key bindings at bottom
    render_keybindings(
        f,
        "F1: Full REPL | Tab: Switch Pane | Ctrl+E: Send Line | Ctrl+R: Run All | Ctrl+S: Save | Esc: Quit",
    );

    // Return the active pane's area
    match app.active_pane {
        WorkspacePane::Repl => workspace_chunks[0],
        WorkspacePane::Editor => workspace_chunks[1],
        WorkspacePane::Preview => workspace_chunks[0], // Should not happen
    }
}

fn draw_editor_pane(f: &mut Frame, area: Rect, app: &mut App, title: &str) {
    use crate::highlight;

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

    // Set cursor position if this is the active pane (accounting for borders)
    if app.active_pane == WorkspacePane::Editor {
        let cursor_x = area.x + 1 + cursor_col as u16;
        let cursor_y = area.y + 1 + cursor_row as u16;

        // Only set cursor if it's within bounds
        if cursor_y < area.y + area.height.saturating_sub(1) {
            f.set_cursor_position((cursor_x, cursor_y));
        }
    }
}

fn draw_repl_pane(f: &mut Frame, area: Rect, app: &App, title: &str) {
    // Build REPL content with artwork at the top
    let mut repl_text: Vec<Line> = Vec::new();

    // Add ASCII artwork if this is the first interaction (no output yet)
    if app.output.is_empty() {
        repl_text.extend(banner::get_repl_artwork());
        repl_text.push(Line::from(""));
    }

    // Add history output with word wrapping
    let repl_width = area.width.saturating_sub(2) as usize;
    for line in &app.output {
        if line.is_empty() {
            repl_text.push(Line::from(""));
        } else if line.starts_with('┌')
            || line.starts_with('├')
            || line.starts_with('│')
            || line.starts_with('└')
        {
            repl_text.push(Line::from(line.as_str()));
        } else {
            let wrapped = wrap_text(line, repl_width);
            for wrapped_line in wrapped {
                repl_text.push(Line::from(wrapped_line));
            }
        }
    }

    // Add current prompt and input (only if REPL pane is active)
    if app.active_pane == WorkspacePane::Repl {
        if app.input.is_empty() {
            repl_text.push(Line::from(vec![Span::styled(
                "flow> ",
                Style::default().fg(Color::Green),
            )]));
        } else {
            let input_lines: Vec<&str> = app.input.lines().collect();
            if let Some(first_line) = input_lines.first() {
                repl_text.push(Line::from(vec![
                    Span::styled("flow> ", Style::default().fg(Color::Green)),
                    Span::styled(*first_line, Style::default().fg(Color::Yellow)),
                ]));
            }
            for line in input_lines.iter().skip(1) {
                repl_text.push(Line::from(vec![
                    Span::styled("...> ", Style::default().fg(Color::Green)),
                    Span::styled(*line, Style::default().fg(Color::Yellow)),
                ]));
            }
        }
    }

    let total_lines = repl_text.len();
    let visible_lines = area.height.saturating_sub(2) as usize;

    let scroll_offset = if app.auto_scroll_output {
        total_lines.saturating_sub(visible_lines) as u16
    } else {
        app.output_scroll_offset
            .min(total_lines.saturating_sub(visible_lines) as u16)
    };

    let paragraph = Paragraph::new(repl_text)
        .block(Block::default().borders(Borders::ALL).title(title))
        .scroll((scroll_offset, 0));

    f.render_widget(paragraph, area);

    // Add scrollbar if needed
    if total_lines > visible_lines {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .style(Style::default().fg(Color::Cyan));

        let mut scrollbar_state = ScrollbarState::new(total_lines.saturating_sub(visible_lines))
            .position(scroll_offset as usize);

        let scrollbar_area = Rect {
            x: area.x + area.width.saturating_sub(1),
            y: area.y + 1,
            width: 1,
            height: area.height.saturating_sub(2),
        };

        f.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }

    // Set cursor position if REPL pane is active
    if app.active_pane == WorkspacePane::Repl && !app.input.is_empty() {
        let text_before_cursor = &app.input[..app.cursor_position];
        let cursor_input_line = text_before_cursor.matches('\n').count();
        let input_line_count = app.input.lines().count().max(1);
        let cursor_global_line = total_lines.saturating_sub(input_line_count) + cursor_input_line;
        let cursor_visible_line = cursor_global_line.saturating_sub(scroll_offset as usize);

        let line_start = text_before_cursor
            .rfind('\n')
            .map(|pos| pos + 1)
            .unwrap_or(0);
        let cursor_col_in_input = app.cursor_position - line_start;
        let prompt_width = if cursor_input_line == 0 { 6 } else { 5 };
        let cursor_x = area.x + 1 + prompt_width + cursor_col_in_input as u16;
        let cursor_y = area.y + 1 + cursor_visible_line as u16;

        if cursor_visible_line < visible_lines && cursor_y < area.y + area.height.saturating_sub(1)
        {
            f.set_cursor(cursor_x, cursor_y);
        }
    }
}

fn draw_type_explorer_mode(f: &mut Frame, app: &mut App) -> Rect {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(f.size());

    preview::render_type_explorer(f, chunks[0], app);

    // Render key bindings at bottom
    render_keybindings(f, "F1: REPL | F2: Editor | F4: Preview | Esc: Quit");

    chunks[0]
}

fn render_keybindings(f: &mut Frame, text: &str) {
    let area = Rect {
        x: 0,
        y: f.size().height.saturating_sub(1),
        width: f.size().width,
        height: 1,
    };

    let keybindings = Paragraph::new(Line::from(vec![Span::styled(
        format!(" {} ", text),
        Style::default().fg(Color::Black).bg(Color::Cyan),
    )]));

    f.render_widget(keybindings, area);
}
