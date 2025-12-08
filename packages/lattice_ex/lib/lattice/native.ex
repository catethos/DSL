defmodule Lattice.Native do
  @moduledoc """
  NIF bindings for the Lattice runtime.

  This module provides low-level access to the Lattice runtime through
  precompiled Rustler NIFs. For most use cases, use the higher-level `Lattice` module.
  """

  @version Mix.Project.config()[:version]
  @nif_filename "liblattice_nif-v#{@version}-nif-2.17-aarch64-apple-darwin.so"

  # Check if local dev build exists
  if File.exists?(Path.join(["priv", "native", @nif_filename])) do
    # Local development mode - load pre-built NIF directly
    @on_load :load_nif

    @doc false
    def load_nif do
      nif_name = "liblattice_nif-v#{unquote(@version)}-nif-2.17-aarch64-apple-darwin"
      path = :filename.join(:code.priv_dir(:lattice), ~c"native/#{nif_name}")
      :erlang.load_nif(path, 0)
    end
  else
    # Production mode - download from GitHub releases
    use RustlerPrecompiled,
      otp_app: :lattice,
      crate: "lattice_nif",
      base_url: "https://github.com/catethos/DSL/releases/download/v#{@version}",
      targets: ["aarch64-apple-darwin"],
      nif_versions: ["2.17"],
      version: @version
  end

  # NIF function stubs - replaced when NIF is loaded
  def new_runtime(), do: :erlang.nif_error(:nif_not_loaded)
  def new_runtime_with_llm(), do: :erlang.nif_error(:nif_not_loaded)
  def new_runtime_with_sql(), do: :erlang.nif_error(:nif_not_loaded)
  def new_runtime_with_all(), do: :erlang.nif_error(:nif_not_loaded)
  def eval(_runtime, _source), do: :erlang.nif_error(:nif_not_loaded)
  def eval_file(_runtime, _path), do: :erlang.nif_error(:nif_not_loaded)
  def eval_with_base_path(_runtime, _source, _base_path), do: :erlang.nif_error(:nif_not_loaded)
  def eval_with_bindings(_runtime, _source, _bindings), do: :erlang.nif_error(:nif_not_loaded)
  def call_function(_runtime, _name, _args), do: :erlang.nif_error(:nif_not_loaded)
  def get_types(_runtime), do: :erlang.nif_error(:nif_not_loaded)
  def get_function_signatures(_runtime), do: :erlang.nif_error(:nif_not_loaded)
  def has_function(_runtime, _name), do: :erlang.nif_error(:nif_not_loaded)
  def get_global(_runtime, _name), do: :erlang.nif_error(:nif_not_loaded)
  def set_global(_runtime, _name, _value), do: :erlang.nif_error(:nif_not_loaded)
  def reset(_runtime), do: :erlang.nif_error(:nif_not_loaded)
end
