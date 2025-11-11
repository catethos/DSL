# Using IR Files with the Elixir DSL Interpreter

This guide shows you how to load and execute IR JSON files (like `test.ir.json`) using the Elixir interpreter.

## Quick Start

### 1. Start IEx

```bash
cd /Users/catethos/workspace/DSL/bindings/elixir/dsl_interpreter
iex -S mix
```

### 2. Load test.ir.json

```elixir
# Option A: Use the quick loader script
import_file "load_test_ir.exs"

# Option B: Load manually
ir = File.read!("/Users/catethos/workspace/DSL/test.ir.json") |> Jason.decode!()
{:ok, interpreter} = DSL.Interpreter.from_ir(ir)

# Option C: Run the full example
import_file "examples/test_ir_example.exs"
```

## Understanding test.ir.json

The test.ir.json file contains:

- **Types**: `People` class with fields (name, age, address, email)
- **Functions**: `test` function that uses LLM to extract information
- **Entry Expression**: Calls `test("regina is 30 yo living in Sydney...")`

This IR demonstrates LLM-based information extraction.

## Step-by-Step Usage

### Step 1: Load the IR File

```elixir
# Read the JSON file from disk
ir_json = File.read!("/Users/catethos/workspace/DSL/test.ir.json")

# Parse JSON into Elixir map
ir = Jason.decode!(ir_json)

# Inspect the structure
IO.inspect(ir, label: "IR Structure", limit: 5)
```

### Step 2: Examine the IR

```elixir
# Check version
ir["version"]  # => "0.1.0"

# List types
ir["types"] |> Enum.map(& &1["name"])  # => ["People"]

# List functions
ir["functions"] |> Enum.map(& &1["name"])  # => ["test"]

# View entry expression
IO.inspect(ir["entry_expr"], pretty: true)
```

### Step 3: Create Interpreter from IR

```elixir
# Create interpreter that knows about the types and functions
{:ok, interpreter} = DSL.Interpreter.from_ir(ir)
```

This loads:
- Type definitions (People)
- Function definitions (test)
- Into the interpreter's runtime

### Step 4: Execute the Entry Expression

```elixir
# Execute the main program
result = DSL.Interpreter.eval(interpreter, ir["entry_expr"])

case result do
  {:ok, value} ->
    IO.puts("Success!")
    IO.inspect(value, pretty: true)

  {:error, error} ->
    IO.puts("Error:")
    IO.inspect(error, pretty: true)
end
```

**Note**: The test.ir.json uses LLM functions which require:
- LLM API configuration (OpenAI, Anthropic, etc.)
- API key in environment
- Network connectivity

Without these, you'll get an error, which is expected!

### Step 5: Execute with Tracing

```elixir
# Create tracing interpreter from IR
{:ok, tracer} = DSL.Interpreter.from_ir_tracing(ir)

# Execute with trace
{:ok, result, trace} = DSL.Interpreter.eval_with_trace(tracer, ir["entry_expr"])

# View trace events
IO.puts("Trace events: #{length(trace["events"])}")
trace["events"] |> Enum.each(fn event ->
  IO.puts("  - #{event["node_type"]}: #{event["description"]}")
end)
```

## Working with Different IR Files

### Load Any IR File

```elixir
# Load from any path
load_ir = fn path ->
  path
  |> File.read!()
  |> Jason.decode!()
end

# Use it
my_ir = load_ir.("/path/to/your/program.ir.json")
{:ok, interp} = DSL.Interpreter.from_ir(my_ir)
```

### Validate IR Structure

```elixir
# Check required fields
validate_ir = fn ir ->
  required = ["version", "types", "enums", "functions", "function_groups", "agents", "entry_expr"]
  missing = required -- Map.keys(ir)

  if missing == [] do
    IO.puts("✓ IR structure is valid")
    {:ok, ir}
  else
    IO.puts("✗ Missing fields: #{inspect(missing)}")
    {:error, "Invalid IR structure"}
  end
end

validate_ir.(ir)
```

### Execute Specific Functions

Instead of executing the entry expression, you can call specific functions:

```elixir
# Call the 'test' function with different input
call_test = %{
  "FunctionCall" => %{
    "name" => "test",
    "args" => [
      %{
        "TemplateString" => [
          %{"Text" => "John is 25 years old, lives in NYC, email: john@example.com"}
        ]
      }
    ],
    "effect_kind" => nil,
    "source_span" => nil
  }
}

DSL.Interpreter.eval(interpreter, call_test)
```

## Creating Test IR Without External Dependencies

For testing without LLM/HTTP/SQL, create pure function IR:

```elixir
# Simple arithmetic function
simple_ir = %{
  "version" => "0.1.0",
  "types" => [],
  "enums" => [],
  "functions" => [
    %{
      "name" => "add",
      "params" => ["a", "b"],
      "return_type" => "Int",
      "properties" => %{},
      "execution" => %{
        "Pure" => %{
          "BinaryOp" => %{
            "left" => %{"Variable" => "a"},
            "op" => "+",
            "right" => %{"Variable" => "b"}
          }
        }
      }
    }
  ],
  "function_groups" => [],
  "agents" => [],
  "entry_expr" => %{
    "FunctionCall" => %{
      "name" => "add",
      "args" => [%{"Int" => 10}, %{"Int" => 32}],
      "effect_kind" => nil,
      "source_span" => nil
    }
  }
}

# Execute
{:ok, result} = DSL.Interpreter.run(simple_ir)
IO.inspect(result)  # => %{"Int" => 42}
```

## Advanced: Modifying IR at Runtime

```elixir
# Load base IR
ir = File.read!("/Users/catethos/workspace/DSL/test.ir.json") |> Jason.decode!()

# Change the entry expression
ir = Map.put(ir, "entry_expr", %{
  "String" => "Modified program!"
})

# Execute modified IR
{:ok, result} = DSL.Interpreter.run(ir)
```

## Saving Results Back to Files

```elixir
# Execute and save result
{:ok, result} = DSL.Interpreter.run(ir)

# Save result as JSON
result_json = Jason.encode!(result, pretty: true)
File.write!("/tmp/result.json", result_json)

IO.puts("✓ Result saved to /tmp/result.json")
```

## Working with Multiple IR Files

```elixir
# Load multiple IR files
ir_files = [
  "/Users/catethos/workspace/DSL/test.ir.json",
  "/path/to/another.ir.json"
]

results = Enum.map(ir_files, fn path ->
  ir = File.read!(path) |> Jason.decode!()

  case DSL.Interpreter.run(ir) do
    {:ok, result} ->
      {path, :ok, result}

    {:error, error} ->
      {path, :error, error}
  end
end)

# Display results
Enum.each(results, fn {path, status, data} ->
  IO.puts("\n#{Path.basename(path)}: #{status}")
  IO.inspect(data, limit: 3)
end)
```

## Batch Processing IR Files

```elixir
# Process directory of IR files
process_ir_directory = fn dir_path ->
  dir_path
  |> File.ls!()
  |> Enum.filter(&String.ends_with?(&1, ".ir.json"))
  |> Enum.map(fn file ->
    full_path = Path.join(dir_path, file)
    ir = File.read!(full_path) |> Jason.decode!()

    {file, DSL.Interpreter.run(ir)}
  end)
end

# Use it
# results = process_ir_directory.("/path/to/ir/files")
```

## Debugging IR Execution

```elixir
# Load IR
ir = File.read!("/Users/catethos/workspace/DSL/test.ir.json") |> Jason.decode!()

# Create tracing interpreter
{:ok, tracer} = DSL.Interpreter.from_ir_tracing(ir)

# Execute with detailed tracing
{:ok, result, trace} = DSL.Interpreter.eval_with_trace(tracer, ir["entry_expr"])

# Analyze trace
IO.puts("Execution trace:")
IO.puts("  Total events: #{length(trace["events"])}")

# Group by node type
trace["events"]
|> Enum.group_by(& &1["node_type"])
|> Enum.each(fn {type, events} ->
  IO.puts("  #{type}: #{length(events)} events")
end)

# Find errors
errors = Enum.filter(trace["events"], & &1["error"])
if length(errors) > 0 do
  IO.puts("\nErrors found:")
  Enum.each(errors, fn event ->
    IO.inspect(event, label: "Error")
  end)
end
```

## Common Patterns

### Pattern 1: One-Shot Execution from File

```elixir
"/Users/catethos/workspace/DSL/test.ir.json"
|> File.read!()
|> Jason.decode!()
|> DSL.Interpreter.run()
|> case do
  {:ok, result} -> IO.inspect(result, label: "Success")
  {:error, error} -> IO.inspect(error, label: "Error")
end
```

### Pattern 2: Reusable Interpreter

```elixir
# Load once
ir = File.read!("/Users/catethos/workspace/DSL/test.ir.json") |> Jason.decode!()
{:ok, interpreter} = DSL.Interpreter.from_ir(ir)

# Execute multiple expressions
expressions = [
  %{"Int" => 1},
  %{"Int" => 2},
  %{"Int" => 3}
]

results = Enum.map(expressions, fn expr ->
  {:ok, result} = DSL.Interpreter.eval(interpreter, expr)
  result
end)
```

### Pattern 3: Pipeline with Error Handling

```elixir
with {:ok, json} <- File.read("/Users/catethos/workspace/DSL/test.ir.json"),
     {:ok, ir} <- Jason.decode(json),
     {:ok, interpreter} <- DSL.Interpreter.from_ir(ir),
     {:ok, result} <- DSL.Interpreter.eval(interpreter, ir["entry_expr"]) do
  IO.puts("Success!")
  IO.inspect(result)
else
  {:error, reason} ->
    IO.puts("Failed: #{inspect(reason)}")
end
```

## Summary

The key workflow is:

```
IR File → Read → Parse JSON → Create Interpreter → Execute → Result
```

```elixir
# All in one line
File.read!("path/to/file.ir.json")
|> Jason.decode!()
|> DSL.Interpreter.run()
```

See also:
- `examples/test_ir_example.exs` - Complete example with test.ir.json
- `load_test_ir.exs` - Quick loader script
- `QUICKSTART.md` - General usage examples
- `TESTING.md` - Testing guide
