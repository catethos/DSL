# Language Features

## Core Syntax

The DSL provides a concise, expressive syntax for building AI-powered workflows.

## Data Types

### Primitive Types

#### String
Text data enclosed in quotes.

```javascript
"hello world"
'single quotes also work'
```

**Operations:**
```javascript
Upper("hello")      // "HELLO"
Lower("WORLD")      // "world"
Length("hello")     // 5
```

#### Int
Integer numbers (whole numbers).

```javascript
42
-10
0
1000
```

**Operations:**
```javascript
10 + 5      // 15
20 - 3      // 17
4 * 5       // 20
100 / 4     // 25
```

#### Float
Floating-point numbers (decimals).

```javascript
3.14
-0.5
2.71828
```

**Operations:**
```javascript
3.14 * 2        // 6.28
10.5 + 2.3      // 12.8
5.0 / 2.0       // 2.5
```

**Auto-promotion:**
```javascript
5 + 3.14        // 8.14 (Int → Float)
10 * 2.5        // 25.0 (Int → Float)
```

#### Bool
Boolean values (true/false).

```javascript
true
false
```

### Composite Types

#### List
Ordered collections of values.

```javascript
[1, 2, 3, 4, 5]
["apple", "banana", "cherry"]
[true, false, true]
[[1, 2], [3, 4]]        // Nested lists
```

**Indexing:**
```javascript
[10, 20, 30][0]         // 10 (first element)
[10, 20, 30][2]         // 30 (third element)
```

**Operations:**
```javascript
Length([1, 2, 3])       // 3
Join(["a", "b"], "-")   // "a-b"
```

#### Map
Key-value pairs (objects).

```javascript
{ "name": "Alice", "age": 30 }
{ "x": 10, "y": 20 }
```

**Field Access:**
```javascript
{ "name": "Alice" }.name        // "Alice"
user.email                      // Access field
user.address.city               // Nested access
```

### Special Types

#### Null
Represents absence of a value.

```javascript
null
```

#### Table
SQL query results (from DuckDB).

```javascript
SQL("SELECT * FROM 'data.csv'")     // Returns Table
```

---

## Variables and Binding

### Variable Binding with `as`

Bind expression results to variables using the `as` keyword.

**Syntax:**
```javascript
expression as variable_name
```

**Examples:**
```javascript
42 as answer
"hello" as greeting
[1, 2, 3] as numbers
```

**Rules:**
- Variables are immutable (cannot be reassigned)
- Variables persist in the session
- Variable names must start with letter or underscore
- Variables can be used in later expressions

### The Underscore `_` Variable

The special variable `_` always contains the last result.

**Examples:**
```javascript
flow> 100
✓ 100 : Int

flow> _
✓ 100 : Int

flow> _ + 50
✓ 150 : Int

flow> _ * 2
✓ 300 : Int
```

**Use Cases:**
- Quick calculations without naming
- Pipeline operations
- Referencing recent results

---

## Operators

### Arithmetic Operators

```javascript
+   // Addition
-   // Subtraction
*   // Multiplication
/   // Division
```

**Examples:**
```javascript
10 + 5          // 15
20 - 3          // 17
4 * 5           // 20
100 / 4         // 25

// With variables
5 as x
x + 10          // 15
x * 2           // 10
```

**Type Mixing:**
```javascript
5 + 3.14        // 8.14 (Int + Float = Float)
10 * 2.5        // 25.0 (Int * Float = Float)
```

### Sequential Operator `>>`

Chain operations together, passing the result forward.

**Syntax:**
```javascript
expr1 >> expr2 >> expr3
```

**The result of each expression flows to the next.**

**Examples:**
```javascript
// Simple chain
"hello" >> Upper(_)
// Result: "HELLO"

// Multi-step
"hello" >> Upper(_) >> Length(_)
// Result: 5

// With binding
["a", "b", "c"] >> Join(_, "-") as joined >> Upper(joined)
// Result: "A-B-C"

// Arithmetic
5 >> _ * 2 >> _ + 3
// Result: 13 (5 * 2 = 10, 10 + 3 = 13)
```

**How It Works:**
1. Evaluate `expr1`
2. Store result in `_`
3. Evaluate `expr2` (which can use `_`)
4. Store result in `_`
5. Continue...

### Parallel Operator `||`

Execute expressions concurrently and collect results.

**Syntax:**
```javascript
(expr1 || expr2 || expr3)
```

**Results are collected into a list.**

**Examples:**
```javascript
// Simple parallel
(5 || 10 || 15)
// Result: [5, 10, 15]

// With binding
(5 || 10 || 15) as numbers
// Result: numbers = [5, 10, 15]

// Destructuring
(10 || 20 || 30) as [a, b, c]
// Result: a=10, b=20, c=30

// Use destructured values
(10 || 20 || 30) as [a, b, c] >> a + b + c
// Result: 60
```

**Use Cases:**
- Parallel API calls
- Concurrent LLM queries
- Batch data fetching

### Conditional Operator `?:` (Future)

Branch based on a condition.

**Syntax:**
```javascript
condition ? true_expr : false_expr
```

**Example:**
```javascript
score > 0.8 ? "high" : "low"
```

**Note:** Not yet implemented in current version.

---

## Field and Index Access

### Field Access

Access fields in maps/objects using dot notation.

**Syntax:**
```javascript
object.field
object.nested.field
```

**Examples:**
```javascript
{ "name": "Alice", "age": 30 } as user
user.name           // "Alice"
user.age            // 30

// Nested
{ "address": { "city": "Boston" } } as data
data.address.city   // "Boston"
```

### Index Access

Access list elements by index (0-based).

**Syntax:**
```javascript
list[index]
```

**Examples:**
```javascript
[10, 20, 30, 40, 50] as numbers
numbers[0]          // 10 (first)
numbers[2]          // 30 (third)
numbers[4]          // 50 (last)
```

---

## Comments

### Single-Line Comments

```javascript
// This is a comment
42 as answer    // Comment after code
```

### Multi-Line Comments (Future)

```javascript
/*
  This is a
  multi-line comment
*/
```

**Note:** Not yet implemented in current version.

---

## Template Strings

Interpolate variables into strings using `${variable}`.

**Syntax:**
```javascript
"Text with ${variable}"
```

**Examples:**
```javascript
"Alice" as name
"Hello, ${name}!"
// Result: "Hello, Alice!"

42 as age
"You are ${age} years old"
// Result: "You are 42 years old"
```

**Use Cases:**
- Building prompts for LLMs
- Constructing URLs
- Dynamic messages

---

## Expressions

### Literal Expressions

```javascript
42              // Int literal
3.14            // Float literal
"hello"         // String literal
true            // Bool literal
[1, 2, 3]       // List literal
{"key": "val"}  // Map literal
```

### Variable Expressions

```javascript
answer          // Variable reference
_               // Last result
```

### Function Call Expressions

```javascript
Upper("hello")                  // Built-in function
Ask("What is AI?")              // LLM function
myFunction(arg1, arg2)          // User-defined function
```

### Arithmetic Expressions

```javascript
10 + 5
x * 2
(a + b) * c
```

### Sequential Expressions

```javascript
expr1 >> expr2 >> expr3
```

### Parallel Expressions

```javascript
(expr1 || expr2 || expr3)
```

### Binding Expressions

```javascript
expr as name
(expr1 || expr2) as [a, b]
```

---

## Parsing and Evaluation

### Order of Operations

1. **Parentheses** - `()`
2. **Field/Index Access** - `.`, `[]`
3. **Function Calls** - `func(args)`
4. **Arithmetic** - `*`, `/`, then `+`, `-`
5. **Parallel** - `||`
6. **Sequential** - `>>`
7. **Binding** - `as`

### Precedence Examples

```javascript
// Arithmetic first
10 + 5 * 2              // 20 (not 30)

// Function call first
Length("hello") + 1     // 6

// Parallel before sequential
5 || 10 >> _ * 2
// (5 || 10) = [5, 10]
// [5, 10] * 2 = error (can't multiply list)

// Use parentheses for clarity
(5 >> _ * 2) || (10 >> _ * 2)
// (5 * 2) || (10 * 2) = [10, 20]
```

---

## Scoping Rules

### Variable Scope

Variables persist in the current session.

```javascript
42 as x             // x is available
x + 10              // Can use x
[1, 2, 3] as y      // y is available
x + y[0]            // Can use both x and y
```

### The `_` Variable

`_` is always updated with the last result.

```javascript
10                  // _ = 10
"hello"             // _ = "hello"
[1, 2, 3]           // _ = [1, 2, 3]
```

---

## Type System Integration

### Type Annotations

Variables have runtime type information.

```javascript
flow> 42 as x
✓ Bound 'x' to 42 : Int

flow> :vars
✓ Variables:
  x = 42 : Int
```

### Type Coercion

Some types are automatically converted.

```javascript
5 + 3.14        // Int → Float, result: 8.14
10 * 2.5        // Int → Float, result: 25.0
```

---

## Error Handling

### Parse Errors

```javascript
flow> 42 +
✗ Parse error: Unexpected end of input
```

### Runtime Errors

```javascript
flow> unknown_var
✗ Error: Variable 'unknown_var' not defined

flow> 10 / 0
✗ Error: Division by zero
```

### Type Errors

```javascript
flow> "hello" + 42
✗ Error: Cannot add String and Int
```

---

## Best Practices

### 1. Use Meaningful Variable Names

**Good:**
```javascript
SQL("SELECT * FROM 'users.csv'") as users
```

**Bad:**
```javascript
SQL("SELECT * FROM 'users.csv'") as x
```

### 2. Break Complex Expressions

**Good:**
```javascript
data as raw
  >> Clean(raw) as cleaned
  >> Analyze(cleaned) as results
```

**Bad:**
```javascript
Analyze(Clean(data))
```

### 3. Use Comments

```javascript
// Fetch user data
SQL("SELECT * FROM 'users.csv'") as users

// Filter active users
SQL("SELECT * FROM users WHERE active = true") as active
```

### 4. Leverage Parallel Execution

**Good (parallel):**
```javascript
(getUser(1) || getUser(2) || getUser(3)) as users
```

**Slow (sequential):**
```javascript
getUser(1) as u1
getUser(2) as u2
getUser(3) as u3
```

### 5. Use the `_` Variable

**Quick calculations:**
```javascript
100 >> _ * 2 >> _ + 50      // 250
```

---

## Common Patterns

### Pattern 1: Data Pipeline

```javascript
SQL("SELECT * FROM 'data.csv'") as raw
  >> Clean(raw) as cleaned
  >> Transform(cleaned) as transformed
  >> Analyze(transformed) as results
```

### Pattern 2: Parallel Data Fetching

```javascript
(fetchA() || fetchB() || fetchC()) as [dataA, dataB, dataC]
  >> Combine(dataA, dataB, dataC) as combined
```

### Pattern 3: Iterative Refinement

```javascript
Draft(topic) as v1
  >> Review(v1) as feedback
  >> Revise(v1, feedback) as v2
  >> Finalize(v2) as final
```

### Pattern 4: Conditional Processing (future)

```javascript
Analyze(data) as score
  >> score > 0.8 ? ProcessHigh(data) : ProcessLow(data)
```

---

## Next Steps

- **[05-Type-System.md](05-Type-System.md)** - Define custom types
- **[06-Functions.md](06-Functions.md)** - Built-in and user-defined functions
- **[07-Workflow-Constructs.md](07-Workflow-Constructs.md)** - Advanced workflows
- **[examples/](../examples/)** - Try example code

---

## Related Documents

- **[DESIGN.md](../DESIGN.md)** - Complete language specification
- **[EXAMPLES.md](../EXAMPLES.md)** - Code examples
- **[README.md](../README.md)** - Project overview
