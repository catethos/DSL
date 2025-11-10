# Error Handling Architecture

**Last Updated:** 2025-11-10
**Status:** Phase 1 Complete, Phase 2 Planned

---

## Table of Contents

- [Overview](#overview)
- [InterpreterError Types](#interpretererror-types)
- [Current Implementation](#current-implementation)
- [Error Display Integration](#error-display-integration)
- [Planned Enhancements](#planned-enhancements)
- [Best Practices](#best-practices)

---

## Overview

The DSL uses a **structured error system** with rich `InterpreterError` types that provide detailed context for debugging. Errors are displayed in the egui GUI with expandable sections showing function context, suggestions, and error-specific information.

### Architecture Goals

1. **Type-Safe Errors:** Compile-time checking of error handling
2. **Rich Context:** Error-specific fields (variable name, function name, etc.)
3. **Better UX:** Categorized errors with actionable suggestions
4. **Span Tracking:** Source locations for every error (planned)
5. **No String Parsing:** Direct access to error information

---

## InterpreterError Types

The `InterpreterError` enum has **9 variants** covering all error categories:

### 1. UnknownVariable

**When:** Variable not found in current scope

```rust
InterpreterError::UnknownVariable {
    name: String,           // Variable name that was referenced
    source_span: Option<Span>,  // Source location (future)
}
```

**Example:**
```javascript
x + 5  // Error: Variable 'x' not found
```

**Display:**
```
❌ UNKNOWN VARIABLE

Variable 'x' not found

💡 Suggestions:
- Define it with: let x = value
- Check spelling
```

---

### 2. UnknownFunction

**When:** Function not found (neither builtin nor user-defined)

```rust
InterpreterError::UnknownFunction {
    name: String,           // Function name
    source_span: Option<Span>,
}
```

**Example:**
```javascript
unknownFunc()  // Error: Unknown function 'unknownFunc'
```

**Display:**
```
❌ UNKNOWN FUNCTION

Function 'unknownFunc' not found

💡 Suggestions:
- Check function name spelling
- Define it with: function name() { ... }
- Available builtins: upper, lower, length, ...
```

---

### 3. TypeError

**When:** Type mismatch in operations

```rust
InterpreterError::TypeError {
    message: String,        // Human-readable message
    expected: String,       // Expected type
    got: String,            // Actual type
    source_span: Option<Span>,
}
```

**Example:**
```javascript
"hello".age  // Error: Cannot access field on String
```

**Display:**
```
❌ TYPE ERROR

Cannot access field on String

Expected: Map or Object
Got: String

💡 Suggestions:
- Use a Map or type instance instead
- Check that variable contains expected type
```

---

### 4. RuntimeError

**When:** General runtime failures (division by zero, etc.)

```rust
InterpreterError::RuntimeError {
    message: String,        // Error description
    source_span: Option<Span>,
}
```

**Example:**
```javascript
100 / 0  // Error: Division by zero
```

**Display:**
```
❌ RUNTIME ERROR

Division by zero

💡 Suggestions:
- Check divisor is not zero before division
- Use conditional: if divisor != 0 { ... }
```

---

### 5. InvalidArguments

**When:** Wrong number or type of arguments

```rust
InterpreterError::InvalidArguments {
    message: String,        // Details about mismatch
    source_span: Option<Span>,
}
```

**Example:**
```javascript
upper()  // Error: upper() requires 1 argument, got 0
```

**Display:**
```
❌ INVALID ARGUMENTS

upper() requires 1 argument, got 0

💡 Suggestions:
- Check function signature
- Provide required arguments
```

---

### 6. PatternMatchError

**When:** Pattern matching fails or patterns don't match

```rust
InterpreterError::PatternMatchError {
    message: String,        // Match failure details
    source_span: Option<Span>,
}
```

**Example:**
```javascript
match x {
  1 => "one"
  2 => "two"
}  // Error: No pattern matched value 3
```

**Display:**
```
❌ PATTERN MATCH ERROR

No pattern matched value 3

💡 Suggestions:
- Add wildcard pattern: _ => "default"
- Add more specific patterns
- Check match coverage
```

---

### 7. LLMError

**When:** LLM API call fails

```rust
InterpreterError::LLMError {
    message: String,        // Error from LLM provider
    provider: String,       // Provider name (OpenAI, etc.)
    model: String,          // Model name
    source_span: Option<Span>,
}
```

**Example:**
```javascript
Ask("Generate report")  // Error: API key not set
```

**Display:**
```
❌ LLM ERROR

Provider: OpenAI
Model: gpt-4

API key not set

💡 Suggestions:
- Set OPENAI_API_KEY environment variable
- Check API key is valid
- Verify model name is correct
```

---

### 8. HTTPError

**When:** HTTP request fails

```rust
InterpreterError::HTTPError {
    message: String,        // Error description
    url: String,            // Request URL
    status: Option<u16>,    // HTTP status code
    source_span: Option<Span>,
}
```

**Example:**
```javascript
GET("https://api.example.com/data")  // Error: 404 Not Found
```

**Display:**
```
❌ HTTP ERROR

URL: https://api.example.com/data
Status: 404 Not Found

Resource not found

💡 Suggestions:
- Check URL is correct
- Verify endpoint exists
- Check network connectivity
```

---

### 9. SQLError

**When:** SQL query fails

```rust
InterpreterError::SQLError {
    message: String,        // Error from DuckDB
    query: String,          // SQL query that failed
    source_span: Option<Span>,
}
```

**Example:**
```javascript
SQL("SELECT * FROM nonexistent")  // Error: Table not found
```

**Display:**
```
❌ SQL ERROR

Query: SELECT * FROM nonexistent

Table 'nonexistent' not found

💡 Suggestions:
- Check table name spelling
- List available tables with SHOW TABLES
- Create table first
```

---

## Current Implementation

### Phase 1: Basic Error Types ✅ COMPLETE

**Status:** Implemented and working

**What's Done:**
- ✅ All 9 error variants defined
- ✅ Display trait for human-readable output
- ✅ Error categorization (UNKNOWN VARIABLE, TYPE ERROR, etc.)
- ✅ Smart pattern recognition in error strings
- ✅ Context-aware suggestions
- ✅ Expandable error display in egui

**Files:**
- `crates/dsl-interpreter/src/error.rs` - Error type definitions
- `crates/dsl-egui/src/renderers/error.rs` - Error display UI

---

### How Errors Work Currently

#### 1. Error Creation

Errors are created throughout the interpreter:

```rust
// In interpreter.rs
pub async fn eval(&mut self, node: &IRNode) -> Result<Value, String> {
    match node {
        IRNode::Variable(name) => {
            self.runtime.get_var(name)
                .map_err(|_| format!("Variable '{}' not found", name))?
        }
        // ...
    }
}
```

**Note:** Currently returns `Result<Value, String>`. Phase 2 will change to `Result<Value, InterpreterError>`.

#### 2. Error Display in egui

When an error occurs, it's converted to `ErrorDetail`:

```rust
// In egui/src/repl.rs
let result = interpreter.eval(&ir_node).await;
match result {
    Ok(value) => OutputItem::Value(value),
    Err(err_string) => {
        let error_detail = ErrorDetail::from_string(err_string);
        OutputItem::Error(error_detail)
    }
}
```

#### 3. String Parsing (Current Workaround)

`ErrorDetail::from_string()` uses pattern matching to extract context:

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
                // ...
            };
        }

        // Similar patterns for other error types...
    }
}
```

**Limitation:** Requires parsing error strings, fragile if format changes.

#### 4. Rich Display

Errors display in egui with expandable sections:

```rust
// In egui/src/renderers/error.rs
pub fn render_error(ui: &mut egui::Ui, error: &ErrorDetail) {
    ui.horizontal(|ui| {
        ui.label("❌");
        ui.label(error.error_type);  // "UNKNOWN VARIABLE"
    });

    ui.label(&error.message);

    // Show suggestions
    if let Some(suggestions) = &error.suggestions {
        ui.collapsing("💡 Suggestions", |ui| {
            for suggestion in suggestions {
                ui.label(format!("• {}", suggestion));
            }
        });
    }

    // Show context (function stack, etc.)
    if let Some(context) = &error.context {
        render_context(ui, context);
    }
}
```

---

## Error Display Integration

### egui Error Renderer

**Location:** `crates/dsl-egui/src/renderers/error.rs`

**Features:**
- ✅ Expandable error sections
- ✅ Syntax-highlighted error messages
- ✅ Context-aware suggestions
- ✅ Function call stack (when available)
- ✅ Source code snippets (future)

**Layout:**

```
┌─────────────────────────────────────────┐
│ ❌ UNKNOWN VARIABLE                     │
├─────────────────────────────────────────┤
│ Variable 'x' not found                  │
│                                         │
│ ▼ 💡 Suggestions                       │
│   • Define it with: let x = value      │
│   • Check spelling                     │
│                                         │
│ ▼ 📍 Source Location (future)          │
│   at script.dsl:5:3                    │
│                                         │
│ ▼ 📚 Function Context                  │
│   in function: calculate               │
│   called from: main                    │
└─────────────────────────────────────────┘
```

### TUI Error Display

**Location:** `crates/dsl-tui/src/app.rs`

Simple text-based display:

```
Error: Variable 'x' not found

Suggestions:
- Define it with: let x = value
- Check spelling
```

---

## Planned Enhancements

### Phase 2: Span Tracking (Planned)

**Goal:** Add source location to every error

**Status:** Design complete, not implemented

**Required Changes:**

#### 2.1 Update Return Types

Change from `Result<Value, String>` to `Result<Value, InterpreterError>`:

```rust
// OLD:
pub async fn eval(&mut self, node: &IRNode) -> Result<Value, String>

// NEW:
pub async fn eval(&mut self, node: &IRNode) -> Result<Value, InterpreterError>
```

**Files to Update:**
- `crates/dsl-interpreter/src/interpreter.rs` - Main eval loop
- `crates/dsl-interpreter/src/runtime.rs` - Runtime methods
- `crates/dsl-interpreter/src/builtins.rs` - Builtin functions

#### 2.2 Convert Error Sites

Replace string errors with structured errors:

```rust
// OLD:
Err(format!("Variable '{}' not found", name))

// NEW:
Err(InterpreterError::UnknownVariable {
    name: name.to_string(),
    source_span: None,  // TODO: Add span
})
```

#### 2.3 Add Span Tracking

**Option A:** Add spans to IRNode (breaking change)

```rust
pub enum IRNode {
    Variable {
        name: String,
        span: Option<Span>,  // NEW
    },
    // ... all variants get span field
}
```

**Option B:** Separate span table (recommended)

```rust
pub struct Interpreter {
    runtime: Runtime,
    span_table: HashMap<NodeId, Span>,  // Map node to span
}
```

#### 2.4 Preserve Spans Through Compilation

Update compiler to track spans:

```rust
// In compiler.rs
fn compile_expr(ast: &Expr) -> (IRNode, Option<Span>) {
    let span = ast.span.clone();
    let ir_node = /* compile logic */;
    (ir_node, span)
}
```

#### 2.5 Thread Spans Through Evaluation

Pass spans to error creation:

```rust
pub async fn eval(&mut self, node: &IRNode, span: Option<Span>)
    -> Result<Value, InterpreterError>
{
    match node {
        IRNode::Variable(name) => {
            self.runtime.get_var(name).map_err(|mut e| {
                e.source_span = span.clone();
                e
            })
        }
    }
}
```

**Estimated Time:** 4-6 hours

**Benefit:** Errors show exact source location (file:line:column)

---

### Phase 3: Clickable Locations (Future)

**Goal:** Click error location to jump to source

**Requirements:**
- Phase 2 complete (span tracking)
- egui file viewer integration
- Source file caching

**UI Mockup:**

```
┌─────────────────────────────────────────┐
│ ❌ TYPE ERROR                           │
├─────────────────────────────────────────┤
│ Cannot access field on String           │
│                                         │
│ ▼ 📍 Source Location                   │
│   📄 script.dsl:15:8 [View]            │  ← Clickable
│                                         │
│   13 | let name = "Alice"              │
│   14 | let age = 25                    │
│   15 | let greeting = name.upper       │  ← Error here
│        ──────────────^                  │
│   16 | greeting                        │
└─────────────────────────────────────────┘
```

---

### Phase 4: Stack Traces (Future)

**Goal:** Show full call stack for errors

**Example:**

```
❌ DIVISION BY ZERO

at divide (script.dsl:8:12)
  in calculate (script.dsl:12:5)
  in main (script.dsl:15:1)

Stack trace:
  1. main()
  2.  └─ calculate(10, 0)
  3.      └─ divide(10, 0)  ← error here
```

---

## Best Practices

### For Interpreter Developers

#### 1. Return Specific Error Types

Don't use generic RuntimeError when a specific type exists:

```rust
// BAD:
Err(InterpreterError::RuntimeError {
    message: "Variable not found".to_string(),
    source_span: None,
})

// GOOD:
Err(InterpreterError::UnknownVariable {
    name: var_name.to_string(),
    source_span: None,
})
```

#### 2. Include Context

Provide as much context as possible:

```rust
// BAD:
Err(InterpreterError::TypeError {
    message: "Type error".to_string(),
    expected: "".to_string(),
    got: "".to_string(),
    source_span: None,
})

// GOOD:
Err(InterpreterError::TypeError {
    message: format!("Cannot access field '{}' on {}", field, value.type_name()),
    expected: "Map or Object".to_string(),
    got: value.type_name(),
    source_span: None,
})
```

#### 3. Add Suggestions When Possible

Help users fix errors:

```rust
let suggestions = vec![
    "Define the variable with: let x = value".to_string(),
    "Check variable name spelling".to_string(),
    "Use :vars to see defined variables".to_string(),
];
```

#### 4. Preserve Error Chain

Use `.map_err()` to add context:

```rust
some_operation()
    .map_err(|e| InterpreterError::RuntimeError {
        message: format!("Failed to do X: {}", e),
        source_span: None,
    })?
```

---

### For Error Display Developers

#### 1. Make Errors Scannable

Use consistent formatting:
- ❌ Red for error type
- 💡 Lightbulb for suggestions
- 📍 Pin for source location
- 📚 Books for function context

#### 2. Provide Actionable Suggestions

Bad: "Fix the error"
Good: "Define it with: let x = value"

#### 3. Use Expandable Sections

Don't overwhelm users:
- Show critical info by default (error type, message)
- Hide details in collapsible sections (suggestions, stack trace)

#### 4. Syntax Highlight Code

When showing code snippets:
- Use syntax highlighting
- Underline error location
- Show surrounding context (±2 lines)

---

## Testing Error Handling

### Manual Testing

Test each error type:

```bash
# 1. Unknown Variable
echo "unknown_var" | cargo run --bin dsl-tui

# 2. Unknown Function
echo "unknownFunc()" | cargo run --bin dsl-tui

# 3. Type Error
echo '"hello".age' | cargo run --bin dsl-tui

# 4. Division by Zero
echo "100 / 0" | cargo run --bin dsl-tui

# 5. Invalid Arguments
echo "upper()" | cargo run --bin dsl-tui

# 6. Pattern Match Error
echo 'match 3 { 1 => "one" 2 => "two" }' | cargo run --bin dsl-tui
```

### Automated Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_unknown_variable_error() {
        let mut interp = Interpreter::new().unwrap();
        let result = interp.eval(&IRNode::Variable("x".to_string())).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            InterpreterError::UnknownVariable { name, .. } => {
                assert_eq!(name, "x");
            }
            _ => panic!("Expected UnknownVariable error"),
        }
    }

    // Similar tests for other error types...
}
```

---

## Migration Path (Phase 1 → Phase 2)

### Quick Win (2-4 hours)

1. Update Runtime module signatures
2. Update Interpreter.eval() signature
3. Convert all error sites to use InterpreterError
4. Update Builtins module
5. Update consumers (REPL, TUI, egui)

**Result:** Structured errors without span tracking

### Full Implementation (8-12 hours)

6. Add span tracking infrastructure
7. Update compiler to preserve spans
8. Thread spans through evaluation

**Result:** Full rich errors with source locations

---

## Related Documentation

- [Architecture](architecture.md) - Overall system architecture
- [Known Issues](known-issues.md) - Parser errors and edge cases
- [egui Implementation](egui-implementation.md) - Error display UI

---

## References

**Source Files:**
- `crates/dsl-interpreter/src/error.rs` - Error type definitions
- `crates/dsl-egui/src/renderers/error.rs` - Error display implementation
- `crates/dsl-tui/src/app.rs` - TUI error handling
- `SYSTEMATIC_ERROR_REFACTORING_PLAN.md` - Detailed refactoring plan

**Created:** 2025-11-10
**Status:** Living document - updated as error handling evolves
