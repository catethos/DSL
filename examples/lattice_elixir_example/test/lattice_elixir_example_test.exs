defmodule LatticeElixirExampleTest do
  use ExUnit.Case

  describe "Lattice runtime" do
    test "creates a new runtime" do
      assert {:ok, rt} = Lattice.new()
      assert is_reference(rt)
    end

    test "evaluates simple arithmetic" do
      {:ok, rt} = Lattice.new()
      assert {:ok, 6} = Lattice.eval(rt, "1 + 2 + 3")
      assert {:ok, 47} = Lattice.eval(rt, "10 * 5 - 3")
    end

    test "evaluates strings" do
      {:ok, rt} = Lattice.new()
      assert {:ok, "hello world"} = Lattice.eval(rt, ~s("hello" + " " + "world"))
    end

    test "evaluates lists" do
      {:ok, rt} = Lattice.new()
      assert {:ok, [1, 2, 3]} = Lattice.eval(rt, "[1, 2, 3]")
    end

    test "evaluates maps" do
      {:ok, rt} = Lattice.new()
      assert {:ok, %{"a" => 1, "b" => 2}} = Lattice.eval(rt, ~s({"a": 1, "b": 2}))
    end

    test "evaluates with bindings" do
      {:ok, rt} = Lattice.new()

      assert {:ok, 30} =
               Lattice.eval_with_bindings(rt, "x + y", [{"x", 10}, {"y", 20}])
    end

    test "defines and calls functions" do
      {:ok, rt} = Lattice.new()

      # Define function
      {:ok, _} =
        Lattice.eval(rt, """
          def add(a: Int, b: Int) -> Int {
            a + b
          }
        """)

      # Call function
      assert {:ok, 7} = Lattice.call(rt, "add", [3, 4])
    end

    test "function existence check" do
      {:ok, rt} = Lattice.new()
      {:ok, _} = Lattice.eval(rt, "def foo() -> Int { 42 }")

      assert Lattice.has_function?(rt, "foo")
      refute Lattice.has_function?(rt, "bar")
    end

    test "global variables" do
      {:ok, rt} = Lattice.new()

      # Set global
      :ok = Lattice.set_global(rt, "x", 42)

      # Get global
      assert {:ok, 42} = Lattice.get_global(rt, "x")

      # Non-existent global
      assert {:error, "not_found"} = Lattice.get_global(rt, "nonexistent")
    end

    test "type definitions" do
      {:ok, rt} = Lattice.new()

      {:ok, _} =
        Lattice.eval(rt, """
          type Person { name: String, age: Int }
        """)

      types = Lattice.get_types(rt)
      assert length(types) == 1

      [type] = types
      assert type.type_schema == :struct_type
      assert type.name == "Person"
    end

    test "reset clears state" do
      {:ok, rt} = Lattice.new()

      {:ok, _} = Lattice.eval(rt, "let x = 42")
      :ok = Lattice.reset(rt)

      # After reset, x should not exist
      assert {:error, _} = Lattice.eval(rt, "x")
    end

    test "function signatures" do
      {:ok, rt} = Lattice.new()

      {:ok, _} =
        Lattice.eval(rt, """
          def add(a: Int, b: Int) -> Int { a + b }
          def greet(name: String) -> String { "Hello " + name }
        """)

      sigs = Lattice.get_function_signatures(rt)
      assert length(sigs) == 2

      names = Enum.map(sigs, & &1.name)
      assert "add" in names
      assert "greet" in names
    end

    test "runtime isolation" do
      # Create two separate runtimes
      {:ok, rt1} = Lattice.new()
      {:ok, rt2} = Lattice.new()

      # Define different values in each
      :ok = Lattice.set_global(rt1, "x", 100)
      :ok = Lattice.set_global(rt2, "x", 200)

      # They should be independent
      assert {:ok, 100} = Lattice.get_global(rt1, "x")
      assert {:ok, 200} = Lattice.get_global(rt2, "x")
    end
  end
end
