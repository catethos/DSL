defmodule Lattice.Native do
  @moduledoc """
  Low-level NIF bindings for the Lattice runtime.

  This module provides direct access to the Rust NIF functions.
  For most use cases, use the higher-level `Lattice` module instead.
  """

  use Rustler,
    otp_app: :lattice_elixir_example,
    crate: "lattice_nif"

  # NIF function stubs - these are replaced when the NIF is loaded

  @doc """
  Create a new Lattice runtime instance (without LLM support).
  """
  def new_runtime(), do: :erlang.nif_error(:nif_not_loaded)

  @doc """
  Create a new Lattice runtime instance with LLM support.
  Requires OPENROUTER_API_KEY environment variable.
  """
  def new_runtime_with_llm(), do: :erlang.nif_error(:nif_not_loaded)

  @doc """
  Evaluate Lattice source code.
  Returns `{:ok, value}` or `{:error, reason}`.
  """
  def eval(_runtime, _source), do: :erlang.nif_error(:nif_not_loaded)

  @doc """
  Evaluate Lattice source code with pre-bound variables.
  Bindings is a list of `{name, value}` tuples.
  """
  def eval_with_bindings(_runtime, _source, _bindings), do: :erlang.nif_error(:nif_not_loaded)

  @doc """
  Call a Lattice function by name with arguments.
  """
  def call_function(_runtime, _name, _args), do: :erlang.nif_error(:nif_not_loaded)

  @doc """
  Get all registered type schemas.
  """
  def get_types(_runtime), do: :erlang.nif_error(:nif_not_loaded)

  @doc """
  Get all function signatures.
  """
  def get_function_signatures(_runtime), do: :erlang.nif_error(:nif_not_loaded)

  @doc """
  Check if a function exists.
  """
  def has_function(_runtime, _name), do: :erlang.nif_error(:nif_not_loaded)

  @doc """
  Get a global variable value.
  """
  def get_global(_runtime, _name), do: :erlang.nif_error(:nif_not_loaded)

  @doc """
  Set a global variable.
  """
  def set_global(_runtime, _name, _value), do: :erlang.nif_error(:nif_not_loaded)

  @doc """
  Reset the runtime, clearing all state.
  """
  def reset(_runtime), do: :erlang.nif_error(:nif_not_loaded)
end
