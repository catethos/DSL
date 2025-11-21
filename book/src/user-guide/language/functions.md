# Functions

Functions in DSL allow you to define reusable pieces of code. DSL supports both single-expression and multi-statement functions.

## Defining Functions

Functions are defined using the `def` keyword:

```dsl
def function_name(parameters) {
    body
}
```

### Single-Expression Functions

The simplest function form contains a single expression that is automatically returned:

```dsl
def double(x) { x * 2 }

def greet(name) { "Hello, ${name}!" }

def add(a, b) { a + b }
```

**Using single-expression functions:**
```dsl
flow> def double(x) { x * 2 }
✓ Defined function 'double'

flow> double(5)
✓ 10 : Int

flow> double(21)
✓ 42 : Int
```

### Multi-Statement Functions

Functions can contain multiple statements for complex logic:

```dsl
def analyze_number(n) {
    let squared = n * n
    let cubed = squared * n
    let sum = squared + cubed

    {
        original: n,
        squared: squared,
        cubed: cubed,
        sum: sum
    }
}
```

**Multi-statement rules:**
- Each statement is executed in sequence
- Statements can be `let` bindings or expressions
- The **final expression** is the return value
- Semicolons between statements are optional but recommended
- Intermediate variables are scoped to the function

**Example:**
```dsl
def classify_value(value) {
    let abs_val = value < 0 ? -value : value
    let category = abs_val < 10 ? "small" :
                   abs_val < 100 ? "medium" : "large"

    {
        value: value,
        absolute: abs_val,
        category: category
    }
}

classify_value(-42)
```

## Function Parameters

### Single Parameter

```dsl
def square(x) { x * x }

square(5)
```

### Multiple Parameters

```dsl
def multiply(a, b) { a * b }

multiply(3, 4)
```

### No Parameters

```dsl
def get_pi() { 3.14159 }

get_pi()
```

## Return Values

### Implicit Return

The last expression in a function is automatically returned:

```dsl
def add(a, b) {
    a + b
}
```

### Explicit Final Expression

```dsl
def compute(x) {
    let temp = x * 2
    let result = temp + 1
    result
}
```

### Returning Complex Values

```dsl
def make_user(name, age) {
    {
        name: name,
        age: age,
        is_adult: age >= 18
    }
}

make_user("Alice", 30)
```

## Function Scope

### Local Variables

Variables defined within functions are local to that function:

```dsl
def compute(x) {
    let temp = x * 2
    let result = temp + 1
    result
}

compute(5)
```

### Accessing Global Variables

Functions can access variables from outer scopes:

```dsl
let multiplier = 10

def scale(x) {
    x * multiplier
}

scale(5)
```

### Variable Shadowing

Inner scopes can shadow outer scope variables:

```dsl
let x = 100

def compute() {
    let x = 10
    x * 2
}

compute()
x
```

## Scope Stack Implementation

DSL uses an efficient scope stack for variable management:

### How It Works

1. **Global Scope** (scope\[0\]): Never removed, persists across REPL sessions
2. **Function Scopes**: Created when functions are called, removed when they return
3. **Variable Lookup**: Searches from innermost (current) to outermost (global) scope
4. **Variable Binding**: Always creates variables in the current scope

### Performance Benefits

```admonish tip
The scope stack provides O(1) scope creation/cleanup, making it efficient for recursive functions and deeply nested calls.
```

### Nested Scopes Example

```dsl
let global_var = 100

def outer(x) {
    let outer_var = x * 2

    def inner(y) {
        let inner_var = y + 1
        global_var + outer_var + inner_var
    }

    inner(5)
}

outer(10)
```

## Higher-Order Functions

Functions can accept other functions as parameters:

### Anonymous Functions

Use `fn` to create anonymous functions (lambdas):

```dsl
map([1, 2, 3], fn n => n * 2 end)

filter([1, 2, 3, 4, 5], fn n => n > 2 end)

reduce([1, 2, 3], 0, fn acc, n => acc + n end)
```

### Passing Named Functions

You can pass function names without calling them:

```dsl
def double(x) { x * 2 }

Map([1, 2, 3], double)
```

### Creating Higher-Order Functions

```dsl
def apply_twice(f, x) {
    let once = f(x)
    f(once)
}

def increment(n) { n + 1 }

apply_twice(increment, 5)
```

## Recursive Functions

Functions can call themselves:

```dsl
def factorial(n) {
    n == 0 ? 1 : n * factorial(n - 1)
}

factorial(5)
```

```admonish note
The scope stack makes recursive functions efficient by automatically managing local variables for each call.
```

### Tail Recursion Example

```dsl
def sum_list(lst, acc) {
    Length(lst) == 0 ? acc : sum_list(Rest(lst), acc + First(lst))
}

sum_list([1, 2, 3, 4, 5], 0)
```

Or use match for clearer pattern matching:

```dsl
def sum_list(lst, acc) {
    match lst {
        [] => acc
        _ => sum_list(Rest(lst), acc + First(lst))
    }
}
```

## Function Composition

### Using Pipelines

```dsl
def double(x) { x * 2 }
def increment(x) { x + 1 }
def square(x) { x * x }

5 |> double |> increment |> square
```

### Nested Calls

```dsl
square(increment(double(5)))
```

## Checking Defined Functions

### List All Functions

Use the `:funcs` command:

```dsl
flow> def double(x) { x * 2 }
flow> def triple(x) { x * 3 }

flow> :funcs
✓ User-defined functions:
  double(x)
  triple(x)
```

## Common Patterns

### Pattern 1: Data Transformation

```dsl
def process_text(text) {
    text
    |> Upper
    |> Split(" ")
    |> filter(_, fn w => Length(w end) > 3 end)
    |> Join(_, ", ")
}

process_text("hello world from dsl")
```

### Pattern 2: Validation

```dsl
def validate_age(age) {
    age < 0 ? "Age cannot be negative" :
    age > 150 ? "Age seems unrealistic" :
    "Valid age"
}

validate_age(25)
validate_age(-5)
```

### Pattern 3: Builder Pattern

```dsl
def create_user(name, age, email) {
    {
        name: name,
        age: age,
        email: email,
        is_adult: age >= 18,
        username: Lower(name)
    }
}

create_user("Alice", 30, "alice@example.com")
```

### Pattern 4: Currying (Simulated)

```dsl
def make_multiplier(factor) {
    fn x => x * factor end
}

let double = make_multiplier(2)
let triple = make_multiplier(3)

Map([1, 2, 3], double)
Map([1, 2, 3], triple)
```

## Best Practices

### 1. Use Descriptive Names

```admonish tip "Good"
def calculate_total_price(items, tax_rate) { ... }
def validate_email_format(email) { ... }
```

```admonish warning "Bad"
def calc(x, y) { ... }
def check(s) { ... }
```

### 2. Keep Functions Focused

```admonish tip
Each function should do one thing well. If a function is doing multiple unrelated tasks, split it into smaller functions.
```

### 3. Use Multi-Statement for Clarity

```dsl
def process(data) {
    Transform(Clean(Parse(data)))
}

def process(data) {
    let parsed = Parse(data)
    let cleaned = Clean(parsed)
    Transform(cleaned)
}
```

### 4. Document Complex Logic

```dsl
def compound_interest(principal, rate, times, years) {
    let r_over_n = rate / times
    let exponent = times * years
    principal * ((1 + r_over_n) ^ exponent)
}
```

### 5. Leverage Pipelines in Functions

```dsl
def process_users(users) {
    users
    |> filter(_, fn u => u.active end)
    |> map(_, fn u => u.name end)
    |> Sort(_)
}
```

## Common Mistakes

### Mistake 1: Forgetting to Return

```admonish warning "Wrong"
def compute(x) {
    let result = x * 2
    // Missing: result
}
// Returns null instead of result
```

```admonish tip "Correct"
def compute(x) {
    let result = x * 2
    result              // Explicitly return
}
```

### Mistake 2: Side Effects Confusion

```dsl
let counter = 0

def increment() {
    counter = counter + 1
}

```

### Mistake 3: Missing Parameters

```dsl
def greet(name) { "Hello, ${name}!" }

greet()
greet("Alice")
```

## Debug Commands

### View Function Scope

```dsl
flow> :scopes
Scope Stack (1 scope):

Scope [0] (global):
  double = <function>
  triple = <function>
```

### Check Global Functions

```dsl
flow> :globals
Global variables:
  double = <function>
  triple = <function>
  x = 42 : Int
```

## What's Next?

- [Control Flow](control-flow.md) - Use functions with conditionals
- [Built-in Functions](../builtins/README.md) - Explore 50+ built-in functions
- [Variables](variables.md) - Function parameters and scope
- [Operators](operators.md) - Use operators in functions
