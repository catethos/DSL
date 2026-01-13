# Lattice Python Library Usage Guide

This guide covers different ways to use the Lattice Python bindings.

## Table of Contents

- [Installation](#installation)
- [Basic Usage](#basic-usage)
- [Working with Functions](#working-with-functions)
- [Variables and Bindings](#variables-and-bindings)
- [Type Definitions](#type-definitions)
- [SQL Support](#sql-support)
- [LLM Functions](#llm-functions)
- [Introspection](#introspection)
- [Pythonic Features](#pythonic-features)
- [Error Handling](#error-handling)
- [Advanced Patterns](#advanced-patterns)

---

## Installation

### From PyPI

```bash
pip install lattice-lang
```

### From Wheel (Local Build)

```bash
pip install /path/to/lattice_lang-0.1.8-cp312-cp312-macosx_11_0_arm64.whl
```

### Development Install

```bash
cd crates/lattice-py
uv sync --dev
```

---

## Basic Usage

### Creating a Runtime

```python
from lattice import Runtime

# Basic runtime (no LLM, no SQL)
rt = Runtime()

# Runtime with SQL support
rt = Runtime(sql=True)

# Runtime with LLM support
rt = Runtime(llm=True)

# Runtime with both
rt = Runtime(llm=True, sql=True)
```

### Evaluating Expressions

```python
rt = Runtime()

# Arithmetic
result = rt.eval("1 + 2 * 3")  # 7

# Strings
result = rt.eval('"hello" + " world"')  # "hello world"

# Booleans
result = rt.eval("true && false")  # False
result = rt.eval("10 > 5")  # True

# Lists
result = rt.eval("[1, 2, 3, 4, 5]")  # [1, 2, 3, 4, 5]

# Maps/Dicts
result = rt.eval('{"name": "Alice", "age": 30}')  # {"name": "Alice", "age": 30}

# Null
result = rt.eval("null")  # None
```

### Multi-line Code

```python
rt = Runtime()

code = '''
let x = 10
let y = 20
x + y
'''
result = rt.eval(code)  # 30
```

---

## Working with Functions

### Defining and Calling Functions

```python
rt = Runtime()

# Define a function
rt.eval("def add(a: Int, b: Int) -> Int { a + b }")

# Call it from Python
result = rt.call("add", 3, 4)  # 7

# Define a more complex function
rt.eval('''
def greet(name: String, times: Int) -> String {
    let greeting = f"Hello, {name}!"
    greeting
}
''')

result = rt.call("greet", "World", 3)  # "Hello, World!"
```

### Checking Function Existence

```python
rt = Runtime()
rt.eval("def foo() -> Int { 42 }")

print(rt.has_function("foo"))  # True
print(rt.has_function("bar"))  # False
```

### Functions with Complex Types

```python
rt = Runtime()

# Function returning a list
rt.eval("def range_list(n: Int) -> [Int] { [1, 2, 3] }")
result = rt.call("range_list", 3)  # [1, 2, 3]

# Function returning a map
rt.eval('def make_person(name: String, age: Int) -> {String: Any} { {"name": name, "age": age} }')
result = rt.call("make_person", "Bob", 25)  # {"name": "Bob", "age": 25}
```

---

## Variables and Bindings

### Using Bindings in Eval

Pass Python values directly into Lattice expressions:

```python
rt = Runtime()

# Simple bindings
result = rt.eval("x + y", bindings={"x": 10, "y": 20})  # 30

# List bindings
result = rt.eval("items[0] + items[1]", bindings={"items": [3, 4]})  # 7

# Dict bindings
result = rt.eval('person["name"]', bindings={"person": {"name": "Alice", "age": 30}})  # "Alice"

# Mixed bindings
result = rt.eval(
    "name + \" is \" + age + \" years old\"",
    bindings={"name": "Bob", "age": "25"}
)  # "Bob is 25 years old"
```

### Global Variables

```python
rt = Runtime()

# Set globals using methods
rt.set_global("count", 42)
rt.set_global("name", "Alice")
rt.set_global("items", [1, 2, 3])

# Get globals
print(rt.get_global("count"))  # 42
print(rt.get_global("nonexistent"))  # None

# Use globals in eval
result = rt.eval("count * 2")  # 84
```

### Dict-style Access (Pythonic)

```python
rt = Runtime()

# Set using dict syntax
rt["x"] = 100
rt["data"] = {"key": "value"}

# Get using dict syntax
print(rt["x"])  # 100
print(rt["missing"])  # None

# Check existence
print("x" in rt)  # True
print("missing" in rt)  # False
```

---

## Type Definitions

### Structs

```python
rt = Runtime()

# Define a struct
rt.eval('''
type Person {
    name: String,
    age: Int,
    email: String?  // Optional field
}
''')

# Use the struct
result = rt.eval('''
Person {
    name: "Alice",
    age: 30,
    email: "alice@example.com"
}
''')
print(result)  # {"name": "Alice", "age": 30, "email": "alice@example.com"}
```

### Enums

```python
rt = Runtime()

# Define an enum
rt.eval('''
enum Status {
    Pending,
    Active,
    Completed,
    Failed
}
''')

# Use enum values
result = rt.eval("Status::Active")
print(result)  # "Active"
```

### Nested Types

```python
rt = Runtime()

rt.eval('''
type Address {
    street: String,
    city: String,
    country: String
}

type Company {
    name: String,
    address: Address,
    employees: [String]
}
''')

result = rt.eval('''
Company {
    name: "Acme Inc",
    address: Address {
        street: "123 Main St",
        city: "Springfield",
        country: "USA"
    },
    employees: ["Alice", "Bob", "Carol"]
}
''')
```

---

## SQL Support

Enable SQL support to query data using DuckDB:

```python
rt = Runtime(sql=True)

# Basic query
result = rt.eval('SQL("SELECT 1 + 1 as result")')
print(result)  # [{"result": 2}]

# Query with multiple rows
result = rt.eval('''
SQL("
    SELECT * FROM (
        SELECT 1 as id, 'Alice' as name
        UNION ALL
        SELECT 2 as id, 'Bob' as name
        UNION ALL
        SELECT 3 as id, 'Carol' as name
    )
")
''')
print(result)  # [{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}, {"id": 3, "name": "Carol"}]

# Aggregation
result = rt.eval('''
SQL("
    SELECT COUNT(*) as count, SUM(value) as total
    FROM (SELECT 10 as value UNION ALL SELECT 20 UNION ALL SELECT 30)
")
''')
print(result)  # [{"count": 3, "total": 60}]
```

### SQL with Interpolation

```python
rt = Runtime(sql=True)

result = rt.eval('''
let min_age = 25
let query = f"SELECT * FROM (
    SELECT 'Alice' as name, 30 as age
    UNION ALL SELECT 'Bob' as name, 20 as age
    UNION ALL SELECT 'Carol' as name, 35 as age
) WHERE age >= {min_age}"
SQL(query)
''')
# Returns only Alice and Carol
```

---

## LLM Functions

Define functions that call language models:

```python
import os
os.environ["OPENROUTER_API_KEY"] = "your-api-key"

rt = Runtime(llm=True)

# Define an LLM function
rt.eval('''
def summarize(text: String) -> String {
    base_url: "https://openrouter.ai/api/v1"
    model: "openai/gpt-4o-mini"
    api_key_env: "OPENROUTER_API_KEY"
    temperature: 0.0
    prompt: "Summarize the following text concisely: ${text}"
}
''')

# Call it
result = rt.call("summarize", "This is a long text that needs summarizing...")
print(result)
```

### Structured LLM Responses

```python
rt = Runtime(llm=True)

# LLM returning an enum
rt.eval('''
enum Sentiment { Positive, Negative, Neutral }

def analyze(text: String) -> Sentiment {
    base_url: "https://openrouter.ai/api/v1"
    model: "openai/gpt-4o-mini"
    api_key_env: "OPENROUTER_API_KEY"
    temperature: 0.0
    prompt: "Analyze the sentiment: ${text}"
}
''')

result = rt.call("analyze", "I love this product!")
print(result)  # "Positive"
```

### LLM Debug Information

```python
rt = Runtime(llm=True)

rt.eval('''
def echo(msg: String) -> String {
    base_url: "https://openrouter.ai/api/v1"
    model: "openai/gpt-4o-mini"
    api_key_env: "OPENROUTER_API_KEY"
    prompt: "Echo: ${msg}"
}
''')

result = rt.call("echo", "hello")

# Get debug info from the last LLM call
debug = rt.take_llm_debug()
if debug:
    print("Function:", debug["function_name"])
    print("Prompt:", debug["prompt"])
    print("Response:", debug["raw_response"])
    print("Return Type:", debug["return_type"])
```

---

## Introspection

### Get Type Schemas

```python
rt = Runtime()

rt.eval('''
type Person { name: String, age: Int }
enum Color { Red, Green, Blue }
''')

types = rt.get_types()
for t in types:
    print(f"Type: {t['name']}, Kind: {t['type']}")
    if t['type'] == 'struct':
        for field in t['fields']:
            print(f"  - {field['name']}: {field['type_schema']['type']}")
    elif t['type'] == 'enum':
        print(f"  Variants: {t['variants']}")
```

Output:
```
Type: Person, Kind: struct
  - name: string
  - age: int
Type: Color, Kind: enum
  Variants: ['Red', 'Green', 'Blue']
```

### Get Function Signatures

```python
rt = Runtime()

rt.eval('''
def add(a: Int, b: Int) -> Int { a + b }
def greet(name: String) -> String { f"Hello, {name}!" }
''')

sigs = rt.get_function_signatures()
for sig in sigs:
    params = ", ".join(f"{p['name']}: {p['type_schema']['type']}" for p in sig['params'])
    ret = sig['return_type']['type']
    llm_marker = " [LLM]" if sig['is_llm'] else ""
    print(f"def {sig['name']}({params}) -> {ret}{llm_marker}")
```

Output:
```
def add(a: int, b: int) -> int
def greet(name: string) -> string
```

---

## Pythonic Features

### Dict-style Access

```python
rt = Runtime()

# Set and get like a dict
rt["config"] = {"debug": True, "version": "1.0"}
print(rt["config"])  # {"debug": True, "version": "1.0"}

# Check membership
if "config" in rt:
    print("Config exists!")
```

### Context Manager

Use the runtime as a context manager for automatic cleanup:

```python
rt = Runtime()
rt["x"] = 42

with rt:
    print(rt["x"])  # 42
    rt.eval("def foo() -> Int { 1 }")

# After exiting, runtime is reset
print(rt["x"])  # None
print(rt.has_function("foo"))  # False
```

### REPL-friendly Repr

```python
rt = Runtime()
rt.eval("def add(a: Int, b: Int) -> Int { a + b }")
rt.eval("type Person { name: String }")

print(repr(rt))  # <Runtime: 1 functions, 0 LLM functions, 1 types>
```

---

## Error Handling

Lattice raises `RuntimeError` for evaluation errors:

```python
rt = Runtime()

# Syntax error
try:
    rt.eval("1 +")
except RuntimeError as e:
    print(f"Syntax error: {e}")

# Undefined variable
try:
    rt.eval("undefined_variable")
except RuntimeError as e:
    print(f"Undefined: {e}")

# Type error
try:
    rt.eval('"hello" + 42')
except RuntimeError as e:
    print(f"Type error: {e}")

# Undefined function
try:
    rt.call("nonexistent_function")
except RuntimeError as e:
    print(f"Function not found: {e}")
```

---

## Advanced Patterns

### Building a Configuration System

```python
rt = Runtime()

# Define configuration schema
rt.eval('''
type DatabaseConfig {
    host: String,
    port: Int,
    database: String,
    pool_size: Int?
}

type AppConfig {
    debug: Bool,
    db: DatabaseConfig
}
''')

# Load and validate configuration
config_data = {
    "debug": True,
    "db": {
        "host": "localhost",
        "port": 5432,
        "database": "myapp",
        "pool_size": 10
    }
}

rt["config"] = config_data

# Access config in Lattice
result = rt.eval("config.db.host")
print(result)  # "localhost"
```

### Data Processing Pipeline

```python
rt = Runtime(sql=True)

# Define transformations
rt.eval('''
def process_row(row: {String: Any}) -> {String: Any} {
    {
        "id": row["id"],
        "name": row["name"],
        "score": row["value"] * 10
    }
}
''')

# Get data via SQL
data = rt.eval('''
SQL("SELECT 1 as id, 'Alice' as name, 8 as value
     UNION ALL SELECT 2, 'Bob', 9")
''')

# Process in Python
processed = [rt.call("process_row", row) for row in data]
print(processed)
# [{"id": 1, "name": "Alice", "score": 80}, {"id": 2, "name": "Bob", "score": 90}]
```

### State Machine

```python
rt = Runtime()

rt.eval('''
enum State { Idle, Running, Paused, Stopped }

type Machine {
    state: State,
    counter: Int
}

def transition(m: Machine, action: String) -> Machine {
    match action {
        "start" => Machine { state: State::Running, counter: m.counter },
        "pause" => Machine { state: State::Paused, counter: m.counter },
        "stop" => Machine { state: State::Stopped, counter: 0 },
        "tick" => Machine { state: m.state, counter: m.counter + 1 },
        _ => m
    }
}
''')

# Initialize
rt["machine"] = {"state": "Idle", "counter": 0}

# Transitions
rt["machine"] = rt.eval('transition(machine, "start")')
rt["machine"] = rt.eval('transition(machine, "tick")')
rt["machine"] = rt.eval('transition(machine, "tick")')

print(rt["machine"])  # {"state": "Running", "counter": 2}
```

### Evaluating Files

```python
rt = Runtime()

# Evaluate a .lat file
result = rt.eval_file("/path/to/script.lat")

# The file can define functions that you call later
rt.eval_file("/path/to/utils.lat")
result = rt.call("utility_function", arg1, arg2)
```

---

## Type Conversion Reference

| Lattice Type | Python Type |
|--------------|-------------|
| `Null` | `None` |
| `Bool` | `bool` |
| `Int` | `int` |
| `Float` | `float` |
| `String` | `str` |
| `Path` | `pathlib.Path` |
| `[T]` (List) | `list` |
| `{K: V}` (Map) | `dict` |
| Struct | `dict` |
| Enum variant | `str` |
