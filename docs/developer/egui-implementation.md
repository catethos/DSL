# egui Implementation Guide

**Last Updated:** 2025-11-10
**Version:** Phase 1 Complete

---

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Core Components](#core-components)
- [Renderer System](#renderer-system)
- [Output Items](#output-items)
- [Error Display](#error-display)
- [Performance Optimizations](#performance-optimizations)
- [Integration Patterns](#integration-patterns)
- [Development Workflow](#development-workflow)

---

## Overview

The DSL desktop GUI is built with **egui**, an immediate-mode GUI framework for Rust. This document provides technical details for developers working on or extending the GUI.

### Why Immediate Mode?

Immediate-mode GUIs rebuild the UI on every frame, rather than maintaining a persistent UI tree. This simplifies state management and makes the code easier to reason about.

**Key Characteristics:**
- UI code describes **what** to display, not **how** to manage state
- No explicit event handlers or callbacks
- State is external to the UI (in your application)
- UI code runs every frame (~60 FPS)

**Example:**
```rust
// This runs every frame
ui.horizontal(|ui| {
    if ui.button("Click me").clicked() {
        // Do something
    }
    ui.label(format!("Count: {}", count));
});
```

---

## Architecture

### File Structure

```
crates/dsl-egui/
├── src/
│   ├── main.rs                 # Entry point, eframe setup
│   ├── lib.rs                  # Re-exports
│   ├── app.rs                  # Main application state (~800 lines)
│   ├── repl.rs                 # REPL pane logic (~600 lines)
│   ├── editor.rs               # Editor pane logic (~400 lines)
│   ├── output_item.rs          # Output item types (~200 lines)
│   ├── autocomplete.rs         # Autocomplete UI (~250 lines)
│   ├── syntax.rs               # Syntax highlighting (~150 lines)
│   ├── formatter.rs            # Value formatting (~100 lines)
│   ├── animations.rs           # UI animations (~100 lines)
│   └── renderers/
│       ├── mod.rs              # Renderer traits
│       ├── text.rs             # Text output (~50 lines)
│       ├── table.rs            # Table renderer (~300 lines)
│       ├── tree.rs             # Tree renderer (~200 lines)
│       ├── markdown.rs         # Markdown renderer (~350 lines)
│       ├── image.rs            # Image renderer (~150 lines)
│       ├── chart.rs            # Chart renderer (~400 lines)
│       └── error.rs            # Error display (~450 lines)
└── Cargo.toml

Total: ~4,000 lines
```

---

### Application State

The main application struct holds all state:

```rust
pub struct DslApp {
    // Pane states
    repl: ReplState,
    editor: EditorState,
    active_pane: ActivePane,

    // Layout
    split_ratio: f32,           // Horizontal split (0.0 to 1.0)

    // File operations
    current_file: Option<PathBuf>,
    file_dialog: Option<FileDialog>,
    recent_files: VecDeque<PathBuf>,

    // Status bar
    status_message: Option<(String, Instant)>,
    status_color: Color32,

    // Performance
    output_limit: usize,        // Max 1000 items
    frame_count: u64,

    // Dependencies
    interpreter: Interpreter,
}
```

### Execution Flow

```
┌─────────────┐
│   eframe    │  Platform integration (windowing, events)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   DslApp    │  Main application struct
└──────┬──────┘
       │
       ├─────────────────┬─────────────────┐
       ▼                 ▼                 ▼
┌──────────┐      ┌──────────┐     ┌──────────┐
│   REPL   │      │  Editor  │     │ Renderer │
│  State   │      │  State   │     │  System  │
└──────┬───┘      └──────┬───┘     └──────┬───┘
       │                 │                 │
       ▼                 ▼                 ▼
┌──────────────────────────────────────────────┐
│           egui (immediate-mode UI)           │
└──────────────────────────────────────────────┘
```

---

## Core Components

### Component 1: ReplState

**Purpose:** Manages REPL input, output, and history

**Structure:**
```rust
pub struct ReplState {
    pub input_buffer: String,
    pub output_items: VecDeque<OutputItem>,  // Limited to 1000
    pub history: VecDeque<String>,           // Command history
    pub history_index: Option<usize>,        // Current position in history
    pub autocomplete: AutocompleteState,
    pub scroll_to_bottom: bool,              // Auto-scroll flag
}
```

**Key Methods:**
```rust
impl ReplState {
    // Add command to history
    pub fn add_to_history(&mut self, input: String);

    // Navigate history with arrow keys
    pub fn history_prev(&mut self);
    pub fn history_next(&mut self);

    // Add output item (with limit enforcement)
    pub fn add_output(&mut self, item: OutputItem);

    // Clear all output
    pub fn clear_output(&mut self);

    // Render the REPL pane
    pub fn ui(&mut self, ui: &mut egui::Ui, app: &mut DslApp);
}
```

**Rendering:**
```rust
pub fn ui(&mut self, ui: &mut egui::Ui, app: &mut DslApp) {
    ui.vertical(|ui| {
        // Output area (scrollable)
        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                for item in &self.output_items {
                    render_output_item(ui, item);
                }
            });

        // Input area (bottom)
        ui.add_space(10.0);
        let response = TextEdit::multiline(&mut self.input_buffer)
            .desired_rows(3)
            .show(ui);

        // Handle Enter key
        if response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
            self.submit_input(app);
        }
    });
}
```

---

### Component 2: EditorState

**Purpose:** Manages editor content and file operations

**Structure:**
```rust
pub struct EditorState {
    pub content: String,
    pub modified: bool,                  // Unsaved changes flag
    pub syntax_highlighter: SyntaxHighlighter,
}
```

**Key Methods:**
```rust
impl EditorState {
    // Run editor content in interpreter
    pub async fn run(&mut self, app: &mut DslApp);

    // File operations
    pub fn open_file(&mut self, path: PathBuf) -> Result<()>;
    pub fn save_file(&mut self, path: PathBuf) -> Result<()>;

    // Render the editor pane
    pub fn ui(&mut self, ui: &mut egui::Ui);
}
```

**Rendering:**
```rust
pub fn ui(&mut self, ui: &mut egui::Ui) {
    ScrollArea::vertical().show(ui, |ui| {
        let mut layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
            // Syntax highlighting with Tree-sitter
            self.syntax_highlighter.layout(ui, text, wrap_width)
        };

        let response = TextEdit::multiline(&mut self.content)
            .code_editor()
            .desired_width(f32::INFINITY)
            .layouter(&mut layouter)
            .show(ui);

        if response.changed() {
            self.modified = true;
        }
    });
}
```

---

### Component 3: Autocomplete System

**Purpose:** Provide context-aware suggestions

**Structure:**
```rust
pub struct AutocompleteState {
    pub suggestions: Vec<Completion>,
    pub selected_index: usize,
    pub popup_visible: bool,
    pub trigger_position: usize,
    pub engine: AutocompleteEngine,    // From dsl-autocomplete
}

pub struct Completion {
    pub label: String,
    pub kind: CompletionKind,           // Keyword, Function, Variable, etc.
    pub detail: Option<String>,         // Extra info
    pub insert_text: Option<String>,    // Text to insert
}
```

**Usage:**
```rust
// Update suggestions when input changes
if input_changed {
    let cursor_pos = response.cursor_range.unwrap().primary.ccursor.index;
    autocomplete.update_suggestions(&input_buffer, cursor_pos);
}

// Render popup
if autocomplete.popup_visible {
    autocomplete.show_popup(ui, &mut input_buffer);
}
```

**Popup Rendering:**
```rust
pub fn show_popup(&mut self, ui: &mut egui::Ui, input: &mut String) {
    egui::Window::new("autocomplete")
        .title_bar(false)
        .resizable(false)
        .show(ui.ctx(), |ui| {
            for (i, completion) in self.suggestions.iter().enumerate() {
                let selected = i == self.selected_index;

                let response = ui.selectable_label(
                    selected,
                    format!("{} {}", completion.icon(), completion.label)
                );

                if response.clicked() || (selected && ui.input(|i| i.key_pressed(Key::Enter))) {
                    self.apply_completion(input, completion);
                    self.popup_visible = false;
                }
            }
        });
}
```

---

## Renderer System

### Renderer Architecture

Each output type has a dedicated renderer:

```rust
pub trait Renderer {
    fn can_render(&self, item: &OutputItem) -> bool;
    fn render(&self, ui: &mut egui::Ui, item: &OutputItem);
}
```

**Dispatch:**
```rust
pub fn render_output_item(ui: &mut egui::Ui, item: &OutputItem) {
    match item {
        OutputItem::Text(text) => render_text(ui, text),
        OutputItem::Table(table) => render_table(ui, table),
        OutputItem::Tree(tree) => render_tree(ui, tree),
        OutputItem::Markdown(md) => render_markdown(ui, md),
        OutputItem::Image(img) => render_image(ui, img),
        OutputItem::Chart(chart) => render_chart(ui, chart),
        OutputItem::Error(error) => render_error(ui, error),
    }
}
```

---

### Renderer 1: Text

**File:** `src/renderers/text.rs`

**Simple plain text output:**

```rust
pub fn render_text(ui: &mut egui::Ui, text: &str) {
    ui.monospace(text);
}
```

---

### Renderer 2: Table

**File:** `src/renderers/table.rs`

**Features:**
- Automatic column sizing
- Header row styling
- Scrollable for large tables
- Max 20 rows displayed (with pagination)

**Implementation:**
```rust
pub fn render_table(ui: &mut egui::Ui, table: &TableData) {
    // Extract headers
    let headers = &table.headers;

    // Build table
    TableBuilder::new(ui)
        .striped(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .columns(Column::auto(), headers.len())
        .header(20.0, |mut header| {
            for h in headers {
                header.col(|ui| {
                    ui.strong(h);
                });
            }
        })
        .body(|mut body| {
            for row in &table.rows {
                body.row(18.0, |mut row_ui| {
                    for cell in row {
                        row_ui.col(|ui| {
                            ui.label(cell);
                        });
                    }
                });
            }
        });
}
```

**Value Detection:**
```rust
// In output_item.rs
impl From<Value> for OutputItem {
    fn from(value: Value) -> Self {
        if value.is_table() {
            // List of maps with same keys
            OutputItem::Table(value.to_table_data())
        } else {
            OutputItem::Text(value.display())
        }
    }
}
```

---

### Renderer 3: Tree

**File:** `src/renderers/tree.rs`

**Features:**
- Expandable/collapsible nodes
- Recursive rendering
- Indentation for nested structures

**Implementation:**
```rust
pub fn render_tree(ui: &mut egui::Ui, tree: &TreeNode) {
    render_tree_node(ui, tree, 0);
}

fn render_tree_node(ui: &mut egui::Ui, node: &TreeNode, depth: usize) {
    let id = ui.make_persistent_id(format!("tree_{}", node.id));

    ui.horizontal(|ui| {
        ui.add_space(depth as f32 * 20.0);  // Indent

        if !node.children.is_empty() {
            // Collapsing header for nodes with children
            egui::CollapsingHeader::new(&node.label)
                .id_source(id)
                .show(ui, |ui| {
                    for child in &node.children {
                        render_tree_node(ui, child, depth + 1);
                    }
                });
        } else {
            // Leaf node
            ui.label(&node.label);
        }
    });
}
```

---

### Renderer 4: Markdown

**File:** `src/renderers/markdown.rs`

**Features:**
- Full CommonMark support
- Syntax highlighting for code blocks
- Tables, lists, quotes
- Inline formatting (bold, italic, code)

**Implementation:**
```rust
use pulldown_cmark::{Parser, Event, Tag};

pub fn render_markdown(ui: &mut egui::Ui, markdown: &str) {
    let parser = Parser::new(markdown);

    for event in parser {
        match event {
            Event::Start(Tag::Heading(level)) => {
                ui.heading(format!("H{}", level));
            }
            Event::Start(Tag::Paragraph) => {
                ui.add_space(5.0);
            }
            Event::Code(code) => {
                ui.code(code.as_ref());
            }
            Event::Text(text) => {
                ui.label(text.as_ref());
            }
            // ... handle other events
            _ => {}
        }
    }
}
```

---

### Renderer 5: Image

**File:** `src/renderers/image.rs`

**Features:**
- Lazy loading (load on first display)
- Smart sizing (fit to pane width)
- Supported formats: PNG, JPEG, GIF, WebP

**Implementation:**
```rust
pub struct ImageData {
    path: PathBuf,
    texture: Option<TextureHandle>,   // Lazy-loaded
    size: Option<[usize; 2]>,
}

pub fn render_image(ui: &mut egui::Ui, image: &mut ImageData) {
    // Lazy load on first render
    if image.texture.is_none() {
        image.load(ui.ctx());
    }

    if let Some(texture) = &image.texture {
        // Fit to available width
        let available_width = ui.available_width();
        let aspect = texture.size()[0] as f32 / texture.size()[1] as f32;
        let height = available_width / aspect;

        ui.image(texture, [available_width, height]);
    } else {
        ui.label("Failed to load image");
    }
}
```

---

### Renderer 6: Chart

**File:** `src/renderers/chart.rs`

**Features:**
- Line charts
- Bar charts
- Scatter plots
- Automatic scaling
- Grid lines and axes

**Implementation:**
```rust
use egui_plot::{Line, Plot, PlotPoints};

pub fn render_chart(ui: &mut egui::Ui, chart: &ChartData) {
    Plot::new(chart.title)
        .view_aspect(2.0)
        .show(ui, |plot_ui| {
            match chart.chart_type {
                ChartType::Line => {
                    let points: PlotPoints = chart.data.iter()
                        .enumerate()
                        .map(|(x, y)| [x as f64, *y])
                        .collect();

                    plot_ui.line(Line::new(points));
                }
                ChartType::Bar => {
                    // Bar chart implementation
                }
                ChartType::Scatter => {
                    // Scatter plot implementation
                }
            }
        });
}
```

---

### Renderer 7: Error Display

**File:** `src/renderers/error.rs`

**Features:**
- Categorized errors (color-coded)
- Expandable sections
- Context-aware suggestions
- Function call stack
- Source location (future)

**Implementation:**
```rust
pub fn render_error(ui: &mut egui::Ui, error: &ErrorDetail) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("❌").color(Color32::RED));
        ui.strong(RichText::new(&error.error_type).color(Color32::RED));
    });

    ui.label(&error.message);

    // Suggestions (expandable)
    if let Some(suggestions) = &error.suggestions {
        ui.collapsing("💡 Suggestions", |ui| {
            for suggestion in suggestions {
                ui.label(format!("• {}", suggestion));
            }
        });
    }

    // Function context (expandable)
    if let Some(context) = &error.function_context {
        ui.collapsing("📚 Function Context", |ui| {
            ui.monospace(format!("in function: {}", context.function_name));
            if let Some(caller) = &context.called_from {
                ui.monospace(format!("called from: {}", caller));
            }
        });
    }

    // Source location (future - when span tracking is added)
    if let Some(span) = &error.source_span {
        ui.collapsing("📍 Source Location", |ui| {
            ui.monospace(format!("at {}:{}:{}", span.file, span.line, span.column));
        });
    }
}
```

---

## Output Items

### OutputItem Enum

**File:** `src/output_item.rs`

All REPL output is represented as an `OutputItem`:

```rust
#[derive(Clone)]
pub enum OutputItem {
    Text(String),
    Table(TableData),
    Tree(TreeNode),
    Markdown(String),
    Image(ImageData),
    Chart(ChartData),
    Error(ErrorDetail),
}
```

### Automatic Detection

Values are automatically converted to appropriate output items:

```rust
impl From<Value> for OutputItem {
    fn from(value: Value) -> Self {
        match value {
            // List of maps with same keys → Table
            Value::List(items) if Self::is_table(&items) => {
                OutputItem::Table(Self::to_table_data(&items))
            }

            // Markdown value → Markdown renderer
            Value::Markdown(md) => OutputItem::Markdown(md),

            // Default → Text
            _ => OutputItem::Text(value.display()),
        }
    }
}
```

### Table Detection

```rust
fn is_table(items: &[Value]) -> bool {
    if items.is_empty() {
        return false;
    }

    // All items must be maps
    if !items.iter().all(|v| matches!(v, Value::Map(_))) {
        return false;
    }

    // All maps must have same keys
    let first_keys: HashSet<_> = items[0].as_map().unwrap().keys().collect();

    items.iter().all(|item| {
        let keys: HashSet<_> = item.as_map().unwrap().keys().collect();
        keys == first_keys
    })
}
```

---

## Error Display

### ErrorDetail Structure

```rust
pub struct ErrorDetail {
    pub error_type: String,             // "UNKNOWN VARIABLE", "TYPE ERROR", etc.
    pub message: String,                // Human-readable message
    pub suggestions: Option<Vec<String>>,  // Actionable suggestions
    pub function_context: Option<FunctionContext>,
    pub source_span: Option<Span>,      // Source location (future)
    pub context: Option<ErrorContext>,  // Error-specific context
}

pub struct FunctionContext {
    pub function_name: String,
    pub called_from: Option<String>,
    pub call_stack: Vec<String>,
}

pub enum ErrorContext {
    UnknownVariable { variable_name: String },
    UnknownFunction { function_name: String },
    TypeError { expected: String, got: String },
    // ... other context types
}
```

### Smart String Parsing (Current)

Until Phase 2 (structured errors), we parse error strings:

```rust
impl ErrorDetail {
    pub fn from_string(err: String) -> Self {
        // Pattern: "Variable 'x' not found"
        if let Some(caps) = UNKNOWN_VAR_RE.captures(&err) {
            return ErrorDetail {
                error_type: "UNKNOWN VARIABLE".to_string(),
                message: err.clone(),
                context: Some(ErrorContext::UnknownVariable {
                    variable_name: caps[1].to_string(),
                }),
                suggestions: Some(vec![
                    "Define it with: let x = value".to_string(),
                    "Check variable name spelling".to_string(),
                ]),
                ..Default::default()
            };
        }

        // Similar patterns for other error types...

        // Fallback: Generic error
        ErrorDetail {
            error_type: "ERROR".to_string(),
            message: err,
            ..Default::default()
        }
    }
}
```

---

## Performance Optimizations

### 1. Output History Limit

**Problem:** Unlimited output accumulation causes slowdown

**Solution:** Limit to 1000 items

```rust
impl ReplState {
    const MAX_OUTPUT_ITEMS: usize = 1000;

    pub fn add_output(&mut self, item: OutputItem) {
        self.output_items.push_back(item);

        // Trim old items
        if self.output_items.len() > Self::MAX_OUTPUT_ITEMS {
            self.output_items.pop_front();
        }
    }
}
```

### 2. Lazy Image Loading

**Problem:** Loading all images upfront is slow

**Solution:** Load images on first display

```rust
impl ImageData {
    pub fn load(&mut self, ctx: &egui::Context) {
        if self.texture.is_none() {
            let image = image::open(&self.path).unwrap();
            let size = [image.width() as usize, image.height() as usize];
            let rgba = image.to_rgba8();
            let pixels = rgba.as_flat_samples();

            self.texture = Some(ctx.load_texture(
                "image",
                ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()),
                Default::default(),
            ));
            self.size = Some(size);
        }
    }
}
```

### 3. Visibility Culling

**Problem:** Rendering off-screen items wastes time

**Solution:** ScrollArea automatically culls non-visible items

```rust
ScrollArea::vertical()
    .auto_shrink([false; 2])
    .show(ui, |ui| {
        // egui only renders visible rows
        for item in &self.output_items {
            render_output_item(ui, item);
        }
    });
```

### 4. Request Repaint Only When Needed

**Problem:** Continuous repainting wastes CPU/battery

**Solution:** Only request repaint on changes

```rust
impl eframe::App for DslApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Don't request continuous repaint
        // ctx.request_repaint(); // ❌ DON'T DO THIS

        // egui automatically repaints when:
        // - User input (mouse, keyboard)
        // - Widget state changes
        // - Explicit ctx.request_repaint()

        // Our app only repaints when needed
    }
}
```

---

## Integration Patterns

### Pattern 1: Async Operations

**Problem:** Interpreter is async, egui is sync

**Solution:** Use tokio::spawn and channels

```rust
pub fn submit_input(&mut self, app: &mut DslApp) {
    let input = self.input_buffer.clone();
    let interpreter = app.interpreter.clone();

    // Spawn async task
    tokio::spawn(async move {
        let result = interpreter.eval(&input).await;
        // Send result back to UI thread
        tx.send(result).unwrap();
    });

    // Clear input immediately
    self.input_buffer.clear();
    self.scroll_to_bottom = true;
}

// In update loop, check for results
if let Ok(result) = self.result_receiver.try_recv() {
    self.add_output(OutputItem::from(result));
}
```

### Pattern 2: File Dialogs

**Problem:** Native file pickers block UI

**Solution:** Use rfd (non-blocking)

```rust
use rfd::AsyncFileDialog;

pub fn open_file_dialog(&mut self) {
    let tx = self.file_channel.0.clone();

    tokio::spawn(async move {
        if let Some(file) = AsyncFileDialog::new()
            .add_filter("DSL", &["dsl"])
            .pick_file()
            .await
        {
            tx.send(FileOperation::Open(file.path())).unwrap();
        }
    });
}

// In update loop
if let Ok(op) = self.file_receiver.try_recv() {
    match op {
        FileOperation::Open(path) => self.editor.open_file(path),
        FileOperation::Save(path) => self.editor.save_file(path),
    }
}
```

### Pattern 3: State Persistence

**Problem:** Want to save state across sessions

**Solution:** Use eframe's storage

```rust
impl eframe::App for DslApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        // Save REPL history
        eframe::set_value(storage, "repl_history", &self.repl.history);

        // Save recent files
        eframe::set_value(storage, "recent_files", &self.recent_files);

        // Save layout
        eframe::set_value(storage, "split_ratio", &self.split_ratio);
    }
}

impl Default for DslApp {
    fn default() -> Self {
        // Load from storage if available
        Self {
            repl: ReplState::load_from_storage(),
            // ...
        }
    }
}
```

---

## Development Workflow

### Building

```bash
# Debug build (faster compile, slower runtime)
cargo build -p dsl-egui

# Release build (slower compile, faster runtime)
cargo build --release -p dsl-egui
```

### Running

```bash
# Run debug
cargo run -p dsl-egui

# Run release
cargo run --release -p dsl-egui

# Run with logging
RUST_LOG=debug cargo run -p dsl-egui
```

### Testing

```bash
# Run unit tests
cargo test -p dsl-egui

# Run with output
cargo test -p dsl-egui -- --nocapture
```

### Adding a New Renderer

**Steps:**

1. **Define the output type in `output_item.rs`:**
   ```rust
   pub enum OutputItem {
       // ... existing variants
       NewType(NewData),
   }
   ```

2. **Create renderer file:**
   ```rust
   // src/renderers/new_type.rs
   pub fn render_new_type(ui: &mut egui::Ui, data: &NewData) {
       // Render implementation
   }
   ```

3. **Add to dispatcher:**
   ```rust
   // In src/renderers/mod.rs
   pub fn render_output_item(ui: &mut egui::Ui, item: &OutputItem) {
       match item {
           // ... existing cases
           OutputItem::NewType(data) => render_new_type(ui, data),
       }
   }
   ```

4. **Add conversion logic:**
   ```rust
   // In output_item.rs
   impl From<Value> for OutputItem {
       fn from(value: Value) -> Self {
           // Add detection logic
       }
   }
   ```

### Debugging UI

**Enable egui inspector:**

```rust
ui.ctx().set_debug_on_hover(true);  // Show widget IDs on hover
```

**View frame stats:**

```rust
ui.ctx().set_visuals(Visuals::dark());
ui.ctx().set_debug_on_hover(true);

// In update()
egui::Window::new("Debug").show(ctx, |ui| {
    ui.label(format!("FPS: {:.1}", 1.0 / ctx.input(|i| i.unstable_dt)));
    ui.label(format!("Widgets: {}", ctx.used_ids().len()));
});
```

---

## Related Documentation

- [Architecture](architecture.md) - Overall system architecture
- [Error Handling](error-handling.md) - Error types and display
- [User Guide](../gui/egui-desktop-gui.md) - User-facing documentation

---

## External Resources

- [egui Documentation](https://docs.rs/egui/)
- [egui GitHub](https://github.com/emilk/egui)
- [egui Demo](https://www.egui.rs/) - Interactive examples
- [eframe Documentation](https://docs.rs/eframe/) - Application framework

---

**Created:** 2025-11-10
**Status:** Living document - updated as GUI evolves
