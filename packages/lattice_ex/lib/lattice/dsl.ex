defmodule Lattice.DSL do
  @moduledoc """
  Generate Elixir modules from Lattice source files at compile time.

  ## Usage

  Given a Lattice file `lib/math.lat`:

      type Data {
          x: Int,
          y: Int
      }

      def sum(d: Data) -> Int {
          d.x + d.y
      }

      def add(a: Int, b: Int) -> Int {
          a + b
      }

  You can create an Elixir module that wraps these functions:

      defmodule MyMath do
        use Lattice.DSL, file: "lib/math.lat"
      end

  Now you can call the Lattice functions directly:

      MyMath.sum(%{"x" => 10, "y" => 20})
      #=> {:ok, 30}

      MyMath.add(3, 4)
      #=> {:ok, 7}

  ## Options

  - `:file` - Path to the `.lat` file (required). Can be absolute or relative to the project root.
  - `:unwrap` - If `true`, functions will return values directly instead of `{:ok, value}` tuples,
    and raise on errors. Default: `false`.
  - `:sql` - If `true`, enables SQL support (DuckDB) in the runtime. Default: `false`.
  - `:llm` - If `true`, enables LLM support in the runtime. Default: `false`.

  ## With unwrap option

      defmodule MyMath do
        use Lattice.DSL, file: "lib/math.lat", unwrap: true
      end

      MyMath.add(3, 4)
      #=> 7
  """

  defmacro __using__(opts) do
    file = Keyword.fetch!(opts, :file)
    unwrap = Keyword.get(opts, :unwrap, false)
    sql = Keyword.get(opts, :sql, false)
    llm = Keyword.get(opts, :llm, false)

    quote do
      @lattice_file unquote(file)
      @lattice_unwrap unquote(unwrap)
      @lattice_sql unquote(sql)
      @lattice_llm unquote(llm)
      @before_compile Lattice.DSL
    end
  end

  defmacro __before_compile__(env) do
    file = Module.get_attribute(env.module, :lattice_file)
    unwrap = Module.get_attribute(env.module, :lattice_unwrap)
    sql = Module.get_attribute(env.module, :lattice_sql)
    llm = Module.get_attribute(env.module, :lattice_llm)

    # Resolve the file path relative to the project
    resolved_file = resolve_file_path(file, env)

    # Load at compile time to get signatures
    {:ok, rt} = create_runtime(sql, llm)

    case Lattice.Native.eval_file(rt, resolved_file) do
      {:ok, _} -> :ok
      {:error, reason} -> raise CompileError, description: "Failed to load Lattice file: #{reason}"
    end

    signatures = Lattice.Native.get_function_signatures(rt)

    functions =
      for sig <- signatures do
        name = String.to_atom(sig[:name])
        arity = length(sig[:params])
        args = Macro.generate_arguments(arity, __MODULE__)
        name_str = sig[:name]

        if unwrap do
          quote do
            def unquote(name)(unquote_splicing(args)) do
              case Lattice.Native.call_function(
                     __lattice_runtime__(),
                     unquote(name_str),
                     [unquote_splicing(args)]
                   ) do
                {:ok, result} -> result
                {:error, reason} -> raise RuntimeError, message: "Lattice error: #{reason}"
              end
            end
          end
        else
          quote do
            def unquote(name)(unquote_splicing(args)) do
              Lattice.Native.call_function(
                __lattice_runtime__(),
                unquote(name_str),
                [unquote_splicing(args)]
              )
            end
          end
        end
      end

    # Generate function docs from signatures
    function_names = Enum.map(signatures, & &1[:name])

    quote do
      @moduledoc """
      Auto-generated module from Lattice file: `#{unquote(file)}`

      ## Available Functions

      #{unquote(Enum.map_join(function_names, "\n", fn name -> "- `#{name}/...`" end))}
      """

      @lattice_runtime_pid nil

      def __lattice_file__, do: unquote(resolved_file)

      def __lattice_runtime__ do
        case Process.get(:lattice_dsl_runtime) do
          nil ->
            {:ok, rt} = Lattice.DSL.create_runtime(unquote(sql), unquote(llm))
            {:ok, _} = Lattice.Native.eval_file(rt, unquote(resolved_file))
            Process.put(:lattice_dsl_runtime, rt)
            rt

          rt ->
            rt
        end
      end

      @doc """
      Reset the Lattice runtime, reloading the source file.
      """
      def reset do
        Process.delete(:lattice_dsl_runtime)
        __lattice_runtime__()
        :ok
      end

      @doc """
      Get all function signatures defined in this module.
      """
      def __lattice_signatures__ do
        Lattice.Native.get_function_signatures(__lattice_runtime__())
      end

      @doc """
      Get all type definitions from this module.
      """
      def __lattice_types__ do
        Lattice.Native.get_types(__lattice_runtime__())
      end

      unquote_splicing(functions)
    end
  end

  defp resolve_file_path(file, _env) do
    cond do
      # Absolute path
      String.starts_with?(file, "/") ->
        file

      # Relative path - resolve from project root
      true ->
        project_root = File.cwd!()
        Path.join(project_root, file)
    end
  end

  @doc false
  def create_runtime(sql, llm) do
    case {sql, llm} do
      {true, true} -> Lattice.Native.new_runtime_with_all()
      {true, false} -> Lattice.Native.new_runtime_with_sql()
      {false, true} -> Lattice.Native.new_runtime_with_llm()
      {false, false} -> Lattice.Native.new_runtime()
    end
  end
end
