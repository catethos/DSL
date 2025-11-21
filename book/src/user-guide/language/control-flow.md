# Control Flow

Control flow in DSL allows you to make decisions, branch execution, and handle different cases in your programs.

## Conditional Expressions

### The Ternary Operator `?:`

The most common way to branch in DSL is using the ternary conditional operator:

```dsl
condition ? true_expression : false_expression
```

**Basic Example:**
```dsl
let age = 25
age >= 18 ? "adult" : "minor"
```

### Chained Conditionals

For multiple conditions, chain ternary operators:

```dsl
let score = 85

score >= 90 ? "A" :
score >= 80 ? "B" :
score >= 70 ? "C" :
score >= 60 ? "D" : "F"
```

### Conditionals with Variables

```dsl
def main() {
    let temperature = 75

    let weather = temperature > 80 ? "hot" :
                  temperature > 60 ? "warm" :
                  temperature > 40 ? "cool" : "cold"

    weather
}
main()
```

### In Pipelines

Conditionals work seamlessly in pipelines:

```dsl
[1, 2, 3, 4, 5]
|> Length
|> (_ > 3 ? "many items" : "few items")
```

## Multi-Statement Conditional Branches

For conditionals that need multiple statements, use blocks within the ternary operator:

```dsl
def main() {
    let x = 10

    x > 5 ? {
        let doubled = x * 2
        let message = "Greater than 5: ${doubled}"
        message
    } : {
        let message = "Not greater than 5"
        message
    }
}
main()
```

### In Functions

Functions can use ternary operators for conditional logic:

```dsl
def classify_age(age) {
    age < 13 ? "child" :
    age < 20 ? "teenager" :
    age < 65 ? "adult" : "senior"
}

classify_age(25)
```

Or use match expressions for clearer multi-case handling:

```dsl
def classify_age(age) {
    match age {
        a if a < 13 => "child"
        a if a < 20 => "teenager"
        a if a < 65 => "adult"
        _ => "senior"
    }
}
```

## Logical Conditions

### AND Operator `&&`

Both conditions must be true:

```dsl
def main() {
    let age = 25
    let verified = true

    age >= 18 && verified ? "access granted" : "access denied"
}
main()
```

### OR Operator `||`

At least one condition must be true:

```dsl
def main() {
    let is_admin = false
    let is_owner = true

    is_admin || is_owner ? "can edit" : "read only"
}
main()
```

### NOT Operator `!`

Negates a boolean value:

```dsl
let is_logged_out = false

!is_logged_out ? "welcome back" : "please log in"
```

### Complex Conditions

```dsl
def main() {
    let age = 25
    let has_license = true
    let has_car = false

    age >= 16 && has_license && has_car ? "can drive own car" :
    age >= 16 && has_license ? "can drive but needs car" :
    "cannot drive"
}
main()
```

## Pattern Matching (Experimental)

```admonish warning
Pattern matching is an experimental feature (60% complete) and may have limitations.
```

### Match Expressions

```dsl
match value {
    pattern1 => result1
    pattern2 => result2
    _ => default_result
}
```

### Matching Literals

```dsl
let status_code = 200

match status_code {
    200 => "OK"
    404 => "Not Found"
    500 => "Server Error"
    _ => "Unknown Status"
}
```

### Matching with Variables

```dsl
match numbers {
    [] => "empty list"
    [x] => "single item: ${x}"
    [x, y] => "two items: ${x} and ${y}"
    _ => "many items"
}
```

### Guards

Add conditions to patterns:

```dsl
match x {
    n if n < 0 => "negative"
    n if n > 0 => "positive"
    _ => "zero"
}
```

## Conditional Returns in Functions

Use ternary operators for conditional logic in functions:

```dsl
def divide(a, b) {
    b == 0 ? "Error: division by zero" : a / b
}

divide(10, 2)
divide(10, 0)
```

### Validation Pattern

```dsl
def process_user(user) {
    user == null ? "Error: user is null" :
    !Contains(user.email, "@") ? "Error: invalid email" :
    user.age < 0 ? "Error: invalid age" :
    "User is valid"
}
```

Or use match expressions with guards for clearer validation:

```dsl
def process_user(user) {
    match user {
        null => "Error: user is null"
        u if !Contains(u.email, "@") => "Error: invalid email"
        u if u.age < 0 => "Error: invalid age"
        _ => "User is valid"
    }
}
```

## Conditional Data Processing

### Filtering with Conditions

```dsl
let numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

filter(numbers, fn n => n % 2 == 0 end)

filter(numbers, fn n => n > 5 end)
```

### Conditional Mapping

```dsl
let numbers = [1, 2, 3, 4, 5]

map(numbers, fn n => n % 2 == 0 ? n * 2 : n end)
```

### Conditional Aggregation

```dsl
let numbers = [1, 2, 3, 4, 5]

reduce(numbers, 0, fn acc, n => n % 2 == 0 ? acc + n : acc end)
```

## Short-Circuit Evaluation

Logical operators use short-circuit evaluation:

```dsl
false && expensive_operation()

true || expensive_operation()
```

**Example:**
```dsl
let user = null

user != null && user.age > 18
```

## Common Patterns

### Pattern 1: Null Checking

```dsl
let value = get_user()

value != null ? process(value) : "No user found"
```

### Pattern 2: Range Checking

```dsl
let value = 42

value >= 0 && value <= 100 ? "in range" : "out of range"
```

### Pattern 3: Type-Based Logic

```dsl
def process_value(val) {
    typeof(val) == "Int" ? val * 2 :
    typeof(val) == "String" ? Upper(val) :
    val
}
```

Or use match for clearer type handling:

```dsl
def process_value(val) {
    match val {
        v if typeof(v) == "Int" => v * 2
        v if typeof(v) == "String" => Upper(v)
        _ => val
    }
}
```

### Pattern 4: State Machine

```dsl
def handle_state(state, event) {
    state == "idle" && event == "start" ? "running" :
    state == "running" && event == "pause" ? "paused" :
    state == "paused" && event == "resume" ? "running" :
    state == "running" && event == "stop" ? "idle" :
    state
}
```

Match expressions work well for state machines:

```dsl
def handle_state(state, event) {
    match [state, event] {
        ["idle", "start"] => "running"
        ["running", "pause"] => "paused"
        ["paused", "resume"] => "running"
        ["running", "stop"] => "idle"
        _ => state
    }
}
```

### Pattern 5: Default Values

```dsl
def main() {
    let config = load_config()

    let timeout = config != null && config.timeout != null ?
        config.timeout : 30
}
main()
```

## Best Practices

### 1. Prefer Ternary for Simple Cases

```admonish tip "Simple conditional"
age >= 18 ? "adult" : "minor"
```

### 2. Use Blocks for Multi-Statement Logic

```admonish tip "Multi-line logic"
score >= 90 ? {
    let grade = "A"
    let message = "Excellent work!"
    { grade: grade, message: message }
} : {
    let grade = "B"
    let message = "Good job!"
    { grade: grade, message: message }
}
```

### 3. Use Match Expressions for Complex Cases

```admonish tip "Match for clarity"
match score {
    s if s >= 90 => {
        let message = "Excellent work!"
        { grade: "A", message: message }
    }
    s if s >= 80 => {
        let message = "Good job!"
        { grade: "B", message: message }
    }
    _ => { grade: "C", message: "Keep trying!" }
}
```

### 4. Avoid Deep Nesting

```admonish warning "Too nested"
a ? (b ? (c ? result : default) : other) : fallback
```

```admonish tip "Flatter structure"
!a ? fallback :
!b ? other :
!c ? default :
result
```

### 5. Use Match with Guards for Clarity

```dsl
def process(x) {
    x < 0 ? "negative" : (x == 0 ? "zero" : "positive")
}

def process(x) {
    match x {
        n if n < 0 => "negative"
        n if n == 0 => "zero"
        _ => "positive"
    }
}
```

### 6. Leverage Short-Circuit for Safety

```dsl
user != null && user.email != null && Contains(user.email, "@")

user.email != null
```

## Common Mistakes

### Mistake 1: Forgetting Else Branch

```admonish warning "Incomplete"
let result = x > 0 ? "positive"
// What if x <= 0? Result could be undefined
```

```admonish tip "Complete"
let result = x > 0 ? "positive" : "non-positive"
```

### Mistake 2: Comparing with = Instead of ==

```dsl
x = 5 ? "five" : "other"

x == 5 ? "five" : "other"
```

### Mistake 3: Not Handling Null

```dsl
def main() {
    let name = user.name

    let name = user != null ? user.name : "Unknown"
}
main()
```

## What's Next?

- [Functions](functions.md) - Use control flow in functions
- [Operators](operators.md) - Comparison and logical operators
- [Variables](variables.md) - Store conditional results
- [Built-in Functions](../builtins/README.md) - Filter, Map with conditions
