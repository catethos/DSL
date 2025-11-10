# DSL Architecture

**Last Updated:** 2025-11-10
**Status:** IR Migration Complete (Phases 1-11)
**Current Work:** Phase 10B Pattern Matching (60% complete)

---

## Table of Contents

- [Overview](#overview)
- [Architecture Principles](#architecture-principles)
- [Crate Structure](#crate-structure)
- [Compilation Pipeline](#compilation-pipeline)
- [IR Architecture](#ir-architecture)
- [Runtime Architecture](#runtime-architecture)
- [Phase History](#phase-history)
- [Design Decisions](#design-decisions)

---

## Overview

The DSL compiler uses a modern **Intermediate Representation (IR)** architecture that separates parsing, compilation, and execution. This enables both interpreted (REPL) and compiled (binary) execution modes with guaranteed identical behavior.

### Architecture at a Glance

```
┌─────────────┐
│ DSL Source  │  "let x = upper('hello')"
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Parser    │  Pest-based PEG parser
│  (grammar)  │  grammar.pest (363 lines)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│    AST      │  Abstract Syntax Tree
│  (13 expr)  │  Expr enum with 13 variants
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ IR Compiler │  AST → IR translation
│             │  Preserves all semantics
└──────┬──────┘
       │
       ▼
┌─────────────┐
│     IR      │  Intermediate Representation
│  (25 nodes) │  Serializable (JSON/MessagePack)
└──────┬──────┘
       │
       ├─────────────────────────┐
       ▼                         ▼
┌──────────────┐        ┌──────────────┐
│ Interpreter  │        │   Codegen    │
│ (Tree-walk)  │        │ (Embed IR)   │
└──────┬───────┘        └──────┬───────┘
       │                       │
       ▼                       ▼
┌──────────────┐        ┌──────────────┐
│  REPL/TUI    │        │ Rust Binary  │
│   (egui)     │        │  (Cargo)     │
└──────────────┘        └──────────────┘
```

---

## Architecture Principles

### 1. Single Source of Truth

**DSL grammar (`grammar.pest`) is the definitive language specification.**

All language semantics flow from the grammar. The parser is generated from grammar, not hand-written.

### 2. Separation of Concerns

```
Parsing    ─→  Syntax validation (grammar rules)
Compilation ─→  Semantic transformation (AST → IR)
Interpretation ─→  Runtime behavior (IR evaluation)
```

Each phase has clear responsibilities and boundaries.

### 3. Serializable IR

The IR is fully serializable to:
- **MessagePack** (binary, for efficient storage)
- **JSON** (text, for debugging and inspection)

This enables:
- Caching compiled programs
- Network transmission
- Debugging and introspection

### 4. Embedded Interpreter Approach

Compiled binaries **embed the IR as JSON** and use the interpreter at runtime, rather than generating complex Rust code. This ensures:
- ✅ Identical behavior between REPL and compiled binaries
- ✅ Simpler code generation (~50 lines vs 400+)
- ✅ Single implementation to maintain
- ✅ All language features work immediately

---

## Crate Structure

The project consists of **11 crates** organized by responsibility:

### Core Language Crates

#### `dsl-ir` - Intermediate Representation
**Purpose:** IR type definitions and serialization

**Key Components:**
- `IRNode` enum (25 variants) - Expression nodes
- `IR` struct - Complete program container
- `Value` enum (8 variants) - Runtime values
- MessagePack and JSON serialization

**Dependencies:** serde, rmp-serde, simplify_baml

**Lines of Code:** ~1,200

---

#### `dsl-core` - Parser and Compiler
**Purpose:** Parse DSL source and compile to IR

**Key Components:**
- `grammar.pest` - PEG grammar (363 lines)
- Parser - Pest-based parser
- `Expr` AST (13 variants) - Abstract syntax tree
- IR Compiler - AST → IR translation

**Dependencies:** pest, pest_derive, dsl-ir

**Lines of Code:** ~2,100

---

#### `dsl-interpreter` - IR Interpreter
**Purpose:** Execute IR nodes

**Key Components:**
- `Interpreter` - Tree-walk interpreter
- `Runtime` - Variable and function storage
- `BuiltinFunctions` - 11 builtin functions
- `PatternMatcher` - Pattern matching engine
- SQL executor (DuckDB integration)

**Dependencies:** dsl-ir, tokio, simplify_baml, reqwest, duckdb

**Lines of Code:** ~1,800

---

### Compilation & Runtime Crates

#### `dsl-compiler` - CLI Compiler
**Purpose:** Command-line tool for compiling DSL to binaries

**Commands:**
- `check` - Validate syntax
- `ir` - Compile to IR (JSON/MessagePack)
- `build` - Compile to Rust binary

**Dependencies:** dsl-core, dsl-ir, dsl-codegen, clap

**Lines of Code:** ~350

---

#### `dsl-codegen` - Code Generator
**Purpose:** Generate Rust code from IR

**Key Components:**
- Rust AST types - Intermediate Rust representation
- Program generator - Creates main() or library
- Type generator - Converts DSL types to Rust structs
- Expression generator - Embeds IR as JSON

**Strategy:** Embedded interpreter (not line-by-line translation)

**Dependencies:** dsl-ir

**Lines of Code:** ~800

---

#### `dsl-runtime` - Runtime Support Library
**Purpose:** Shared runtime for generated code

**Exports:**
- `Value` type
- `Runtime` struct
- Builtin functions
- Common dependencies (tokio, anyhow, serde)

**Dependencies:** dsl-ir, dsl-interpreter

**Lines of Code:** ~200

---

### User Interface Crates

#### `dsl-tui` - Terminal UI (TUI)
**Purpose:** Interactive terminal REPL with ratatui

**Features:**
- Line editing with history
- Syntax highlighting
- Output paging
- REPL commands
- Split-pane layout

**Dependencies:** dsl-core, dsl-interpreter, ratatui, crossterm

**Lines of Code:** ~1,200

---

#### `dsl-egui` - Desktop GUI
**Purpose:** Native desktop GUI with egui

**Features:**
- Rich output rendering (tables, charts, images, markdown)
- Error display with expandable sections
- Autocomplete
- Keyboard shortcuts
- Dark/light themes

**Dependencies:** dsl-core, dsl-interpreter, egui, eframe

**Lines of Code:** ~2,500

---

#### `dsl-repl` - Basic REPL
**Purpose:** Simple command-line REPL (legacy)

**Note:** Mostly superseded by dsl-tui and dsl-egui

**Lines of Code:** ~400

---

### Utility Crates

#### `dsl-autocomplete` - Autocomplete Engine
**Purpose:** Context-aware completions for REPL/editors

**Providers:**
- Keywords (`let`, `type`, `enum`, `def`)
- Builtin functions
- User-defined functions and types
- Variables in scope

**Dependencies:** dsl-core

**Lines of Code:** ~600

---

#### `simplify_baml` - LLM Integration Framework
**Purpose:** Type-safe LLM function calls

**Note:** External dependency used for structured extraction

**Location:** `crates/simplify_baml/`

**Documentation:** See `/crates/simplify_baml/README.md`

---

## Compilation Pipeline

### Detailed Pipeline Flow

```
┌──────────────────────────────────────────────────────┐
│ Phase 1: Lexing & Parsing                             │
└──────────────────────────────────────────────────────┘
    Input:  DSL Source (String)
    Tool:   Pest Parser (grammar.pest)
    Output: Parse Tree (Pest Pairs)

    Example: "upper('hello')" → Pairs { rule: expr, ... }

┌──────────────────────────────────────────────────────┐
│ Phase 2: AST Construction                             │
└──────────────────────────────────────────────────────┘
    Input:  Parse Tree
    Tool:   build_expr() recursive descent
    Output: Expr AST

    Example: FunctionCall {
               name: "upper",
               args: [String("hello")]
             }

┌──────────────────────────────────────────────────────┐
│ Phase 3: IR Compilation                               │
└──────────────────────────────────────────────────────┘
    Input:  Expr AST
    Tool:   compile_to_ir()
    Output: IR structure

    Example: IR {
               entry_expr: IRNode::FunctionCall {
                 name: "upper",
                 args: [IRNode::String("hello")]
               }
             }

┌──────────────────────────────────────────────────────┐
│ Phase 4a: Interpretation (REPL Mode)                  │
└──────────────────────────────────────────────────────┘
    Input:  IR
    Tool:   Interpreter::eval()
    Output: Value

    Example: Value::String("HELLO")

┌──────────────────────────────────────────────────────┐
│ Phase 4b: Code Generation (Binary Mode)               │
└──────────────────────────────────────────────────────┘
    Input:  IR
    Tool:   generate_executable()
    Output: Rust source code

    Example: Generated main() with embedded IR JSON

┌──────────────────────────────────────────────────────┐
│ Phase 5: Binary Compilation (Binary Mode)             │
└──────────────────────────────────────────────────────┘
    Input:  Rust source
    Tool:   cargo build
    Output: Native executable
```

### Parse → AST → IR Example

**Input DSL:**
```javascript
let x = 5
let y = 10
x + y
```

**Parse Tree (simplified):**
```
program
  ├─ let_statement ("let x = 5")
  ├─ let_statement ("let y = 10")
  └─ expr (binary_op)
      ├─ identifier "x"
      ├─ operator "+"
      └─ identifier "y"
```

**AST:**
```rust
Expr::Sequential {
    left: Box::new(Expr::Int(5)),
    right: Box::new(Expr::Sequential {
        left: Box::new(Expr::Int(10)),
        right: Box::new(Expr::BinaryOp {
            left: Box::new(Expr::Variable("x".to_string())),
            op: "+".to_string(),
            right: Box::new(Expr::Variable("y".to_string())),
        }),
        binding: Some(Binding::Single("y".to_string())),
    }),
    binding: Some(Binding::Single("x".to_string())),
}
```

**IR:**
```rust
IR {
    version: "0.1.0",
    types: vec![],
    enums: vec![],
    functions: vec![],
    function_groups: vec![],
    entry_expr: IRNode::Sequential {
        left: Box::new(IRNode::Int(5)),
        right: Box::new(IRNode::Sequential {
            left: Box::new(IRNode::Int(10)),
            right: Box::new(IRNode::BinaryOp {
                left: Box::new(IRNode::Variable("x".to_string())),
                op: "+".to_string(),
                right: Box::new(IRNode::Variable("y".to_string())),
            }),
            binding: Some(IRBinding::Single("y".to_string())),
        }),
        binding: Some(IRBinding::Single("x".to_string())),
    },
}
```

**Execution:**
```
Runtime { vars: {} }
  ↓ eval Sequential
    ↓ eval Int(5) → Value::Int(5)
    ↓ bind "x" → Value::Int(5)
Runtime { vars: { "x": Int(5) } }
    ↓ eval Sequential
      ↓ eval Int(10) → Value::Int(10)
      ↓ bind "y" → Value::Int(10)
Runtime { vars: { "x": Int(5), "y": Int(10) } }
      ↓ eval BinaryOp
        ↓ eval Variable("x") → Value::Int(5)
        ↓ eval Variable("y") → Value::Int(10)
        ↓ apply_binary_op("+", 5, 10) → Value::Int(15)
Result: Value::Int(15)
```

---

## IR Architecture

### IRNode Variants (25 Total)

#### Literals (5)
- `String(String)` - String literals
- `TemplateString(Vec<IRTemplateSegment>)` - Template strings with interpolation
- `Int(i64)` - Integer literals
- `Float(f64)` - Float literals
- `Bool(bool)` - Boolean literals

#### Collections (2)
- `List(Vec<IRNode>)` - List literals
- `Map(Vec<(String, IRNode)>)` - Map literals (order-preserving)

#### Variables & Calls (3)
- `Variable(String)` - Variable references
- `FunctionCall { name, args }` - Function calls
- `TypeInstantiation { type_name, fields }` - Type construction

#### Access (2)
- `FieldAccess { base, field }` - Object field access
- `IndexAccess { base, index }` - Array/map indexing

#### Operations (3)
- `BinaryOp { left, op, right }` - Binary operations (+, -, *, etc.)
- `UnaryOp { op, operand }` - Unary operations (-, not)
- `Conditional { condition, then_expr, else_expr }` - Ternary operator

#### Composition (2)
- `Sequential { left, right, binding }` - Pipeline operator (|>)
- `Parallel { exprs, binding }` - Parallel composition

#### Pattern Matching (1)
- `Match { value, cases }` - Match expressions

#### Agent Primitives (5)
- `SpawnAgent { agent_type, init_state }` - Create agent
- `SendMessage { target, message }` - Fire-and-forget send
- `CallAgent { target, message, timeout_ms }` - Request-reply
- `ReceiveMessage { pattern }` - Receive with pattern matching
- `Broadcast { targets, message }` - Broadcast to multiple agents

#### Control Flow (5)
- `Loop { body }` - Infinite loop
- `While { condition, body }` - Conditional loop
- `For { var, iterable, body }` - For-each loop
- `Break { value }` - Break from loop
- `Continue` - Continue to next iteration

#### Error Handling (2)
- `TryBlock { body, catch_var, catch_body }` - Try-catch
- `Throw { error }` - Throw error

### IR Container Structure

```rust
pub struct IR {
    pub version: String,
    pub types: Vec<Class>,               // Type definitions
    pub enums: Vec<Enum>,                // Enum definitions
    pub functions: Vec<IRFunction>,      // Regular functions
    pub function_groups: Vec<IRFunctionGroup>,  // Pattern-based functions
    pub entry_expr: IRNode,              // Main expression
}
```

### Value Types (Runtime)

```rust
pub enum Value {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    List(Vec<Value>),
    Map(IndexMap<String, Value>),  // Order-preserving
    Null,
    Markdown(String),              // Rich text output
}
```

---

## Runtime Architecture

### Interpreter Structure

```rust
pub struct Interpreter {
    runtime: Runtime,
    builtins: BuiltinFunctions,
    sql_executor: SqlExecutor,
    pattern_matcher: PatternMatcher,
}
```

### Runtime State

```rust
pub struct Runtime {
    pub vars: HashMap<String, Value>,
    pub types: TypeRegistry,
    pub functions: HashMap<String, IRFunction>,
    pub function_groups: HashMap<String, IRFunctionGroup>,
}
```

### Execution Model

**Tree-walk interpreter:**
1. Recursive descent through IR nodes
2. Evaluate children first (post-order)
3. Apply operations on results
4. Update runtime state for bindings

**Async execution:**
- All eval() methods are async
- Supports LLM calls, HTTP requests, SQL queries
- Uses tokio runtime

---

## Phase History

### ✅ Phase 1: Create IR Crate (Week 1)
**Date Completed:** 2025-11-06

- Created `dsl-ir` crate with full IR type definitions
- Added MessagePack and JSON serialization
- Defined 13 core IRNode variants
- Copied Value type with all methods preserved

### ✅ Phase 2: Extend IR for Agentic Features (Week 2)
**Date Completed:** 2025-11-06

- Added 12 new IRNode variants (agents, control flow, error handling)
- Added pattern matching types (IRPattern, IRMatchCase)
- Added agent types (IRAgent, IRMessageHandler)
- Total: 25 IRNode variants

### ✅ Phase 3: Create IR Compiler (Week 2-3)
**Date Completed:** 2025-11-06

- Created `compiler.rs` module in dsl-core
- Implemented AST → IR translation for all Expr variants
- Preserved all function execution modes
- 7 compiler tests passing

### ✅ Phase 4: Create IR Interpreter (Week 3-5)
**Date Completed:** 2025-11-06

- Created `dsl-interpreter` crate
- Ported Runtime, TypeRegistry, BuiltinFunctions
- Implemented tree-walk interpreter
- All 11 builtin functions working

### ✅ Phase 5: Update REPL to Use IR (Week 5)
**Date Completed:** 2025-11-06

- Updated dsl-tui to use IR pipeline
- Ported all REPL commands (`:vars`, `:types`, `:funcs`, etc.)
- Removed duplicate Value type
- All 82 tests passing

### ✅ Phase 6: Create Rust Code Generator (Week 6-8)
**Date Completed:** 2025-11-06

- Created `dsl-codegen` crate
- Rust AST types (~600 lines)
- Expression, type, and function generators
- Embedded interpreter approach

### ✅ Phase 7: Create Compiler CLI (Week 8)
**Date Completed:** 2025-11-06

- Created `dsl-compiler` binary
- 3 commands: check, ir, build
- Integration tests
- 99 total tests passing

### ✅ Phase 8: Create Runtime Support Library (Week 8-9)
**Date Completed:** 2025-11-06

- Created `dsl-runtime` crate
- Re-exported Value, Runtime, builtins
- 107 tests passing workspace-wide

### ✅ Phase 9: Testing and Validation (Week 9-10)
**Date Completed:** 2025-11-06

- Adopted embedded interpreter approach
- Simplified code generation (~50 lines)
- All tests passing
- Binary compilation working

### ✅ Phase 10B: Pattern Matching (Partial) (Week 10-11)
**Date Completed:** 2025-11-07

- Added expression functions (`function double(x) { x * 2 }`)
- Multi-arm function definitions working (`def factorial(0) { 1 }`)
- Match expressions fully functional
- Pattern matching engine (8 pattern types)
- **Status:** 60% complete

### ✅ Phase 11: Remove Legacy Evaluator
**Date Completed:** 2025-11-07

- Removed legacy Evaluator from dsl-core
- Removed legacy builtin.rs
- Cleaned up 3,900 lines of dead code
- 100% IR-based architecture

---

## Design Decisions

### Decision 1: Embedded Interpreter vs Full Codegen

**Options Considered:**
1. Generate Rust code line-by-line (traditional compiler)
2. Embed IR and interpret at runtime (embedded interpreter)

**Chosen:** Embedded interpreter

**Rationale:**
- ✅ Simpler implementation (~50 lines vs 400+)
- ✅ Guaranteed identical behavior (REPL = Binary)
- ✅ Single interpreter to maintain
- ✅ All features work immediately
- ✅ No complex variable substitution or control flow translation
- ⚠️ Tradeoff: Slower startup (parse JSON), but execution speed same

### Decision 2: MessagePack + JSON for IR Serialization

**Options Considered:**
1. Binary only (faster but not debuggable)
2. JSON only (debuggable but larger)
3. Both (best of both worlds)

**Chosen:** Both (MessagePack for storage, JSON for debug)

**Rationale:**
- MessagePack: Compact binary format for caching
- JSON: Human-readable for inspection
- Easy to switch between them

### Decision 3: PEG Parser (Pest) vs Hand-written

**Options Considered:**
1. Hand-written recursive descent parser
2. Parser generator (Pest, LALRPOP, etc.)

**Chosen:** Pest PEG parser

**Rationale:**
- ✅ Grammar as documentation
- ✅ Faster development
- ✅ Easier to modify
- ⚠️ Backtracking issues (see Known Issues)

### Decision 4: Order-Preserving Maps (IndexMap)

**Options Considered:**
1. HashMap (unordered, faster)
2. BTreeMap (sorted by key)
3. IndexMap (insertion-order)

**Chosen:** IndexMap

**Rationale:**
- Preserves field order for type definitions
- Better for serialization (consistent output)
- Important for table display (columns in order)

### Decision 5: Separate GUI Crates (TUI vs egui)

**Options Considered:**
1. Single GUI with feature flags
2. Separate crates

**Chosen:** Separate crates (`dsl-tui` and `dsl-egui`)

**Rationale:**
- Different use cases (terminal vs desktop)
- Allows independent development
- Users can choose what to install
- No feature flag complexity

---

## Current Architecture Status

### What's Complete ✅
- **Core Language:** 13 expression types working
- **Builtin Functions:** All 11 functions working
- **REPL Mode:** TUI and egui fully functional
- **Binary Mode:** Compilation to native executables working
- **Pattern Matching:** Match expressions and multi-arm functions working
- **Expression Functions:** Simple and recursive functions working

### What's In Progress 🚧
- **Pattern Matching (Phase 10B):** 60% complete
  - ✅ IR types and pattern matcher
  - ✅ Match expressions
  - ✅ Multi-arm functions with clause merging
  - ✅ Grammar and parser support
  - ⏳ Remaining: Optimization and edge cases

### What's Planned 📋
- **Phase 10A:** Grammar extensions for agents (optional)
- **Phase 12:** Documentation and polish (ongoing)
- Error span tracking for better error messages
- Exhaustiveness checking for pattern matching
- Type inference and checking
- Agent runtime implementation

---

## Performance Characteristics

### Compilation Times

| Phase | Time | Notes |
|-------|------|-------|
| Parse | <1ms | Pest parser is very fast |
| AST → IR | <1ms | Simple translation |
| IR → JSON | ~1ms | Serialization overhead |
| Cargo build (cold) | ~36s | First build with dependencies |
| Cargo build (warm) | ~2s | Cached dependencies (16x faster) |

### Runtime Performance

| Operation | Time | Notes |
|-----------|------|-------|
| Variable lookup | O(1) | HashMap |
| Function call (builtin) | ~10μs | Direct dispatch |
| Function call (user) | ~50μs | Includes pattern matching |
| LLM call | ~500ms | Network latency |
| SQL query | ~5ms | DuckDB in-memory |

---

## Related Documentation

- [Error Handling](error-handling.md) - Error types and handling strategy
- [Known Issues](known-issues.md) - Parser and runtime issues
- [egui Implementation](egui-implementation.md) - GUI architecture details
- [Building from Source](building.md) - Build instructions

---

**For Questions or Contributions:**
- See main [README.md](../../README.md)
- Check [Documentation Index](../README.md)
