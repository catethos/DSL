# Known Issues and Workarounds

**Last Updated:** 2025-11-10
**Status:** Active - 12 parser issues identified

---

## Table of Contents

- [Overview](#overview)
- [Critical Parser Issues](#critical-parser-issues)
- [Root Cause Analysis](#root-cause-analysis)
- [Planned Fixes](#planned-fixes)
- [Workarounds](#workarounds)
- [Testing](#testing)

---

## Overview

This document tracks known issues in the DSL, primarily related to **parser backtracking** that allows malformed input to parse successfully. These issues result in runtime "Unknown Variable" errors instead of clear parse-time syntax errors.

### Issue Summary

**Status:** 🔴 CRITICAL
**Affected Component:** Parser (Pest PEG grammar)
**Impact:** Poor error messages, confused users
**Root Cause:** PEG backtracking allows incomplete expressions to parse as simpler constructs

**Test Results:**
- ✅ 14/26 edge case tests passing (54%)
- ❌ 12/26 edge case tests failing (46%)

---

## Critical Parser Issues

### Issue 1: Unclosed Delimiters Parse Successfully

**Severity:** 🔴 Critical
**Status:** Not fixed

#### Symptoms

Unclosed function calls, type instantiations, and array accesses parse as simple identifiers:

| Input | Current Parse | Expected |
|-------|---------------|----------|
| `AnalyzeWithClaude("test` | `Variable("AnalyzeWithClaude")` | **Parse Error:** Unclosed string |
| `foo(1, 2` | `Variable("foo")` | **Parse Error:** Unclosed paren |
| `Person { name: "Alice"` | `Variable("Person")` | **Parse Error:** Unclosed brace |
| `arr[5` | `Variable("arr")` | **Parse Error:** Unclosed bracket |
| `user.` | `Variable("user")` | **Parse Error:** Expected field name |

#### Example Session

```javascript
> AnalyzeWithClaude("test
Variable 'AnalyzeWithClaude' not found  // ❌ BAD ERROR

// Expected:
Parse error at line 1, column 21:
  Unclosed string literal
  AnalyzeWithClaude("test
                         ^
```

#### Why This Happens

PEG grammar with ordered choice:

```pest
primary = _{ ... | function_call | ... | identifier }
```

Parsing steps:
1. Try `function_call` rule
2. Matches `identifier` and `(`
3. Fails on unclosed `)`
4. **Backtracks** to `identifier` rule
5. Successfully parses `AnalyzeWithClaude` as identifier
6. Leaves `("test` unconsumed

Since `parse_expr()` doesn't require EOI (End Of Input), the remaining unparsed input is silently ignored.

---

### Issue 2: Incomplete Operators Parse as Operand Only

**Severity:** 🔴 Critical
**Status:** Not fixed

#### Symptoms

Incomplete binary operations parse as just the left operand:

| Input | Current Parse | Expected |
|-------|---------------|----------|
| `5 +` | `Int(5)` | **Parse Error:** Expected right operand |
| `5 \|>` | `Int(5)` | **Parse Error:** Expected right operand |
| `x ? 1 :` | `Variable("x")` | **Parse Error:** Expected else expression |
| `a &&` | `Variable("a")` | **Parse Error:** Expected right operand |

#### Example Session

```javascript
> 5 +
5  // ❌ BAD - silently ignores the +

// Expected:
Parse error at line 1, column 3:
  Expected expression after '+'
  5 +
    ^
```

#### Why This Happens

Similar backtracking issue:
1. Try to parse `binary_op`
2. Matches left operand `5`
3. Matches operator `+`
4. Fails to find right operand
5. **Backtracks** to just parsing `5`
6. Leaves `+` unconsumed

---

### Issue 3: Unclosed Strings with Interpolation

**Severity:** 🟡 High
**Status:** Not fixed

#### Symptoms

Template strings with unclosed interpolation parse as regular strings:

```javascript
> "Hello ${name
"Hello ${name  // ❌ Parsed as literal string (no interpolation)

// Expected:
Parse error: Unclosed interpolation in template string
```

---

### Issue 4: Incomplete Conditionals

**Severity:** 🟡 High
**Status:** Not fixed

#### Symptoms

```javascript
> x ? 1 :
x  // ❌ Parses as just 'x', ignores '? 1 :'

// Expected:
Parse error: Expected else expression after ':'
```

---

### Issue 5: Trailing Operators in Pipelines

**Severity:** 🟠 Medium
**Status:** Not fixed

#### Symptoms

```javascript
> upper("hello") |>
"HELLO"  // ❌ Silently ignores the pipe operator

// Expected:
Parse error: Expected expression after '|>'
```

---

## Root Cause Analysis

### PEG Grammar Backtracking

**Problem:** Pest PEG parser uses **ordered choice** with backtracking.

When a parsing rule fails, the parser backtracks and tries the next alternative:

```pest
primary = _{
    // Try these in order:
    string_literal
    | int_literal
    | float_literal
    | bool_literal
    | list_literal
    | map_literal
    | type_instantiation    // ← Tries this first
    | function_call         // ← Then this
    | field_access
    | index_access
    | identifier            // ← Falls back to this
    | "(" ~ expr ~ ")"
}
```

### Why Backtracking Causes Issues

**Scenario:** Parsing `foo(`

1. **Try `function_call`:**
   ```pest
   function_call = { identifier ~ "(" ~ expr_list? ~ ")" }
   ```
   - ✅ Matches `identifier` (`foo`)
   - ✅ Matches `(`
   - ❌ **Fails** to match `)`

2. **Backtrack:**
   - Parser rewinds to before `function_call` attempt
   - State reset to start of input

3. **Try next option (`identifier`):**
   ```pest
   identifier = @{ ... }
   ```
   - ✅ Matches `foo`
   - **Success!**
   - Leaves `(` unconsumed

4. **parse_expr() returns successfully:**
   - Returns `Expr::Variable("foo")`
   - Doesn't check if all input consumed
   - `(` is ignored

### The Missing Check

The issue is that `parse_expr()` doesn't require EOI:

```rust
// Current implementation (BAD):
pub fn parse_expr(input: &str) -> Result<Expr, String> {
    let mut pairs = DslParser::parse(Rule::expr, input)?;
    let pair = pairs.next().ok_or(...)?;
    build_expr(pair)  // ❌ Doesn't check if pairs.next() is Some
}
```

Should be:

```rust
// Fixed implementation (GOOD):
pub fn parse_expr(input: &str) -> Result<Expr, String> {
    let mut pairs = DslParser::parse(Rule::expr_with_eoi, input)?;
    let pair = pairs.next().ok_or(...)?;

    // ✅ Verify no remaining input
    if pairs.next().is_some() {
        return Err("Unexpected input after expression".to_string());
    }

    build_expr(pair)
}
```

---

## Planned Fixes

### Priority 1: Critical Fixes (Week 1)

#### Fix 1.1: Add EOI Requirement ⭐ HIGHEST PRIORITY

**File:** `crates/dsl-core/src/parser/grammar.pest`

**Add new rule:**
```pest
expr_with_eoi = _{ SOI ~ expr ~ EOI }
```

**Update parser:**
```rust
// File: crates/dsl-core/src/parser/parsing.rs
pub fn parse_expr(input: &str) -> Result<Expr, String> {
    let mut pairs = DslParser::parse(Rule::expr_with_eoi, input)
        .map_err(|e| format!("Parse error: {}", e))?;

    let pair = pairs.next()
        .ok_or_else(|| "No expression found".to_string())?;

    // Verify no remaining input
    if pairs.next().is_some() {
        return Err("Unexpected input after expression".to_string());
    }

    build_expr(pair)
}
```

**Expected Result:**
`parse_expr("foo(")` fails with "expected ')'"

**Estimated Time:** 30 minutes

---

#### Fix 1.2: Make String Literals Atomic

**File:** `crates/dsl-core/src/parser/grammar.pest:391-394`

**Change:**
```pest
// OLD (allows backtracking):
string_literal = ${
    "\"" ~ plain_string_inner ~ "\""
    | "'" ~ single_string_inner ~ "'"
}

// NEW (atomic, no backtracking):
string_literal = @{
    "\"" ~ plain_string_inner ~ "\""
    | "'" ~ single_string_inner ~ "'"
}
```

**Why:** `@` makes the rule **atomic** - once it starts matching, it must complete or fail entirely (no backtracking).

**Expected Result:**
`"Hello` fails immediately with "Unclosed string" instead of parsing as identifier.

**Estimated Time:** 15 minutes

---

#### Fix 1.3: Add Delimiter Balance Validation

**Create:** `crates/dsl-core/src/parser/validation.rs`

Pre-parse validation to catch obvious errors:

```rust
pub fn validate_delimiters(input: &str) -> Result<(), String> {
    let mut stack = Vec::new();

    for (i, ch) in input.chars().enumerate() {
        match ch {
            '(' | '[' | '{' => stack.push((ch, i)),
            ')' => check_close(&mut stack, '(', i)?,
            ']' => check_close(&mut stack, '[', i)?,
            '}' => check_close(&mut stack, '{', i)?,
            '"' | '\'' => handle_string(input, i)?,
            _ => {}
        }
    }

    if let Some((open, pos)) = stack.pop() {
        return Err(format!("Unclosed '{}' at position {}", open, pos));
    }

    Ok(())
}
```

**Usage:**
```rust
pub fn parse_expr(input: &str) -> Result<Expr, String> {
    // ✅ Check delimiters first
    validate_delimiters(input)?;

    // Then parse normally
    let mut pairs = DslParser::parse(Rule::expr_with_eoi, input)?;
    // ...
}
```

**Expected Result:**
`foo(1, 2` fails fast with "Unclosed '(' at position 3"

**Estimated Time:** 1 hour

---

#### Fix 1.4: Improve Error Messages

**Goal:** Show helpful errors with context

**Implementation:**
```rust
fn format_parse_error(input: &str, error: pest::error::Error<Rule>) -> String {
    let (line, col) = error.line_col;
    let line_str = input.lines().nth(line - 1).unwrap_or("");

    format!(
        "Parse error at line {}, column {}:\n\
         {}\n\
         {}{}\n\
         Expected: {}",
        line, col,
        line_str,
        " ".repeat(col - 1), "^",
        format_expected(&error.expected)
    )
}
```

**Example Output:**
```
Parse error at line 1, column 4:
foo(
   ^
Expected: expression or ')'
```

**Estimated Time:** 1 hour

---

### Priority 2: Important Fixes (Week 2)

#### Fix 2.1: Lookahead Assertions

Add negative lookahead to prevent ambiguity:

```pest
identifier = @{
    !("let" | "type" | "enum" | "def" | "if" | "match") ~
    ASCII_ALPHA ~ (ASCII_ALPHANUMERIC | "_")*
}
```

**Estimated Time:** 30 minutes

---

#### Fix 2.2: Error Recovery for REPL

Allow incomplete input in REPL mode for better UX:

```rust
pub fn parse_expr_repl(input: &str) -> Result<Expr, ReplParseError> {
    match parse_expr(input) {
        Ok(expr) => Ok(expr),
        Err(e) if is_incomplete(&e) => {
            Err(ReplParseError::Incomplete)  // Ask for more input
        }
        Err(e) => Err(ReplParseError::Invalid(e))
    }
}
```

**Estimated Time:** 2 hours

---

#### Fix 2.3: Comprehensive Test Coverage

Add tests for all edge cases:

```rust
// File: crates/dsl-core/tests/parser_edge_cases.rs

#[test]
fn test_unclosed_function_call() {
    let result = parse_expr("foo(1, 2");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unclosed"));
}

#[test]
fn test_incomplete_binary_op() {
    let result = parse_expr("5 +");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Expected expression"));
}

// Add 20+ more tests...
```

**Estimated Time:** 2 hours

---

### Priority 3: Nice to Have (Week 3)

#### Fix 3.1: Separate REPL and File Parsing

Different parsing modes:
- **File mode:** Strict, require complete expressions
- **REPL mode:** Lenient, support multi-line input

**Estimated Time:** 3 hours

---

#### Fix 3.2: Fuzzing Tests

Automated testing with random inputs:

```rust
#[cfg(test)]
mod fuzz {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn doesnt_crash(s in "\\PC*") {
            let _ = parse_expr(&s);  // Shouldn't panic
        }
    }
}
```

**Estimated Time:** 2 hours

---

## Workarounds

### For Users

Until fixes are implemented:

#### 1. Check for Unclosed Delimiters

**Before running:**
```javascript
foo(1, 2  // ❌ Will fail with confusing error
```

**Manually verify:**
- Count opening `(` and closing `)`
- Check all strings are closed
- Ensure no trailing operators

#### 2. Use :check Command (Future)

```javascript
> :check foo(1, 2
Error: Unclosed '(' at position 3
```

#### 3. Test in Small Chunks

Break complex expressions into parts:

```javascript
// Instead of:
> foo(bar(baz(1, 2

// Do:
> let x = baz(1, 2)
> let y = bar(x)
> foo(y)
```

---

### For Developers

#### 1. Add Validation Before Parsing

```rust
// In your code:
if input.contains('(') && !input.contains(')') {
    return Err("Possible unclosed parenthesis".to_string());
}
```

#### 2. Use Alternative Parsing API

```rust
// Instead of parse_expr():
pub fn parse_expr_strict(input: &str) -> Result<Expr, String> {
    validate_delimiters(input)?;
    parse_expr(input)
}
```

#### 3. Test Edge Cases

Always test with:
- Unclosed delimiters
- Trailing operators
- Empty expressions
- Nested structures

---

## Testing

### Current Test Suite

**Location:** `crates/dsl-core/tests/parser_edge_cases.rs`

**Results (as of 2025-11-09):**
```
running 26 tests
test test_unclosed_function_call ... FAILED
test test_unclosed_type_instantiation ... FAILED
test test_unclosed_array_access ... FAILED
test test_trailing_field_access ... FAILED
test test_incomplete_binary_op_plus ... FAILED
test test_incomplete_binary_op_pipe ... FAILED
test test_incomplete_conditional ... FAILED
test test_unclosed_string ... FAILED
test test_unclosed_template ... FAILED
test test_trailing_and ... FAILED
test test_trailing_or ... FAILED
test test_incomplete_map ... FAILED

test result: FAILED. 14 passed; 12 failed; 0 ignored; 0 measured; 0 filtered out
```

### After Priority 1 Fixes

**Expected:**
```
running 26 tests
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Running Tests

```bash
# Run all parser tests
cargo test --package dsl-core --test parser_edge_cases

# Run specific test
cargo test --package dsl-core --test parser_edge_cases test_unclosed_function_call

# Run with verbose output
cargo test --package dsl-core --test parser_edge_cases -- --nocapture
```

---

## Impact Assessment

### User Impact: HIGH 🔴

**Affected Users:** All (especially beginners)

**Pain Points:**
- Confusing error messages ("Variable not found" instead of "Syntax error")
- Silent failures (operators ignored)
- Hard to debug (error location is wrong)

**User Sentiment:**
- "Why does `foo(` say variable not found?"
- "My code just silently ignores the + sign"
- "Error messages are misleading"

### Developer Impact: MEDIUM 🟡

**Affected Code:** Parser and error handling

**Workarounds Available:** Yes (validation functions)

**Maintenance Burden:** Medium (requires testing all changes)

---

## Progress Tracking

### Completion Status

| Fix | Priority | Status | ETA |
|-----|----------|--------|-----|
| 1.1 EOI requirement | P1 | Not started | Week 1 |
| 1.2 Atomic strings | P1 | Not started | Week 1 |
| 1.3 Delimiter validation | P1 | Not started | Week 1 |
| 1.4 Better error messages | P1 | Not started | Week 1 |
| 2.1 Lookahead assertions | P2 | Not started | Week 2 |
| 2.2 REPL error recovery | P2 | Not started | Week 2 |
| 2.3 Test coverage | P2 | Not started | Week 2 |
| 3.1 REPL vs file mode | P3 | Not started | Week 3 |
| 3.2 Fuzzing | P3 | Not started | Week 3 |

**Overall Progress:** 0% (Not started)

---

## Related Issues

### Issue: Pattern Matching Edge Cases

**Status:** ✅ Resolved

Pattern matching had incomplete implementation, now working:
- ✅ Multi-arm function definitions
- ✅ Clause merging
- ✅ Match expressions

**Resolution:** Phase 10B completion (2025-11-07)

---

### Issue: Span Tracking Missing

**Status:** Planned (Phase 2 of error handling)

Errors don't include source locations.

**See:** [Error Handling Documentation](error-handling.md#phase-2-span-tracking-planned)

---

## Additional Resources

**Source Files:**
- `crates/dsl-core/src/parser/grammar.pest` - Grammar definition
- `crates/dsl-core/src/parser/mod.rs` - Parser implementation
- `crates/dsl-core/tests/parser_edge_cases.rs` - Edge case tests
- `PARSER_ISSUES_AND_FIXES.md` - Detailed analysis

**Documentation:**
- [Architecture](architecture.md) - Parser pipeline
- [Error Handling](error-handling.md) - Error system design

**External Resources:**
- [Pest Book](https://pest.rs/book/) - PEG parser documentation
- [Pest Tutorial](https://pest.rs/book/examples.html) - Examples

---

**Created:** 2025-11-10
**Status:** Living document - updated as issues are fixed
