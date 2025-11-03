use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

use crate::app::{App, WorkspacePane};
use crate::renderers::OutputRenderer;
use crate::ui::{autocomplete, banner};

pub fn draw(f: &mut Frame, app: &mut App) -> Rect {
    draw_workspace(f, app)
}

fn draw_workspace(f: &mut Frame, app: &mut App) -> Rect {
    // Create main layout with keybindings at bottom
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(f.area());

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
        "Tab: Switch Pane | Ctrl+E: Send Line | Ctrl+R: Run All | Ctrl+S: Save | :q or Ctrl+C: Quit",
    );

    // Return the active pane's area
    match app.active_pane {
        WorkspacePane::Repl => workspace_chunks[0],
        WorkspacePane::Editor => workspace_chunks[1],
        WorkspacePane::Preview => workspace_chunks[0], // Should not happen
    }
}

fn draw_editor_pane(f: &mut Frame, area: Rect, app: &mut App, title: &str) {
    use crate::ui::highlight;

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

fn draw_repl_pane(f: &mut Frame, area: Rect, app: &mut App, title: &str) {
    // Build REPL content with artwork at the top
    let mut repl_text: Vec<Line> = Vec::new();

    // Add ASCII artwork if this is the first interaction (no output yet)
    if app.output.is_empty() {
        repl_text.extend(banner::get_repl_artwork());
        repl_text.push(Line::from(""));
    }

    // Add history output using the OutputRenderer
    let repl_width = area.width.saturating_sub(2) as usize;

    // Update cached render width for click handling
    app.last_render_width = repl_width;

    // Calculate selection range if any
    let (selection_start, selection_end) =
        if let (Some(start), Some(end)) = (app.selection_start, app.selection_end) {
            if start <= end {
                (start, end)
            } else {
                (end, start)
            }
        } else {
            (usize::MAX, usize::MAX) // No selection
        };

    let mut current_line = 0;

    for output_item in &app.output {
        let mut lines = output_item.to_lines(repl_width);

        // Apply selection highlighting to lines in selection range
        for line in &mut lines {
            if current_line >= selection_start && current_line <= selection_end {
                // Apply selection highlighting by modifying span styles
                for span in &mut line.spans {
                    span.style = span.style.bg(Color::DarkGray);
                }
            }
            current_line += 1;
        }

        repl_text.extend(lines);
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

    // Update app scroll offset to clamped value (important after content height changes)
    app.output_scroll_offset = scroll_offset;

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
        use crate::utf8_utils::safe_prefix;

        let text_before_cursor = safe_prefix(&app.input, app.cursor_position);
        let cursor_input_line = text_before_cursor.matches('\n').count();
        let input_line_count = app.input.lines().count().max(1);
        let cursor_global_line = total_lines.saturating_sub(input_line_count) + cursor_input_line;
        let cursor_visible_line = cursor_global_line.saturating_sub(scroll_offset as usize);

        let line_start = text_before_cursor
            .rfind('\n')
            .map(|pos| pos + 1)
            .unwrap_or(0);

        // Calculate display width (visual columns) instead of byte length
        // Chinese characters take 2 columns each, ASCII takes 1
        use unicode_width::UnicodeWidthStr;
        let current_line_text = &text_before_cursor[line_start..];
        let cursor_col_in_input = current_line_text.width();
        let prompt_width = if cursor_input_line == 0 { 6 } else { 5 };
        let cursor_x = area.x + 1 + prompt_width + cursor_col_in_input as u16;
        let cursor_y = area.y + 1 + cursor_visible_line as u16;

        if cursor_visible_line < visible_lines && cursor_y < area.y + area.height.saturating_sub(1)
        {
            f.set_cursor_position((cursor_x, cursor_y));

            // Render autocomplete popup if visible
            if app.is_autocomplete_visible() {
                // Calculate cursor position relative to area
                let cursor_rel_x = cursor_x.saturating_sub(area.x);
                let cursor_rel_y = cursor_y.saturating_sub(area.y);

                autocomplete::render_autocomplete_popup(
                    area,
                    f.buffer_mut(),
                    &app.autocomplete,
                    cursor_rel_x,
                    cursor_rel_y,
                );
            }
        }
    }
}

fn render_keybindings(f: &mut Frame, text: &str) {
    let area = Rect {
        x: 0,
        y: f.area().height.saturating_sub(1),
        width: f.area().width,
        height: 1,
    };

    let keybindings = Paragraph::new(Line::from(vec![Span::styled(
        format!(" {} ", text),
        Style::default().fg(Color::Black).bg(Color::Cyan),
    )]));

    f.render_widget(keybindings, area);
}
