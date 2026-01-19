defmodule Lattice.Native do
  @moduledoc """
  NIF bindings for the Lattice runtime.

  This module provides low-level access to the Lattice runtime through
  Rustler NIFs. For most use cases, use the higher-level `Lattice` module.
  """

  @version Mix.Project.config()[:version]
  @nif_version "2.17"

  # Use local binaries if available, otherwise download from GitHub
  @local_mode System.get_env("LATTICE_LOCAL") == "1" or
                File.exists?(Path.join([__DIR__, "../../priv/native"]))

  if @local_mode and File.dir?(Path.join([__DIR__, "../../priv/native"])) do
    # Local mode: load from priv/native directly
    @on_load :load_nif

    def load_nif do
      nif_file = nif_filename()
      nif_path = :filename.join(:code.priv_dir(:lattice), ~c"native/#{nif_file}")

      case :erlang.load_nif(nif_path, 0) do
        :ok -> :ok
        {:error, {:reload, _}} -> :ok
        {:error, reason} -> {:error, reason}
      end
    end

    defp nif_filename do
      target = current_target()
      "liblattice_nif-v#{@version}-nif-#{@nif_version}-#{target}"
    end

    defp current_target do
      case {:os.type(), :erlang.system_info(:system_architecture)} do
        {{:unix, :darwin}, arch} when is_list(arch) ->
          case List.to_string(arch) do
            "aarch64" <> _ -> "aarch64-apple-darwin"
            "arm64" <> _ -> "aarch64-apple-darwin"
            "x86_64" <> _ -> "x86_64-apple-darwin"
            _ -> "aarch64-apple-darwin"
          end

        {{:unix, :darwin}, _} ->
          "aarch64-apple-darwin"

        {{:unix, _}, arch} when is_list(arch) ->
          case List.to_string(arch) do
            "aarch64" <> _ -> "aarch64-unknown-linux-gnu"
            "x86_64" <> _ -> "x86_64-unknown-linux-gnu"
            _ -> "x86_64-unknown-linux-gnu"
          end

        {{:unix, _}, _} ->
          "x86_64-unknown-linux-gnu"

        _ ->
          raise "Unsupported platform"
      end
    end
  else
    # Production mode: use rustler_precompiled to download from GitHub
    use RustlerPrecompiled,
      otp_app: :lattice,
      crate: "lattice_nif",
      base_url: "https://github.com/catethos/lattice/releases/download/v#{@version}",
      version: @version,
      force_build: System.get_env("LATTICE_BUILD") in ["1", "true"],
      nif_versions: ["2.17"],
      targets: ~w(
        aarch64-apple-darwin
        x86_64-unknown-linux-gnu
      )
  end

  # NIF function stubs - these are replaced when the NIF is loaded
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
