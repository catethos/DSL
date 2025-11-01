# Implementation Details

## Technical Architecture

### Component Overview

```
┌─────────────────────────────────────────┐
│         TUI Layer (Ratatui)             │
│  - app.rs: Application state            │
│  - ui.rs: Rendering                     │
│  - editor.rs: Multi-line editor         │
│  - preview.rs: Execution preview        │
│  - banner.rs: Welcome screen            │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│        Runtime Layer (Async)            │
│  - eval.rs: Expression evaluator        │
│  - parser.rs: Pest parser               │
│  - builtin.rs: Built-in functions       │
│  - types.rs: Type registry              │
│  - value.rs: Value representation       │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│      Integration Layer                  │
│  - simplify_baml: LLM integration       │
│  - duckdb: SQL execution                │
│  - reqwest: HTTP client                 │
└─────────────────────────────────────────┘
```

## Core Components

### 1. Parser (parser.rs)

**Technology:** Pest parser generator

**Grammar:** `grammar.pest`

**AST Nodes:**
- `Declaration` - Type/enum/function definitions
- `Expr` - Expressions (literal, function call, sequential, parallel)
- `TypeDecl` - Type definitions
- `EnumDecl` - Enum definitions
- `FunctionDef` - User-defined functions

**Key Functions:**
- `parse_input()` - Parse user input
- `parse_type_definition()` - Parse type declarations
- `parse_function_definition()` - Parse function definitions
- `parse_expr()` - Parse expressions

### 2. Evaluator (eval.rs)

**Technology:** Async Rust with Tokio

**State Management:**
- Variables stored in `HashMap<String, Value>`
- Types stored in `TypeRegistry`
- Functions stored in `HashMap<String, FunctionDef>`

**Key Functions:**
- `eval()` - Main async evaluation function
- `eval_sequential()` - Handle `>>` operator
- `eval_parallel()` - Handle `||` operator
- `call_builtin()` - Dispatch built-in functions
- `call_user_function()` - Execute user-defined functions

### 3. Type System (types.rs)

**Integration:** Uses simplify_baml's `IR` (Intermediate Representation)

**Components:**
- `TypeRegistry` - Stores registered types
- `Class` - Struct type definitions (from BAML)
- `Enum` - Enum type definitions (from BAML)
- `Field` - Field definitions with types

**Key Functions:**
- `register_type()` - Add new type
- `register_enum()` - Add new enum
- `rebuild_runtime()` - Rebuild BAML runtime with new types

### 4. Value System (value.rs)

**Value Enum:**
```rust
pub enum Value {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    List(Vec<Value>),
    Map(HashMap<String, Value>),
    Null,
    Table(Vec<HashMap<String, Value>>),
}
```

**Key Functions:**
- `type_name()` - Get type name string
- `display_value()` - Format for output
- `from_json()` - Convert from JSON
- Field/index access operations

### 5. Built-in Functions (builtin.rs)

**Built-in Functions:**
- String: `Upper`, `Lower`, `Length`, `Join`
- LLM: `Ask`
- Math: `Sum`, `Avg`, `Min`, `Max` (planned)

**Integration:**
- Uses `BamlRuntime` for LLM calls
- Async execution for all functions

### 6. SQL Executor (sql.rs)

**Technology:** DuckDB (in-memory database)

**Features:**
- In-memory tables
- Automatic table registration
- CSV file reading
- Template variable substitution

**Key Functions:**
- `execute()` - Run SQL query
- `register_table()` - Add table from data
- `substitute_variables()` - Replace `${var}` with values

### 7. TUI (app.rs, ui.rs)

**Framework:** Ratatui + Crossterm

**Modes:**
- `Mode::Repl` - Full REPL
- `Mode::Workspace` - Editor + REPL + Preview
- `Mode::TypeExplorer` - Type browser

**State:**
- Input/output buffers
- Editor lines and cursor position
- Preview execution steps
- Loading indicators

## Data Flow

### Expression Evaluation

```
User Input
    ↓
Pest Parser → AST
    ↓
Evaluator (eval.rs)
    ↓
Match AST node type:
  - Literal → Return value
  - FunctionCall → Call function
  - Sequential → eval left >> eval right
  - Parallel → tokio::join_all([eval1, eval2, ...])
  - Binding → Store in variables
    ↓
Result (Value)
```

### LLM Function Call

```
User calls function with LLM prompt
    ↓
Evaluator: call_user_function()
    ↓
Check FunctionExecution type
    ↓
execute_llm_function()
    ↓
Build prompt with template
    ↓
Generate schema from return type
    ↓
Call simplify_baml
    ↓
Parse response with BAML
    ↓
Return Value
```

### HTTP Function Call

```
User calls function with HTTP config
    ↓
Evaluator: call_user_function()
    ↓
Check FunctionExecution type
    ↓
execute_http_function()
    ↓
Build request (URL, headers, body)
    ↓
Template interpolation
    ↓
reqwest sends HTTP request
    ↓
Parse JSON response
    ↓
Convert to Value
    ↓
Return Value
```

## Performance Characteristics

### Parsing
- **Fast:** Pest parser is highly optimized
- **Lazy:** Parses only what's needed
- **Error Recovery:** Good error messages

### Execution
- **Async:** All I/O is non-blocking
- **Parallel:** True concurrent execution with Tokio
- **Efficient:** Minimal allocations

### Memory
- **Variables:** Stored in HashMap
- **Types:** Cached in memory
- **SQL:** In-memory database

### Bottlenecks
- **LLM Calls:** Network + API latency (1-10s)
- **HTTP Requests:** Network latency (0.1-5s)
- **Large CSV Files:** Memory constraints

## Technology Stack

### Core
- **Rust** - Systems programming language
- **Tokio** - Async runtime
- **Pest** - Parser generator

### UI
- **Ratatui** - Terminal UI framework
- **Crossterm** - Terminal manipulation

### Integration
- **simplify_baml** - LLM structured outputs
- **duckdb** - Embedded SQL database
- **reqwest** - HTTP client

### Utilities
- **serde** - Serialization
- **anyhow** - Error handling

## Build System

### Cargo Workspace

```toml
[workspace]
members = ["crates/dsl-repl"]
```

### Dependencies

Main dependencies in `crates/dsl-repl/Cargo.toml`:
- ratatui
- crossterm
- pest / pest_derive
- tokio
- simplify_baml
- duckdb
- reqwest
- serde / serde_json
- anyhow

### Build Commands

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run
cargo run --bin dsl

# Run release
cargo run --release --bin dsl
```

## Development Approach

### REPL-First Development

The project was built incrementally:

1. **Phase 0:** Minimal REPL
2. **Phase 1:** Expression evaluator
3. **Phase 2:** Variable binding
4. **Phase 3:** LLM integration
5. **Phase 4:** Type system
6. **Phase 5:** Sequential operator (`>>`)
7. **Phase 6:** SQL/DuckDB
8. **Phase 7:** Parallel operator (`||`)
9. **Phase 8:** Full TUI

Each phase was fully tested before proceeding.

### Testing Strategy

- Manual REPL testing at each phase
- Example files for regression testing
- Parser unit tests
- Integration tests for workflows

## File Structure

```
DSL/
├── Cargo.toml                 # Workspace config
├── crates/
│   └── dsl-repl/
│       ├── Cargo.toml         # Dependencies
│       ├── grammar.pest       # Parser grammar
│       └── src/
│           ├── main.rs        # Entry point
│           ├── app.rs         # Application state
│           ├── ui.rs          # UI rendering
│           ├── editor.rs      # Editor component
│           ├── preview.rs     # Preview component
│           ├── banner.rs      # Banner display
│           ├── eval.rs        # Expression evaluator
│           ├── parser.rs      # Pest parser
│           ├── builtin.rs     # Built-in functions
│           ├── types.rs       # Type system
│           ├── value.rs       # Value representation
│           └── sql.rs         # SQL executor
├── examples/                  # Example workflows
└── docs/                      # Documentation
```

## Code Metrics

Approximate lines of code:
- `eval.rs`: ~800 lines
- `parser.rs`: ~600 lines
- `ui.rs`: ~500 lines
- `app.rs`: ~400 lines
- `types.rs`: ~300 lines
- `builtin.rs`: ~250 lines
- `value.rs`: ~200 lines
- `sql.rs`: ~150 lines
- `editor.rs`: ~180 lines
- `preview.rs`: ~150 lines

Total: ~3,500 lines of Rust code

## Future Enhancements

### Planned Features
- Conditional operator (`?:`)
- Error handling (`try/catch`)
- Loops (`for`, `while`)
- More LLM providers (Anthropic, etc.)
- Streaming LLM responses
- File upload/download
- WebSocket support

### Performance Improvements
- Caching frequently-used results
- Query optimization
- Memory management for large datasets
- Parallel SQL execution

### UI Enhancements
- File picker (Ctrl+O)
- Auto-save
- Configuration file
- Syntax error highlighting
- Debugger with breakpoints

## Contributing

### Development Setup

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone repository
cd /Users/catethos/workspace/DSL

# Build
cargo build

# Run
cargo run --bin dsl
```

### Code Style

- Follow Rust naming conventions
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting

## Related Documents

- **[DESIGN.md](../DESIGN.md)** - Complete design specification
- **[PROGRESS.md](../PROGRESS.md)** - Development progress
- **[REPL_FIRST_PLAN.md](../REPL_FIRST_PLAN.md)** - Incremental development plan
- **[REFACTORING_SUMMARY.md](../REFACTORING_SUMMARY.md)** - Architecture refactoring
