# Lattice Examples

This directory contains example programs demonstrating all features of the Lattice language.

## Running Examples

```bash
# Run a specific example
cargo run -p lattice-cli -- run example/basics.lat

# Start the REPL and load an example
cargo run -p lattice-cli -- repl
>>> :load example/functions.lat
```

## Examples Overview

### 1. `hello.lat` - Hello World
Simple introduction showing variable binding and arithmetic.

### 2. `basics.lat` - Literals, Arithmetic, Comparisons, Logic
- Integer, float, string, boolean, and null literals
- Arithmetic operators: `+`, `-`, `*`, `/`, `%`
- Comparison operators: `==`, `!=`, `<`, `<=`, `>`, `>=`
- Logical operators: `!`, `&&`, `||`
- Operator precedence and parentheses

### 3. `variables.lat` - Let Bindings and Assignments
- `let` bindings for variable declaration
- Optional type annotations: `let x: Int = 42`
- Reassignment: `x = x + 1`
- Complex assignment targets: `arr[0] = 10`, `obj.field = "value"`
- Variable scoping in blocks

### 4. `collections.lat` - Lists and Maps
- List literals: `[1, 2, 3]`
- Map literals: `{"key": "value"}`
- Indexing: `list[0]`, `map["key"]`
- Field access: `map.key`
- Nested collections
- Collection operations: `len()`, `push()`, `pop()`, `keys()`, `values()`
- Type annotations: `[Int]`, `Map<String, Int>`

### 5. `types.lat` - Type Definitions and Enums
- Type definitions: `type Person { name: String, age: Int }`
- Field descriptions for LLM context: `name: String @"Description"`
- Optional fields: `bio: String?`
- Enum definitions: `enum Color { Red, Green, Blue }`
- Struct literals: `Person { name: "Alice", age: 30 }`
- Nested types

### 6. `functions.lat` - Function Definitions and Calls
- Basic functions: `def add(a: Int, b: Int) -> Int { a + b }`
- Return types and explicit returns
- Multiple parameters
- Collection parameters and return values
- Higher-order functions
- Recursive functions
- Early return with `return`

### 7. `control_flow.lat` - Control Flow Statements
- If statements: `if cond { } else if cond { } else { }`
- If as expression (returns value)
- While loops: `while cond { }`
- For loops: `for item in collection { }`
- Match expressions: `match value { pattern => result, _ => default }`
- Pattern matching: literals, enums, wildcards, bindings

### 8. `lambdas.lat` - Anonymous Functions
- Basic syntax: `|x| x * 2`, `|a, b| a + b`
- No-parameter lambdas: `|| 42`
- Block body: `|x| { let y = x * 2; y + 1 }`
- Higher-order functions with lambdas
- Closures (capturing variables)
- Callback patterns

### 9. `llm.lat` - LLM Function Definitions
- LLM-backed functions with typed outputs
- Configuration: `base_url`, `model`, `api_key_env`, `temperature`, `max_tokens`
- Prompt templates with string interpolation: `${variable}`
- Use cases: sentiment analysis, entity extraction, summarization, classification, translation
- Different LLM providers

### 10. `fstrings.lat` - F-Strings (Interpolated Strings)
- Basic syntax: `f"Hello, {name}!"`
- Expressions inside braces: `f"{x} + {y} = {x + y}"`
- Automatic type coercion to string
- Escape braces: `{{` and `}}`
- Perfect for building dynamic SQL queries

### 11. `sql_and_parallel.lat` - SQL Queries and Parallel Execution
- SQL expressions: `SQL("SELECT ...")`
- Typed SQL results: `SQL<Type>("...")`
- Dynamic SQL with f-strings: `SQL(f"SELECT * FROM {table}")`
- Parallel blocks: `parallel { expr1, expr2, expr3 }`
- Parallel map: `parallel_map(items, |x| x * 2)`
- Combining SQL and parallel execution

## Language Features Quick Reference

```lattice
// Variables
let x = 42
let y: Int = 100
x = x + 1

// Collections
let list = [1, 2, 3]
let map = {"a": 1, "b": 2}

// Types
type Person { name: String, age: Int }
enum Color { Red, Green, Blue }

// Functions
def greet(name: String) -> String {
    "Hello, " + name + "!"
}

// Control Flow
if condition { } else { }
while condition { }
for item in list { }
match value { pattern => result }

// Lambdas
let double = |x| x * 2
let sum = |a, b| a + b

// F-Strings (Interpolated Strings)
let name = "Alice"
let msg = f"Hello, {name}!"
let calc = f"{x} + {y} = {x + y}"

// LLM Functions
def analyze(text: String) -> Sentiment {
    model: "gpt-4"
    api_key_env: "OPENAI_API_KEY"
    prompt: "Analyze: ${text}"
}

// SQL
let data = SQL<[Row]>("SELECT * FROM table")

// Parallel Execution
let results = parallel { task1(), task2(), task3() }
let mapped = parallel_map(items, |x| process(x))
```

## REPL Commands

```
:help       - Show help
:quit       - Exit REPL
:clear      - Clear VM state
:vars       - List variables
:types      - List defined types
:functions  - List functions
:load file  - Load and run a file
```
