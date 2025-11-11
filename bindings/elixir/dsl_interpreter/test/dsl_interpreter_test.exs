defmodule DslInterpreterTest do
  use ExUnit.Case
  doctest DslInterpreter

  test "greets the world" do
    assert DslInterpreter.hello() == :world
  end
end
