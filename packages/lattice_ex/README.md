# Lattice

Elixir bindings for the Lattice DSL runtime - a domain-specific language with LLM integration.

## Installation

Add `lattice` to your list of dependencies in `mix.exs`:

```elixir
def deps do
  [
    {:lattice, "~> 0.1.0"}
  ]
end
```

**Note**: This package currently only supports macOS Apple Silicon (aarch64-apple-darwin).

## Usage

```elixir
# Create a runtime
{:ok, rt} = Lattice.new()

# Evaluate expressions
{:ok, 6} = Lattice.eval(rt, "1 + 2 + 3")
{:ok, "hello world"} = Lattice.eval(rt, ~s("hello" + " " + "world"))

# Define and call functions
Lattice.eval(rt, """
  def add(a: Int, b: Int) -> Int {
    a + b
  }
""")
{:ok, 7} = Lattice.call(rt, "add", [3, 4])

# Define types
Lattice.eval(rt, "type Person { name: String, age: Int }")
{:ok, person} = Lattice.eval(rt, ~s(Person { name: "Alice", age: 30 }))
# => %{"name" => "Alice", "age" => 30}

# Use with bindings
{:ok, 30} = Lattice.eval_with_bindings(rt, "x + y", [{"x", 10}, {"y", 20}])
```

## Building from source

If you need to compile the NIF locally (requires Rust toolchain):

```bash
LATTICE_BUILD=true mix deps.compile lattice --force
```

## License

MIT
