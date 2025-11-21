# Operators

DSL provides a rich set of operators for arithmetic, comparison, logical operations, composition, and data flow.

## Arithmetic Operators

Basic mathematical operations on numbers.

```dsl
+   // Addition
-   // Subtraction
*   // Multiplication
/   // Division
^   // Exponentiation
```

### Examples

```dsl
10 + 5          // 15
20 - 3          // 17
4 * 5           // 20
100 / 4         // 25
2 ^ 8           // 256
```

### With Variables

```dsl
let x = 5
x + 10          // 15
x * 2           // 10
x ^ 2           // 25
```

### Type Mixing

Int and Float can be mixed - Int is automatically promoted to Float:

```dsl
5 + 3.14        // 8.14 (Int + Float = Float)
10 * 2.5        // 25.0 (Int * Float = Float)
```

```admonish tip
Automatic type promotion means you don't need to manually cast integers to floats.
```

## Comparison Operators

Compare values and return boolean results.

```dsl
==  // Equal to
!=  // Not equal to
<   // Less than
>   // Greater than
<=  // Less than or equal to
>=  // Greater than or equal to
```

### Examples

```dsl
5 == 5          // true
5 != 3          // true
3 < 5           // true
10 > 5          // true
5 <= 5          // true
10 >= 5         // true
```

### With Variables

```dsl
let age = 18
age >= 18       // true
age < 21        // true
```

### String Comparison

```dsl
"hello" == "hello"      // true
"a" != "b"              // true
"apple" < "banana"      // true (lexicographic)
```

## Logical Operators

Combine and manipulate boolean values.

```dsl
&&  // Logical AND
||  // Logical OR
!   // Logical NOT
```

### Examples

```dsl
true && true        // true
true && false       // false
true || false       // true
false || false      // false
!true               // false
!false              // true
```

### Complex Conditions

```dsl
let age = 18
let verified = true

age >= 18 && verified           // true
age < 13 || age > 65            // false
!(age < 18)                     // true
```

### Short-Circuit Evaluation

```admonish note
`&&` and `||` use short-circuit evaluation: the second operand is only evaluated if needed.
```

```dsl
false && expensive_function()   // expensive_function() not called
true || expensive_function()    // expensive_function() not called
```

## Pipeline Operator `|>`

The pipeline operator chains operations left-to-right, passing results forward.

### Syntax

```dsl
expr1 |> expr2 |> expr3
```

The result of each expression automatically becomes available to the next.

### Basic Pipelines

```dsl
"hello" |> Upper
// Result: "HELLO"

"hello" |> Upper |> Length
// Result: 5

[1, 2, 3, 4, 5] |> Length
// Result: 5
```

### Multi-Step Pipelines

```dsl
"hello world"
|> Upper
|> Split(" ")
|> Join("-")
// Result: "HELLO-WORLD"
```

### Pipeline with Functions

```dsl
[1, 2, 3, 4, 5]
|> Map(_, fn n => n * 2 end)
|> Filter(_, fn n => n > 5 end)
|> Sum(_)
// Result: 24
```

### Pipeline with Binding

```dsl
["a", "b", "c"]
|> Join("-") as joined
|> Upper
// Result: "A-B-C"
```

```admonish tip "Why Use Pipelines?"
Pipelines make data transformations readable by showing the flow of data left-to-right, just like reading text.
```

## Conditional Operator `?:`

Branch based on a condition (ternary operator).

### Syntax

```dsl
condition ? true_expr : false_expr
```

### Examples

```dsl
// Simple conditional
18 >= 18 ? "adult" : "minor"
// Result: "adult"

// With variables
let age = 25
age >= 18 ? "adult" : "minor"
// Result: "adult"

// With logical operators
let active = true
let verified = true
active && verified ? "proceed" : "reject"
// Result: "proceed"
```

### In Pipelines

```dsl
let age = 30
age |> (age >= 18 ? "adult" : "minor")
// Result: "adult"
```

### Nested Conditionals

```dsl
let score = 85
score >= 90 ? "A" : (score >= 80 ? "B" : (score >= 70 ? "C" : "F"))
// Result: "B"
```

```admonish tip
For multiple conditions, consider using pattern matching (experimental feature) for better readability.
```

## Parallel Execution `par()`

Execute expressions concurrently and collect results.

### Syntax

```dsl
par(expr1, expr2, expr3)
```

Results are collected into a list.

### Basic Parallel

```dsl
par(5, 10, 15)
// Result: [5, 10, 15]

par(5, 10, 15) as numbers
// numbers = [5, 10, 15]
```

### Parallel Destructuring

```dsl
par(10, 20, 30) as [a, b, c]
// a = 10, b = 20, c = 30

a + b + c
// Result: 60
```

### Parallel Function Calls

```dsl
par(
    Sum([1, 2, 3]),
    Sum([4, 5, 6]),
    Sum([7, 8, 9])
) as results
// results = [6, 15, 24]

Sum(results)
// Result: 45
```

### Use Cases

- Parallel API calls
- Concurrent LLM queries
- Batch data fetching
- Independent computations

```admonish tip "Performance"
Use `par()` when operations are independent and I/O-bound for significant performance improvements.
```

## Field Access Operator `.`

Access fields in maps and custom types.

### Syntax

```dsl
object.field
object.nested.field
```

### Examples

```dsl
let user = { "name": "Alice", "age": 30 }
user.name           // "Alice"
user.age            // 30

// Nested access
let data = { "address": { "city": "Boston" } }
data.address.city   // "Boston"
```

### With Custom Types

```dsl
class Person {
    name: String
    age: Int
}

let alice = Person { name: "Alice", age: 30 }
alice.name          // "Alice"
alice.age           // 30
```

## Index Access Operator `[]`

Access list elements by index (0-based).

### Syntax

```dsl
list[index]
```

### Examples

```dsl
let numbers = [10, 20, 30, 40, 50]
numbers[0]          // 10 (first)
numbers[2]          // 30 (third)
numbers[4]          // 50 (last)
```

### Dynamic Indexing

```dsl
let i = 2
numbers[i]          // 30
```

## Operator Precedence

From highest to lowest priority:

1. **Parentheses** - `()`
2. **Field/Index Access** - `.`, `[]`
3. **Function Calls** - `func(args)`
4. **Exponentiation** - `^`
5. **Multiplication/Division** - `*`, `/`
6. **Addition/Subtraction** - `+`, `-`
7. **Comparison** - `==`, `!=`, `<`, `>`, `<=`, `>=`
8. **Logical NOT** - `!`
9. **Logical AND** - `&&`
10. **Logical OR** - `||`
11. **Conditional** - `?:`
12. **Parallel** - `par()`
13. **Pipeline** - `|>`
14. **Binding** - `as`, `let`

### Precedence Examples

```dsl
// Multiplication before addition
10 + 5 * 2          // 20 (not 30)
(10 + 5) * 2        // 30

// Exponentiation before multiplication
2 ^ 3 * 4           // 32 (2^3 = 8, 8*4 = 32)
2 ^ (3 * 4)         // 4096 (3*4 = 12, 2^12 = 4096)

// Comparison before logical
5 > 3 && 10 < 20    // true && true = true

// Use parentheses for clarity
(5 > 3) && (10 < 20)    // Same, but clearer
```

```admonish tip
When in doubt, use parentheses to make your intent explicit.
```

## Operator Combinations

### Arithmetic + Comparison

```dsl
let x = 10
let y = 20
(x + y) > 25        // true (30 > 25)
```

### Pipeline + Conditional

```dsl
[1, 2, 3, 4, 5]
|> Length
|> (_ > 3 ? "many" : "few")
// Result: "many"
```

### Parallel + Pipeline

```dsl
par(
    [1, 2, 3] |> Sum,
    [4, 5, 6] |> Sum,
    [7, 8, 9] |> Sum
)
// Result: [6, 15, 24]
```

## Common Operator Patterns

### Pattern 1: Data Transformation Pipeline

```dsl
"hello world"
|> Upper(_)
|> Split(_, " ")
|> Map(_, fn w => w + "!" end)
|> Join(_, " ")
// Result: "HELLO! WORLD!"
```

### Pattern 2: Conditional Processing

```dsl
let score = 85
let grade = score >= 90 ? "A" :
            score >= 80 ? "B" :
            score >= 70 ? "C" :
            "F"
// Result: "B"
```

### Pattern 3: Parallel + Aggregate

```dsl
par(
    fetchUserData(1),
    fetchUserData(2),
    fetchUserData(3)
) as users
|> Map(_, fn u => u.age end)
|> Avg(_)
```

### Pattern 4: Complex Logic

```dsl
let age = 25
let verified = true
let premium = false

let access = (age >= 18 && verified) || premium ?
    "granted" : "denied"
// Result: "granted"
```

## Operator Cheat Sheet

| Operator | Type | Example | Result |
|----------|------|---------|--------|
| `+` | Arithmetic | `5 + 3` | `8` |
| `-` | Arithmetic | `10 - 4` | `6` |
| `*` | Arithmetic | `3 * 4` | `12` |
| `/` | Arithmetic | `15 / 3` | `5` |
| `^` | Arithmetic | `2 ^ 3` | `8` |
| `==` | Comparison | `5 == 5` | `true` |
| `!=` | Comparison | `5 != 3` | `true` |
| `<` | Comparison | `3 < 5` | `true` |
| `>` | Comparison | `10 > 5` | `true` |
| `<=` | Comparison | `5 <= 5` | `true` |
| `>=` | Comparison | `10 >= 5` | `true` |
| `&&` | Logical | `true && false` | `false` |
| `\|\|` | Logical | `true \|\| false` | `true` |
| `!` | Logical | `!true` | `false` |
| `?:` | Conditional | `true ? "yes" : "no"` | `"yes"` |
| `\|>` | Pipeline | `5 \|> _ * 2` | `10` |
| `par()` | Parallel | `par(1, 2, 3)` | `[1, 2, 3]` |
| `.` | Field Access | `user.name` | (field value) |
| `[]` | Index Access | `list[0]` | (element value) |

## What's Next?

- [Variables](variables.md) - Variable binding and scoping
- [Functions](functions.md) - Using operators in functions
- [Control Flow](control-flow.md) - Conditional logic and branching
- [Workflows](../workflows/sequential.md) - Advanced operator patterns
