# SQL Variable Auto-Registration and Caching

**Feature Added:** Table auto-registration with `$variable` syntax and performance caching

## What's New

### 1. Auto-Registration with `$variable`

You can now use `$variable` syntax in SQL queries to automatically register DSL variables as tables:

```javascript
users = [
    {name: "Alice", age: 25},
    {name: "Bob", age: 30}
]

SQL("SELECT * FROM $users WHERE age > 28")
```

This replaces the need for manual table registration.

### 2. Table Caching

Tables are automatically cached after first registration, dramatically improving performance for repeated queries:

```javascript
data = [{id: 1}, {id: 2}, {id: 3}]

SQL("SELECT * FROM $data")  // Registers table
SQL("SELECT COUNT(*) FROM $data")  // Uses cache (fast!)
SQL("SELECT AVG(id) FROM $data")  // Still using cache
```

### 3. Manual Refresh

Use `refresh_table()` to update cached tables when data changes:

```javascript
users = [{name: "Alice"}]
SQL("SELECT * FROM $users")  // Returns 1 row

users = [{name: "Bob"}, {name: "Charlie"}]
refresh_table("users")
SQL("SELECT * FROM $users")  // Returns 2 rows (refreshed)
```

## Schema Requirements

Variables used with `$variable` must meet these requirements:

✅ **Valid:**
```javascript
// List of Maps with consistent schema
data = [
    {name: "Alice", age: 25},
    {name: "Bob", age: 30}
]
```

❌ **Invalid:**
```javascript
// Inconsistent schema
data = [
    {name: "Alice", age: 25},
    {name: "Bob"}  // Missing 'age' field
]

// Nested structures
data = [
    {name: "Alice", address: {street: "Main"}}
]

// Not all Maps
data = [{name: "Alice"}, "Bob"]
```

## Performance Impact

**Before (no caching):**
- Each query re-creates table
- 1000 rows × 10 queries = 10,000 INSERT operations

**After (with caching):**
- Table created once, cached
- 1000 rows × 10 queries = 1,000 INSERT operations
- **90% reduction in overhead**

## Migration Guide

### Old Pattern
```javascript
// No direct equivalent before
// Had to use CSV files or direct SQL
```

### New Pattern
```javascript
// Direct variable usage
users = [{name: "Alice", age: 25}]
SQL("SELECT * FROM $users")
```

### Refreshing Data
```javascript
// When data changes
users = new_data
refresh_table("users")  // Force update
SQL("SELECT * FROM $users")
```

## API Reference

### New Functions

**`SQL(query: String) -> Table`**
- Now supports `$variable` syntax
- Automatically caches registered tables
- See: [SQL Functions](../user-guide/09-SQL-DuckDB.md)

**`refresh_table(name: String) -> Null`**
- Clears cached table by name
- Forces re-registration on next use
- See: [Builtin Functions](../user-guide/06-Builtin-Functions.md#refresh_table)

## Examples

### Example 1: Simple Query
```javascript
products = [
    {id: 1, name: "Widget", price: 9.99},
    {id: 2, name: "Gadget", price: 19.99}
]

SQL("SELECT * FROM $products WHERE price > 10")
```

### Example 2: Join Multiple Tables
```javascript
users = [
    {id: 1, name: "Alice"},
    {id: 2, name: "Bob"}
]

orders = [
    {user_id: 1, amount: 100},
    {user_id: 2, amount: 200}
]

SQL("""
  SELECT u.name, o.amount
  FROM $users u
  JOIN $orders o ON u.id = o.user_id
""")
```

### Example 3: Refresh After Update
```javascript
scores = [{player: "Alice", score: 100}]

SQL("SELECT AVG(score) FROM $scores")  // avg = 100

scores = [
    {player: "Alice", score: 100},
    {player: "Bob", score: 200}
]

refresh_table("scores")
SQL("SELECT AVG(score) FROM $scores")  // avg = 150
```

## Breaking Changes

None - this is a purely additive feature.

## See Also

- [SQL and DuckDB Integration](../user-guide/09-SQL-DuckDB.md)
- [Builtin Functions Reference](../user-guide/06-Builtin-Functions.md)
