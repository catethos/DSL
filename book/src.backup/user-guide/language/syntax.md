# Syntax Overview

The DSL provides a concise, expressive syntax for building AI-powered workflows.

## Basic Structure

### Expressions

Everything in DSL is an expression that evaluates to a value:

```dsl
42                  // Evaluates to Int
"hello"             // Evaluates to String
[1, 2, 3]           // Evaluates to List
```

### Statements

Programs are composed of statements:

```dsl
let x = 42          // Variable binding
x + 10              // Expression statement
```

## Core Concepts

### 1. Values Are First-Class

Every expression produces a value:

```dsl
flow> 42
✓ 42 : Int

flow> "hello" |> Upper
✓ "HELLO" : String
```

### 2. Variables Are Immutable

Once bound, variables cannot be reassigned:

```dsl
let x = 42
x = 50  # Error: cannot reassign (Note: comments not fully supported)
```

### 3. Type Information at Runtime

All values carry type information:

```dsl
flow> 42 as x
✓ Bound 'x' to 42 : Int
```

## Comments

```admonish warning "Comments Not Yet Fully Implemented"
Comments are partially parsed by the grammar but **not yet fully supported** in the current version. While `//` and `#` style comments may be parsed, they could cause unexpected behavior or errors. Avoid using comments in production code until this feature is completed.
```

### Single-Line Comments (Partial Support)

```dsl
// This is a single-line comment (may not work reliably)
# This is also a single-line comment (may not work reliably)
42 as answer
```

### Multi-Line Comments (Not Implemented)

```dsl
/*
  Multi-line comments are not yet implemented
  and will cause parser errors
*/
```

## Template Strings

Interpolate variables into strings using `${}`:

```dsl
let name = "Alice"
"Hello, ${name}!"
// Result: "Hello, Alice!"

let age = 42
"You are ${age} years old"
// Result: "You are 42 years old"
```

**Use Cases:**
- Building prompts for LLMs
- Constructing URLs
- Dynamic messages

## Expression Types

### Literal Expressions

```dsl
42              // Int literal
3.14            // Float literal
"hello"         // String literal
true            // Bool literal
[1, 2, 3]       // List literal
{"key": "val"}  // Map literal
null            // Null literal
```

### Variable Expressions

```dsl
answer          // Variable reference
_               // Last result
```

### Function Call Expressions

```dsl
Upper("hello")                  // Built-in function
Ask("What is AI?")              // LLM function
myFunction(arg1, arg2)          // User-defined function
```

### Arithmetic Expressions

```dsl
10 + 5
x * 2
(a + b) * c
```

### Pipeline Expressions

```dsl
"hello" |> Upper |> Length
```

### Parallel Expressions

```dsl
par(expr1, expr2, expr3)
```

### Binding Expressions

```dsl
// Using 'as'
42 as answer

// Using 'let'
let answer = 42
```

## Field and Index Access

### Field Access

Access fields in maps/objects using dot notation:

```dsl
let user = { "name": "Alice", "age": 30 }
user.name           // "Alice"
user.age            // 30

// Nested access
let data = { "address": { "city": "Boston" } }
data.address.city   // "Boston"
```

### Index Access

Access list elements by index (0-based):

```dsl
let numbers = [10, 20, 30, 40, 50]
numbers[0]          // 10 (first)
numbers[2]          // 30 (third)
numbers[4]          // 50 (last)
```

## Order of Operations

Operator precedence (highest to lowest):

1. **Parentheses** - `()`
2. **Field/Index Access** - `.`, `[]`
3. **Function Calls** - `func(args)`
4. **Arithmetic** - `*`, `/`, then `+`, `-`
5. **Comparison** - `==`, `!=`, `<`, `>`, `<=`, `>=`
6. **Logical** - `&&`, `||`, `!`
7. **Parallel** - `par()`
8. **Pipeline** - `|>`
9. **Binding** - `as`, `let`

### Precedence Examples

```dsl
// Arithmetic first
10 + 5 * 2              // 20 (not 30)

// Function call first
Length("hello") + 1     // 6

// Use parentheses for clarity
(10 + 5) * 2            // 30
```

## Best Practices

### 1. Use Meaningful Names

```admonish tip "Good"
SQL("SELECT * FROM 'users.csv'") as users
```

```admonish warning "Bad"
SQL("SELECT * FROM 'users.csv'") as x
```

### 2. Break Complex Expressions

```admonish tip "Good"
data
|> Clean
|> Transform
|> Analyze
```

```admonish warning "Bad"
Analyze(Transform(Clean(data)))
```

### 3. Use Comments

```dsl
// Fetch user data
SQL("SELECT * FROM 'users.csv'") as users

// Filter active users
SQL("SELECT * FROM users WHERE active = true") as active
```

### 4. Leverage Pipelines

```admonish tip "Readable"
"hello world"
|> Upper
|> Split(" ")
|> Join("-")
```

### 5. Add Type Context

```dsl
// Instead of generic names
let result = process(data)

// Use descriptive names with type hints
let processed_users = process(users)
```

## Common Patterns

### Pattern 1: Data Pipeline

```dsl
SQL("SELECT * FROM 'data.csv'")
|> Clean
|> Transform
|> Analyze
```

### Pattern 2: Parallel Fetching

```dsl
par(fetchA(), fetchB(), fetchC()) as [dataA, dataB, dataC]
```

### Pattern 3: Iterative Refinement

```dsl
Draft(topic)
|> Review
|> Revise
|> Finalize
```

## What's Next?

Now that you understand the syntax, explore:

- [Data Types](data-types.md) - Primitive and composite types
- [Operators](operators.md) - All operators in detail
- [Variables](variables.md) - Variable binding and scoping
- [Functions](functions.md) - Defining and calling functions
