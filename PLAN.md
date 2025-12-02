# Lattice Language Implementation Plan

## Overview
Lattice is a statically-typed language designed for structured LLM interactions with first-class support for data manipulation via DuckDB.

## Language Design Summary

### Syntax Examples

```lattice
// Type definitions
type Person {
  name: String
  age: Int
  tags: [String]
}

enum Sentiment {
  Positive
  Negative
  Neutral
}

type Analysis {
  summary: String
  sentiment: Sentiment
  key_points: [String]
}

// LLM function definition (syntactic sugar - expands to normal function)
def AnalyzeText(text: String) -> Result<Analysis, LLMError> {
  base_url: "https://openrouter.ai/api/v1"
  model: "anthropic/claude-3.5-sonnet"
  api_key_env: "OPENROUTER_API_KEY"
  temperature: 0.7
  prompt: """
    Analyze this text: ${text}
  """
}

// The above desugars to something like:
// def AnalyzeText(text: String) -> Result<Analysis, LLMError> {
//   __llm_call__(LLMConfig {
//     base_url: "https://openrouter.ai/api/v1",
//     model: "anthropic/claude-3.5-sonnet",
//     ...
//   }, Analysis)
// }

// Regular function (same keyword)
def add(a: Int, b: Int) -> Int {
  a + b
}

// SQL integration
let data = SQL("SELECT * FROM users.csv WHERE age > 21")

// Parallel execution
let results = parallel {
  AnalyzeText("first document")
  AnalyzeText("second document")
}

// Parallel map
let analyses = parallel_map(documents, |doc| AnalyzeText(doc))

// Result handling
match AnalyzeText("hello") {
  Ok(analysis) => print(analysis.summary)
  Err(e) => print("Error: " + e.message)
}
```

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│              Tauri App (Jupyter-like UI)                │
│         [Cell 1] [Cell 2] [Cell 3] ... [Output]         │
├─────────────────────────────────────────────────────────┤
│                    Tauri Commands                       │
│     (eval_cell, get_state, reset, save, load, etc)     │
├─────────────────────────────────────────────────────────┤
│                   VM (Bytecode Interpreter)             │
│              (maintains state across cells)             │
├──────────┬──────────┬──────────┬───────────────────────┤
│  Parser  │Type Check│ Compiler │   Runtime Services    │
│  (pest)  │          │ (to BC)  │                       │
├──────────┴──────────┴──────────┼───────────────────────┤
│       AST / Bytecode IR        │  LLM Client │ DuckDB  │
├────────────────────────────────┴─────────────┴─────────┤
│                    Core Types / Values                  │
├─────────────────────────────────────────────────────────┤
│           FFI Layer (PyO3 / Rustler / Neon)            │
└─────────────────────────────────────────────────────────┘
```

### Compilation Model (Python-style)
- Source code → AST → Bytecode
- VM maintains persistent state (globals, functions, types)
- Each cell compiles to bytecode, appends to VM state
- Incremental: new cells can reference definitions from previous cells

### Tauri App Features
- **Cells**: Code cells with syntax highlighting (Monaco editor)
- **Output**: Rich output display (text, tables for SQL results, structured data)
- **State inspector**: View current variables, types, functions
- **Cell operations**: Run, run all, run above, clear output
- **Notebook files**: Save/load `.lat.nb` (JSON format with cells + outputs)
- **Autocomplete**: Type-aware completion from VM state

### Tauri Commands (Rust -> JS Bridge)
```rust
#[tauri::command]
async fn eval_cell(state: State<VM>, code: String) -> Result<CellOutput, String>;

#[tauri::command]
fn get_state(state: State<VM>) -> VMStateSnapshot;  // variables, types, functions

#[tauri::command]
fn reset_vm(state: State<VM>);

#[tauri::command]
async fn save_notebook(path: String, notebook: Notebook) -> Result<(), String>;

#[tauri::command]
async fn load_notebook(path: String) -> Result<Notebook, String>;

#[tauri::command]
fn get_completions(state: State<VM>, prefix: String) -> Vec<Completion>;
```

### CellOutput Types
```rust
enum CellOutput {
    Text(String),
    Table { columns: Vec<String>, rows: Vec<Vec<Value>> },  // SQL results
    Struct { type_name: String, fields: HashMap<String, Value> },
    Error { message: String, span: Option<Span> },
    None,  // for statements that don't produce output
}
```

---

## Phase 1: Project Setup & Type System Foundation {#phase-1}

> **Epic**: `DSL-rewrite-e9s`

Build the language core (VM, parser, compiler) as a library, then wrap with Tauri.

### Project Structure
```
lattice/
├── Cargo.toml                 # Workspace root
├── crates/
│   └── lattice/               # Single unified crate (subsumes simplify_baml)
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs         # Library root
│           │
│           │── # Core types (from simplify_baml, now native to Lattice)
│           ├── types/
│           │   ├── mod.rs     # Re-exports
│           │   ├── ir.rs      # Class, Enum, Field, FieldType, BamlValue
│           │   └── checker.rs # Type checking
│           │
│           │── # LLM integration (from simplify_baml)
│           ├── llm/
│           │   ├── mod.rs
│           │   ├── schema.rs      # Schema formatter for prompts
│           │   ├── parser.rs      # Lenient JSON parser with coercion
│           │   ├── partial_parser.rs  # Streaming partial JSON
│           │   ├── client.rs      # LLM HTTP client
│           │   ├── runtime.rs     # BamlRuntime orchestration
│           │   ├── renderer.rs    # Template rendering (minijinja)
│           │   └── streaming.rs   # StreamingBamlValue
│           │
│           │── # Language implementation
│           ├── syntax/
│           │   ├── mod.rs
│           │   ├── lexer.rs       # Token definitions
│           │   ├── grammar.pest   # Pest grammar
│           │   └── ast.rs         # AST node definitions
│           │
│           ├── compiler/
│           │   ├── mod.rs         # AST -> Bytecode
│           │   └── bytecode.rs    # OpCode definitions
│           │
│           ├── vm/
│           │   ├── mod.rs         # Virtual machine
│           │   └── state.rs       # VM state (persists across cells)
│           │
│           ├── sql/
│           │   ├── mod.rs         # DuckDB integration
│           │   └── convert.rs     # DuckDB <-> Lattice type conversion
│           │
│           ├── stdlib/
│           │   ├── mod.rs
│           │   ├── string.rs
│           │   ├── list.rs
│           │   ├── map.rs
│           │   └── math.rs
│           │
│           └── error.rs
│
└── apps/
    └── lattice-notebook/      # Tauri app
        ├── Cargo.toml
        ├── tauri.conf.json
        ├── src/
        │   ├── main.rs        # Tauri entry point
        │   └── commands.rs    # Tauri commands (eval_cell, etc)
        └── ui/                # Frontend (React)
            ├── package.json
            ├── src/
            │   ├── App.tsx
            │   ├── components/
            │   │   ├── Cell.tsx
            │   │   ├── Output.tsx
            │   │   ├── Toolbar.tsx
            │   │   └── StateInspector.tsx
            │   └── stores/
            │       └── notebook.ts
            └── index.html
```

### Key Insight: Unified Type System
- `simplify_baml`'s IR types (`Class`, `Enum`, `Field`, `FieldType`, `BamlValue`) become Lattice's native types
- No conversion layer needed - the VM operates directly on these types
- LLM integration is seamless since types are already in the right format

### Files to Copy from simplify_baml {#simplify-baml-files}

Source: `/Users/catethos/workspace/DSL/crates/simplify_baml/src/`

| Source File | Destination | Notes |
|-------------|-------------|-------|
| `ir.rs` | `types/ir.rs` | Core types - inline `dsl-types` |
| `schema.rs` | `llm/schema.rs` | Schema formatter |
| `parser.rs` | `llm/parser.rs` | Lenient JSON parser |
| `partial_parser.rs` | `llm/partial_parser.rs` | Streaming support |
| `client.rs` | `llm/client.rs` | LLM HTTP client |
| `runtime.rs` | `llm/runtime.rs` | BamlRuntime |
| `renderer.rs` | `llm/renderer.rs` | Template rendering |
| `streaming_value.rs` | `llm/streaming.rs` | StreamingBamlValue |
| `registry.rs` | (skip) | Not needed - Lattice manages IR directly |

---

## Phase 2: Bytecode VM Implementation {#phase-2}

> **Epic**: `DSL-rewrite-1k0`

### Bytecode Instructions
```rust
enum OpCode {
    // Stack operations
    Const(usize),        // Push constant from pool
    Pop,                 // Pop top of stack

    // Variables
    GetLocal(usize),     // Get local variable
    SetLocal(usize),     // Set local variable
    GetGlobal(String),   // Get global variable
    SetGlobal(String),   // Set global variable

    // Arithmetic
    Add, Sub, Mul, Div, Mod, Neg,

    // Comparison
    Eq, Ne, Lt, Le, Gt, Ge,

    // Logic
    Not, And, Or,

    // Control flow
    Jump(usize),         // Unconditional jump
    JumpIfFalse(usize),  // Conditional jump

    // Functions
    Call(usize),         // Call function with N args
    Return,              // Return from function

    // Collections
    MakeList(usize),     // Create list from N stack items
    MakeMap(usize),      // Create map from N key-value pairs
    Index,               // list[index] or map[key]

    // Objects
    GetField(String),    // Get struct field
    SetField(String),    // Set struct field
}
```

### VM Structure
- Stack-based execution
- Constant pool for literals
- Call frames for function calls
- Global environment for top-level bindings
- IR registry for type definitions

---

## Phase 3: Language Syntax & Parser {#phase-3}

> **Epic**: `DSL-rewrite-684`

### Grammar Overview (Pest)
- Expressions: literals, binary ops, unary ops, function calls, field access, indexing
- Statements: let bindings, assignments, if/else, match, while, for
- Definitions: `type`, `enum`, `def`
- Special forms: `SQL()`, `parallel {}`, `parallel_map()`

### AST Node Types
- `Expr`: Expression nodes
- `Stmt`: Statement nodes
- `TypeDef`: Type definitions (struct, enum)
- `FunctionDef`: Function definitions (regular and LLM)
- `Pattern`: Match patterns

---

## Phase 4: Compiler (AST to Bytecode) {#phase-4}

> **Epic**: `DSL-rewrite-fpv`

### Compilation Passes
1. **Parse**: Source → AST
2. **Type Check**: Validate types, infer where needed
3. **Compile**: AST → Bytecode
4. **Register**: Add types/functions to IR

---

## Phase 5: Tauri Notebook App {#phase-5}

> **Epic**: `DSL-rewrite-rva`

### Components
- **Cell.tsx**: Monaco editor with Lattice syntax highlighting
- **Output.tsx**: Render different output types (text, table, struct, error)
- **Toolbar.tsx**: Run, run all, clear, save, load buttons
- **StateInspector.tsx**: Show variables, types, functions

### State Management (Zustand)
```typescript
interface NotebookState {
  cells: Cell[];
  outputs: Map<string, CellOutput>;
  vmState: VMStateSnapshot;
  addCell: () => void;
  deleteCell: (id: string) => void;
  runCell: (id: string) => Promise<void>;
  runAll: () => Promise<void>;
}
```

---

## Phase 6: LLM Function Integration {#phase-6}

> **Epic**: `DSL-rewrite-lnh`

### LLM Function Syntax
```lattice
def AnalyzeText(text: String) -> Result<Analysis, LLMError> {
  base_url: "https://openrouter.ai/api/v1"
  model: "anthropic/claude-3.5-sonnet"
  api_key_env: "OPENROUTER_API_KEY"
  temperature: 0.7
  prompt: """
    Analyze this text: ${text}
  """
}
```

### Implementation
1. Parser detects config-style body (key: value pairs with `prompt:`)
2. Compiler generates `IR::Function` with prompt template
3. Runtime creates `LLMClient`, renders prompt, calls API
4. Response parsed using `llm/parser.rs` with type coercion

---

## Phase 7: DuckDB SQL Integration {#phase-7}

> **Epic**: `DSL-rewrite-4ru`

### Type Mapping
| DuckDB Type | Lattice Type |
|-------------|--------------|
| INTEGER     | Int          |
| BIGINT      | Int          |
| DOUBLE      | Float        |
| VARCHAR     | String       |
| BOOLEAN     | Bool         |
| DATE        | Date         |
| TIMESTAMP   | DateTime     |
| ARRAY       | [T]          |
| STRUCT      | type { }     |

### SQL Forms
- `SQL("SELECT * FROM file.csv")` - returns `List<Map<String, Value>>`
- `SQL<Person>("SELECT name, age FROM people.csv")` - typed result

---

## Phase 8: Standard Library {#phase-8}

> **Epic**: `DSL-rewrite-vwt`

### String Functions
`len`, `split`, `join`, `trim`, `upper`, `lower`, `contains`, `starts_with`, `ends_with`, `replace`, `substring`

### List Functions
`len`, `push`, `pop`, `get`, `set`, `map`, `filter`, `reduce`, `find`, `sort`, `reverse`, `flatten`

### Map Functions
`keys`, `values`, `get`, `set`, `has`, `merge`, `remove`

### Math Functions
`abs`, `min`, `max`, `floor`, `ceil`, `round`, `sqrt`, `pow`

---

## Phase 9: CLI REPL & Runner Tool {#phase-9}

> **Epic**: `DSL-rewrite-mc5`

A command-line interface for testing and debugging Lattice programs before the Tauri notebook app is ready.

### CLI Commands

```bash
# Run a file
lat run script.lat

# Interactive REPL
lat repl

# Evaluate expression
lat eval "1 + 2"

# Check syntax without executing
lat check script.lat

# Debug commands
lat dump-ast script.lat
lat dump-bytecode script.lat
```

### Crate Structure
```
crates/lattice-cli/
├── Cargo.toml
└── src/
    ├── main.rs        # Entry point, clap CLI definition
    ├── commands/
    │   ├── mod.rs
    │   ├── run.rs     # File execution
    │   ├── repl.rs    # Interactive REPL
    │   ├── eval.rs    # Single expression eval
    │   ├── check.rs   # Syntax/type checking
    │   └── dump.rs    # AST/bytecode inspection
    └── output.rs      # Result formatting (pretty, json, table)
```

### Dependencies
```toml
[dependencies]
lattice = { path = "../lattice" }
clap = { version = "4", features = ["derive"] }
rustyline = "14"
colored = "2"
```

### REPL Features
- Persistent VM state across inputs
- Multiline input support (detect incomplete expressions)
- Command history (saved to ~/.lat_history)
- Special commands: `:help`, `:vars`, `:types`, `:reset`, `:quit`
- Syntax error reporting with line/column

### Output Formats
- `--format=pretty` (default): Human-readable colored output
- `--format=json`: Structured JSON for tooling integration
- `--format=table`: Tabular output for SQL query results

---

## Phase 10: FFI Bindings {#phase-10}

> **Epic**: `DSL-rewrite-egq`

### Python API (PyO3)
```python
import lattice

result = lattice.eval("1 + 2")
program = lattice.load("analysis.lat")
result = program.call("AnalyzeText", text="Hello")
```

### TypeScript API (Neon)
```typescript
import { Lattice } from 'lattice';

const lat = new Lattice();
const result = await lat.eval('1 + 2');
```

### Elixir API (Rustler)
```elixir
{:ok, result} = Lattice.eval("1 + 2")
{:ok, analysis} = Lattice.call("AnalyzeText", text: "Hello")
```

---

## Rust Crate Dependencies

### crates/lattice/Cargo.toml
```toml
[package]
name = "lattice"
version = "0.1.0"
edition = "2021"

[dependencies]
# Parsing
pest = "2.7"
pest_derive = "2.7"

# Async runtime
tokio = { version = "1", features = ["full"] }

# HTTP client for LLM
reqwest = { version = "0.12", features = ["json"] }

# JSON handling
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Template rendering (for LLM prompts)
minijinja = "2.0"

# DuckDB
duckdb = "1.0"

# Error handling
thiserror = "1"
anyhow = "1"

# FFI (feature-gated)
[features]
python = ["pyo3"]
elixir = ["rustler"]
node = ["neon"]

[dependencies.pyo3]
version = "0.22"
optional = true

[dependencies.rustler]
version = "0.34"
optional = true

[dependencies.neon]
version = "1"
optional = true
```

### apps/lattice-notebook/Cargo.toml
```toml
[package]
name = "lattice-notebook"
version = "0.1.0"
edition = "2021"

[dependencies]
lattice = { path = "../../crates/lattice" }
tauri = { version = "2", features = ["devtools"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
```

### apps/lattice-notebook/ui/package.json
```json
{
  "dependencies": {
    "@tauri-apps/api": "^2",
    "react": "^18",
    "react-dom": "^18",
    "@monaco-editor/react": "^4",
    "zustand": "^4"
  },
  "devDependencies": {
    "@types/react": "^18",
    "@types/react-dom": "^18",
    "typescript": "^5",
    "vite": "^5",
    "@vitejs/plugin-react": "^4"
  }
}
```

---

## File Extensions
- `.lat` - Lattice source files
- `.lat.nb` - Lattice notebook files (JSON with cells + outputs)

---

## Issue Tracking

All tasks are tracked in **beads** (`bd`). Run `bd ready` to see unblocked work.

### Epics
| ID | Epic | Priority | Plan Section |
|----|------|----------|--------------|
| DSL-rewrite-e9s | Project Setup & Type System Foundation | P1 | [Phase 1](#phase-1) |
| DSL-rewrite-1k0 | Bytecode VM Implementation | P1 | [Phase 2](#phase-2) |
| DSL-rewrite-684 | Language Syntax & Parser | P1 | [Phase 3](#phase-3) |
| DSL-rewrite-fpv | Compiler (AST to Bytecode) | P1 | [Phase 4](#phase-4) |
| DSL-rewrite-rva | Tauri Notebook App | P1 | [Phase 5](#phase-5) |
| DSL-rewrite-lnh | LLM Function Integration | P2 | [Phase 6](#phase-6) |
| DSL-rewrite-4ru | DuckDB SQL Integration | P2 | [Phase 7](#phase-7) |
| DSL-rewrite-vwt | Standard Library | P3 | [Phase 8](#phase-8) |
| DSL-rewrite-mc5 | CLI REPL & Runner Tool | P1 | [Phase 9](#phase-9) |
| DSL-rewrite-egq | FFI Bindings | P3 | [Phase 10](#phase-10) |
