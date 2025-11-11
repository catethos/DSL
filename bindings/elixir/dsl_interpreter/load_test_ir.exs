# Quick script to load and execute test.ir.json
# Usage in IEx: import_file "load_test_ir.exs"

# Load the IR file
ir = File.read!("/Users/catethos/workspace/DSL/test.ir.json") |> Jason.decode!()

IO.puts("\n✓ Loaded test.ir.json")
IO.puts("  Functions: #{Enum.map(ir["functions"], & &1["name"]) |> Enum.join(", ")}")
IO.puts("  Entry: #{ir["entry_expr"]["FunctionCall"]["name"]}(...)")

# Create interpreter
{:ok, interpreter} = DSL.Interpreter.from_ir(ir)
IO.puts("\n✓ Interpreter created")

IO.puts("\nThe IR is loaded and ready!")
IO.puts("\nTo execute the entry expression:")
IO.puts("  DSL.Interpreter.eval(interpreter, ir[\"entry_expr\"])")
IO.puts("\nNote: This requires LLM API configuration to actually run.")
IO.puts("The interpreter variable and ir variable are available for you to use.\n")
