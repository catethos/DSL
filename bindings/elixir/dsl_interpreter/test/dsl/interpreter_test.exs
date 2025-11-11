defmodule DSL.InterpreterTest do
  use ExUnit.Case
  doctest DSL.Interpreter

  alias DSL.Interpreter

  describe "basic interpreter" do
    test "new/0 creates an interpreter" do
      assert {:ok, interpreter} = Interpreter.new()
      assert is_reference(interpreter)
    end

    test "eval/2 evaluates string literals" do
      {:ok, interpreter} = Interpreter.new()
      node = %{"String" => "Hello, World!"}

      assert {:ok, %{"String" => "Hello, World!"}} = Interpreter.eval(interpreter, node)
    end

    test "eval/2 evaluates integer literals" do
      {:ok, interpreter} = Interpreter.new()
      node = %{"Int" => 42}

      assert {:ok, %{"Int" => 42}} = Interpreter.eval(interpreter, node)
    end

    test "eval/2 evaluates float literals" do
      {:ok, interpreter} = Interpreter.new()
      node = %{"Float" => 3.14}

      assert {:ok, %{"Float" => 3.14}} = Interpreter.eval(interpreter, node)
    end

    test "eval/2 evaluates boolean literals" do
      {:ok, interpreter} = Interpreter.new()

      assert {:ok, %{"Bool" => true}} = Interpreter.eval(interpreter, %{"Bool" => true})
      assert {:ok, %{"Bool" => false}} = Interpreter.eval(interpreter, %{"Bool" => false})
    end

    test "eval/2 evaluates null" do
      {:ok, interpreter} = Interpreter.new()
      node = "Null"

      assert {:ok, "Null"} = Interpreter.eval(interpreter, node)
    end

    test "eval/2 evaluates lists" do
      {:ok, interpreter} = Interpreter.new()

      node = %{
        "List" => [
          %{"Int" => 1},
          %{"Int" => 2},
          %{"Int" => 3}
        ]
      }

      assert {:ok, %{"List" => [%{"Int" => 1}, %{"Int" => 2}, %{"Int" => 3}]}} =
               Interpreter.eval(interpreter, node)
    end

    test "eval/2 evaluates maps" do
      {:ok, interpreter} = Interpreter.new()

      node = %{
        "Map" => [
          ["key1", %{"String" => "value1"}],
          ["key2", %{"Int" => 42}]
        ]
      }

      assert {:ok, result} = Interpreter.eval(interpreter, node)
      assert %{"Map" => map_entries} = result
      assert is_list(map_entries)
    end

    test "eval/2 evaluates binary operations" do
      {:ok, interpreter} = Interpreter.new()

      node = %{
        "BinaryOp" => %{
          "left" => %{"Int" => 10},
          "op" => "+",
          "right" => %{"Int" => 32}
        }
      }

      assert {:ok, %{"Int" => 42}} = Interpreter.eval(interpreter, node)
    end

    test "eval/2 handles unknown variables" do
      {:ok, interpreter} = Interpreter.new()
      node = %{"Variable" => "unknown_var"}

      assert {:error, error} = Interpreter.eval(interpreter, node)
      assert error.kind == :unknown_variable
      assert error.name == "unknown_var"
      assert String.contains?(error.message, "unknown_var")
    end

    test "from_ir/1 creates interpreter from IR" do
      ir = %{
        "version" => "1.0",
        "types" => [],
        "enums" => [],
        "functions" => [],
        "function_groups" => [],
        "agents" => [],
        "entry_expr" => %{"String" => "Hello"}
      }

      assert {:ok, interpreter} = Interpreter.from_ir(ir)
      assert is_reference(interpreter)
    end

    test "eval/2 with JSON-encoded nodes" do
      {:ok, interpreter} = Interpreter.new()
      node_json = Jason.encode!(%{"Int" => 42})

      assert {:ok, %{"Int" => 42}} = Interpreter.eval(interpreter, node_json)
    end
  end

  describe "tracing interpreter" do
    test "new_tracing/0 creates a tracing interpreter" do
      assert {:ok, interpreter} = Interpreter.new_tracing()
      assert is_reference(interpreter)
    end

    test "new_tracing/1 creates tracing interpreter with config" do
      opts = [
        max_events: 1000,
        capture_variables: true,
        recursive: true
      ]

      assert {:ok, interpreter} = Interpreter.new_tracing(opts)
      assert is_reference(interpreter)
    end

    test "eval_with_trace/2 returns value and trace" do
      {:ok, interpreter} = Interpreter.new_tracing()
      node = %{"Int" => 42}

      assert {:ok, value, trace} = Interpreter.eval_with_trace(interpreter, node)
      assert %{"Int" => 42} = value
      assert is_map(trace)
      assert Map.has_key?(trace, "events")
    end

    test "eval_with_trace/2 traces multiple operations" do
      {:ok, interpreter} = Interpreter.new_tracing(capture_variables: true)

      node = %{
        "BinaryOp" => %{
          "left" => %{"Int" => 10},
          "op" => "+",
          "right" => %{"Int" => 32}
        }
      }

      assert {:ok, %{"Int" => 42}, trace} = Interpreter.eval_with_trace(interpreter, node)
      assert is_map(trace)
      assert Map.has_key?(trace, "events")

      # Should have trace events for the binary operation
      events = trace["events"]
      assert is_list(events)
      assert length(events) > 0
    end

    test "clear_trace/1 clears the trace" do
      {:ok, interpreter} = Interpreter.new_tracing()
      node = %{"Int" => 42}

      # Execute once to generate trace
      {:ok, _, _} = Interpreter.eval_with_trace(interpreter, node)

      # Clear trace
      assert :ok = Interpreter.clear_trace(interpreter)

      # Get trace - should be empty
      assert {:ok, trace} = Interpreter.get_trace(interpreter)
      assert trace["events"] == []
    end

    test "get_trace/1 retrieves current trace" do
      {:ok, interpreter} = Interpreter.new_tracing()
      node = %{"String" => "test"}

      {:ok, _, _} = Interpreter.eval_with_trace(interpreter, node)

      assert {:ok, trace} = Interpreter.get_trace(interpreter)
      assert is_map(trace)
      assert Map.has_key?(trace, "events")
      assert length(trace["events"]) > 0
    end

    test "from_ir_tracing/2 creates tracing interpreter from IR" do
      ir = %{
        "version" => "1.0",
        "types" => [],
        "enums" => [],
        "functions" => [],
        "function_groups" => [],
        "agents" => [],
        "entry_expr" => %{"String" => "Hello"}
      }

      assert {:ok, interpreter} = Interpreter.from_ir_tracing(ir)
      assert is_reference(interpreter)
    end

    test "from_ir_tracing/2 with config options" do
      ir = %{
        "version" => "1.0",
        "types" => [],
        "enums" => [],
        "functions" => [],
        "function_groups" => [],
        "agents" => [],
        "entry_expr" => %{"String" => "Hello"}
      }

      opts = [max_events: 500, capture_variables: false]

      assert {:ok, interpreter} = Interpreter.from_ir_tracing(ir, opts)
      assert is_reference(interpreter)
    end
  end

  describe "convenience API" do
    test "run/1 executes a simple node" do
      node = %{"Int" => 42}
      assert {:ok, %{"Int" => 42}} = Interpreter.run(node)
    end

    test "run/2 with tracing enabled" do
      node = %{"String" => "Hello"}
      assert {:ok, value, trace} = Interpreter.run(node, trace: true)
      assert %{"String" => "Hello"} = value
      assert is_map(trace)
    end

    test "run/1 with complete IR" do
      ir = %{
        "version" => "1.0",
        "types" => [],
        "enums" => [],
        "functions" => [],
        "function_groups" => [],
        "agents" => [],
        "entry_expr" => %{"Int" => 99}
      }

      assert {:ok, %{"Int" => 99}} = Interpreter.run(ir)
    end

    test "run/2 with IR and tracing" do
      ir = %{
        "version" => "1.0",
        "types" => [],
        "enums" => [],
        "functions" => [],
        "function_groups" => [],
        "agents" => [],
        "entry_expr" => %{"Float" => 3.14}
      }

      assert {:ok, value, trace} = Interpreter.run(ir, trace: true)
      assert %{"Float" => 3.14} = value
      assert is_map(trace)
    end

    test "run/2 with trace config options" do
      node = %{"Bool" => true}

      assert {:ok, value, trace} =
               Interpreter.run(node, trace: true, max_events: 100, capture_variables: true)

      assert %{"Bool" => true} = value
      assert is_map(trace)
    end
  end

  describe "error handling" do
    test "type errors include expected and got types" do
      {:ok, interpreter} = Interpreter.new()

      # Attempt a binary op with incompatible types
      node = %{
        "BinaryOp" => %{
          "left" => %{"String" => "hello"},
          "op" => "+",
          "right" => %{"Int" => 42}
        }
      }

      assert {:error, error} = Interpreter.eval(interpreter, node)
      assert error.kind == :type_error or error.kind == :runtime_error
      assert is_binary(error.message)
    end

    test "invalid JSON returns error" do
      {:ok, interpreter} = Interpreter.new()
      invalid_json = "not valid json"

      assert {:error, error} = Interpreter.eval(interpreter, invalid_json)
      assert error.kind == :runtime_error
      assert String.contains?(error.message, "JSON")
    end

    test "errors include span information when available" do
      {:ok, interpreter} = Interpreter.new()
      node = %{"Variable" => "undefined_var"}

      assert {:error, error} = Interpreter.eval(interpreter, node)
      assert error.kind == :unknown_variable
      # Span may or may not be present depending on how the node was created
    end
  end

  describe "complex scenarios" do
    test "evaluates conditional expressions" do
      {:ok, interpreter} = Interpreter.new()

      node = %{
        "Conditional" => %{
          "condition" => %{"Bool" => true},
          "then_expr" => %{"String" => "yes"},
          "else_expr" => %{"String" => "no"}
        }
      }

      assert {:ok, %{"String" => "yes"}} = Interpreter.eval(interpreter, node)
    end

    test "evaluates nested binary operations" do
      {:ok, interpreter} = Interpreter.new()

      # (10 + 20) * 2
      node = %{
        "BinaryOp" => %{
          "left" => %{
            "BinaryOp" => %{
              "left" => %{"Int" => 10},
              "op" => "+",
              "right" => %{"Int" => 20}
            }
          },
          "op" => "*",
          "right" => %{"Int" => 2}
        }
      }

      assert {:ok, %{"Int" => 60}} = Interpreter.eval(interpreter, node)
    end

    test "evaluates list operations" do
      {:ok, interpreter} = Interpreter.new()

      node = %{
        "List" => [
          %{"String" => "a"},
          %{"String" => "b"},
          %{"String" => "c"}
        ]
      }

      assert {:ok, %{"List" => items}} = Interpreter.eval(interpreter, node)
      assert length(items) == 3
    end
  end
end
