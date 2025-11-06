# DSL Compiler Guide

Complete guide to using the `dsl-compiler` tool to compile DSL programs into native executables.

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Commands](#commands)
- [Compilation Process](#compilation-process)
- [Examples](#examples)
- [Command-Line Arguments](#command-line-arguments)
- [Debugging](#debugging)
- [Performance](#performance)
- [Troubleshooting](#troubleshooting)

## Installation

### Build from Source

```bash
# Clone the repository
cd DSL

# Build the compiler
cargo build --release -p dsl-compiler

# The binary will be at:
# ./target/release/dsl-compiler
```

### Add to PATH (Optional)

```bash
# macOS/Linux
export PATH="$PATH:/path/to/DSL/target/release"

# Or copy to system location
sudo cp target/release/dsl-compiler /usr/local/bin/
```

## Quick Start

**1. Create a DSL program:**

```javascript
// hello.dsl
upper("Hello, World!")
```

**2. Compile to binary:**

```bash
dsl-compiler build hello.dsl -o hello
```

**3. Run the compiled program:**

```bash
./hello
# Output: "HELLO, WORLD!"
```

That's it! 🎉

## Commands

The `dsl-compiler` provides three main commands:

### `check` - Validate DSL Programs

Validates DSL syntax and semantics without compilation.

```bash
dsl-compiler check <INPUT>
```

**Example:**

```bash
dsl-compiler check examples/greet.dsl
```

**Output:**
- ✅ Success: "✓ No errors found"
- ❌ Error: Detailed error message with line/column

**Use Cases:**
- CI/CD validation
- Pre-commit hooks
- Quick syntax checking
- Editor integration

### `ir` - Compile to IR

Compiles DSL to Intermediate Representation (IR) format.

```bash
dsl-compiler ir <INPUT> -o <OUTPUT> [--json]
```

**Options:**
- `-o, --output <FILE>` - Output IR file (required)
- `--json` - Output JSON instead of MessagePack

**Example:**

```bash
# Compile to JSON IR
dsl-compiler ir program.dsl -o program.ir.json --json

# Inspect the IR
cat program.ir.json
```

**Use Cases:**
- Debugging compilation
- Understanding IR structure
- Serialization/deserialization
- IR analysis tools

### `build` - Compile to Binary

Compiles DSL to a native executable.

```bash
dsl-compiler build <INPUT> -o <OUTPUT> [OPTIONS]
```

**Options:**
- `-o, --output <FILE>` - Output binary path (required)
- `--emit-ir <FILE>` - Also save IR to file
- `--emit-rust <FILE>` - Also save generated Rust code
- `--release` - Build with release optimizations
- `--lib` - Generate a library instead of executable

**Example:**

```bash
# Basic compilation
dsl-compiler build program.dsl -o program

# With debug artifacts
dsl-compiler build program.dsl -o program \
  --emit-ir program.ir.json \
  --emit-rust program.rs

# Release build (slower compile, faster runtime)
dsl-compiler build program.dsl -o program --release
```

**Use Cases:**
- Production deployment
- Distributing DSL programs
- Performance-critical applications
- Standalone tools

## Compilation Process

### How It Works

The compiler uses an **embedded interpreter approach**:

```
┌─────────────┐
│ DSL Source  │  program.dsl
└──────┬──────┘
       │ parse
       ▼
┌─────────────┐
│     AST     │  Abstract Syntax Tree
└──────┬──────┘
       │ compile
       ▼
┌─────────────┐
│     IR      │  Intermediate Representation (serializable)
└──────┬──────┘
       │ codegen
       ▼
┌─────────────┐
│ Rust Code   │  Embeds IR + uses interpreter
│             │  use dsl_interpreter::Interpreter;
│             │  let ir_json = r#"..."#;
│             │  interpreter.eval(&ir)
└──────┬──────┘
       │ cargo build
       ▼
┌─────────────┐
│   Binary    │  Native executable
└─────────────┘
```

### Why Embedded Interpreter?

✅ **Simplicity** - No complex code generation
✅ **Correctness** - Identical to REPL behavior
✅ **Completeness** - All DSL features work automatically
✅ **Maintainability** - Single interpreter codebase
✅ **Reliability** - No translation bugs possible

### What Gets Embedded

The generated Rust code contains:

1. **Serialized IR** - Your DSL program as JSON
2. **Interpreter** - Runtime evaluation engine
3. **Runtime Library** - Builtin functions (LLM, SQL, HTTP)
4. **Dependencies** - tokio, duckdb, simplify_baml, etc.

## Examples

### Example 1: Simple Function

**Input:** `greet.dsl`
```javascript
arg1 |> upper("Hello, ${_}!")
```

**Compile:**
```bash
dsl-compiler build greet.dsl -o greet
```

**Run:**
```bash
./greet "World"
# Output: "HELLO, WORLD!"

./greet "Alice"
# Output: "HELLO, ALICE!"
```

### Example 2: Text Processor

**Input:** `text_processor.dsl`
```javascript
arg2 as text |> arg1 as operation |>
  operation == "upper" ? upper(text) :
  operation == "lower" ? lower(text) :
  operation == "length" ? length(text) :
  "Unknown operation"
```

**Compile:**
```bash
dsl-compiler build text_processor.dsl -o text_processor
```

**Run:**
```bash
./text_processor "upper" "hello world"
# Output: "HELLO WORLD"

./text_processor "length" "hello"
# Output: 5

./text_processor "unknown" "test"
# Output: "Unknown operation"
```

### Example 3: Data Pipeline

**Input:** `pipeline.dsl`
```javascript
// Read data, transform, and analyze
sql("SELECT * FROM 'data.csv'") |>
  _ as data |>
  length(data) as count |>
  upper("Processed ${count} rows")
```

**Compile:**
```bash
dsl-compiler build pipeline.dsl -o pipeline
```

**Run:**
```bash
./pipeline
# Output: "PROCESSED 1000 ROWS"
```

### Example 4: LLM Integration

**Input:** `summarize.dsl`
```javascript
arg1 as text |>
  ask("Summarize this text in one sentence: ${text}")
```

**Compile:**
```bash
dsl-compiler build summarize.dsl -o summarize
```

**Run:**
```bash
export OPENAI_API_KEY="sk-..."
./summarize "Long text here..."
# Output: AI-generated summary
```

## Command-Line Arguments

### Accessing Arguments

DSL programs can access command-line arguments through special variables:

| Variable | Description | Example |
|----------|-------------|---------|
| `arg1` | First argument | `./program "hello"` |
| `arg2` | Second argument | `./program "a" "b"` |
| `arg3` | Third argument | `./program "a" "b" "c"` |
| `args` | List of all arguments | `[arg1, arg2, arg3]` |

### Example Program

```javascript
// cli_tool.dsl
args as all_args |>
  length(all_args) == 0 ?
    "Usage: cli_tool <command> [args...]" :
  arg1 == "help" ?
    "Commands: help, version, process" :
  arg1 == "version" ?
    "v1.0.0" :
  arg1 == "process" ?
    upper("Processing: ${arg2}") :
  "Unknown command: ${arg1}"
```

**Usage:**
```bash
./cli_tool
# Output: "Usage: cli_tool <command> [args...]"

./cli_tool help
# Output: "Commands: help, version, process"

./cli_tool process "data"
# Output: "PROCESSING: DATA"
```

## Debugging

### Debug Artifacts

Save intermediate files for debugging:

```bash
dsl-compiler build program.dsl -o program \
  --emit-ir program.ir.json \
  --emit-rust program.rs
```

**Inspect IR:**
```bash
cat program.ir.json | jq .
```

**Inspect Generated Rust:**
```bash
cat program.rs
```

### Common Issues

#### 1. Compilation Fails

**Check DSL syntax first:**
```bash
dsl-compiler check program.dsl
```

**Look at the IR:**
```bash
dsl-compiler ir program.dsl -o debug.ir.json --json
cat debug.ir.json
```

#### 2. Runtime Errors

The binary will show detailed error messages:

```bash
./program
# Error: Undefined variable: foo
```

**Fix the DSL and recompile:**
```bash
# Edit program.dsl
dsl-compiler build program.dsl -o program
./program  # Try again
```

#### 3. Missing Dependencies

If you see "function not found" errors:

```bash
# Check available builtins
# upper, lower, length, join, not, render_markdown
# ask, extract_as (require OPENAI_API_KEY)
# sql (requires data)
```

### Verbose Output

The compiler shows progress during compilation:

```
Parsing examples/greet.dsl...
Compiling to IR...
Generating Rust code...
Creating temporary Cargo project...
Compiling with cargo...
   Compiling dsl_generated v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.25s
✓ Built successfully: greet
```

## Performance

### Build Times

| Mode | Time | Use Case |
|------|------|----------|
| Debug | ~2-3s | Development, testing |
| Release | ~8-10s | Production deployment |

**First build** (downloads dependencies): ~30-60s
**Incremental builds** (cached): ~2-3s

### Binary Size

| Configuration | Size | Notes |
|---------------|------|-------|
| Debug | ~29 MB | Includes debug symbols |
| Release (strip) | ~20 MB | Debug symbols stripped |

**Size breakdown:**
- Interpreter: ~5 MB
- Runtime (tokio, duckdb): ~15 MB
- Your DSL program (IR): <100 KB

### Runtime Performance

Compiled binaries have similar performance to the REPL:

- **Cold start**: ~10-50ms (interpreter initialization)
- **Execution**: Same as REPL (tree-walk interpreter)
- **I/O operations**: Native speed (HTTP, SQL, LLM calls)

**Note**: The embedded interpreter approach prioritizes correctness and simplicity over raw performance. For CPU-intensive workloads, consider:
- Caching results
- Using SQL for data processing
- Batching operations

## Troubleshooting

### Problem: Compiler not found

```bash
dsl-compiler: command not found
```

**Solution:**
```bash
# Use full path
./target/release/dsl-compiler build program.dsl -o program

# Or add to PATH
export PATH="$PATH:$(pwd)/target/release"
```

### Problem: Cargo build fails

```bash
error: could not compile `dsl_generated`
```

**Solutions:**

1. **Check Rust installation:**
   ```bash
   rustc --version  # Should be 1.70+
   ```

2. **Clean build cache:**
   ```bash
   rm -rf /tmp/dsl_compiler_cache
   ```

3. **Update dependencies:**
   ```bash
   cd DSL
   cargo update
   cargo build --release -p dsl-compiler
   ```

### Problem: Binary crashes

```bash
./program
Segmentation fault
```

**Debug:**

1. **Check with REPL first:**
   ```bash
   cargo run --release --bin dsl
   # Test the same DSL code interactively
   ```

2. **Enable Rust backtrace:**
   ```bash
   RUST_BACKTRACE=1 ./program
   ```

3. **Recompile with debug info:**
   ```bash
   dsl-compiler build program.dsl -o program
   # (Debug mode includes more info)
   ```

### Problem: Environment variables

```bash
./program
Error: OPENAI_API_KEY not set
```

**Solution:**
```bash
export OPENAI_API_KEY="sk-..."
./program
```

**Or pass inline:**
```bash
OPENAI_API_KEY="sk-..." ./program
```

## Advanced Usage

### Compile to Library

Generate a reusable library instead of executable:

```bash
dsl-compiler build mylib.dsl -o libmylib.rlib --lib
```

**Use cases:**
- Embedding DSL in Rust projects
- Creating DSL modules
- Code reuse across programs

### CI/CD Integration

**GitHub Actions example:**

```yaml
name: Validate DSL

on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Build compiler
        run: cargo build --release -p dsl-compiler

      - name: Validate all DSL files
        run: |
          for file in examples/*.dsl; do
            ./target/release/dsl-compiler check "$file"
          done

      - name: Build binaries
        run: |
          ./target/release/dsl-compiler build examples/greet.dsl -o greet
          ./greet "CI"  # Test it works
```

### Cross-Compilation

The compiler generates Rust code, so you can cross-compile:

```bash
# 1. Compile DSL to Rust
dsl-compiler build program.dsl -o program --emit-rust program.rs

# 2. Cross-compile the Rust code
cd /tmp/dsl_compiler_cache/build
cargo build --release --target x86_64-pc-windows-gnu
```

**Supported targets:**
- Linux: x86_64-unknown-linux-gnu
- macOS: x86_64-apple-darwin, aarch64-apple-darwin
- Windows: x86_64-pc-windows-gnu

## Best Practices

### 1. Use Meaningful Names

```javascript
// Good
user_data |> process_records(_) |> save_results(_)

// Bad
x |> f(_) |> g(_)
```

### 2. Validate Input

```javascript
// Validate command-line arguments
length(args) == 0 ?
  "Error: Missing arguments" :
  process(arg1)
```

### 3. Handle Errors

```javascript
// Check before using
sql("SELECT * FROM nonexistent")  // Will error

// Better: validate first
file_exists("data.csv") ?
  sql("SELECT * FROM 'data.csv'") :
  "Error: data.csv not found"
```

### 4. Comment Your Code

```javascript
// Process user input
arg1 as input |>

// Validate and transform
upper(input) as normalized |>

// Generate result
"Result: ${normalized}"
```

### 5. Test Before Compiling

```bash
# 1. Test in REPL first
cargo run --release --bin dsl
# Try your code interactively

# 2. Then compile
dsl-compiler build program.dsl -o program
```

## Next Steps

- Read [DESIGN.md](DESIGN.md) for language specification
- Browse [examples/](examples/) for more examples
- Check [IR_MIGRATION_PLAN.md](IR_MIGRATION_PLAN.md) for architecture details
- Try the interactive REPL with `cargo run --bin dsl`

## Getting Help

- **Issues**: Report bugs at GitHub issues
- **Questions**: Check the documentation
- **Examples**: See `examples/` directory

---

**Happy compiling!** 🚀
