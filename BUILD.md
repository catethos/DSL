# Build Instructions

This document provides detailed instructions for building and compiling the DSL TUI application.

## Architecture Overview

The project consists of three crates:

1. **dsl-core** - Core language engine (library)
2. **dsl-tui** - Terminal user interface (library)
3. **dsl-repl** - Main application binary (thin wrapper)

## Prerequisites

- **Rust toolchain** (1.70+)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- **Optional: OpenAI API Key** for LLM features
  ```bash
  export OPENAI_API_KEY="sk-..."
  ```

## Quick Start

### Compile and Run

```bash
# Navigate to project directory
cd DSL

# Build and run in one command (development mode)
cargo run --bin dsl

# Build and run in release mode (optimized)
cargo run --release --bin dsl
```

## Detailed Build Options

### Build Everything

```bash
# Build all crates in the workspace
cargo build --workspace

# Build in release mode (optimized)
cargo build --workspace --release
```

### Build Individual Crates

```bash
# Build only the core engine
cargo build -p dsl-core

# Build only the TUI library
cargo build -p dsl-tui

# Build only the main application
cargo build -p dsl-repl
```

### Build Targets

The compiled binary will be located at:
- **Debug:** `target/debug/dsl`
- **Release:** `target/release/dsl`

Run directly:
```bash
# Debug build
./target/debug/dsl

# Release build
./target/release/dsl
```

## Development Workflow

### Check Compilation

```bash
# Fast check without building binaries
cargo check --workspace

# Check with all features
cargo check --workspace --all-features
```

### Run Tests

```bash
# Run all tests
cargo test --workspace

# Run tests for specific crate
cargo test -p dsl-core
```

### Code Quality

```bash
# Format code
cargo fmt --all

# Run linter
cargo clippy --workspace

# Apply lint suggestions
cargo clippy --workspace --fix
```

## Incremental Builds

For faster development iteration:

```bash
# Only rebuild changed crates
cargo build -p dsl-core  # If you changed core logic
cargo build -p dsl-tui   # If you changed UI
cargo build -p dsl-repl  # Usually very fast (5 lines of code!)
```

## Build Profiles

### Debug Mode (default)

```bash
cargo build
# - Fast compilation
# - Slower runtime
# - Includes debug symbols
# - Best for development
```

### Release Mode

```bash
cargo build --release
# - Slower compilation
# - Fast runtime (optimized)
# - No debug symbols
# - Best for production/benchmarking
```

### Custom Profile

Edit `Cargo.toml` to add custom profiles:

```toml
[profile.dev-optimized]
inherits = "dev"
opt-level = 1
```

Build with:
```bash
cargo build --profile dev-optimized
```

## Troubleshooting

### Clean Build

If you encounter build errors:

```bash
# Remove all build artifacts
cargo clean

# Rebuild from scratch
cargo build --workspace
```

### Update Dependencies

```bash
# Update to latest compatible versions
cargo update

# Rebuild
cargo build --workspace
```

### Common Issues

**Issue:** `error: could not find 'dsl-core'`
- **Solution:** Make sure you're in the workspace root directory

**Issue:** Grammar file not found
- **Solution:** The `grammar.pest` path in `dsl-core/src/parser/mod.rs` should be `parser/grammar.pest`

**Issue:** Tree-sitter build fails
- **Solution:** Run `cd tree-sitter-dsl && npm install` first

## Using dsl-core as a Library

To use `dsl-core` in your own project:

### 1. Add to Cargo.toml

```toml
[dependencies]
dsl-core = { path = "../DSL/crates/dsl-core" }
tokio = { version = "1.0", features = ["full"] }
```

### 2. Example Usage

```rust
use dsl_core::{Evaluator, Value};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut eval = Evaluator::new();
    
    // Execute DSL code
    let (value, _) = eval.eval("42 + 58").await?;
    println!("Result: {}", value.display());
    
    Ok(())
}
```

### 3. Compile Your Project

```bash
cargo build
cargo run
```

## Cross-Platform Notes

### macOS
- No special requirements
- Built and tested on macOS

### Linux
- Install build essentials: `sudo apt-get install build-essential`
- May need: `libssl-dev pkg-config`

### Windows
- Use MSVC toolchain (recommended)
- Or use GNU toolchain with MinGW

## Performance Tips

1. **Use release builds** for actual usage:
   ```bash
   cargo build --release
   ```

2. **Enable link-time optimization** in `Cargo.toml`:
   ```toml
   [profile.release]
   lto = true
   ```

3. **Parallel compilation** (set in `~/.cargo/config.toml`):
   ```toml
   [build]
   jobs = 8  # adjust to your CPU
   ```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Build

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo build --workspace --release
      - run: cargo test --workspace
```

## Binary Size Optimization

To reduce binary size:

```toml
[profile.release]
opt-level = 'z'     # Optimize for size
lto = true          # Enable Link Time Optimization
codegen-units = 1   # Better optimization
strip = true        # Remove debug symbols
```

Build:
```bash
cargo build --release
```

Check size:
```bash
ls -lh target/release/dsl
```

---

For more information, see the main [README.md](README.md)
