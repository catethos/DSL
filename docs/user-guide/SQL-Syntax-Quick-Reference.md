# SQL $variable Syntax Quick Reference

## Map Syntax

Use `{...}` for map literals (NOT `#{...}`):

```javascript
// ✅ Correct
users = [{name: "Alice", age: 25}, {name: "Bob", age: 30}]

// ❌ Wrong
users = [#{name: "Alice", age: 25}]
```

## Variable Binding

Two ways to bind variables:

### Using `let` (recommended for readability)
```javascript
let users = [{name: "Alice", age: 25}]
let result = SQL("SELECT * FROM $users")
```

### Using arrow binding `as`
```javascript
[{name: "Alice", age: 25}] as users
SQL("SELECT * FROM $users") as result
```

## Complete Examples

### Example 1: Basic Query
```javascript
let users = [
    {name: "Alice", age: 25},
    {name: "Bob", age: 30}
]

SQL("SELECT * FROM $users WHERE age > 28")
```

### Example 2: Multiple Tables
```javascript
let users = [
    {id: 1, name: "Alice"},
    {id: 2, name: "Bob"}
]

let orders = [
    {user_id: 1, amount: 100},
    {user_id: 2, amount: 200}
]

SQL("""
  SELECT u.name, o.amount
  FROM $users u
  JOIN $orders o ON u.id = o.user_id
""")
```

### Example 3: With Refresh
```javascript
let scores = [{player: "Alice", score: 100}]
SQL("SELECT AVG(score) FROM $scores")  // avg = 100

let scores = [{player: "Alice", score: 100}, {player: "Bob", score: 200}]
refresh_table("scores")
SQL("SELECT AVG(score) FROM $scores")  // avg = 150
```

## Common Mistakes

### ❌ Using `#{...}` instead of `{...}`
```javascript
// Wrong
users = [#{name: "Alice"}]
```

### ❌ Forgetting to bind variable
```javascript
// Wrong - anonymous list can't be referenced
[{name: "Alice"}]
SQL("SELECT * FROM $users")  // Error: users not found
```

### ✅ Correct Patterns
```javascript
// Bind first
let users = [{name: "Alice"}]
SQL("SELECT * FROM $users")

// Or use as
[{name: "Alice"}] as users
SQL("SELECT * FROM $users")
```
