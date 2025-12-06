# Lattice Elixir Example

This is an example Elixir project demonstrating how to embed the Lattice runtime
using Rustler NIFs.

## Prerequisites

- Elixir 1.18+
- Erlang/OTP 28+
- Rust (stable)
- Cargo

## Setup

1. Install dependencies:

```bash
mix deps.get
```

2. Compile the project (this will also compile the Rust NIF):

```bash
mix compile
```

## Quick Start

```elixir
# Create a runtime
{:ok, rt} = Lattice.new()

# Evaluate expressions
{:ok, 6} = Lattice.eval(rt, "1 + 2 + 3")
{:ok, "hello world"} = Lattice.eval(rt, ~s["hello" + " " + "world"])

# Define and call functions
Lattice.eval(rt, """
  def greet(name: String) -> String {
    "Hello, " + name + "!"
  }
""")
{:ok, "Hello, World!"} = Lattice.call(rt, "greet", ["World"])

# Define types
Lattice.eval(rt, "type Person { name: String, age: Int }")
{:ok, person} = Lattice.eval(rt, ~s[Person { name: "Alice", age: 30 }])
# => %{"__type" => "Person", "name" => "Alice", "age" => 30}

# Pass values from Elixir to Lattice
{:ok, 30} = Lattice.eval_with_bindings(rt, "x + y", [{"x", 10}, {"y", 20}])
```

## Usage

### Interactive (IEx)

Start an IEx session:

```bash
iex -S mix
```

Then try:

```elixir
# Create a runtime
{:ok, rt} = Lattice.new()

# Evaluate expressions
{:ok, result} = Lattice.eval(rt, "1 + 2 + 3")
# => {:ok, 6}

# String operations
{:ok, result} = Lattice.eval(rt, ~s["hello" + " " + "world"])
# => {:ok, "hello world"}

# List operations
{:ok, result} = Lattice.eval(rt, "[1, 2, 3, 4, 5]")
# => {:ok, [1, 2, 3, 4, 5]}

{:ok, result} = Lattice.eval(rt, "[10, 20, 30][1]")
# => {:ok, 20}

{:ok, result} = Lattice.eval(rt, "len([1, 2, 3, 4])")
# => {:ok, 4}

# Map operations
{:ok, result} = Lattice.eval(rt, ~s[{"name": "Alice", "age": 30}])
# => {:ok, %{"age" => 30, "name" => "Alice"}}

{:ok, result} = Lattice.eval(rt, ~s[{"name": "Alice"}["name"]])
# => {:ok, "Alice"}

# Define a function
Lattice.eval(rt, """
  def add(a: Int, b: Int) -> Int {
    a + b
  }
""")

# Call it from Elixir
{:ok, result} = Lattice.call(rt, "add", [10, 20])
# => {:ok, 30}

# Define types
Lattice.eval(rt, "type Person { name: String, age: Int }")

# Create instances
{:ok, person} = Lattice.eval(rt, ~s[Person { name: "Alice", age: 30 }])
# => {:ok, %{"__type" => "Person", "name" => "Alice", "age" => 30}}

# Define enums
Lattice.eval(rt, "enum Status { Active, Pending, Done }")
{:ok, status} = Lattice.eval(rt, "Status::Active")
# => {:ok, "Status::Active"}

# Get type information
types = Lattice.get_types(rt)
# => [%{type_schema: :struct_type, name: "Person", ...}, %{type_schema: :enum_type, name: "Status", ...}]

# Get function signatures
sigs = Lattice.get_function_signatures(rt)
# => [%{name: "add", params: [...], return_type: %{...}, ...}]

# Use globals to share state between Elixir and Lattice
:ok = Lattice.set_global(rt, "multiplier", 100)
Lattice.eval(rt, "def scale(x: Int) -> Int { x * multiplier }")
{:ok, 500} = Lattice.call(rt, "scale", [5])
```

### Run Examples

```bash
mix run -e "LatticeElixirExample.run_examples()"
```

Example output:

```
=== Lattice Elixir Example ===

Created Lattice runtime

--- Basic Arithmetic ---
1 + 2 + 3 = 6
10 * 5 - 3 = 47
2.5 * 4.0 = 10.0

--- String Operations ---
"hello" + " " + "world" = "hello world"
len("testing") = 7
str(42) = "42"

--- Collections ---
[1, 2, 3, 4, 5] = [1, 2, 3, 4, 5]
[10, 20, 30][1] = 20
len([1, 2, 3, 4]) = 4
{"name": "Alice", "age": 30} = %{"age" => 30, "name" => "Alice"}
{"name": "Alice", "age": 30}["name"] = "Alice"

--- Functions ---
Defined: def add(a: Int, b: Int) -> Int { a + b }
add(10, 20) = 30
greet("Elixir") = "Hello, Elixir!"

Registered functions:
  - add
  - greet

--- Types ---
Defined: type Person { name: String, age: Int }
Person { name: "Bob", age: 25 } = %{"__type" => "Person", "age" => 25, "name" => "Bob"}
Defined: enum Status { Active, Pending, Done }
Status::Active = "Status::Active"

Registered types:
  - struct Person
  - enum Status

--- Globals ---
Set global: multiplier = 100
scale(5) = 500 (using multiplier global)
Get global multiplier = 100

=== All examples completed! ===
```

### Run Tests

```bash
mix test
```

## API Reference

### `Lattice.new/0`

Create a new Lattice runtime (without LLM support).

```elixir
{:ok, runtime} = Lattice.new()
```

### `Lattice.new_with_llm/0`

Create a runtime with LLM support. Requires `OPENROUTER_API_KEY` environment variable.

```elixir
{:ok, runtime} = Lattice.new_with_llm()
```

### `Lattice.eval/2`

Evaluate Lattice source code.

```elixir
{:ok, result} = Lattice.eval(runtime, "1 + 2")
# => {:ok, 3}
```

### `Lattice.eval_with_bindings/3`

Evaluate with pre-bound variables.

```elixir
{:ok, result} = Lattice.eval_with_bindings(runtime, "x + y", [{"x", 10}, {"y", 20}])
# => {:ok, 30}
```

### `Lattice.call/3`

Call a Lattice function by name.

```elixir
Lattice.eval(runtime, "def double(x: Int) -> Int { x * 2 }")
{:ok, result} = Lattice.call(runtime, "double", [5])
# => {:ok, 10}
```

### `Lattice.get_types/1`

Get all registered type schemas.

```elixir
Lattice.eval(runtime, "type Point { x: Int, y: Int }")
types = Lattice.get_types(runtime)
# => [%{type_schema: :struct_type, name: "Point", fields: [...]}]
```

### `Lattice.get_function_signatures/1`

Get all function signatures.

```elixir
Lattice.eval(runtime, "def foo() -> Int { 42 }")
sigs = Lattice.get_function_signatures(runtime)
# => [%{name: "foo", params: [], return_type: %{...}, ...}]
```

### `Lattice.has_function?/2`

Check if a function exists.

```elixir
Lattice.has_function?(runtime, "foo")
# => true
```

### `Lattice.get_global/2`, `Lattice.set_global/3`

Manage global variables.

```elixir
:ok = Lattice.set_global(runtime, "x", 42)
{:ok, 42} = Lattice.get_global(runtime, "x")
```

### `Lattice.reset/1`

Clear all state from the runtime.

```elixir
:ok = Lattice.reset(runtime)
```

## Value Marshaling

| Lattice Type | Elixir Type |
|--------------|-------------|
| null         | :null atom  |
| Bool         | boolean     |
| Int          | integer     |
| Float        | float       |
| String       | binary      |
| Path         | {:path, binary} |
| List         | list        |
| Map          | map         |

## Project Structure

```
lib/
  lattice.ex           # High-level Elixir API
  lattice/
    native.ex          # Low-level NIF bindings
  lattice_elixir_example.ex  # Example usage code
  lattice_elixir_example/
    application.ex     # OTP Application
test/
  lattice_elixir_example_test.exs  # Unit tests
```
