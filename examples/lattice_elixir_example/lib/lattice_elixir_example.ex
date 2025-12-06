defmodule LatticeElixirExample do
  @moduledoc """
  Example application demonstrating Lattice embedded in Elixir.

  This module shows various ways to use the Lattice runtime from Elixir:
  - Basic evaluation
  - Function definitions and calls
  - Type definitions
  - Working with globals
  - LLM integration (optional)
  """

  @doc """
  Run all examples and print results.
  """
  def run_examples do
    IO.puts("=== Lattice Elixir Example ===\n")

    # Create a runtime
    {:ok, rt} = Lattice.new()
    IO.puts("Created Lattice runtime\n")

    # Example 1: Basic arithmetic
    basic_arithmetic(rt)

    # Example 2: String operations
    string_operations(rt)

    # Example 3: Lists and maps
    collections(rt)

    # Example 4: Function definitions
    functions(rt)

    # Example 5: Type definitions
    types(rt)

    # Example 6: Globals
    globals(rt)

    IO.puts("\n=== All examples completed! ===")
  end

  defp basic_arithmetic(rt) do
    IO.puts("--- Basic Arithmetic ---")

    {:ok, result} = Lattice.eval(rt, "1 + 2 + 3")
    IO.puts("1 + 2 + 3 = #{inspect(result)}")

    {:ok, result} = Lattice.eval(rt, "10 * 5 - 3")
    IO.puts("10 * 5 - 3 = #{inspect(result)}")

    {:ok, result} = Lattice.eval(rt, "2.5 * 4.0")
    IO.puts("2.5 * 4.0 = #{inspect(result)}")

    IO.puts("")
  end

  defp string_operations(rt) do
    IO.puts("--- String Operations ---")

    {:ok, result} = Lattice.eval(rt, ~s["hello" + " " + "world"])
    IO.puts(~s["hello" + " " + "world" = #{inspect(result)}])

    {:ok, result} = Lattice.eval(rt, ~s[len("testing")])
    IO.puts(~s[len("testing") = #{inspect(result)}])

    {:ok, result} = Lattice.eval(rt, ~s[str(42)])
    IO.puts(~s[str(42) = #{inspect(result)}])

    IO.puts("")
  end

  defp collections(rt) do
    IO.puts("--- Collections ---")

    # Lists
    {:ok, result} = Lattice.eval(rt, "[1, 2, 3, 4, 5]")
    IO.puts("[1, 2, 3, 4, 5] = #{inspect(result)}")

    # List indexing
    {:ok, result} = Lattice.eval(rt, "[10, 20, 30][1]")
    IO.puts("[10, 20, 30][1] = #{inspect(result)}")

    # Push and length
    {:ok, result} = Lattice.eval(rt, "len([1, 2, 3, 4])")
    IO.puts("len([1, 2, 3, 4]) = #{inspect(result)}")

    # Maps
    {:ok, result} = Lattice.eval(rt, ~s/{"name": "Alice", "age": 30}/)
    IO.puts(~s/{"name": "Alice", "age": 30} = #{inspect(result)}/)

    # Map field access
    {:ok, result} = Lattice.eval(rt, ~s/{"name": "Alice", "age": 30}["name"]/)
    IO.puts(~s/{"name": "Alice", "age": 30}["name"] = #{inspect(result)}/)

    IO.puts("")
  end

  defp functions(rt) do
    IO.puts("--- Functions ---")

    # Define a simple function
    {:ok, _} =
      Lattice.eval(rt, """
        def add(a: Int, b: Int) -> Int {
          a + b
        }
      """)

    IO.puts("Defined: def add(a: Int, b: Int) -> Int { a + b }")

    # Call the function
    {:ok, result} = Lattice.call(rt, "add", [10, 20])
    IO.puts("add(10, 20) = #{inspect(result)}")

    # Define another function
    {:ok, _} =
      Lattice.eval(rt, """
        def greet(name: String) -> String {
          "Hello, " + name + "!"
        }
      """)

    {:ok, result} = Lattice.call(rt, "greet", ["Elixir"])
    IO.puts(~s[greet("Elixir") = #{inspect(result)}])

    # List functions
    sigs = Lattice.get_function_signatures(rt)
    IO.puts("\nRegistered functions:")

    for sig <- sigs do
      IO.puts("  - #{sig.name}")
    end

    IO.puts("")
  end

  defp types(rt) do
    IO.puts("--- Types ---")

    # Define a struct type
    {:ok, _} =
      Lattice.eval(rt, """
        type Person {
          name: String,
          age: Int
        }
      """)

    IO.puts("Defined: type Person { name: String, age: Int }")

    # Create an instance
    {:ok, person} = Lattice.eval(rt, ~s/Person { name: "Bob", age: 25 }/)
    IO.puts(~s/Person { name: "Bob", age: 25 } = #{inspect(person)}/)

    # Define an enum
    {:ok, _} =
      Lattice.eval(rt, """
        enum Status { Active, Pending, Done }
      """)

    IO.puts("Defined: enum Status { Active, Pending, Done }")

    {:ok, status} = Lattice.eval(rt, "Status::Active")
    IO.puts("Status::Active = #{inspect(status)}")

    # List types
    types = Lattice.get_types(rt)
    IO.puts("\nRegistered types:")

    for type <- types do
      case type.type_schema do
        :struct_type -> IO.puts("  - struct #{type.name}")
        :enum_type -> IO.puts("  - enum #{type.name}")
        _ -> :ok
      end
    end

    IO.puts("")
  end

  defp globals(rt) do
    IO.puts("--- Globals ---")

    # Set a global from Elixir
    :ok = Lattice.set_global(rt, "multiplier", 100)
    IO.puts("Set global: multiplier = 100")

    # Use it in Lattice code
    {:ok, _} =
      Lattice.eval(rt, """
        def scale(x: Int) -> Int {
          x * multiplier
        }
      """)

    {:ok, result} = Lattice.call(rt, "scale", [5])
    IO.puts("scale(5) = #{inspect(result)} (using multiplier global)")

    # Get global back
    {:ok, value} = Lattice.get_global(rt, "multiplier")
    IO.puts("Get global multiplier = #{inspect(value)}")

    IO.puts("")
  end
end
