use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

mod app;
mod banner;
mod builtin;
mod editor;
mod eval;
mod parser;
mod preview;
mod sql;
mod types;
mod ui;
mod value;

use app::App;

#[tokio::main]
async fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    // Track output area bounds for mouse events
    let mut output_area = ratatui::layout::Rect::default();

    loop {
        terminal.draw(|f| {
            output_area = ui::draw(f, app);
        })?;

        let event = event::read()?;
        match event {
            Event::Mouse(mouse) => {
                // Check if mouse event is in the REPL area
                if mouse.column >= output_area.x
                    && mouse.column < output_area.x + output_area.width
                    && mouse.row >= output_area.y
                    && mouse.row < output_area.y + output_area.height
                {
                    match mouse.kind {
                        MouseEventKind::ScrollUp => {
                            app.scroll_up(3);
                        }
                        MouseEventKind::ScrollDown => {
                            app.scroll_down(3);
                        }
                        _ => {}
                    }
                }
            }
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                // Global keybindings (work in all modes)
                match key.code {
                    KeyCode::Esc => return Ok(()),
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(())
                    }
                    KeyCode::F(1) => {
                        app.mode = app::Mode::Repl;
                        continue;
                    }
                    KeyCode::F(2) => {
                        app.mode = app::Mode::Workspace;
                        continue;
                    }
                    KeyCode::F(3) => {
                        app.mode = app::Mode::TypeExplorer;
                        continue;
                    }
                    _ => {}
                }

                // Mode-specific keybindings
                match app.mode {
                    app::Mode::Repl => handle_repl_input(app, key).await,
                    app::Mode::Workspace => handle_workspace_input(app, key).await,
                    app::Mode::TypeExplorer => {
                        // Read-only mode, no input handling needed
                    }
                }
            }
            _ => {}
        }
    }
}

async fn handle_repl_input(app: &mut App, key: event::KeyEvent) {
    match key.code {
        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.move_cursor_home();
        }
        KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.move_cursor_end();
        }
        KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.insert_char('\n');
        }
        KeyCode::Char('m') if key.modifiers.contains(KeyModifiers::ALT) => {
            app.insert_char('\n');
        }
        KeyCode::Char(c) => {
            app.insert_char(c);
        }
        KeyCode::Backspace => {
            app.delete_char();
        }
        KeyCode::Delete => {
            app.delete_char_forward();
        }
        KeyCode::Left => {
            app.move_cursor_left();
        }
        KeyCode::Right => {
            app.move_cursor_right();
        }
        KeyCode::Up => {
            // Always use history navigation in REPL mode
            app.history_up();
        }
        KeyCode::Down => {
            // Always use history navigation in REPL mode
            app.history_down();
        }
        KeyCode::PageUp => {
            app.scroll_up(10);
        }
        KeyCode::PageDown => {
            app.scroll_down(10);
        }
        KeyCode::Home => {
            app.move_cursor_home();
        }
        KeyCode::End => {
            app.move_cursor_end();
        }
        KeyCode::Enter => {
            if key.modifiers.contains(KeyModifiers::ALT)
                || key.modifiers.contains(KeyModifiers::SHIFT)
            {
                app.insert_newline();
            } else {
                if app.should_continue_multiline() {
                    app.insert_newline();
                } else {
                    app.submit_input().await;
                }
            }
        }
        _ => {}
    }
}

async fn handle_workspace_input(app: &mut App, key: event::KeyEvent) {
    use app::WorkspacePane;

    // Global workspace keybindings
    match key.code {
        KeyCode::Tab => {
            app.next_pane();
            return;
        }
        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if let Err(e) = app.editor_save_file() {
                app.output.push(format!("✗ Save error: {}", e));
            } else {
                app.output.push("✓ File saved".to_string());
            }
            return;
        }
        KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            // Send current line to REPL
            app.send_current_line_to_repl().await;
            return;
        }
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            // Run all editor content
            app.send_all_to_repl().await;
            return;
        }
        _ => {}
    }

    // Pane-specific keybindings
    match app.active_pane {
        WorkspacePane::Editor => handle_editor_pane_input(app, key),
        WorkspacePane::Repl => handle_repl_input(app, key).await,
        WorkspacePane::Preview => {
            // Preview pane removed, should not reach here
            // Do nothing
        }
    }
}

fn handle_editor_pane_input(app: &mut App, key: event::KeyEvent) {
    match key.code {
        KeyCode::Char(c) => {
            app.editor_insert_char(c);
        }
        KeyCode::Backspace => {
            app.editor_delete_char();
        }
        KeyCode::Left => {
            app.editor_move_left();
        }
        KeyCode::Right => {
            app.editor_move_right();
        }
        KeyCode::Up => {
            app.editor_move_up();
        }
        KeyCode::Down => {
            app.editor_move_down();
        }
        KeyCode::PageUp => {
            // Scroll up in editor
            app.editor_scroll_offset = app.editor_scroll_offset.saturating_sub(10);
            app.editor_cursor_row = app.editor_cursor_row.saturating_sub(10);
        }
        KeyCode::PageDown => {
            // Scroll down in editor
            app.editor_scroll_offset =
                (app.editor_scroll_offset + 10).min(app.editor_lines.len().saturating_sub(1));
            app.editor_cursor_row =
                (app.editor_cursor_row + 10).min(app.editor_lines.len().saturating_sub(1));
        }
        KeyCode::Enter => {
            app.editor_insert_newline();
        }
        KeyCode::Home => {
            app.editor_cursor_col = 0;
        }
        KeyCode::End => {
            if app.editor_cursor_row < app.editor_lines.len() {
                app.editor_cursor_col = app.editor_lines[app.editor_cursor_row].len();
            }
        }
        _ => {}
    }

    // Update scroll to keep cursor visible
    let visible_height = 30; // Approximate, will be calculated properly in render
    if app.editor_cursor_row < app.editor_scroll_offset {
        app.editor_scroll_offset = app.editor_cursor_row;
    } else if app.editor_cursor_row >= app.editor_scroll_offset + visible_height {
        app.editor_scroll_offset = app.editor_cursor_row - visible_height + 1;
    }
}

fn handle_editor_input(app: &mut App, key: event::KeyEvent) {
    match key.code {
        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            // Save file
            if let Err(e) = app.editor_save_file() {
                // TODO: Show error message
                eprintln!("Failed to save: {}", e);
            }
        }
        KeyCode::Char('o') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            // TODO: Open file dialog or prompt
        }
        KeyCode::Char(c) => {
            app.editor_insert_char(c);
        }
        KeyCode::Backspace => {
            app.editor_delete_char();
        }
        KeyCode::Left => {
            app.editor_move_left();
        }
        KeyCode::Right => {
            app.editor_move_right();
        }
        KeyCode::Up => {
            app.editor_move_up();
        }
        KeyCode::Down => {
            app.editor_move_down();
        }
        KeyCode::Enter => {
            app.editor_insert_newline();
        }
        KeyCode::Home => {
            app.editor_cursor_col = 0;
        }
        KeyCode::End => {
            if app.editor_cursor_row < app.editor_lines.len() {
                app.editor_cursor_col = app.editor_lines[app.editor_cursor_row].len();
            }
        }
        _ => {}
    }

    // Update scroll to keep cursor visible
    let visible_height = 30; // Approximate, will be calculated properly in render
    if app.editor_cursor_row < app.editor_scroll_offset {
        app.editor_scroll_offset = app.editor_cursor_row;
    } else if app.editor_cursor_row >= app.editor_scroll_offset + visible_height {
        app.editor_scroll_offset = app.editor_cursor_row - visible_height + 1;
    }
}
