# Using Rich Error Display in egui

## Current Situation

**Problem:** The egui interface currently displays errors as plain red text strings, losing all the rich context we added in Phase 4.

```rust
// Current: errors get stringified and lose structure
OutputItem::Error(error_string)  // Just "LLM Error at file.dsl:42:10\n  Message..."
```

**What's Lost:**
- Source location (file, line, column) - not clickable
- Error context (prompts, URLs, queries) - buried in text
- Type information (expected vs got) - hard to parse
- Interactive possibilities - can't expand/collapse details

## What Phase 4 Gives Us

The interpreter now returns `InterpreterError` enum with rich information:

```rust
pub enum InterpreterError {
    LLMError {
        message: String,
        function_name: Option<String>,
        source_span: Option<Span>,  // file:line:column
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
    TypeError {
        message: String,
        expected: String,
        got: String,
        source_span: Option<Span>,
    },
    // ... and 5 more variants
}
```

## How to Use It in egui

### Quick Win (30 minutes): Better Text Display

**Change:** Format errors with structure instead of plain text

**Before:**
```
LLM Error at main.dsl:42:10
  Failed to call LLM: Connection timeout
  Prompt: Extract person from ...
```

**After:**
```
╔══════════════════════════════════════╗
║ LLM ERROR at main.dsl:42:10          ║
╠══════════════════════════════════════╣
║ Failed to call LLM: Connection timeout
║
║ Function: extract_user_info
║ Prompt: Extract person from ...
║ Response: (timeout)
╚══════════════════════════════════════╝
```

**Implementation:**
1. Change `OutputItem::Error(String)` to `OutputItem::Error(InterpreterError)`
2. Update error renderer to use structured formatting
3. Use colors/fonts to highlight different parts

### Medium Enhancement (2-3 hours): Clickable Locations

**Feature:** Click on "main.dsl:42:10" to jump to the error location

**Benefits:**
- Instant navigation to problem source
- Shows surrounding context
- Highlights the exact line/column

**Implementation:**
```rust
// In error display
if let Some(span) = &error.source_span {
    if ui.link(format!("{}:{}:{}", span.file, span.line, span.column)).clicked() {
        // Jump to file location
        app_state.open_file(&span.file, span.line, span.column);
    }
}
```

### Full Enhancement (1-2 days): Rich Interactive Display

**Features:**
- Collapsible sections for prompts, queries, responses
- Color-coded error types (red for type errors, orange for LLM, blue for HTTP)
- Inline suggestions
- Copy buttons for prompts/URLs/queries

**Visual Example:**

```
┌─────────────────────────────────────────┐
│ 🔴 TYPE ERROR at main.dsl:42:15         │ ← Clickable location
├─────────────────────────────────────────┤
│ Cannot apply string operation to integer│
│                                          │
│ Expected: String                         │
│ Got:      Int                            │
│                                          │
│ In function: format_message              │
├─────────────────────────────────────────┤
│ ▶ Source Context                         │ ← Click to expand
│ ▶ Suggestions                            │ ← Click to expand
└─────────────────────────────────────────┘
```

**Implementation Steps:**

1. **Update OutputItem** (src/output_item.rs):
```rust
pub enum OutputItem {
    Error(InterpreterError),  // Instead of Error(String)
    // ... rest unchanged
}
```

2. **Update Error Handling** (src/app.rs):
```rust
// Change from:
Err(e) => OutputItem::Error(e.to_string())

// To:
Err(e) => OutputItem::Error(e)  // Preserve the error structure
```

3. **Create Rich Error Renderer** (src/renderers/error.rs - NEW FILE):
```rust
pub fn render_error(ui: &mut egui::Ui, error: &InterpreterError, state: &mut ErrorState) {
    match error {
        InterpreterError::LLMError { message, source_span, prompt, response, .. } => {
            // Header with icon and location
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("🔴 LLM ERROR")
                    .color(egui::Color32::RED)
                    .strong());

                if let Some(span) = source_span {
                    if ui.link(format!("{}:{}:{}", span.file, span.line, span.column))
                        .clicked() {
                        // Jump to location
                    }
                }
            });

            // Main message
            ui.label(message);

            // Expandable prompt section
            if let Some(prompt_text) = prompt {
                egui::CollapsingHeader::new("📝 Prompt")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.monospace(prompt_text);
                        if ui.button("📋 Copy").clicked() {
                            ui.output_mut(|o| o.copied_text = prompt_text.clone());
                        }
                    });
            }

            // Expandable response section
            if let Some(resp) = response {
                egui::CollapsingHeader::new("💬 Response")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.monospace(resp);
                    });
            }
        },

        InterpreterError::TypeError { expected, got, message, source_span } => {
            // Type error specific rendering
            ui.label(egui::RichText::new("🔴 TYPE ERROR")
                .color(egui::Color32::RED)
                .strong());

            ui.label(message);

            egui::Grid::new("type_details").show(ui, |ui| {
                ui.label("Expected:");
                ui.monospace(expected);
                ui.end_row();

                ui.label("Got:");
                ui.monospace(got);
                ui.end_row();
            });
        },

        InterpreterError::HTTPError { method, url, message, .. } => {
            // HTTP error specific rendering
            ui.label(egui::RichText::new("🌐 HTTP ERROR")
                .color(egui::Color32::from_rgb(255, 140, 0))
                .strong());

            ui.label(message);

            if let Some(m) = method {
                ui.label(format!("Method: {}", m));
            }
            if let Some(u) = url {
                ui.horizontal(|ui| {
                    ui.label("URL:");
                    ui.hyperlink(u);
                });
            }
        },

        // ... handle other error types similarly
    }
}
```

4. **Update Output Renderer** (src/renderers/text.rs):
```rust
// In render_output_item function:
match item {
    OutputItem::Error(err) => {
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(40, 20, 20))  // Dark red background
            .stroke(egui::Stroke::new(1.0, egui::Color32::RED))
            .rounding(4.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                render_error(ui, err, &mut app_state.error_states[index]);
            });
    },
    // ... rest unchanged
}
```

## Implementation Priority

### Phase 1: Preserve Error Structure (30 min)
**Goal:** Stop losing error information
- Change `OutputItem::Error(String)` to `OutputItem::Error(InterpreterError)`
- Update serialization/deserialization
- Basic structured text display

### Phase 2: Better Formatting (1-2 hours)
**Goal:** Make errors easier to read
- Use frames and colors
- Grid layout for structured data
- Icons for error types

### Phase 3: Interactive Features (1-2 days)
**Goal:** Make errors actionable
- Clickable file locations
- Expandable sections
- Copy buttons
- Inline suggestions

## Benefits

1. **Faster Debugging**
   - Click to jump to error location
   - See full context without scrolling

2. **Better Error Understanding**
   - Structured display shows what went wrong clearly
   - Type mismatches are obvious with "Expected vs Got"

3. **Improved Workflow**
   - Copy LLM prompts for testing
   - Click HTTP URLs to open in browser
   - Expand/collapse to manage screen space

4. **Professional UI**
   - Looks polished and well-designed
   - Matches quality of modern IDEs

## Example: Before vs After

### Before (Current)
```
Error: LLM Error at examples/test.dsl:42:10
  Failed to parse LLM response: invalid JSON
  Prompt: Extract person from: John is 30 years old and works as a Software Engineer...
  Response: {"name": "John", "age": 30, "occupation": "Software Engineer"
```

### After (Proposed)
```
┌─────────────────────────────────────────────────┐
│ 🔴 LLM ERROR                   test.dsl:42:10 ← │ Clickable
├─────────────────────────────────────────────────┤
│ Failed to parse LLM response: invalid JSON      │
│                                                  │
│ Function: extract_user                          │
├─────────────────────────────────────────────────┤
│ ▼ Prompt                                  📋    │ Copy button
│   Extract person from: John is 30 years...     │
│                                                  │
│ ▼ Response                                      │
│   {"name": "John", "age": 30, ...              │
│                            ^ Missing closing }  │
└─────────────────────────────────────────────────┘
```

## Getting Started

See `/Users/catethos/workspace/DSL/ERROR_DISPLAY_IMPLEMENTATION_GUIDE.md` for detailed step-by-step implementation with complete code examples.

## Related Files

- **Error types:** `crates/dsl-interpreter/src/error.rs`
- **Current output:** `crates/dsl-egui/src/output_item.rs`
- **Current renderer:** `crates/dsl-egui/src/renderers/text.rs`
- **App state:** `crates/dsl-egui/src/app.rs`
