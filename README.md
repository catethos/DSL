# Agentic LLM Workflow DSL

A powerful Domain-Specific Language (DSL) for building agentic LLM workflows with both **interactive REPL** and **native binary compilation**.

## 🎯 Overview

This project provides a complete DSL for orchestrating AI-powered workflows, featuring:

- **🔄 Dual Execution Modes**
  - **Interactive REPL** - Test expressions and explore the language interactively
  - **Binary Compiler** - Compile DSL to standalone native executables
- **📝 Full-featured Editor** - Write multi-line workflows with syntax highlighting
- **👁️ Live Preview** - Watch your workflows execute step-by-step
- **🏗️ Type System** - Define custom types for structured data
- **🤖 LLM Integration** - Use AI models via simplify_baml
- **🗄️ SQL/DuckDB Support** - Process data with SQL queries
- **⚡ Parallel Execution** - Run operations concurrently

## 🏛️ Architecture

The DSL uses a modern IR-based architecture with multiple execution paths:

```
DSL Source (.dsl)
       ↓
    Parser (Pest)
       ↓
      AST
       ↓
  IR Compiler
       ↓
   IR (JSON/MessagePack)
       ↓
    ┌──┴──┐
    ↓     ↓
  REPL   Binary
    ↓     ↓
Execute  Native Executable
```

**Key Components**:
- **dsl-core** - Parser, AST, IR compiler
- **dsl-ir** - Intermediate Representation (serializable)
- **dsl-interpreter** - Tree-walk interpreter for IR
- **dsl-codegen** - Rust code generator (embeds IR + interpreter)
- **dsl-compiler** - CLI tool for building binaries
- **dsl-runtime** - Runtime support library for compiled programs
- **dsl-repl/dsl-tui** - Interactive REPL interface

## 🚀 Quick Start

### Prerequisites

- Rust (latest stable version)
- Optional: OpenAI API key for LLM features

### Installation

```bash
# Clone the repository
cd DSL

# Build the entire workspace (includes REPL + compiler)
cargo build --release --workspace

# Or build individually
cargo build --release -p dsl-repl      # Interactive REPL
cargo build --release -p dsl-compiler  # Binary compiler
```

### Usage: Interactive REPL

```bash
# Run the interactive TUI
cargo run --release --bin dsl

# Or use the built binary
./target/release/dsl
```

### Usage: Compile to Binary

```bash
# Build the compiler first
cargo build --release -p dsl-compiler

# Compile a DSL program to native binary
./target/release/dsl-compiler build input.dsl -o output

# Run the compiled binary
./output

# With arguments (args are available as arg1, arg2, etc.)
./output "hello" "world"
```

**Compiler Commands**:

```bash
# Check DSL for errors (no compilation)
dsl-compiler check program.dsl

# Compile to IR (for inspection)
dsl-compiler ir program.dsl -o program.ir.json

# Full compilation with debug info
dsl-compiler build program.dsl -o program --emit-rust generated.rs --emit-ir program.ir.json
```

### First Steps

1. **REPL Mode (F1)** - Start here for interactive exploration
   ```
   flow> 42 as answer
   ✓ Bound 'answer' to 42 : Int
   
   flow> answer * 2
   ✓ 84 : Int
   ```

2. **Editor Mode (F2)** - Write multi-line workflows
   - Type your DSL code
   - Press F2 to execute
   - View results in Preview mode

3. **Type Explorer (F3)** - Browse registered types
   - Define types with `type` keyword
   - View all registered types and their fields

4. **Preview Mode (F4)** - See execution results
   - Step-by-step execution tracking
   - Timing and status for each operation

## 📚 Language Features

### Variables & Data Types

```javascript
// Primitives
42 as number
"hello" as text
3.14 as pi
true as flag

// Lists
[1, 2, 3, 4, 5] as numbers

// Access
numbers[0]  // First element
```

### Built-in Functions

```javascript
// String operations
Upper("hello")      // "HELLO"
Lower("WORLD")      // "world"
Length("hello")     // 5
Join(["a","b"], ",") // "a,b"

// LLM (requires OPENAI_API_KEY)
Ask("What is Rust?")
```

### Sequential Composition (`|>`)

Chain operations together with the pipe operator:

```javascript
// Simple chaining
"hello" |> upper(_) |> length(_)

// With binding
[1,2,3,4,5] |> length(_) as count |> count * 2

// Access previous result with _
"world" |> upper("Hello, ${_}!")
```

### Parallel Execution (`||`)

Run operations concurrently:

```javascript
// Collect results
(5 || 10 || 15) as numbers

// With destructuring
(Ask("color") || Ask("fruit")) as [color, fruit]
```

### Custom Types

```javascript
// Define types
type Person {
  name: String
  age: Int
  email: String
}

// Define enums
enum Status {
  Pending
  InProgress
  Completed
  Failed
}
```

### User-Defined Functions

```javascript
def greet(name: String) {
  prompt """
  Generate a friendly greeting for ${name}
  """
}

greet("Alice")
```

### Multi-Provider LLM Support

Use OpenRouter to access 100+ models from multiple providers with separate API keys:

```javascript
// Claude via OpenRouter
def AnalyzeWithClaude(text: String) {
  base_url: "https://openrouter.ai/api/v1"
  model: "anthropic/claude-3.5-sonnet"
  api_key_env: "OPENROUTER_API_KEY"
  prompt: "Analyze: ${text}"
}

// GPT-4 via OpenRouter
def AnalyzeWithGPT4(text: String) {
  base_url: "https://openrouter.ai/api/v1"
  model: "openai/gpt-4-turbo"
  api_key_env: "OPENROUTER_API_KEY"
  prompt: "Analyze: ${text}"
}

// Native OpenAI (no base_url needed)
def AnalyzeWithOpenAI(text: String) {
  model: "gpt-4o-mini"
  // api_key_env defaults to "OPENAI_API_KEY"
  prompt: "Analyze: ${text}"
}

// Native Anthropic API
def AnalyzeWithAnthropic(text: String) {
  base_url: "https://api.anthropic.com/v1"
  model: "claude-3-5-sonnet-20241022"
  api_key_env: "ANTHROPIC_API_KEY"
  prompt: "Analyze: ${text}"
}
```

**Setup Multiple Providers:**
```bash
# OpenRouter (access to 100+ models)
export OPENROUTER_API_KEY="sk-or-v1-..."

# Native OpenAI (default)
export OPENAI_API_KEY="sk-..."

# Native Anthropic
export ANTHROPIC_API_KEY="sk-ant-..."
```

**Benefits:**
- ✅ Use different API keys for different providers
- ✅ Switch between providers without changing environment variables
- ✅ Mix OpenRouter, native OpenAI, and native Anthropic in same project
- ✅ Better cost control and provider management

### SQL/DuckDB Integration

```javascript
// Query CSV files directly
SQL("SELECT * FROM 'data.csv' WHERE age > 25")

// Create tables from data
my_data as table
SQL("SELECT COUNT(*) FROM table")
```

## 🎮 UI Modes

### Mode Switching

| Key | Mode | Description |
|-----|------|-------------|
| **F1** | REPL | Interactive command line |
| **F2** | Editor | Multi-line workflow editor (press F2 again to run) |
| **F3** | Type Explorer | Browse registered types |
| **F4** | Preview | View execution results |
| **Esc** | - | Quit application |

### Editor Shortcuts

| Key | Action |
|-----|--------|
| **Ctrl+S** | Save file |
| **Arrow Keys** | Navigate text |
| **Home/End** | Start/end of line |
| **Enter** | New line |
| **Backspace** | Delete character |

## 🏗️ Detailed Architecture

The project uses a modern **IR-based compilation pipeline** with multiple crates:

### **Core Compilation Pipeline**

1. **dsl-core** - Parser & IR Compiler
   - Pest-based parser → AST
   - AST → IR compiler
   - Type system and validation

2. **dsl-ir** - Intermediate Representation
   - Serializable IR (JSON/MessagePack)
   - Type-safe representation
   - Platform-independent

3. **dsl-interpreter** - Runtime Interpreter
   - Tree-walk interpreter for IR
   - Builtin functions (LLM, SQL, HTTP)
   - Runtime type registry

4. **dsl-codegen** - Code Generator
   - IR → Rust code generation
   - Embeds IR + interpreter approach
   - Type definitions generation

5. **dsl-compiler** - CLI Tool
   - `check` - Validate DSL programs
   - `ir` - Compile to IR
   - `build` - Compile to native binary

6. **dsl-runtime** - Runtime Support
   - Re-exports for generated code
   - Builtin functions library
   - Value types and utilities

### **User Interface**

7. **dsl-tui** - Terminal UI Library
   - Interactive REPL mode
   - Multi-line editor with syntax highlighting
   - Type explorer & preview

8. **dsl-repl** - Binary Application
   - Main TUI application
   - Combines core + interpreter + UI

### **Why IR-Based Architecture?**

✅ **Dual Execution**: Same code runs in REPL and as compiled binary
✅ **Serialization**: Programs can be saved/loaded as IR
✅ **Optimization**: IR enables future optimization passes
✅ **Multiple Targets**: Easy to add WASM, other platforms
✅ **Debugging**: IR is human-readable (JSON format)

## 📁 Project Structure

```
DSL/
├── Cargo.toml                 # Workspace configuration
├── crates/
│   ├── dsl-core/             # Parser & IR compiler
│   │   ├── src/parser/       # Pest grammar, AST
│   │   └── src/compiler.rs   # AST → IR compiler
│   │
│   ├── dsl-ir/               # Intermediate Representation
│   │   └── src/ir.rs         # Serializable IR types
│   │
│   ├── dsl-interpreter/      # IR Interpreter
│   │   ├── src/interpreter.rs  # Tree-walk interpreter
│   │   ├── src/builtins.rs     # Builtin functions
│   │   └── src/sql.rs          # SQL execution
│   │
│   ├── dsl-codegen/          # Code Generator
│   │   ├── src/program.rs    # Main codegen (embeds IR)
│   │   └── src/rust_ast.rs   # Rust AST types
│   │
│   ├── dsl-compiler/         # CLI Compiler (binary)
│   │   └── src/main.rs       # check/ir/build commands
│   │
│   ├── dsl-runtime/          # Runtime Support Library
│   │   └── src/lib.rs        # Re-exports for generated code
│   │
│   ├── dsl-tui/              # Terminal UI (library)
│   │   ├── src/app.rs        # Application state
│   │   └── src/ui/           # Rendering components
│   │
│   └── dsl-repl/             # REPL Application (binary)
│       └── src/main.rs       # TUI entry point
│
├── tree-sitter-dsl/          # Syntax highlighting
├── examples/                 # DSL programs
│   ├── greet.dsl
│   ├── text_processor.dsl
│   └── openrouter_multi_provider.dsl
├── docs/                     # Documentation
│   ├── IR_MIGRATION_PLAN.md
│   ├── PHASE_9_SUMMARY.md
│   └── PHASE_9.5_SUMMARY.md
└── README.md
```

## 🔧 Development Status

**Current Status:** Phase 9 Complete - Production Ready! ✅

### **Original Phases (Complete)**
- ✅ Phase 0: Minimal REPL
- ✅ Phase 1: Expression Evaluator
- ✅ Phase 2: Variable Binding
- ✅ Phase 3: LLM Integration
- ✅ Phase 4: Type System
- ✅ Phase 5: Sequential Composition
- ✅ Phase 6: SQL/DuckDB
- ✅ Phase 7: Parallel Execution
- ✅ Phase 8: Full Editor & Preview

### **IR Migration (Complete)**
- ✅ Phase 1: dsl-ir crate with serialization
- ✅ Phase 2: IR extended for agentic features
- ✅ Phase 3: IR compiler in dsl-core
- ✅ Phase 4: dsl-interpreter crate
- ✅ Phase 5: REPL using IR
- ✅ Phase 6: dsl-codegen (code generator)
- ✅ Phase 7: dsl-compiler CLI
- ✅ Phase 8: dsl-runtime support library
- ✅ Phase 9: Testing and validation (embedded interpreter approach)
- ✅ Phase 9.5: Quality & Polish (bug fixes, optimization)

### **Features Implemented**
- ✅ Interactive REPL with history
- ✅ **Native Binary Compilation** (new!)
- ✅ Multi-mode TUI (REPL/Editor/Preview/Type Explorer)
- ✅ Syntax highlighting
- ✅ Type system (classes and enums)
- ✅ 11 builtin functions
- ✅ LLM integration via simplify_baml
- ✅ SQL queries with DuckDB
- ✅ Sequential composition (`|>`)
- ✅ Parallel execution (`||`)
- ✅ User-defined functions (LLM, HTTP, SQL)
- ✅ Template string interpolation
- ✅ **Comparison operators** (==, !=, <, >, <=, >=)
- ✅ Session save/load
- ✅ Workflow execution with step tracking
- ✅ **124 tests passing**

## 📖 Documentation

- **[BUILD.md](BUILD.md)** - Comprehensive build and compilation guide
- **[DESIGN.md](DESIGN.md)** - Complete language specification
- **[REPL_FIRST_PLAN.md](REPL_FIRST_PLAN.md)** - Incremental development plan
- **[PROGRESS.md](PROGRESS.md)** - Detailed progress tracking
- **[examples/README.md](examples/README.md)** - Example workflows guide

## 🧪 Examples

See the `examples/` directory for complete workflow examples:

1. **greet.dsl** - Simple greeting with string operations
   ```bash
   ./dsl-compiler build examples/greet.dsl -o greet
   ./greet "World"  # Output: "HELLO, WORLD!"
   ```

2. **text_processor.dsl** - Command-line text processor with conditionals
   ```bash
   ./text_processor "upper" "hello"   # Output: "HELLO"
   ./text_processor "length" "hello"  # Output: 5
   ```

3. **openrouter_multi_provider.dsl** - Multi-provider LLM access
   - Use Claude, GPT-4, and other models with separate API keys

4. **Sequential workflows** - Chaining with `|>`
5. **Parallel workflows** - Concurrent execution with `||`

## 🔑 Environment Variables

```bash
# For LLM features
export OPENAI_API_KEY="sk-..."

# Run the DSL TUI
cargo run --bin dsl

# Or with release optimizations
cargo run --release --bin dsl
```

## 🧩 Using DSL as a Library

The DSL can be embedded in your Rust applications using the IR-based API:

```rust
use dsl_core::{parse_expr, compile_to_ir};
use dsl_interpreter::Interpreter;
use dsl_ir::Value;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse DSL code
    let expr = parse_expr("42 * 2")?;

    // Compile to IR
    let ir_node = compile_to_ir(&expr)?;

    // Create interpreter and execute
    let mut interpreter = Interpreter::new()?;
    let result = interpreter.eval(&ir_node).await?;

    println!("Result: {}", result.display());  // "84"
    Ok(())
}
```

**Add to your Cargo.toml:**
```toml
[dependencies]
dsl-core = { path = "../DSL/crates/dsl-core" }
dsl-interpreter = { path = "../DSL/crates/dsl-interpreter" }
dsl-ir = { path = "../DSL/crates/dsl-ir" }
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
```

**Alternative: Use the full IR API for complex programs:**

```rust
use dsl_core::compile_to_ir;
use dsl_interpreter::Interpreter;
use std::fs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load and compile a .dsl file
    let source = fs::read_to_string("program.dsl")?;
    let ir = compile_to_ir(&source)?;

    // Create interpreter from IR (includes types and functions)
    let mut interpreter = Interpreter::from_ir(&ir)?;

    // Execute the program
    let result = interpreter.eval(&ir.entry_expr).await?;
    println!("{}", result.display());

    Ok(())
}
```

## 🎨 Features Showcase

### Syntax Highlighting

The editor provides full syntax highlighting:
- **Keywords** (type, enum, def, as) - Magenta
- **Types** (String, Int, Float, Bool) - Blue
- **Operators** (|>, ||, ==, !=, ?:, ->) - Cyan
- **Strings** - Green
- **Numbers** - Yellow

### Live Preview

Watch your workflow execute:
```
┌─ Preview ────────────────────────┐
│ ✓ Step 1: 42 as answer [0.01s]  │
│ ⏳ Step 2: answer * 2            │
│ ○ Step 3: Length("hello")        │
└──────────────────────────────────┘
```

### Type Explorer

Browse all registered types:
```
type Person
  name: String
  age: Int
  email: String

enum Status
  Pending
  InProgress
  Completed
```

## 🚧 Roadmap

**Completed (Phase 9):**
- ✅ IR-based compilation pipeline
- ✅ Multi-crate architecture (8 crates)
- ✅ Reusable core library
- ✅ Interactive REPL with TUI
- ✅ **Native binary compiler** (dsl-compiler CLI)
- ✅ Comprehensive test suite (124 tests)
- ✅ All comparison operators
- ✅ Clean code generation (no warnings)

**Optional Enhancements (Phase 10-12):**
- 📋 Phase 10: Agent syntax (spawn, send, receive, broadcast)
- 📋 Phase 11: Analysis passes (type checking, variable resolution)
- 📋 Phase 12: Documentation site

**Future Features:**
- Language server (`dsl-lsp`) for IDE integration
- WASM compilation target
- Binary size optimization (LTO, UPX)
- File picker (Ctrl+O) in TUI
- Auto-save functionality
- Configuration file (`~/.dsl-config.toml`)
- Streaming LLM responses
- More builtin functions
- Plugin system

## 🤝 Contributing

This is a learning project following the REPL-first development approach. See PROGRESS.md for current status and DESIGN.md for the complete specification.

## 📄 License

[Add your license here]

## 🙏 Acknowledgments

- **simplify_baml** - LLM integration framework
- **Ratatui** - Terminal UI framework
- **Pest** - Parser generator
- **DuckDB** - Embedded SQL database

---

**Built with ❤️ using Rust and Ratatui**
