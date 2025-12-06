defmodule Lattice do
  @moduledoc """
  High-level Elixir interface for the Lattice DSL runtime.

  Lattice is a domain-specific language for working with structured data
  and LLM calls. This module provides an Elixir interface to embed the
  Lattice runtime in your application.

  ## Getting Started

      # Create a runtime
      {:ok, rt} = Lattice.new()

      # Evaluate expressions
      {:ok, 6} = Lattice.eval(rt, "1 + 2 + 3")

      # Define and call functions
      :ok = Lattice.eval(rt, \"\"\"
        def add(a: Int, b: Int) -> Int {
          a + b
        }
      \"\"\")
      {:ok, 7} = Lattice.call(rt, "add", [3, 4])

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

  ## Type System

  Lattice has a rich type system with structs and enums:

      # Define a type
      Lattice.eval(rt, "type Person { name: String, age: Int }")

      # Create instances
      {:ok, person} = Lattice.eval(rt, ~s(Person { name: "Alice", age: 30 }))
      # => %{"name" => "Alice", "age" => 30}

      # Enums
      Lattice.eval(rt, "enum Status { Active, Pending, Done }")
  """

  alias Lattice.Native

  @doc """
  Create a new Lattice runtime.

  The runtime is isolated - each instance has its own globals, functions,
  and type definitions.

  ## Examples

      {:ok, rt} = Lattice.new()
  """
  @spec new() :: {:ok, reference()} | {:error, String.t()}
  def new do
    Native.new_runtime()
  end

  @doc """
  Create a new Lattice runtime with LLM support.

  This enables `llm` functions that call language models.
  Requires the `OPENROUTER_API_KEY` environment variable to be set.

  ## Examples

      {:ok, rt} = Lattice.new_with_llm()

      # Now you can use LLM functions
      Lattice.eval(rt, \"\"\"
        llm def summarize(text: String) -> String
      \"\"\")
      {:ok, summary} = Lattice.call(rt, "summarize", ["Long article text..."])
  """
  @spec new_with_llm() :: {:ok, reference()} | {:error, String.t()}
  def new_with_llm do
    Native.new_runtime_with_llm()
  end

  @doc """
  Evaluate Lattice source code.

  Returns `{:ok, value}` on success or `{:error, reason}` on failure.

  ## Examples

      # Arithmetic
      {:ok, 42} = Lattice.eval(rt, "40 + 2")

      # Strings
      {:ok, "hello world"} = Lattice.eval(rt, ~s("hello" + " " + "world"))

      # Lists
      {:ok, [1, 2, 3]} = Lattice.eval(rt, "[1, 2, 3]")

      # Maps
      {:ok, %{"a" => 1}} = Lattice.eval(rt, ~s({"a": 1}))

      # Variables persist across calls
      Lattice.eval(rt, "let x = 10")
      {:ok, 20} = Lattice.eval(rt, "x * 2")
  """
  @spec eval(reference(), String.t()) :: {:ok, any()} | {:error, String.t()}
  def eval(runtime, source) do
    Native.eval(runtime, source)
  end

  @doc """
  Evaluate Lattice source code with pre-bound variables.

  ## Examples

      {:ok, 30} = Lattice.eval_with_bindings(rt, "x + y", [{"x", 10}, {"y", 20}])

      # Complex bindings
      {:ok, "Hello Alice"} = Lattice.eval_with_bindings(
        rt,
        ~s("Hello " + name),
        [{"name", "Alice"}]
      )
  """
  @spec eval_with_bindings(reference(), String.t(), [{String.t(), any()}]) ::
          {:ok, any()} | {:error, String.t()}
  def eval_with_bindings(runtime, source, bindings) do
    Native.eval_with_bindings(runtime, source, bindings)
  end

  @doc """
  Call a Lattice function by name with arguments.

  ## Examples

      # First define a function
      Lattice.eval(rt, \"\"\"
        def greet(name: String) -> String {
          "Hello, " + name + "!"
        }
      \"\"\")

      # Then call it
      {:ok, "Hello, World!"} = Lattice.call(rt, "greet", ["World"])

      # Multiple arguments
      Lattice.eval(rt, "def add(a: Int, b: Int) -> Int { a + b }")
      {:ok, 7} = Lattice.call(rt, "add", [3, 4])
  """
  @spec call(reference(), String.t(), [any()]) :: {:ok, any()} | {:error, String.t()}
  def call(runtime, name, args) do
    Native.call_function(runtime, name, args)
  end

  @doc """
  Get all registered type schemas.

  Returns a list of type schema maps. Useful for generating Elixir
  structs or validating data.

  ## Examples

      Lattice.eval(rt, "type Person { name: String, age: Int }")
      types = Lattice.get_types(rt)
      # => [%{type_schema: :struct_type, name: "Person", fields: [...]}]
  """
  @spec get_types(reference()) :: [map()]
  def get_types(runtime) do
    Native.get_types(runtime)
  end

  @doc """
  Get all function signatures.

  Returns a list of function signature maps with:
  - `name`: Function name
  - `params`: List of parameter maps with name and type
  - `return_type`: Return type schema
  - `is_llm`: Whether this is an LLM function
  - `is_async`: Whether this function is async

  ## Examples

      Lattice.eval(rt, "def add(a: Int, b: Int) -> Int { a + b }")
      sigs = Lattice.get_function_signatures(rt)
      # => [%{name: "add", params: [...], return_type: %{type_schema: :int}, ...}]
  """
  @spec get_function_signatures(reference()) :: [map()]
  def get_function_signatures(runtime) do
    Native.get_function_signatures(runtime)
  end

  @doc """
  Check if a function exists.

  ## Examples

      Lattice.eval(rt, "def foo() -> Int { 42 }")
      true = Lattice.has_function?(rt, "foo")
      false = Lattice.has_function?(rt, "bar")
  """
  @spec has_function?(reference(), String.t()) :: boolean()
  def has_function?(runtime, name) do
    Native.has_function(runtime, name)
  end

  @doc """
  Get a global variable value.

  Returns `{:ok, value}` if found, `{:error, "not_found"}` otherwise.

  ## Examples

      Lattice.eval(rt, "let count = 42")
      {:ok, 42} = Lattice.get_global(rt, "count")
      {:error, "not_found"} = Lattice.get_global(rt, "missing")
  """
  @spec get_global(reference(), String.t()) :: {:ok, any()} | {:error, String.t()}
  def get_global(runtime, name) do
    Native.get_global(runtime, name)
  end

  @doc """
  Set a global variable.

  ## Examples

      :ok = Lattice.set_global(rt, "multiplier", 10)
      Lattice.eval(rt, "def scale(x: Int) -> Int { x * multiplier }")
      {:ok, 50} = Lattice.call(rt, "scale", [5])
  """
  @spec set_global(reference(), String.t(), any()) :: :ok
  def set_global(runtime, name, value) do
    Native.set_global(runtime, name, value)
  end

  @doc """
  Reset the runtime, clearing all state.

  After reset, all globals, functions, and types are cleared.

  ## Examples

      Lattice.eval(rt, "let x = 42")
      :ok = Lattice.reset(rt)
      {:error, _} = Lattice.eval(rt, "x")  # x no longer exists
  """
  @spec reset(reference()) :: :ok
  def reset(runtime) do
    Native.reset(runtime)
  end
end
