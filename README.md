# Agentic LLM Workflow DSL

A powerful Domain-Specific Language (DSL) for building agentic LLM workflows with an interactive TUI (Terminal User Interface).

## 🎯 Overview

This project provides a complete DSL for orchestrating AI-powered workflows, featuring:

- **Interactive REPL** - Test expressions and explore the language
- **Full-featured Editor** - Write multi-line workflows with syntax highlighting
- **Live Preview** - Watch your workflows execute step-by-step
- **Type System** - Define custom types for structured data
- **LLM Integration** - Use AI models via simplify_baml
- **SQL/DuckDB Support** - Process data with SQL queries
- **Parallel Execution** - Run operations concurrently

## 🚀 Quick Start

### Prerequisites

- Rust (latest stable version)
- Optional: OpenAI API key for LLM features

### Installation & Running

```bash
# Clone the repository
cd DSL

# Build the entire workspace
cargo build --workspace

# Or build just the TUI app (faster)
cargo build -p dsl-repl

# Run the DSL TUI
cargo run --bin dsl

# Or run in release mode for better performance
cargo run --release --bin dsl
```

### Building Individual Crates

```bash
# Build only the core engine (no UI)
cargo build -p dsl-core

# Build only the TUI library
cargo build -p dsl-tui

# Build the main application
cargo build -p dsl-repl
```

### Quick Build & Run Guide

```bash
# Development build (faster compilation, slower runtime)
cargo run --bin dsl

# Release build (slower compilation, faster runtime)
cargo run --release --bin dsl

# Build without running
cargo build --workspace

# Run tests (when available)
cargo test --workspace

# Check all crates compile
cargo check --workspace
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

### Sequential Composition (`>>`)

Chain operations together:

```javascript
// Simple chaining
"hello" >> Upper(_) >> Length(_)

// With binding
[1,2,3,4,5] >> Length(_) as count >> count * 2
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

## 🏗️ Architecture

The project is organized into three separate crates for modularity and reusability:

### **dsl-core** (Library)
Core language engine with no UI dependencies. Can be used in:
- CLI tools
- Language servers (LSP)
- Web backends
- Testing frameworks

**Modules:**
- `parser/` - Pest-based parser with AST definitions
- `eval/` - Expression evaluator and built-in functions
- `types/` - Type system and value representation

### **dsl-tui** (Library)
Terminal user interface built with Ratatui. Provides:
- Interactive REPL mode
- Multi-line editor with syntax highlighting
- Type explorer
- Workflow preview

**Modules:**
- `app.rs` - Application state management
- `ui/` - Rendering, highlighting, and UI components
- `editor.rs` - Text editor integration

### **dsl-repl** (Binary)
Thin application wrapper that combines `dsl-core` + `dsl-tui`. Just 5 lines of code!

## 📁 Project Structure

```
DSL/
├── Cargo.toml                 # Workspace configuration
├── crates/
│   ├── dsl-core/             # Core execution engine (library)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs        # Public API
│   │       ├── parser/       # AST, grammar, parsing
│   │       │   ├── mod.rs
│   │       │   └── grammar.pest
│   │       ├── eval/         # Evaluator, builtins, SQL
│   │       │   ├── mod.rs
│   │       │   ├── evaluator.rs
│   │       │   ├── builtin.rs
│   │       │   └── sql.rs
│   │       └── types/        # Value, TypeRegistry
│   │           ├── mod.rs
│   │           ├── value.rs
│   │           └── registry.rs
│   │
│   ├── dsl-tui/              # Terminal UI (library)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs        # TUI entry point
│   │       ├── app.rs        # Application state
│   │       ├── editor.rs     # Editor component
│   │       └── ui/           # UI rendering
│   │           ├── mod.rs
│   │           ├── render.rs
│   │           ├── banner.rs
│   │           ├── preview.rs
│   │           └── highlight.rs
│   │
│   └── dsl-repl/             # Main application (binary)
│       ├── Cargo.toml
│       └── src/
│           └── main.rs       # Entry point (5 lines!)
│
├── tree-sitter-dsl/          # Syntax highlighting grammar
├── examples/                 # Example workflows
│   ├── basic_workflow.dsl
│   ├── types_example.dsl
│   ├── sequential_workflow.dsl
│   └── parallel_workflow.dsl
├── DESIGN.md
├── REPL_FIRST_PLAN.md
├── PROGRESS.md
└── README.md
```

## 🔧 Development Status

**Current Status:** Phase 8 Complete ✅

- ✅ Phase 0: Minimal REPL
- ✅ Phase 1: Expression Evaluator
- ✅ Phase 2: Variable Binding
- ✅ Phase 3: LLM Integration
- ✅ Phase 4: Type System
- ✅ Phase 5: Sequential Composition
- ✅ Phase 6: SQL/DuckDB
- ✅ Phase 7: Parallel Execution
- ✅ Phase 8: Full Editor & Preview

**Features Implemented:**
- Interactive REPL with history
- Multi-mode TUI (REPL/Editor/Preview/Type Explorer)
- Syntax highlighting
- Type system (classes and enums)
- Built-in functions
- LLM integration via simplify_baml
- SQL queries with DuckDB
- Sequential composition (`>>`)
- Parallel execution (`||`)
- User-defined functions
- Template string interpolation
- Session save/load
- Workflow execution with step tracking

## 📖 Documentation

- **[BUILD.md](BUILD.md)** - Comprehensive build and compilation guide
- **[DESIGN.md](DESIGN.md)** - Complete language specification
- **[REPL_FIRST_PLAN.md](REPL_FIRST_PLAN.md)** - Incremental development plan
- **[PROGRESS.md](PROGRESS.md)** - Detailed progress tracking
- **[examples/README.md](examples/README.md)** - Example workflows guide

## 🧪 Examples

See the `examples/` directory for complete workflow examples:

1. **basic_workflow.dsl** - Variables, functions, basic operations
2. **types_example.dsl** - Custom type definitions
3. **sequential_workflow.dsl** - Chaining with `>>`
4. **parallel_workflow.dsl** - Concurrent execution with `||`

## 🔑 Environment Variables

```bash
# For LLM features
export OPENAI_API_KEY="sk-..."

# Run the DSL TUI
cargo run --bin dsl

# Or with release optimizations
cargo run --release --bin dsl
```

## 🧩 Using dsl-core as a Library

The core engine can be used independently in your own projects:

```rust
use dsl_core::{Evaluator, Value};

#[tokio::main]
async fn main() {
    let mut eval = Evaluator::new();

    // Execute DSL code
    let (value, binding) = eval.eval("42 * 2").await.unwrap();
    println!("Result: {}", value.display());  // "84"

    // With LLM (requires OPENAI_API_KEY)
    let (response, _) = eval.eval(r#"Ask("What is Rust?")"#).await.unwrap();
    println!("{}", response.display());
}
```

**Add to your Cargo.toml:**
```toml
[dependencies]
dsl-core = { path = "../DSL/crates/dsl-core" }
tokio = { version = "1.0", features = ["full"] }
```

## 🎨 Features Showcase

### Syntax Highlighting

The editor provides full syntax highlighting:
- **Keywords** (type, enum, def, as) - Magenta
- **Types** (String, Int, Float, Bool) - Blue
- **Operators** (>>, ||, ?:, ->) - Cyan
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

**Completed:**
- ✅ Multi-crate architecture (core/tui/repl separation)
- ✅ Reusable core library
- ✅ Clean API boundaries

**Next Features:**
- CLI tool (`dsl-cli`) for non-interactive use
- Language server (`dsl-lsp`) for IDE integration
- WASM compilation for web use
- File picker for Ctrl+O
- Auto-save functionality
- Configuration file (`~/.dsl-config.toml`)
- More comprehensive error reporting
- Additional example workflows
- Documentation site

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
