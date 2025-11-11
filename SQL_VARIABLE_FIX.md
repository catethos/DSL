# SQL $variable Syntax Fix

## Issue

Users were getting the error:
```
Runtime error: SQL execution failed: SQL error: Parser Error: syntax error at or near "$"
LINE 1: SELECT * FROM $users
```

## Root Cause

The `$variable` syntax processing was only implemented for the intrinsic `__sql` path (used in function definitions), not for direct `SQL()` calls in user code.

**Call Paths:**
- **Intrinsic path** (`__sql`): Used for SQL blocks in function definitions → Had scope access ✓
- **Regular builtin path** (`SQL()`): Used for direct SQL calls → Missing scope access ✗

## Solution

Added scope access to regular builtin function calls:

### 1. New Method: `call_with_scope()`

```rust
pub async fn call_with_scope(
    &mut self,
    name: &str,
    args: Vec<Value>,
    scope_lookup: impl Fn(&str) -> Option<Value>,
) -> Result<Value>
```

Routes SQL calls to `sql_with_scope()` which has access to variable lookups.

### 2. New Method: `sql_with_scope()`

```rust
fn sql_with_scope(
    &mut self,
    args: Vec<Value>,
    scope_lookup: impl Fn(&str) -> Option<Value>,
) -> Result<Value>
```

Processes `$variable` syntax before executing the query.

### 3. Updated Interpreter

Changed builtin function calls to use `call_with_scope()`:

```rust
// Before
self.builtins.call(name, arg_values).await

// After
let runtime = &self.runtime;
self.builtins.call_with_scope(
    name,
    arg_values,
    |var_name| runtime.get_var(var_name).ok()
).await
```

## Files Modified

1. **`crates/dsl-interpreter/src/builtins.rs`**
   - Added `call_with_scope()` method
   - Added `sql_with_scope()` method
   - Kept existing `sql()` for backward compatibility

2. **`crates/dsl-interpreter/src/interpreter.rs`**
   - Updated builtin calls to use `call_with_scope()`
   - Passes scope lookup closure

## Testing

All 7 SQL tests pass:
- ✅ `test_sql_variable_auto_registration`
- ✅ `test_sql_variable_not_found`
- ✅ `test_sql_invalid_schema`
- ✅ `test_sql_nested_structures`
- ✅ `test_sql_table_caching`
- ✅ `test_sql_clear_table`
- ✅ `test_sql_clear_all_tables`

## Usage Example

```javascript
users = [
    {name: "Alice", age: 25},
    {name: "Bob", age: 30}
]

// Now works!
SQL("SELECT * FROM $users WHERE age > 28")
```

## Impact

- ✅ No breaking changes
- ✅ Backward compatible
- ✅ Adds functionality to regular SQL() calls
- ✅ Maintains existing intrinsic path
- ✅ Performance caching still works
