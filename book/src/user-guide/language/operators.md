# Operators

DSL provides a rich set of operators for arithmetic, comparison, logical operations, composition, and data flow.

## Arithmetic Operators

Basic mathematical operations on numbers.

```dsl
+
-
*
/
^
```

### Examples

```dsl
10 + 5
20 - 3
4 * 5
100 / 4
2 ^ 8
```

### With Variables

```dsl
let x = 5
x + 10
x * 2
x ^ 2
```

### Type Mixing

Int and Float can be mixed - Int is automatically promoted to Float:

```dsl
5 + 3.14
10 * 2.5
```

```admonish tip
Automatic type promotion means you don't need to manually cast integers to floats.
```

## Comparison Operators

Compare values and return boolean results.

```dsl
==
!=
<
>
<=
>=
```

### Examples

```dsl
5 == 5
5 != 3
3 < 5
10 > 5
5 <= 5
10 >= 5
```

### With Variables

```dsl
let age = 18
age >= 18
age < 21
```

### String Comparison

```dsl
"hello" == "hello"
"a" != "b"
"apple" < "banana"
```

## Logical Operators

Combine and manipulate boolean values.

```dsl
&&
||
!
```

### Examples

```dsl
true && true
true && false
true || false
false || false
!true
!false
```

### Complex Conditions

```dsl
def main() {
    let age = 18
    let verified = true

    age >= 18 && verified
    age < 13 || age > 65
    !(age < 18)
}
main()
```

### Short-Circuit Evaluation

```admonish note
`&&` and `||` use short-circuit evaluation: the second operand is only evaluated if needed.
```

```dsl
false && expensive_function()
true || expensive_function()
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

"hello" |> Upper |> Length

[1, 2, 3, 4, 5] |> Length
```

### Multi-Step Pipelines

```dsl
"hello world"
|> Upper
|> Split(" ")
|> Join("-")
```

### Pipeline with Functions

```dsl
[1, 2, 3, 4, 5]
|> map(_, fn n => n * 2 end)
|> filter(_, fn n => n > 5 end)
|> Sum(_)
```

### Pipeline with Binding

```dsl
["a", "b", "c"]
|> Join("-") as joined
|> Upper
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
def main() {
    18 >= 18 ? "adult" : "minor"

    let age = 25
    age >= 18 ? "adult" : "minor"

    let active = true
    let verified = true
    active && verified ? "proceed" : "reject"
}
main()
```

### In Pipelines

```dsl
let age = 30
age |> (age >= 18 ? "adult" : "minor")
```

### Nested Conditionals

```dsl
let score = 85
score >= 90 ? "A" : (score >= 80 ? "B" : (score >= 70 ? "C" : "F"))
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

par(5, 10, 15) as numbers
```

### Parallel Destructuring

```dsl
par(10, 20, 30) as [a, b, c]

a + b + c
```

### Parallel Function Calls

```dsl
par(
    Sum([1, 2, 3]),
    Sum([4, 5, 6]),
    Sum([7, 8, 9])
) as results

Sum(results)
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
def main() {
    let user = { "name": "Alice", "age": 30 }
    user.name
    user.age

    let data = { "address": { "city": "Boston" } }
    data.address.city
}
main()
```

### With Custom Types

```dsl
class Person {
    name: String
    age: Int
}

let alice = Person { name: "Alice", age: 30 }
alice.name
alice.age
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
numbers[0]
numbers[2]
numbers[4]
```

### Dynamic Indexing

```dsl
let i = 2
numbers[i]
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
10 + 5 * 2
(10 + 5) * 2

2 ^ 3 * 4
2 ^ (3 * 4)

5 > 3 && 10 < 20

(5 > 3) && (10 < 20)
```

```admonish tip
When in doubt, use parentheses to make your intent explicit.
```

## Operator Combinations

### Arithmetic + Comparison

```dsl
def main() {
    let x = 10
    let y = 20
    (x + y) > 25
}
main()
```

### Pipeline + Conditional

```dsl
[1, 2, 3, 4, 5]
|> Length
|> (_ > 3 ? "many" : "few")
```

### Parallel + Pipeline

```dsl
par(
    [1, 2, 3] |> Sum,
    [4, 5, 6] |> Sum,
    [7, 8, 9] |> Sum
)
```

## Common Operator Patterns

### Pattern 1: Data Transformation Pipeline

```dsl
"hello world"
|> Upper(_)
|> Split(_, " ")
|> map(_, fn w => w + "!" end)
|> Join(_, " ")
```

### Pattern 2: Conditional Processing

```dsl
def main() {
    let score = 85
    let grade = score >= 90 ? "A" :
                score >= 80 ? "B" :
                score >= 70 ? "C" :
                "F"
}
main()
```

### Pattern 3: Parallel + Aggregate

```dsl
par(
    fetchUserData(1),
    fetchUserData(2),
    fetchUserData(3)
) as users
|> map(_, fn u => u.age end)
|> Avg(_)
```

### Pattern 4: Complex Logic

```dsl
def main() {
    let age = 25
    let verified = true
    let premium = false

    let access = (age >= 18 && verified) || premium ?
        "granted" : "denied"
}
main()
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
