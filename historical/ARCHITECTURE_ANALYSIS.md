# DSL Compiler Architecture Analysis
## Feasibility of DSL → IR → Target Language Restructuring

**Date:** November 5, 2025  
**Status:** Detailed Analysis Complete  
**Conclusion:** IR layer can be partially added with moderate effort; full three-stage pipeline already partially present

---

## Executive Summary

The DSL compiler currently uses a **direct AST-to-Value execution model** (DSL → AST → Runtime Value), NOT a traditional DSL → Target Language compilation flow. However, an **intermediate representation (IR) layer already exists partially** through the BAML runtime integration and type system.

**Key Findings:**
- No traditional code generation to target languages currently exists
- An IR-like layer exists through `simplify_baml::IR` for LLM-based function definitions
- The evaluator is **tightly coupled to the AST** but can be refactored
- Introducing a true IR stage between AST and execution would require **moderate restructuring**
- The architecture is relatively clean for inserting IR transformations

---

## 1. Current Compilation Pipeline Analysis

### 1.1 Code Flow: DSL → Execution

```
┌─────────────────────────────────────────────────────────────┐
│                    DSL Source Code                           │
│                    (e.g., "5 |> _ * 2")                     │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│        PARSING STAGE (crates/dsl-core/src/parser)            │
│  - File: src/parser/mod.rs                                  │
│  - Grammar: src/parser/grammar.pest (Pest parser)           │
│  - Output: AST (Expr enum)                                  │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│       EVALUATION STAGE (crates/dsl-core/src/eval)            │
│  - File: src/eval/evaluator.rs (Evaluator struct)           │
│  - Direct AST interpretation (no IR layer)                  │
│  - Output: Value (runtime value)                            │
│  - Subtasks:                                                 │
│    • Template interpolation                                 │
│    • Binary operations                                      │
│    • Function calls (user-defined & builtin)                │
│    • Control flow (conditionals, sequences)                 │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│     FUNCTION EXECUTION (Builtin & User-Defined)              │
│  - File: src/eval/builtin.rs                                │
│  - Execution types:                                          │
│    • LLM (via BAML runtime)                                 │
│    • HTTP (via reqwest)                                      │
│    • SQL (via DuckDB)                                        │
│    • Hybrid (HTTP then LLM)                                 │
│  - Output: Value (result)                                   │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                   Final Output Value                         │
│              (String, Int, Float, List, Map, etc.)          │
└─────────────────────────────────────────────────────────────┘
```

**Key Characteristic:** This is **interpretation, NOT compilation**. Code is executed directly from the AST without generating intermediate code.

---

## 2. Intermediate Representations Currently Present

### 2.1 Existing IR Layer: BAML IR (Partial)

**Location:** Integrated through `simplify_baml` crate dependency

**Purpose:** Used for function definitions with structured output types

**Structure:**
```rust
// From simplify_baml crate (external dependency)
pub struct IR {
    pub classes: Vec<Class>,    // User-defined types
    pub enums: Vec<Enum>,       // User-defined enums  
    pub functions: Vec<Function>, // Function definitions
}

pub struct Function {
    pub name: String,
    pub inputs: Vec<Field>,
    pub output: FieldType,
    pub prompt_template: String,
    pub client: String,
}
```

**Where It's Built:**
- File: `src/eval/builtin.rs` (line 100+)
- Method: `BuiltinFunctions::rebuild_runtime(&mut self, type_registry: &TypeRegistry)`
- Triggered when: New types/enums are registered

**Current Usage:**
- Only used for LLM function execution
- Not used for HTTP or SQL execution paths
- Not exposed as a general-purpose compilation stage

### 2.2 AST Layer (Current Primary IR)

**Location:** `src/parser/mod.rs`

**Structure:**
```rust
#[derive(Debug, Clone)]
pub enum Expr {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    List(Vec<Expr>),
    Map(Vec<(String, Expr)>),
    Variable(String),
    FunctionCall { name: String, args: Vec<Expr> },
    TypeInstantiation { type_name: String, fields: Vec<(String, Expr)> },
    FieldAccess { base: Box<Expr>, field: String },
    IndexAccess { base: Box<Expr>, index: Box<Expr> },
    BinaryOp { left: Box<Expr>, op: String, right: Box<Expr> },
    Conditional { condition: Box<Expr>, then_expr: Box<Expr>, else_expr: Box<Expr> },
    Sequential { left: Box<Expr>, right: Box<Expr>, binding: Option<Binding> },
    Parallel { exprs: Vec<Expr>, binding: Option<Binding> },
}
```

**Characteristics:**
- Rich AST with all language constructs
- Well-structured for direct evaluation
- Already includes composition operators (Sequential, Parallel)
- Bindings attached directly to AST nodes

### 2.3 Type System (Secondary IR)

**Location:** `src/types/mod.rs` and `src/types/registry.rs`

**Components:**
```rust
pub struct TypeRegistry {
    classes: HashMap<String, Class>,  // From simplify_baml
    enums: HashMap<String, Enum>,     // From simplify_baml
}

pub enum Value {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    List(Vec<Value>),
    Map(IndexMap<String, Value>),
    Null,
    Markdown(String),
}
```

**Purpose:**
- Runtime value representation
- Type tracking during evaluation
- Bridges to BAML IR for LLM operations

---

## 3. Code Generation Analysis

### 3.1 Current Code Generation Status

**CONCLUSION: No code generation to target languages exists.**

The system does NOT currently:
- Generate Python, JavaScript, or other language code
- Compile to bytecode or machine code
- Generate intermediate execution graphs

Instead, it:
- **Interprets the AST directly** through `Evaluator::eval_expr()`
- **Delegates to external runtimes**: LLM (BAML), HTTP (reqwest), SQL (DuckDB)
- **Produces Values** that are displayed/returned to the user

### 3.2 Coupling Analysis: AST to Evaluation

**Tight Coupling Points:**

1. **Direct Pattern Matching in Evaluator**
   - File: `src/eval/evaluator.rs` (line ~160-500)
   - Method: `async fn eval_expr(&mut self, expr: &Expr) -> Result<Value, String>`
   - Every AST variant has a corresponding evaluation branch
   - Example:
     ```rust
     match expr {
         Expr::BinaryOp { left, op, right } => {
             let left_val = Box::pin(self.eval_expr(left)).await?;
             let right_val = Box::pin(self.eval_expr(right)).await?;
             self.apply_op(&left_val, op, &right_val)
         }
         Expr::Sequential { left, right, binding } => { ... }
         // ... 10+ other variants
     }
     ```
   - **Coupling Level: TIGHT** - Evaluator must know about all Expr variants

2. **Function Execution Hard-Coded**
   - File: `src/eval/evaluator.rs` (line ~350+)
   - Methods: `execute_llm_function()`, `execute_http_function()`, `execute_sql_function()`
   - String interpolation and parameter binding happen during execution
   - **Coupling Level: MODERATE** - Could be abstracted

3. **Variable & Type Resolution at Runtime**
   - File: `src/eval/evaluator.rs`
   - Variables resolved in `eval()` at execution time
   - Type checking not pre-computed
   - **Coupling Level: TIGHT** - No pre-compilation type analysis

### 3.3 Separation Analysis

**What IS Separated:**

- ✅ **Parser from Evaluator**: Clean boundary
  - Parser produces `Expr` enum
  - Evaluator consumes `Expr` without parsing concerns
  
- ✅ **Type System from Execution**: Moderate separation
  - `TypeRegistry` maintains types independently
  - Used by both BAML IR builder and evaluator
  
- ✅ **Builtin Functions from Core Evaluator**: Good separation
  - `BuiltinFunctions` struct handles all builtins
  - Evaluator calls `builtins.call(name, args)`
  - Could easily extend with new builtin implementations

**What IS NOT Separated:**

- ❌ **AST Variants from Evaluation Logic**: Tightly coupled
  - New AST construct requires new eval branch
  - No intermediate compilation phase
  
- ❌ **Function Type Resolution from Execution**: Tightly coupled
  - `FunctionExecution` enum determined at parse time
  - Execution logic in evaluator for each variant
  - No separate transformation/optimization pass

---

## 4. Current Optimization & Analysis Passes

### 4.1 Existing Passes

**None.** The system currently has:
- No constant folding
- No dead code elimination
- No type inference
- No optimization passes
- No analysis passes

The code goes directly from AST → Execution.

### 4.2 Implicit Analysis During Execution

Some analysis happens implicitly:
- Variable scoping (via `vars: HashMap`)
- Type coercion (automatic Int→Float promotion in arithmetic)
- Field validation (at runtime when accessing maps)

---

## 5. Proposed Three-Stage Pipeline

### 5.1 Current vs. Proposed Architecture

**Current (Interpretation):**
```
DSL Source → Pest Parser → AST (Expr) → Evaluator → Values
```

**Proposed (Compilation with IR):**
```
DSL Source → Pest Parser → AST (Expr) → IR Translation → IR (Tasks) → Execution Engine → Values
```

### 5.2 What a New IR Layer Could Represent

**Option A: Execution Plan IR (Recommended)**

```rust
// New module: src/compiler/ir.rs

#[derive(Debug, Clone)]
pub enum IRTask {
    // Literals
    Literal(Value),
    
    // Variables
    LoadVar(String),
    StoreVar(String, Box<IRTask>),
    
    // Operations
    BinaryOp {
        op: String,
        left: Box<IRTask>,
        right: Box<IRTask>,
    },
    
    // Function calls with explicit type info
    Call {
        func_name: String,
        args: Vec<IRTask>,
        return_type: Option<FieldType>,
    },
    
    // Sequencing (pipe operator)
    Sequence {
        tasks: Vec<IRTask>,
        bindings: Vec<(String, usize)>, // variable name → task index
    },
    
    // Parallel execution
    Parallel {
        tasks: Vec<IRTask>,
    },
    
    // Control flow
    If {
        condition: Box<IRTask>,
        then_task: Box<IRTask>,
        else_task: Box<IRTask>,
    },
    
    // Array/map access
    Index {
        base: Box<IRTask>,
        index: Box<IRTask>,
    },
    FieldAccess {
        base: Box<IRTask>,
        field: String,
    },
}

pub struct CompiledProgram {
    pub main_task: IRTask,
    pub type_info: TypeInfo,
    pub required_functions: Vec<String>,
}
```

**Benefits:**
- Can be analyzed/optimized before execution
- Can be inspected for debugging
- Can be serialized for caching
- Foundation for code generation to other languages
- Enables compilation to bytecode/WAT/etc.

**Option B: Dataflow Graph IR**

Alternative: Build an explicit dataflow graph showing dependencies:

```rust
pub struct DataflowGraph {
    pub nodes: Vec<DataflowNode>,
    pub edges: Vec<(NodeId, NodeId)>,
}

pub enum DataflowNode {
    Source(String),              // Input variable
    Operation(String),           // Binary op, function call
    Sink(String),                // Output variable
}
```

**Advantages:** Better for optimization and parallel execution analysis.

---

## 6. Difficulty Assessment: IR Layer Insertion

### 6.1 Estimated Effort Breakdown

| Component | Difficulty | Effort | Notes |
|-----------|-----------|--------|-------|
| **Define IR AST** | Easy | 2-3 days | Well-defined structure, mostly straightforward |
| **AST → IR Compiler** | Easy-Medium | 3-4 days | Straightforward transformation (no optimizations) |
| **IR Validator/Analyzer** | Medium | 2-3 days | Type checking, variable resolution, function lookup |
| **IR → Execution Engine** | Medium | 4-5 days | Replace current `eval_expr()` with IR interpreter |
| **Testing & Refactoring** | Medium | 3-4 days | Ensure all current tests pass |
| **Optimization Passes** | Medium-Hard | 5-7 days | Optional: constant folding, dead code elimination |
| **Code Generation** | Hard | 10-20 days | Generate Python/JS/etc (requires full language knowledge) |
| **TOTAL (IR only)** | **Easy-Medium** | **14-19 days** | Feasible in 2-3 weeks |
| **TOTAL (IR + codegen)** | **Hard** | **25-40+ days** | Feasible but requires more expertise |

### 6.2 Risk Assessment

**Low Risk:**
- ✅ Parser is already separate and well-tested
- ✅ Evaluator is in one file (easy to understand)
- ✅ No performance-critical code paths currently
- ✅ Existing test suite can validate changes

**Medium Risk:**
- ⚠️ Builtin functions are diverse (LLM, HTTP, SQL)
- ⚠️ Type system is simple (no generics)
- ⚠️ Variable scoping could be complex if functions need closure support

**High Risk:**
- ❌ None identified for IR-only refactoring
- ❌ Code generation would require careful design

---

## 7. Key Modules & Code Flow Details

### 7.1 Parser Module Structure

**File:** `crates/dsl-core/src/parser/mod.rs` (1200+ lines)

**Key Types:**
- `Expr` enum - 14 variants covering all language constructs
- `Binding` enum - Variable binding patterns
- `FunctionDef` struct - User-defined function definitions
- `FunctionExecution` enum - 4 execution modes (LLM, HTTP, SQL, HTTPWithLLM)
- `PropertyValue` enum - Configuration values

**Key Functions:**
- `parse_expr(input: &str) -> Result<Expr>` - Main entry point
- `build_expr(pair) -> Result<Expr>` - Recursive AST builder
- `parse_function_definition(input)` - Function parser
- `parse_type_definition(input)` - Type parser

**Grammar File:** `src/parser/grammar.pest`
- Comprehensive PEG grammar with operator precedence
- Clear separation: literals, expressions, declarations
- No significant gaps or missing constructs

### 7.2 Evaluator Module Structure

**File:** `crates/dsl-core/src/eval/evaluator.rs` (650+ lines)

**Core Components:**

```
Evaluator struct:
├── vars: HashMap<String, Value>           // Variable store
├── types: TypeRegistry                    // Type definitions
├── functions: HashMap<String, FunctionDef> // User functions
└── builtins: BuiltinFunctions             // Built-in execution engine

Key Methods:
├── pub async fn eval(&mut self, input) -> Result<(Value, Option<String>)>
│   ├── Handles commands (:vars, :types, etc)
│   ├── Handles type definitions (type X { ... })
│   ├── Handles enum definitions (enum X { ... })
│   ├── Handles function definitions (def f { ... })
│   └── Parses & evaluates expressions
│
└── async fn eval_expr(&mut self, expr: &Expr) -> Result<Value>
    ├── Literal evaluation (String, Int, Float, Bool)
    ├── List/Map construction
    ├── Variable resolution
    ├── Binary operations (arithmetic, logical, comparison)
    ├── Function calls (builtin & user-defined)
    ├── Field/index access
    ├── Control flow (conditionals, sequences, parallels)
    └── Template string interpolation

Supporting Methods:
├── async fn call_user_function(&mut self, func_def, args)
├── async fn execute_llm_function(...)
├── async fn execute_http_function(...)
├── async fn execute_sql_function(...)
├── async fn execute_llm_with_input(...)
├── fn interpolate_string_template(...)
└── fn apply_op(&self, left, op, right) -> Result<Value>
```

**Data Flow in eval_expr:**
1. **Pattern match on Expr variant**
2. **Recursively evaluate sub-expressions**
3. **Apply operation-specific logic**
4. **Return Value result**

**Example: Sequential operator (pipe)**
```rust
Expr::Sequential { left, right, binding } => {
    // 1. Evaluate left expression
    let left_result = Box::pin(self.eval_expr(left)).await?;
    
    // 2. Store as _ and binding variables
    self.vars.insert("_".to_string(), left_result.clone());
    if let Some(bind) = binding {
        match bind {
            Binding::Single(name) => 
                self.vars.insert(name.clone(), left_result.clone()),
            Binding::List(names) => 
                // Destructure left_result into named variables
        }
    }
    
    // 3. Evaluate right expression (can reference _ or bindings)
    Box::pin(self.eval_expr(right)).await
}
```

### 7.3 Builtin Functions Module

**File:** `crates/dsl-core/src/eval/builtin.rs` (400+ lines)

**Structure:**
```rust
pub struct BuiltinFunctions {
    runtime: Option<BamlRuntime>,        // BAML IR for LLM
    llm_client: Option<LLMClient>,       // Direct LLM access
    api_key: Option<String>,             // API key
    sql_executor: Option<SQLExecutor>,   // DuckDB SQL
    last_prompt: Option<String>,         // Debug info
}

Key Methods:
├── pub async fn call(&mut self, name, args) -> Result<Value>
│   └── Routes to specific builtin implementation
│
├── pub fn rebuild_runtime(&mut self, type_registry) -> Result<()>
│   └── Builds BAML IR from type registry
│
├── async fn ask(&mut self, args) -> Result<Value>
│   └── Simple LLM prompt
│
├── async fn extract_as(&mut self, args) -> Result<Value>
│   └── Structured LLM extraction
│
├── fn sql(&mut self, args) -> Result<Value>
│   └── DuckDB SQL execution
│
└── fn length/upper/lower/join/etc()
    └── String/list manipulation
```

### 7.4 Type System Module

**File:** `crates/dsl-core/src/types/`

```
types/
├── mod.rs           - TypeRegistry, Value enum
├── registry.rs      - TypeRegistry implementation
└── value.rs         - Value display/formatting
```

**TypeRegistry:**
- Stores user-defined classes and enums
- Can export to BAML IR format
- No optimization or analysis currently

**Value enum:**
- 8 variants: String, Int, Float, Bool, List, Map, Null, Markdown
- Includes display logic and table formatting
- No type schema metadata (only type names)

---

## 8. Specific Recommendations for IR Layer

### 8.1 Recommended Approach

**Step 1: Create IR Module** (2-3 days)
```
crates/dsl-core/src/compiler/
├── mod.rs
├── ir.rs              # New: IRTask enum and structures
└── compiler.rs        # New: AST → IR translation
```

**Step 2: Implement Compiler** (3-4 days)
```rust
// AST → IR transformation
pub fn compile_expr(expr: &Expr, context: &CompileContext) -> Result<IRTask> {
    match expr {
        Expr::Literal(val) => Ok(IRTask::Literal(val.clone())),
        Expr::BinaryOp { left, op, right } => {
            let left_ir = compile_expr(left, context)?;
            let right_ir = compile_expr(right, context)?;
            Ok(IRTask::BinaryOp {
                op: op.clone(),
                left: Box::new(left_ir),
                right: Box::new(right_ir),
            })
        }
        // ... etc for each variant
    }
}
```

**Step 3: Refactor Evaluator** (4-5 days)
- Keep current `Evaluator::eval()` interface
- Internally: parse → compile → interpret IR
- Gradually migrate eval logic to IR interpreter

**Step 4: Add Analyzer** (2-3 days)
```rust
// Optional: Static analysis on IR
pub struct IRAnalyzer;
impl IRAnalyzer {
    pub fn analyze(&self, ir: &IRTask) -> Result<TypeInfo>;
    pub fn find_undefined_vars(&self, ir: &IRTask) -> Vec<String>;
    pub fn find_unused_vars(&self, ir: &IRTask) -> Vec<String>;
}
```

### 8.2 Where to Start

**Least Disruptive Entry Point:**

1. **Start with simple expressions** (numbers, strings, operators)
   - No function calls, no variable binding
   - Can run IR interpreter alongside current evaluator for testing
   
2. **Add function calls next**
   - Keep existing function execution logic
   - Just wrap in IR Call node
   
3. **Add sequencing/binding last**
   - Most complex part
   - Can refactor variable scoping more thoroughly

**Safest Testing Strategy:**
- Add IR compilation as optional debug feature
- Keep existing evaluator as primary path
- Compare outputs: `evaluator.eval(expr)` vs `ir_eval(compile(expr))`
- Gradually migrate to IR-only execution

---

## 9. Challenges & Mitigations

### 9.1 Key Challenges

| Challenge | Impact | Mitigation |
|-----------|--------|-----------|
| **Variable Scoping** | Medium | Design IR scoping carefully; maintain current HashMap-based approach |
| **Function Type Resolution** | Medium | Pre-compute function signatures in analyzer pass |
| **Async Execution** | Medium | IR must encode async/await boundaries; Rust futures handle rest |
| **Binding Patterns** | Low | Destructuring already handled; translate to IR stores |
| **Template Interpolation** | Low | Happens at compile time in new approach |
| **BAML Integration** | Medium | IR compiler recognizes LLM/HTTP functions and emits Call nodes |

### 9.2 Why the Current Architecture is Friendly to IR Addition

✅ **Strengths:**
- Parser already separate and clean
- AST is well-structured (not deeply nested)
- Evaluator is one file (easy to understand and modify)
- No complex flow analysis needed
- Variable scoping is simple (flat HashMap)
- No lambda functions or closures to worry about

❌ **Weaknesses:**
- Implicit type coercion (Int→Float) happens at runtime
- No type schema for complex structures
- Function definitions stored as data (not validated at definition time)

---

## 10. Path to Code Generation (Future)

### 10.1 How IR Enables Code Generation

With IR in place, generating target code becomes straightforward:

```rust
// Future: IR → Python translator
pub fn ir_to_python(ir: &IRTask) -> String {
    match ir {
        IRTask::Literal(Value::String(s)) => format!(r#""{}""#, s),
        IRTask::Literal(Value::Int(n)) => n.to_string(),
        IRTask::BinaryOp { op, left, right } => {
            let l = ir_to_python(left);
            let r = ir_to_python(right);
            format!("({} {} {})", l, op, r)
        }
        IRTask::Call { func_name, args, .. } => {
            let args_str = args.iter()
                .map(ir_to_python)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}({})", func_name, args_str)
        }
        // ... etc
    }
}
```

**Required additions for codegen:**
- Type information in IR (currently missing)
- Runtime behavior specification (how to execute each node)
- Target language-specific runtime libraries

### 10.2 Target Languages

**Easy targets:**
- Python (dynamic typing, similar operators)
- JavaScript (similar syntax, async/await)
- SQL (only for SQL functions)

**Hard targets:**
- Rust (memory safety, borrowing rules)
- C++ (type system complexity)
- Go (different concurrency model)

---

## 11. Conclusion & Recommendations

### 11.1 Summary Table

| Aspect | Current State | Refactoring Feasibility | Effort |
|--------|---------------|------------------------|--------|
| **AST layer** | Clean & separate | ✅ Already good | - |
| **IR layer** | Partial (BAML only) | ✅ Can be completed | 2-3 weeks |
| **Target codegen** | Non-existent | ⚠️ Possible but complex | 3-4 weeks |
| **Optimization passes** | None | ✅ Easy to add after IR | 1-2 weeks |
| **Type analysis** | Runtime only | ⚠️ Can add pre-compilation | 1 week |

### 11.2 Recommended Path Forward

**Phase 1: Create General-Purpose IR** (2-3 weeks)
- Define `IRTask` enum and compiler
- Refactor evaluator to use IR internally
- Maintain 100% compatibility with current behavior
- Add IR debugging/inspection capabilities

**Phase 2: Add Static Analysis** (1 week) - Optional
- Variable resolution pass
- Type inference (basic)
- Function signature checking

**Phase 3: Code Generation** (3-4 weeks) - Optional
- Start with Python target
- Build target-specific runtime helpers
- Add type information to IR

### 11.3 Risk vs. Reward

**Recommended Approach: Do Phase 1 Now**

**Rewards:**
- Foundation for future optimization
- Better code structure for maintenance
- Enables debugging/inspection features
- Path to compilation if needed later

**Risks:**
- Moderate refactoring effort (2-3 weeks)
- Need thorough testing
- BAML integration needs careful handling

**Effort is justified because:**
1. Foundational work (once done, enables everything else)
2. Moderate complexity (not highly risky)
3. Codebase is small enough to refactor safely
4. Will improve code quality even if codegen never happens

---

## 12. Appendix: Key Files Reference

| File | Lines | Purpose | Coupling |
|------|-------|---------|----------|
| `src/parser/mod.rs` | 1200+ | AST definition & parsing | Low (clean parser) |
| `src/parser/grammar.pest` | 200+ | Grammar rules | Self-contained |
| `src/eval/evaluator.rs` | 650+ | Direct AST execution | **High (tightly couples to AST)** |
| `src/eval/builtin.rs` | 400+ | Builtin function implementation | Medium (diverse execution types) |
| `src/types/mod.rs` | 150+ | Value & type registry | Low (utility module) |
| `src/types/registry.rs` | 80+ | Type storage | Low (data structure) |
| `src/eval/sql.rs` | 100+ | DuckDB SQL execution | Low (isolated) |

---

## 13. Appendix: Example IR Transformation

### Before (AST):
```rust
// Input: "[1, 2, 3] |> Length(_)"
Expr::Sequential {
    left: Box::new(Expr::List(vec![
        Expr::Int(1),
        Expr::Int(2),
        Expr::Int(3),
    ])),
    right: Box::new(Expr::FunctionCall {
        name: "Length".to_string(),
        args: vec![Expr::Variable("_".to_string())],
    }),
    binding: None,
}
```

### After (IR):
```rust
IRTask::Sequence {
    tasks: vec![
        IRTask::Literal(Value::List(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
        ])),
        IRTask::Call {
            func_name: "Length".to_string(),
            args: vec![IRTask::LoadVar("_".to_string())],
            return_type: Some(FieldType::Int),
        },
    ],
    bindings: vec![
        ("_".to_string(), 0), // _ = result of task 0
    ],
}
```

**Advantages of IR version:**
- Explicit sequencing order
- Variable bindings are explicit
- Type information attached to function call
- Can be executed without re-parsing
- Can be optimized (e.g., pre-compute literal list)
- Can be inspected/debugged
- Can be compiled to other languages

---

**End of Analysis**
