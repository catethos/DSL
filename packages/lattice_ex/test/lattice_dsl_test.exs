defmodule Lattice.DSLTest do
  use ExUnit.Case

  # Define test module using the DSL
  defmodule TestMath do
    use Lattice.DSL, file: "test/fixtures/math.lat"
  end

  defmodule TestMathUnwrapped do
    use Lattice.DSL, file: "test/fixtures/math.lat", unwrap: true
  end

  describe "Lattice.DSL" do
    test "generates functions from .lat file" do
      assert function_exported?(TestMath, :sum, 1)
      assert function_exported?(TestMath, :add, 2)
      assert function_exported?(TestMath, :greet, 1)
    end

    test "sum/1 works with map argument" do
      assert {:ok, 30} = TestMath.sum(%{"x" => 10, "y" => 20})
    end

    test "add/2 works with integer arguments" do
      assert {:ok, 7} = TestMath.add(3, 4)
    end

    test "greet/1 works with string argument" do
      assert {:ok, "Hello, World!"} = TestMath.greet("World")
    end

    test "__lattice_signatures__/0 returns function metadata" do
      sigs = TestMath.__lattice_signatures__()
      names = Enum.map(sigs, & &1[:name])
      assert "sum" in names
      assert "add" in names
      assert "greet" in names
    end

    test "__lattice_types__/0 returns type definitions" do
      types = TestMath.__lattice_types__()
      names = Enum.map(types, & &1[:name])
      assert "Data" in names
    end
  end

  describe "Lattice.DSL with unwrap: true" do
    test "returns values directly without :ok tuple" do
      assert 30 = TestMathUnwrapped.sum(%{"x" => 10, "y" => 20})
      assert 7 = TestMathUnwrapped.add(3, 4)
      assert "Hello, World!" = TestMathUnwrapped.greet("World")
    end
  end
end
