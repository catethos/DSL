# Example: Using test.ir.json from the DSL repository
#
# This example shows how to:
# 1. Load an IR JSON file from disk
# 2. Parse it into an Elixir map
# 3. Execute it using the DSL interpreter
#
# Usage:
#   cd /Users/catethos/workspace/DSL/bindings/elixir/dsl_interpreter
#   iex -S mix
#   import_file "examples/test_ir_example.exs"

IO.puts("\n" <> String.duplicate("=", 70))
IO.puts("Example: Executing test.ir.json")
IO.puts(String.duplicate("=", 70) <> "\n")

# Path to the IR file (relative to DSL repo root)
ir_file_path = "/Users/catethos/workspace/DSL/test.ir.json"

IO.puts("Step 1: Reading IR file from disk...")
IO.puts("  Path: #{ir_file_path}")

# Read the IR file
ir_json = File.read!(ir_file_path)
IO.puts("✓ IR file loaded (#{byte_size(ir_json)} bytes)")

# Parse JSON into Elixir map
ir = Jason.decode!(ir_json)
IO.puts("✓ JSON parsed successfully")

IO.puts("\nIR Summary:")
IO.puts("  Version: #{ir["version"]}")
IO.puts("  Types: #{length(ir["types"])} (#{Enum.map(ir["types"], & &1["name"]) |> Enum.join(", ")})")
IO.puts("  Functions: #{length(ir["functions"])} (#{Enum.map(ir["functions"], & &1["name"]) |> Enum.join(", ")})")
IO.puts("  Entry expression: FunctionCall(#{ir["entry_expr"]["FunctionCall"]["name"]})")

IO.puts("\n" <> String.duplicate("-", 70))
IO.puts("Step 2: Creating interpreter from IR...")

# Create interpreter from IR
case DSL.Interpreter.from_ir(ir) do
  {:ok, interpreter} ->
    IO.puts("✓ Interpreter created successfully")

    IO.puts("\n" <> String.duplicate("-", 70))
    IO.puts("Step 3: Executing entry expression...")
    IO.puts("\nNOTE: This IR uses LLM function which requires:")
    IO.puts("  - An LLM API (OpenAI, Anthropic, etc.)")
    IO.puts("  - API key in environment variable")
    IO.puts("  - Network connectivity")
    IO.puts("\nIf you don't have LLM configured, the execution will fail with an error.")
    IO.puts("That's expected! The important part is that the IR loaded correctly.\n")

    # Try to execute (will likely fail without LLM setup, which is fine)
    case DSL.Interpreter.eval(interpreter, ir["entry_expr"]) do
      {:ok, result} ->
        IO.puts("✓ Execution succeeded!")
        IO.puts("\nResult:")
        IO.inspect(result, pretty: true, limit: :infinity)

      {:error, error} ->
        IO.puts("✗ Execution failed (expected if LLM not configured)")
        IO.puts("\nError details:")
        IO.inspect(error, pretty: true)
        IO.puts("\nThis is normal if you don't have LLM API configured.")
    end

  {:error, error} ->
    IO.puts("✗ Failed to create interpreter")
    IO.inspect(error, pretty: true)
end

IO.puts("\n" <> String.duplicate("=", 70))
IO.puts("Example: Testing with simpler IR (no LLM)")
IO.puts(String.duplicate("=", 70) <> "\n")

# Create a simpler version without LLM for testing
simple_ir = %{
  "version" => "0.1.0",
  "types" => [
    %{
      "name" => "Person",
      "description" => nil,
      "fields" => [
        %{"name" => "name", "field_type" => "String", "optional" => false, "description" => nil},
        %{"name" => "age", "field_type" => "Int", "optional" => false, "description" => nil}
      ]
    }
  ],
  "enums" => [],
  "functions" => [
    %{
      "name" => "double",
      "params" => ["x"],
      "return_type" => "Int",
      "properties" => %{},
      "execution" => %{
        "Pure" => %{
          "BinaryOp" => %{
            "left" => %{"Variable" => "x"},
            "op" => "*",
            "right" => %{"Int" => 2}
          }
        }
      }
    }
  ],
  "function_groups" => [],
  "agents" => [],
  "entry_expr" => %{
    "FunctionCall" => %{
      "name" => "double",
      "args" => [%{"Int" => 21}],
      "effect_kind" => nil,
      "source_span" => nil
    }
  }
}

IO.puts("Executing simplified IR (pure function: double(21))...")

case DSL.Interpreter.run(simple_ir) do
  {:ok, result} ->
    IO.puts("✓ Success!")
    IO.inspect(result, label: "Result")

  {:error, error} ->
    IO.puts("✗ Error:")
    IO.inspect(error, pretty: true)
end

IO.puts("\n" <> String.duplicate("=", 70))
IO.puts("Summary")
IO.puts(String.duplicate("=", 70))
IO.puts("""

Key Takeaways:
1. ✓ Load IR from JSON file using File.read!/1 and Jason.decode!/1
2. ✓ Create interpreter from IR using DSL.Interpreter.from_ir/1
3. ✓ Execute entry expression using DSL.Interpreter.eval/2
4. ✓ Or use one-shot execution with DSL.Interpreter.run/1

The test.ir.json uses LLM functions which require additional setup.
For testing without external dependencies, use pure functions or simpler IR.

Try these commands in IEx:
  # Load and inspect the IR
  ir = File.read!("/Users/catethos/workspace/DSL/test.ir.json") |> Jason.decode!()

  # Create interpreter
  {:ok, interp} = DSL.Interpreter.from_ir(ir)

  # Inspect the entry expression
  IO.inspect(ir["entry_expr"], pretty: true)

  # Try to execute (needs LLM setup)
  DSL.Interpreter.eval(interp, ir["entry_expr"])
""")
