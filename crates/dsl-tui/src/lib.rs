//! DSL TUI - Terminal User Interface
//!
//! This crate provides a terminal-based user interface for the DSL language using Ratatui.
//! It includes:
//! - Workspace mode with split Editor and REPL panes
//! - Syntax highlighting with Tree-sitter
//! - Text editor integration
//! - Interactive REPL with history

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

pub mod app;
pub mod autocomplete;
pub mod editor;
pub mod output_item;
pub mod renderers;
pub mod ui;
pub mod utf8_utils;

pub use app::App;

/// Run the TUI in non-interactive mode, reading from stdin
pub async fn run_stdin() -> io::Result<()> {
    use dsl_core::{parse_expr, compile_expr};
    use dsl_interpreter::Interpreter;
    use std::io::{BufRead, BufReader};

    let mut interpreter = Interpreter::new().expect("Failed to initialize interpreter");
    let stdin = io::stdin();
    let reader = BufReader::new(stdin);

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }

        // Execute the line using IR pipeline
        match parse_expr(trimmed) {
            Ok(ast) => {
                match compile_expr(&ast) {
                    Ok(ir_node) => {
                        match interpreter.eval(&ir_node).await {
                            Ok(value) => {
                                println!("✓ {}", value.type_name());
                                // Print the value
                                println!("{}", format_value_for_output(&value));
                            }
                            Err(err) => {
                                eprintln!("Error: {}", err);
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!("Compile error: {}", err);
                    }
                }
            }
            Err(err) => {
                eprintln!("Parse error: {}", err);
            }
        }
    }

    Ok(())
}

/// Format a value for output in non-interactive mode
fn format_value_for_output(value: &dsl_ir::Value) -> String {
    use dsl_ir::Value;

    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::String(s) => format!("\"{}\"", s),
        Value::List(items) => {
            let items_str: Vec<String> = items.iter().map(format_value_for_output).collect();
            format!("[{}]", items_str.join(", "))
        }
        Value::Map(map) => {
            let entries: Vec<String> = map
                .iter()
                .map(|(k, v)| format!("{}: {}", k, format_value_for_output(v)))
                .collect();
            format!("{{{}}}", entries.join(", "))
        }
        Value::Markdown(s) => s.clone(),
    }
}

/// Run the TUI application
pub async fn run_tui() -> io::Result<()> {
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

        // Check if app has requested to quit
        if app.should_quit {
            return Ok(());
        }

        let event = event::read()?;
        match event {
            Event::Mouse(mouse) => {
                // Check if mouse event is in the REPL area
                if mouse.column >= output_area.x
                    && mouse.column < output_area.x + output_area.width
                    && mouse.row >= output_area.y
                    && mouse.row < output_area.y + output_area.height
                {
                    let click_row = mouse.row.saturating_sub(output_area.y + 1); // +1 for border
                    let actual_line = click_row as usize + app.output_scroll_offset as usize;

                    match mouse.kind {
                        MouseEventKind::ScrollUp => {
                            app.scroll_up(3);
                        }
                        MouseEventKind::ScrollDown => {
                            app.scroll_down(3);
                        }
                        MouseEventKind::Down(button) => {
                            use crossterm::event::MouseButton;

                            // Right-click or Shift+Left-click starts selection
                            if button == MouseButton::Right
                                || (button == MouseButton::Left
                                    && mouse.modifiers.contains(KeyModifiers::SHIFT))
                            {
                                app.start_selection(actual_line);
                            } else {
                                // Regular left-click: handle tree/table interactions
                                app.handle_output_click(actual_line);
                                // Clear selection on regular click
                                app.clear_selection();
                            }
                        }
                        MouseEventKind::Drag(button) => {
                            // If we're dragging with right button or shift+left, update selection
                            use crossterm::event::MouseButton;
                            if button == MouseButton::Right
                                || (button == MouseButton::Left
                                    && mouse.modifiers.contains(KeyModifiers::SHIFT))
                            {
                                app.update_selection(actual_line);
                            }
                        }
                        MouseEventKind::Up(_button) => {
                            // End selection on mouse up
                            if app.is_selecting {
                                app.end_selection();
                            }
                        }
                        _ => {}
                    }
                }
            }
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                // Global keybindings
                match key.code {
                    // Ctrl+Shift+C: Copy selected text to clipboard
                    KeyCode::Char('C') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        use crate::output_item::OutputItem;
                        match app.copy_selection_to_clipboard() {
                            Ok(()) => {
                                app.output.push(OutputItem::text("✓ Copied to clipboard"));
                            }
                            Err(e) => {
                                app.output
                                    .push(OutputItem::error(format!("Copy failed: {}", e)));
                            }
                        }
                        // Don't quit - let the event handling continue
                    }
                    // Vim-like: Ctrl-C to quit (ESC removed to be more vim-like)
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(())
                    }
                    _ => {}
                }

                // Handle workspace input
                handle_workspace_input(app, key).await;
            }
            _ => {}
        }
    }
}

async fn handle_repl_input(app: &mut App, key: event::KeyEvent) {
    // Handle autocomplete-specific keys first
    if app.is_autocomplete_visible() {
        match key.code {
            // Tab accepts the selected suggestion
            KeyCode::Tab => {
                app.accept_autocomplete();
                return;
            }
            KeyCode::Down => {
                app.autocomplete_next();
                return;
            }
            KeyCode::Up => {
                app.autocomplete_previous();
                return;
            }
            KeyCode::Esc => {
                app.hide_autocomplete();
                return;
            }
            KeyCode::Enter => {
                // Enter accepts suggestion instead of executing
                if !key.modifiers.contains(KeyModifiers::ALT)
                    && !key.modifiers.contains(KeyModifiers::SHIFT)
                    && !app.should_continue_multiline()
                {
                    app.accept_autocomplete();
                    return;
                }
            }
            _ => {
                // Continue to handle other keys normally
            }
        }
    } else {
        // Tab triggers autocomplete when not visible (if not switching panes)
        if matches!(key.code, KeyCode::Tab) && !key.modifiers.contains(KeyModifiers::SHIFT) {
            app.trigger_autocomplete();
            return;
        }
    }

    match key.code {
        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.move_cursor_home();
            app.hide_autocomplete();
        }
        KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.move_cursor_end();
            app.hide_autocomplete();
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
            app.hide_autocomplete();
        }
        KeyCode::Right => {
            app.move_cursor_right();
            app.hide_autocomplete();
        }
        KeyCode::Up => {
            // History navigation when autocomplete not visible
            app.history_up();
            app.hide_autocomplete();
        }
        KeyCode::Down => {
            // History navigation when autocomplete not visible
            app.history_down();
            app.hide_autocomplete();
        }
        KeyCode::PageUp => {
            app.scroll_up(10);
            app.hide_autocomplete();
        }
        KeyCode::PageDown => {
            app.scroll_down(10);
            app.hide_autocomplete();
        }
        KeyCode::Home => {
            app.move_cursor_home();
            app.hide_autocomplete();
        }
        KeyCode::End => {
            app.move_cursor_end();
            app.hide_autocomplete();
        }
        KeyCode::Enter => {
            // Don't hide autocomplete here - it's handled above
            if key.modifiers.contains(KeyModifiers::ALT)
                || key.modifiers.contains(KeyModifiers::SHIFT)
                || app.should_continue_multiline()
            {
                app.hide_autocomplete();
                app.insert_newline();
            } else {
                // If autocomplete is visible, it was already accepted above
                // Otherwise, submit the input
                if !app.is_autocomplete_visible() {
                    app.submit_input().await;
                }
            }
        }
        KeyCode::Esc => {
            app.hide_autocomplete();
        }
        _ => {}
    }
}

async fn handle_workspace_input(app: &mut App, key: event::KeyEvent) {
    use app::WorkspacePane;

    // Global workspace keybindings
    match key.code {
        KeyCode::BackTab => {
            // Shift+Tab switches panes
            app.next_pane();
            return;
        }
        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            use crate::output_item::OutputItem;
            if let Err(e) = app.editor_save_file() {
                app.output
                    .push(OutputItem::error(format!("Save error: {}", e)));
            } else {
                app.output.push(OutputItem::text("✓ File saved"));
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
    // Let TextArea handle all input
    app.editor.input(key);
}
