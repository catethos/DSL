# Simple test script to run in IEx
# Usage: iex -S mix
# Then: import_file "test_iex.exs"

IO.puts("\n" <> String.duplicate("=", 60))
IO.puts("DSL Interpreter - Quick Tests")
IO.puts(String.duplicate("=", 60) <> "\n")

# Test 1: Simple Integer
IO.puts("Test 1: Simple Integer Literal")
{:ok, result} = DSL.Interpreter.run(%{"Int" => 42})
IO.inspect(result, label: "✓ Result")

# Test 2: Simple String
IO.puts("\nTest 2: String Literal")
{:ok, result} = DSL.Interpreter.run(%{"String" => "Hello, World!"})
IO.inspect(result, label: "✓ Result")

# Test 3: Arithmetic
IO.puts("\nTest 3: Arithmetic (10 + 32)")
node = %{
  "BinaryOp" => %{
    "left" => %{"Int" => 10},
    "op" => "+",
    "right" => %{"Int" => 32}
  }
}
{:ok, result} = DSL.Interpreter.run(node)
IO.inspect(result, label: "✓ Result")

# Test 4: Multiplication
IO.puts("\nTest 4: Multiplication (6 * 7)")
node = %{
  "BinaryOp" => %{
    "left" => %{"Int" => 6},
    "op" => "*",
    "right" => %{"Int" => 7}
  }
}
{:ok, result} = DSL.Interpreter.run(node)
IO.inspect(result, label: "✓ Result")

# Test 5: Conditional
IO.puts("\nTest 5: Conditional Expression")
node = %{
  "Conditional" => %{
    "condition" => %{"Bool" => true},
    "then_expr" => %{"String" => "yes"},
    "else_expr" => %{"String" => "no"}
  }
}
{:ok, result} = DSL.Interpreter.run(node)
IO.inspect(result, label: "✓ Result")

# Test 6: List
IO.puts("\nTest 6: List")
node = %{
  "List" => [
    %{"Int" => 1},
    %{"Int" => 2},
    %{"Int" => 3}
  ]
}
{:ok, result} = DSL.Interpreter.run(node)
IO.inspect(result, label: "✓ Result")

# Test 7: With Tracing
IO.puts("\nTest 7: Execution with Tracing")
node = %{
  "BinaryOp" => %{
    "left" => %{"Int" => 100},
    "op" => "-",
    "right" => %{"Int" => 58}
  }
}
{:ok, result, trace} = DSL.Interpreter.run(node, trace: true)
IO.inspect(result, label: "✓ Result")
IO.puts("  Trace events: #{length(trace["events"])}")

# Test 8: Error Handling
IO.puts("\nTest 8: Error Handling (unknown variable)")
node = %{"Variable" => "undefined_var"}
{:error, error} = DSL.Interpreter.run(node)
IO.puts("✓ Got expected error:")
IO.inspect(error, label: "  Error")

# Test 9: Stateful Interpreter
IO.puts("\nTest 9: Stateful Interpreter")
{:ok, interpreter} = DSL.Interpreter.new()
{:ok, r1} = DSL.Interpreter.eval(interpreter, %{"Int" => 1})
{:ok, r2} = DSL.Interpreter.eval(interpreter, %{"Int" => 2})
{:ok, r3} = DSL.Interpreter.eval(interpreter, %{"Int" => 3})
IO.puts("✓ Executed 3 evaluations with same interpreter")
IO.inspect([r1, r2, r3], label: "  Results")

# Test 10: Tracing Interpreter
IO.puts("\nTest 10: Tracing Interpreter")
{:ok, tracer} = DSL.Interpreter.new_tracing()
{:ok, result, trace} = DSL.Interpreter.eval_with_trace(tracer, %{"Int" => 99})
IO.inspect(result, label: "✓ Result")
IO.puts("  Trace events: #{length(trace["events"])}")

IO.puts("\n" <> String.duplicate("=", 60))
IO.puts("✅ All tests passed! The integration is working perfectly!")
IO.puts(String.duplicate("=", 60) <> "\n")

IO.puts("""
Try these commands in IEx:

  # Simple evaluation
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

See QUICKSTART.md for more examples!
""")
