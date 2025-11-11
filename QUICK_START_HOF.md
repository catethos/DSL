# Quick Start: Higher-Order Functions

## Syntax

```dsl
// Lambda syntax
fn param => expression end
fn x, y => expression end  // Multi-param (for reduce)

// Map: transform each element
map(list, fn x => x * 2 end)
map(list, "functionName")

// Filter: keep elements matching predicate
filter(list, fn x => x > 5 end)
filter(list, "predicateFunction")
```

## Examples

### Basic Operations

```dsl
// Double all numbers
map([1, 2, 3], fn x => x * 2 end)
// Result: [2, 4, 6]

// Keep only large numbers
filter([1, 2, 3, 4, 5], fn x => x > 3 end)
// Result: [4, 5]
```

### Chaining with Pipes

```dsl
[1, 2, 3, 4, 5, 6]
  |> filter(_, fn x => x > 2 end)
  |> map(_, fn x => x * 2 end)
// Result: [6, 8, 10, 12]
```

### Working with Objects

```dsl
// Extract names
map([{name: "Alice", age: 30}, {name: "Bob", age: 25}], fn x => x.name end)
// Result: ["Alice", "Bob"]

// Filter by age
filter([{name: "Alice", age: 30}, {name: "Bob", age: 25}], fn x => x.age > 28 end)
// Result: [{name: "Alice", age: 30}]

// Pipeline
[{name: "Alice", score: 85}, {name: "Bob", score: 92}, {name: "Charlie", score: 78}]
  |> filter(_, fn x => x.score > 80 end)
  |> map(_, fn x => x.name end)
// Result: ["Alice", "Bob"]
```

### String References

```dsl
// Define functions
def double(x) { x * 2 }
def isEven(x) { x > 0 }

// Use by name
map([1, 2, 3], "double")
filter([1, 2, 3, 4, 5], "isEven")
```

### Complex Expressions

```dsl
// Arithmetic
map([1, 2, 3], fn x => x * x + 1 end)
// Result: [2, 5, 10]

// Compound conditions
filter([1, 2, 3, 4, 5, 6], fn x => x > 2 && x < 6 end)
// Result: [3, 4, 5]

// OR conditions
filter([1, 2, 3, 4, 5, 6, 7, 8, 9, 10], fn x => x < 3 || x > 8 end)
// Result: [1, 2, 9, 10]

// Field arithmetic
map([{x: 1, y: 2}, {x: 3, y: 4}], fn m => m.x + m.y end)
// Result: [3, 7]
```

## Try It Yourself

```bash
# Start REPL
cargo run --bin dsl

# Or run from stdin
echo "map([1,2,3], fn x => x * 2 end)" | cargo run --bin dsl -- -c
```

## Common Patterns

### Data Transformation Pipeline
```dsl
raw_data
  |> filter(_, fn x => x.valid end)          // Clean data
  |> map(_, fn x => x.value * 100 end)       // Scale
  |> filter(_, fn x => x > 50 end)           // Threshold
```

### Extract and Filter
```dsl
users
  |> filter(_, fn u => u.active end)
  |> map(_, fn u => u.email end)
```

### Multi-step Transformation
```dsl
[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
  |> filter(_, fn x => x > 4 end)            // [5, 6, 7, 8, 9, 10]
  |> map(_, fn x => x * x end)               // [25, 36, 49, 64, 81, 100]
  |> filter(_, fn x => x > 30 end)           // [36, 49, 64, 81, 100]
```

## Operators Supported

**Arithmetic:** `+`, `-`, `*`, `/`  
**Comparison:** `>`, `<`, `>=`, `<=`, `==`, `!=`  
**Logical:** `&&`, `||`, `!`  
**Field access:** `.field`

## Current Limitations

- No modulo operator `%` yet
- `if-then-else` not yet supported in lambda body
- Map literals `{key: value}` not yet supported in lambda body
- Pattern-matched functions can't be used as string references yet

## Coming Soon (Phase 2)

- `reduce(list, init, fn acc, x => acc + x end)` - Fold/aggregate
- `sortby(list, fn x => x.field end)` - Sort by computed key
- `groupby(list, fn x => x.category end)` - Group by key
