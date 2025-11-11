defmodule DSL.Interpreter.Native do
  @moduledoc false
  # Internal NIF module - do not use directly

  version = Mix.Project.config()[:version]

  use RustlerPrecompiled,
    otp_app: :dsl_interpreter,
    crate: "dsl_nif",
    base_url: "https://github.com/yourorg/dsl_interpreter/releases/download/v#{version}",
    force_build: System.get_env("RUSTLER_PRECOMPILATION_DSL_NIF_BUILD") in ["1", "true"],
    version: version

  # Basic interpreter NIFs
  def new_interpreter, do: :erlang.nif_error(:nif_not_loaded)
  def interpreter_from_ir(_ir_json), do: :erlang.nif_error(:nif_not_loaded)
  def eval(_interpreter, _node_json), do: :erlang.nif_error(:nif_not_loaded)

  # Tracing interpreter NIFs
  def new_tracing_interpreter(_config_json), do: :erlang.nif_error(:nif_not_loaded)
  def tracing_interpreter_from_ir(_ir_json, _config_json), do: :erlang.nif_error(:nif_not_loaded)
  def eval_with_trace(_interpreter, _node_json), do: :erlang.nif_error(:nif_not_loaded)
  def clear_trace(_interpreter), do: :erlang.nif_error(:nif_not_loaded)
  def get_trace(_interpreter), do: :erlang.nif_error(:nif_not_loaded)
end
