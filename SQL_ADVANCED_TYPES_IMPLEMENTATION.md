# SQL Advanced Data Types Implementation

## Summary

Extended `$variable` syntax to support **all DSL data types**, not just tables, by leveraging DuckDB's native support for arrays, structs, and primitives.

## What Was Changed

### Strategy

Instead of trying to register everything as tables, we now:
1. **List<Map>** → Register as table (existing behavior with caching)
2. **Other types** → Interpolate as DuckDB literals (new!)

This gives users access to DuckDB's powerful array and struct functions.

### Implementation

**File: `crates/dsl-interpreter/src/sql.rs`**

#### 1. Added `is_table_data()` 
Checks if a value should be registered as a table:
```rust
fn is_table_data(value: &Value) -> bool {
    match value {
        Value::List(items) if !items.is_empty() => {
            matches!(items[0], Value::Map(_))
        }
        _ => false,
    }
}
```

#### 2. Added `value_to_duckdb_literal()`
Converts DSL values to DuckDB literal syntax:
```rust
fn value_to_duckdb_literal(&self, value: &Value) -> String {
    match value {
        Value::String(s) => format!("'{}'", s.replace("'", "''")),
        Value::Int(n) => n.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "NULL".to_string(),
        Value::List(items) => format!("[{}]", ...),  // Array literal
        Value::Map(map) => format!("{{{}}}", ...),    // Struct literal
        Value::Markdown(s) | Value::Image(s) => format!("'{}'", ...),
    }
}
```

#### 3. Updated `prepare_query_with_variables()`
Now handles both tables and literals:
```rust
if Self::is_table_data(&value) {
    // Register as table (cached)
    self.register_table(&var_name, &value)?;
    query = query.replace(&placeholder, &var_name);
} else {
    // Interpolate as literal
    let literal = self.value_to_duckdb_literal(&value);
    query = query.replace(&placeholder, &literal);
}
```

## What Users Can Now Do

### Arrays
```javascript
let numbers = [1, 2, 3, 4, 5]
SQL("SELECT unnest($numbers) as value")
SQL("SELECT list_sum($numbers) as total")
```

### Structs
```javascript
let person = {name: "Alice", age: 25}
SQL("SELECT $person.name as name, $person.age as age")
```

### Primitives
```javascript
let threshold = 100
SQL("SELECT * FROM sales WHERE amount > $threshold")
```

### Nested Structures
```javascript
let matrix = [[1, 2], [3, 4]]
SQL("SELECT list_flatten($matrix) as flat")
```

### Combined with Tables
```javascript
let users = [{id: 1, name: "Alice"}]
let active_ids = [1, 2, 3]
SQL("SELECT * FROM $users WHERE id IN (SELECT unnest($active_ids))")
```

## Benefits

### 1. Leverages DuckDB's Power
Users get access to:
- `unnest()` - expand arrays
- `list_sum()`, `list_avg()` - array aggregations
- `list_concat()`, `array_intersect()` - array operations
- Struct field access
- Nested structure support

### 2. No Performance Overhead
- Literals are embedded directly (no table creation)
- Only List<Map> creates cached tables
- Best of both worlds

### 3. Flexible Data Processing
```javascript
// Process array inline
let scores = [85, 92, 78]
SQL("SELECT AVG(unnest($scores)) as avg")

// Process struct inline  
let config = {host: "localhost", port: 5432}
SQL("SELECT $config.host || ':' || $config.port as url")

// Mix with tables
let data = [{value: 10}, {value: 20}]
let filter = [10, 20, 30]
SQL("SELECT * FROM $data WHERE value IN (SELECT unnest($filter))")
```

## Testing

Added 4 new tests:
- ✅ `test_sql_array_literal` - Array processing
- ✅ `test_sql_primitive_values` - Primitive interpolation
- ✅ `test_sql_struct_literal` - Struct literal generation
- ✅ `test_sql_nested_array` - Nested array support

All 11 SQL tests passing.

## Documentation

Created:
- **`docs/user-guide/SQL-Advanced-Types.md`** - Complete guide with examples
- Updated existing SQL documentation

## Examples

### Before (Only Tables)
```javascript
// ❌ Only List<Map> worked
users = [{name: "Alice"}]
SQL("SELECT * FROM $users")

// ❌ Other types gave errors
numbers = [1, 2, 3]
SQL("SELECT unnest($numbers)")  // Error!
```

### After (All Types)
```javascript
// ✅ Tables still work (cached)
users = [{name: "Alice"}]
SQL("SELECT * FROM $users")

// ✅ Arrays work!
numbers = [1, 2, 3]
SQL("SELECT unnest($numbers) as value")
SQL("SELECT list_sum($numbers) as total")

// ✅ Structs work!
person = {name: "Alice", age: 25}
SQL("SELECT $person.name, $person.age")

// ✅ Primitives work!
threshold = 100
SQL("SELECT * FROM sales WHERE amount > $threshold")
```

## Impact

- ✅ **No breaking changes** - existing table registration still works
- ✅ **Backward compatible** - old queries unchanged
- ✅ **Additive feature** - extends capabilities
- ✅ **Performance optimized** - tables cached, literals inline
- 🚀 **Unlocks DuckDB power** - full array/struct function access

## Files Modified

1. `crates/dsl-interpreter/src/sql.rs`
   - Added `is_table_data()`
   - Added `value_to_duckdb_literal()`
   - Updated `prepare_query_with_variables()`

2. `crates/dsl-interpreter/tests/test_sql_variables.rs`
   - Added 4 new tests for advanced types

3. `docs/user-guide/SQL-Advanced-Types.md`
   - New comprehensive guide with examples
