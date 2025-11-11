# Helper module for working with test.ir.json
# Usage: import_file "test_helper.exs"

defmodule TestHelper do
  @moduledoc """
  Helper functions for calling the 'test' function from test.ir.json
  with different inputs.
  """

  @doc """
  Call the 'test' function with custom input text.

  ## Examples

      TestHelper.call("John is 25, NYC, john@example.com")
      TestHelper.call("Alice, age 35, London, alice@email.co.uk")
  """
  def call(interpreter, input_text) do
    call_expr = build_call("test", input_text)
    DSL.Interpreter.eval(interpreter, call_expr)
  end

  @doc """
  Call the 'test' function and extract a clean Elixir map.

  Returns a simplified map with string keys and native Elixir values.

  ## Examples

      {:ok, person} = TestHelper.extract(interpreter, "Bob, 40, Tokyo, bob@jp.com")
      # => {:ok, %{"name" => "Bob", "age" => 40, "address" => "Tokyo", ...}}
  """
  def extract(interpreter, input_text) do
    case call(interpreter, input_text) do
      {:ok, %{"Map" => person_data}} ->
        person = for {k, v} <- person_data, into: %{} do
          {k, unwrap_value(v)}
        end
        {:ok, person}

      {:error, error} ->
        {:error, error}
    end
  end

  @doc """
  Process multiple inputs in batch.

  ## Examples

      inputs = [
        "John, 25, NYC, john@example.com",
        "Alice, 35, London, alice@email.co.uk"
      ]
      TestHelper.batch(interpreter, inputs)
  """
  def batch(interpreter, input_texts) when is_list(input_texts) do
    Enum.map(input_texts, fn text ->
      {text, call(interpreter, text)}
    end)
  end

  @doc """
  Process batch and print results nicely.
  """
  def batch_print(interpreter, input_texts) do
    results = batch(interpreter, input_texts)

    IO.puts("\n" <> String.duplicate("=", 70))
    IO.puts("Batch Processing Results (#{length(results)} inputs)")
    IO.puts(String.duplicate("=", 70) <> "\n")

    Enum.with_index(results, 1) |> Enum.each(fn {{text, result}, idx} ->
      IO.puts("#{idx}. Input: #{text}")
      case result do
        {:ok, output} ->
          IO.puts("   ✓ Success")
          IO.inspect(output, label: "   Output", limit: 5)
        {:error, error} ->
          IO.puts("   ✗ Error: #{error.message}")
      end
      IO.puts("")
    end)
  end

  @doc """
  Build a function call expression for any function.
  """
  def build_call(function_name, input_text) do
    %{
      "FunctionCall" => %{
        "name" => function_name,
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
  end

  # Private helper to unwrap IR values to native Elixir
  defp unwrap_value(%{"String" => v}), do: v
  defp unwrap_value(%{"Int" => v}), do: v
  defp unwrap_value(%{"Float" => v}), do: v
  defp unwrap_value(%{"Bool" => v}), do: v
  defp unwrap_value(%{"List" => v}), do: Enum.map(v, &unwrap_value/1)
  defp unwrap_value(%{"Map" => v}) do
    for {k, val} <- v, into: %{}, do: {k, unwrap_value(val)}
  end
  defp unwrap_value("Null"), do: nil
  defp unwrap_value(v), do: v
end

# Auto-load the IR when this file is imported
IO.puts("\n" <> String.duplicate("=", 70))
IO.puts("TestHelper Module - Loading test.ir.json...")
IO.puts(String.duplicate("=", 70))

ir = File.read!("/Users/catethos/workspace/DSL/test.ir.json") |> Jason.decode!()
{:ok, interpreter} = DSL.Interpreter.from_ir(ir)

IO.puts("✓ test.ir.json loaded")
IO.puts("✓ Interpreter created")
IO.puts("✓ TestHelper module available\n")

# Make interpreter available as a module attribute for convenience
defmodule Global do
  def interpreter, do: Process.get(:test_interpreter)
  def set_interpreter(interp), do: Process.put(:test_interpreter, interp)
end

Global.set_interpreter(interpreter)

IO.puts(String.duplicate("-", 70))
IO.puts("Quick Examples:")
IO.puts(String.duplicate("-", 70))

# Example 1
IO.puts("\n1. Simple call:")
IO.puts("   TestHelper.call(interpreter, \"Bob, 40, Tokyo, bob@jp.com\")")

{:ok, result1} = TestHelper.call(interpreter, "Bob is 40 years old, Tokyo, bob@jp.com")
IO.inspect(result1, label: "   Result", limit: 5)

# Example 2
IO.puts("\n2. Extract clean map:")
IO.puts("   TestHelper.extract(interpreter, \"Emma, 27, SF, emma@startup.io\")")

{:ok, person} = TestHelper.extract(interpreter, "Emma, 27 years old, San Francisco, emma@startup.io")
IO.inspect(person, label: "   Person")

# Example 3
IO.puts("\n3. Batch processing:")
IO.puts("   TestHelper.batch_print(interpreter, [...])")

test_inputs = [
  "Regina is 30 yo living in Sydney, regina@example.com",
  "John Smith, 25, NYC, john@example.com"
]

TestHelper.batch_print(interpreter, test_inputs)

IO.puts(String.duplicate("=", 70))
IO.puts("Ready to use!")
IO.puts(String.duplicate("=", 70))

IO.puts("""

Available functions:
  TestHelper.call(interpreter, "text")          - Call test function
  TestHelper.extract(interpreter, "text")       - Get clean map
  TestHelper.batch(interpreter, ["t1", "t2"])   - Process multiple
  TestHelper.batch_print(interpreter, [...])    - Process and print

Variables available:
  interpreter  - The DSL interpreter with test.ir.json loaded
  ir           - The parsed IR map

Try it yourself:
  TestHelper.call(interpreter, "Sarah, 22, Boston, sarah@test.com")
  TestHelper.extract(interpreter, "Mike, 45, Chicago, mike@co.com")
""")
