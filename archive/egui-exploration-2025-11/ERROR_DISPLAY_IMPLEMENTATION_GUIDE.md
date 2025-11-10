# Rich Error Display - Implementation Guide

## Quick Reference: Current vs. Proposed

### Current Error Display
```
ERROR: Parse error: expected expression, found EOF
```
Just a string, red text, no context.

### Proposed Rich Error Display
```
┌─────────────────────────────────────────────────┐
│ TYPE ERROR at main.dsl:42:15                    │
├─────────────────────────────────────────────────┤
│ Cannot apply string operation to integer        │
│                                                 │
│ Expected: String                                │
│ Got:      Int                                   │
│                                                 │
│ In function: format_message                     │
├─────────────────────────────────────────────────┤
│ ▼ Source Context (click to expand)              │
│   Line 42: let result = upper(count)            │
│            ^^^^^^^^^^^^                         │
│   Line 41: let count = 42                       │
│            ^^^^^                                │
├─────────────────────────────────────────────────┤
│ ▼ Suggestions (click to expand)                 │
│   • Convert to string: str(count)               │
│   • Check variable type with typeof()           │
└─────────────────────────────────────────────────┘
```

---

## Implementation Roadmap

### STEP 1: Update Data Structures
**File:** `src/output_item.rs`

#### 1.1 Import Required Types
```rust
use dsl_ir::Span;
use dsl_interpreter::InterpreterError;
```

#### 1.2 Create Error Detail Structure
```rust
#[derive(Clone, Debug)]
pub struct ErrorDetail {
    /// Error category: "Type Error", "Runtime Error", "LLM Error", etc.
    pub error_type: String,
    
    /// Main error message
    pub message: String,
    
    /// Where error occurred (file:line:column)
    pub source_span: Option<Span>,
    
    /// Function context
    pub function_context: Option<String>,
    
    /// Error-specific details
    pub details: ErrorDetails,
    
    /// Suggestions for fixing the error
    pub suggestions: Vec<String>,
    
    /// For expandable UI sections
    pub expanded_sections: std::collections::HashSet<String>,
}

#[derive(Clone, Debug)]
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
        status_code: Option<u16>,
    },
    SQL {
        query: Option<String>,
    },
    UnknownVariable {
        name: String,
    },
    UnknownFunction {
        name: String,
    },
    Runtime,
    Other(String),
}
```

#### 1.3 Update OutputItem Enum
```rust
pub enum OutputItem {
    Text(String),
    Table { columns, rows, selected },
    Tree { root, expanded_paths },
    Error(ErrorDetail),        // <-- CHANGED from String
    Markdown(String),
    Image { path, data },
    Chart { chart_type, data },
}
```

#### 1.4 Implement Conversion from InterpreterError
```rust
impl ErrorDetail {
    pub fn from_interpreter_error(error: InterpreterError) -> Self {
        use dsl_interpreter::InterpreterError as IE;
        
        match error {
            IE::TypeError {
                message,
                expected,
                got,
                source_span,
            } => {
                Self {
                    error_type: "Type Error".to_string(),
                    message,
                    source_span,
                    function_context: None,
                    details: ErrorDetails::Type { expected, got },
                    suggestions: vec![
                        "Check the variable type using typeof()".to_string(),
                        "Use explicit type conversion if needed".to_string(),
                    ],
                    expanded_sections: std::collections::HashSet::new(),
                }
            }
            
            IE::LLMError {
                message,
                function_name,
                source_span,
                prompt,
                response,
            } => {
                Self {
                    error_type: "LLM Error".to_string(),
                    message,
                    source_span,
                    function_context: function_name,
                    details: ErrorDetails::LLM { prompt, response },
                    suggestions: vec![
                        "Check API credentials and rate limits".to_string(),
                        "Verify prompt format matches API requirements".to_string(),
                    ],
                    expanded_sections: std::collections::HashSet::new(),
                }
            }
            
            IE::HTTPError {
                message,
                function_name,
                source_span,
                method,
                url,
            } => {
                Self {
                    error_type: "HTTP Error".to_string(),
                    message,
                    source_span,
                    function_context: function_name,
                    details: ErrorDetails::HTTP {
                        method,
                        url,
                        status_code: None,
                    },
                    suggestions: vec![
                        "Check URL and network connectivity".to_string(),
                        "Verify HTTP method (GET, POST, etc.) is correct".to_string(),
                    ],
                    expanded_sections: std::collections::HashSet::new(),
                }
            }
            
            IE::SQLError {
                message,
                function_name,
                source_span,
                query,
            } => {
                Self {
                    error_type: "SQL Error".to_string(),
                    message,
                    source_span,
                    function_context: function_name,
                    details: ErrorDetails::SQL { query },
                    suggestions: vec![
                        "Check SQL syntax".to_string(),
                        "Verify table and column names exist".to_string(),
                    ],
                    expanded_sections: std::collections::HashSet::new(),
                }
            }
            
            IE::UnknownVariable { name, source_span } => {
                Self {
                    error_type: "Unknown Variable".to_string(),
                    message: format!("Variable '{}' is not defined", name),
                    source_span,
                    function_context: None,
                    details: ErrorDetails::UnknownVariable { name },
                    suggestions: vec![
                        "Define the variable with 'let'".to_string(),
                        "Check for typos in the variable name".to_string(),
                    ],
                    expanded_sections: std::collections::HashSet::new(),
                }
            }
            
            IE::UnknownFunction { name, source_span } => {
                Self {
                    error_type: "Unknown Function".to_string(),
                    message: format!("Function '{}' is not defined", name),
                    source_span,
                    function_context: None,
                    details: ErrorDetails::UnknownFunction { name },
                    suggestions: vec![
                        "Check function name spelling".to_string(),
                        "Import required module if needed".to_string(),
                        "Define function with 'def'".to_string(),
                    ],
                    expanded_sections: std::collections::HashSet::new(),
                }
            }
            
            IE::RuntimeError { message, source_span } => {
                Self {
                    error_type: "Runtime Error".to_string(),
                    message,
                    source_span,
                    function_context: None,
                    details: ErrorDetails::Runtime,
                    suggestions: vec![],
                    expanded_sections: std::collections::HashSet::new(),
                }
            }
            
            _ => {
                Self {
                    error_type: "Error".to_string(),
                    message: error.to_string(),
                    source_span: None,
                    function_context: None,
                    details: ErrorDetails::Other(error.to_string()),
                    suggestions: vec![],
                    expanded_sections: std::collections::HashSet::new(),
                }
            }
        }
    }
}
```

---

### STEP 2: Create Error Renderer
**New File:** `src/renderers/error.rs`

```rust
use eframe::egui;
use crate::output_item::{ErrorDetail, ErrorDetails};

pub fn render_error(ui: &mut egui::Ui, error: &mut ErrorDetail) {
    egui::Frame::default()
        .fill(egui::Color32::from_rgb(40, 30, 30))        // Dark red background
        .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(200, 60, 60)))
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                // Header: Error Type and Location
                render_error_header(ui, error);
                
                ui.separator();
                
                // Main message
                ui.label(egui::RichText::new(&error.message)
                    .color(egui::Color32::WHITE)
                    .size(14.0));
                
                ui.separator();
                
                // Error-specific details
                match &error.details {
                    ErrorDetails::Type { expected, got } => {
                        render_type_error_details(ui, expected, got);
                    }
                    ErrorDetails::LLM { prompt, response } => {
                        render_llm_error_details(ui, error, prompt, response);
                    }
                    ErrorDetails::HTTP { method, url, status_code } => {
                        render_http_error_details(ui, method, url, status_code);
                    }
                    ErrorDetails::SQL { query } => {
                        render_sql_error_details(ui, error, query);
                    }
                    ErrorDetails::UnknownVariable { name } => {
                        ui.horizontal(|ui| {
                            ui.label("Variable:");
                            ui.monospace(name);
                        });
                    }
                    ErrorDetails::UnknownFunction { name } => {
                        ui.horizontal(|ui| {
                            ui.label("Function:");
                            ui.monospace(name);
                        });
                    }
                    _ => {}
                }
                
                // Suggestions
                if !error.suggestions.is_empty() {
                    render_suggestions_section(ui, error);
                }
                
                // Source context
                if error.source_span.is_some() {
                    render_source_context_section(ui, error);
                }
            });
        });
}

fn render_error_header(ui: &mut egui::Ui, error: &ErrorDetail) {
    ui.horizontal(|ui| {
        // Error icon and type
        ui.colored_label(
            egui::Color32::from_rgb(255, 100, 100),
            format!("ERROR: {}", error.error_type),
        );
        
        // Location
        if let Some(span) = &error.source_span {
            ui.label(
                egui::RichText::new(format!("at {}:{}:{}", span.file, span.line, span.column))
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .small()
            );
        }
        
        // Function context
        if let Some(func) = &error.function_context {
            ui.label(
                egui::RichText::new(format!("in {}", func))
                    .color(egui::Color32::from_rgb(180, 180, 200))
                    .small()
            );
        }
    });
}

fn render_type_error_details(ui: &mut egui::Ui, expected: &str, got: &str) {
    ui.vertical(|ui| {
        ui.label("Type Mismatch:");
        
        ui.horizontal(|ui| {
            ui.label("Expected:");
            ui.monospace(
                egui::RichText::new(expected)
                    .color(egui::Color32::from_rgb(100, 200, 100))
            );
        });
        
        ui.horizontal(|ui| {
            ui.label("Got:     ");
            ui.monospace(
                egui::RichText::new(got)
                    .color(egui::Color32::from_rgb(255, 100, 100))
            );
        });
    });
}

fn render_llm_error_details(
    ui: &mut egui::Ui,
    error: &mut ErrorDetail,
    prompt: &Option<String>,
    response: &Option<String>,
) {
    if let Some(p) = prompt {
        render_expandable_section(ui, error, "Prompt", p);
    }
    if let Some(r) = response {
        render_expandable_section(ui, error, "Response", r);
    }
}

fn render_http_error_details(
    ui: &mut egui::Ui,
    method: &Option<String>,
    url: &Option<String>,
    status_code: &Option<u16>,
) {
    ui.vertical(|ui| {
        if let Some(m) = method {
            ui.horizontal(|ui| {
                ui.label("Method:");
                ui.monospace(m);
            });
        }
        if let Some(u) = url {
            ui.horizontal(|ui| {
                ui.label("URL:");
                ui.monospace(u);
            });
        }
        if let Some(code) = status_code {
            ui.horizontal(|ui| {
                ui.label("Status:");
                let color = if *code >= 400 {
                    egui::Color32::from_rgb(255, 100, 100)
                } else {
                    egui::Color32::from_rgb(100, 200, 100)
                };
                ui.colored_label(color, code.to_string());
            });
        }
    });
}

fn render_sql_error_details(ui: &mut egui::Ui, error: &mut ErrorDetail, query: &Option<String>) {
    if let Some(q) = query {
        render_expandable_section(ui, error, "Query", q);
    }
}

fn render_expandable_section(ui: &mut egui::Ui, error: &mut ErrorDetail, title: &str, content: &str) {
    let is_expanded = error.expanded_sections.contains(title);
    
    ui.horizontal(|ui| {
        let arrow = if is_expanded { "▼" } else { "▶" };
        if ui.selectable_label(false, arrow).clicked() {
            if is_expanded {
                error.expanded_sections.remove(title);
            } else {
                error.expanded_sections.insert(title.to_string());
            }
        }
        
        ui.label(egui::RichText::new(title).strong());
    });
    
    if is_expanded {
        ui.vertical(|ui| {
            ui.add_space(4.0);
            
            // Limit displayed length
            let display_text = if content.len() > 500 {
                format!("{}...", &content[..500])
            } else {
                content.to_string()
            };
            
            egui::Frame::default()
                .fill(egui::Color32::from_rgb(20, 20, 20))
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.monospace(display_text);
                });
        });
    }
}

fn render_suggestions_section(ui: &mut egui::Ui, error: &mut ErrorDetail) {
    ui.separator();
    
    ui.horizontal(|ui| {
        let arrow = if error.expanded_sections.contains("suggestions") {
            "▼"
        } else {
            "▶"
        };
        
        if ui.selectable_label(false, arrow).clicked() {
            if error.expanded_sections.contains("suggestions") {
                error.expanded_sections.remove("suggestions");
            } else {
                error.expanded_sections.insert("suggestions".to_string());
            }
        }
        
        ui.label(egui::RichText::new("Suggestions").strong().color(egui::Color32::YELLOW));
    });
    
    if error.expanded_sections.contains("suggestions") {
        ui.vertical(|ui| {
            for suggestion in &error.suggestions {
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    ui.label(format!("• {}", suggestion));
                });
            }
        });
    }
}

fn render_source_context_section(ui: &mut egui::Ui, error: &mut ErrorDetail) {
    ui.separator();
    
    ui.horizontal(|ui| {
        let arrow = if error.expanded_sections.contains("source") {
            "▼"
        } else {
            "▶"
        };
        
        if ui.selectable_label(false, arrow).clicked() {
            if error.expanded_sections.contains("source") {
                error.expanded_sections.remove("source");
            } else {
                error.expanded_sections.insert("source".to_string());
            }
        }
        
        ui.label(egui::RichText::new("Source Context").strong().color(egui::Color32::LIGHT_GRAY));
    });
    
    if error.expanded_sections.contains("source") {
        if let Some(span) = &error.source_span {
            ui.vertical(|ui| {
                ui.label(format!("File: {}", span.file));
                ui.label(format!("Line: {} | Column: {}", span.line, span.column));
                // Could load actual source file content here if available
            });
        }
    }
}
```

#### Update `src/renderers/mod.rs`
```rust
pub mod error;
pub use error::*;
```

---

### STEP 3: Update REPL Integration
**File:** `src/repl.rs`

#### 3.1 Update Error Result Type
```rust
#[derive(Debug)]
struct EvalResult {
    result: Result<dsl_ir::Value, dsl_interpreter::InterpreterError>,  // <-- Rich error type
}
```

#### 3.2 Update Error Handling in render_output_item_at
```rust
fn render_output_item_at(&mut self, ui: &mut egui::Ui, index: usize) {
    let item = &mut self.output[index];
    match item {
        // ... other variants ...
        
        OutputItem::Error(err) => {
            renderers::render_error(ui, err);
        }
    }
}
```

#### 3.3 Update eval_code Error Handling
Replace error stringification:
```rust
// OLD:
Err(e) => Err(format!("Parse error: {}", e)),

// NEW: Preserve error structure
Err(e) => Err(e),  // Let the error type through
```

#### 3.4 Update handle_eval_result
```rust
fn handle_eval_result(&mut self, result: EvalResult) {
    match result.result {
        Ok(value) => {
            self.push_output(OutputItem::from_value(&value));
        }
        Err(err) => {
            // Convert interpreter error to error detail
            self.push_output(OutputItem::Error(
                OutputItem::error_from_interpreter_error(err)
            ));
        }
    }
    self.auto_scroll = true;
}
```

---

## Step 4: Update OutputItem Implementation

**File:** `src/output_item.rs` - Add helper method

```rust
impl OutputItem {
    /// Create an error output from an interpreter error
    pub fn error_from_interpreter_error(error: dsl_interpreter::InterpreterError) -> ErrorDetail {
        ErrorDetail::from_interpreter_error(error)
    }
}
```

---

## Migration Strategy

### Phase 1: Data Structures (Non-Breaking)
1. Add new `ErrorDetail` type alongside existing error handling
2. Keep `OutputItem::Error` accepting both old strings and new structure
3. Test rendering with both types

### Phase 2: Gradual Integration
1. Start converting simple errors (unknown variable, function)
2. Add type error rendering
3. Test each error type individually

### Phase 3: Full Integration
1. Update evaluation pipeline to pass through rich errors
2. Remove string error handling
3. Clean up legacy code

### Phase 4: Enhancement
1. Add source code context loading
2. Implement error filtering
3. Add error history/replay

---

## Testing Checklist

- [ ] Parse errors display with location and suggestions
- [ ] Type mismatches show expected vs actual
- [ ] Expandable sections collapse/expand correctly
- [ ] Error details render without UI lag
- [ ] Long content is properly truncated
- [ ] Colors are readable in both light/dark themes
- [ ] Keyboard navigation works in expandable sections
- [ ] Multiple errors display cleanly
- [ ] Error stack doesn't exceed reasonable memory
- [ ] Performance acceptable with many errors

---

## Color Scheme Reference

```rust
// Error components
const ERROR_RED: Color32 = Color32::from_rgb(255, 100, 100);
const ERROR_BG: Color32 = Color32::from_rgb(40, 30, 30);
const ERROR_BORDER: Color32 = Color32::from_rgb(200, 60, 60);

// Context indicators
const EXPECTED_GREEN: Color32 = Color32::from_rgb(100, 200, 100);
const ACTUAL_RED: Color32 = Color32::from_rgb(255, 100, 100);

// Highlights
const SECTION_HEADER: Color32 = Color32::LIGHT_GRAY;
const SUGGESTION_YELLOW: Color32 = Color32::YELLOW;
const LOCATION_GRAY: Color32 = Color32::from_rgb(200, 200, 200);
```

