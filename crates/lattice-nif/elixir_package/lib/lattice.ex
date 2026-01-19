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
      {:ok, _} = Lattice.eval(rt, ~s(
        def add(a: Int, b: Int) -> Int {
          a + b
        }
      ))
      {:ok, 7} = Lattice.call(rt, "add", [3, 4])

  ## SQL Support

      # Create a runtime with SQL support
      {:ok, rt} = Lattice.new(sql: true)

      # Query Lattice data with SQL
      {:ok, _} = Lattice.eval(rt, ~s(
        let users = [
          {id: 1, name: "Alice", age: 30},
          {id: 2, name: "Bob", age: 25}
        ]
      ))
      {:ok, result} = Lattice.eval(rt, ~s(SQL("SELECT * FROM users WHERE age > 25")))

  ## Value Marshaling

  Values are automatically converted between Elixir and Lattice:

  | Lattice Type | Elixir Type |
  |--------------|-------------|
  | null         | :null atom  |
  | Bool         | boolean     |
  | Int          | integer     |
  | Float        | float       |
  | String       | binary      |
  | Path         | {:lattice_path, binary} |
  | List         | list        |
  | Map          | map         |
  """

  alias Lattice.Native

  @doc """
  Create a new Lattice runtime.

  ## Options

  - `:sql` - Enable SQL support (DuckDB). Default: `false`
  - `:llm` - Enable LLM support. Default: `false`

  ## Examples

      {:ok, rt} = Lattice.new()
      {:ok, rt} = Lattice.new(sql: true)
      {:ok, rt} = Lattice.new(llm: true)
      {:ok, rt} = Lattice.new(sql: true, llm: true)
  """
  def new(opts \\ []) do
    sql = Keyword.get(opts, :sql, false)
    llm = Keyword.get(opts, :llm, false)

    cond do
      sql and llm -> Native.new_runtime_with_all()
      sql -> Native.new_runtime_with_sql()
      llm -> Native.new_runtime_with_llm()
      true -> Native.new_runtime()
    end
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

  Bindings should be a keyword list or list of `{name, value}` tuples.

  ## Examples

      {:ok, 30} = Lattice.eval_with_bindings(rt, "x + y", [{"x", 10}, {"y", 20}])
      {:ok, 30} = Lattice.eval_with_bindings(rt, "x + y", x: 10, y: 20)
  """
  def eval_with_bindings(runtime, source, bindings) do
    # Convert keyword list to list of tuples with string keys
    binding_tuples = Enum.map(bindings, fn
      {key, value} when is_atom(key) -> {Atom.to_string(key), value}
      {key, value} when is_binary(key) -> {key, value}
    end)

    Native.eval_with_bindings(runtime, source, binding_tuples)
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

  Returns `{:ok, value}` if found, `{:error, "not_found"}` otherwise.
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
