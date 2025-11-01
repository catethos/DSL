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

### Installation

```bash
# Clone the repository
cd /Users/catethos/workspace/DSL

# Build the project
cargo build --release

# Run the DSL REPL
cargo run --bin dsl
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

## 📁 Project Structure

```
DSL/
├── crates/
│   └── dsl-repl/          # Main REPL/TUI application
│       ├── src/
│       │   ├── app.rs      # Application state
│       │   ├── editor.rs   # Editor component
│       │   ├── preview.rs  # Preview component
│       │   ├── ui.rs       # UI rendering
│       │   ├── eval.rs     # Expression evaluator
│       │   ├── parser.rs   # Pest-based parser
│       │   ├── builtin.rs  # Built-in functions
│       │   ├── types.rs    # Type system
│       │   ├── value.rs    # Value representation
│       │   └── sql.rs      # DuckDB integration
│       └── Cargo.toml
├── examples/               # Example workflows
│   ├── basic_workflow.dsl
│   ├── types_example.dsl
│   ├── sequential_workflow.dsl
│   ├── parallel_workflow.dsl
│   └── README.md
├── DESIGN.md              # Complete design specification
├── REPL_FIRST_PLAN.md     # Development roadmap
├── PROGRESS.md            # Development progress tracker
└── README.md              # This file
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

# Run the DSL
cargo run --bin dsl
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

**Next Features:**
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
