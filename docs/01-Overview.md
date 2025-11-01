# DSL TUI - Overview and Architecture

## What is This Project?

The **Agentic LLM Workflow DSL** is a powerful domain-specific language (DSL) for building AI-powered workflows with an interactive Terminal User Interface (TUI). It combines the simplicity of a scripting language with the power of Large Language Models (LLMs), SQL databases, and HTTP APIs.

## Key Features

### 🎯 Core Capabilities

- **Interactive REPL** - Test expressions and explore the language interactively
- **Full-featured Editor** - Write multi-line workflows with syntax highlighting
- **Live Preview** - Watch your workflows execute step-by-step
- **Multi-Mode Interface** - Switch between REPL, Workspace, and Type Explorer modes
- **Type System** - Define custom types and enums for structured data
- **LLM Integration** - Use AI models via simplify_baml with structured outputs
- **SQL/DuckDB Support** - Process data with SQL queries on CSV files
- **HTTP Client** - Make API requests with full REST support
- **Parallel Execution** - Run operations concurrently with the `||` operator
- **Sequential Composition** - Chain operations with the `>>` operator

### 🚀 Why This DSL?

**Traditional Approach:**
```python
# Verbose, lots of boilerplate
import openai
import duckdb
import requests

response = openai.ChatCompletion.create(...)
data = duckdb.execute("SELECT ...").fetchall()
api_result = requests.get(url).json()
# More glue code...
```

**DSL Approach:**
```javascript
// Concise, expressive, composable
Search(topic) as results
  >> SQL("SELECT * FROM results WHERE score > 0.8") as filtered
  >> Analyze(filtered) as insights
```

## Project Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     TUI Layer (Ratatui)                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │    REPL     │  │   Editor    │  │   Preview   │         │
│  │    Mode     │  │     +       │  │     +       │         │
│  │             │  │    REPL     │  │   Types     │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│                    DSL Runtime Layer                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   Parser     │  │  Evaluator   │  │ Type System  │      │
│  │   (Pest)     │  │   (Async)    │  │  (Registry)  │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│               Integration Layer (Functions)                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │     LLM      │  │     SQL      │  │     HTTP     │      │
│  │ (simplify_   │  │  (DuckDB)    │  │  (reqwest)   │      │
│  │   baml)      │  │              │  │              │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
```

### Component Breakdown

#### 1. **TUI Layer** (`crates/dsl-repl/src/`)
- **app.rs** - Application state management, mode handling
- **ui.rs** - UI rendering for all modes
- **editor.rs** - Multi-line editor with syntax highlighting
- **preview.rs** - Execution preview and type explorer
- **banner.rs** - Welcome banner with ASCII art

#### 2. **Parser** (`crates/dsl-repl/src/parser.rs`)
- **Grammar-based parsing** using Pest parser
- **AST construction** for all language constructs
- **Type definitions** parsing (classes, enums)
- **Function definitions** parsing (LLM, HTTP, SQL)
- **Expression parsing** with operator precedence

#### 3. **Evaluator** (`crates/dsl-repl/src/eval.rs`)
- **Async execution** with Tokio runtime
- **Variable binding** and scope management
- **Expression evaluation** (literals, operators, functions)
- **Workflow orchestration** (sequential, parallel, conditional)
- **Built-in functions** dispatcher

#### 4. **Type System** (`crates/dsl-repl/src/types.rs`)
- **TypeRegistry** - Stores custom types and enums
- **Integration with BAML** - Uses simplify_baml's IR
- **Type validation** - Ensures type safety
- **Schema generation** - For LLM structured outputs

#### 5. **Built-in Functions** (`crates/dsl-repl/src/builtin.rs`)
- **String operations** - Upper, Lower, Length, Join, etc.
- **LLM calls** - Ask() for unstructured queries
- **Math operations** - Sum, Avg, Min, Max
- **Data conversion** - Type conversion utilities

#### 6. **SQL Integration** (`crates/dsl-repl/src/sql.rs`)
- **DuckDB in-memory database**
- **CSV file reading** with native DuckDB support
- **Table management** - Create, query, join tables
- **SQL execution** - Full SQL support with error handling

#### 7. **Value System** (`crates/dsl-repl/src/value.rs`)
- **Value enum** - String, Int, Float, Bool, List, Map, Null, Table
- **Type display** - User-friendly type names
- **JSON conversion** - Bidirectional JSON support
- **Display formatting** - Pretty-printing values

## Design Principles

### 1. **Uniformity**
All operations are functions with consistent syntax. Whether it's an LLM call, SQL query, or HTTP request, the syntax is the same.

### 2. **Composability**
Functions compose using mathematical operators:
- `>>` - Sequential composition (pipeline)
- `||` - Parallel composition (concurrent)
- `?:` - Conditional composition (branching)

### 3. **Type Safety**
Strong typing with compile-time checking and runtime validation using BAML for structured LLM outputs.

### 4. **Ergonomics**
- Concise `as` syntax for variable binding
- Implicit `_` for last result
- Template string interpolation with `${variable}`
- Multi-line input with smart delimiter detection

### 5. **Separation of Concerns**
- **simplify_baml** handles: Type system, schema generation, LLM parsing
- **Our DSL** handles: Workflow orchestration, control flow, TUI
- **DuckDB** handles: SQL execution, data processing
- **reqwest** handles: HTTP requests, API integration

## Development Philosophy: REPL-First

This project was built using a **REPL-first incremental development** approach:

1. **Phase 0** - Minimal REPL (basic input/output)
2. **Phase 1** - Expression evaluator (literals, arithmetic)
3. **Phase 2** - Variable binding (state management)
4. **Phase 3** - LLM integration (first external API)
5. **Phase 4** - Type system (custom types and enums)
6. **Phase 5** - Sequential composition (`>>` operator)
7. **Phase 6** - SQL/DuckDB integration
8. **Phase 7** - Parallel execution (`||` operator)
9. **Phase 8** - Full editor and live preview

Each phase was fully tested before moving to the next, ensuring a solid foundation.

## Technology Stack

### Core Technologies
- **Rust** - Systems programming language for performance and safety
- **Ratatui** - Terminal UI framework for the TUI
- **Crossterm** - Cross-platform terminal manipulation
- **Pest** - Parser generator for grammar-based parsing
- **Tokio** - Async runtime for concurrent execution

### Integration Libraries
- **simplify_baml** - LLM integration with structured outputs
- **duckdb** - Embedded SQL database
- **reqwest** - HTTP client for API requests
- **serde** - Serialization/deserialization
- **anyhow** - Error handling

### Development Tools
- **Cargo** - Rust package manager and build system
- **Git** - Version control

## Project Structure

```
DSL/
├── Cargo.toml                    # Workspace configuration
├── README.md                     # User-facing documentation
├── DESIGN.md                     # Complete design specification
├── PROGRESS.md                   # Development progress tracker
├── docs/                         # Detailed documentation (this folder)
│   ├── README.md                 # Documentation index
│   ├── 01-Overview.md            # This file
│   ├── 02-Getting-Started.md     # Quick start guide
│   └── ...                       # More documentation files
├── crates/
│   └── dsl-repl/                 # Main TUI application
│       ├── Cargo.toml            # Dependencies
│       ├── src/
│       │   ├── main.rs           # Entry point
│       │   ├── app.rs            # Application state
│       │   ├── ui.rs             # UI rendering
│       │   ├── editor.rs         # Editor component
│       │   ├── preview.rs        # Preview component
│       │   ├── eval.rs           # Expression evaluator
│       │   ├── parser.rs         # Pest parser
│       │   ├── builtin.rs        # Built-in functions
│       │   ├── types.rs          # Type system
│       │   ├── value.rs          # Value representation
│       │   ├── sql.rs            # DuckDB integration
│       │   └── banner.rs         # Welcome banner
│       └── grammar.pest          # Pest grammar file
├── examples/                     # Example workflows
│   ├── README.md                 # Examples guide
│   ├── basic_workflow.dsl
│   ├── types_example.dsl
│   ├── sequential_workflow.dsl
│   └── parallel_workflow.dsl
└── tests/                        # Test files
```

## Performance Characteristics

### Strengths
- **Fast startup** - TUI launches in milliseconds
- **Efficient parsing** - Pest parser is highly optimized
- **Concurrent execution** - Parallel operator leverages Tokio
- **In-memory SQL** - DuckDB is extremely fast for data processing
- **Low latency** - Minimal overhead between DSL and execution

### Bottlenecks
- **LLM calls** - Network latency and API processing time
- **Large CSV files** - Loading multi-GB files into memory
- **HTTP requests** - Network latency for external APIs

### Optimizations
- **Async everywhere** - Non-blocking I/O for all external calls
- **Streaming support** - For LLM responses (future enhancement)
- **Smart caching** - Type definitions cached in memory
- **Efficient rendering** - Ratatui only redraws changed areas

## Use Cases

### 1. **AI-Powered Data Processing**
```javascript
// Load data, filter with SQL, analyze with LLM
SQL("SELECT * FROM 'data.csv' WHERE score > 0.8") as data
  >> Analyze(data) as insights
```

### 2. **API Orchestration**
```javascript
// Fetch from multiple APIs in parallel, combine results
(getWeather("NYC") || getNews("NYC") || getEvents("NYC"))
  >> Summarize(_) as report
```

### 3. **Content Generation**
```javascript
// Research topic, generate content, format output
Search(topic) as research
  >> GenerateOutline(research) as outline
  >> WriteArticle(outline) as article
```

### 4. **Data Transformation Pipelines**
```javascript
// Read CSV, transform, query, export
SQL("SELECT * FROM 'input.csv'") as raw
  >> CleanData(raw) as cleaned
  >> SQL("SELECT category, COUNT(*) FROM cleaned GROUP BY category")
```

## What's Next?

Continue to the next documentation files:
- **[02-Getting-Started.md](02-Getting-Started.md)** - Installation and quick start
- **[03-TUI-Interface.md](03-TUI-Interface.md)** - UI modes and navigation
- **[04-Language-Features.md](04-Language-Features.md)** - Core language syntax

## Related Documents

- **[DESIGN.md](../DESIGN.md)** - Complete design specification
- **[PROGRESS.md](../PROGRESS.md)** - Development progress tracker
- **[README.md](../README.md)** - Project README
- **[examples/README.md](../examples/README.md)** - Example workflows guide
