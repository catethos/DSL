use crate::banner;
use crate::eval::Evaluator;
use crate::value::Value;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Repl,      // Full-screen REPL (original mode)
    Workspace, // Three-pane: Editor | REPL | Preview
    TypeExplorer,
}

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
    // Mode state
    pub mode: Mode,
    pub active_pane: WorkspacePane,

    // REPL state
    pub input: String,
    pub output: Vec<String>,
    pub evaluator: Evaluator,
    pub show_banner: bool,
    pub is_loading: bool,
    pub cursor_position: usize,
    pub output_scroll_offset: u16,
    pub auto_scroll_output: bool,
    pub multiline_mode: bool,
    pub history: Vec<String>,
    pub history_index: Option<usize>,
    pub history_temp: String,

    // Editor state
    pub editor_lines: Vec<String>,
    pub editor_cursor_row: usize,
    pub editor_cursor_col: usize,
    pub editor_scroll_offset: usize,
    pub editor_file_path: Option<String>,

    // Preview state
    pub preview_steps: Vec<ExecutionStep>,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            mode: Mode::Workspace,            // Start in workspace mode
            active_pane: WorkspacePane::Repl, // Start with REPL active
            input: String::new(),
            output: vec![],
            evaluator: Evaluator::new(),
            show_banner: true,
            is_loading: false,
            cursor_position: 0,
            output_scroll_offset: 0,
            auto_scroll_output: true,
            multiline_mode: false,
            history: vec![],
            history_index: None,
            history_temp: String::new(),
            editor_lines: vec![String::new()],
            editor_cursor_row: 0,
            editor_cursor_col: 0,
            editor_scroll_offset: 0,
            editor_file_path: None,
            preview_steps: vec![],
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
        let mut escape_next = false;

        for ch in self.input.chars() {
            if escape_next {
                escape_next = false;
                continue;
            }

            if ch == '\\' {
                escape_next = true;
                continue;
            }

            if ch == '"' {
                in_string = !in_string;
                continue;
            }

            if !in_string {
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
        }

        brace_count > 0 || bracket_count > 0 || paren_count > 0
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
            self.output.push("flow> ".to_string());
            for line in input_text.lines() {
                self.output.push(format!("  {}", line));
            }
        } else {
            self.output.push(format!("flow> {}", input_text));
        }

        // Handle special commands
        if input_text.trim() == ":help" {
            for line in banner::get_help_text() {
                self.output.push(line);
            }
            self.output.push("".to_string());
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

        // Set loading state
        self.is_loading = true;

        match self.evaluator.eval(&input_text).await {
            Ok((value, var_name)) => {
                let display_str = value.display();
                let type_str = value.type_name();

                if let Some(name) = var_name {
                    // Check if display is multi-line (e.g., table)
                    if display_str.contains('\n') {
                        self.output
                            .push(format!("✓ Bound '{}' : {}", name, type_str));
                        for line in display_str.lines() {
                            self.output.push(line.to_string());
                        }
                    } else {
                        self.output.push(format!(
                            "✓ Bound '{}' to {} : {}",
                            name, display_str, type_str
                        ));
                    }
                } else {
                    // Check if display is multi-line (e.g., table)
                    if display_str.contains('\n') {
                        self.output.push(format!("✓ {}", type_str));
                        for line in display_str.lines() {
                            self.output.push(line.to_string());
                        }
                    } else {
                        self.output
                            .push(format!("✓ {} : {}", display_str, type_str));
                    }
                }
            }
            Err(err) => {
                self.output.push(format!("✗ Error: {}", err));
            }
        }

        // Clear loading state
        self.is_loading = false;

        self.output.push("".to_string());
        self.input.clear();
        self.cursor_position = 0;
        self.multiline_mode = false;
    }

    pub fn move_cursor_left(&mut self) {
        self.cursor_position = self.cursor_position.saturating_sub(1);
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input.len() {
            self.cursor_position += 1;
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
        self.cursor_position += 1;
    }

    pub fn delete_char(&mut self) {
        // Exit history browsing mode when deleting
        self.history_index = None;
        if self.cursor_position > 0 {
            self.input.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
        }
    }

    pub fn delete_char_forward(&mut self) {
        // Exit history browsing mode when deleting
        self.history_index = None;
        if self.cursor_position < self.input.len() {
            self.input.remove(self.cursor_position);
        }
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

    pub fn editor_insert_char(&mut self, c: char) {
        // Ensure we have at least one line
        if self.editor_lines.is_empty() {
            self.editor_lines.push(String::new());
            self.editor_cursor_row = 0;
            self.editor_cursor_col = 0;
        }

        // Ensure cursor row is valid
        if self.editor_cursor_row >= self.editor_lines.len() {
            self.editor_lines.push(String::new());
        }

        // Ensure cursor col is within bounds
        let line_len = self.editor_lines[self.editor_cursor_row].len();
        if self.editor_cursor_col > line_len {
            self.editor_cursor_col = line_len;
        }

        self.editor_lines[self.editor_cursor_row].insert(self.editor_cursor_col, c);
        self.editor_cursor_col += 1;
    }

    pub fn editor_insert_newline(&mut self) {
        if self.editor_cursor_row >= self.editor_lines.len() {
            self.editor_lines.push(String::new());
            self.editor_cursor_row += 1;
            self.editor_cursor_col = 0;
            return;
        }

        let current_line = &self.editor_lines[self.editor_cursor_row];
        let after = current_line[self.editor_cursor_col..].to_string();
        self.editor_lines[self.editor_cursor_row].truncate(self.editor_cursor_col);
        self.editor_lines.insert(self.editor_cursor_row + 1, after);
        self.editor_cursor_row += 1;
        self.editor_cursor_col = 0;
    }

    pub fn editor_delete_char(&mut self) {
        if self.editor_cursor_col > 0 {
            self.editor_lines[self.editor_cursor_row].remove(self.editor_cursor_col - 1);
            self.editor_cursor_col -= 1;
        } else if self.editor_cursor_row > 0 {
            // Merge with previous line
            let current_line = self.editor_lines.remove(self.editor_cursor_row);
            self.editor_cursor_row -= 1;
            self.editor_cursor_col = self.editor_lines[self.editor_cursor_row].len();
            self.editor_lines[self.editor_cursor_row].push_str(&current_line);
        }
    }

    pub fn editor_move_left(&mut self) {
        if self.editor_cursor_col > 0 {
            self.editor_cursor_col -= 1;
        } else if self.editor_cursor_row > 0 {
            self.editor_cursor_row -= 1;
            self.editor_cursor_col = self.editor_lines[self.editor_cursor_row].len();
        }
    }

    pub fn editor_move_right(&mut self) {
        if self.editor_cursor_col < self.editor_lines[self.editor_cursor_row].len() {
            self.editor_cursor_col += 1;
        } else if self.editor_cursor_row < self.editor_lines.len() - 1 {
            self.editor_cursor_row += 1;
            self.editor_cursor_col = 0;
        }
    }

    pub fn editor_move_up(&mut self) {
        if self.editor_cursor_row > 0 {
            self.editor_cursor_row -= 1;
            let line_len = self.editor_lines[self.editor_cursor_row].len();
            self.editor_cursor_col = self.editor_cursor_col.min(line_len);
        }
    }

    pub fn editor_move_down(&mut self) {
        if self.editor_cursor_row < self.editor_lines.len() - 1 {
            self.editor_cursor_row += 1;
            let line_len = self.editor_lines[self.editor_cursor_row].len();
            self.editor_cursor_col = self.editor_cursor_col.min(line_len);
        }
    }

    pub fn editor_load_file(&mut self, path: &str) -> Result<(), std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        self.editor_lines = if content.is_empty() {
            vec![String::new()]
        } else {
            content.lines().map(|s| s.to_string()).collect()
        };
        self.editor_cursor_row = 0;
        self.editor_cursor_col = 0;
        self.editor_scroll_offset = 0;
        self.editor_file_path = Some(path.to_string());
        Ok(())
    }

    pub fn editor_save_file(&self) -> Result<(), std::io::Error> {
        if let Some(path) = &self.editor_file_path {
            let content = self.editor_lines.join("\n");
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
        self.editor_lines.join("\n")
    }

    /// Send current line from editor to REPL for execution
    pub async fn send_current_line_to_repl(&mut self) {
        if self.editor_cursor_row < self.editor_lines.len() {
            let line = self.editor_lines[self.editor_cursor_row].clone();
            if !line.trim().is_empty() && !line.trim().starts_with("//") {
                // Set the REPL input to this line
                self.input = line;
                self.cursor_position = self.input.len();
                // Execute it
                self.submit_input().await;
                // Move to next line in editor
                self.editor_move_down();
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

            // Execute the line
            match self.evaluator.eval(line).await {
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
}
