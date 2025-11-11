defmodule DSL.Interpreter do
  @moduledoc """
  Elixir wrapper for the DSL IR interpreter.

  This module provides a high-level interface to execute DSL intermediate representation (IR)
  using a Rust-based interpreter via Rustler NIFs.

  ## Basic Usage

      # Create a simple IR node
      node = %{"String" => "Hello, World!"}
      {:ok, interpreter} = DSL.Interpreter.new()
      {:ok, result} = DSL.Interpreter.eval(interpreter, node)

  ## With Tracing

      {:ok, tracing_interpreter} = DSL.Interpreter.new_tracing()
      {:ok, result, trace} = DSL.Interpreter.eval_with_trace(tracing_interpreter, node)

  ## Error Handling

  All functions return either `{:ok, result}` or `{:error, error_map}` tuples.
  Error maps contain:
  - `:kind` - The error type (`:llm_error`, `:http_error`, `:type_error`, etc.)
  - `:message` - Human-readable error message
  - Additional context fields depending on error type
  """

  alias DSL.Interpreter.Native

  @type interpreter :: reference()
  @type tracing_interpreter :: reference()
  @type ir_node :: map() | binary()
  @type ir :: map() | binary()
  @type value :: map()
  @type trace :: map()
  @type error_map :: map()

  ## Basic Interpreter API

  @doc """
  Creates a new interpreter instance.

  ## Examples

      {:ok, interpreter} = DSL.Interpreter.new()
  """
  @spec new() :: {:ok, interpreter()} | {:error, error_map()}
  def new do
    Native.new_interpreter()
  end

  @doc """
  Creates an interpreter from IR.

  The IR can be provided as either a map or a JSON-encoded binary.

  ## Examples

      ir = %{
        "version" => "1.0",
        "types" => [],
        "enums" => [],
        "functions" => [],
        "function_groups" => [],
        "agents" => [],
        "entry_expr" => %{"String" => "Hello"}
      }
      {:ok, interpreter} = DSL.Interpreter.from_ir(ir)
  """
  @spec from_ir(ir()) :: {:ok, interpreter()} | {:error, error_map()}
  def from_ir(ir) do
    ir_json = encode_ir(ir)
    Native.interpreter_from_ir(ir_json)
  end

  @doc """
  Evaluates an IR node using the given interpreter.

  The node can be provided as either a map or a JSON-encoded binary.

  ## Examples

      {:ok, interpreter} = DSL.Interpreter.new()
      node = %{"Int" => 42}
      {:ok, result} = DSL.Interpreter.eval(interpreter, node)
  """
  @spec eval(interpreter(), ir_node()) :: {:ok, value()} | {:error, error_map()}
  def eval(interpreter, node) do
    node_json = encode_node(node)

    case Native.eval(interpreter, node_json) do
      {:ok, value_json} -> {:ok, decode_value(value_json)}
      {:error, error_map} -> {:error, error_map}
    end
  end

  ## Tracing Interpreter API

  @doc """
  Creates a new tracing interpreter.

  ## Options

  - `:max_events` - Maximum number of trace events to collect (default: unlimited)
  - `:capture_variables` - Whether to capture variable states (default: false)
  - `:node_filter` - List of node types to trace (default: all)
  - `:min_duration_micros` - Minimum duration in microseconds to record (default: 0)
  - `:recursive` - Whether to capture nested traces (default: true)

  ## Examples

      {:ok, interpreter} = DSL.Interpreter.new_tracing()

      {:ok, interpreter} = DSL.Interpreter.new_tracing(
        max_events: 1000,
        capture_variables: true
      )
  """
  @spec new_tracing(keyword()) :: {:ok, tracing_interpreter()} | {:error, error_map()}
  def new_tracing(opts \\ []) do
    config_json = encode_trace_config(opts)
    Native.new_tracing_interpreter(config_json)
  end

  @doc """
  Creates a tracing interpreter from IR.

  ## Examples

      ir = %{
        "version" => "1.0",
        "entry_expr" => %{"String" => "Hello"}
      }
      {:ok, interpreter} = DSL.Interpreter.from_ir_tracing(ir)
  """
  @spec from_ir_tracing(ir(), keyword()) :: {:ok, tracing_interpreter()} | {:error, error_map()}
  def from_ir_tracing(ir, opts \\ []) do
    ir_json = encode_ir(ir)
    config_json = encode_trace_config(opts)
    Native.tracing_interpreter_from_ir(ir_json, config_json)
  end

  @doc """
  Evaluates an IR node with tracing enabled.

  Returns the result value and the execution trace.

  ## Examples

      {:ok, interpreter} = DSL.Interpreter.new_tracing()
      node = %{"Int" => 42}
      {:ok, result, trace} = DSL.Interpreter.eval_with_trace(interpreter, node)
  """
  @spec eval_with_trace(tracing_interpreter(), ir_node()) ::
          {:ok, value(), trace()} | {:error, error_map()}
  def eval_with_trace(interpreter, node) do
    node_json = encode_node(node)

    case Native.eval_with_trace(interpreter, node_json) do
      {:ok, value_json, trace_json} ->
        {:ok, decode_value(value_json), decode_trace(trace_json)}

      {:error, error_map} ->
        {:error, error_map}
    end
  end

  @doc """
  Clears the trace from a tracing interpreter.

  ## Examples

      :ok = DSL.Interpreter.clear_trace(interpreter)
  """
  @spec clear_trace(tracing_interpreter()) :: :ok | {:error, error_map()}
  def clear_trace(interpreter) do
    Native.clear_trace(interpreter)
  end

  @doc """
  Gets the current trace from a tracing interpreter.

  ## Examples

      {:ok, trace} = DSL.Interpreter.get_trace(interpreter)
  """
  @spec get_trace(tracing_interpreter()) :: {:ok, trace()} | {:error, error_map()}
  def get_trace(interpreter) do
    case Native.get_trace(interpreter) do
      {:ok, trace_json} -> {:ok, decode_trace(trace_json)}
      {:error, error_map} -> {:error, error_map}
    end
  end

  ## Convenience API

  @doc """
  One-shot execution of an IR node or complete IR program.

  This is a convenience function that creates an interpreter, evaluates the node/IR,
  and returns the result. Optionally enables tracing.

  ## Options

  - `:trace` - Enable tracing (default: false)
  - Trace config options (when `:trace` is true) - see `new_tracing/1`

  ## Examples

      # Simple evaluation
      node = %{"Int" => 42}
      {:ok, result} = DSL.Interpreter.run(node)

      # With tracing
      {:ok, result, trace} = DSL.Interpreter.run(node, trace: true)

      # Full IR execution
      ir = %{"version" => "1.0", "entry_expr" => node}
      {:ok, result} = DSL.Interpreter.run(ir)
  """
  @spec run(ir() | ir_node(), keyword()) ::
          {:ok, value()} | {:ok, value(), trace()} | {:error, error_map()}
  def run(ir_or_node, opts \\ []) do
    trace_enabled = Keyword.get(opts, :trace, false)

    cond do
      trace_enabled ->
        with {:ok, interpreter} <- new_tracing(opts),
             {:ok, result, trace} <- eval_with_trace(interpreter, ir_or_node) do
          {:ok, result, trace}
        end

      is_complete_ir?(ir_or_node) ->
        with {:ok, interpreter} <- from_ir(ir_or_node),
             entry_expr = get_entry_expr(ir_or_node),
             {:ok, result} <- eval(interpreter, entry_expr) do
          {:ok, result}
        end

      true ->
        with {:ok, interpreter} <- new(),
             {:ok, result} <- eval(interpreter, ir_or_node) do
          {:ok, result}
        end
    end
  end

  ## Private Helpers

  defp encode_ir(ir) when is_binary(ir), do: ir

  defp encode_ir(ir) when is_map(ir) do
    Jason.encode!(ir)
  end

  defp encode_node(node) when is_binary(node), do: node

  defp encode_node(node) when is_map(node) do
    Jason.encode!(node)
  end

  defp encode_trace_config([]), do: ""

  defp encode_trace_config(opts) do
    config =
      opts
      |> Enum.into(%{})
      |> Map.take([:max_events, :capture_variables, :node_filter, :min_duration_micros, :recursive])

    Jason.encode!(config)
  end

  defp decode_value(value_json) when is_binary(value_json) do
    Jason.decode!(value_json)
  end

  defp decode_trace(trace_json) when is_binary(trace_json) do
    Jason.decode!(trace_json)
  end

  defp is_complete_ir?(data) when is_map(data) do
    Map.has_key?(data, "version") or Map.has_key?(data, :version)
  end

  defp is_complete_ir?(_), do: false

  defp get_entry_expr(ir) when is_map(ir) do
    ir["entry_expr"] || ir[:entry_expr] || %{"String" => ""}
  end
end
