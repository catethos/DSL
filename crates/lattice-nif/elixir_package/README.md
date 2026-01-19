# Lattice

Elixir bindings for the [Lattice](https://github.com/catethos/lattice) DSL runtime.

## Installation

Add `lattice` to your list of dependencies in `mix.exs`:

```elixir
def deps do
  [
    {:lattice, "~> 0.1.8"}
  ]
end
```

## Supported Platforms

This package includes precompiled NIFs for:
- macOS (Apple Silicon / ARM64)
- Linux (x86_64)

## Usage

```elixir
# Create a runtime
{:ok, runtime} = Lattice.new()

# Evaluate Lattice code
{:ok, result} = Lattice.eval(runtime, "1 + 2")

# Create a runtime with SQL support
{:ok, runtime} = Lattice.new_with_sql()
```

## License

MIT
