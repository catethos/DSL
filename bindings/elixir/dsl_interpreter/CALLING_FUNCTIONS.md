# Calling Functions with Different Inputs

## Quick Answer

You already have the `test` function loaded in your interpreter. To call it with different input:

```elixir
# Call the 'test' function with new input
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

result = DSL.Interpreter.eval(interpreter, call_test)
```

## Detailed Explanation

The `test` function is already loaded in your `interpreter` from the IR. You just need to create a new function call expression with different arguments.

### Understanding the Structure

Looking at your original `ir["entry_expr"]`:

```elixir
%{
  "FunctionCall" => %{
    "name" => "test",                    # Function name
    "args" => [                          # Arguments list
      %{
        "TemplateString" => [            # String argument
          %{"Text" => "regina is 30..."}
        ]
      }
    ],
    "effect_kind" => nil,
    "source_span" => nil
  }
}
```

To call it with different input, just change the `args` part!

## Examples

### Example 1: Different Person

```elixir
# John from NYC
john_call = %{
  "FunctionCall" => %{
    "name" => "test",
    "args" => [
      %{
        "TemplateString" => [
          %{"Text" => "John is 25 years old, lives in New York City, email: john@example.com"}
        ]
      }
    ],
    "effect_kind" => nil,
    "source_span" => nil
  }
}

{:ok, result} = DSL.Interpreter.eval(interpreter, john_call)
IO.inspect(result, pretty: true)
```

### Example 2: Alice from London

```elixir
alice_call = %{
  "FunctionCall" => %{
    "name" => "test",
    "args" => [
      %{
        "TemplateString" => [
          %{"Text" => "Alice Smith, 35 years old, London UK, alice.smith@email.co.uk"}
        ]
      }
    ],
    "effect_kind" => nil,
    "source_span" => nil
  }
}

{:ok, result} = DSL.Interpreter.eval(interpreter, alice_call)
```

### Example 3: Multiple People in a Loop

```elixir
# Define different inputs
people_texts = [
  "Bob is 40, lives in Tokyo, bob@company.jp",
  "Carol, age 28, Paris France, carol@mail.fr",
  "Dave, 33 years old, Berlin, dave@web.de"
]

# Process each one
results = Enum.map(people_texts, fn text ->
  call = %{
    "FunctionCall" => %{
      "name" => "test",
      "args" => [
        %{
          "TemplateString" => [
            %{"Text" => text}
          ]
        }
      ],
      "effect_kind" => nil,
      "source_span" => nil
    }
  }

  case DSL.Interpreter.eval(interpreter, call) do
    {:ok, result} -> {text, result}
    {:error, error} -> {text, {:error, error}}
  end
end)

# Display results
Enum.each(results, fn {text, result} ->
  IO.puts("\nInput: #{text}")
  IO.inspect(result, label: "Output")
end)
```

## Helper Function for Easy Calling

Create a helper to make this easier:

```elixir
# Define a helper function
call_test = fn interpreter, input_text ->
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

  DSL.Interpreter.eval(interpreter, call_expr)
end

# Now it's super easy to use!
{:ok, result1} = call_test.(interpreter, "Sarah is 22, lives in Boston, sarah@test.com")
{:ok, result2} = call_test.(interpreter, "Mike, age 45, Chicago IL, mike@company.com")

IO.inspect(result1, label: "Sarah")
IO.inspect(result2, label: "Mike")
```

## Better: Create a Module with Helper Functions

```elixir
defmodule TestHelper do
  @doc "Call the 'test' function with custom input"
  def call_test(interpreter, input_text) do
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

    DSL.Interpreter.eval(interpreter, call_expr)
  end

  @doc "Call test and extract just the result map"
  def extract_person(interpreter, input_text) do
    case call_test(interpreter, input_text) do
      {:ok, %{"Map" => person_data}} ->
        # Convert to a nicer Elixir map
        person = for {k, v} <- person_data, into: %{} do
          {k, extract_value(v)}
        end
        {:ok, person}

      {:error, error} ->
        {:error, error}
    end
  end

  defp extract_value(%{"String" => v}), do: v
  defp extract_value(%{"Int" => v}), do: v
  defp extract_value(%{"Float" => v}), do: v
  defp extract_value(%{"Bool" => v}), do: v
  defp extract_value(v), do: v
end

# Usage - much cleaner!
{:ok, person} = TestHelper.extract_person(
  interpreter,
  "Emma, 27, San Francisco, emma@startup.io"
)

IO.inspect(person)
# => %{
#   "name" => "Emma",
#   "age" => 27,
#   "address" => "San Francisco",
#   "email" => "emma@startup.io"
# }
```

## Testing Multiple Inputs

```elixir
# Load your IR
ir = File.read!("/Users/catethos/workspace/DSL/test.ir.json") |> Jason.decode!()
{:ok, interpreter} = DSL.Interpreter.from_ir(ir)

# Test data
test_cases = [
  "Regina is 30 yo living in Sydney, with email regina@example.com",
  "John Smith, 25 years old, NYC, john@example.com",
  "Alice, age 35, London UK, alice@email.co.uk",
  "Bob is 40, Tokyo Japan, bob@company.jp"
]

# Process all
results = Enum.map(test_cases, fn input ->
  call = %{
    "FunctionCall" => %{
      "name" => "test",
      "args" => [%{"TemplateString" => [%{"Text" => input}]}],
      "effect_kind" => nil,
      "source_span" => nil
    }
  }

  {input, DSL.Interpreter.eval(interpreter, call)}
end)

# Display results
IO.puts("\nResults:\n")
Enum.each(results, fn {input, result} ->
  IO.puts("Input: #{input}")
  case result do
    {:ok, output} -> IO.inspect(output, label: "✓ Output")
    {:error, error} -> IO.inspect(error, label: "✗ Error")
  end
  IO.puts("")
end)
```

## Pro Tip: Create a Reusable Function Call Builder

```elixir
defmodule IRBuilder do
  @doc "Build a function call expression"
  def function_call(name, args) when is_list(args) do
    %{
      "FunctionCall" => %{
        "name" => name,
        "args" => Enum.map(args, &to_ir_arg/1),
        "effect_kind" => nil,
        "source_span" => nil
      }
    }
  end

  # Convert Elixir values to IR arguments
  defp to_ir_arg(value) when is_binary(value) do
    %{"TemplateString" => [%{"Text" => value}]}
  end

  defp to_ir_arg(value) when is_integer(value) do
    %{"Int" => value}
  end

  defp to_ir_arg(value) when is_float(value) do
    %{"Float" => value}
  end

  defp to_ir_arg(value) when is_boolean(value) do
    %{"Bool" => value}
  end

  defp to_ir_arg(value) when is_map(value) do
    # Already an IR node
    value
  end
end

# Usage - super clean!
call1 = IRBuilder.function_call("test", ["Sarah is 22, Boston, sarah@test.com"])
call2 = IRBuilder.function_call("test", ["Mike, 45, Chicago, mike@company.com"])

{:ok, result1} = DSL.Interpreter.eval(interpreter, call1)
{:ok, result2} = DSL.Interpreter.eval(interpreter, call2)
```

## Complete Example Script

Save this to a file and use it in IEx:

```elixir
# File: call_test_function.exs
# Usage: import_file "call_test_function.exs"

# Load the IR
ir = File.read!("/Users/catethos/workspace/DSL/test.ir.json") |> Jason.decode!()
{:ok, interpreter} = DSL.Interpreter.from_ir(ir)

# Helper function
call_test = fn input_text ->
  call = %{
    "FunctionCall" => %{
      "name" => "test",
      "args" => [%{"TemplateString" => [%{"Text" => input_text}]}],
      "effect_kind" => nil,
      "source_span" => nil
    }
  }
  DSL.Interpreter.eval(interpreter, call)
end

IO.puts("\n=== Testing 'test' function with different inputs ===\n")

# Test 1
IO.puts("Test 1:")
{:ok, result} = call_test.("Regina is 30 yo living in Sydney, with email regina@example.com")
IO.inspect(result, pretty: true)

# Test 2
IO.puts("\nTest 2:")
{:ok, result} = call_test.("John Smith, 25, New York, john@example.com")
IO.inspect(result, pretty: true)

# Test 3
IO.puts("\nTest 3:")
{:ok, result} = call_test.("Alice, age 35, lives in London, alice@email.co.uk")
IO.inspect(result, pretty: true)

IO.puts("\n✓ All tests complete!")
IO.puts("\nThe 'call_test' function is available for you to use.")
IO.puts("Try: call_test.(\"Your custom input here\")\n")
```

## Summary

**To call the function with different input, you need to:**

1. Keep the same structure (`FunctionCall` with `name: "test"`)
2. Change the `args` array to contain your new input text
3. Call `DSL.Interpreter.eval(interpreter, your_call)`

**Quick template:**
```elixir
%{
  "FunctionCall" => %{
    "name" => "test",
    "args" => [%{"TemplateString" => [%{"Text" => "YOUR INPUT HERE"}]}],
    "effect_kind" => nil,
    "source_span" => nil
  }
}
```

The interpreter already has the function definition loaded, so you can call it as many times as you want with different inputs! 🎉
