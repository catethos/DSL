defmodule Lattice do
  @moduledoc """
  High-level Elixir interface for the Lattice DSL runtime.

  ## Usage

      # Create a runtime
      {:ok, rt} = Lattice.new()

      # Evaluate expressions
      {:ok, 6} = Lattice.eval(rt, "1 + 2 + 3")
      {:ok, "hello world"} = Lattice.eval(rt, ~s("hello" + " " + "world"))

      # Define and call functions
      Lattice.eval(rt, ~s(
        def add(a: Int, b: Int) -> Int {
          a + b
        }
      ))
      {:ok, 7} = Lattice.call(rt, "add", [3, 4])

      # Define types and use them
      Lattice.eval(rt, "type Person { name: String, age: Int }")
      {:ok, person} = Lattice.eval(rt, ~s(Person { name: "Alice", age: 30 }))
      # person is %{"name" => "Alice", "age" => 30}

  ## Value Marshaling

  Values are automatically converted between Elixir and Lattice:

  | Lattice Type | Elixir Type |
  |--------------|-------------|
  | null         | :null atom  |
  | Bool         | boolean     |
  | Int          | integer     |
  | Float        | float       |
  | String       | binary      |
  | Path         | {:path, binary} |
  | List         | list        |
  | Map          | map         |
  """

  alias Lattice.Native

  @doc """
  Create a new Lattice runtime.

  The runtime is isolated - it has its own globals, functions, and types.
  """
  def new do
    Native.new_runtime()
  end

  @doc """
  Create a new Lattice runtime with LLM support.

  Requires OPENROUTER_API_KEY environment variable to be set.
  """
  def new_with_llm do
    Native.new_runtime_with_llm()
  end

  @doc """
  Create a new Lattice runtime with SQL support (DuckDB).

  Enables the SQL() function for querying CSV files and in-memory data.
  """
  def new_with_sql do
    Native.new_runtime_with_sql()
  end

  @doc """
  Create a new Lattice runtime with both LLM and SQL support.

  Requires OPENROUTER_API_KEY environment variable to be set.
  """
  def new_with_all do
    Native.new_runtime_with_all()
  end

  @doc """
  Evaluate Lattice source code.

  Returns `{:ok, value}` on success or `{:error, reason}` on failure.

  ## Examples

      {:ok, 42} = Lattice.eval(rt, "40 + 2")
      {:ok, "hello"} = Lattice.eval(rt, ~s("hello"))
      {:ok, [1, 2, 3]} = Lattice.eval(rt, "[1, 2, 3]")
  """
  def eval(runtime, source) do
    Native.eval(runtime, source)
  end

  @doc """
  Evaluate with pre-bound variables.

  ## Examples

      {:ok, 30} = Lattice.eval_with_bindings(rt, "x + y", [{"x", 10}, {"y", 20}])
  """
  def eval_with_bindings(runtime, source, bindings) do
    Native.eval_with_bindings(runtime, source, bindings)
  end

  @doc """
  Call a Lattice function by name.

  ## Examples

      # First define a function
      Lattice.eval(rt, "def double(x: Int) -> Int { x * 2 }")

      # Then call it
      {:ok, 10} = Lattice.call(rt, "double", [5])
  """
  def call(runtime, name, args) do
    Native.call_function(runtime, name, args)
  end

  @doc """
  Get all registered type schemas.

  Returns a list of type schema maps that can be used to generate
  Elixir structs or validate data.
  """
  def get_types(runtime) do
    Native.get_types(runtime)
  end

  @doc """
  Get all function signatures.

  Returns a list of function signature maps with name, params,
  return type, and metadata (is_llm, is_async).
  """
  def get_function_signatures(runtime) do
    Native.get_function_signatures(runtime)
  end

  @doc """
  Check if a function exists.
  """
  def has_function?(runtime, name) do
    Native.has_function(runtime, name)
  end

  @doc """
  Get a global variable.

  Returns `{:ok, value}` if found, `{:error, :not_found}` otherwise.
  """
  def get_global(runtime, name) do
    Native.get_global(runtime, name)
  end

  @doc """
  Set a global variable.
  """
  def set_global(runtime, name, value) do
    Native.set_global(runtime, name, value)
  end

  @doc """
  Reset the runtime, clearing all state.
  """
  def reset(runtime) do
    Native.reset(runtime)
  end
end
