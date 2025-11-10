# Agentic LLM Workflow DSL

A powerful Domain-Specific Language (DSL) for building agentic LLM workflows with both **interactive REPL** and **native binary compilation**.

## 🎯 Overview

This project provides a complete DSL for orchestrating AI-powered workflows, featuring:

- **🔄 Dual Execution Modes**
  - **Interactive REPL** - Test expressions and explore the language interactively
  - **Binary Compiler** - Compile DSL to standalone native executables
- **📝 Full-featured Editor** - Write multi-line workflows with syntax highlighting
- **👁️ Live Preview** - Watch your workflows execute step-by-step
- **🔍 Execution Tracing** - Debug and profile with detailed trace recording
- **🏗️ Type System** - Define custom types for structured data
- **🤖 LLM Integration** - Use AI models via simplify_baml
- **🗄️ SQL/DuckDB Support** - Process data with SQL queries
- **⚡ Parallel Execution** - Run operations concurrently

## 🏛️ Architecture

The DSL uses a modern IR-based architecture with **clean separation between compilation and runtime**:

```
DSL Source (.dsl)
       ↓
    Parser (Pest)          ← dsl-core (lightweight)
       ↓
      AST
       ↓
  IR Compiler             ← dsl-core (no runtime deps)
       ↓
   IR (JSON/MessagePack)  ← dsl-ir (minimal deps)
       ↓
  Interpreter             ← dsl-interpreter (all runtime features)
       ↓
   Execution
```

**Key Components** (11 crates):
- **dsl-types** - Shared type definitions (Class, Enum, Field, FieldType)
- **dsl-core** - Parser, AST, IR compiler (compilation only)
- **dsl-ir** - Intermediate Representation (serializable, minimal deps)
- **dsl-interpreter** - Tree-walk interpreter for IR (all runtime features)
- **simplify_baml** - LLM integration framework (now in workspace)
- **simplify_baml_macros** - BAML derive macros
- **dsl-repl** - Interactive REPL with check/ir/run commands
- **dsl-tui** - Terminal UI components
- **dsl-egui** - Desktop GUI (experimental)
- **dsl-autocomplete** - Autocomplete engine for REPL
- **tree-sitter-dsl** - Syntax highlighting grammar

## 🚀 Quick Start

### Prerequisites

- Rust (latest stable version)
- Optional: OpenAI API key for LLM features

### Installation

```bash
# Clone the repository
cd DSL

# Build the entire workspace
cargo build --release --workspace

# Or build the REPL specifically
cargo build --release -p dsl-repl
```

### Usage: Interactive REPL

```bash
# Run the interactive TUI
cargo run --release --bin dsl

# Or use the built binary
./target/release/dsl
```

### Usage: Working with DSL Files

```bash
# Check a DSL file for errors
./target/release/dsl check program.dsl

# Compile DSL to IR (JSON format)
./target/release/dsl ir program.dsl -o program.ir.json --json

# Compile DSL to IR (MessagePack format)
./target/release/dsl ir program.dsl -o program.ir

# Run a DSL file directly
./target/release/dsl run program.dsl

# Run with execution tracing (for debugging/profiling)
./target/release/dsl run program.dsl --trace
./target/release/dsl run program.dsl --trace --trace-verbose
./target/release/dsl run program.dsl --trace --trace-output trace.json
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

The project uses a modern **IR-based compilation pipeline** with a clean separation between compilation and runtime:

### **Core Compilation Pipeline**

1. **dsl-types** - Shared Type Definitions
   - Lightweight type system shared across all crates
   - `Class`, `Enum`, `Field`, `FieldType` definitions
   - No runtime dependencies - just data structures
   - Dependencies: `serde` only

2. **dsl-core** - Parser & IR Compiler (Compilation Only)
   - Pest-based parser → AST
   - AST → IR compiler
   - Type system and validation
   - **No runtime dependencies** (duckdb, reqwest, etc. removed)
   - Clean separation: parsing/compilation only

3. **dsl-ir** - Intermediate Representation
   - Serializable IR (JSON/MessagePack)
   - Type-safe representation
   - Platform-independent
   - **Minimal dependencies** (no duckdb, uses dsl-types)

4. **dsl-interpreter** - Runtime Interpreter (Execution Only)
   - Tree-walk interpreter for IR
   - Builtin functions (LLM, SQL, HTTP, charts)
   - Runtime type registry
   - SQLExecutor for database operations
   - **All runtime dependencies** (duckdb, reqwest, plotters, etc.)

### **LLM Integration**

5. **simplify_baml** - Simplified BAML Runtime
   - LLM integration framework (now part of workspace)
   - Uses `dsl-types` for shared type definitions
   - Template rendering with Jinja2
   - Multi-provider support (OpenAI, Anthropic, OpenRouter)
   - Streaming and partial parsing

6. **simplify_baml_macros** - BAML Derive Macros
   - Code generation for BAML schemas
   - Compile-time type checking

### **User Interface**

7. **dsl-tui** - Terminal UI Library
   - Interactive REPL mode
   - Multi-line editor with syntax highlighting
   - Type explorer & preview
   - Image rendering support

8. **dsl-repl** - Binary Application
   - Main TUI application with CLI commands
   - `check` - Validate DSL programs
   - `ir` - Compile to IR (JSON/MessagePack)
   - `run` - Execute DSL files directly
   - Combines core + interpreter + UI

9. **dsl-egui** - Desktop GUI (Experimental)
   - Rich desktop interface using egui
   - Advanced rendering (images, tables, plots)
   - File picker and save/load

10. **dsl-autocomplete** - Autocomplete Engine
    - Context-aware suggestions
    - Keyword, function, and variable completion
    - Fuzzy matching support

### **Why IR-Based Architecture?**

✅ **Serialization**: Programs can be saved/loaded as IR
✅ **Language Independence**: IR enables multiple frontends
✅ **Optimization**: IR enables future optimization passes
✅ **Multiple Targets**: Easy to add WASM, other platforms
✅ **Debugging**: IR is human-readable (JSON format)
✅ **Consistency**: Same interpreter runs REPL and file execution
✅ **Clean Separation**: Compilation (dsl-core/dsl-ir) vs Runtime (dsl-interpreter)
✅ **Faster Builds**: Compilation crates are lightweight (~59% faster)
✅ **Shared Types**: Single source of truth via `dsl-types`

## 📁 Project Structure

```
DSL/
├── Cargo.toml                 # Workspace configuration (11 members)
├── crates/
│   ├── dsl-types/            # 🆕 Shared type definitions
│   │   └── src/lib.rs        # Class, Enum, Field, FieldType
│   │
│   ├── dsl-core/             # Parser & IR compiler (LIGHTWEIGHT)
│   │   ├── src/parser/       # Pest grammar, AST
│   │   ├── src/compiler.rs   # AST → IR compiler
│   │   └── src/resolver.rs   # Function grouping & resolution
│   │
│   ├── dsl-ir/               # Intermediate Representation (MINIMAL DEPS)
│   │   ├── src/ir.rs         # Serializable IR types
│   │   └── src/value.rs      # Runtime values
│   │
│   ├── dsl-interpreter/      # IR Interpreter (ALL RUNTIME DEPS)
│   │   ├── src/interpreter.rs  # Tree-walk interpreter
│   │   ├── src/builtins.rs     # Builtin functions
│   │   ├── src/runtime.rs      # Runtime type registry
│   │   ├── src/sql.rs          # 🆕 SQLExecutor (moved from dsl-ir)
│   │   └── src/tracing.rs      # Execution tracing
│   │
│   ├── simplify_baml/        # 🆕 BAML Runtime (now in workspace)
│   │   ├── src/ir.rs         # BAML IR (uses dsl-types)
│   │   ├── src/runtime.rs    # LLM runtime
│   │   ├── src/client.rs     # Multi-provider LLM client
│   │   └── src/renderer.rs   # Jinja2 template rendering
│   │
│   ├── simplify_baml_macros/ # 🆕 BAML Macros (now in workspace)
│   │   └── src/lib.rs        # Derive macros for BAML
│   │
│   ├── dsl-autocomplete/     # Autocomplete Engine
│   │   ├── src/engine.rs     # Matching engine
│   │   └── src/providers/    # Suggestion providers
│   │
│   ├── dsl-tui/              # Terminal UI (library)
│   │   ├── src/app.rs        # Application state
│   │   ├── src/ui/           # Rendering components
│   │   └── src/renderers/    # Output renderers (text, tables, images)
│   │
│   ├── dsl-repl/             # REPL Application (binary)
│   │   └── src/main.rs       # TUI + CLI commands (check/ir/run)
│   │
│   └── dsl-egui/             # Desktop GUI (experimental)
│       ├── src/app.rs        # egui application
│       └── src/renderers/    # Rich rendering (plots, images)
│
├── tree-sitter-dsl/          # Syntax highlighting
├── examples/                 # DSL programs
│   ├── greet.dsl
│   ├── text_processor.dsl
│   └── openrouter_multi_provider.dsl
├── docs/                     # Documentation
│   ├── 00-Documentation-Summary.md
│   ├── 04-Language-Features.md
│   └── 06-Builtin-Functions.md
└── README.md
```

### **Dependency Graph**

```
dsl-types (foundation)
    ↑
    ├─── dsl-ir
    │      ↑
    │      ├─── dsl-core
    │      │      ↑
    │      │      └─── dsl-interpreter
    │      │             ↑
    │      │             ├─── dsl-tui
    │      │             ├─── dsl-egui
    │      │             └─── dsl-repl
    │      │
    │      └─── dsl-autocomplete
    │
    └─── simplify_baml
           ↑
           └─── dsl-interpreter
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
- ✅ Phase 5: REPL using IR with check/ir/run commands
- ✅ Phase 6: dsl-autocomplete engine
- ✅ Phase 7: Advanced TUI features
- ✅ Phase 8: Pattern matching and function polymorphism
- ✅ Phase 9: Testing and validation
- ✅ Phase 9.5: Quality & Polish (bug fixes, optimization)

### **Features Implemented**
- ✅ Interactive REPL with history
- ✅ IR compilation (check/ir/run commands)
- ✅ Multi-mode TUI (REPL/Editor/Preview/Type Explorer)
- ✅ Syntax highlighting with tree-sitter
- ✅ Type system (classes and enums)
- ✅ 50+ builtin functions
- ✅ LLM integration via simplify_baml
- ✅ SQL queries with DuckDB
- ✅ Sequential composition (`>>`)
- ✅ Parallel execution (`par()`)
- ✅ Pattern matching with guards
- ✅ User-defined functions (LLM, HTTP, SQL, polymorphic)
- ✅ Template string interpolation
- ✅ Comparison operators (==, !=, <, >, <=, >=)
- ✅ Session save/load
- ✅ Workflow execution with step tracking
- ✅ Context-aware autocomplete
- ✅ Desktop GUI (dsl-egui) with rich rendering

## 📖 Documentation

### Quick Links

- **[Documentation Index](docs/README.md)** - Complete documentation guide
- **[Getting Started](docs/user-guide/02-Getting-Started.md)** - Start here!
- **[Language Features](docs/user-guide/04-Language-Features.md)** - Full language reference
- **[Developer Guide](docs/developer/architecture.md)** - System architecture

### Documentation Structure

- **[`docs/user-guide/`](docs/user-guide/)** - User documentation
  - Overview, getting started, language features, types, builtins, workflows, LLM, SQL, HTTP
- **[`docs/gui/`](docs/gui/)** - GUI documentation
  - [egui Desktop GUI](docs/gui/egui-desktop-gui.md) - Modern desktop interface
  - [TUI Interface](docs/gui/tui-interface.md) - Terminal UI
- **[`docs/advanced/`](docs/advanced/)** - Advanced topics
  - Advanced features, chart generation, tracing and debugging
- **[`docs/developer/`](docs/developer/)** - Developer documentation
  - [Architecture](docs/developer/architecture.md) - IR-based system design
  - [Error Handling](docs/developer/error-handling.md) - Error system internals
  - [Known Issues](docs/developer/known-issues.md) - Current limitations and fixes
  - [egui Implementation](docs/developer/egui-implementation.md) - GUI development guide
  - [Building](docs/developer/building.md) - Build and compilation guide
- **[`docs/references/`](docs/references/)** - Reference materials
  - DSPy comparison, operator changes
- **[`docs/migration-guides/`](docs/migration-guides/)** - Migration guides
  - [Autocomplete](docs/migration-guides/autocomplete.md) - Autocomplete system guide

### LLM Integration

For details on the simplify_baml LLM integration framework:
- **[`crates/simplify_baml/README.md`](crates/simplify_baml/README.md)** - Framework documentation
- Related docs in that directory

### Historical Documentation

- **[`historical/README.md`](historical/README.md)** - Development history and phase summaries

## 🧪 Examples

See the `examples/` directory for complete workflow examples:

1. **greet.dsl** - Simple greeting with string operations
   ```bash
   # Check for syntax errors
   ./target/release/dsl check examples/greet.dsl

   # Run directly
   ./target/release/dsl run examples/greet.dsl
   ```

2. **text_processor.dsl** - Text processor with conditionals
   ```bash
   # Compile to IR for inspection
   ./target/release/dsl ir examples/text_processor.dsl -o text.ir.json --json

   # Run directly
   ./target/release/dsl run examples/text_processor.dsl
   ```

3. **openrouter_multi_provider.dsl** - Multi-provider LLM access
   - Use Claude, GPT-4, and other models with separate API keys

4. **Sequential workflows** - Chaining with `>>`
5. **Parallel workflows** - Concurrent execution with `par()`

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

The DSL can be embedded in your Rust applications using the IR-based API. The architecture separates compilation from execution:

### **Simple Expression Evaluation**

```rust
use dsl_core::compile_to_ir;
use dsl_interpreter::Interpreter;
use dsl_ir::Value;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Compile DSL code to IR
    let ir_node = compile_to_ir("42 * 2")?;

    // Create interpreter and execute
    let mut interpreter = Interpreter::new()?;
    let result = interpreter.eval(&ir_node).await?;

    println!("Result: {}", result.display());  // "84"
    Ok(())
}
```

**Minimal dependencies (compilation + execution):**
```toml
[dependencies]
# Compilation (lightweight - no runtime deps)
dsl-core = { path = "../DSL/crates/dsl-core" }
dsl-ir = { path = "../DSL/crates/dsl-ir" }

# Execution (includes all runtime features)
dsl-interpreter = { path = "../DSL/crates/dsl-interpreter" }

# Async runtime
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
```

### **Compilation-Only Usage**

If you only need to parse and compile (e.g., for linting, validation, or code generation):

```rust
use dsl_core::compile_to_ir;
use std::fs;

fn main() -> anyhow::Result<()> {
    let source = fs::read_to_string("program.dsl")?;

    // Parse and compile to IR (no runtime dependencies needed!)
    let ir = compile_to_ir(&source)?;

    // Save IR for later execution
    let json = ir.to_json_pretty()?;
    fs::write("program.ir.json", json)?;

    println!("✓ Compiled successfully");
    Ok(())
}
```

**Lightweight dependencies (compilation only):**
```toml
[dependencies]
# Only compilation - very fast builds!
dsl-core = { path = "../DSL/crates/dsl-core" }
dsl-ir = { path = "../DSL/crates/dsl-ir" }
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
- ✅ Multi-crate architecture (6 crates)
- ✅ Reusable core library
- ✅ Interactive REPL with TUI
- ✅ CLI commands (check/ir/run)
- ✅ Comprehensive test suite
- ✅ All comparison operators
- ✅ Pattern matching and polymorphic functions
- ✅ Context-aware autocomplete
- ✅ Desktop GUI with egui

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

- **simplify_baml** - LLM integration framework (now integrated in workspace)
- **Ratatui** - Terminal UI framework
- **egui** - Immediate mode GUI framework
- **Pest** - Parser generator
- **DuckDB** - Embedded SQL database
- **genai** - Multi-provider LLM client
- **Tree-sitter** - Incremental parsing for syntax highlighting

---

**Built with ❤️ using Rust and Ratatui**
