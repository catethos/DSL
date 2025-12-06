# Example Elixir module for using lattice-nif
#
# This file shows how to integrate the Lattice NIF into an Elixir application.
# You would typically copy this into your Elixir project's lib/ directory.
#
# Prerequisites:
# 1. Add rustler to your mix.exs dependencies:
#    {:rustler, "~> 0.34"}
#
# 2. Create a Native module as shown below
#
# 3. Configure rustler in mix.exs:
#    def project do
#      [
#        ...,
#        rustler_crates: [lattice_nif: [path: "path/to/lattice-nif"]]
#      ]
#    end

defmodule Lattice.Native do
  @moduledoc """
  NIF bindings for the Lattice runtime.

  This module provides low-level access to the Lattice runtime through
  Rustler NIFs. For most use cases, use the higher-level `Lattice` module.
  """

  use Rustler, otp_app: :my_app, crate: "lattice_nif"

  # NIF function stubs - these are replaced when the NIF is loaded
  def new_runtime(), do: :erlang.nif_error(:nif_not_loaded)
  def new_runtime_with_llm(), do: :erlang.nif_error(:nif_not_loaded)
  def eval(_runtime, _source), do: :erlang.nif_error(:nif_not_loaded)
  def eval_with_bindings(_runtime, _source, _bindings), do: :erlang.nif_error(:nif_not_loaded)
  def call_function(_runtime, _name, _args), do: :erlang.nif_error(:nif_not_loaded)
  def get_types(_runtime), do: :erlang.nif_error(:nif_not_loaded)
  def get_function_signatures(_runtime), do: :erlang.nif_error(:nif_not_loaded)
  def has_function(_runtime, _name), do: :erlang.nif_error(:nif_not_loaded)
  def get_global(_runtime, _name), do: :erlang.nif_error(:nif_not_loaded)
  def set_global(_runtime, _name, _value), do: :erlang.nif_error(:nif_not_loaded)
  def reset(_runtime), do: :erlang.nif_error(:nif_not_loaded)
end

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
      :ok = Lattice.eval(rt, ~s(
        def add(a: Int, b: Int) -> Int {
          a + b
        }
      ))
      {:ok, 7} = Lattice.call(rt, "add", [3, 4])

      # Define types and use them
      :ok = Lattice.eval(rt, "type Person { name: String, age: Int }")
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

# Example usage in IEx:
#
# iex> {:ok, rt} = Lattice.new()
# {:ok, #Reference<0.123.456.789>}
#
# iex> {:ok, result} = Lattice.eval(rt, "1 + 2 + 3")
# {:ok, 6}
#
# iex> {:ok, result} = Lattice.eval(rt, "[1, 2, 3] |> map(fn x -> x * 2)")
# {:ok, [2, 4, 6]}
#
# iex> Lattice.eval(rt, "type User { name: String, email: String }")
# {:ok, :null}
#
# iex> Lattice.get_types(rt)
# [%{type_schema: :struct_type, name: "User", fields: [...]}]
