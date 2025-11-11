# Quick script to call the 'test' function with custom input
# Usage in IEx:
#   import_file "call_test_with_input.exs"
#   call_test.("Your custom input text here")

IO.puts("\n" <> String.duplicate("=", 70))
IO.puts("Loading test.ir.json and setting up helper function...")
IO.puts(String.duplicate("=", 70))

# Load the IR and create interpreter
ir = File.read!("/Users/catethos/workspace/DSL/test.ir.json") |> Jason.decode!()
{:ok, interpreter} = DSL.Interpreter.from_ir(ir)

IO.puts("✓ Interpreter ready with 'test' function loaded\n")

# Create a helper function that's available in IEx
call_test = fn input_text ->
  IO.puts("\n→ Calling test(\"#{input_text}\")")

  call_expr = %{
    "FunctionCall" => %{
      "name" => "test",
      "args" => [
        %{
          "TemplateString" => [
            %{"Text" => input_text}
          ]
        }
      ],
      "effect_kind" => nil,
      "source_span" => nil
    }
  }

  case DSL.Interpreter.eval(interpreter, call_expr) do
    {:ok, result} ->
      IO.puts("✓ Success!")
      IO.inspect(result, pretty: true, label: "Result")
      {:ok, result}

    {:error, error} ->
      IO.puts("✗ Error:")
      IO.inspect(error, pretty: true)
      {:error, error}
  end
end

# Run a few examples
IO.puts(String.duplicate("-", 70))
IO.puts("Running example calls...")
IO.puts(String.duplicate("-", 70))

# Example 1: Original
call_test.("regina is 30 yo living in Sydney, with email regina@example.com")

# Example 2: John
call_test.("John Smith is 25 years old, lives in New York City, email: john@example.com")

# Example 3: Alice
call_test.("Alice, age 35, London UK, alice@email.co.uk")

IO.puts("\n" <> String.duplicate("=", 70))
IO.puts("✅ Setup complete!")
IO.puts(String.duplicate("=", 70))

IO.puts("""

The 'call_test' function is now available for you to use!

Try it with your own input:
  call_test.("Bob is 40 years old, lives in Tokyo, bob@company.jp")
  call_test.("Emma, 27, San Francisco, emma@startup.io")
  call_test.("Mike Johnson, age 45, Chicago IL, mike@company.com")

Or use any custom text:
  call_test.("Your custom person description here")

The 'interpreter' and 'ir' variables are also available if you want
to explore or build custom function calls manually.
""")
