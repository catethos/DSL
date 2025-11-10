# Systematic Error Refactoring Plan

## Goal

Replace string-based errors (`Result<Value, String>`) with structured `InterpreterError` types throughout the interpreter, enabling:

- ✅ Automatic source location tracking (file:line:column)
- ✅ Function context preservation
- ✅ Type-safe error handling
- ✅ Rich error display without string parsing
- ✅ Better error messages for users

## Current State

### What Works Now ✅
- **Phase 4 Complete:** Rich `InterpreterError` types exist with 9 variants
- **egui Integration:** Error display UI with expandable sections
- **String Parsing Fallback:** Smart parsing extracts info from error strings
- **Intrinsic Functions:** Use `InterpreterError` for LLM, HTTP, SQL errors

### The Problem ❌
- **Main eval loop:** Returns `Result<Value, String>`
- **Runtime methods:** Return `Result<Value, String>`
- **Helper methods:** Return string errors
- **No span tracking:** Error locations not preserved during evaluation
- **String parsing needed:** GUI must parse error messages to extract context

## Architecture Overview

```
┌─────────────────────────────────────────────────┐
│  Parser (produces AST with spans)               │
└───────────────────┬─────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────┐
│  Compiler (produces IR, loses spans!)           │
└───────────────────┬─────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────┐
│  Interpreter.eval() → Result<Value, String>     │  ❌ Current
│                                                  │
│  Should be: → Result<Value, InterpreterError>   │  ✅ Target
└─────────────────────────────────────────────────┘
```

## Refactoring Strategy

### Phase 1: Update Type Signatures
Change return types from `Result<Value, String>` to `Result<Value, InterpreterError>`

### Phase 2: Update Error Sites
Replace `Err(format!(...))` with proper `InterpreterError` variants

### Phase 3: Add Span Tracking
Preserve span information through compilation and evaluation

### Phase 4: Update Consumers
Fix code that calls eval() to handle `InterpreterError`

---

## Detailed Implementation Plan

### STEP 1: Update Runtime Module ⚙️

**File:** `crates/dsl-interpreter/src/runtime.rs`

#### 1.1 Add InterpreterError Import
```rust
use crate::error::InterpreterError;
```

#### 1.2 Update get_var() Method
```rust
// OLD:
pub fn get_var(&self, name: &str) -> Result<Value, String> {
    // ...
    Err(format!("Variable '{}' not found", name))
}

// NEW:
pub fn get_var(&self, name: &str) -> Result<Value, InterpreterError> {
    // ...
    Err(InterpreterError::UnknownVariable {
        name: name.to_string(),
        source_span: None, // TODO: Add span tracking
    })
}
```

#### 1.3 Update Other Runtime Methods
- `get_function()` → return `InterpreterError::UnknownFunction`
- Any other methods returning `Result<_, String>`

**Estimated Time:** 30 minutes

---

### STEP 2: Update Interpreter Module 🔧

**File:** `crates/dsl-interpreter/src/interpreter.rs`

#### 2.1 Update Main eval() Signature
```rust
// OLD:
pub async fn eval(&mut self, node: &IRNode) -> Result<Value, String>

// NEW:
pub async fn eval(&mut self, node: &IRNode) -> Result<Value, InterpreterError>
```

#### 2.2 Update Helper Methods
All these methods need signature updates:
- `access_field()` → `Result<Value, InterpreterError>`
- `access_index()` → `Result<Value, InterpreterError>`
- `apply_binary_op()` → `Result<Value, InterpreterError>`
- `apply_unary_op()` → `Result<Value, InterpreterError>`
- `call_user_function()` → `Result<Value, InterpreterError>`
- `call_overloaded_function()` → `Result<Value, InterpreterError>`
- `apply_binding()` → `Result<InterpreterError>`
- `interpolate_template()` → `Result<String, InterpreterError>`

#### 2.3 Convert Error Sites

**Example: Variable Access**
```rust
// OLD:
IRNode::Variable(name) => self.runtime.get_var(name),

// NEW: (no change needed, runtime already returns InterpreterError)
IRNode::Variable(name) => self.runtime.get_var(name),
```

**Example: Type Errors**
```rust
// OLD:
Err(format!("Expected string, got {:?}", value))

// NEW:
Err(InterpreterError::TypeError {
    message: "Expected string for field access".to_string(),
    expected: "String".to_string(),
    got: format!("{:?}", value),
    source_span: None, // TODO: Add span
})
```

**Example: Runtime Errors**
```rust
// OLD:
Err(format!("Division by zero"))

// NEW:
Err(InterpreterError::RuntimeError {
    message: "Division by zero".to_string(),
    source_span: None, // TODO: Add span
})
```

**Example: Invalid Arguments**
```rust
// OLD:
Err(format!("Expected {} arguments, got {}", expected, got))

// NEW:
Err(InterpreterError::InvalidArguments {
    message: format!("Expected {} arguments, got {}", expected, got),
    source_span: None, // TODO: Add span
})
```

**Estimated Time:** 2-3 hours

---

### STEP 3: Update Builtins Module 🛠️

**File:** `crates/dsl-interpreter/src/builtins.rs`

The intrinsic functions already use `InterpreterError`, but regular builtins need updates:

#### 3.1 Update Builtin Function Signatures
```rust
// OLD:
fn builtin_len(&self, args: &[Value]) -> Result<Value, String>

// NEW:
fn builtin_len(&self, args: &[Value]) -> Result<Value, InterpreterError>
```

#### 3.2 Update Error Returns
```rust
// OLD:
Err(format!("len() requires 1 argument, got {}", args.len()))

// NEW:
Err(InterpreterError::InvalidArguments {
    message: format!("len() requires 1 argument, got {}", args.len()),
    source_span: None,
})
```

**Estimated Time:** 1-2 hours

---

### STEP 4: Update Consumers 📡

#### 4.1 REPL Module
**File:** `crates/dsl-repl/src/main.rs`

```rust
// OLD:
match interpreter.eval(&ir_node).await {
    Ok(value) => { /* ... */ }
    Err(e) => eprintln!("Error: {}", e),
}

// NEW:
match interpreter.eval(&ir_node).await {
    Ok(value) => { /* ... */ }
    Err(e) => eprintln!("Error: {}", e), // Display trait still works
}
```

#### 4.2 TUI Module
**File:** `crates/dsl-tui/src/app.rs`

Already handles errors as strings via Display, should work as-is.

#### 4.3 egui Module
**File:** `crates/dsl-egui/src/repl.rs`

```rust
// OLD:
let result: Result<Value, String> = interpreter.eval(&ir_node).await;
match result {
    Ok(value) => { /* ... */ }
    Err(err) => OutputItem::Error(ErrorDetail::from_string(err)),
}

// NEW:
let result: Result<Value, InterpreterError> = interpreter.eval(&ir_node).await;
match result {
    Ok(value) => { /* ... */ }
    Err(err) => OutputItem::Error(ErrorDetail::from_interpreter_error(err)),
}
```

Update `EvalResult` struct:
```rust
// OLD:
struct EvalResult {
    result: Result<dsl_ir::Value, String>,
}

// NEW:
struct EvalResult {
    result: Result<dsl_ir::Value, InterpreterError>,
}
```

**Estimated Time:** 1 hour

---

### STEP 5: Add Span Tracking 📍

This is the most complex part, requiring changes to IR and compiler.

#### 5.1 Add Spans to IRNode
**File:** `crates/dsl-ir/src/ir.rs`

```rust
// Add optional span to all IRNode variants that can cause errors
pub enum IRNode {
    Variable {
        name: String,
        span: Option<Span>,  // NEW
    },
    FunctionCall {
        name: String,
        args: Vec<IRNode>,
        effect_kind: Option<EffectKind>,
        source_span: Option<Span>,  // Already exists!
    },
    // ... update other variants
}
```

**Problem:** This is a breaking change affecting the entire codebase.

**Alternative Approach (Recommended):**
Keep IRNode as-is, but add a separate span tracking table:

```rust
// In interpreter
pub struct Interpreter {
    // ... existing fields
    span_table: HashMap<usize, Span>,  // Map node ID to span
}
```

#### 5.2 Update Compiler to Preserve Spans
**File:** `crates/dsl-core/src/compiler.rs`

When compiling AST → IR, record spans:
```rust
fn compile_expr(ast: &Expr) -> (IRNode, Option<Span>) {
    let span = ast.span.clone();
    let ir_node = /* compile logic */;
    (ir_node, span)
}
```

#### 5.3 Pass Spans Through Eval
```rust
pub async fn eval(&mut self, node: &IRNode, span: Option<Span>)
    -> Result<Value, InterpreterError>
{
    match node {
        IRNode::Variable(name) => {
            self.runtime.get_var(name).map_err(|mut e| {
                // Add span to error
                e.set_span(span.clone());
                e
            })
        }
        // ...
    }
}
```

**Estimated Time:** 4-6 hours (complex refactoring)

---

## Recommended Implementation Order

### Quick Win (2-4 hours) - Do This First! ✅
1. **Step 1:** Update Runtime module
2. **Step 2:** Update Interpreter.eval() signature
3. **Step 2:** Convert all error sites to use InterpreterError
4. **Step 3:** Update Builtins module
5. **Step 4:** Update consumers (REPL, TUI, egui)

**Result:** Structured errors work, but no span information yet.

### Full Implementation (8-12 hours total)
6. **Step 5:** Add span tracking infrastructure
7. **Step 5:** Update compiler to preserve spans
8. **Step 5:** Thread spans through evaluation

**Result:** Full rich errors with source locations!

---

## Testing Strategy

### After Quick Win (Steps 1-4)

**Test These Error Types:**

1. **Unknown Variable:**
   ```
   unknown_var
   ```
   Expected: `InterpreterError::UnknownVariable` with name

2. **Unknown Function:**
   ```
   unknown_func()
   ```
   Expected: `InterpreterError::UnknownFunction` with name

3. **Type Error:**
   ```
   "hello".age
   ```
   Expected: `InterpreterError::TypeError` with expected/got

4. **Invalid Arguments:**
   ```
   let f = |x| x * 2
   f()
   ```
   Expected: `InterpreterError::InvalidArguments`

5. **Runtime Error:**
   ```
   100 / 0
   ```
   Expected: `InterpreterError::RuntimeError`

**Verification:**
- Run `cargo test` - all tests should pass
- Run `cargo run --bin dsl-gui` - errors display with proper categories
- Check egui - suggestions should appear without string parsing
- Verify error types match (UNKNOWN VARIABLE, TYPE ERROR, etc.)

### After Full Implementation (Step 5)

**Test Span Information:**

1. Create test file `test.dsl`:
   ```
   let x = 10
   y + x
   ```

2. Run and verify error shows: `at test.dsl:2:1`

3. Check in egui - source location section should show actual file/line

---

## Migration Checklist

### Phase 1: Type Signatures ✅
- [ ] Add `InterpreterError` import to runtime.rs
- [ ] Update `Runtime::get_var()` signature
- [ ] Update `Runtime::get_function()` signature
- [ ] Update `Interpreter::eval()` signature
- [ ] Update all interpreter helper method signatures
- [ ] Update builtin function signatures
- [ ] Update `EvalResult` in egui/repl.rs

### Phase 2: Error Sites ✅
- [ ] Convert variable not found errors
- [ ] Convert function not found errors
- [ ] Convert type mismatch errors
- [ ] Convert invalid argument errors
- [ ] Convert runtime errors (division by zero, etc.)
- [ ] Convert index out of bounds errors
- [ ] Convert pattern match errors
- [ ] Convert builtin function errors

### Phase 3: Consumer Updates ✅
- [ ] Update dsl-repl error handling
- [ ] Update dsl-tui error handling
- [ ] Update dsl-egui error handling
- [ ] Remove string parsing fallback in egui (optional)

### Phase 4: Span Tracking (Advanced) 📍
- [ ] Design span tracking approach (table vs IRNode fields)
- [ ] Update compiler to capture spans
- [ ] Thread spans through eval chain
- [ ] Add span to all error sites
- [ ] Test source location accuracy
- [ ] Update egui to show clickable locations

### Phase 5: Testing & Documentation ✅
- [ ] Run all unit tests
- [ ] Test each error type manually
- [ ] Verify error display in all UIs (REPL, TUI, egui)
- [ ] Update error handling documentation
- [ ] Add examples for each error type
- [ ] Create troubleshooting guide

---

## Benefits After Implementation

### Immediate Benefits (After Quick Win)
- ✅ **Type-safe errors:** Compile-time checking of error handling
- ✅ **Better categorization:** Errors properly typed (UnknownVariable vs RuntimeError)
- ✅ **Rich context:** Error-specific fields (variable name, function name, etc.)
- ✅ **No string parsing:** Direct access to error information
- ✅ **Consistent errors:** All error sites use same types

### Full Benefits (After Span Tracking)
- ✅ **Source locations:** Exact file:line:column for every error
- ✅ **Better debugging:** Click to jump to error location (future)
- ✅ **IDE integration:** Error locations for editor plugins
- ✅ **Stack traces:** Track error through call chain
- ✅ **Context preservation:** Know which function/file caused error

---

## Potential Issues & Solutions

### Issue 1: Breaking Changes
**Problem:** Changing eval() signature breaks all callers

**Solution:**
1. Do it all at once in one commit
2. Or add a temporary wrapper:
   ```rust
   pub async fn eval_compat(&mut self, node: &IRNode) -> Result<Value, String> {
       self.eval(node).await.map_err(|e| e.to_string())
   }
   ```

### Issue 2: Span Information Not Available
**Problem:** Compiler doesn't preserve spans from parser

**Solution:**
1. Start with `span: None` everywhere
2. Add span tracking as separate enhancement
3. Gradually add spans to high-priority locations

### Issue 3: Error Conversion Complexity
**Problem:** Many error sites need careful conversion

**Solution:**
1. Use helper functions:
   ```rust
   fn type_error(expected: &str, got: &str) -> InterpreterError {
       InterpreterError::TypeError {
           message: format!("Expected {}, got {}", expected, got),
           expected: expected.to_string(),
           got: got.to_string(),
           source_span: None,
       }
   }
   ```

### Issue 4: Async Propagation
**Problem:** Can't use `?` operator with different error types

**Solution:**
- Already solved - all methods return same error type
- Use `.map_err()` if needed for specific conversions

---

## Success Criteria

### Quick Win Success ✅
- [ ] All code compiles without errors
- [ ] All existing tests pass
- [ ] Error types correctly identified in egui
- [ ] Suggestions still appear
- [ ] No regressions in functionality

### Full Success ✅
- [ ] All errors include source spans
- [ ] Source location section shows in egui
- [ ] Click location to navigate (future feature)
- [ ] Error messages improved
- [ ] Performance unchanged

---

## Timeline Estimates

| Phase | Tasks | Time | Difficulty |
|-------|-------|------|------------|
| Step 1 | Runtime updates | 30 min | Easy |
| Step 2 | Interpreter eval | 2-3 hrs | Medium |
| Step 3 | Builtins | 1-2 hrs | Easy |
| Step 4 | Consumers | 1 hr | Easy |
| **Quick Win Total** | **Steps 1-4** | **4-6 hrs** | **Medium** |
| Step 5 | Span tracking | 4-6 hrs | Hard |
| **Full Total** | **All steps** | **8-12 hrs** | **Hard** |

---

## Next Steps

### Immediate (Do Now)
1. **Read this plan** carefully
2. **Decide:** Quick Win only, or Full Implementation?
3. **Start with Step 1:** Update Runtime module (30 minutes)
4. **Test incrementally:** Build after each step

### Recommended Approach
1. **Do Quick Win first** (Steps 1-4) - Get structured errors working
2. **Test thoroughly** - Make sure everything works
3. **Commit & merge** - Don't lose progress
4. **Plan Step 5** - Design span tracking approach
5. **Implement Step 5** - Add source locations

---

## Resources

### Related Files
- **Error types:** `crates/dsl-interpreter/src/error.rs`
- **Runtime:** `crates/dsl-interpreter/src/runtime.rs`
- **Interpreter:** `crates/dsl-interpreter/src/interpreter.rs`
- **Builtins:** `crates/dsl-interpreter/src/builtins.rs`
- **egui integration:** `crates/dsl-egui/src/repl.rs`
- **Error display:** `crates/dsl-egui/src/renderers/error.rs`

### Documentation
- **Phase 4 Summary:** `docs/ir_refactoring_plan.md` (lines 59-72)
- **Error display:** `EGUI_RICH_ERRORS_COMPLETE.md`
- **String parsing fallback:** `FIXED_ERROR_DISPLAY.md`

### Testing Resources
- **Test cases:** `examples/test_rich_errors.dsl`
- **Testing guide:** `TESTING_RICH_ERRORS.md`

---

**Created:** November 9, 2025
**Status:** 📋 Planning Complete - Ready to Implement
**Difficulty:** Medium (Quick Win) → Hard (Full Implementation)
**Impact:** High - Professional error handling throughout system
