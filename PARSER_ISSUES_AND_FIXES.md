# Parser Issues and Fixes

## Executive Summary

**Status**: 🔴 CRITICAL - 12 parsing failures identified
**Root Cause**: PEG backtracking allows malformed input to parse as simpler constructs
**Impact**: Runtime "Unknown Variable" errors instead of clear parse-time syntax errors

## Critical Issues Found

### 1. Unclosed Delimiters Parse Successfully ❌

| Input | Current Parse | Expected |
|-------|---------------|----------|
| `AnalyzeWithClaude("test` | `Variable("AnalyzeWithClaude")` | Parse Error |
| `foo(1, 2` | `Variable("foo")` | Parse Error |
| `Person { name: "Alice"` | `Variable("Person")` | Parse Error |
| `arr[5` | `Variable("arr")` | Parse Error |
| `user.` | `Variable("user")` | Parse Error |

### 2. Incomplete Operators Parse as Operand Only ❌

| Input | Current Parse | Expected |
|-------|---------------|----------|
| `5 +` | `Int(5)` | Parse Error |
| `5 \|>` | `Int(5)` | Parse Error |
| `x ? 1 :` | `Variable("x")` | Parse Error |

### 3. Root Cause

**PEG Grammar with Backtracking**:
```pest
primary = _{ ... | function_call | ... | identifier }
```

When `function_call` fails (unclosed paren), parser backtracks to `identifier` which succeeds, leaving remaining input unconsumed.

## Implementation Plan

### Priority 1: Critical Fixes (Week 1)

#### Fix 1.1: Add EOI Requirement to Expression Parsing ⭐ HIGHEST PRIORITY

**Problem**: `parse_expr()` doesn't require consuming all input

**Solution**:
```rust
// File: crates/dsl-core/src/parser/parsing.rs:222-231
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

**Grammar change**:
```pest
// File: crates/dsl-core/src/parser/grammar.pest (after line 242)
expr_with_eoi = _{ SOI ~ expr ~ EOI }
```

**Expected Result**: `parse_expr("foo(")` will fail with "expected ')'"

#### Fix 1.2: Make String Literals Atomic

**File**: `crates/dsl-core/src/parser/grammar.pest:391-394`

```pest
// Change from $ to @ to prevent backtracking
string_literal = @{
    "\"" ~ plain_string_inner ~ "\""
    | "'" ~ single_string_inner ~ "'"
}
```

#### Fix 1.3: Add Validation Layer for Unclosed Delimiters

**Create**: `crates/dsl-core/src/parser/validation.rs`

Implement delimiter balance checking to catch unclosed `()`, `[]`, `{}`, `""` before parsing.

#### Fix 1.4: Improve Error Messages

Add enhanced error formatting using Pest's error information to show:
- Line and column numbers
- What was expected vs what was found
- Clear, actionable messages

### Priority 2: Important Fixes (Week 2)

- Add lookahead assertions for better error detection
- Implement proper error recovery for REPL (allow incomplete input)
- Add comprehensive test coverage

### Priority 3: Nice to Have (Week 3)

- Separate REPL and file parsing modes
- Add fuzzing tests
- Performance optimization

## Test Results

**Test File**: `crates/dsl-core/tests/parser_edge_cases.rs`

Current Status:
- ✅ 14/26 tests passing (54%)
- ❌ 12/26 tests failing (46%)

After Priority 1 fixes:
- Expected: ✅ 26/26 tests passing (100%)

## References

- Full analysis: See agent output above
- Pest documentation: https://pest.rs/book/
- Related issue: Parser too permissive with backtracking

---

**Created**: 2025-11-09
**Updated**: 2025-11-09
**Status**: Ready for implementation
