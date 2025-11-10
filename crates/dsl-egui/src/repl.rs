use eframe::egui;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use crate::animations::AnimationType;
use crate::autocomplete::AutocompleteState;
use crate::output_item::{OutputItem, ErrorDetail};
use crate::renderers;
use crate::syntax::{highlight_code, ColorScheme};

use dsl_core::{compile_function_group, resolve_program, SymbolTable};

// Re-export AnimationType for convenience
pub use crate::animations::AnimationType as Animation;

/// Maximum number of output items to keep in history
/// This prevents unbounded memory growth with large outputs
const MAX_OUTPUT_ITEMS: usize = 1000;

/// REPL pane with input and output
pub struct ReplPane {
    /// Current input text
    input: String,

    /// Cursor position within input (byte offset, not char offset)
    cursor_position: usize,

    /// Output history
    output: VecDeque<OutputItem>,

    /// Command history
    history: Vec<String>,

    /// Current position in history (None = at end)
    history_index: Option<usize>,

    /// Temporary storage for current input when navigating history
    temp_input: String,

    /// Auto-scroll to bottom
    auto_scroll: bool,

    /// Interpreter (wrapped in Arc<Mutex> for thread safety)
    interpreter: Arc<Mutex<dsl_interpreter::Interpreter>>,

    /// Channel for receiving evaluation results
    result_rx: Option<mpsc::UnboundedReceiver<EvalResult>>,

    /// Channel for sending evaluation requests
    result_tx: mpsc::UnboundedSender<EvalResult>,

    /// Color scheme for syntax highlighting
    color_scheme: ColorScheme,

    /// Current animation type
    animation_type: AnimationType,

    /// Autocomplete state
    autocomplete: AutocompleteState,

    /// Flag to indicate we're accepting an autocomplete suggestion
    accepting_suggestion: bool,

    /// Persistent symbol table for resolver (thread-safe)
    symbol_table: Arc<Mutex<SymbolTable>>,

    /// Cache for markdown rendering
    markdown_cache: egui_commonmark::CommonMarkCache,
}

#[derive(Debug)]
struct EvalResult {
    result: Result<dsl_ir::Value, String>,
}

impl ReplPane {
    /// Add an output item and trim old items if we exceed MAX_OUTPUT_ITEMS
    fn push_output(&mut self, item: OutputItem) {
        self.output.push_back(item);

        // Trim old items if we exceed the limit
        while self.output.len() > MAX_OUTPUT_ITEMS {
            self.output.pop_front();
        }
    }

    pub fn new() -> Self {
        let mut output = VecDeque::new();
        output.push_back(OutputItem::Text(
            "Welcome to DSL Interactive Environment!".to_string(),
        ));
        output.push_back(OutputItem::Text(
            "Type ':help' for help, or enter DSL expressions to evaluate.".to_string(),
        ));

        let interpreter = Arc::new(Mutex::new(
            dsl_interpreter::Interpreter::new().expect("Failed to initialize interpreter"),
        ));

        let (result_tx, result_rx) = mpsc::unbounded_channel();

        // Initialize autocomplete with runtime
        let autocomplete = {
            let interp = interpreter.lock().unwrap();
            AutocompleteState::new(&interp.runtime)
        };

        Self {
            input: String::new(),
            cursor_position: 0,
            output,
            history: Vec::new(),
            history_index: None,
            temp_input: String::new(),
            auto_scroll: true,
            interpreter,
            result_rx: Some(result_rx),
            result_tx,
            color_scheme: ColorScheme::dark(),
            animation_type: AnimationType::None, // Default to None for best performance
            autocomplete,
            accepting_suggestion: false,
            symbol_table: Arc::new(Mutex::new(SymbolTable::new())),
            markdown_cache: egui_commonmark::CommonMarkCache::default(),
        }
    }

    pub fn set_animation_type(&mut self, animation_type: AnimationType) {
        self.animation_type = animation_type;
    }

    pub fn animation_type(&self) -> AnimationType {
        self.animation_type
    }

    /// Handle Tab key for autocomplete (returns true if Tab was handled)
    pub fn handle_tab(&mut self) -> bool {
        if self.autocomplete.is_visible() {
            // Accept the selected suggestion
            self.autocomplete
                .accept_selected(&mut self.input, &mut self.cursor_position);
            true
        } else {
            // Trigger autocomplete
            self.trigger_autocomplete();
            true
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        // Check for evaluation results
        let mut results = Vec::new();
        if let Some(rx) = &mut self.result_rx {
            while let Ok(result) = rx.try_recv() {
                results.push(result);
            }
        }
        for result in results {
            self.handle_eval_result(result);
        }

        // CRITICAL: Consume Tab BEFORE any widgets if autocomplete is visible
        let tab_consumed_for_autocomplete = self.autocomplete.is_visible()
            && ui.input_mut(|i| {
                if i.key_pressed(egui::Key::Tab) {
                    i.consume_key(egui::Modifiers::NONE, egui::Key::Tab);
                    true
                } else {
                    false
                }
            });

        let total_height = ui.available_height();

        ui.vertical(|ui| {
            // Output area - account for input area and separators (approx 100px)
            let output_height = (total_height - 100.0).max(100.0);

            egui::ScrollArea::vertical()
                .id_salt("repl_output_scroll_area")
                .min_scrolled_height(output_height)
                .max_height(output_height)
                .auto_shrink([false, false])
                .stick_to_bottom(self.auto_scroll)
                .show(ui, |ui| {
                    // Display animated plot header
                    let time = ui.input(|i| i.time);
                    crate::animations::render_animation(ui, self.animation_type, time);

                    // Request continuous repaint ONLY if animation needs it
                    if self.animation_type.needs_animation() {
                        ui.ctx().request_repaint();
                    }

                    ui.add_space(8.0); // Add space after header

                    // Render output items with conditional visibility-based culling
                    // Only use culling if we have many items to avoid flickering with small lists
                    let total_items = self.output.len();
                    let use_culling = total_items > 30; // Only cull if more than 30 items

                    if !use_culling {
                        // Small number of items - just render them all for smoothness
                        for i in 0..total_items {
                            let item_id = ui.id().with(("output_item", i));
                            ui.push_id(item_id, |ui| {
                                self.render_output_item_at(ui, i);
                            });
                            ui.add_space(4.0);
                        }
                    } else {
                        // Many items - use visibility-based culling
                        let viewport_rect = ui.clip_rect();
                        let mut current_y = ui.cursor().min.y;

                        for i in 0..total_items {
                            let item_id = ui.id().with(("output_item", i));

                            // Estimate item height (rough heuristic)
                            let estimated_height = match &self.output[i] {
                                OutputItem::Chart { .. } => 320.0, // Charts are ~300px + padding
                                OutputItem::Table { rows, .. } => {
                                    (rows.len() as f32 * 20.0 + 30.0).min(400.0)
                                }
                                OutputItem::Image { .. } => 250.0, // Images vary but estimate conservatively
                                OutputItem::Tree { .. } => 100.0,  // Trees vary, estimate medium
                                OutputItem::Markdown(_) => 80.0,
                                _ => 30.0, // Text, errors
                            };

                            // Check if this item is potentially visible (with very generous margin)
                            let item_bottom = current_y + estimated_height;
                            let margin = 1500.0; // Large margin to prevent flickering during scroll
                            let is_near_viewport = (current_y >= viewport_rect.min.y - margin
                                && current_y <= viewport_rect.max.y + margin)
                                || (item_bottom >= viewport_rect.min.y - margin
                                    && item_bottom <= viewport_rect.max.y + margin);

                            if is_near_viewport {
                                // Render full item
                                ui.push_id(item_id, |ui| {
                                    self.render_output_item_at(ui, i);
                                });
                                ui.add_space(4.0);
                                current_y = ui.cursor().min.y;
                            } else {
                                // Render lightweight placeholder to maintain scroll position
                                ui.add_space(estimated_height);
                                current_y += estimated_height + 4.0;
                            }
                        }
                    }
                });

            ui.separator();

            // Input area
            let text_edit_id = egui::Id::new("repl_text_input");

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("flow>")
                    .family(egui::FontFamily::Monospace));

                // Use the tab_consumed_for_autocomplete flag from above
                let tab_pressed = tab_consumed_for_autocomplete;

                // Build TextEdit with lock_focus to prevent Tab from stealing focus
                let mut text_edit = egui::TextEdit::multiline(&mut self.input)
                    .id(text_edit_id)
                    .desired_width(f32::INFINITY)
                    .desired_rows(3)
                    .font(egui::TextStyle::Monospace);

                // Lock focus when autocomplete is visible OR Tab was consumed
                // This prevents the TextEdit from losing focus when Tab is pressed
                if self.autocomplete.is_visible() || tab_pressed {
                    text_edit = text_edit.lock_focus(true);
                }

                let mut response = ui.add(text_edit);

                // POST-PROCESS: Accept autocomplete with Tab (after TextEdit is rendered)
                if tab_pressed {
                    self.accepting_suggestion = true;
                    self.autocomplete
                        .accept_selected(&mut self.input, &mut self.cursor_position);

                    // Request focus and mark as gained
                    response.request_focus();
                    ui.memory_mut(|mem| mem.request_focus(text_edit_id));

                    // Update TextEdit state to match Enter handling
                    if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), text_edit_id) {
                        // Convert byte position to character position
                        let char_pos = self.input[..self.cursor_position].chars().count();
                        let ccursor = egui::text::CCursor::new(char_pos);
                        state
                            .cursor
                            .set_char_range(Some(egui::text::CCursorRange::one(ccursor)));
                        state.store(ui.ctx(), text_edit_id);
                    }
                    response.mark_changed();

                    // Request repaint to show updated cursor
                    ui.ctx().request_repaint();
                }

                // Handle autocomplete keyboard input AFTER TextEdit is rendered
                if self.autocomplete.is_visible() && response.has_focus() {
                    // Enter to accept autocomplete
                    if ui.input_mut(|i| {
                        if i.key_pressed(egui::Key::Enter) && !i.modifiers.shift {
                            i.consume_key(egui::Modifiers::NONE, egui::Key::Enter);
                            true
                        } else {
                            false
                        }
                    }) {
                        self.accepting_suggestion = true;
                        self.autocomplete
                            .accept_selected(&mut self.input, &mut self.cursor_position);

                        // Update TextEdit state
                        if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), text_edit_id)
                        {
                            // Convert byte position to character position
                            let char_pos = self.input[..self.cursor_position].chars().count();
                            let ccursor = egui::text::CCursor::new(char_pos);
                            state
                                .cursor
                                .set_char_range(Some(egui::text::CCursorRange::one(ccursor)));
                            state.store(ui.ctx(), text_edit_id);
                        }
                        response.mark_changed();
                        response.request_focus();
                    }
                    // Arrow Up
                    else if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                        self.autocomplete.select_previous();
                    }
                    // Arrow Down
                    else if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                        self.autocomplete.select_next();
                    }
                    // Escape to hide autocomplete
                    else if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        self.autocomplete.hide();
                    }
                }

                // Update cursor position when text changes
                if response.changed() {
                    if self.accepting_suggestion {
                        // When accepting, cursor position is already set correctly by accept_selected
                        // Don't trigger autocomplete immediately - wait for next character
                    } else {
                        // Normal text change: update cursor to end and trigger autocomplete
                        self.cursor_position = self.input.len();
                        self.trigger_autocomplete();
                    }
                }

                // Handle keyboard shortcuts when input has focus (and autocomplete NOT visible)
                if !self.autocomplete.is_visible() && response.has_focus() {
                    // Handle Up/Down for history when autocomplete not visible
                    if ui.input(|i| i.key_pressed(egui::Key::ArrowUp) && !i.modifiers.command) {
                        self.navigate_history_up();
                    }
                    if ui.input(|i| i.key_pressed(egui::Key::ArrowDown) && !i.modifiers.command) {
                        self.navigate_history_down();
                    }

                    // Handle Enter to submit (Shift+Enter for new line)
                    if ui.input_mut(|i| {
                        if i.key_pressed(egui::Key::Enter) && !i.modifiers.shift {
                            i.consume_key(egui::Modifiers::NONE, egui::Key::Enter);
                            true
                        } else {
                            false
                        }
                    }) {
                        self.submit_input();
                    }
                }

                // IMPORTANT: Reset the flag at the end of each frame so autocomplete works on next text change
                self.accepting_suggestion = false;

                // Render autocomplete popup if visible
                if self.autocomplete.is_visible() {
                    self.render_autocomplete_popup(ui, response.rect);
                }
            });
        });
    }

    fn render_output_item_at(&mut self, ui: &mut egui::Ui, index: usize) {
        let item = &mut self.output[index];
        match item {
            OutputItem::Text(text) => {
                // Check if this is a code line (starts with "flow>")
                if let Some(code) = text.strip_prefix("flow> ") {
                    // Render with syntax highlighting
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("flow>")
                            .family(egui::FontFamily::Monospace));
                        let font_id = egui::FontId::new(16.0, egui::FontFamily::Monospace);
                        let job = highlight_code(code, font_id, &self.color_scheme);
                        ui.label(job);
                    });
                } else {
                    ui.monospace(text.as_str());
                }
            }
            OutputItem::Error(err) => {
                // Use the rich error renderer
                renderers::render_error(ui, err);
            }
            OutputItem::Table {
                columns,
                rows,
                selected,
            } => {
                // Use the dedicated table renderer
                renderers::render_table(ui, columns, rows, selected);
            }
            OutputItem::Tree {
                root,
                expanded_paths,
            } => {
                // Use the tree renderer with expand/collapse functionality
                renderers::render_tree(ui, root, expanded_paths);
            }
            OutputItem::Markdown(md) => {
                // Use the dedicated markdown renderer
                renderers::render_markdown(ui, &mut self.markdown_cache, md);
            }
            OutputItem::Image { path, data } => {
                // Lazy load: Load the image now if not already loaded
                if data.is_none() {
                    *data = image::open(path.as_str()).ok().map(Arc::new);
                }
                // Use the dedicated image renderer
                renderers::render_image(ui, path, data);
            }
            OutputItem::Chart { chart_type, data } => {
                // Use the dedicated chart renderer
                renderers::render_chart(ui, chart_type, data);
            }
        }
    }

    fn submit_input(&mut self) {
        let input = self.input.trim().to_string();
        if input.is_empty() {
            return;
        }

        self.eval_code(input);

        // Clear input
        self.input.clear();
        self.auto_scroll = true;
    }

    /// Evaluate code from external source (e.g., editor)
    pub fn eval_code(&mut self, input: String) {
        // Add to output
        self.push_output(OutputItem::Text(format!("flow> {}", input)));

        // Add to history
        if self.history.last() != Some(&input) {
            self.history.push(input.clone());
        }
        self.history_index = None;
        self.temp_input.clear();

        // Handle special commands
        if input.starts_with(':') {
            self.handle_command(&input);
        } else {
            // Check if this is a declaration or statement (type, enum, def, let)
            let is_declaration = input.trim().starts_with("type ")
                || input.trim().starts_with("enum ")
                || input.trim().starts_with("def ")
                || input.trim().starts_with("let ");

            // Evaluate asynchronously
            let input_clone = input.clone();
            let interpreter = Arc::clone(&self.interpreter);
            let symbol_table = Arc::clone(&self.symbol_table);
            let tx = self.result_tx.clone();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let result = rt.block_on(async {
                    // This runs in block_on() within a spawned thread, so std::sync::Mutex is appropriate
                    #![allow(clippy::await_holding_lock)]
                    if is_declaration {
                        // Handle declarations and statements using resolver
                        match dsl_core::parse_program(&input_clone) {
                            Ok(mut program) => {
                                let mut interp = interpreter.lock().unwrap();

                                // Extract entry_expr early before program is moved
                                let entry_expr = program.entry_expr.take();

                                // Register types
                                for class in &program.types {
                                    interp.runtime.types.register_class(class.clone());
                                }

                                // Register enums
                                for enum_def in &program.enums {
                                    interp.runtime.types.register_enum(enum_def.clone());
                                }

                                // Rebuild BAML runtime with new types
                                if !program.types.is_empty() || !program.enums.is_empty() {
                                    if let Err(e) = interp.rebuild_runtime() {
                                        return Err(format!("Failed to rebuild runtime: {}", e));
                                    }
                                }

                                // Generate a summary message
                                let mut messages = Vec::new();
                                if !program.types.is_empty() {
                                    messages.push(format!("{} type(s) registered", program.types.len()));
                                }
                                if !program.enums.is_empty() {
                                    messages.push(format!("{} enum(s) registered", program.enums.len()));
                                }

                                // NEW: Use resolver to group functions
                                let groups = {
                                    let mut st = symbol_table.lock().unwrap();
                                    match resolve_program(program, &mut st) {
                                        Ok(g) => g,
                                        Err(e) => return Err(format!("Resolver error: {}", e)),
                                    }
                                };

                                // Compile each group
                                for group in &groups {
                                    match compile_function_group(group) {
                                        Ok((ir_func, ir_group)) => {
                                            if let Some(func) = ir_func {
                                                // Trivial function - check if we need to convert to function_group
                                                let func_name = &func.name;

                                                // Check if there's already a function or function_group with this name
                                                let has_existing_func = interp.runtime.functions.contains_key(func_name);
                                                let has_existing_group = interp.runtime.function_groups.contains_key(func_name);

                                                if has_existing_func || has_existing_group {
                                                    // Need to convert to function_group to support multiple arities
                                                    use dsl_ir::{IRFunctionClause, IRPattern, IRFunctionGroup};

                                                    // Get or create function group
                                                    let mut func_group = if let Some(existing_group) = interp.runtime.function_groups.get(func_name) {
                                                        existing_group.clone()
                                                    } else if let Some(existing_func) = interp.runtime.functions.remove(func_name) {
                                                        // Convert existing IRFunction to IRFunctionGroup
                                                        let param_patterns: Vec<IRPattern> = existing_func.params.iter()
                                                            .map(|p| IRPattern::Variable(p.clone()))
                                                            .collect();

                                                        let body = match &existing_func.execution {
                                                            dsl_ir::IRExecution::Expression { body } => *body.clone(),
                                                            _ => return Err("Cannot convert non-expression function to group".to_string()),
                                                        };

                                                        IRFunctionGroup {
                                                            name: existing_func.name.clone(),
                                                            clauses: vec![IRFunctionClause {
                                                                param_patterns,
                                                                guard: None,
                                                                body: Box::new(body),
                                                            }],
                                                            return_type: existing_func.return_type,
                                                        }
                                                    } else {
                                                        return Err(format!("Internal error: function {} not found", func_name));
                                                    };

                                                    // Add new clause from current function
                                                    let new_param_patterns: Vec<IRPattern> = func.params.iter()
                                                        .map(|p| IRPattern::Variable(p.clone()))
                                                        .collect();

                                                    let new_body = match &func.execution {
                                                        dsl_ir::IRExecution::Expression { body } => *body.clone(),
                                                        _ => return Err("Cannot convert non-expression function to group".to_string()),
                                                    };

                                                    func_group.clauses.push(IRFunctionClause {
                                                        param_patterns: new_param_patterns.clone(),
                                                        guard: None,
                                                        body: Box::new(new_body),
                                                    });

                                                    // Store as function group
                                                    interp.runtime.function_groups.insert(func_name.clone(), func_group.clone());
                                                    messages.push(format!("Defined function: {}/{}", func_name, func.params.len()));
                                                } else {
                                                    // No existing function with this name, register as simple function
                                                    interp.runtime.functions.insert(func.name.clone(), func.clone());
                                                    messages.push(format!("Defined function: {}/{}", func.name, func.params.len()));
                                                }
                                            }

                                            if let Some(func_group) = ir_group {
                                                // Pattern function
                                                if interp.runtime.function_groups.contains_key(&func_group.name) {
                                                    // Update existing
                                                    interp.runtime.function_groups.insert(func_group.name.clone(), func_group.clone());
                                                    messages.push(format!("Updated pattern function {}/{} ({} clauses)",
                                                        func_group.name,
                                                        func_group.clauses[0].param_patterns.len(),
                                                        func_group.clauses.len()));
                                                } else {
                                                    // Create new - but check if there's a simple function first
                                                    if let Some(existing_func) = interp.runtime.functions.remove(&func_group.name) {
                                                        // Convert existing IRFunction to a clause and merge
                                                        use dsl_ir::{IRFunctionClause, IRPattern};

                                                        let param_patterns: Vec<IRPattern> = existing_func.params.iter()
                                                            .map(|p| IRPattern::Variable(p.clone()))
                                                            .collect();

                                                        let body = match &existing_func.execution {
                                                            dsl_ir::IRExecution::Expression { body } => *body.clone(),
                                                            _ => return Err("Cannot convert non-expression function to group".to_string()),
                                                        };

                                                        let mut merged_group = func_group.clone();
                                                        merged_group.clauses.insert(0, IRFunctionClause {
                                                            param_patterns,
                                                            guard: None,
                                                            body: Box::new(body),
                                                        });

                                                        interp.runtime.function_groups.insert(merged_group.name.clone(), merged_group.clone());
                                                        messages.push(format!("Pattern function {}/{} defined ({} clauses)",
                                                            func_group.name,
                                                            func_group.clauses[0].param_patterns.len(),
                                                            merged_group.clauses.len()));
                                                    } else {
                                                        interp.runtime.function_groups.insert(func_group.name.clone(), func_group.clone());
                                                        messages.push(format!("Pattern function {}/{} defined ({} clause{})",
                                                            func_group.name,
                                                            func_group.clauses[0].param_patterns.len(),
                                                            func_group.clauses.len(),
                                                            if func_group.clauses.len() == 1 { "" } else { "s" }));
                                                    }
                                                }
                                            }
                                        }
                                        Err(e) => return Err(format!("Compile error: {}", e)),
                                    }
                                }

                                // If there's an entry expression (like a let statement), evaluate it
                                if let Some(entry_expr) = entry_expr {
                                    use dsl_core::compile_expr;
                                    use dsl_ir::IRBinding;

                                    let ir_node = compile_expr(&entry_expr).map_err(|e| format!("Compile error: {}", e))?;

                                    // Check for binding
                                    let binding = match &ir_node {
                                        dsl_ir::IRNode::Parallel { binding: Some(bind), .. } => Some(bind.clone()),
                                        dsl_ir::IRNode::Sequential { binding: Some(bind), .. } => Some(bind.clone()),
                                        _ => None,
                                    };

                                    // Evaluate the expression
                                    let value = interp.eval(&ir_node).await.map_err(|e| e.to_string())?;

                                    // Handle binding for let statements
                                    if let Some(binding) = binding {
                                        match binding {
                                            IRBinding::Single(name) => {
                                                if let Ok(bound_value) = interp.runtime.get_var(&name) {
                                                    interp.runtime.set_global_var(name.clone(), bound_value);
                                                    return Ok(value);
                                                }
                                            }
                                            IRBinding::List(names) => {
                                                for name in &names {
                                                    if let Ok(bound_value) = interp.runtime.get_var(name) {
                                                        interp.runtime.set_global_var(name.clone(), bound_value);
                                                    }
                                                }
                                                return Ok(value);
                                            }
                                        }
                                    }

                                    return Ok(value);
                                }

                                let summary = if messages.is_empty() {
                                    "Declaration processed".to_string()
                                } else {
                                    messages.join(", ")
                                };

                                Ok(dsl_ir::Value::String(format!("✓ {}", summary)))
                            }
                            Err(e) => Err(format!("Parse error: {}", e)),
                        }
                    } else {
                        // Handle expressions
                        match dsl_core::parse_expr(&input_clone) {
                            Ok(ast) => {
                                // Compile to IR
                                match dsl_core::compile_expr(&ast) {
                                    Ok(ir_node) => {
                                        // Evaluate
                                        let mut interp = interpreter.lock().unwrap();
                                        let result = interp.eval(&ir_node).await;
                                        match result {
                                            Ok(value) => Ok(value),
                                            Err(e) => Err(format!("Runtime error: {}", e)),
                                        }
                                    }
                                    Err(e) => Err(format!("Compile error: {}", e)),
                                }
                            }
                            Err(e) => Err(format!("Parse error: {}", e)),
                        }
                    }
                });

                let _ = tx.send(EvalResult { result });
            });
        }

        self.auto_scroll = true;
    }

    fn handle_eval_result(&mut self, result: EvalResult) {
        match result.result {
            Ok(value) => {
                // Add value to output
                self.push_output(OutputItem::from_value(&value));
            }
            Err(err) => {
                // Convert string errors to ErrorDetail
                self.push_output(OutputItem::Error(ErrorDetail::from_string(err)));
            }
        }
        self.auto_scroll = true;
    }

    fn handle_command(&mut self, cmd: &str) {
        match cmd {
            ":help" => {
                self.push_output(OutputItem::Text("Available commands:".to_string()));
                self.push_output(OutputItem::Text("  :help     - Show this help".to_string()));
                self.push_output(OutputItem::Text("  :clear    - Clear output".to_string()));
                self.push_output(OutputItem::Text(
                    "  :quit     - Quit application".to_string(),
                ));
            }
            ":clear" => {
                self.output.clear();
            }
            ":quit" | ":q" => {
                // TODO: Signal quit
                self.push_output(OutputItem::Text(
                    "Use File > Quit or Ctrl+Q to quit".to_string(),
                ));
            }
            _ => {
                self.push_output(OutputItem::Error(
                    ErrorDetail::from_string(format!("Unknown command: {}", cmd))
                ));
            }
        }
    }

    fn navigate_history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }

        match self.history_index {
            None => {
                self.temp_input = self.input.clone();
                self.history_index = Some(self.history.len() - 1);
                self.input = self.history[self.history.len() - 1].clone();
            }
            Some(idx) if idx > 0 => {
                self.history_index = Some(idx - 1);
                self.input = self.history[idx - 1].clone();
            }
            _ => {}
        }
    }

    fn navigate_history_down(&mut self) {
        match self.history_index {
            Some(idx) if idx + 1 < self.history.len() => {
                self.history_index = Some(idx + 1);
                self.input = self.history[idx + 1].clone();
            }
            Some(_) => {
                self.history_index = None;
                self.input = self.temp_input.clone();
            }
            None => {}
        }
    }

    fn trigger_autocomplete(&mut self) {
        // Update autocomplete with current runtime state
        {
            let interp = self.interpreter.lock().unwrap();
            self.autocomplete.update_with_runtime(&interp.runtime);
        }

        // Get suggestions for current input
        self.autocomplete.update(&self.input, self.cursor_position);
    }

    fn render_autocomplete_popup(&self, ui: &mut egui::Ui, input_rect: egui::Rect) {
        use dsl_autocomplete::SuggestionKind;

        // Position popup below the input field
        let popup_pos = egui::pos2(input_rect.left(), input_rect.bottom() + 5.0);

        egui::Window::new("autocomplete_popup")
            .title_bar(false)
            .resizable(false)
            .fixed_pos(popup_pos)
            .default_width(400.0)
            .interactable(false) // Prevent window from stealing focus!
            .show(ui.ctx(), |ui| {
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        for (idx, suggestion) in self.autocomplete.suggestions.iter().enumerate() {
                            let is_selected = idx == self.autocomplete.selected_index;

                            ui.horizontal(|ui| {
                                // Color code by suggestion kind
                                let color = match suggestion.kind {
                                    SuggestionKind::Keyword => {
                                        egui::Color32::from_rgb(86, 156, 214)
                                    }
                                    SuggestionKind::Function => {
                                        egui::Color32::from_rgb(220, 220, 170)
                                    }
                                    SuggestionKind::Variable => {
                                        egui::Color32::from_rgb(156, 220, 254)
                                    }
                                    SuggestionKind::Type => egui::Color32::from_rgb(78, 201, 176),
                                    SuggestionKind::Command => {
                                        egui::Color32::from_rgb(197, 134, 192)
                                    }
                                    _ => egui::Color32::GRAY,
                                };

                                let text = if is_selected {
                                    egui::RichText::new(&suggestion.label).color(color).strong()
                                } else {
                                    egui::RichText::new(&suggestion.label).color(color)
                                };

                                let _ = ui.selectable_label(is_selected, text);

                                // Show detail if available
                                if let Some(detail) = &suggestion.detail {
                                    ui.label(
                                        egui::RichText::new(detail)
                                            .small()
                                            .color(ui.visuals().weak_text_color()),
                                    );
                                }
                            });
                        }
                    });
            });
    }
}

impl Default for ReplPane {
    fn default() -> Self {
        Self::new()
    }
}
