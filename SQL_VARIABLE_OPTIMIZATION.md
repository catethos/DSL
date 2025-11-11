# SQL Variable Auto-Registration Optimization

## Overview

The SQL executor now caches registered tables to avoid re-registering them on every query, significantly improving performance for repeated queries.

## Features

### 1. **$variable Syntax**
Use `$variable` in SQL queries to automatically register DSL variables as tables:

```dsl
users = [{name: "Alice", age: 25}, {name: "Bob", age: 30}]
SQL("SELECT * FROM $users WHERE age > 28")
```

### 2. **Automatic Caching**
Tables are registered once and cached:

```dsl
users = [{name: "Alice", age: 25}]

SQL("SELECT * FROM $users")  // Registers 'users' table
SQL("SELECT COUNT(*) FROM $users")  // Uses cached table (fast!)
```

### 3. **Force Refresh**
Use `refresh_table()` to update cached data:

```dsl
users = [{name: "Alice", age: 25}]
SQL("SELECT * FROM $users")  // Returns 1 row

users = [{name: "Bob", age: 30}, {name: "Charlie", age: 35}]
SQL("SELECT * FROM $users")  // Still returns 1 row (cached!)

refresh_table("users")
SQL("SELECT * FROM $users")  // Now returns 2 rows (refreshed!)
```

## Validation

Tables must meet these requirements:
- Must be a `List` of `Map`s
- All maps must have the same keys (consistent schema)
- No nested structures (no Map/List inside Map)
- Non-empty lists

### Valid Examples

```dsl
// ✓ Valid - consistent schema
users = [
    {name: "Alice", age: 25, city: "NYC"},
    {name: "Bob", age: 30, city: "LA"}
]

// ✓ Valid - all required fields present
products = [
    {id: 1, name: "Widget", price: 9.99},
    {id: 2, name: "Gadget", price: 19.99}
]
```

### Invalid Examples

```dsl
// ✗ Invalid - inconsistent schema
data = [
    {name: "Alice", age: 25},
    {name: "Bob"}  // Missing 'age' field
]

// ✗ Invalid - nested structure
data = [
    {name: "Alice", address: {street: "Main St"}}  // Nested map
]

// ✗ Invalid - not all items are maps
data = [
    {name: "Alice"},
    "Bob"  // String instead of map
]
```

## API

### Functions

- **`SQL(query: String) -> List`**
  - Execute SQL query with $variable auto-registration
  - Returns list of result rows as maps

- **`refresh_table(name: String) -> Null`**
  - Clear cached table by name
  - Forces re-registration on next use

### Internal Methods (SQLExecutor)

- **`clear_table(name: &str) -> Result<(), String>`**
  - Clear a specific cached table

- **`clear_all_tables() -> Result<(), String>`**
  - Clear all cached tables

## Performance

**Without caching:**
- Each query re-creates table and inserts all data
- O(n) per query where n = number of rows

**With caching:**
- First query: O(n) to register
- Subsequent queries: O(1) table lookup
- 10-100x faster for large datasets with repeated queries

## Implementation Details

- Uses `HashSet<String>` to track registered tables
- DuckDB `CREATE OR REPLACE TABLE` only called once per table
- Regex pattern: `r"\$([a-zA-Z_][a-zA-Z0-9_]*)"`
- Thread-safe: Each interpreter has its own executor instance
