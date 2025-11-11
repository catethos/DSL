# DSL Interpreter - Elixir Bindings

Elixir bindings for the DSL IR (Intermediate Representation) interpreter, implemented in Rust via Rustler.

This library provides a high-performance interpreter for executing DSL IR nodes and complete IR programs from Elixir, with support for execution tracing and comprehensive error handling.

## Features

- **High Performance**: Rust-based interpreter via Rustler NIFs
- **Execution Tracing**: Optional detailed trace collection for debugging and observability
- **Comprehensive Error Handling**: Rich error types with source location information
- **Async Support**: Handles async operations (HTTP, SQL, LLM) internally via Tokio
- **Thread-Safe**: Uses Arc<Mutex> for safe concurrent access
- **JSON Serialization**: Simple JSON-based data exchange between Elixir and Rust

## Installation

Add `dsl_interpreter` to your dependencies in `mix.exs`:

```elixir
def deps do
  [
    {:dsl_interpreter, path: "../path/to/dsl_interpreter"}
  ]
end
```

Then run:

```bash
mix deps.get
mix compile
```

## Quick Start

### Option 1: Use Existing IR File (test.ir.json)

```elixir
# In IEx
import_file "load_test_ir.exs"

# Or manually
ir = File.read!("/Users/catethos/workspace/DSL/test.ir.json") |> Jason.decode!()
{:ok, result} = DSL.Interpreter.run(ir)
```

### Option 2: Create IR Directly

```elixir
# Simple evaluation
{:ok, result} = DSL.Interpreter.run(%{"Int" => 42})

# With tracing
{:ok, result, trace} = DSL.Interpreter.run(%{"Int" => 42}, trace: true)

# Arithmetic
{:ok, result} = DSL.Interpreter.run(%{
  "BinaryOp" => %{
    "left" => %{"Int" => 10},
    "op" => "+",
    "right" => %{"Int" => 32}
  }
})
```

## Usage

### Basic Evaluation

```elixir
# Create an interpreter
{:ok, interpreter} = DSL.Interpreter.new()

# Evaluate a simple node
node = %{"Int" => 42}
{:ok, result} = DSL.Interpreter.eval(interpreter, node)
# => {:ok, %{"Int" => 42}}

# Evaluate expressions
node = %{
  "BinaryOp" => %{
    "left" => %{"Int" => 10},
    "op" => "+",
    "right" => %{"Int" => 32}
  }
}
{:ok, result} = DSL.Interpreter.eval(interpreter, node)
# => {:ok, %{"Int" => 42}}
```

### Execution Tracing

Enable tracing to collect detailed execution information:

```elixir
# Create tracing interpreter
{:ok, interpreter} = DSL.Interpreter.new_tracing(
  max_events: 1000,
  capture_variables: true
)

# Evaluate with trace
{:ok, result, trace} = DSL.Interpreter.eval_with_trace(interpreter, node)
```

### Error Handling

All functions return `{:ok, result}` or `{:error, error_map}` tuples:

```elixir
case DSL.Interpreter.eval(interpreter, node) do
  {:ok, result} ->
    IO.puts("Success: #{inspect(result)}")

  {:error, %{kind: :type_error, message: msg}} ->
    IO.puts("Type Error: #{msg}")

  {:error, %{kind: :unknown_variable, name: var_name}} ->
    IO.puts("Unknown variable: #{var_name}")

  {:error, error} ->
    IO.puts("Error: #{error.message}")
end
```

## Documentation

For full documentation, see the `DSL.Interpreter` module docs or run:

```bash
mix docs
```

## Development

### Building

```bash
mix deps.get
mix compile
```

### Testing

```bash
mix test
```

## Architecture

```
Elixir (DSL.Interpreter)
    ↓ JSON-encoded IR/Values
Rustler NIF Layer (Native)
    ↓ Rust function calls
Rust Interpreter (dsl-interpreter crate)
```

Key design decisions:
- JSON serialization for simplicity
- Global Tokio runtime for async operations
- Arc<Mutex> for thread-safe interpreter access
- DirtyIO scheduler to prevent blocking BEAM

