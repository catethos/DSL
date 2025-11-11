# SQL Advanced Data Types

## Overview

DuckDB supports rich data types beyond simple tables. The `$variable` syntax automatically handles different data types appropriately:

- **List<Map>** → Registered as table (cached)
- **List<Primitive>** → Array literal `[1, 2, 3]`
- **Map** → Struct literal `{'key': 'value'}`
- **Primitives** → Direct literal values

## Data Type Handling

### List of Maps → Tables

```javascript
let users = [
    {id: 1, name: "Alice"},
    {id: 2, name: "Bob"}
]

SQL("SELECT * FROM $users WHERE id = 1")
// Registers 'users' as a cached table
```

### List of Primitives → Arrays

```javascript
let numbers = [1, 2, 3, 4, 5]

SQL("SELECT unnest($numbers) as value")
// Expands: SELECT unnest([1, 2, 3, 4, 5]) as value

SQL("SELECT $numbers as arr")
// Returns: [{arr: [1, 2, 3, 4, 5]}]
```

### Maps → Structs

```javascript
let person = {name: "Alice", age: 25}

SQL("SELECT $person.name as name, $person.age as age")
// Expands: SELECT {'name': 'Alice', 'age': 25}.name as name, ...
```

### Primitives → Literals

```javascript
let count = 42
let name = "Alice"
let pi = 3.14

SQL("SELECT $count as num, $name as text, $pi as f")
// Expands: SELECT 42 as num, 'Alice' as text, 3.14 as f
```

## DuckDB Array Functions

### UNNEST - Expand Arrays

```javascript
let tags = ["sql", "database", "analytics"]

SQL("SELECT unnest($tags) as tag")
// Returns:
// [{tag: "sql"}, {tag: "database"}, {tag: "analytics"}]
```

### LIST Functions

```javascript
let numbers = [1, 2, 3, 4, 5]

SQL("SELECT list_sum($numbers) as total")
// Returns: [{total: 15}]

SQL("SELECT list_avg($numbers) as avg")
// Returns: [{avg: 3.0}]

SQL("SELECT list_max($numbers) as max")
// Returns: [{max: 5}]
```

### Array Operations

```javascript
let arr1 = [1, 2, 3]
let arr2 = [3, 4, 5]

SQL("SELECT list_concat($arr1, $arr2) as combined")
// Returns: [{combined: [1, 2, 3, 3, 4, 5]}]

SQL("SELECT array_intersect($arr1, $arr2) as common")
// Returns: [{common: [3]}]
```

## DuckDB Struct Functions

### Accessing Struct Fields

```javascript
let user = {name: "Alice", age: 25, city: "NYC"}

SQL("SELECT $user.name as name, $user.age as age")
// Access nested fields directly
```

### Struct Arrays

```javascript
let people = [
    {name: "Alice", skills: ["Python", "SQL"]},
    {name: "Bob", skills: ["Java", "C++"]}
]

SQL("""
  SELECT 
    name,
    unnest(skills) as skill
  FROM $people
""")
// Expands nested arrays within table
```

## Complex Nested Structures

### Nested Arrays

```javascript
let matrix = [
    [1, 2, 3],
    [4, 5, 6],
    [7, 8, 9]
]

SQL("SELECT unnest($matrix) as row")
// Returns: [{row: [1,2,3]}, {row: [4,5,6]}, {row: [7,8,9]}]

SQL("SELECT list_flatten($matrix) as flat")
// Returns: [{flat: [1,2,3,4,5,6,7,8,9]}]
```

### Mixed Type Arrays

```javascript
let data = [1, "text", 3.14, true]

SQL("SELECT unnest($data) as value")
// DuckDB handles mixed types automatically
```

## Practical Examples

### Example 1: Array Filtering

```javascript
let scores = [85, 92, 78, 95, 88]

SQL("""
  SELECT 
    unnest($scores) as score
  WHERE score >= 90
""")
// Returns: [{score: 92}, {score: 95}]
```

### Example 2: Struct Transformation

```javascript
let config = {
    database: "mydb",
    port: 5432,
    ssl: true
}

SQL("""
  SELECT 
    $config.database || ':' || CAST($config.port AS VARCHAR) as connection,
    $config.ssl as secure
""")
```

### Example 3: Combining Arrays and Tables

```javascript
let users = [{id: 1, name: "Alice"}, {id: 2, name: "Bob"}]
let active_ids = [1]

SQL("""
  SELECT u.*
  FROM $users u
  WHERE u.id IN (SELECT unnest($active_ids))
""")
// Filter table by array membership
```

### Example 4: Array Aggregation

```javascript
let categories = [
    {product: "A", tags: ["new", "sale"]},
    {product: "B", tags: ["popular", "sale"]},
    {product: "C", tags: ["new"]}
]

SQL("""
  SELECT 
    unnest(tags) as tag,
    list_agg(product) as products
  FROM $categories
  GROUP BY tag
""")
// Aggregate products by tag
```

## Performance Considerations

### Tables (List<Map>)
- ✅ **Cached** - Registered once, reused
- ✅ **Indexed** - DuckDB can optimize queries
- ✅ **Best for**: Large datasets, multiple queries

### Literals (Arrays, Structs, Primitives)
- ⚡ **Inline** - No registration overhead
- ⚡ **Direct** - Embedded in query
- ✅ **Best for**: Small datasets, single use, filters

### When to Use Each

**Use Tables ($variable as List<Map>):**
```javascript
// Large dataset, multiple queries
let sales = [...1000 records...]
SQL("SELECT * FROM $sales WHERE region = 'US'")
SQL("SELECT SUM(amount) FROM $sales")
SQL("SELECT * FROM $sales WHERE date > '2024-01-01'")
```

**Use Literals ($variable as other types):**
```javascript
// Small filters or parameters
let regions = ["US", "EU", "APAC"]
SQL("SELECT * FROM sales WHERE region IN (SELECT unnest($regions))")

// Configuration values
let threshold = 1000
SQL("SELECT * FROM sales WHERE amount > $threshold")
```

## Limitations

### Cannot Mix Approaches
```javascript
// ❌ Cannot register non-List<Map> as table
let numbers = [1, 2, 3]
SQL("SELECT * FROM $numbers")  // Error: not a table

// ✅ Use array functions instead
SQL("SELECT unnest($numbers) as value")
```

### Struct Field Access
```javascript
// ✅ Works
let person = {name: "Alice"}
SQL("SELECT $person.name")

// ❌ Doesn't work - use table instead
let people = [{name: "Alice"}]
SQL("SELECT $people.name")  // Error: people is a table, not struct
SQL("SELECT name FROM $people")  // ✅ Correct
```

## See Also

- [SQL and DuckDB Integration](09-SQL-DuckDB.md)
- [Builtin Functions](06-Builtin-Functions.md#sql)
- [DuckDB Array Functions](https://duckdb.org/docs/sql/functions/array)
- [DuckDB Struct Functions](https://duckdb.org/docs/sql/functions/struct)
