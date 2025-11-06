use crate::autocomplete::AutocompleteState;
use crate::output_item::OutputItem;
use crate::ui::banner;
use dsl_ir::Value;
use dsl_interpreter::Interpreter;
use ratatui_image::picker::Picker;
use std::time::Duration;
use tui_textarea::TextArea;

#[derive(Debug, Clone, PartialEq)]
pub enum WorkspacePane {
    Editor,
    Repl,
    Preview,
}

#[derive(Clone)]
pub struct ExecutionStep {
    pub name: String,
    pub status: StepStatus,
    pub duration: Option<Duration>,
    pub output: Option<Value>,
}

#[derive(Clone)]
pub enum StepStatus {
    Pending,
    Running,
    Complete,
    Error(String),
}

pub struct App {
    // Workspace state
    pub active_pane: WorkspacePane,

    // REPL state
    pub input: String,
    pub output: Vec<OutputItem>,
    pub interpreter: Interpreter,
    pub show_banner: bool,
    pub is_loading: bool,
    pub cursor_position: usize,
    pub output_scroll_offset: u16,
    pub auto_scroll_output: bool,
    pub multiline_mode: bool,
    pub history: Vec<String>,
    pub history_index: Option<usize>,
    pub history_temp: String,
    pub should_quit: bool,
    pub autocomplete: AutocompleteState,

    // Editor state
    pub editor: TextArea<'static>,
    pub editor_file_path: Option<String>,

    // Preview state
    pub preview_steps: Vec<ExecutionStep>,

    // Focused output item for interaction (future use)
    pub focused_output_index: Option<usize>,

    // Selection state
    pub selection_start: Option<usize>, // Line number in output
    pub selection_end: Option<usize>,   // Line number in output
    pub is_selecting: bool,             // Whether mouse is currently held down

    // Cached render width (updated during rendering)
    pub last_render_width: usize,

    // Image protocol picker for rendering images
    pub image_picker: Picker,

    // Cached image protocols to avoid recreating on every frame (indexed by output item index)
    pub image_protocols: std::collections::HashMap<usize, Box<dyn ratatui_image::protocol::StatefulProtocol>>,

    // Toggle to show/hide images (for performance)
    pub show_images: bool,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let interpreter = Interpreter::new().expect("Failed to initialize interpreter");
        let autocomplete = AutocompleteState::new_with_runtime(&interpreter.runtime);

        // Initialize image picker for terminal graphics
        let mut image_picker = Picker::from_termios().unwrap_or_else(|_| {
            // Fallback to halfblocks if protocol detection fails
            Picker::new((8, 16))
        });
        image_picker.guess_protocol();

        let mut app = Self {
            active_pane: WorkspacePane::Repl, // Start with REPL active
            input: String::new(),
            output: vec![],
            interpreter,
            show_banner: true,
            is_loading: false,
            cursor_position: 0,
            output_scroll_offset: 0,
            auto_scroll_output: true,
            multiline_mode: false,
            history: vec![],
            history_index: None,
            history_temp: String::new(),
            should_quit: false,
            autocomplete,
            editor: TextArea::default(),
            editor_file_path: None,
            preview_steps: vec![],
            focused_output_index: None,
            selection_start: None,
            selection_end: None,
            is_selecting: false,
            last_render_width: 80, // Default, will be updated during render
            image_picker,
            image_protocols: std::collections::HashMap::new(),
            show_images: true, // Show images by default
        };
        app.load_history();
        app
    }

    /// Check if input has unclosed braces/brackets
    pub fn has_unclosed_delimiters(&self) -> bool {
        let mut brace_count = 0;
        let mut bracket_count = 0;
        let mut paren_count = 0;
        let mut in_string = false;
        let mut in_triple_string = false;
        let mut in_single_quote = false;
        let mut escape_next = false;

        let chars: Vec<char> = self.input.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if escape_next {
                escape_next = false;
                i += 1;
                continue;
            }

            let ch = chars[i];

            if ch == '\\' && (in_string || in_single_quote) {
                escape_next = true;
                i += 1;
                continue;
            }

            // Check for triple-quoted strings
            if i + 2 < chars.len()
                && chars[i] == '"'
                && chars[i + 1] == '"'
                && chars[i + 2] == '"'
                && !in_string
                && !in_single_quote
            {
                in_triple_string = !in_triple_string;
                i += 3;
                continue;
            }

            // Regular double quote
            if ch == '"' && !in_single_quote && !in_triple_string {
                in_string = !in_string;
                i += 1;
                continue;
            }

            // Single quote
            if ch == '\'' && !in_string && !in_triple_string {
                in_single_quote = !in_single_quote;
                i += 1;
                continue;
            }

            if !in_string && !in_triple_string && !in_single_quote {
                match ch {
                    '{' => brace_count += 1,
                    '}' => brace_count -= 1,
                    '[' => bracket_count += 1,
                    ']' => bracket_count -= 1,
                    '(' => paren_count += 1,
                    ')' => paren_count -= 1,
                    _ => {}
                }
            }

            i += 1;
        }

        brace_count > 0
            || bracket_count > 0
            || paren_count > 0
            || in_triple_string
            || in_string
            || in_single_quote
    }

    /// Check if we should enter multiline mode
    pub fn should_continue_multiline(&self) -> bool {
        // Check for unclosed delimiters
        if self.has_unclosed_delimiters() {
            return true;
        }

        // Check if line starts with keywords that typically span multiple lines
        let trimmed = self.input.trim_start();
        if trimmed.starts_with("def ")
            || trimmed.starts_with("type ")
            || trimmed.starts_with("enum ")
            || trimmed.starts_with("workflow ")
        {
            // If it starts with a def/type/enum but doesn't have matching braces, continue
            return self.has_unclosed_delimiters();
        }

        false
    }

    /// Insert newline for multiline input
    pub fn insert_newline(&mut self) {
        self.input.insert(self.cursor_position, '\n');
        self.cursor_position += 1;
        self.multiline_mode = true;
    }

    pub async fn submit_input(&mut self) {
        if self.input.is_empty() {
            return;
        }

        // Hide banner after first input
        self.show_banner = false;

        // Enable auto-scroll to show new output
        self.auto_scroll_output = true;

        let input_text = self.input.clone();

        // Add to history (avoid duplicate consecutive entries)
        if self.history.last() != Some(&input_text) {
            self.history.push(input_text.clone());
            self.save_history();
        }

        // Reset history navigation
        self.history_index = None;
        self.history_temp.clear();

        // Display input with proper formatting for multiline
        if input_text.contains('\n') {
            let mut input_display = String::from("flow> \n");
            for line in input_text.lines() {
                input_display.push_str(&format!("  {}\n", line));
            }
            self.output.push(OutputItem::text(input_display));
        } else {
            self.output
                .push(OutputItem::text(format!("flow> {}", input_text)));
        }

        // Handle special commands
        if input_text.trim() == ":help" {
            let help_text = banner::get_help_text().join("\n");
            self.output.push(OutputItem::text(help_text));
            self.output.push(OutputItem::text(""));
            self.input.clear();
            self.multiline_mode = false;
            return;
        }

        if input_text.trim() == ":clear" {
            self.output.clear();
            self.input.clear();
            self.cursor_position = 0;
            self.multiline_mode = false;
            self.output_scroll_offset = 0;
            self.auto_scroll_output = true;
            return;
        }

        if input_text.trim() == ":q" || input_text.trim() == ":quit" {
            // Vim-like quit command - signal to exit gracefully
            self.should_quit = true;
            self.input.clear();
            self.cursor_position = 0;
            self.multiline_mode = false;
            return;
        }

        // Set loading state
        self.is_loading = true;

        // New IR pipeline: parse -> compile -> interpret
        match self.eval_with_ir(&input_text).await {
            Ok((value, var_name)) => {
                let type_str = value.type_name();

                // Add a header line indicating the result
                if let Some(name) = var_name {
                    self.output.push(OutputItem::text(format!(
                        "✓ Bound '{}' : {}",
                        name, type_str
                    )));
                } else {
                    self.output
                        .push(OutputItem::text(format!("✓ {}", type_str)));
                }

                // Use the OutputItem system to display the value
                self.output.push(OutputItem::from_value(&value));

                // Refresh autocomplete to pick up new functions/variables/types
                self.refresh_autocomplete();
            }
            Err(err) => {
                self.output.push(OutputItem::error(format!("{}", err)));
            }
        }

        // Clear loading state
        self.is_loading = false;

        // Clear cached image protocols since output indices have changed
        self.image_protocols.clear();

        self.output.push(OutputItem::text(""));
        self.input.clear();
        self.cursor_position = 0;
        self.multiline_mode = false;
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            // Move to the start of the previous character (handles multi-byte UTF-8)
            let mut new_pos = self.cursor_position - 1;
            while new_pos > 0 && !self.input.is_char_boundary(new_pos) {
                new_pos -= 1;
            }
            self.cursor_position = new_pos;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input.len() {
            // Move to the start of the next character (handles multi-byte UTF-8)
            let mut new_pos = self.cursor_position + 1;
            while new_pos < self.input.len() && !self.input.is_char_boundary(new_pos) {
                new_pos += 1;
            }
            self.cursor_position = new_pos;
        }
    }

    pub fn move_cursor_home(&mut self) {
        self.cursor_position = 0;
    }

    pub fn move_cursor_end(&mut self) {
        self.cursor_position = self.input.len();
    }

    pub fn insert_char(&mut self, c: char) {
        // Exit history browsing mode when typing
        self.history_index = None;
        self.input.insert(self.cursor_position, c);
        // Move cursor by the UTF-8 byte length of the character (e.g., 3 bytes for Chinese)
        self.cursor_position += c.len_utf8();

        // Trigger autocomplete
        self.trigger_autocomplete();
    }

    pub fn delete_char(&mut self) {
        // Exit history browsing mode when deleting
        self.history_index = None;
        if self.cursor_position > 0 {
            // Find the start of the character before cursor (handles multi-byte UTF-8)
            let mut char_start = self.cursor_position - 1;
            while char_start > 0 && !self.input.is_char_boundary(char_start) {
                char_start -= 1;
            }

            // Remove the character and update cursor position
            self.input.remove(char_start);
            self.cursor_position = char_start;
        }

        // Trigger autocomplete
        self.trigger_autocomplete();
    }

    pub fn delete_char_forward(&mut self) {
        // Exit history browsing mode when deleting
        self.history_index = None;
        if self.cursor_position < self.input.len() {
            // The cursor is already at a character boundary (maintained by insert_char)
            // String::remove() will remove the full character at this position
            self.input.remove(self.cursor_position);
            // Cursor position stays the same after forward delete
        }

        // Trigger autocomplete
        self.trigger_autocomplete();
    }

    pub fn scroll_up(&mut self, lines: u16) {
        self.auto_scroll_output = false;
        self.output_scroll_offset = self.output_scroll_offset.saturating_sub(lines);
    }

    pub fn scroll_down(&mut self, lines: u16) {
        // We'll re-enable auto-scroll in the UI when we reach the bottom
        self.auto_scroll_output = false;
        self.output_scroll_offset += lines;
    }

    pub fn enable_auto_scroll(&mut self) {
        self.auto_scroll_output = true;
    }

    /// Handle a click on an output item (for interactive elements like trees)
    /// absolute_line: the line number in the full rendered output (including scroll offset)
    pub fn handle_output_click(&mut self, absolute_line: usize) {
        use crate::output_item::OutputItem;
        use crate::renderers::tree;
        use crate::renderers::OutputRenderer;

        // Account for banner if present
        let banner_lines = if self.show_banner && self.output.is_empty() {
            crate::ui::banner::get_repl_artwork().len() + 1 // +1 for blank line
        } else {
            0
        };

        if absolute_line < banner_lines {
            return; // Clicked on banner
        }

        let line_after_banner = absolute_line - banner_lines;

        // Find which output item contains this line
        let mut current_line = 0;
        for (item_idx, output_item) in self.output.iter_mut().enumerate() {
            // Calculate height with current expansion state
            let item_height = output_item.to_lines(self.last_render_width).len();

            if line_after_banner >= current_line && line_after_banner < current_line + item_height {
                // This is the output item that was clicked
                let line_within_item = line_after_banner - current_line;

                // If it's a tree, handle the click
                if let OutputItem::Tree {
                    root,
                    expanded_paths,
                } = output_item
                {
                    // Account for the hint line at the top (line 0)
                    // The actual tree nodes start at line 1
                    if line_within_item > 0 {
                        let tree_line = line_within_item - 1;

                        // Find which node was clicked (using current expansion state)
                        // Make a temporary clone of expanded_paths for finding the node
                        let temp_expanded = expanded_paths.clone();
                        if let Some(node) = tree::find_node_at_line(root, &temp_expanded, tree_line)
                        {
                            // Toggle expansion for this node
                            let path = node.path.clone();
                            let current_state = expanded_paths.get(&path).copied().unwrap_or(false);
                            expanded_paths.insert(path, !current_state);

                            // After toggling, adjust scroll if needed
                            // (prevents scroll offset from being beyond content after collapse)
                            self.auto_scroll_output = false;
                        }
                    }
                }

                // Store which output item is focused (for future enhancements)
                self.focused_output_index = Some(item_idx);
                break;
            }

            current_line += item_height;
        }
    }

    /// Start a selection at the given line
    pub fn start_selection(&mut self, line: usize) {
        self.selection_start = Some(line);
        self.selection_end = Some(line);
        self.is_selecting = true;
    }

    /// Update the selection end point (during drag)
    pub fn update_selection(&mut self, line: usize) {
        if self.is_selecting {
            self.selection_end = Some(line);
        }
    }

    /// End the selection
    pub fn end_selection(&mut self) {
        self.is_selecting = false;
    }

    /// Clear the selection
    pub fn clear_selection(&mut self) {
        self.selection_start = None;
        self.selection_end = None;
        self.is_selecting = false;
    }

    /// Get the selected text from the output
    pub fn get_selected_text(&mut self) -> Option<String> {
        use crate::renderers::OutputRenderer;

        let start = self.selection_start?;
        let end = self.selection_end?;

        let (start_line, end_line) = if start <= end {
            (start, end)
        } else {
            (end, start)
        };

        // Collect all lines from output
        let mut all_lines = Vec::new();

        // Add banner if present
        if self.show_banner && self.output.is_empty() {
            all_lines.extend(
                crate::ui::banner::get_repl_artwork()
                    .into_iter()
                    .map(|line| line.to_string()),
            );
            all_lines.push(String::new());
        }

        // Add output items
        for output_item in &mut self.output {
            let lines = output_item.to_lines(80);
            for line in lines {
                // Convert Line to String (extract text without styling)
                let text: String = line
                    .spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect();
                all_lines.push(text);
            }
        }

        // Extract selected lines
        let selected_lines: Vec<String> = all_lines
            .into_iter()
            .enumerate()
            .filter(|(i, _)| *i >= start_line && *i <= end_line)
            .map(|(_, line)| line)
            .collect();

        if selected_lines.is_empty() {
            None
        } else {
            Some(selected_lines.join("\n"))
        }
    }

    /// Copy selected text to clipboard
    pub fn copy_selection_to_clipboard(&mut self) -> Result<(), String> {
        if let Some(text) = self.get_selected_text() {
            use arboard::Clipboard;
            let mut clipboard = Clipboard::new().map_err(|e| format!("Clipboard error: {}", e))?;
            clipboard
                .set_text(text)
                .map_err(|e| format!("Failed to copy: {}", e))?;
            Ok(())
        } else {
            Err("No text selected".to_string())
        }
    }

    /// Navigate to previous history entry
    pub fn history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }

        match self.history_index {
            None => {
                // First time browsing history, save current input
                self.history_temp = self.input.clone();
                // Start from the last history entry
                self.history_index = Some(self.history.len() - 1);
                self.input = self.history[self.history.len() - 1].clone();
                self.cursor_position = self.input.len();
            }
            Some(index) => {
                if index > 0 {
                    self.history_index = Some(index - 1);
                    self.input = self.history[index - 1].clone();
                    self.cursor_position = self.input.len();
                }
            }
        }
    }

    /// Navigate to next history entry
    pub fn history_down(&mut self) {
        match self.history_index {
            None => {
                // Not browsing history, do nothing
            }
            Some(index) => {
                if index + 1 < self.history.len() {
                    self.history_index = Some(index + 1);
                    self.input = self.history[index + 1].clone();
                    self.cursor_position = self.input.len();
                } else {
                    // Reached the end, restore temporary input
                    self.history_index = None;
                    self.input = self.history_temp.clone();
                    self.cursor_position = self.input.len();
                    self.history_temp.clear();
                }
            }
        }
    }

    /// Get history file path
    fn history_file_path() -> Option<std::path::PathBuf> {
        if let Some(home) = std::env::var_os("HOME") {
            let mut path = std::path::PathBuf::from(home);
            path.push(".flow_history");
            Some(path)
        } else {
            None
        }
    }

    /// Load history from file
    fn load_history(&mut self) {
        if let Some(path) = Self::history_file_path() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                self.history = content
                    .lines()
                    .filter(|line| !line.is_empty())
                    .map(|line| line.to_string())
                    .collect();
            }
        }
    }

    /// Save history to file
    fn save_history(&self) {
        if let Some(path) = Self::history_file_path() {
            // Keep only last 1000 entries
            let start = self.history.len().saturating_sub(1000);
            let content = self.history[start..].join("\n");
            let _ = std::fs::write(&path, content);
        }
    }

    // Editor methods
    // TextArea handles all input internally, so we don't need manual methods

    pub fn editor_load_file(&mut self, path: &str) -> Result<(), std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        let lines: Vec<String> = if content.is_empty() {
            vec![String::new()]
        } else {
            content.lines().map(|s| s.to_string()).collect()
        };
        self.editor = TextArea::new(lines);
        self.editor_file_path = Some(path.to_string());
        Ok(())
    }

    pub fn editor_save_file(&self) -> Result<(), std::io::Error> {
        if let Some(path) = &self.editor_file_path {
            let content = self.editor.lines().join("\n");
            std::fs::write(path, content)?;
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No file path set",
            ))
        }
    }

    pub fn editor_get_content(&self) -> String {
        self.editor.lines().join("\n")
    }

    /// Send current line from editor to REPL for execution
    pub async fn send_current_line_to_repl(&mut self) {
        let (row, _col) = self.editor.cursor();
        if row < self.editor.lines().len() {
            let line = self.editor.lines()[row].clone();
            if !line.trim().is_empty() && !line.trim().starts_with("//") {
                // Set the REPL input to this line
                self.input = line;
                self.cursor_position = self.input.len();
                // Execute it
                self.submit_input().await;
                // Move to next line in editor
                self.editor.move_cursor(tui_textarea::CursorMove::Down);
            }
        }
    }

    /// Send all editor content to REPL for execution as logical blocks
    pub async fn send_all_to_repl(&mut self) {
        let content = self.editor_get_content();
        if content.trim().is_empty() {
            return;
        }

        // Split content into logical blocks (type definitions, function definitions, expressions)
        let blocks = self.split_into_blocks(&content);

        for block in blocks {
            if !block.trim().is_empty() && !block.trim().starts_with("//") {
                self.input = block;
                self.cursor_position = self.input.len();
                self.submit_input().await;
            }
        }
    }

    /// Split content into logical blocks (handles multi-line type/function definitions)
    fn split_into_blocks(&self, content: &str) -> Vec<String> {
        let mut blocks = Vec::new();
        let mut current_block = String::new();
        let mut brace_depth = 0;
        let mut in_block = false;

        for line in content.lines() {
            let trimmed = line.trim();

            // Skip comment-only lines
            if trimmed.starts_with("//") || trimmed.is_empty() {
                continue;
            }

            // Check if this line starts a new block (type, enum, def)
            if trimmed.starts_with("type ")
                || trimmed.starts_with("enum ")
                || trimmed.starts_with("def ")
            {
                // If we were in a previous block, save it
                if in_block && !current_block.trim().is_empty() {
                    blocks.push(current_block.trim().to_string());
                    current_block.clear();
                }
                in_block = true;
                brace_depth = 0;
            }

            if in_block {
                // Add line to current block
                if !current_block.is_empty() {
                    current_block.push('\n');
                }
                current_block.push_str(line);

                // Track braces to know when block is complete
                for ch in line.chars() {
                    match ch {
                        '{' => brace_depth += 1,
                        '}' => {
                            brace_depth -= 1;
                            if brace_depth == 0 {
                                // Block is complete
                                blocks.push(current_block.trim().to_string());
                                current_block.clear();
                                in_block = false;
                            }
                        }
                        _ => {}
                    }
                }
            } else if !trimmed.is_empty() {
                // Single-line expression
                blocks.push(trimmed.to_string());
            }
        }

        // Don't forget the last block if there is one
        if !current_block.trim().is_empty() {
            blocks.push(current_block.trim().to_string());
        }

        blocks
    }

    /// Switch active pane in workspace mode
    pub fn switch_pane(&mut self, pane: WorkspacePane) {
        self.active_pane = pane;
    }

    /// Cycle to next pane in workspace mode
    /// Cycle between Editor and REPL only (Preview is read-only, skip it)
    pub fn next_pane(&mut self) {
        self.active_pane = match self.active_pane {
            WorkspacePane::Editor => WorkspacePane::Repl,
            WorkspacePane::Repl => WorkspacePane::Editor,
            WorkspacePane::Preview => WorkspacePane::Editor, // Should not happen, but default to Editor
        };
    }

    /// Execute the editor content and populate preview steps
    pub async fn execute_editor_workflow(&mut self) -> Result<(), String> {
        use std::time::Instant;

        let content = self.editor_get_content();
        if content.trim().is_empty() {
            return Err("Editor is empty".to_string());
        }

        // Clear previous preview steps
        self.preview_steps.clear();

        // Split content into lines and execute each non-empty, non-comment line
        let lines: Vec<&str> = content
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with("//"))
            .collect();

        for (idx, line) in lines.iter().enumerate() {
            let step_name = format!(
                "Step {}: {}",
                idx + 1,
                if line.len() > 50 {
                    format!("{}...", &line[..47])
                } else {
                    line.to_string()
                }
            );

            // Add pending step
            self.preview_steps.push(ExecutionStep {
                name: step_name.clone(),
                status: StepStatus::Pending,
                duration: None,
                output: None,
            });

            // Update to running
            if let Some(step) = self.preview_steps.last_mut() {
                step.status = StepStatus::Running;
            }

            let start = Instant::now();

            // Execute the line using IR pipeline
            match self.eval_with_ir(line).await {
                Ok((value, _var_name)) => {
                    let duration = start.elapsed();
                    if let Some(step) = self.preview_steps.last_mut() {
                        step.status = StepStatus::Complete;
                        step.duration = Some(duration);
                        step.output = Some(value);
                    }
                }
                Err(err) => {
                    let duration = start.elapsed();
                    if let Some(step) = self.preview_steps.last_mut() {
                        step.status = StepStatus::Error(err.to_string());
                        step.duration = Some(duration);
                    }
                    // Continue execution even on error
                }
            }
        }

        Ok(())
    }

    // ========================================
    // IR Pipeline Evaluation
    // ========================================

    /// Evaluate input using the IR pipeline (parse -> compile -> interpret)
    async fn eval_with_ir(&mut self, input: &str) -> Result<(Value, Option<String>), String> {
        use dsl_core::{parse_expr, compile_expr};

        let input = input.trim();

        // Check for special commands first
        if input == ":vars" {
            return self.handle_vars_command();
        }
        if input == ":types" {
            return self.handle_types_command();
        }
        if input == ":funcs" || input == ":functions" {
            return self.handle_funcs_command();
        }
        if input.starts_with(":copy ") {
            return self.handle_copy_command(&input[6..]);
        }
        if input.starts_with(":save ") {
            return self.handle_save_command(&input[6..]);
        }
        if input.starts_with(":load ") {
            return self.handle_load_command(&input[6..]).await;
        }
        if input == ":debug" {
            return self.handle_debug_command();
        }

        // Parse the input to AST
        let ast = parse_expr(input).map_err(|e| format!("Parse error: {}", e))?;

        // Compile AST to IR
        let ir_node = compile_expr(&ast).map_err(|e| format!("Compile error: {}", e))?;

        // Evaluate IR
        let value = self.interpreter.eval(&ir_node).await?;

        // Check if this was a variable binding
        // For now, we don't track variable names in IR (future enhancement)
        // The interpreter handles bindings internally
        Ok((value, None))
    }

    /// Handle :vars command
    fn handle_vars_command(&self) -> Result<(Value, Option<String>), String> {
        let mut result = String::new();
        for (name, value) in &self.interpreter.runtime.vars {
            result.push_str(&format!("{} = {}\n", name, value.display()));
        }
        Ok((Value::String(result), None))
    }

    /// Handle :types command
    fn handle_types_command(&self) -> Result<(Value, Option<String>), String> {
        let mut result = String::new();
        for class in self.interpreter.runtime.types.all_classes() {
            result.push_str(&format!("{} {{\n", class.name));
            for field in &class.fields {
                result.push_str(&format!("  {}: {:?}\n", field.name, field.field_type));
            }
            result.push_str("}\n");
        }
        Ok((Value::String(result), None))
    }

    /// Handle :funcs command
    fn handle_funcs_command(&self) -> Result<(Value, Option<String>), String> {
        let mut result = String::new();
        for (name, func) in &self.interpreter.runtime.functions {
            result.push_str(&format!(
                "{}({}) -> {:?}\n",
                name,
                func.params.join(", "),
                func.return_type
            ));
        }
        Ok((Value::String(result), None))
    }

    /// Handle :copy command
    fn handle_copy_command(&self, _filename: &str) -> Result<(Value, Option<String>), String> {
        // This would need access to last_result - not implemented yet
        Err(":copy command not yet implemented in IR mode".to_string())
    }

    /// Handle :save command
    fn handle_save_command(&self, filename: &str) -> Result<(Value, Option<String>), String> {
        use std::fs;
        let json = serde_json::to_string_pretty(&self.interpreter.runtime.vars)
            .map_err(|e| format!("Serialization error: {}", e))?;
        fs::write(filename, json).map_err(|e| format!("Write error: {}", e))?;
        Ok((Value::String(format!("Session saved to {}", filename)), None))
    }

    /// Handle :load command
    async fn handle_load_command(&mut self, filename: &str) -> Result<(Value, Option<String>), String> {
        use std::fs;
        let json = fs::read_to_string(filename).map_err(|e| format!("Read error: {}", e))?;
        let vars: std::collections::HashMap<String, Value> = serde_json::from_str(&json)
            .map_err(|e| format!("Deserialization error: {}", e))?;
        self.interpreter.runtime.vars = vars;
        Ok((Value::String(format!("Session loaded from {}", filename)), None))
    }

    /// Handle :debug command
    fn handle_debug_command(&self) -> Result<(Value, Option<String>), String> {
        // This would need access to last_prompt from builtins - not implemented yet
        Err(":debug command not yet implemented in IR mode".to_string())
    }

    // ========================================
    // Autocomplete methods
    // ========================================

    /// Trigger autocomplete based on current input
    pub fn trigger_autocomplete(&mut self) {
        // Don't show autocomplete in multiline mode
        if self.multiline_mode {
            self.autocomplete.hide();
            return;
        }

        // Update autocomplete with current input
        self.autocomplete.update(&self.input, self.cursor_position);
    }

    /// Accept the currently selected autocomplete suggestion
    pub fn accept_autocomplete(&mut self) {
        self.autocomplete
            .accept_selected(&mut self.input, &mut self.cursor_position);
    }

    /// Navigate to next autocomplete suggestion
    pub fn autocomplete_next(&mut self) {
        self.autocomplete.select_next();
    }

    /// Navigate to previous autocomplete suggestion
    pub fn autocomplete_previous(&mut self) {
        self.autocomplete.select_previous();
    }

    /// Hide autocomplete popup
    pub fn hide_autocomplete(&mut self) {
        self.autocomplete.hide();
    }

    /// Check if autocomplete is currently visible
    pub fn is_autocomplete_visible(&self) -> bool {
        self.autocomplete.is_visible()
    }

    /// Refresh autocomplete providers (call after defining new functions/types)
    pub fn refresh_autocomplete(&mut self) {
        self.autocomplete = AutocompleteState::new_with_runtime(&self.interpreter.runtime);
    }

    /// Toggle image display on/off (for performance)
    pub fn toggle_image_display(&mut self) {
        self.show_images = !self.show_images;
    }
}
