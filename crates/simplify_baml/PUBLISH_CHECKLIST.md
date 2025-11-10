# Quick Publishing Checklist

## Before Publishing

- [ ] Update author information in both Cargo.toml files
- [ ] Update repository URLs in both Cargo.toml files  
- [ ] Update @source_url in mix.exs
- [ ] Run tests: `cd simplify_baml && cargo test`
- [ ] Run examples: `cd simplify_baml && cargo run --example with_macros`
- [ ] Ensure LICENSE file exists
- [ ] Update README.md with correct installation instructions

## Publishing to crates.io

```bash
# Terminal 1: Publish macros crate
cd /Users/catethos/workspace/simplify_baml/simplify_baml_macros
cargo publish --dry-run  # Check first
cargo publish            # Actually publish

# Wait 1-2 minutes for crates.io to index

# Terminal 2: Update and publish main crate
cd /Users/catethos/workspace/simplify_baml

# Edit Cargo.toml: Change macros dependency from path to version "0.1.0"
cargo build --release    # Test it works
cargo publish --dry-run  # Check
cargo publish            # Publish
```

## Publishing to Hex.pm

```bash
cd /Users/catethos/workspace/simplify_baml_ex

# Edit native/simplify_baml_nif/Cargo.toml: Change dependency to "0.1.0"

mix clean
mix deps.get
mix compile              # Test NIF builds with crates.io version
mix test                 # Run tests

mix hex.build            # Check package contents
mix hex.publish          # Publish to Hex
```

## Post-Publishing

- [ ] Check crates.io: https://crates.io/crates/simplify_baml
- [ ] Check crates.io: https://crates.io/crates/simplify_baml_macros
- [ ] Check hex.pm: https://hex.pm/packages/simplify_baml
- [ ] Test installation in fresh project
- [ ] Update documentation with installation instructions
- [ ] Tag release in git: `git tag v0.1.0 && git push --tags`

## Installation Commands (for users)

### Rust
```toml
[dependencies]
simplify_baml = "0.1.0"
```

### Elixir
```elixir
def deps do
  [
    {:simplify_baml, "~> 0.1.0"}
  ]
end
```
