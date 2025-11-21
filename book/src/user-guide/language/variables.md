# Variables and Binding

Variables in DSL store values that can be referenced later in your program. DSL supports two equivalent syntaxes for variable binding.

## Variable Binding Syntaxes

### Syntax 1: Using `as` (Pipeline Style)

```dsl
expression as variable_name
```

The `as` keyword binds the result of an expression to a variable name, flowing naturally in pipelines.

**Examples:**
```dsl
42 as answer
"hello" as greeting
[1, 2, 3] as numbers

(10 + 5) as sum
Upper("hello") as uppercase
```

### Syntax 2: Using `let` (Traditional Style)

```dsl
let variable_name = expression
```

The `let` keyword uses traditional variable declaration syntax familiar to many programming languages.

**Examples:**
```dsl
def main() {
    let answer = 42
    let greeting = "hello"
    let numbers = [1, 2, 3]

    let sum = 10 + 5
    let uppercase = Upper("hello")
}
main()
```

### Both Are Equivalent

```admonish note
Both `as` and `let` syntaxes compile to the same internal representation. Choose the style that feels most natural for your use case.
```

```dsl
def main() {
    42 as x
    let x = 42

    "hello" |> Upper as result

    let result = Upper("hello")
}
main()
```

## Variable Rules

### 1. Variables Are Immutable

Once a variable is bound, it cannot be reassigned:

```dsl
let x = 42
```

```admonish tip
Immutability prevents bugs and makes code easier to reason about. To "change" a value, create a new variable with a different name.
```

### 2. Variables Persist in Session

Variables remain available throughout your REPL session:

```dsl
flow> 42 as x
✓ Bound 'x' to 42 : Int

flow> x + 10
✓ 52 : Int

flow> let y = x * 2
✓ Bound 'y' to 84 : Int
```

### 3. Naming Rules

Variable names must follow these rules:
- Start with a letter or underscore (`_`)
- Can contain letters, numbers, and underscores
- Are case-sensitive
- Cannot be keywords (`let`, `as`, `if`, `else`, `fun`, etc.)

**Valid names:**
```dsl
def main() {
    let name = "Alice"
    let user_count = 10
    let _temp = 42
    let userID = 123
    let age2 = 25
}
main()
```

**Invalid names:**
```dsl
def main() {
    let 2age = 25
    let user-count = 10
    let let = 42
}
main()
```

## The Underscore `_` Variable

The special variable `_` always contains the **last evaluated result**.

### Basic Usage

```dsl
flow> 100
✓ 100 : Int

flow> _
✓ 100 : Int

flow> _ + 50
✓ 150 : Int

flow> _ * 2
✓ 300 : Int
```

### Use Cases

**1. Quick Calculations**
```dsl
flow> 42
flow> _ * 2
flow> _ + 10
flow> _ / 2
```

**2. Pipeline Operations**
```dsl
"hello" |> Upper |> _
```

**3. Referencing Recent Results**
```dsl
flow> Length("hello")
✓ 5 : Int

flow> "The length is ${_}"
✓ "The length is 5" : String
```

```admonish tip
The `_` variable is perfect for interactive exploration and quick experiments without naming intermediate values.
```

### When `_` Updates

The `_` variable is updated after every expression evaluation:

```dsl
flow> 10

flow> "hello"

flow> [1, 2, 3]
```

## List Destructuring

Both syntaxes support destructuring lists into multiple variables:

### With `as`

```dsl
[1, 2, 3] as [a, b, c]

par(10, 20, 30) as [x, y, z]
```

### With `let`

```dsl
def main() {
    let [a, b, c] = [1, 2, 3]

    let [x, y, z] = par(10, 20, 30)
}
main()
```

### Partial Destructuring

```admonish warning
Currently, partial destructuring (ignoring some elements) is not supported. You must provide a name for each element.
```

```dsl
```

## Variable Scope

### Global Scope

Variables defined in the REPL are in the global scope:

```dsl
flow> 42 as x
✓ Bound 'x' to 42 : Int

flow> x + 10
✓ 52 : Int
```

### Function Scope

Variables defined inside functions are local to that function:

```dsl
def compute(n) {
    let temp = n * 2
    temp + 1
}

compute(5)
```

See [Functions](functions.md) for detailed scoping rules.

### Shadowing

Inner scopes can shadow outer scope variables:

```dsl
let x = 100

def test() {
    let x = 10
    x * 2
}

test()
x
```

## Checking Variables

### List All Variables

Use the `:vars` command to see all variables in scope:

```dsl
flow> 42 as answer
flow> "hello" as greeting
flow> [1, 2, 3] as numbers

flow> :vars
✓ Variables:
  answer = 42 : Int
  greeting = "hello" : String
  numbers = [1, 2, 3] : List
  _ = [1, 2, 3] : List
```

### Global Variables Only

Use `:globals` to see only global scope:

```dsl
flow> :globals
Global variables:
  answer = 42 : Int
  greeting = "hello" : String
  numbers = [1, 2, 3] : List
```

### Scope Stack (Debug)

Use `:scopes` for detailed scope information:

```dsl
flow> :scopes
Scope Stack (1 scope):

Scope [0] (global):
  answer = 42 : Int
  greeting = "hello" : String
  numbers = [1, 2, 3] : List
```

## Type Information

Variables carry runtime type information:

```dsl
flow> 42 as x
✓ Bound 'x' to 42 : Int

flow> :vars
✓ Variables:
  x = 42 : Int
```

See [Data Types](data-types.md) for more on the type system.

## Common Patterns

### Pattern 1: Incremental Computation

```dsl
[1, 2, 3, 4, 5] as numbers
Sum(numbers) as total
total / Length(numbers) as average
```

### Pattern 2: Pipeline with Binding

```dsl
"hello world"
|> Upper as uppercase
|> Split(uppercase, " ") as words
|> Join(words, "-")
```

### Pattern 3: Parallel Destructuring

```dsl
par(
    Sum([1, 2, 3]),
    Sum([4, 5, 6]),
    Sum([7, 8, 9])
) as [sum1, sum2, sum3]

sum1 + sum2 + sum3
```

### Pattern 4: Template Strings with Variables

```dsl
"Alice" as name
30 as age
"NYC" as city

"${name} is ${age} years old and lives in ${city}"
```

## Best Practices

### 1. Use Descriptive Names

```admonish tip "Good"
let user_count = 10
let total_price = 99.99
let active_users = filter_active(users)
```

```admonish warning "Bad"
let x = 10
let t = 99.99
let d = filter_active(users)
```

### 2. Choose Appropriate Syntax

```dsl
def main() {
    data |> Clean as cleaned |> Transform as transformed

    let width = 800
    let height = 600
    let area = width * height
}
main()
```

### 3. Leverage the `_` Variable

```dsl
42
_ * 2
_ + 10

[1, 2, 3]
Length(_)
```

### 4. Group Related Bindings

```dsl
def main() {
    let host = "localhost"
    let port = 8080
    let timeout = 30

    let name = "Alice"
    let email = "alice@example.com"
    let age = 30
}
main()
```

## What's Next?

- [Functions](functions.md) - Define reusable code with variables
- [Control Flow](control-flow.md) - Use variables in conditionals
- [Operators](operators.md) - Operations on variables
- [Data Types](data-types.md) - Types of values variables can hold
