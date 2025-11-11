# Step-by-Step Guide: Using test.ir.json in Elixir

This is a complete walkthrough for using your existing `test.ir.json` file with the Elixir DSL interpreter.

## Prerequisites

- Elixir installed (you have 1.18.4)
- The DSL repository at `/Users/catethos/workspace/DSL/`
- The Elixir bindings compiled (already done)

## Step 1: Start IEx

Open a terminal and run:

```bash
cd /Users/catethos/workspace/DSL/bindings/elixir/dsl_interpreter
iex -S mix
```

You should see Elixir compile the project and start an interactive shell.

## Step 2: Quick Test with the Loader Script

Once IEx is running, type:

```elixir
import_file "load_test_ir.exs"
```

This will:
- ✓ Load `/Users/catethos/workspace/DSL/test.ir.json`
- ✓ Parse the JSON
- ✓ Create an interpreter
- ✓ Set up `ir` and `interpreter` variables for you to use

You should see output like:
```
✓ Loaded test.ir.json
  Functions: test
  Entry: test(...)

✓ Interpreter created

The IR is loaded and ready!
```

## Step 3: Inspect the IR

Now you have the `ir` variable available. Explore it:

```elixir
# See the version
ir["version"]

# List all types
ir["types"]

# See the People type definition
ir["types"] |> List.first() |> IO.inspect(pretty: true)

# List all functions
ir["functions"] |> Enum.map(& &1["name"])

# See what the entry expression does
ir["entry_expr"] |> IO.inspect(pretty: true)
```

## Step 4: Understand What This IR Does

The `test.ir.json` defines:

1. **A Type**: `People` with fields: name, age, address, email
2. **A Function**: `test(x)` that uses LLM to extract People info from text
3. **Entry Expression**: Calls `test("regina is 30 yo living in Sydney, with email regina@example.com")`

So this program uses AI to parse unstructured text into structured data!

## Step 5: Try to Execute (Will Need LLM Setup)

```elixir
# Try to execute the entry expression
result = DSL.Interpreter.eval(interpreter, ir["entry_expr"])
```

**Expected behavior:**
- If you have LLM configured (API key, etc.) → It works! You get structured People data
- If you don't have LLM configured → You get an error (this is normal!)

The error might look like:
```elixir
{:error, %{kind: :llm_error, message: "LLM API not configured", ...}}
```

This is **expected** and shows the interpreter is working correctly!

## Step 6: Test with a Simpler Expression

Let's test the interpreter with something that doesn't need LLM:

```elixir
# Simple integer
DSL.Interpreter.eval(interpreter, %{"Int" => 42})
# => {:ok, %{"Int" => 42}}

# Simple string
DSL.Interpreter.eval(interpreter, %{"String" => "Hello from Elixir!"})
# => {:ok, %{"String" => "Hello from Elixir!"}}

# Arithmetic
DSL.Interpreter.eval(interpreter, %{
  "BinaryOp" => %{
    "left" => %{"Int" => 10},
    "op" => "+",
    "right" => %{"Int" => 32}
  }
})
# => {:ok, %{"Int" => 42}}
```

All of these should work perfectly!

## Step 7: Execute with Tracing

Let's see what happens during execution:

```elixir
# Create a tracing interpreter from the same IR
{:ok, tracer} = DSL.Interpreter.from_ir_tracing(ir)

# Execute a simple expression with trace
{:ok, result, trace} = DSL.Interpreter.eval_with_trace(
  tracer,
  %{"Int" => 42}
)

# Look at the trace
IO.puts("Trace events: #{length(trace["events"])}")
trace["events"] |> IO.inspect(pretty: true)
```

## Step 8: Create Your Own Simple IR

Let's create a simple IR that works without external dependencies:

```elixir
# Define a simple IR with a function
my_ir = %{
  "version" => "0.1.0",
  "types" => [],
  "enums" => [],
  "functions" => [
    %{
      "name" => "greet",
      "params" => ["name"],
      "return_type" => "String",
      "properties" => %{},
      "execution" => %{
        "Pure" => %{
          "TemplateString" => [
            %{"Text" => "Hello, "},
            %{"Interpolation" => %{"Variable" => "name"}},
            %{"Text" => "!"}
          ]
        }
      }
    }
  ],
  "function_groups" => [],
  "agents" => [],
  "entry_expr" => %{
    "FunctionCall" => %{
      "name" => "greet",
      "args" => [%{"String" => "Elixir"}],
      "effect_kind" => nil,
      "source_span" => nil
    }
  }
}

# Execute it!
{:ok, result} = DSL.Interpreter.run(my_ir)
IO.inspect(result)
# => %{"String" => "Hello, Elixir!"}
```

## Step 9: Save an IR to a File

```elixir
# Create an IR
simple_ir = %{
  "version" => "0.1.0",
  "types" => [],
  "enums" => [],
  "functions" => [],
  "function_groups" => [],
  "agents" => [],
  "entry_expr" => %{"Int" => 42}
}

# Convert to pretty JSON
json = Jason.encode!(simple_ir, pretty: true)

# Save to file
File.write!("/tmp/my_program.ir.json", json)

IO.puts("✓ Saved to /tmp/my_program.ir.json")

# Load it back
loaded_ir = File.read!("/tmp/my_program.ir.json") |> Jason.decode!()
{:ok, result} = DSL.Interpreter.run(loaded_ir)
```

## Step 10: Batch Process Multiple IR Files

```elixir
# Create a helper function
run_ir_file = fn path ->
  try do
    result = path
    |> File.read!()
    |> Jason.decode!()
    |> DSL.Interpreter.run()

    {Path.basename(path), result}
  rescue
    e -> {Path.basename(path), {:error, Exception.message(e)}}
  end
end

# Use it with test.ir.json
result = run_ir_file.("/Users/catethos/workspace/DSL/test.ir.json")
IO.inspect(result)
```

## Common Commands Summary

```elixir
# --- Loading IR ---
ir = File.read!("/Users/catethos/workspace/DSL/test.ir.json") |> Jason.decode!()

# --- Creating Interpreters ---
{:ok, interp} = DSL.Interpreter.from_ir(ir)          # Basic
{:ok, tracer} = DSL.Interpreter.from_ir_tracing(ir)  # With tracing

# --- Executing ---
{:ok, result} = DSL.Interpreter.eval(interp, ir["entry_expr"])
{:ok, result, trace} = DSL.Interpreter.eval_with_trace(tracer, ir["entry_expr"])

# --- One-shot execution ---
{:ok, result} = DSL.Interpreter.run(ir)
{:ok, result, trace} = DSL.Interpreter.run(ir, trace: true)

# --- Inspecting ---
IO.inspect(result, pretty: true)
IO.inspect(trace["events"], limit: :infinity)
```

## Next Steps

1. **Run the full example**: `import_file "examples/test_ir_example.exs"`
2. **Read the guides**:
   - `USING_IR_FILES.md` - Comprehensive IR usage guide
   - `QUICKSTART.md` - All IR node types and examples
   - `TESTING.md` - Testing and troubleshooting
3. **Experiment**: Create your own IR files and test them!

## Troubleshooting

### "Module not found" or "NIF not loaded"

```bash
# Exit IEx (Ctrl+C twice or type: System.halt())
# Recompile everything
mix clean
RUSTLER_PRECOMPILATION_DSL_NIF_BUILD=1 mix compile
# Start IEx again
iex -S mix
```

### "File not found" when loading test.ir.json

Check the path:
```elixir
File.exists?("/Users/catethos/workspace/DSL/test.ir.json")
# Should return true
```

If false, update the path in the scripts to match your setup.

### LLM Errors

The test.ir.json requires LLM configuration. This is normal!
To test without LLM, use the simple IR examples in Step 8.

## Summary

You've learned how to:
- ✅ Load IR from JSON files
- ✅ Create interpreters from IR
- ✅ Execute IR programs
- ✅ Use tracing for debugging
- ✅ Create and save your own IR
- ✅ Handle errors gracefully

**The complete workflow:**
```
IR File → Load → Parse → Create Interpreter → Execute → Result
```

Enjoy using the DSL interpreter from Elixir! 🎉
