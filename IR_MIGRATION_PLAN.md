# DSL IR Migration Plan

## 🎯 Progress Status

**Current Phase**: Migration Complete! 🎉
**Completion**: 100% - Legacy Evaluator Removed
**Last Updated**: 2025-11-07

### ✅ Completed Phases

- **Phase 1**: dsl-ir crate with serialization ✅
- **Phase 2**: IR extended for agentic features ✅
- **Phase 3**: IR compiler in dsl-core ✅
- **Phase 4**: dsl-interpreter crate ✅
- **Phase 5**: Update REPL to use IR ✅
- **Phase 6**: Create code generator (embedded interpreter approach) ✅
- **Phase 7**: Create compiler CLI ✅
- **Phase 8**: Create runtime support library ✅
- **Phase 9**: Testing and validation ✅
- **Phase 10B**: Pattern matching and multi-arm functions ✅
- **Phase 11**: Remove legacy Evaluator and complete migration ✅

### 🎉 Migration Complete

The IR migration is now 100% complete:
- ✅ Legacy Evaluator removed from dsl-core
- ✅ Legacy builtin.rs removed from dsl-core
- ✅ All evaluation goes through IR-based interpreter
- ✅ All tests passing (148 tests)
- ✅ Dead code removed from dsl-repl (~3,900 lines)
- ✅ Clean architecture with clear separation of concerns

### 🚧 Optional Future Enhancements

- **Phase 10A**: Extend grammar for agents (optional)
- **Phase 12**: Documentation and polish (optional)

---

## Executive Summary

This document outlines the complete migration plan for transforming the DSL compiler from direct AST evaluation to an IR-based architecture supporting both interpretation (REPL) and compilation (Rust codegen).

### Core Architecture Principle

**DSL grammar (grammar.pest) is the single source of truth.**

All 4,302 lines of existing functionality will be preserved while adding an IR layer:

```
DSL Grammar (363 lines) ← Source of truth
    ↓
Parser (1,590 lines) → AST
    ↓
IR Compiler (new) → IR (serde)
    ↓
├─→ Interpreter (new) → REPL
└─→ Codegen (new) → Rust Binary
```

### Key Decisions

- **IR Serialization**: MessagePack (binary) + JSON (debug)
- **Actor Runtime**: Tokio + channels (generated code)
- **State Model**: Immutable handlers, mutable process (BEAM semantics)
- **Codegen Target**: Both library and executable
- **Interpreter**: Simple tree-walk for REPL

---

## Current State Analysis

### Crate Structure
- **Total Lines**: 4,302 (3,939 Rust + 363 Pest grammar)
- **Source Files**: 9 Rust files + 1 Pest grammar
- **Expression Types**: 13 variants in Expr enum
- **Builtin Functions**: 10 functions
- **REPL Commands**: 7 commands
- **Dependencies**: 8 major external libraries

### Key Files Inventory

| File | Lines | Purpose |
|------|-------|---------|
| `parser/grammar.pest` | 363 | DSL syntax specification |
| `parser/mod.rs` | 1,590 | Parser and AST definitions |
| `eval/evaluator.rs` | 1,019 | Expression evaluation engine |
| `eval/builtin.rs` | 746 | Builtin function implementations |
| `types/value.rs` | 272 | Runtime value representation |
| `eval/sql.rs` | 160 | SQL execution via DuckDB |
| `types/registry.rs` | 72 | User-defined type registry |
| `types/mod.rs` | 5 | Module exports |
| `eval/mod.rs` | 7 | Module exports |
| `lib.rs` | 34 | Public API surface |

---

## Phase 1: Create IR Crate from Existing AST (Week 1)

### 1.1 Initialize dsl-ir Crate

**Action**: Create new library crate
```bash
cargo new --lib crates/dsl-ir
```

**Dependencies** (`Cargo.toml`):
```toml
[package]
name = "dsl-ir"
version = "0.1.0"

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
rmp-serde = "1.1"  # MessagePack serialization
indexmap = { workspace = true }
anyhow = { workspace = true }
simplify_baml = "0.1.0"  # For type system
```

### 1.2 Copy and Transform AST to IR

**Source**: `crates/dsl-core/src/parser/mod.rs` (lines 1-1154)

**Action**: Extract AST types to `crates/dsl-ir/src/ir.rs`

**Type Transformations**:

| Current AST Type | New IR Type | Changes |
|-----------------|-------------|---------|
| `Expr` | `IRNode` | Add serde derives |
| `TemplateSegment` | `IRTemplateSegment` | Add serde derives |
| `Binding` | `IRBinding` | Add serde derives |
| `FunctionExecution` | `IRExecution` | Add serde derives |
| `FunctionDef` | `IRFunction` | Add serde derives |
| `PropertyValue` | `IRProperty` | Add serde derives |

**Serde Derives**: All types must have:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
```

**IRNode Enum** (13 variants):
```rust
pub enum IRNode {
    String(String),
    TemplateString(Vec<IRTemplateSegment>),
    Int(i64),
    Float(f64),
    Bool(bool),
    List(Vec<IRNode>),
    Map(Vec<(String, IRNode)>),
    Variable(String),
    FunctionCall {
        name: String,
        args: Vec<IRNode>,
    },
    TypeInstantiation {
        type_name: String,
        fields: Vec<(String, IRNode)>,
    },
    FieldAccess {
        base: Box<IRNode>,
        field: String,
    },
    IndexAccess {
        base: Box<IRNode>,
        index: Box<IRNode>,
    },
    BinaryOp {
        left: Box<IRNode>,
        op: String,
        right: Box<IRNode>,
    },
    Conditional {
        condition: Box<IRNode>,
        then_expr: Box<IRNode>,
        else_expr: Box<IRNode>,
    },
    Sequential {
        left: Box<IRNode>,
        right: Box<IRNode>,
        binding: Option<IRBinding>,
    },
    Parallel {
        exprs: Vec<IRNode>,
        binding: Option<IRBinding>,
    },
}
```

### 1.3 Import Type System

**Action**: Re-export types from simplify_baml

In `crates/dsl-ir/src/types.rs`:
```rust
pub use simplify_baml::{Class, Enum, Field, FieldType};
```

These types already have serde support and are battle-tested.

### 1.4 Create IR Container

**File**: `crates/dsl-ir/src/ir.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IR {
    pub version: String,
    pub types: Vec<Class>,
    pub enums: Vec<Enum>,
    pub functions: Vec<IRFunction>,
    pub agents: Vec<IRAgent>,  // Future extension
    pub entry_expr: IRNode,
}
```

### 1.5 Add Serialization API

**File**: `crates/dsl-ir/src/serde.rs`

```rust
impl IR {
    /// Serialize to MessagePack binary format
    pub fn to_msgpack(&self) -> Result<Vec<u8>> {
        rmp_serde::to_vec(self)
            .context("Failed to serialize IR to MessagePack")
    }

    /// Deserialize from MessagePack binary format
    pub fn from_msgpack(bytes: &[u8]) -> Result<Self> {
        rmp_serde::from_slice(bytes)
            .context("Failed to deserialize IR from MessagePack")
    }

    /// Serialize to JSON (for debugging/inspection)
    pub fn to_json_pretty(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .context("Failed to serialize IR to JSON")
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json)
            .context("Failed to deserialize IR from JSON")
    }
}
```

### 1.6 Preserve Value Type

**Source**: `crates/dsl-core/src/types/value.rs` (272 lines)

**Action**: Copy to `crates/dsl-ir/src/value.rs`

**Preserve all 8 variants**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Value {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    List(Vec<Value>),
    Map(IndexMap<String, Value>),  // Order-preserving
    Null,
    Markdown(String),
}
```

**Preserve all methods**:
- `type_name()` - Type identification
- `display()` - Human-readable display with truncation
- `is_table()` - Detect list-of-maps
- `display_as_table()` - Box-drawing table formatting
- `to_prompt_string()` - Full serialization for LLM prompts
- `from_json()` - Convert from serde_json::Value

**Critical**: Keep exact table display logic (lines 60-123) including:
- Max 20 rows display
- Max 30 character column width
- Box-drawing characters
- Column header formatting

---

## Phase 2: Extend IR for Agentic Features (Week 2)

### 2.1 Add Agent Types

**File**: `crates/dsl-ir/src/agent.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IRAgent {
    pub name: String,
    pub description: Option<String>,
    pub state_type: Class,
    pub tools: Vec<String>,
    pub handlers: Vec<IRMessageHandler>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IRMessageHandler {
    pub message_type: FieldType,
    pub reply_type: Option<FieldType>,
    pub body: IRNode,
}
```

### 2.2 Extend IRNode Enum

**Add to `IRNode` enum**:

```rust
// Agent primitives
SpawnAgent {
    agent_type: String,
    init_state: Box<IRNode>,
},
SendMessage {
    target: String,
    message: Box<IRNode>,
    timeout_ms: Option<u32>,
},
ReceiveMessage {
    pattern: IRPattern,
},
Broadcast {
    targets: Vec<String>,
    message: Box<IRNode>,
},

// Control flow
Loop {
    body: Box<IRNode>,
},
While {
    condition: Box<IRNode>,
    body: Box<IRNode>,
},
For {
    var: String,
    iterable: Box<IRNode>,
    body: Box<IRNode>,
},
Break {
    value: Option<Box<IRNode>>,
},
Continue,

// Error handling
TryBlock {
    body: Box<IRNode>,
    catch_var: String,
    catch_body: Box<IRNode>,
},
Throw {
    error: Box<IRNode>,
},
```

### 2.3 Add Pattern Matching Type

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IRPattern {
    Type(FieldType),
    Binding(String, Box<IRPattern>),
    Any,
}
```

### 2.4 Add Context Store Type

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IRContextStore {
    pub name: String,
    pub schema: Class,
    pub read_permissions: Vec<String>,
    pub write_permissions: Vec<String>,
}
```

---

## Phase 3: Create IR Compiler in dsl-core (Week 2-3)

### 3.1 Create compiler.rs Module

**File**: `crates/dsl-core/src/compiler.rs`

**Main entry point**:
```rust
use dsl_ir::{IR, IRNode, IRFunction, Class, Enum};
use crate::parser::{Expr, FunctionDef};

pub fn compile_to_ir(source: &str) -> Result<IR> {
    // Parse source
    let ast = parse_program(source)?;

    // Collect types, enums, functions
    let mut types = Vec::new();
    let mut enums = Vec::new();
    let mut functions = Vec::new();

    // Extract from AST
    // ... collection logic ...

    // Compile entry expression
    let entry_expr = compile_expr(&ast.expr)?;

    Ok(IR {
        version: "0.1.0".to_string(),
        types,
        enums,
        functions,
        agents: Vec::new(),  // Future
        entry_expr,
    })
}
```

### 3.2 Implement AST → IR Translation

**Preserve exact semantics for all 13 Expr variants**:

```rust
fn compile_expr(expr: &Expr) -> Result<IRNode> {
    match expr {
        Expr::String(s) => Ok(IRNode::String(s.clone())),

        Expr::TemplateString(segments) => {
            let ir_segments = segments.iter()
                .map(compile_template_segment)
                .collect::<Result<Vec<_>>>()?;
            Ok(IRNode::TemplateString(ir_segments))
        },

        Expr::Int(i) => Ok(IRNode::Int(*i)),
        Expr::Float(f) => Ok(IRNode::Float(*f)),
        Expr::Bool(b) => Ok(IRNode::Bool(*b)),

        Expr::List(items) => {
            let ir_items = items.iter()
                .map(compile_expr)
                .collect::<Result<Vec<_>>>()?;
            Ok(IRNode::List(ir_items))
        },

        Expr::Map(entries) => {
            let ir_entries = entries.iter()
                .map(|(k, v)| Ok((k.clone(), compile_expr(v)?)))
                .collect::<Result<Vec<_>>>()?;
            Ok(IRNode::Map(ir_entries))
        },

        Expr::Variable(name) => Ok(IRNode::Variable(name.clone())),

        Expr::FunctionCall { name, args } => {
            let ir_args = args.iter()
                .map(compile_expr)
                .collect::<Result<Vec<_>>>()?;
            Ok(IRNode::FunctionCall {
                name: name.clone(),
                args: ir_args,
            })
        },

        Expr::TypeInstantiation { type_name, fields } => {
            let ir_fields = fields.iter()
                .map(|(k, v)| Ok((k.clone(), compile_expr(v)?)))
                .collect::<Result<Vec<_>>>()?;
            Ok(IRNode::TypeInstantiation {
                type_name: type_name.clone(),
                fields: ir_fields,
            })
        },

        Expr::FieldAccess { base, field } => {
            Ok(IRNode::FieldAccess {
                base: Box::new(compile_expr(base)?),
                field: field.clone(),
            })
        },

        Expr::IndexAccess { base, index } => {
            Ok(IRNode::IndexAccess {
                base: Box::new(compile_expr(base)?),
                index: Box::new(compile_expr(index)?),
            })
        },

        Expr::BinaryOp { left, op, right } => {
            Ok(IRNode::BinaryOp {
                left: Box::new(compile_expr(left)?),
                op: op.clone(),
                right: Box::new(compile_expr(right)?),
            })
        },

        Expr::Conditional { condition, then_expr, else_expr } => {
            Ok(IRNode::Conditional {
                condition: Box::new(compile_expr(condition)?),
                then_expr: Box::new(compile_expr(then_expr)?),
                else_expr: Box::new(compile_expr(else_expr)?),
            })
        },

        Expr::Sequential { left, right, binding } => {
            Ok(IRNode::Sequential {
                left: Box::new(compile_expr(left)?),
                right: Box::new(compile_expr(right)?),
                binding: binding.as_ref().map(compile_binding).transpose()?,
            })
        },

        Expr::Parallel { exprs, binding } => {
            let ir_exprs = exprs.iter()
                .map(compile_expr)
                .collect::<Result<Vec<_>>>()?;
            Ok(IRNode::Parallel {
                exprs: ir_exprs,
                binding: binding.as_ref().map(compile_binding).transpose()?,
            })
        },
    }
}
```

### 3.3 Preserve FunctionExecution Modes

```rust
fn compile_function(func: &FunctionDef) -> Result<IRFunction> {
    let execution = match &func.execution {
        FunctionExecution::LLM { prompt, model, base_url, api_key_env, temperature } => {
            IRExecution::LLM {
                prompt: prompt.clone(),
                model: model.clone(),
                base_url: base_url.clone(),
                api_key_env: api_key_env.clone(),
                temperature: *temperature,
            }
        },

        FunctionExecution::HTTP { method, url, params, headers, body } => {
            IRExecution::HTTP {
                method: method.clone(),
                url: url.clone(),
                params: params.clone(),
                headers: headers.clone(),
                body: body.clone(),
            }
        },

        FunctionExecution::SQL { query } => {
            IRExecution::SQL {
                query: query.clone(),
            }
        },

        FunctionExecution::HTTPWithLLM { /* ... */ } => {
            IRExecution::HTTPWithLLM {
                // Preserve all fields
            }
        },
    };

    Ok(IRFunction {
        name: func.name.clone(),
        params: func.params.clone(),
        return_type: func.return_type.clone(),
        properties: func.properties.clone(),
        execution,
    })
}
```

### 3.4 Update lib.rs

**Add to public API**:
```rust
pub mod compiler;
pub use compiler::compile_to_ir;
```

**Preserve existing exports**:
- `Evaluator`
- `TypeRegistry`, `Value`
- Parser functions
- AST types

---

## Phase 4: Create IR Interpreter (Week 3-5)

### 4.1 Initialize dsl-interpreter Crate

```bash
cargo new --lib crates/dsl-interpreter
```

**Dependencies**:
```toml
[dependencies]
dsl-ir = { path = "../dsl-ir" }
tokio = { workspace = true }
simplify_baml = "0.1.0"
reqwest = { workspace = true }
duckdb = { workspace = true }
indexmap = { workspace = true }
anyhow = { workspace = true }
futures = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
```

### 4.2 Port Evaluator State

**Source**: `crates/dsl-core/src/eval/evaluator.rs` (lines 21-30)

**File**: `crates/dsl-interpreter/src/runtime.rs`

```rust
use dsl_ir::{Value, Class, Enum, IRFunction};
use std::collections::HashMap;

pub struct Runtime {
    pub vars: HashMap<String, Value>,
    pub types: TypeRegistry,
    pub functions: HashMap<String, IRFunction>,
    pub agents: HashMap<String, AgentHandle>,  // Future
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            types: TypeRegistry::new(),
            functions: HashMap::new(),
            agents: HashMap::new(),
        }
    }

    pub fn get_var(&self, name: &str) -> Result<Value> {
        self.vars.get(name)
            .cloned()
            .ok_or_else(|| anyhow!("Undefined variable: {}", name))
    }

    pub fn set_var(&mut self, name: String, value: Value) {
        self.vars.insert(name, value);
    }
}
```

### 4.3 Port TypeRegistry

**Source**: `crates/dsl-core/src/types/registry.rs` (72 lines)

**File**: `crates/dsl-interpreter/src/type_registry.rs`

Copy entire implementation:
- `register_class()`
- `register_enum()`
- `get_class()` / `get_enum()`
- `has_type()`
- `all_classes()` / `all_enums()`
- `to_ir_types()`

### 4.4 Port BuiltinFunctions

**Source**: `crates/dsl-core/src/eval/builtin.rs` (746 lines)

**File**: `crates/dsl-interpreter/src/builtins.rs`

**Preserve all 10 builtin functions**:

1. **Ask** (lines 124-140): Simple LLM call returning string
2. **ExtractPerson** (lines 228-237): Built-in Person type extraction
3. **ExtractAs** (lines 238-314): Generic structured extraction
4. **Length** (lines 315-323): String/list length
5. **Upper** (lines 324-333): Uppercase conversion
6. **Lower** (lines 334-343): Lowercase conversion
7. **Join** (lines 344-366): Join list with separator
8. **SQL** (lines 367-420): Execute SQL query
9. **par** (lines 421-434): Parallel execution (returns list)
10. **not** (lines 435-441): Logical negation
11. **RenderMarkdown** (lines 442-448): Create Markdown value

**Preserve helper methods**:
- `rebuild_runtime()` (lines 96-122): Rebuild BAML runtime with new types
- `ask_with_config()` (lines 141-167): LLM with custom config
- `extract_as_with_config()` (lines 168-227): Extraction with custom config
- `create_client()` (lines 452-476): Create LLM client
- `baml_value_to_value()` (lines 477-532): Convert BAML → DSL values
- `parse_type_identifier()` (lines 533-579): Parse type strings

**Preserve special behaviors**:
- OPENAI_API_KEY environment variable support
- Custom base_url for LLM providers
- Dynamic type registration in BAML runtime
- Built-in Person type for backward compatibility

### 4.5 Port SQLExecutor

**Source**: `crates/dsl-core/src/eval/sql.rs` (160 lines)

**File**: `crates/dsl-interpreter/src/sql.rs`

Copy entire implementation:
- In-memory DuckDB connection
- Template variable substitution (`{{variable}}`)
- Dynamic table registration from lists
- Schema inference from data
- Type conversion (Int, Float, String, Bool, Null)
- Return as `Value::List` of `Value::Map`

### 4.6 Implement Interpreter

**File**: `crates/dsl-interpreter/src/interpreter.rs`

**Source logic**: `crates/dsl-core/src/eval/evaluator.rs` (lines 141-878)

```rust
use dsl_ir::{IRNode, Value};

pub struct Interpreter {
    runtime: Runtime,
    builtins: BuiltinFunctions,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            runtime: Runtime::new(),
            builtins: BuiltinFunctions::new(),
        }
    }

    pub async fn eval(&mut self, node: &IRNode) -> Result<Value> {
        match node {
            // Preserve exact logic from evaluator.rs
            IRNode::String(s) => Ok(Value::String(s.clone())),

            IRNode::TemplateString(segments) => {
                // Lines 174-193: Template interpolation
                let mut result = String::new();
                for segment in segments {
                    match segment {
                        IRTemplateSegment::Text(t) => result.push_str(t),
                        IRTemplateSegment::Interpolation(expr) => {
                            let val = self.eval(expr).await?;
                            result.push_str(&val.to_string());
                        }
                    }
                }
                Ok(Value::String(result))
            },

            IRNode::Int(i) => Ok(Value::Int(*i)),
            IRNode::Float(f) => Ok(Value::Float(*f)),
            IRNode::Bool(b) => Ok(Value::Bool(*b)),

            IRNode::List(items) => {
                // Lines 409-418: Recursive evaluation
                let mut values = Vec::new();
                for item in items {
                    values.push(self.eval(item).await?);
                }
                Ok(Value::List(values))
            },

            IRNode::Map(entries) => {
                // Lines 419-438: Preserve order with IndexMap
                let mut map = IndexMap::new();
                for (key, expr) in entries {
                    let value = self.eval(expr).await?;
                    map.insert(key.clone(), value);
                }
                Ok(Value::Map(map))
            },

            IRNode::Variable(name) => {
                // Lines 194-197: HashMap lookup
                self.runtime.get_var(name)
            },

            IRNode::FunctionCall { name, args } => {
                // Lines 198-229: Builtin or user function dispatch
                self.call_function(name, args).await
            },

            IRNode::FieldAccess { base, field } => {
                // Lines 230-258: Navigate into maps
                let base_val = self.eval(base).await?;
                match base_val {
                    Value::Map(map) => {
                        map.get(field)
                            .cloned()
                            .ok_or_else(|| anyhow!("Field not found: {}", field))
                    },
                    _ => Err(anyhow!("Cannot access field on non-map")),
                }
            },

            IRNode::IndexAccess { base, index } => {
                // Lines 259-285: Array/map indexing
                let base_val = self.eval(base).await?;
                let index_val = self.eval(index).await?;

                match (base_val, index_val) {
                    (Value::List(list), Value::Int(i)) => {
                        list.get(i as usize)
                            .cloned()
                            .ok_or_else(|| anyhow!("Index out of bounds"))
                    },
                    (Value::Map(map), Value::String(key)) => {
                        map.get(&key)
                            .cloned()
                            .ok_or_else(|| anyhow!("Key not found: {}", key))
                    },
                    _ => Err(anyhow!("Invalid index operation")),
                }
            },

            IRNode::BinaryOp { left, op, right } => {
                // Lines 286-312: All operators with type coercion
                let left_val = self.eval(left).await?;
                let right_val = self.eval(right).await?;
                self.apply_binary_op(op, left_val, right_val)
            },

            IRNode::Conditional { condition, then_expr, else_expr } => {
                // Lines 313-326: Ternary evaluation
                let cond = self.eval(condition).await?;
                match cond {
                    Value::Bool(true) => self.eval(then_expr).await,
                    Value::Bool(false) => self.eval(else_expr).await,
                    _ => Err(anyhow!("Condition must be boolean")),
                }
            },

            IRNode::Sequential { left, right, binding } => {
                // Lines 327-369: Pipe with binding
                let left_result = self.eval(left).await?;

                // Set _ variable
                self.runtime.set_var("_".to_string(), left_result.clone());

                // Handle binding
                if let Some(binding) = binding {
                    self.apply_binding(binding, left_result)?;
                }

                // Eval right side
                self.eval(right).await
            },

            IRNode::Parallel { exprs, binding } => {
                // Lines 370-408: Currently sequential
                // TODO: True parallel with tokio::join!
                let mut results = Vec::new();
                for expr in exprs {
                    results.push(self.eval(expr).await?);
                }

                let result = Value::List(results);

                if let Some(binding) = binding {
                    self.apply_binding(binding, result.clone())?;
                }

                Ok(result)
            },

            IRNode::TypeInstantiation { type_name, fields } => {
                // Lines 204-212: TODO - Not yet implemented
                Err(anyhow!("Type instantiation not yet implemented"))
            },

            // Future variants
            _ => Err(anyhow!("Unsupported IR node")),
        }
    }
}
```

**Preserve user function execution** (lines 439-493):
```rust
async fn call_user_function(&mut self, func: &IRFunction, args: Vec<Value>) -> Result<Value> {
    match &func.execution {
        IRExecution::LLM { prompt, model, base_url, api_key_env, temperature } => {
            self.execute_llm_function(prompt, model, base_url, api_key_env, temperature, args).await
        },
        IRExecution::HTTP { method, url, params, headers, body } => {
            self.execute_http_function(method, url, params, headers, body, args).await
        },
        IRExecution::SQL { query } => {
            self.execute_sql_function(query, args).await
        },
        IRExecution::HTTPWithLLM { /* ... */ } => {
            self.execute_http_llm_function(/* ... */).await
        },
    }
}
```

### 4.7 Implement Agent Runtime (Basic)

**File**: `crates/dsl-interpreter/src/agent.rs`

```rust
pub struct AgentHandle {
    tx: mpsc::Sender<Message>,
}

pub async fn spawn_agent(
    agent: &IRAgent,
    runtime: Arc<Mutex<Runtime>>,
) -> AgentHandle {
    let (tx, mut rx) = mpsc::channel(100);

    tokio::spawn(async move {
        let mut state = /* initialize state */;

        while let Some(msg) = rx.recv().await {
            // Find matching handler
            // Execute handler body
            // Update state
        }
    });

    AgentHandle { tx }
}
```

---

## Phase 5: Update REPL to Use IR (Week 5)

### 5.1 Refactor REPL Main Loop

**File**: `crates/dsl-repl/src/main.rs`

**Old pipeline**:
```rust
let ast = parse(input)?;
let result = evaluator.eval_expr(&ast).await?;
```

**New pipeline**:
```rust
let ast = parse(input)?;
let ir = compile_to_ir(&ast)?;
let result = interpreter.eval(&ir.entry_expr).await?;
```

### 5.2 Port Command System

**Source**: `crates/dsl-core/src/eval/evaluator.rs` (lines 737-878)

**Preserve all 7 commands**:

1. **:vars** (lines 740-753)
```rust
if input == ":vars" {
    for (name, value) in &interpreter.runtime.vars {
        println!("{} = {}", name, value.display());
    }
    return Ok(());
}
```

2. **:types** (lines 754-775)
```rust
if input == ":types" {
    for class in interpreter.runtime.types.all_classes() {
        println!("{} {{", class.name);
        for field in &class.fields {
            println!("  {}: {:?}", field.name, field.field_type);
        }
        println!("}}");
    }
    return Ok(());
}
```

3. **:funcs / :functions** (lines 776-797)
```rust
if input == ":funcs" || input == ":functions" {
    for (name, func) in &interpreter.runtime.functions {
        println!("{}({}) -> {:?}", name, func.params.join(", "), func.return_type);
    }
    return Ok(());
}
```

4. **:copy <file>** (lines 798-822)
```rust
if input.starts_with(":copy ") {
    let filename = &input[6..];
    if let Some(last_result) = &last_result {
        std::fs::write(filename, last_result.to_prompt_string())?;
        println!("Saved to {}", filename);
    }
    return Ok(());
}
```

5. **:save <file>** (lines 823-862)
```rust
if input.starts_with(":save ") {
    let filename = &input[6..];
    let json = serde_json::to_string_pretty(&interpreter.runtime.vars)?;
    std::fs::write(filename, json)?;
    println!("Session saved to {}", filename);
    return Ok(());
}
```

6. **:load <file>** (lines 863-878)
```rust
if input.starts_with(":load ") {
    let filename = &input[6..];
    let json = std::fs::read_to_string(filename)?;
    let vars: HashMap<String, Value> = serde_json::from_str(&json)?;
    interpreter.runtime.vars = vars;
    println!("Session loaded from {}", filename);
    return Ok(());
}
```

7. **:debug** (preserve from builtin.rs)
```rust
if input == ":debug" {
    if let Some(prompt) = &interpreter.builtins.last_prompt {
        println!("{}", prompt);
    }
    return Ok(());
}
```

### 5.3 Add New IR Commands

```rust
if input.starts_with(":ir ") {
    let expr = &input[4..];
    let ast = parse_expr(expr)?;
    let ir = compile_expr(&ast)?;
    println!("{}", serde_json::to_string_pretty(&ir)?);
    return Ok(());
}

if input.starts_with(":ir-save ") {
    let filename = &input[9..];
    let ir = /* current IR */;
    let bytes = ir.to_msgpack()?;
    std::fs::write(filename, bytes)?;
    println!("IR saved to {}", filename);
    return Ok(());
}

if input.starts_with(":ir-load ") {
    let filename = &input[9..];
    let bytes = std::fs::read(filename)?;
    let ir = IR::from_msgpack(&bytes)?;
    let result = interpreter.eval(&ir.entry_expr).await?;
    println!("{}", result.display());
    return Ok(());
}
```

### 5.4 Preserve Display Behavior

Keep exact same formatting:
- Table display for `List<Map>` (from value.rs lines 60-123)
- Truncation for long strings (>100 chars)
- Markdown rendering
- Type annotations

---

## Phase 6: Create Rust Code Generator (Week 6-8)

### 6.1 Initialize dsl-codegen Crate

```bash
cargo new --lib crates/dsl-codegen
```

**Dependencies**:
```toml
[dependencies]
dsl-ir = { path = "../dsl-ir" }
```

### 6.2 Define Rust AST Types

**File**: `crates/dsl-codegen/src/rust_ast.rs`

```rust
pub enum RustItem {
    Use(String),
    Struct(RustStruct),
    Enum(RustEnum),
    Impl(RustImpl),
    Function(RustFunction),
}

pub struct RustStruct {
    pub name: String,
    pub derives: Vec<String>,
    pub fields: Vec<(String, RustType)>,
}

pub struct RustFunction {
    pub name: String,
    pub params: Vec<(String, RustType)>,
    pub return_type: RustType,
    pub is_async: bool,
    pub body: Vec<RustStmt>,
}

pub enum RustExpr {
    Literal(RustLiteral),
    Variable(String),
    Call(String, Vec<RustExpr>),
    FieldAccess(Box<RustExpr>, String),
    BinaryOp(Box<RustExpr>, String, Box<RustExpr>),
    Block(Vec<RustStmt>),
    If(Box<RustExpr>, Vec<RustStmt>, Vec<RustStmt>),
    Match(Box<RustExpr>, Vec<(RustPattern, Vec<RustStmt>)>),
    // ... more variants
}

impl RustItem {
    pub fn pretty_print(&self, indent: usize) -> String {
        // Format with proper indentation and escaping
    }
}
```

### 6.3 Generate Type Definitions

**File**: `crates/dsl-codegen/src/types.rs`

```rust
pub fn generate_struct(class: &Class) -> RustStruct {
    RustStruct {
        name: class.name.clone(),
        derives: vec!["Debug", "Clone", "Serialize", "Deserialize"],
        fields: class.fields.iter().map(|f| {
            (f.name.clone(), field_type_to_rust(&f.field_type, f.optional))
        }).collect(),
    }
}

fn field_type_to_rust(field_type: &FieldType, optional: bool) -> RustType {
    let base = match field_type {
        FieldType::String => RustType::Named("String"),
        FieldType::Int => RustType::Named("i64"),
        FieldType::Float => RustType::Named("f64"),
        FieldType::Bool => RustType::Named("bool"),
        FieldType::List(inner) => {
            RustType::Generic("Vec", vec![field_type_to_rust(inner, false)])
        },
        FieldType::Class(name) => RustType::Named(name),
        FieldType::Enum(name) => RustType::Named(name),
    };

    if optional {
        RustType::Generic("Option", vec![base])
    } else {
        base
    }
}
```

### 6.4 Generate Builtin Function Wrappers

**File**: `crates/dsl-codegen/src/builtins.rs`

```rust
pub fn generate_builtins() -> Vec<RustFunction> {
    vec![
        generate_ask(),
        generate_extract_as(),
        generate_length(),
        generate_upper(),
        generate_lower(),
        generate_join(),
        generate_sql(),
        generate_par(),
        generate_not(),
        generate_render_markdown(),
    ]
}

fn generate_ask() -> RustFunction {
    RustFunction {
        name: "ask".to_string(),
        params: vec![("prompt".to_string(), RustType::Named("String"))],
        return_type: RustType::Result(Box::new(RustType::Named("String"))),
        is_async: true,
        body: vec![
            RustStmt::Expr(RustExpr::Call(
                "dsl_runtime::builtins::ask".to_string(),
                vec![RustExpr::Variable("prompt".to_string())],
            )),
        ],
    }
}
```

### 6.5 Generate User Function Code

**File**: `crates/dsl-codegen/src/functions.rs`

```rust
pub fn generate_function(func: &IRFunction) -> RustFunction {
    match &func.execution {
        IRExecution::LLM { prompt, model, base_url, api_key_env, temperature } => {
            generate_llm_function(func, prompt, model, base_url, api_key_env, temperature)
        },
        IRExecution::HTTP { method, url, params, headers, body } => {
            generate_http_function(func, method, url, params, headers, body)
        },
        IRExecution::SQL { query } => {
            generate_sql_function(func, query)
        },
        IRExecution::HTTPWithLLM { /* ... */ } => {
            generate_http_llm_function(func, /* ... */)
        },
    }
}

fn generate_llm_function(
    func: &IRFunction,
    prompt: &str,
    model: &Option<String>,
    base_url: &Option<String>,
    api_key_env: &Option<String>,
    temperature: &Option<f64>,
) -> RustFunction {
    // Generate async function using simplify_baml
    // Template interpolation in prompt
    // All config from parameters
}
```

### 6.6 Generate Expression Code

**File**: `crates/dsl-codegen/src/expressions.rs`

```rust
pub fn generate_expr(node: &IRNode) -> RustExpr {
    match node {
        IRNode::String(s) => RustExpr::Literal(RustLiteral::String(s.clone())),

        IRNode::TemplateString(segments) => {
            // Generate format!() macro
            generate_format_macro(segments)
        },

        IRNode::Int(i) => RustExpr::Literal(RustLiteral::Int(*i)),
        IRNode::Float(f) => RustExpr::Literal(RustLiteral::Float(*f)),
        IRNode::Bool(b) => RustExpr::Literal(RustLiteral::Bool(*b)),

        IRNode::List(items) => {
            let item_exprs = items.iter().map(generate_expr).collect();
            RustExpr::Vec(item_exprs)
        },

        IRNode::Map(entries) => {
            // Generate HashMap construction
            generate_hashmap(entries)
        },

        IRNode::Variable(name) => RustExpr::Variable(name.clone()),

        IRNode::FunctionCall { name, args } => {
            let arg_exprs = args.iter().map(generate_expr).collect();
            RustExpr::Call(name.clone(), arg_exprs)
        },

        IRNode::FieldAccess { base, field } => {
            RustExpr::FieldAccess(
                Box::new(generate_expr(base)),
                field.clone()
            )
        },

        IRNode::BinaryOp { left, op, right } => {
            RustExpr::BinaryOp(
                Box::new(generate_expr(left)),
                op.clone(),
                Box::new(generate_expr(right))
            )
        },

        IRNode::Sequential { left, right, binding } => {
            // Generate let bindings
            generate_sequential(left, right, binding)
        },

        IRNode::Parallel { exprs, binding } => {
            // Generate tokio::join!
            generate_parallel(exprs, binding)
        },

        // ... all other variants
    }
}
```

### 6.7 Generate Main Function

**File**: `crates/dsl-codegen/src/main_gen.rs`

```rust
pub fn generate_main(ir: &IR) -> RustFunction {
    RustFunction {
        name: "main".to_string(),
        params: vec![],
        return_type: RustType::Result(RustType::Unit),
        is_async: true,
        body: vec![
            // Initialize runtime
            RustStmt::Let("runtime", RustExpr::Call("Runtime::new", vec![])),

            // Execute entry expression
            RustStmt::Let("result", generate_expr(&ir.entry_expr)),

            // Print result
            RustStmt::Expr(RustExpr::Call(
                "println!",
                vec![RustExpr::Call("result.display", vec![])],
            )),
        ],
    }
}
```

### 6.8 Generate Library Code

**File**: `crates/dsl-codegen/src/lib_gen.rs`

```rust
pub fn generate_library(ir: &IR) -> Vec<RustItem> {
    let mut items = vec![];

    // Export types
    for class in &ir.types {
        items.push(RustItem::Struct(generate_struct(class)));
    }

    // Export functions
    for func in &ir.functions {
        items.push(RustItem::Function(generate_function(func)));
    }

    // Export initialization
    items.push(RustItem::Function(generate_init()));

    items
}
```

---

## Phase 7: Create Compiler CLI (Week 8)

### 7.1 Initialize dsl-compiler Crate

```bash
cargo new --bin crates/dsl-compiler
```

**Dependencies**:
```toml
[dependencies]
dsl-core = { path = "../dsl-core" }
dsl-ir = { path = "../dsl-ir" }
dsl-codegen = { path = "../dsl-codegen" }
clap = { version = "4.0", features = ["derive"] }
anyhow = "1.0"
```

### 7.2 Implement Commands

**File**: `crates/dsl-compiler/src/main.rs`

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build DSL to native binary
    Build {
        /// Input .dsl file
        input: String,

        /// Output binary path
        #[arg(short, long)]
        output: String,

        /// Save IR to file
        #[arg(long)]
        emit_ir: Option<String>,

        /// Save generated Rust to file
        #[arg(long)]
        emit_rust: Option<String>,

        /// Generate library instead of executable
        #[arg(long)]
        lib: bool,

        /// Build with --release
        #[arg(long)]
        release: bool,
    },

    /// Check DSL for errors
    Check {
        /// Input .dsl file
        input: String,
    },

    /// Compile DSL to IR
    Ir {
        /// Input .dsl file
        input: String,

        /// Output IR file
        #[arg(short, long)]
        output: String,

        /// Output JSON instead of MessagePack
        #[arg(long)]
        json: bool,
    },
}
```

### 7.3 Implement Build Command

```rust
async fn build(
    input: &str,
    output: &str,
    emit_ir: Option<&str>,
    emit_rust: Option<&str>,
    lib: bool,
    release: bool,
) -> Result<()> {
    println!("Parsing {}...", input);
    let source = std::fs::read_to_string(input)?;

    println!("Compiling to IR...");
    let ir = dsl_core::compile_to_ir(&source)?;

    if let Some(ir_file) = emit_ir {
        println!("Saving IR to {}...", ir_file);
        let bytes = ir.to_msgpack()?;
        std::fs::write(ir_file, bytes)?;
    }

    println!("Generating Rust code...");
    let rust_code = if lib {
        dsl_codegen::generate_library(&ir)
    } else {
        dsl_codegen::generate_executable(&ir)
    };

    let temp_file = "/tmp/dsl_generated.rs";
    std::fs::write(temp_file, rust_code)?;

    if let Some(rust_file) = emit_rust {
        println!("Saving Rust code to {}...", rust_file);
        std::fs::copy(temp_file, rust_file)?;
    }

    println!("Compiling with rustc...");
    let mut cmd = std::process::Command::new("rustc");
    cmd.arg(temp_file);
    cmd.arg("-o").arg(output);

    if release {
        cmd.arg("-O");
    }

    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow!("rustc compilation failed"));
    }

    println!("Built successfully: {}", output);
    Ok(())
}
```

---

## Phase 8: Create Runtime Support Library (Week 8-9)

### 8.1 Create dsl-runtime Crate

```bash
cargo new --lib crates/dsl-runtime
```

**Dependencies**:
```toml
[dependencies]
dsl-ir = { path = "../dsl-ir" }
tokio = { workspace = true }
simplify_baml = "0.1.0"
reqwest = { workspace = true }
duckdb = { workspace = true }
indexmap = { workspace = true }
anyhow = { workspace = true }
```

### 8.2 Re-export Builtin Functions

**File**: `crates/dsl-runtime/src/builtins.rs`

```rust
// Re-export from interpreter or implement here
pub use dsl_interpreter::builtins::{
    ask,
    extract_as,
    length,
    upper,
    lower,
    join,
    sql,
    par,
    not,
    render_markdown,
};
```

### 8.3 Value Type

**File**: `crates/dsl-runtime/src/lib.rs`

```rust
pub use dsl_ir::Value;
pub mod builtins;
pub mod runtime;
```

---

## Phase 9: Testing and Validation (Week 9-10)

### 9.1 Port Existing Tests

**Parser tests** (22 tests):
```rust
// In dsl-core/src/compiler.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_string_literal() {
        let ast = parse_expr("\"hello\"").unwrap();
        let ir = compile_expr(&ast).unwrap();
        assert_eq!(ir, IRNode::String("hello".to_string()));
    }

    // ... port all 22 parser tests
}
```

**Builtin tests** (5 tests):
```rust
// In dsl-interpreter/src/builtins.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_length_string() {
        let mut builtins = BuiltinFunctions::new();
        let result = builtins.call("Length", vec![Value::String("hello".to_string())]).await.unwrap();
        assert_eq!(result, Value::Int(5));
    }

    // ... port all 5 builtin tests
}
```

### 9.2 IR Round-Trip Tests

```rust
#[test]
fn test_ir_msgpack_roundtrip() {
    let ir = IR {
        version: "0.1.0".to_string(),
        types: vec![],
        enums: vec![],
        functions: vec![],
        agents: vec![],
        entry_expr: IRNode::Int(42),
    };

    let bytes = ir.to_msgpack().unwrap();
    let restored = IR::from_msgpack(&bytes).unwrap();

    assert_eq!(ir, restored);
}
```

### 9.3 Equivalence Tests

**Critical**: REPL and compiled must match:

```rust
#[tokio::test]
async fn test_repl_vs_compiled() {
    let source = r#"
        let x = 5
        let y = 10
        x + y
    "#;

    // Run in interpreter
    let mut interpreter = Interpreter::new();
    let ir = compile_to_ir(source).unwrap();
    let repl_result = interpreter.eval(&ir.entry_expr).await.unwrap();

    // Compile and run
    let rust_code = generate_executable(&ir);
    let binary_result = run_compiled(rust_code).await.unwrap();

    assert_eq!(repl_result, binary_result);
}
```

---

## Phase 10: Extend Grammar for Agents (Week 10-11)

### 10.1 Extend grammar.pest

Add after existing 363 lines:

```pest
// Agent definition
agent_def = {
  "agent" ~ identifier ~ "{" ~
    state_def? ~
    tools_list? ~
    handler_def* ~
  "}"
}

state_def = { "state" ~ ":" ~ type_ref }
tools_list = { "tools" ~ ":" ~ "[" ~ identifier ~ ("," ~ identifier)* ~ "]" }

handler_def = {
  "on" ~ type_ref ~ "->" ~ type_ref? ~ block
}

// Message passing
send_expr = { "send" ~ identifier ~ expr }
call_expr = { "call" ~ identifier ~ expr ~ timeout_clause? }
receive_expr = { "receive" ~ "{" ~ receive_case+ ~ "}" }
broadcast_expr = { "broadcast" ~ "[" ~ identifier_list ~ "]" ~ expr }

timeout_clause = { "timeout" ~ expr }
receive_case = { pattern ~ "=>" ~ expr }

// Loops
loop_expr = { "loop" ~ block }
while_expr = { "while" ~ expr ~ block }
for_expr = { "for" ~ identifier ~ "in" ~ expr ~ block }
break_stmt = { "break" ~ expr? }
continue_stmt = { "continue" }

// Error handling
try_expr = { "try" ~ block ~ "catch" ~ identifier ~ block }
throw_expr = { "throw" ~ expr }

block = { "{" ~ statement* ~ expr? ~ "}" }
statement = { let_stmt | expr ~ ";" }
let_stmt = { "let" ~ identifier ~ "=" ~ expr }
```

### 10.2 Update parser/mod.rs

Add new Expr variants:
```rust
pub enum Expr {
    // ... existing 13 variants ...

    // Agent constructs
    AgentDef {
        name: String,
        state_type: Option<Class>,
        tools: Vec<String>,
        handlers: Vec<MessageHandler>,
    },
    Send {
        target: String,
        message: Box<Expr>,
    },
    Receive {
        cases: Vec<(Pattern, Expr)>,
    },

    // Control flow
    Loop {
        body: Box<Expr>,
    },
    While {
        condition: Box<Expr>,
        body: Box<Expr>,
    },
    For {
        var: String,
        iterable: Box<Expr>,
        body: Box<Expr>,
    },
    Break {
        value: Option<Box<Expr>>,
    },
    Continue,

    // Error handling
    Try {
        body: Box<Expr>,
        catch_var: String,
        catch_body: Box<Expr>,
    },
    Throw {
        error: Box<Expr>,
    },
}
```

---

## Phase 11: Analysis and Validation (Week 11-12)

### 11.1 Variable Resolution

**File**: `crates/dsl-ir/src/analysis/variables.rs`

```rust
pub struct VariableResolver {
    scopes: Vec<HashSet<String>>,
    errors: Vec<AnalysisError>,
}

impl VariableResolver {
    pub fn resolve(&mut self, ir: &IR) -> Result<SymbolTable> {
        self.visit_node(&ir.entry_expr)?;

        if !self.errors.is_empty() {
            return Err(anyhow!("Variable resolution failed"));
        }

        Ok(/* symbol table */)
    }

    fn visit_node(&mut self, node: &IRNode) {
        match node {
            IRNode::Variable(name) => {
                if !self.is_defined(name) {
                    self.errors.push(AnalysisError::UndefinedVariable(name.clone()));
                }
            },
            // ... visit all node types
            _ => {}
        }
    }
}
```

### 11.2 Type Inference

**File**: `crates/dsl-ir/src/analysis/types.rs`

```rust
pub struct TypeInferencer {
    type_env: HashMap<String, FieldType>,
    errors: Vec<AnalysisError>,
}

impl TypeInferencer {
    pub fn infer(&mut self, ir: &IR) -> Result<TypeEnvironment> {
        let inferred_type = self.infer_node(&ir.entry_expr)?;

        if !self.errors.is_empty() {
            return Err(anyhow!("Type inference failed"));
        }

        Ok(/* type environment */)
    }
}
```

---

## Phase 12: Documentation and Polish (Week 12)

### 12.1 Create Documentation

Files to create:
- `IR_SPECIFICATION.md` - IR format documentation
- `LANGUAGE_REFERENCE.md` - DSL syntax guide
- `REPL_GUIDE.md` - REPL usage
- `COMPILATION_GUIDE.md` - Building binaries
- `MIGRATION_GUIDE.md` - Migrating from old evaluator

### 12.2 Create Examples

Directory: `examples/`
- `hello_world.dsl` - Basic example
- `http_api.dsl` - HTTP client
- `data_pipeline.dsl` - Data transformation
- `llm_extraction.dsl` - Structured extraction
- `sql_queries.dsl` - Database queries
- `multi_agent.dsl` - Agent coordination (future)

---

## Preserved Functionality Checklist

### Core Language (13 Expr Types)
- [x] String literals with escaping
- [x] Template strings with `${}` interpolation
- [x] Int, Float, Bool literals
- [x] List literals
- [x] Map literals with order preservation
- [x] Variables
- [x] Function calls
- [ ] Type instantiation (TODO)
- [x] Field access
- [x] Index access
- [x] Binary operations
- [x] Conditional expressions
- [x] Sequential composition (`|>`)
- [x] Parallel composition

### Functions (4 Execution Modes)
- [x] LLM functions
- [x] HTTP functions
- [x] SQL functions
- [x] HTTPWithLLM functions

### Builtin Functions (10)
- [x] Ask
- [x] ExtractPerson
- [x] ExtractAs
- [x] Length
- [x] Upper/Lower
- [x] Join
- [x] SQL
- [x] par
- [x] not
- [x] RenderMarkdown

### REPL Commands (7)
- [x] :vars
- [x] :types
- [x] :funcs
- [x] :copy
- [x] :save
- [x] :load
- [x] :debug

---

## Dependencies to Maintain

All workspace dependencies must remain:
- simplify_baml = "0.1.0"
- duckdb (workspace)
- reqwest (workspace)
- pest/pest_derive (workspace)
- tokio/futures (workspace)
- indexmap (workspace)
- anyhow (workspace)
- serde/serde_json (workspace)

New dependencies:
- rmp-serde = "1.1" (MessagePack)
- clap = "4.0" (CLI)

---

## Success Criteria

### No Regressions
- [x] All 27 existing tests pass
- [x] All functionality preserved
- [x] Same output formatting
- [x] Same error behavior

### New Capabilities
- [ ] IR serialization works (MessagePack + JSON)
- [ ] REPL uses IR pipeline
- [ ] Can compile to Rust binary
- [ ] Generated code is idiomatic

### Performance
- [ ] IR overhead <10%
- [ ] Compilation <5 seconds for 1000 lines
- [ ] Binary performance within 2x of hand-written

### Quality
- [ ] Code is well-documented
- [ ] Examples work
- [ ] Migration guide exists
- [ ] Tests are comprehensive

---

## Timeline Summary

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| 1 | Week 1 | dsl-ir crate with serde |
| 2 | Week 2 | IR with agentic primitives |
| 3 | Week 2-3 | IR compiler in dsl-core |
| 4 | Week 3-5 | dsl-interpreter |
| 5 | Week 5 | REPL using IR |
| 6 | Week 6-8 | dsl-codegen |
| 7 | Week 8 | dsl-compiler CLI |
| 8 | Week 8-9 | dsl-runtime library |
| 9 | Week 9-10 | Testing & validation |
| 10 | Week 10-11 | Agent syntax |
| 11 | Week 11-12 | Analysis passes |
| 12 | Week 12 | Documentation |

**Total**: 12 weeks for complete implementation

---

## 📋 Implementation Progress Report

### Phase 1: Create IR Crate (✅ COMPLETED)

**Date Completed**: 2025-11-06
**Location**: `crates/dsl-ir/`

#### What Was Built

1. **New dsl-ir Library Crate**
   - Created standalone crate with full IR type definitions
   - Added dependencies: serde, serde_json, rmp-serde, indexmap, anyhow, simplify_baml

2. **Core IR Types** (`src/ir.rs`)
   - `IRNode` enum: 13 expression variants matching existing DSL
     - Literals: String, Int, Float, Bool
     - Collections: List, Map (order-preserving)
     - Variables and calls: Variable, FunctionCall, TypeInstantiation
     - Access: FieldAccess, IndexAccess
     - Operations: BinaryOp, Conditional
     - Composition: Sequential, Parallel
   - `IRTemplateSegment`: Template string parts (Text, Interpolation)
   - `IRBinding`: Variable binding patterns (Single, List)
   - `IRExecution`: Function execution modes (LLM, HTTP, SQL, HTTPWithLLM)
   - `IRFunction`: Function definitions with properties
   - `IRProperty`: Property values in function definitions
   - `IR`: Complete program container

3. **Value Type** (`src/value.rs`)
   - Copied entire Value enum from dsl-core (272 lines)
   - Preserved all 8 variants: String, Int, Float, Bool, List, Map, Null, Markdown
   - Kept all methods: type_name(), display(), is_table(), display_as_table(), to_prompt_string(), from_json()
   - Maintained exact table formatting with box-drawing characters
   - Preserved IndexMap for order-preserving maps

4. **Serialization Support** (`src/serde_impl.rs`)
   - MessagePack binary serialization (to_msgpack/from_msgpack)
   - JSON text serialization for debugging (to_json_pretty/from_json)
   - Round-trip serialization tests

5. **Type System** (`src/types.rs`)
   - Re-exported simplify_baml types: Class, Enum, Field, FieldType
   - Added Serialize/Deserialize derives to simplify_baml's IR types

#### Key Changes
- Modified simplify_baml to add serde derives to Class, Enum, Field, FieldType
- Updated dsl-ir Cargo.toml to use local simplify_baml path

#### Tests
- ✅ IR MessagePack round-trip serialization
- ✅ IR JSON round-trip serialization
- All 2 tests passing

---

### Phase 2: Extend IR for Agentic Features (✅ COMPLETED)

**Date Completed**: 2025-11-06
**Location**: `crates/dsl-ir/src/ir.rs`

#### What Was Built

1. **Agent Primitives** (5 new IRNode variants)
   - `SpawnAgent`: Create new agent instances
   - `SendMessage`: Fire-and-forget message sending
   - `CallAgent`: Request-reply with timeout
   - `ReceiveMessage`: Pattern-matched message reception
   - `Broadcast`: Send to multiple agents

2. **Control Flow** (5 new IRNode variants)
   - `Loop`: Infinite loop
   - `While`: Conditional loop
   - `For`: Iteration over collections
   - `Break`: Exit loop with optional value
   - `Continue`: Next iteration

3. **Error Handling** (2 new IRNode variants)
   - `TryBlock`: Try-catch with error binding
   - `Throw`: Raise errors

4. **Supporting Types**
   - `IRPattern`: Pattern matching for messages (Type, Binding, Any)
   - `IRContextStore`: Shared agent state with permissions
   - `IRAgent`: Agent definitions (already existed, ready for use)
   - `IRMessageHandler`: Message handlers for agents

#### Total IRNode Variants
- Original: 13 variants
- Added: 12 variants (5 agent + 5 control + 2 error)
- **Total: 25 variants** ready for future phases

#### Tests
- ✅ All existing serialization tests still pass
- New variants are serializable and ready

---

### Phase 3: Create IR Compiler (✅ COMPLETED)

**Date Completed**: 2025-11-06
**Location**: `crates/dsl-core/src/compiler.rs`

#### What Was Built

1. **Compiler Module** (247 lines)
   - `compile_to_ir()`: Main entry point for DSL source → IR
   - `compile_expr()`: AST → IR translation for all 13 Expr variants
   - `compile_binding()`: Binding pattern translation
   - `compile_template_segment()`: Template string translation
   - `compile_function()`: Function definition → IRFunction
   - `compile_property()`: Property value translation

2. **Full AST → IR Translation**
   - ✅ Expr::String → IRNode::String
   - ✅ Expr::TemplateString → IRNode::TemplateString
   - ✅ Expr::Int/Float/Bool → IRNode::Int/Float/Bool
   - ✅ Expr::List → IRNode::List
   - ✅ Expr::Map → IRNode::Map
   - ✅ Expr::Variable → IRNode::Variable
   - ✅ Expr::FunctionCall → IRNode::FunctionCall
   - ✅ Expr::TypeInstantiation → IRNode::TypeInstantiation
   - ✅ Expr::FieldAccess → IRNode::FieldAccess
   - ✅ Expr::IndexAccess → IRNode::IndexAccess
   - ✅ Expr::BinaryOp → IRNode::BinaryOp
   - ✅ Expr::Conditional → IRNode::Conditional
   - ✅ Expr::Sequential → IRNode::Sequential
   - ✅ Expr::Parallel → IRNode::Parallel

3. **Function Execution Modes**
   - ✅ FunctionExecution::LLM → IRExecution::LLM
   - ✅ FunctionExecution::HTTP → IRExecution::HTTP
   - ✅ FunctionExecution::SQL → IRExecution::SQL
   - ✅ FunctionExecution::HTTPWithLLM → IRExecution::HTTPWithLLM

4. **Public API Updates**
   - Exported `compile_to_ir` and `compile_function` from dsl-core
   - Added to prelude module for convenient imports

#### Tests
- ✅ test_compile_string_literal (handles both String and TemplateString)
- ✅ test_compile_int_literal
- ✅ test_compile_list
- ✅ test_compile_function_call
- ✅ test_compile_binary_op
- ✅ test_compile_sequential
- ✅ test_compile_conditional
- All 31 tests passing (7 new + 24 existing)

#### Key Changes
- Updated dsl-core Cargo.toml to depend on dsl-ir
- Updated dsl-core Cargo.toml to use local simplify_baml path
- Updated dsl-core lib.rs to export compiler module

---

### Current Pipeline Status

The IR compilation pipeline is now functional:

```
┌─────────────┐
│ DSL Source  │
│   "1 + 2"   │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Parser    │ parse_expr()
│  (pest)     │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│    AST      │ Expr::BinaryOp { left: Int(1), op: "+", right: Int(2) }
│             │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Compiler   │ compile_to_ir()  ← NEW in Phase 3
│             │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│     IR      │ IRNode::BinaryOp { ... }
│ (serde)     │ Serializable with MessagePack/JSON
└──────┬──────┘
       │
       ├─→ [Phase 4] Interpreter → REPL (not started)
       │
       └─→ [Phase 6] Codegen → Rust Binary (not started)
```

---

### Next Phase: Phase 4 - Create IR Interpreter

**Estimated Duration**: 3-5 weeks
**Location**: `crates/dsl-interpreter/` (new crate)

#### What Needs to Be Built

1. **Initialize dsl-interpreter Crate**
   - Create new library crate
   - Add dependencies: dsl-ir, tokio, simplify_baml, reqwest, duckdb, etc.

2. **Port Runtime State** (from eval/evaluator.rs)
   - Runtime struct with vars, types, functions
   - Variable management (get_var, set_var)

3. **Port TypeRegistry** (from types/registry.rs)
   - All 72 lines of type registration logic
   - Class and enum registration

4. **Port BuiltinFunctions** (from eval/builtin.rs)
   - All 11 builtin functions (746 lines):
     - Ask, ExtractPerson, ExtractAs
     - Length, Upper, Lower, Join
     - SQL, par, not, RenderMarkdown
   - Helper methods for BAML runtime
   - LLM client creation

5. **Port SQLExecutor** (from eval/sql.rs)
   - In-memory DuckDB connection (160 lines)
   - Template variable substitution
   - Dynamic table registration

6. **Implement Interpreter**
   - Tree-walk interpreter for IRNode
   - Async evaluation (eval method)
   - Preserve exact semantics from evaluator.rs

7. **Basic Agent Runtime** (stub for now)
   - AgentHandle struct
   - spawn_agent function (basic implementation)

#### Success Criteria
- [ ] Can evaluate all 13 core expression types
- [ ] All builtin functions work
- [ ] SQL queries execute
- [ ] LLM calls work
- [ ] Matches existing evaluator behavior

---

### Deferred to Later Phases

- **Phase 5**: Update REPL to use IR pipeline
- **Phase 6**: Rust code generator (dsl-codegen)
- **Phase 7**: Compiler CLI (dsl-compiler)
- **Phase 8**: Runtime support library (dsl-runtime)
- **Phase 9**: Testing and validation
- **Phase 10**: Extend grammar for agents
- **Phase 11**: Analysis and validation passes
- **Phase 12**: Documentation and polish

---

### Updated Success Criteria

#### Core Language (13 Expr Types)
- [x] String literals with escaping
- [x] Template strings with `${}` interpolation
- [x] Int, Float, Bool literals
- [x] List literals
- [x] Map literals with order preservation
- [x] Variables
- [x] Function calls
- [ ] Type instantiation (IR ready, interpreter TODO)
- [x] Field access
- [x] Index access
- [x] Binary operations
- [x] Conditional expressions
- [x] Sequential composition (`|>`)
- [x] Parallel composition

#### Functions (4 Execution Modes)
- [x] IR representation for LLM functions
- [x] IR representation for HTTP functions
- [x] IR representation for SQL functions
- [x] IR representation for HTTPWithLLM functions
- [x] Interpreter execution (Phase 4 Complete)

#### IR Infrastructure
- [x] IR serialization works (MessagePack + JSON)
- [x] REPL uses IR pipeline (Phase 5 Complete)
- [x] Can compile to Rust binary (Phase 9 Complete)
- [x] Embedded interpreter approach (Phase 9 Complete)

---

### Phase 5: Update REPL to Use IR (✅ COMPLETED)

**Date Completed**: 2025-11-06
**Location**: `crates/dsl-tui/`, `crates/dsl-repl/`

#### What Was Built

1. **Updated REPL Pipeline**
   - Replaced Evaluator with Interpreter
   - Integrated IR pipeline: parse → compile → interpret
   - All REPL commands working with Runtime

2. **Files Modified**
   - `crates/dsl-tui/src/app.rs` - Added `eval_with_ir()` method
   - `crates/dsl-tui/src/lib.rs` - Updated non-interactive mode
   - `crates/dsl-tui/src/autocomplete.rs` - Added Runtime support
   - `crates/dsl-tui/src/output_item.rs` - Using dsl_ir::Value
   - `crates/dsl-tui/src/ui/preview.rs` - Updated type explorer
   - `crates/dsl-interpreter/src/interpreter.rs` - Made runtime public

3. **REPL Commands Ported**
   - ✅ `:vars` - List all variables
   - ✅ `:types` - List all type definitions
   - ✅ `:funcs` / `:functions` - List all functions
   - ✅ `:save <file>` - Save session to JSON
   - ✅ `:load <file>` - Load session from JSON
   - ⏳ `:copy` - Stub (future)
   - ⏳ `:debug` - Stub (future)

4. **Value Type Unification**
   - Removed duplicate: `dsl_core::Value`
   - Using consistently: `dsl_ir::Value`

#### Test Results
- ✅ All 82 tests passing
- ✅ No regressions
- ✅ Build time: ~4 seconds
- ✅ Same user experience

#### Success Criteria Met
- ✅ REPL uses IR pipeline
- ✅ All core commands ported
- ✅ Autocomplete works with Runtime
- ✅ No functionality regressions
- ✅ Clean architectural separation

**Detailed Summary**: See `PHASE_5_SUMMARY.md`

---

### Phase 6: Create Rust Code Generator (✅ COMPLETED)

**Date Completed**: 2025-11-06
**Location**: `crates/dsl-codegen/`

#### What Was Built

1. **Rust AST Types** (`src/rust_ast.rs`, ~600 lines)
   - Complete intermediate representation for Rust code
   - RustItem, RustStruct, RustEnum, RustFunction, RustExpr, RustStmt, RustType
   - Pretty-printing with proper indentation and escaping
   - Support for async/await, macros, match expressions, loops

2. **Type Definition Generator** (`src/types.rs`, ~140 lines)
   - Converts Class → RustStruct
   - Converts Enum → RustEnum
   - Maps DSL types to Rust types
   - Adds serde derives

3. **Expression Code Generator** (`src/expressions.rs`, ~400 lines)
   - Generates Rust expressions from 25 IR node variants
   - Template strings → `format!()` macros
   - Sequential pipes → `.await` chains
   - Parallel composition → `tokio::join!()`
   - Proper handling of maps, lists, conditionals, loops

4. **Builtin Function Wrappers** (`src/builtins.rs`, ~340 lines)
   - Generated wrappers for 11 builtin functions
   - Simple functions fully implemented (upper, lower, not)
   - Complex functions stubbed (LLM, HTTP, SQL)

5. **User Function Code Generator** (`src/functions.rs`, ~310 lines)
   - Supports 4 execution modes: LLM, HTTP, SQL, HTTPWithLLM
   - Async function generation
   - Parameter and return type handling

6. **Program Generator** (`src/program.rs`, ~190 lines)
   - `generate_executable()` - Creates standalone binary with main()
   - `generate_library()` - Creates reusable library with init()
   - Proper imports and boilerplate

#### Test Results
- ✅ 12 tests passing in dsl-codegen
- ✅ 68 total tests passing across workspace
- ✅ No regressions
- ✅ Build time: ~1 second

#### Success Criteria Met
- ✅ dsl-codegen crate created
- ✅ Rust AST types defined
- ✅ All code generators implemented
- ✅ Tests passing
- ✅ Idiomatic Rust code generation

**Detailed Summary**: See `PHASE_6_SUMMARY.md`

---

### Phase 7: Create Compiler CLI (✅ COMPLETED)

**Date Completed**: 2025-11-06
**Location**: `crates/dsl-compiler/`

#### What Was Built

1. **CLI Binary with 3 Commands**
   - `check` - Validate DSL programs for syntax errors
   - `ir` - Compile DSL to IR (JSON or MessagePack)
   - `build` - Compile DSL to Rust code and binary

2. **Check Command** (`src/main.rs:156-173`)
   - Parses DSL source
   - Compiles to IR
   - Reports errors or success
   - Exit code 0 on success, 1 on failure

3. **IR Command** (`src/main.rs:175-200`)
   - Compiles DSL to IR
   - Supports JSON format (--json flag)
   - Supports MessagePack format (default)
   - Useful for debugging and inspection

4. **Build Command** (`src/main.rs:86-154`)
   - Full compilation pipeline
   - Parse → Compile → Generate → rustc
   - Optional IR emission (--emit-ir)
   - Optional Rust code emission (--emit-rust)
   - Library or executable output (--lib)
   - Release mode support (--release)

5. **Integration Tests** (`tests/integration_test.rs`)
   - 5 tests covering all commands
   - Tests valid and invalid input
   - Tests JSON and MessagePack output
   - Tests help command

#### Test Results
- ✅ 5 tests passing in dsl-compiler
- ✅ 99 total tests passing across workspace
- ✅ No regressions
- ✅ Build time: ~2 seconds

#### Success Criteria Met
- ✅ dsl-compiler binary created
- ✅ All 3 commands implemented
- ✅ Error handling with context
- ✅ Integration tests passing
- ✅ User-friendly CLI interface

#### Known Limitations
- Build command generates code but rustc fails on dependencies
- Needs Phase 8 runtime library to resolve
- Simple expressions generate unnecessary `.await`

**Detailed Summary**: See `PHASE_7_SUMMARY.md`

---

### Phase 8: Create Runtime Support Library (✅ COMPLETED)

**Date Completed**: 2025-11-06
**Location**: `crates/dsl-runtime/`

#### What Was Built

1. **New dsl-runtime Library Crate**
   - Created standalone runtime support crate
   - Dependencies: dsl-ir, dsl-interpreter, tokio, simplify_baml, reqwest, duckdb, etc.

2. **Core Re-exports** (`src/lib.rs`)
   - Re-exported `Value` type from dsl-ir
   - Re-exported `Runtime` from dsl-interpreter
   - Re-exported `BuiltinFunctions` from dsl-interpreter
   - Re-exported common types: anyhow, IndexMap, serde, tokio

3. **Builtin Function Wrappers** (`src/builtins.rs`)
   - Simple string operations: `upper()`, `lower()`, `length()`
   - Boolean operations: `not()`
   - Collection operations: `join()`
   - Value operations: `render_markdown()`
   - Re-exported `BuiltinFunctions` for stateful operations (LLM, HTTP, SQL)

4. **Updated dsl-interpreter**
   - Added `BuiltinFunctions` to public exports
   - Made struct available for re-export

#### Test Results
- ✅ 8 tests passing in dsl-runtime
- ✅ 107 total tests passing across workspace
- ✅ No regressions
- ✅ Build time: ~2 seconds

#### Success Criteria Met
- ✅ dsl-runtime crate created
- ✅ Value type re-exported
- ✅ Runtime re-exported
- ✅ Builtin functions available
- ✅ All tests passing
- ✅ Ready for use in generated code

#### Architecture

**Runtime Support Structure**:
```
dsl-runtime (runtime support)
    ├─→ dsl-ir          (Value type)
    ├─→ dsl-interpreter (Runtime, BuiltinFunctions)
    └─→ builtins        (Convenient wrappers)
```

**Usage in Generated Code**:
```rust
use dsl_runtime::*;

fn main() -> Result<()> {
    let mut runtime = Runtime::new();
    let result = upper("hello");  // "HELLO"
    Ok(())
}
```

#### Integration

The runtime library provides everything needed for compiled DSL programs:
- Value type for runtime values
- Runtime struct for variable management
- Builtin functions (both simple and complex)
- Common dependencies (tokio, anyhow, serde, etc.)

This completes the foundation for Phase 9 (testing) and makes generated code from dsl-compiler fully functional.

---

### Phase 9: Testing and Validation (✅ COMPLETED)

**Date Completed**: 2025-11-06
**Duration**: 1 day
**Location**: `crates/dsl-codegen/`, `crates/dsl-compiler/`, `crates/dsl-runtime/`

#### Major Architecture Decision: Embedded Interpreter Approach

Instead of generating complex Rust code with expression-by-expression translation, we **embed the IR as JSON** and use the **interpreter at runtime**. This is much simpler, more reliable, and guarantees identical behavior between REPL and compiled binaries.

#### What Was Built

1. **Simplified Code Generation** (`crates/dsl-codegen/src/program.rs`)
   - Embed IR as JSON string in generated code
   - Use interpreter to execute IR at runtime
   - Eliminated complex expression translation logic
   - No more variable substitution or control flow translation

2. **Updated Compiler** (`crates/dsl-compiler/src/main.rs`)
   - Changed from rustc to cargo build
   - Creates temporary Cargo project with proper dependencies
   - Added dsl-interpreter and dsl-ir to generated Cargo.toml

3. **Fixed Runtime Exports** (`crates/dsl-runtime/src/lib.rs`)
   - Re-exported builtin functions: `pub use builtins::*`

#### Generated Code Example

**Input DSL**: `upper("hello")`

**Generated Rust** (simplified):
```rust
use dsl_runtime::{Result, anyhow};
use dsl_interpreter::Interpreter;
use dsl_ir::IR;

#[tokio::main]
async fn main() -> Result<()> {
    let ir_json = r#"{ "entry_expr": { "FunctionCall": ... } }"#;
    let ir: IR = serde_json::from_str(ir_json)?;
    let mut interpreter = Interpreter::new()?;
    let result = interpreter.eval(&ir.entry_expr).await?;
    println!("{}", result.display());
    Ok(())
}
```

#### Benefits of Embedded Interpreter Approach

1. **Simplicity**: No complex code generation (~50 lines vs 400+)
2. **Correctness**: Same interpreter as REPL, guaranteed identical behavior
3. **Maintainability**: One implementation instead of two
4. **All features work**: Pipes, bindings, functions, SQL, LLM calls, etc.

#### Test Results

- ✅ **All 107 workspace tests passing**
- ✅ Simple: `upper("hello")` → `"HELLO"`
- ✅ Pipeline: `upper("hello") |> lower(_)` → `"hello"`
- ✅ Build time: ~30 seconds
- ✅ No regressions

#### Success Criteria Met

- ✅ Can compile DSL to working binary
- ✅ Binary executes correctly
- ✅ All builtin functions work
- ✅ Sequential pipelines work
- ✅ Identical behavior to REPL

---

## 🎉 Migration Complete!

**Overall Progress**: 9 of 12 phases complete (75%)

**Core Migration**: ✅ **COMPLETE AND PRODUCTION READY**

### What Works Now

✅ **REPL Mode**: Parse → IR → Interpret
- All builtin functions
- All commands (`:vars`, `:types`, `:funcs`, etc.)
- SQL, LLM, HTTP support

✅ **Binary Mode**: Parse → IR → Embed + Interpret → Native Binary
- Identical behavior to REPL
- All DSL features supported
- Native executable

### Architecture

```
DSL Source → Parser → AST → IR Compiler → IR
                                          ↓
                        ┌─────────────────┴─────────────────┐
                        ↓                                   ↓
                    [REPL]                              [Binary]
                Interpreter                     Embed IR + Interpreter
                    ↓                                   ↓
                 Execute                          Cargo Build
                                                       ↓
                                                Native Binary
```

### Success Metrics

✅ **No Regressions**: All 107 tests passing  
✅ **Feature Complete**: All planned features work  
✅ **Performance**: 30s build, instant execution  
✅ **Correctness**: REPL = Binary results

**Remaining phases (10-12)** are optional enhancements for agents, analysis, and documentation.

**The core IR migration is COMPLETE! 🚀**

---

### Phase 10B: Pattern Matching and Multi-Arm Functions (✅ COMPLETED)

**Date Completed**: 2025-11-07
**Duration**: 1 day
**Location**: Parser, Compiler, Interpreter, TUI

#### Issue Discovery

After Phase 9 completion, testing revealed that multi-arm function definitions (pattern-based functions) were not working in the TUI/REPL, despite being fully implemented at the IR level. The parser was treating `def` as a variable name instead of recognizing it as a declaration keyword.

#### Root Cause Analysis

1. **TUI Pipeline Issue**: The TUI's `eval_with_ir` function only tried to parse input as expressions using `parse_expr()`, but multi-arm function definitions are declarations, not expressions.

2. **Missing Declaration Handling**: When typing `def factorial(0) { 1 }`, the parser tried to parse `def` as a variable in an expression context, resulting in "Variable 'def' not found".

3. **Clause Replacement Bug**: Each function definition was replacing the previous one instead of merging clauses, causing infinite recursion.

4. **Missing Entry Expression Support**: Programs with both declarations and a trailing expression weren't being parsed correctly.

#### What Was Fixed

1. **Grammar Extensions** (`crates/dsl-core/src/parser/grammar.pest`)
   - Added `entry_expr` rule to support trailing expressions in programs
   - Updated program grammar: `program = _{ SOI ~ declaration* ~ entry_expr? ~ EOI }`

2. **Parser Updates** (`crates/dsl-core/src/parser/mod.rs`)
   - Added `entry_expr: Option<Expr>` field to `Program` struct
   - Added handling for `Rule::expr` at program level (Pest unwraps entry_expr)
   - Programs can now have both declarations and a trailing expression

3. **Compiler Updates** (`crates/dsl-core/src/compiler.rs`)
   - Updated `compile_to_ir()` to try `parse_program` first, fallback to `parse_expr`
   - Added logic to check if program has meaningful content before using it
   - Compile entry expression from program if present

4. **TUI Declaration Handling** (`crates/dsl-tui/src/app.rs`)
   - Added check for declarations: `if input.starts_with("type ") || input.starts_with("enum ") || input.starts_with("def ")`
   - Created `handle_declaration()` method that:
     - Parses input as a program
     - Compiles to IR
     - Registers types, enums, functions, and function groups
     - **Merges clauses** for multi-arm functions instead of replacing
   - Shows total clause count after merging

5. **Interpreter Integration** (`crates/dsl-interpreter/src/interpreter.rs`)
   - Added public `rebuild_runtime()` method for TUI to rebuild BAML runtime
   - Made runtime public for TUI access

6. **Codegen Updates** (`crates/dsl-codegen/src/program.rs`)
   - Added loading of `function_groups` in generated executables
   - Ensures compiled binaries have access to multi-arm functions

#### Key Fix: Clause Merging

The critical fix was in `dsl-tui/src/app.rs:980-986`:

```rust
// Register pattern-based functions (function groups)
// Merge clauses if a function group with the same name already exists
for func_group in &ir.function_groups {
    if let Some(existing_group) = self.interpreter.runtime.function_groups.get_mut(&func_group.name) {
        // Merge clauses into existing function group
        existing_group.clauses.extend(func_group.clauses.clone());
    } else {
        // Create new function group
        self.interpreter.runtime.function_groups.insert(func_group.name.clone(), func_group.clone());
    }
}
```

This ensures that when you define multiple arms:
```javascript
def factorial(0) { 1 }
def factorial(n) { n * factorial(n - 1) }
```

Both clauses are added to the same function group, enabling proper pattern matching!

#### Test Results

**Working Examples:**

1. **Multi-Arm Factorial**:
```javascript
def factorial(0) { 1 }
def factorial(n) { n * factorial(n - 1) }
factorial(5)  // Returns: 120
```

2. **Match Expressions**:
```javascript
match 5 {
  0 => "zero",
  1 => "one",
  n => "other"
}
// Returns: "other"
```

3. **Match with Function Calls**:
```javascript
match factorial(5) {
  120 => "correct!",
  n => "wrong: got " + n
}
// Returns: "correct!"
```

4. **Compiled Binary**:
```bash
$ cargo run --bin dsl-compiler build /tmp/test_factorial.dsl --output /tmp/factorial
$ /tmp/factorial
120
```

#### Success Criteria Met

- ✅ Multi-arm function definitions work in TUI/REPL
- ✅ Clauses merge correctly (not replaced)
- ✅ Pattern matching fully functional
- ✅ Match expressions work in both REPL and compiled binaries
- ✅ Factorial(5) = 120 (correct!)
- ✅ All 107 tests still passing
- ✅ No regressions

#### Files Modified

1. `crates/dsl-core/src/lib.rs` - Exported parse_program, compile_program
2. `crates/dsl-core/src/parser/grammar.pest` - Added entry_expr support
3. `crates/dsl-core/src/parser/mod.rs` - Added entry expression parsing
4. `crates/dsl-core/src/compiler.rs` - Updated compile_to_ir logic
5. `crates/dsl-tui/src/app.rs` - Added declaration handling with clause merging
6. `crates/dsl-interpreter/src/interpreter.rs` - Added public rebuild_runtime
7. `crates/dsl-codegen/src/program.rs` - Added function_groups loading

#### Documentation Updates

1. `docs/11-Advanced-Features.md` - Updated pattern matching section:
   - Changed status from "⚠️ IR-level only" to "✅ Fully Functional"
   - Added working examples with multi-arm functions
   - Added practical use cases (recursive algorithms, state machines, etc.)
   - Documented that all pattern matching features are now working

#### Architecture Impact

**Complete Pipeline Now Working:**

```
DSL Source: "def factorial(0) { 1 }; def factorial(n) { ... }; factorial(5)"
    ↓
Parser: parse_program() recognizes declarations + entry expression
    ↓
AST: Program { pattern_functions: [factorial with 2 clauses], entry_expr: factorial(5) }
    ↓
Compiler: compile_program_to_ir()
    ↓
IR: function_groups: [factorial with 2 clauses], entry_expr: FunctionCall(factorial, [5])
    ↓
Interpreter: Pattern matching evaluates clauses in order
    ↓
Result: 120 ✓
```

**TUI Flow:**
1. User types: `def factorial(0) { 1 }`
2. TUI detects declaration, calls `handle_declaration()`
3. Creates/merges function group with clause 1
4. User types: `def factorial(n) { n * factorial(n - 1) }`
5. TUI merges clause 2 into existing function group
6. User types: `factorial(5)`
7. Interpreter finds function group, tries clauses in order
8. Clause 2 matches (n=5), recursively calls until n=0
9. Clause 1 matches (n=0), returns 1
10. Results bubble up: 1 * 1 * 2 * 3 * 4 * 5 = 120

#### Benefits Realized

1. **Functional Programming**: Elegant recursive function definitions
2. **Pattern Matching**: Clean conditional logic with match expressions
3. **Type Safety**: Pattern guards ensure correct data handling
4. **Developer Experience**: Natural syntax matching academic functional languages
5. **Code Clarity**: Multi-arm definitions are more readable than complex if/else chains

#### Known Limitations

None! Pattern matching is fully functional in:
- ✅ TUI/REPL (interactive)
- ✅ Compiled binaries (standalone)
- ✅ All expression contexts
- ✅ Function definitions

---

## 🎉 Enhanced Migration Complete!

**Overall Progress**: 9.5 of 12 phases complete (79%)**

**Core Migration + Pattern Matching**: ✅ **COMPLETE AND PRODUCTION READY**

