# Compiling DSL to Binary

This guide explains how to compile DSL programs into native executable binaries using the `dsl-compiler` tool.

## Table of Contents

- [Overview](#overview)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Compiler Commands](#compiler-commands)
- [Build Options](#build-options)
- [How It Works](#how-it-works)
- [Examples](#examples)
- [Troubleshooting](#troubleshooting)

---

## Overview

The DSL compiler transforms your DSL programs into standalone native binaries. Unlike the REPL which interprets code interactively, compiled binaries:

- **Run independently**: No need for the DSL REPL or runtime
- **Execute faster**: Native code with optimized dependencies
- **Deploy easily**: Single binary you can distribute
- **Behave identically**: Uses the same interpreter as REPL for guaranteed correctness

### Architecture

The compiler uses an **embedded interpreter approach**:

```
DSL Source → Parser → AST → IR Compiler → IR (JSON)
                                            ↓
                                    Code Generator
                                            ↓
                                    Embed IR + Interpreter
                                            ↓
                                    Cargo Build
                                            ↓
                                    Native Binary
```

The generated binary contains:
1. Your program's IR (Intermediate Representation) as embedded JSON
2. The DSL interpreter
3. All runtime dependencies

This ensures **100% behavioral consistency** between REPL and compiled binaries.

---

## Installation

The `dsl-compiler` tool is built as part of the DSL workspace:

```bash
# Build the compiler
cargo build --release -p dsl-compiler

# The binary will be at:
# target/release/dsl-compiler
```

Optionally, install it globally:

```bash
cargo install --path crates/dsl-compiler
```

---

## Quick Start

### 1. Create a DSL Program

**File**: `hello.dsl`
```dsl
upper("hello world")
```

### 2. Check Syntax

```bash
dsl-compiler check hello.dsl
```

**Output**:
```
✅ hello.dsl is valid
```

### 3. Build Binary

```bash
dsl-compiler build hello.dsl -o hello
```

**Output**:
```
Parsing hello.dsl...
Compiling to IR...
Generating Rust code...
Compiling with cargo...
Built successfully: hello
```

### 4. Run the Binary

```bash
./hello
```

**Output**:
```
"HELLO WORLD"
```

---

## Compiler Commands

The `dsl-compiler` tool provides four main commands:

### `check` - Validate Syntax

Validates a DSL program without building it.

```bash
dsl-compiler check <file.dsl>
```

**Usage**:
```bash
dsl-compiler check examples/data_pipeline.dsl
```

**Exit codes**:
- `0` - Program is valid
- `1` - Syntax or compilation errors found

---

### `ir` - Compile to IR

Compiles DSL source to Intermediate Representation (IR) format.

```bash
dsl-compiler ir <input.dsl> -o <output.ir> [--json]
```

**Options**:
- `-o, --output <file>` - Output file path (required)
- `--json` - Output as JSON instead of MessagePack (default is binary MessagePack)

**Examples**:

Compile to MessagePack (binary format):
```bash
dsl-compiler ir program.dsl -o program.ir
```

Compile to JSON (human-readable):
```bash
dsl-compiler ir program.dsl -o program.json --json
```

**Use cases**:
- Debugging program structure
- Inspecting IR format
- Pre-compiling for caching
- Integration with other tools

---

### `clean` - Clear Build Cache

Removes the persistent build cache to free up disk space or force a fresh rebuild.

```bash
dsl-compiler clean
```

**When to use**:
- Free up disk space (~500MB cache)
- Force fresh rebuild after dependency updates
- Troubleshoot build issues

**Cache location**: `/tmp/dsl_compiler_cache` (or `$TMPDIR` on macOS)

**Note**: The cache significantly speeds up subsequent builds (2-3s vs 30-40s). Only clean if necessary.

---

### `build` - Compile to Binary

Compiles DSL source to a native executable binary.

```bash
dsl-compiler build <input.dsl> -o <output> [OPTIONS]
```

**Required**:
- `<input.dsl>` - Input DSL source file
- `-o, --output <file>` - Output binary path

**Options**:
- `--emit-ir <file>` - Save IR to file during build
- `--emit-rust <file>` - Save generated Rust code to file
- `--lib` - Generate a library instead of executable
- `--release` - Build with optimizations (slower build, faster runtime)

---

## Build Options

### Basic Build

```bash
dsl-compiler build program.dsl -o program
```

Produces a debug binary at `./program`.

### Release Build

```bash
dsl-compiler build program.dsl -o program --release
```

Produces an optimized binary:
- **Faster runtime**: 2-10x performance improvement
- **Slower build**: Takes longer to compile
- **Smaller binary**: Better optimization

**Recommendation**: Use `--release` for production deployments.

### Save IR for Inspection

```bash
dsl-compiler build program.dsl -o program --emit-ir program.json
```

Saves the IR to `program.json` (as JSON) so you can inspect the intermediate representation.

### Save Generated Rust Code

```bash
dsl-compiler build program.dsl -o program --emit-rust program.rs
```

Saves the generated Rust source code to `program.rs` for inspection or debugging.

### Build as Library

```bash
dsl-compiler build program.dsl -o libprogram.rlib --lib
```

Generates a Rust library instead of an executable. Useful for:
- Integrating DSL functions into Rust projects
- Creating reusable components
- Building larger systems

---

## How It Works

### Step-by-Step Compilation Process

#### 1. Parse DSL Source

```dsl
upper("hello")
```

The parser converts this to an Abstract Syntax Tree (AST):
```rust
Expr::FunctionCall {
    name: "upper",
    args: vec![Expr::String("hello")]
}
```

#### 2. Compile to IR

The AST is transformed into Intermediate Representation:
```json
{
  "entry_expr": {
    "FunctionCall": {
      "name": "upper",
      "args": [
        {
          "TemplateString": [
            { "Text": "hello" }
          ]
        }
      ]
    }
  }
}
```

#### 3. Generate Rust Code

The code generator creates a Rust program that **embeds the IR**:

```rust
use dsl_runtime::{Result, anyhow};
use dsl_interpreter::Interpreter;
use dsl_ir::IR;

#[tokio::main]
async fn main() -> Result<()> {
    // Embedded IR
    let ir_json = r#"{
      "entry_expr": {
        "FunctionCall": {
          "name": "upper",
          "args": [...]
        }
      }
    }"#;

    // Parse IR
    let ir: IR = serde_json::from_str(ir_json)?;

    // Create interpreter
    let mut interpreter = Interpreter::new()?;

    // Load types and functions
    for class in &ir.types {
        interpreter.runtime.types.register_class(class.clone());
    }

    // Execute
    let result = interpreter.eval(&ir.entry_expr).await?;

    println!("{}", result.display());

    Ok(())
}
```

#### 4. Cargo Build

The compiler creates a temporary Cargo project:

```toml
[package]
name = "dsl_generated"
version = "0.1.0"
edition = "2021"

[dependencies]
dsl-runtime = { path = "..." }
dsl-interpreter = { path = "..." }
dsl-ir = { path = "..." }
tokio = { version = "1", features = ["full"] }
anyhow = "1"
serde_json = "1"
```

Then runs `cargo build` to produce the final binary.

#### 5. Extract Binary

The compiled binary is copied to the target location.

**Important**: The build cache is **preserved** for faster subsequent builds. The cache includes:
- Compiled dependencies (tokio, simplify_baml, duckdb, etc.)
- Intermediate build artifacts
- Cargo metadata

This means:
- **First build**: 30-40 seconds (compiles everything)
- **Subsequent builds**: 2-5 seconds (reuses cache, only recompiles your DSL program)

To free up disk space (~500MB), run:
```bash
dsl-compiler clean
```

---

## Command-Line Arguments

Compiled binaries automatically have access to command-line arguments through special pre-defined variables that are injected at runtime.

### Quick Reference

| Variable | Type | Description | Example |
|----------|------|-------------|---------|
| `args` | `List<String>` | All command-line arguments | `["foo", "bar"]` |
| `arg1` | `String` | First argument | `"foo"` |
| `arg2` | `String` | Second argument | `"bar"` |
| `arg3` | `String` | Third argument | `"baz"` |
| `argN` | `String` | Nth argument | Continues as needed |

**Note**: The program name (argv[0]) is excluded. Only user-provided arguments are included.

### `args` Variable

A list containing all command-line arguments:

```dsl
args  // ["arg1_value", "arg2_value", ...]
```

**Use cases**:
- Iterate over all arguments
- Check argument count with `length(args)`
- Process variable number of arguments

### Positional Variables

Individual arguments are available as `arg1`, `arg2`, `arg3`, etc.:

```dsl
arg1  // First argument
arg2  // Second argument
```

**Use cases**:
- Access specific arguments directly
- Required parameters
- Simple command-line tools

### Example: Greeting Program

**File**: `greet.dsl`
```dsl
arg1 |> upper("Hello, ${_}!")
```

**Compile**:
```bash
dsl-compiler build greet.dsl -o greet
```

**Run with argument**:
```bash
./greet Alice
```

**Output**:
```
"HELLO, ALICE!"
```

**Run with multi-word name**:
```bash
./greet "Bob Smith"
```

**Output**:
```
"HELLO, BOB SMITH!"
```

### Example: Multiple Arguments

**File**: `calculator.dsl`
```dsl
arg1 as operation |> arg2 as a |> arg3 as b |>
  "${operation}: ${a} and ${b}"
```

**Run**:
```bash
./calculator add 5 10
```

**Output**:
```
"add: 5 and 10"
```

### Example: Argument List Processing

**File**: `join_args.dsl`
```dsl
join(args, ", ")
```

**Run**:
```bash
./join_args apple banana cherry
```

**Output**:
```
"apple, banana, cherry"
```

### Example: All Arguments Processing

**File**: `show_all_args.dsl`
```dsl
args |> join(_, " + ") |> upper(_)
```

**Run with no arguments**:
```bash
./show_all_args
```

**Output**:
```
""
```

**Run with arguments**:
```bash
./show_all_args apple banana cherry
```

**Output**:
```
"APPLE + BANANA + CHERRY"
```

---

## Examples

### Example 1: Simple String Processing

**File**: `string_ops.dsl`
```dsl
"hello world" |> upper(_) |> length(_)
```

**Compile and run**:
```bash
dsl-compiler build string_ops.dsl -o string_ops --release
./string_ops
```

**Output**:
```
11
```

---

### Example 2: Data Pipeline with SQL

**File**: `data.dsl`
```dsl
let data = [
  { "name": "Alice", "age": 30 },
  { "name": "Bob", "age": 25 },
  { "name": "Charlie", "age": 35 }
]

sql("SELECT name, age FROM data WHERE age > 28 ORDER BY age DESC")
```

**Compile**:
```bash
dsl-compiler build data.dsl -o data_query
```

**Run**:
```bash
./data_query
```

**Output**:
```
┌─────────┬─────┐
│ name    │ age │
├─────────┼─────┤
│ Charlie │ 35  │
│ Alice   │ 30  │
└─────────┴─────┘
```

---

### Example 3: LLM Integration

**File**: `extract.dsl`
```dsl
type Person {
  name: string
  age: int
  occupation: string
}

let text = "John Smith is a 35-year-old software engineer"

ExtractAs(text, "Person")
```

**Compile**:
```bash
dsl-compiler build extract.dsl -o extract --release
```

**Run** (requires `OPENAI_API_KEY`):
```bash
export OPENAI_API_KEY="sk-..."
./extract
```

**Output**:
```json
{
  "name": "John Smith",
  "age": 35,
  "occupation": "software engineer"
}
```

---

### Example 4: Command-Line Tool with Arguments

**File**: `text_processor.dsl`
```dsl
arg2 as text |> arg1 as operation |>
  operation == "upper" ? upper(text) :
  operation == "lower" ? lower(text) :
  operation == "length" ? length(text) :
  "Unknown operation: ${operation}"
```

**Compile**:
```bash
dsl-compiler build text_processor.dsl -o text_processor --release
```

**Usage examples**:
```bash
# Uppercase
./text_processor upper "hello world"
# Output: "HELLO WORLD"

# Lowercase
./text_processor lower "HELLO WORLD"
# Output: "hello world"

# Length
./text_processor length "hello"
# Output: 5

# Default (no arguments)
./text_processor
# Output: "HELLO WORLD"

# Unknown operation
./text_processor reverse "hello"
# Output: "Unknown operation: reverse"
```

---

### Example 5: File Path from Arguments

**File**: `file_info.dsl`
```dsl
arg1 as filepath |> {
  "filepath": filepath,
  "args_count": length(args),
  "all_args": args
}
```

**Compile and run**:
```bash
dsl-compiler build file_info.dsl -o file_info
./file_info /path/to/file.txt extra arg
```

**Output**:
```json
{
  "filepath": "/path/to/file.txt",
  "args_count": 3,
  "all_args": ["/path/to/file.txt", "extra", "arg"]
}
```

---

### Example 6: HTTP API Client

**File**: `api.dsl`
```dsl
function GetUser(user_id: string) -> string
  @llm(
    prompt: "Extract the user's name from this JSON response",
    model: "gpt-4"
  )
  @http(
    method: "GET",
    url: "https://jsonplaceholder.typicode.com/users/${user_id}"
  )
end

GetUser("1")
```

**Compile**:
```bash
dsl-compiler build api.dsl -o api_client
```

**Run**:
```bash
./api_client
```

**Output**:
```
"Leanne Graham"
```

---

## Troubleshooting

### Build Fails with Dependency Errors

**Problem**: `cargo build` fails to find dependencies.

**Solution**: Ensure you're building from within the DSL workspace:
```bash
# Run from the DSL project root
cd /path/to/DSL
dsl-compiler build program.dsl -o program
```

The compiler uses relative paths to find `dsl-runtime`, `dsl-interpreter`, and `dsl-ir` crates.

---

### Binary is Very Large

**Problem**: Debug binaries are 50-100MB.

**Solution**: Use `--release` flag:
```bash
dsl-compiler build program.dsl -o program --release
```

Release builds are 2-5x smaller due to optimizations and stripping.

---

### Slow Build Times

**Problem**: First build takes 30+ seconds.

**Explanation**: This is normal. Cargo is compiling all dependencies:
- tokio (async runtime)
- simplify_baml (LLM integration)
- duckdb (SQL engine)
- reqwest (HTTP client)

**Solution**: The compiler **caches dependencies automatically**:
- **First build**: ~36 seconds (compiles everything)
- **Second build**: ~2 seconds (reuses cache) ✨
- **16x faster** for subsequent builds!

The cache persists across builds, so you only pay the compilation cost once.

**Additional tips**:
1. **Use REPL for development**: Iterate quickly without compiling
2. **Use `check` command**: Validates syntax instantly without building
3. **Keep the cache**: Only run `dsl-compiler clean` if you need to free disk space

---

### Runtime Errors

**Problem**: Binary crashes or produces wrong output.

**Debug steps**:

1. **Test in REPL first**:
   ```bash
   dsl-tui
   # Run your code interactively
   ```

2. **Inspect IR**:
   ```bash
   dsl-compiler build program.dsl -o program --emit-ir program.json
   cat program.json
   ```

3. **Check generated code**:
   ```bash
   dsl-compiler build program.dsl -o program --emit-rust program.rs
   cat program.rs
   ```

4. **Validate with check**:
   ```bash
   dsl-compiler check program.dsl
   ```

---

### Environment Variables Not Set

**Problem**: LLM functions fail with "API key not found".

**Solution**: Export required environment variables:
```bash
export OPENAI_API_KEY="sk-..."
./program
```

Or set them inline:
```bash
OPENAI_API_KEY="sk-..." ./program
```

---

### Missing Command-Line Arguments

**Problem**: Program fails because `arg1` is undefined.

**Example Error**:
```
Error: Undefined variable: arg1
```

**Solution**: The DSL does not have conditional operators (`>`, `<`, `==`) in expressions yet. Use simple argument access:

```dsl
arg1 |> upper("Hello, ${_}!")
```

If no argument is provided and the program tries to access `arg1`, you'll get an "Undefined variable" error. Design your programs to expect the right number of arguments, or handle this at the shell level:

```bash
#!/bin/bash
if [ $# -eq 0 ]; then
  echo "Usage: $0 <name>"
  exit 1
fi
./greet "$1"
```

---

## Performance Characteristics

### Build Performance

| Build Type | First Build (Cold Cache) | Subsequent Builds (Warm Cache) | Speedup |
|------------|--------------------------|--------------------------------|---------|
| Debug      | ~36 seconds              | ~2 seconds                     | 18x     |
| Release    | ~60 seconds              | ~3-5 seconds                   | 12-20x  |

**Cache benefits**:
- Dependencies are compiled once and reused
- Only your DSL program is recompiled on changes
- Cache persists across different DSL programs
- Automatic - no configuration needed

### Runtime Performance

Compiled binaries have identical performance to REPL:
- Same interpreter
- Same runtime
- Same dependencies

The main advantage is **deployment convenience**, not speed.

---

## Best Practices

### 1. Use REPL for Development

Develop and test interactively:
```bash
dsl-tui
# Iterate quickly
```

Compile only when deploying:
```bash
dsl-compiler build final.dsl -o app --release
```

### 2. Validate Before Building

```bash
dsl-compiler check program.dsl && \
  dsl-compiler build program.dsl -o program --release
```

### 3. Save IR for Documentation

```bash
dsl-compiler build program.dsl -o program \
  --emit-ir program.json \
  --emit-rust program.rs
```

Helps with debugging and understanding compilation.

### 4. Use Release Mode for Production

```bash
dsl-compiler build app.dsl -o app --release
```

Smaller, faster binaries.

### 5. Version Control

**Include**:
- `*.dsl` - Source files
- Build scripts

**Exclude** (add to `.gitignore`):
- Compiled binaries
- `*.ir` files
- Generated `*.rs` files

---

## Advanced Usage

### Building Libraries

Compile DSL code as a library for use in Rust projects:

```bash
dsl-compiler build my_lib.dsl -o libmy_lib.rlib --lib
```

### Cross-Compilation

Use Cargo's cross-compilation features:

```bash
# Target Linux from macOS
cargo build --release --target x86_64-unknown-linux-gnu

# Target Windows from Linux
cargo build --release --target x86_64-pc-windows-gnu
```

### CI/CD Integration

```yaml
# .github/workflows/build.yml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
      - run: cargo build --release -p dsl-compiler
      - run: ./target/release/dsl-compiler build app.dsl -o app --release
      - uses: actions/upload-artifact@v2
        with:
          name: app-binary
          path: app
```

---

## Comparison: REPL vs Compiled

| Feature | REPL | Compiled Binary |
|---------|------|----------------|
| **Development** | ✅ Fast iteration | ❌ Slow iteration |
| **Deployment** | ❌ Needs runtime | ✅ Standalone |
| **Startup Time** | ❌ ~100ms | ✅ ~10ms |
| **Interactive** | ✅ Yes | ❌ No |
| **Distribution** | ❌ Complex | ✅ Single file |
| **Correctness** | ✅ Same | ✅ Same |
| **Performance** | ✅ Same | ✅ Same |

**Use REPL when**: Developing, experimenting, interactive analysis

**Use Compiled when**: Deploying, distributing, production systems

---

## Conclusion

The DSL compiler provides a seamless path from development to deployment:

1. **Develop** interactively in the REPL
2. **Validate** with `dsl-compiler check`
3. **Compile** with `dsl-compiler build`
4. **Deploy** the standalone binary

The embedded interpreter approach ensures your compiled programs behave **exactly** as they do in the REPL, giving you confidence in production deployments.

For more information, see:
- [Language Features](04-Language-Features.md) - DSL syntax and features
- [LLM Integration](08-LLM-Integration.md) - Using LLM functions
- [SQL/DuckDB](09-SQL-DuckDB.md) - SQL queries
- [HTTP Client](10-HTTP-Client.md) - HTTP functions
