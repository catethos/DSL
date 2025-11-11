# Testing the DSL Interpreter Elixir Integration

## Quick Start - Testing in IEx

### 1. Start IEx

```bash
cd /Users/catethos/workspace/DSL/bindings/elixir/dsl_interpreter
iex -S mix
```

Wait for compilation to complete (first time will take a while as it compiles the Rust NIF).

### 2. Run Quick Tests

Once in IEx, load the test script:

```elixir
import_file "test_iex.exs"
```

This will run 10 quick tests and show you the results.

### 3. Try Manual Commands

```elixir
# Simple integer
DSL.Interpreter.run(%{"Int" => 42})

# Arithmetic
DSL.Interpreter.run(%{
  "BinaryOp" => %{
    "left" => %{"Int" => 10},
    "op" => "+",
    "right" => %{"Int" => 32}
  }
})

# With tracing
DSL.Interpreter.run(%{"Int" => 42}, trace: true)

# String
DSL.Interpreter.run(%{"String" => "Hello, Elixir!"})

# List
DSL.Interpreter.run(%{
  "List" => [
    %{"Int" => 1},
    %{"Int" => 2},
    %{"Int" => 3}
  ]
})
```

## Running Automated Tests

### Run All Tests

```bash
mix test
```

### Run with Detailed Output

```bash
mix test --trace
```

### Run Specific Test File

```bash
mix test test/dsl/interpreter_test.exs
```

### Run a Specific Test

```bash
# Run test at line 42
mix test test/dsl/interpreter_test.exs:42
```

## Common Test Scenarios

### 1. Basic Literals

```elixir
# In IEx
DSL.Interpreter.run(%{"Int" => 42})
DSL.Interpreter.run(%{"String" => "test"})
DSL.Interpreter.run(%{"Float" => 3.14})
DSL.Interpreter.run(%{"Bool" => true})
DSL.Interpreter.run("Null")
```

### 2. Operations

```elixir
# Addition
DSL.Interpreter.run(%{
  "BinaryOp" => %{"left" => %{"Int" => 10}, "op" => "+", "right" => %{"Int" => 32}}
})

# Subtraction
DSL.Interpreter.run(%{
  "BinaryOp" => %{"left" => %{"Int" => 100}, "op" => "-", "right" => %{"Int" => 58}}
})

# Multiplication
DSL.Interpreter.run(%{
  "BinaryOp" => %{"left" => %{"Int" => 6}, "op" => "*", "right" => %{"Int" => 7}}
})
```

### 3. Conditionals

```elixir
DSL.Interpreter.run(%{
  "Conditional" => %{
    "condition" => %{"Bool" => true},
    "then_expr" => %{"String" => "yes"},
    "else_expr" => %{"String" => "no"}
  }
})
```

### 4. Collections

```elixir
# List
DSL.Interpreter.run(%{
  "List" => [%{"Int" => 1}, %{"Int" => 2}, %{"Int" => 3}]
})

# Map
DSL.Interpreter.run(%{
  "Map" => [
    ["name", %{"String" => "Alice"}],
    ["age", %{"Int" => 30}]
  ]
})
```

### 5. Tracing

```elixir
# Simple with trace
{:ok, result, trace} = DSL.Interpreter.run(%{"Int" => 42}, trace: true)
IO.inspect(trace["events"])

# Complex operation with trace
{:ok, result, trace} = DSL.Interpreter.run(%{
  "BinaryOp" => %{
    "left" => %{"Int" => 10},
    "op" => "+",
    "right" => %{"Int" => 32}
  }
}, trace: true)

IO.puts("Number of trace events: #{length(trace["events"])}")
```

### 6. Error Handling

```elixir
# Unknown variable
{:error, error} = DSL.Interpreter.run(%{"Variable" => "unknown"})
IO.inspect(error)

# Type error (if applicable)
{:error, error} = DSL.Interpreter.run(%{
  "BinaryOp" => %{
    "left" => %{"String" => "hello"},
    "op" => "+",
    "right" => %{"Int" => 42}
  }
})
IO.inspect(error)
```

## Troubleshooting

### "NIF not loaded" Error

If you see errors about NIF not being loaded:

```bash
# Clean and rebuild
mix clean
RUSTLER_PRECOMPILATION_DSL_NIF_BUILD=1 mix compile

# Then start IEx again
iex -S mix
```

### Compilation Errors

```bash
# Update dependencies
mix deps.get

# Force recompile
RUSTLER_PRECOMPILATION_DSL_NIF_BUILD=1 mix compile --force
```

### Check NIF is Loaded

```elixir
# In IEx
File.exists?("priv/native/dsl_nif.so")  # Should return true
```

## Performance Testing

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

# Run 1000 times and measure
{time_us, _} = :timer.tc(fn ->
  Enum.each(1..1000, fn _ ->
    DSL.Interpreter.run(node)
  end)
end)

avg_us = time_us / 1000
IO.puts("Average execution time: #{avg_us} microseconds")
IO.puts("Average execution time: #{avg_us / 1000} milliseconds")
```

## Advanced Usage

### Stateful Interpreter

```elixir
# Create once, use multiple times
{:ok, interpreter} = DSL.Interpreter.new()

# Execute multiple times
results = Enum.map(1..10, fn i ->
  {:ok, result} = DSL.Interpreter.eval(interpreter, %{"Int" => i})
  result
end)

IO.inspect(results)
```

### Tracing Interpreter with State

```elixir
# Create tracing interpreter
{:ok, tracer} = DSL.Interpreter.new_tracing()

# Execute and accumulate traces
{:ok, r1, t1} = DSL.Interpreter.eval_with_trace(tracer, %{"Int" => 1})
{:ok, r2, t2} = DSL.Interpreter.eval_with_trace(tracer, %{"Int" => 2})

# Get accumulated trace
{:ok, full_trace} = DSL.Interpreter.get_trace(tracer)
IO.puts("Total trace events: #{length(full_trace["events"])}")

# Clear for fresh start
:ok = DSL.Interpreter.clear_trace(tracer)
```

## See Also

- `QUICKSTART.md` - Comprehensive examples and usage guide
- `README.md` - Project overview and installation
- `test/dsl/interpreter_test.exs` - Full test suite with examples
