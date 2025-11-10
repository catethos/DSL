# Operator Changes - Comparison, Logical, and Parallel Syntax Update

## Summary

This document describes the major changes to the DSL operators, including the addition of comparison and logical operators, and the change from `||` parallel operator to `par()` function syntax.

## Changes Made

### 1. Added Comparison Operators

**New operators:** `==`, `!=`, `<`, `>`, `<=`, `>=`

These operators compare values and return boolean results.

**Examples:**
```javascript
5 == 5          // true
5 != 3          // true
3 < 5           // true
10 > 5          // true
5 <= 5          // true
10 >= 5         // true
```

**Supported types:**
- Int vs Int
- Float vs Float
- Int vs Float (with automatic promotion)
- String vs String (equality only)
- Bool vs Bool (equality only)

### 2. Added Logical Operators

**New operators:** `&&`, `||`, `!`

These operators work with boolean values.

**Examples:**
```javascript
true && true        // true
true && false       // false
true || false       // true
false || false      // false
!true               // false
!false              // true
```

**Operator precedence:**
1. `!` (NOT) - highest
2. `&&` (AND)
3. `||` (OR) - lowest

### 3. Conditional Operator Now Fully Functional

The ternary conditional operator `? :` now works with the new comparison and logical operators.

**Examples:**
```javascript
// Simple conditional
18 >= 18 ? "adult" : "minor"
// Result: "adult"

// With logical operators
age >= 18 && verified ? "proceed" : "reject"

// In pipeline
data >> (_.age >= 18 ? ProcessAdult(_) : ProcessMinor(_))
```

### 4. Changed Parallel Operator from `||` to `par()`

**Reason:** The `||` symbol is now used for logical OR, which is more frequently used in conditional expressions.

**Old syntax:**
```javascript
(expr1 || expr2 || expr3)
```

**New syntax:**
```javascript
par(expr1, expr2, expr3)
```

**Migration examples:**

| Old Syntax | New Syntax |
|------------|------------|
| `(5 \|\| 10 \|\| 15)` | `par(5, 10, 15)` |
| `(Ask("a") \|\| Ask("b"))` | `par(Ask("a"), Ask("b"))` |
| `(GetUser(1) \|\| GetUser(2))` | `par(GetUser(1), GetUser(2))` |

**With binding:**
```javascript
// Old
(5 || 10 || 15) as numbers

// New
par(5, 10, 15) as numbers
```

**With destructuring:**
```javascript
// Old
(10 || 20 || 30) as [a, b, c]

// New
par(10, 20, 30) as [a, b, c]
```

## Implementation Details

### Grammar Changes

1. **Added comparison operator precedence level**
   - Comparison operators have lower precedence than arithmetic
   - Higher precedence than logical operators

2. **Added logical operator precedence**
   - `||` (OR) - lowest precedence
   - `&&` (AND) - higher than OR
   - Both are left-associative

3. **Added unary operator support**
   - `!` (NOT) for boolean negation
   - `-` (minus) for numeric negation

4. **Removed `parallel` grammar rule**
   - `||` is now a logical operator, not a composition operator

### Evaluator Changes

**File:** `crates/dsl-core/src/eval/evaluator.rs`

Added operator handling in `apply_op()`:
- Comparison operators for Int, Float, String, Bool
- Logical operators for Bool (&&, ||)
- Type promotion for mixed Int/Float operations

### Builtin Functions

**File:** `crates/dsl-core/src/eval/builtin.rs`

Added two new builtin functions:
1. **`par(args...)`** - Parallel execution, returns list of results
2. **`not(bool)`** - Boolean negation (for `!` operator)

### Parser Changes

**File:** `crates/dsl-core/src/parser/mod.rs`

Added AST building for:
- `Rule::logical_or` - Builds BinaryOp with "||"
- `Rule::logical_and` - Builds BinaryOp with "&&"
- `Rule::comparison` - Builds BinaryOp with comparison operators
- `Rule::unary` - Handles `!` (as function call to "not") and `-`

## Use Cases Enabled

### 1. Data-Driven Conditionals

```javascript
// Process data differently based on its properties
data >> (_.score >= 0.8 ? HighQuality(_) : LowQuality(_))

// Multiple conditions
user >> (_.age >= 18 && _.verified ? Approve(_) : Reject(_))
```

### 2. Complex Boolean Logic

```javascript
// Filter logic
items >> (_.active && (_.priority > 5 || _.urgent) ? Include(_) : Skip(_))

// Access control
request >> (_.auth.role == "admin" || _.auth.role == "moderator" ? Grant(_) : Deny(_))
```

### 3. Pipeline with Conditions

```javascript
// Your original use case
userData
  >> (_.age >= 18 ? ProcessAdult(_) : ProcessMinor(_))
  >> (_.verified ? Approve(_) : RequestVerification(_))
```

### 4. Parallel Execution (with new syntax)

```javascript
// Multiple API calls
par(GetUser(1), GetUser(2), GetUser(3)) as users
  >> ProcessUsers(users)

// Multiple LLM queries
par(
  Ask("What is the capital of France?"),
  Ask("What is the capital of Germany?"),
  Ask("What is the capital of Italy?")
) as [france, germany, italy]
```

## Testing

All changes are covered by tests:

1. **Parser tests** (`crates/dsl-core/src/parser/mod.rs`):
   - `test_parse_comparison_operators`
   - `test_parse_logical_operators`
   - `test_parse_complex_conditional`
   - `test_parse_parallel` (updated for `par()`)

2. **Evaluator tests**: Implicit through parser tests and integration

3. **All tests pass**: `cargo test --package dsl-core` ✓

## Documentation Updated

The following documentation files have been updated:
- `docs/02-Getting-Started.md` - Updated examples and quick reference
- `docs/04-Language-Features.md` - Added new operator sections
- `docs/07-Workflow-Constructs.md` - Changed all `||` to `par()`
- `docs/01-Overview.md` - Updated parallel examples
- `docs/06-Functions.md` - Updated parallel examples
- `docs/08-LLM-Integration.md` - Updated parallel examples
- `docs/09-SQL-DuckDB.md` - Updated parallel examples
- `docs/10-HTTP-Client.md` - Updated parallel examples
- `docs/11-Advanced-Features.md` - Updated parallel examples

## Breaking Changes

### The `||` Operator

**Breaking change:** `||` no longer means parallel execution.

**Migration required:** All code using `(expr1 || expr2)` for parallel execution must be changed to `par(expr1, expr2)`.

**Why this is the right choice:**
- Logical OR (`||`) is used much more frequently than parallel execution
- `||` for logical OR is universal across programming languages
- `par()` is explicit and clear in intent
- Parallel execution is a specialized operation that deserves clear syntax

## Operator Precedence Summary

From highest to lowest:

1. **Function calls**, field access, indexing
2. **Unary operators**: `!`, `-`
3. **Multiplicative**: `*`, `/`
4. **Additive**: `+`, `-`
5. **Comparison**: `==`, `!=`, `<`, `>`, `<=`, `>=`
6. **Logical AND**: `&&`
7. **Logical OR**: `||`
8. **Sequential**: `>>`
9. **Conditional**: `? :`

## Examples Comparing Old vs New

### Example 1: Simple Conditional

**Old (not possible):**
```javascript
// This would error - no comparison operators
age ? ProcessAdult(age) : ProcessMinor(age)
```

**New:**
```javascript
age >= 18 ? ProcessAdult(age) : ProcessMinor(age)
```

### Example 2: Parallel Execution

**Old:**
```javascript
(GetUser(1) || GetUser(2) || GetUser(3)) as users
```

**New:**
```javascript
par(GetUser(1), GetUser(2), GetUser(3)) as users
```

### Example 3: Complex Logic

**Old (not possible):**
```javascript
// No way to express this
```

**New:**
```javascript
userData
  >> (_.age >= 18 && _.verified ? ProcessAdult(_) : ProcessMinor(_))
  >> (_.premium || _.trial ? PremiumFeatures(_) : BasicFeatures(_))
```

## Future Enhancements

Potential future additions:
- Pattern matching with `match` expressions
- Short-circuit evaluation optimization
- More comparison operators (e.g., string matching, regex)
- Null/undefined handling operators (`??`, `?.`)

---

**Date:** 2025-11-03  
**Author:** DSL Core Team  
**Version:** 0.2.0
