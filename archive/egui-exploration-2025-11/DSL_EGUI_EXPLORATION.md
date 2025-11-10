# DSL egui Crate - Error Display and UI Integration Analysis

## Overview
The dsl-egui crate is a desktop GUI application for the DSL language built with egui. It provides an interactive REPL and code editor with rich output rendering capabilities.

## Directory Structure
```
crates/dsl-egui/
├── src/
│   ├── main.rs                 # Application entry point
│   ├── lib.rs                  # Library exports
│   ├── app.rs                  # Main application state and UI layout
│   ├── repl.rs                 # REPL pane with input/output
│   ├── editor.rs               # Code editor pane with preview
│   ├── autocomplete.rs         # Autocomplete engine integration
│   ├── syntax.rs               # Syntax highlighting (Tree-sitter)
│   ├── animations.rs           # Animation utilities
│   ├── output_item.rs          # Output item types and conversions
│   └── renderers/              # Output rendering modules
│       ├── mod.rs
│       ├── text.rs
│       ├── table.rs            # Tabular data display
│       ├── tree.rs             # Hierarchical data display
│       ├── markdown.rs         # Markdown rendering
│       ├── image.rs            # Image display
│       └── chart.rs            # Chart/visualization rendering
├── Cargo.toml
├── README.md
├── QUICKSTART.md
└── MIGRATION_COMPLETE.md
```

---

## 1. CURRENT ERROR DISPLAY IMPLEMENTATION

### 1.1 Error Output Item Type
**File:** `src/output_item.rs`

```rust
pub enum OutputItem {
    Text(String),
    Table { columns, rows, selected },
    Tree { root, expanded_paths },
    Error(String),              // <-- Error display
    Markdown(String),
    Image { path, data },
    Chart { chart_type, data },
}
```

**Current Limitation:** Errors are stored as plain strings with NO:
- Source location information (line, column, file)
- Error type/category
- Context or suggestions
- Multi-part error details (expected vs. actual)

### 1.2 Error Rendering
**File:** `src/repl.rs` (lines 408-410)

```rust
OutputItem::Error(err) => {
    ui.colored_label(egui::Color32::RED, format!("Error: {}", err));
}
```

**Current Implementation:**
- Simple red label with error message string
- No formatting or structure
- No interactive elements
- No ability to click/expand for more details

### 1.3 Error Generation Points
**File:** `src/repl.rs`

1. **Line 656:** Evaluation result errors
   ```rust
   Err(err) => {
       self.push_output(OutputItem::Error(err));
   }
   ```

2. **Line 682:** Unknown command errors
   ```rust
   self.push_output(OutputItem::Error(format!("Unknown command: {}", cmd)));
   ```

3. **Line 617:** Parse errors
   ```rust
   Err(e) => Err(format!("Parse error: {}", e)),
   ```

4. **Line 631:** Runtime errors
   ```rust
   Err(e) => Err(format!("Runtime error: {}", e)),
   ```

---

## 2. INTERPRETER ERROR STRUCTURE

### 2.1 Rich Error Types Available
**File:** `crates/dsl-interpreter/src/error.rs`

The interpreter provides a comprehensive error type with multiple variants:

```rust
pub enum InterpreterError {
    LLMError {
        message: String,
        function_name: Option<String>,
        source_span: Option<Span>,
        prompt: Option<String>,
        response: Option<String>,
    },
    
    HTTPError {
        message: String,
        function_name: Option<String>,
        source_span: Option<Span>,
        method: Option<String>,
        url: Option<String>,
    },
    
    SQLError {
        message: String,
        function_name: Option<String>,
        source_span: Option<Span>,
        query: Option<String>,
    },
    
    TypeError {
        message: String,
        expected: String,
        got: String,
        source_span: Option<Span>,
    },
    
    RuntimeError {
        message: String,
        source_span: Option<Span>,
    },
    
    UnknownVariable {
        name: String,
        source_span: Option<Span>,
    },
    
    UnknownFunction {
        name: String,
        source_span: Option<Span>,
    },
    
    UnknownIntrinsic {
        name: String,
        source_span: Option<Span>,
    },
    
    InvalidArguments {
        message: String,
        source_span: Option<Span>,
    },
}
```

### 2.2 Span Information
**File:** `crates/dsl-ir/src/ir.rs` (lines 18-24)

```rust
pub struct Span {
    pub file: String,
    pub line: usize,
    pub column: usize,
}
```

Contains source location information for precise error reporting.

### 2.3 Display Implementation
The `InterpreterError` implements `Display` with detailed formatting:
- Error type header (e.g., "LLM Error")
- Function name context
- Source span (file:line:column)
- Message body
- Additional context (method, URL, query, prompt, response)
- Truncation for long values (200 char limit)

---

## 3. INTERPRETER INTEGRATION

### 3.1 Evaluation Flow
**File:** `src/repl.rs` (lines 458-647)

The evaluation happens in a spawned thread with these stages:

1. **Parse** → Returns string error on parse failure
2. **Compile** → Returns string error on compile failure
3. **Evaluate** → Returns `dsl_ir::Value` or string error
4. **Result transmission** → Via mpsc channel

**Current Problem:** All errors are converted to strings:
```rust
Err(e) => Err(format!("Parse error: {}", e)),
```

The rich error information is being stringified and lost!

### 3.2 Interpreter Access
```rust
interpreter: Arc<Mutex<dsl_interpreter::Interpreter>>,
```

The interpreter is accessible and could provide more context.

---

## 4. OUTPUT RENDERING SYSTEM

### 4.1 Renderer Architecture
**File:** `src/renderers/mod.rs`

Public rendering functions:
- `render_table()` - Tabular data with columns and rows
- `render_tree()` - Hierarchical data with expand/collapse
- `render_markdown()` - Markdown formatted text
- `render_image()` - Image display with lazy loading
- `render_chart()` - Charts (Bar, Line, Scatter, Pie)

### 4.2 Tree Renderer (Most Sophisticated)
**File:** `src/renderers/tree.rs`

Features:
- Expand/collapse functionality
- Indentation and visual hierarchy
- Color-coded keys and values
- Maintains expand/collapse state per node
- Recursive rendering with depth tracking

This could serve as a model for error display.

### 4.3 Markdown Renderer
**File:** `src/renderers/markdown.rs`

Uses `pulldown-cmark` to parse and render:
- Headings (H1-H6 with size/color)
- Bold, italic, code formatting
- Lists (ordered/unordered)
- Code blocks with dark background
- Separators

This provides good UX for structured text display.

### 4.4 Image and Chart Renderers
- **Image:** Lazy loading, smart sizing, aspect ratio preservation
- **Chart:** Extracts data from various formats (lists, maps, pairs)

---

## 5. UI COMPONENT ARCHITECTURE

### 5.1 Main Application Layout
**File:** `src/app.rs`

```
┌─────────────────────────────────┐
│        Menu Bar                 │
├────────────────────┬────────────┤
│                    │            │
│   REPL Pane        │            │
│  (left, 50%)       │   Editor   │
│                    │   Pane     │
│                    │ (right,    │
│                    │   50%)     │
├────────────────────┴────────────┤
│      Status Bar with Messages   │
└─────────────────────────────────┘
```

Features:
- Draggable splitter between panes
- Keyboard shortcuts (Shift+Tab to switch)
- Status messages (colored, 3-second auto-dismiss)
- File operations (open, save, recent files)

### 5.2 REPL Pane
**File:** `src/repl.rs`

```
┌──────────────────────────────────┐
│     Output Area (ScrollArea)     │
│  ┌──────────────────────────────┐│
│  │ Animation header (optional)  ││
│  │ - Output Item 1              ││
│  │ - Output Item 2              ││
│  │ - Output Item N              ││
│  └──────────────────────────────┘│
├──────────────────────────────────┤
│ flow> [Input TextEdit]           │
│       [Autocomplete popup]       │
└──────────────────────────────────┘
```

Features:
- ScrollArea with auto-scroll-to-bottom
- Output culling for performance (>30 items)
- Autocomplete with popup menu
- Tab autocomplete, arrow key navigation
- History with Up/Down arrows
- Multiline input support

### 5.3 Editor Pane
**File:** `src/editor.rs`

Features:
- Syntax highlighting preview (optional)
- File path display with modified indicator
- Status messages
- Save/load functionality

---

## 6. EXISTING ERROR HANDLING PATTERNS

### 6.1 Status Messages
**File:** `src/app.rs` (lines 163-194)

```rust
pub struct DslApp {
    status_message: Option<(String, f64, StatusKind)>,
}

enum StatusKind {
    Success,
    Error,
}
```

Used for:
- File save/load feedback
- Operation results
- Auto-dismiss after 3 seconds

**This pattern could be extended for inline errors!**

### 6.2 Colored Labels
egui provides:
```rust
ui.colored_label(Color32, text);
ui.label(RichText::new(text).color(...).strong().italics());
```

These are used throughout for highlighting (errors, warnings, success).

### 6.3 Frame Containers
egui provides frame/panel containers for grouping:
```rust
egui::Frame::default()
    .fill(Color32)
    .stroke(Stroke)
    .inner_margin(8.0)
    .show(ui, |ui| { ... })
```

Used for visual separation in tree/table renderers.

---

## 7. OPPORTUNITIES FOR RICH ERROR DISPLAY

### 7.1 Error Item Type Structure

**Proposed Enhancement:**

```rust
pub enum OutputItem {
    // ... existing variants
    
    Error(ErrorDetail),  // Changed from String
}

pub struct ErrorDetail {
    /// Error type/category (e.g., "Type Error", "Runtime Error")
    pub error_type: String,
    
    /// Main error message
    pub message: String,
    
    /// Source location (file:line:column)
    pub source_span: Option<Span>,
    
    /// Function context where error occurred
    pub function_context: Option<String>,
    
    /// Error-specific details (type-dependent)
    pub details: ErrorDetails,
    
    /// Suggestions or fixes
    pub suggestions: Vec<String>,
}

pub enum ErrorDetails {
    Type {
        expected: String,
        got: String,
    },
    LLM {
        prompt: Option<String>,
        response: Option<String>,
    },
    HTTP {
        method: Option<String>,
        url: Option<String>,
    },
    SQL {
        query: Option<String>,
    },
    Runtime,
    Other(String),
}
```

### 7.2 Error Rendering Strategy

**Multi-part rendering with expandable sections:**

```
┌─────────────────────────────────────┐
│ ERROR: Type Mismatch at main.dsl:15 │
├─────────────────────────────────────┤
│ Expected: String, Got: Int          │
│ In function: format_output          │
├─────────────────────────────────────┤
│ Message:                            │
│ Cannot apply string function to     │
│ integer value                       │
├─────────────────────────────────────┤
│ ▼ Source Context (expandable)       │
│   Line 15: format_output(value)     │
│           ^^^^^^^^^^^^^^^^^         │
├─────────────────────────────────────┤
│ ▼ Suggestions (expandable)          │
│   - Cast value to string: str(val)  │
│   - Check value type first          │
└─────────────────────────────────────┘
```

### 7.3 Rendering Implementation Points

1. **Location Display**
   - Add source span rendering: "at file:line:column"
   - Highlight the problematic line with visual indicator
   - Could use syntax highlighting for code snippets

2. **Type Information**
   - Show expected vs. got in structured format
   - Use color coding (expected=green, got=red)

3. **Context Stack**
   - Function name
   - Call stack for nested errors
   - Variable/expression context

4. **Additional Details**
   - LLM: Truncated prompt/response preview
   - HTTP: Method, URL, status code
   - SQL: Query preview with syntax highlighting
   - Pattern: Expected vs actual values

5. **Expandable Sections**
   - Use tree-like expand/collapse pattern
   - Store state in output item
   - Lazy render detailed content

### 7.4 Integration Points

**In `src/output_item.rs`:**
- Add error conversion from `InterpreterError`
- Implement error detail extraction
- Structure error information for UI display

**In `src/repl.rs`:**
- Preserve `InterpreterError` type instead of stringifying
- Convert to `ErrorDetail` when pushing to output
- Handle error result channel properly

**In new `src/renderers/error.rs`:**
- Implement sophisticated error rendering
- Support expand/collapse for sections
- Color-code different error components
- Format source code context

---

## 8. SYNTAX HIGHLIGHTING INTEGRATION

**File:** `src/syntax.rs`

Features:
- Tree-sitter-based syntax highlighting
- Color schemes (dark/light)
- Integrates with egui LayoutJob system
- Used in editor preview and REPL input display

**For error rendering:**
- Could highlight source code snippets in errors
- Show syntax-highlighted queries (SQL, etc.)
- Visualize error location with color

---

## 9. PERFORMANCE CONSIDERATIONS

### 9.1 Output Culling
The REPL already implements visibility-based culling:
- Items > 30: Use distance-based culling
- Items <= 30: Render all (smoothness)
- Large margin for scroll buffer (1500px)

**This would apply to complex error renders too.**

### 9.2 Lazy Loading Pattern
Images use lazy loading:
```rust
if data.is_none() {
    *data = image::open(path.as_str()).ok().map(Arc::new);
}
```

**Could expand error details lazily.**

---

## 10. KEY FILES SUMMARY

| File | Lines | Purpose |
|------|-------|---------|
| `app.rs` | 490 | Main UI layout, pane management |
| `repl.rs` | 797 | REPL UI, evaluation, output |
| `output_item.rs` | 263 | Output type definitions |
| `editor.rs` | 184 | Code editor UI |
| `renderers/tree.rs` | 109 | Tree/hierarchical rendering |
| `renderers/markdown.rs` | 256 | Markdown rendering |
| `renderers/table.rs` | 104 | Table rendering |
| `renderers/chart.rs` | 274 | Chart rendering |
| `syntax.rs` | 269 | Syntax highlighting |
| `autocomplete.rs` | 150+ | Autocomplete integration |

---

## 11. RECOMMENDATIONS FOR RICH ERROR DISPLAY

### Phase 1: Data Structures
1. Create `ErrorDetail` struct with complete error information
2. Update `OutputItem::Error` to use `ErrorDetail`
3. Implement `From<InterpreterError>` for conversion

### Phase 2: Rendering
1. Create `renderers/error.rs` with error rendering logic
2. Implement expandable sections using tree pattern
3. Add color-coding for different error components
4. Support source context display

### Phase 3: Integration
1. Modify REPL evaluation to preserve error types
2. Update error result channel to carry rich errors
3. Integrate renderer into REPL output display

### Phase 4: Enhancement
1. Add syntax highlighting to code snippets
2. Implement error suggestions/quick fixes UI
3. Add error filtering/search in output
4. Support error history/traceback

---

## 12. BENEFITS OF RICH ERROR DISPLAY

1. **Better User Experience**
   - Users see exactly what went wrong
   - Location information helps debugging
   - Suggestions guide problem resolution

2. **Interactive Learning**
   - Expandable details prevent cognitive overload
   - Progressive disclosure of information
   - Context helps understanding

3. **Production Ready**
   - Professional error handling
   - Complete error information for debugging
   - Matches IDE/compiler standards

4. **Development Support**
   - Stack traces for complex errors
   - Error categorization enables filtering
   - Preserves error context for analysis

