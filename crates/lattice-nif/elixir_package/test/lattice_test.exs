defmodule LatticeTest do
  use ExUnit.Case

  describe "Basic Eval" do
    test "arithmetic" do
      {:ok, rt} = Lattice.new()
      {:ok, result} = Lattice.eval(rt, "1 + 2 * 3")
      assert result == 7
    end

    test "string concatenation" do
      {:ok, rt} = Lattice.new()
      {:ok, result} = Lattice.eval(rt, ~s("hello" + " world"))
      assert result == "hello world"
    end

    test "list" do
      {:ok, rt} = Lattice.new()
      {:ok, result} = Lattice.eval(rt, "[1, 2, 3]")
      assert result == [1, 2, 3]
    end

    test "map" do
      {:ok, rt} = Lattice.new()
      {:ok, result} = Lattice.eval(rt, ~s({"a": 1, "b": 2}))
      assert result == %{"a" => 1, "b" => 2}
    end

    test "null" do
      {:ok, rt} = Lattice.new()
      {:ok, result} = Lattice.eval(rt, "null")
      assert result == :null
    end

    test "bool true" do
      {:ok, rt} = Lattice.new()
      {:ok, result} = Lattice.eval(rt, "true")
      assert result == true
    end

    test "bool false" do
      {:ok, rt} = Lattice.new()
      {:ok, result} = Lattice.eval(rt, "false")
      assert result == false
    end

    test "float" do
      {:ok, rt} = Lattice.new()
      {:ok, result} = Lattice.eval(rt, "3.14")
      assert_in_delta result, 3.14, 0.001
    end

    test "negative int" do
      {:ok, rt} = Lattice.new()
      {:ok, result} = Lattice.eval(rt, "-42")
      assert result == -42
    end

    test "nested list" do
      {:ok, rt} = Lattice.new()
      {:ok, result} = Lattice.eval(rt, "[[1, 2], [3, 4]]")
      assert result == [[1, 2], [3, 4]]
    end

    test "nested map" do
      {:ok, rt} = Lattice.new()
      {:ok, result} = Lattice.eval(rt, ~s({"outer": {"inner": 1}}))
      assert result == %{"outer" => %{"inner" => 1}}
    end
  end

  describe "Bindings" do
    test "eval with bindings" do
      {:ok, rt} = Lattice.new()
      {:ok, result} = Lattice.eval_with_bindings(rt, "x + y", [{"x", 10}, {"y", 20}])
      assert result == 30
    end

    test "bindings with keyword list" do
      {:ok, rt} = Lattice.new()
      {:ok, result} = Lattice.eval_with_bindings(rt, "x + y", x: 10, y: 20)
      assert result == 30
    end

    test "bindings with list" do
      {:ok, rt} = Lattice.new()
      {:ok, result} = Lattice.eval_with_bindings(rt, "items[0] + items[1]", [{"items", [3, 4]}])
      assert result == 7
    end

    test "bindings with map" do
      {:ok, rt} = Lattice.new()
      {:ok, result} = Lattice.eval_with_bindings(rt, ~s(data["key"]), [{"data", %{"key" => 42}}])
      assert result == 42
    end

    test "bindings with null" do
      {:ok, rt} = Lattice.new()
      {:ok, result} = Lattice.eval_with_bindings(rt, "x", [{"x", :null}])
      assert result == :null
    end

    test "bindings with bool" do
      {:ok, rt} = Lattice.new()
      {:ok, result} = Lattice.eval_with_bindings(rt, "x", [{"x", true}])
      assert result == true
    end
  end

  describe "Globals" do
    test "set and get global" do
      {:ok, rt} = Lattice.new()
      :ok = Lattice.set_global(rt, "count", 42)
      {:ok, result} = Lattice.get_global(rt, "count")
      assert result == 42
    end

    test "global in eval" do
      {:ok, rt} = Lattice.new()
      :ok = Lattice.set_global(rt, "x", 100)
      {:ok, result} = Lattice.eval(rt, "x * 2")
      assert result == 200
    end

    test "get undefined global returns error" do
      {:ok, rt} = Lattice.new()
      {:error, "not_found"} = Lattice.get_global(rt, "undefined_var")
    end

    test "overwrite global" do
      {:ok, rt} = Lattice.new()
      :ok = Lattice.set_global(rt, "x", 1)
      :ok = Lattice.set_global(rt, "x", 2)
      {:ok, result} = Lattice.get_global(rt, "x")
      assert result == 2
    end
  end

  describe "Functions" do
    test "define and call" do
      {:ok, rt} = Lattice.new()
      {:ok, _} = Lattice.eval(rt, "def add(a: Int, b: Int) -> Int { a + b }")
      {:ok, result} = Lattice.call(rt, "add", [3, 4])
      assert result == 7
    end

    test "has function" do
      {:ok, rt} = Lattice.new()
      {:ok, _} = Lattice.eval(rt, "def foo() -> Int { 42 }")
      assert Lattice.has_function?(rt, "foo") == true
      assert Lattice.has_function?(rt, "bar") == false
    end

    test "function signatures" do
      {:ok, rt} = Lattice.new()
      {:ok, _} = Lattice.eval(rt, "def greet(name: String) -> String { name }")
      sigs = Lattice.get_function_signatures(rt)
      assert length(sigs) == 1
      sig = hd(sigs)
      assert sig.name == "greet"
      assert length(sig.params) == 1
      # Note: Parameter names are not preserved in CompiledFunction (core limitation)
      # The param name will be "arg0" instead of "name"
    end

    test "multiple functions" do
      {:ok, rt} = Lattice.new()
      {:ok, _} = Lattice.eval(rt, "def f1() -> Int { 1 }")
      {:ok, _} = Lattice.eval(rt, "def f2() -> Int { 2 }")
      assert Lattice.has_function?(rt, "f1") == true
      assert Lattice.has_function?(rt, "f2") == true
      {:ok, r1} = Lattice.call(rt, "f1", [])
      {:ok, r2} = Lattice.call(rt, "f2", [])
      assert r1 == 1
      assert r2 == 2
    end
  end

  describe "Types" do
    test "struct" do
      {:ok, rt} = Lattice.new()
      {:ok, _} = Lattice.eval(rt, "type Person { name: String, age: Int }")
      types = Lattice.get_types(rt)
      assert length(types) == 1
      type = hd(types)
      assert type.type_schema == :struct_type
      assert type.name == "Person"
      assert length(type.fields) == 2
    end

    test "enum" do
      {:ok, rt} = Lattice.new()
      {:ok, _} = Lattice.eval(rt, "enum Color { Red, Green, Blue }")
      types = Lattice.get_types(rt)
      assert length(types) == 1
      type = hd(types)
      assert type.type_schema == :enum_type
      assert type.name == "Color"
      assert type.variants == ["Red", "Green", "Blue"]
    end
  end

  describe "Reset" do
    test "reset clears globals" do
      {:ok, rt} = Lattice.new()
      :ok = Lattice.set_global(rt, "x", 42)
      :ok = Lattice.reset(rt)
      {:error, "not_found"} = Lattice.get_global(rt, "x")
    end

    test "reset clears functions" do
      {:ok, rt} = Lattice.new()
      {:ok, _} = Lattice.eval(rt, "def foo() -> Int { 42 }")
      assert Lattice.has_function?(rt, "foo") == true
      :ok = Lattice.reset(rt)
      assert Lattice.has_function?(rt, "foo") == false
    end
  end

  describe "Errors" do
    test "syntax error" do
      {:ok, rt} = Lattice.new()
      {:error, _reason} = Lattice.eval(rt, "1 +")
    end

    test "undefined variable" do
      {:ok, rt} = Lattice.new()
      {:error, _reason} = Lattice.eval(rt, "undefined_var")
    end

    test "undefined function" do
      {:ok, rt} = Lattice.new()
      {:error, _reason} = Lattice.call(rt, "nonexistent", [])
    end

    test "type error" do
      {:ok, rt} = Lattice.new()
      {:error, _reason} = Lattice.eval(rt, ~s("hello" + 1))
    end
  end
end
