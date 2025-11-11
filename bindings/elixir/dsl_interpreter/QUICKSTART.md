# Quick Start Guide - Testing in IEx

## Start Interactive Shell

```bash
cd /Users/catethos/workspace/DSL/bindings/elixir/dsl_interpreter
iex -S mix
```

## Basic Examples

### 1. Simple Literal Evaluation

```elixir
# Evaluate an integer
{:ok, result} = DSL.Interpreter.run(%{"Int" => 42})
# => {:ok, %{"Int" => 42}}

# Evaluate a string
{:ok, result} = DSL.Interpreter.run(%{"String" => "Hello, World!"})
# => {:ok, %{"String" => "Hello, World!"}}

# Evaluate a float
{:ok, result} = DSL.Interpreter.run(%{"Float" => 3.14})
# => {:ok, %{"Float" => 3.14}}

# Evaluate a boolean
{:ok, result} = DSL.Interpreter.run(%{"Bool" => true})
# => {:ok, %{"Bool" => true}}
```

### 2. Arithmetic Operations

```elixir
# Simple addition
node = %{
  "BinaryOp" => %{
    "left" => %{"Int" => 10},
    "op" => "+",
    "right" => %{"Int" => 32}
  }
}
{:ok, result} = DSL.Interpreter.run(node)
# => {:ok, %{"Int" => 42}}

# Multiplication
node = %{
  "BinaryOp" => %{
    "left" => %{"Int" => 6},
    "op" => "*",
    "right" => %{"Int" => 7}
  }
}
{:ok, result} = DSL.Interpreter.run(node)
# => {:ok, %{"Int" => 42}}

# Nested operations: (10 + 20) * 2
node = %{
  "BinaryOp" => %{
    "left" => %{
      "BinaryOp" => %{
        "left" => %{"Int" => 10},
        "op" => "+",
        "right" => %{"Int" => 20}
      }
    },
    "op" => "*",
    "right" => %{"Int" => 2}
  }
}
{:ok, result} = DSL.Interpreter.run(node)
# => {:ok, %{"Int" => 60}}
```

### 3. Collections

```elixir
# List
node = %{
  "List" => [
    %{"Int" => 1},
    %{"Int" => 2},
    %{"Int" => 3}
  ]
}
{:ok, result} = DSL.Interpreter.run(node)
# => {:ok, %{"List" => [%{"Int" => 1}, %{"Int" => 2}, %{"Int" => 3}]}}

# Map
node = %{
  "Map" => [
    ["name", %{"String" => "Alice"}],
    ["age", %{"Int" => 30}]
  ]
}
{:ok, result} = DSL.Interpreter.run(node)
```

### 4. Conditional Expressions

```elixir
# If-then-else
node = %{
  "Conditional" => %{
    "condition" => %{"Bool" => true},
    "then_expr" => %{"String" => "yes"},
    "else_expr" => %{"String" => "no"}
  }
}
{:ok, result} = DSL.Interpreter.run(node)
# => {:ok, %{"String" => "yes"}}

# With false condition
node = %{
  "Conditional" => %{
    "condition" => %{"Bool" => false},
    "then_expr" => %{"String" => "yes"},
    "else_expr" => %{"String" => "no"}
  }
}
{:ok, result} = DSL.Interpreter.run(node)
# => {:ok, %{"String" => "no"}}
```

### 5. Using Stateful Interpreter

```elixir
# Create an interpreter instance
{:ok, interpreter} = DSL.Interpreter.new()

# Evaluate multiple nodes with the same interpreter
{:ok, result1} = DSL.Interpreter.eval(interpreter, %{"Int" => 1})
{:ok, result2} = DSL.Interpreter.eval(interpreter, %{"Int" => 2})
{:ok, result3} = DSL.Interpreter.eval(interpreter, %{"Int" => 3})
```

### 6. Execution Tracing

```elixir
# Run with tracing enabled
node = %{
  "BinaryOp" => %{
    "left" => %{"Int" => 10},
    "op" => "+",
    "right" => %{"Int" => 32}
  }
}

{:ok, result, trace} = DSL.Interpreter.run(node, trace: true)

# Inspect the result
IO.inspect(result, label: "Result")

# Inspect the trace
IO.inspect(trace, label: "Trace")

# Look at trace events
trace["events"] |> IO.inspect(label: "Trace Events")
```

### 7. Using Tracing Interpreter Directly

```elixir
# Create a tracing interpreter
{:ok, tracer} = DSL.Interpreter.new_tracing()

# Execute with trace
node = %{"Int" => 42}
{:ok, result, trace} = DSL.Interpreter.eval_with_trace(tracer, node)

# Execute another node (traces accumulate)
{:ok, result2, trace2} = DSL.Interpreter.eval_with_trace(tracer, %{"Int" => 99})

# Get current trace
{:ok, current_trace} = DSL.Interpreter.get_trace(tracer)

# Clear the trace
:ok = DSL.Interpreter.clear_trace(tracer)

# Verify it's cleared
{:ok, empty_trace} = DSL.Interpreter.get_trace(tracer)
empty_trace["events"]  # => []
```

### 8. Complete IR Program

```elixir
# Define a complete IR with functions
ir = %{
  "version" => "1.0",
  "types" => [],
  "enums" => [],
  "functions" => [],
  "function_groups" => [],
  "agents" => [],
  "entry_expr" => %{
    "BinaryOp" => %{
      "left" => %{"Int" => 100},
      "op" => "-",
      "right" => %{"Int" => 58}
    }
  }
}

# Run the complete IR
{:ok, result} = DSL.Interpreter.run(ir)
# => {:ok, %{"Int" => 42}}

# Or create interpreter from IR
{:ok, interpreter} = DSL.Interpreter.from_ir(ir)
{:ok, result} = DSL.Interpreter.eval(interpreter, ir["entry_expr"])
```

### 9. Error Handling

```elixir
# Try to access an unknown variable
node = %{"Variable" => "undefined_var"}
{:error, error} = DSL.Interpreter.run(node)

IO.inspect(error)
# => %{
#   kind: :unknown_variable,
#   message: "Unknown variable: undefined_var",
#   name: "undefined_var"
# }

# Type error example
node = %{
  "BinaryOp" => %{
    "left" => %{"String" => "hello"},
    "op" => "+",
    "right" => %{"Int" => 42}
  }
}
{:error, error} = DSL.Interpreter.run(node)
IO.inspect(error.kind)  # => :type_error or :runtime_error
```

### 10. Working with JSON

```elixir
# You can also pass JSON strings directly
node_json = Jason.encode!(%{"Int" => 42})
{:ok, interpreter} = DSL.Interpreter.new()
{:ok, result} = DSL.Interpreter.eval(interpreter, node_json)
```

## Helper Functions for Testing

```elixir
# Pretty print results
defmodule TestHelper do
  def run_and_print(node) do
    case DSL.Interpreter.run(node) do
      {:ok, result} ->
        IO.puts("✓ Success:")
        IO.inspect(result, pretty: true)

      {:error, error} ->
        IO.puts("✗ Error:")
        IO.inspect(error, pretty: true)
    end
  end

  def run_with_trace_and_print(node) do
    case DSL.Interpreter.run(node, trace: true) do
      {:ok, result, trace} ->
        IO.puts("✓ Success:")
        IO.inspect(result, label: "Result", pretty: true)
        IO.puts("\nTrace Events: #{length(trace["events"])}")

      {:error, error} ->
        IO.puts("✗ Error:")
        IO.inspect(error, pretty: true)
    end
  end
end

# Use it
TestHelper.run_and_print(%{"Int" => 42})
TestHelper.run_with_trace_and_print(%{
  "BinaryOp" => %{
    "left" => %{"Int" => 10},
    "op" => "+",
    "right" => %{"Int" => 32}
  }
})
```

## Running Tests

```bash
# Run all tests
mix test

# Run specific test file
mix test test/dsl/interpreter_test.exs

# Run with detailed output
mix test --trace

# Run a specific test
mix test test/dsl/interpreter_test.exs:42  # line number
```

## Benchmarking

```elixir
# Simple benchmark
node = %{
  "BinaryOp" => %{
    "left" => %{"Int" => 10},
    "op" => "+",
    "right" => %{"Int" => 32}
  }
}

# Warmup
DSL.Interpreter.run(node)

# Benchmark
:timer.tc(fn ->
  Enum.each(1..1000, fn _ ->
    DSL.Interpreter.run(node)
  end)
end)
|> elem(0)
|> Kernel./(1000)
|> IO.puts()  # Prints average microseconds per execution
```

## Troubleshooting

### NIF Not Loaded Error

If you see `:nif_not_loaded` errors:

```bash
# Clean and recompile
mix clean
RUSTLER_PRECOMPILATION_DSL_NIF_BUILD=1 mix compile
```

### Compilation Errors

```bash
# Update dependencies
mix deps.get
mix deps.compile

# Force rebuild of Rust NIF
RUSTLER_PRECOMPILATION_DSL_NIF_BUILD=1 mix compile --force
```

### Check NIF is Loaded

```elixir
# Verify the NIF shared library exists
File.exists?("priv/native/dsl_nif.so")  # => true

# Check if module is loaded
:code.is_loaded(DSL.Interpreter.Native)  # => {:file, ...}
```
