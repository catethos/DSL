# IR Architecture: Two-Tier Intermediate Representation

## Overview

The DSL compiler uses a **two-tier intermediate representation (IR)** architecture that separates high-level semantic analysis from low-level execution. This design preserves the clarity and analyzability of special constructs (LLM calls, HTTP requests, SQL queries) while providing a simple, composable execution model.

## Architecture Diagram

```
┌─────────────────────────────────────────────────┐
│  Parser (prompt: blocks, HTTP blocks, etc.)     │
└───────────────────┬─────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────┐
│  High-Level IR (HIR)                            │
│  - IRExecution::LLM { prompt, config, ... }     │
│  - IRExecution::HTTP { ... }                    │
│  - IRExecution::SQL { ... }                     │
│                                                  │
│  Used for:                                      │
│  - Type checking                                │
│  - Static analysis                              │
│  - Cost estimation                              │
│  - Policy enforcement                           │
│  - Tooling/IDE features                         │
└───────────────────┬─────────────────────────────┘
                    │
                    │ Lowering Pass
                    ▼
┌─────────────────────────────────────────────────┐
│  Low-Level IR (LIR)                             │
│  - IRExecution::Expression with:                │
│    - __llm_execute(template, params, config)    │
│    - __http(method, url, headers, body)         │
│    - __sql(query, tables)                       │
│                                                  │
│  Features:                                      │
│  - Composable (try/catch, sequencing)           │
│  - Simple interpreter                           │
│  - Effect tags preserved for analysis           │
└───────────────────┬─────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────┐
│  Interpreter (executes LIR only)                │
│  - Routes intrinsic calls to builtins           │
│  - Carries spans/debug info for errors          │
└─────────────────────────────────────────────────┘
```

## Two-Tier Design Rationale

### Why Not Pure Builtin Functions?

A pure builtin approach (where LLM/HTTP/SQL are just function calls) would lose:

1. **Effect System & Policy**
   - Explicit LLM nodes enable tracking of "effects" for policy enforcement
   - Cost limiting, budget checks, and audit logs need declarative visibility
   - PII policies and regulated environments require effect tracking

2. **Optimization Opportunities**
   - Batching identical LLM calls requires IR-level visibility
   - Prompt caching needs to identify invariant prompts at compile time
   - Cannot optimize what you cannot see in the IR

3. **Source Mapping & Debugging**
   - Better error messages with spans from `prompt:` blocks to runtime calls
   - Caching keys and deterministic replays need precise prompt tracking
   - Rich debugging experience requires semantic information

4. **Static Analysis**
   - Cannot perform cost estimation without knowing which calls are LLM operations
   - Security analysis needs to identify data flows through external services
   - IDE tooling (autocomplete, hover info) benefits from semantic nodes

5. **Testability**
   - Special nodes allow clean IR-level mocking/stubbing
   - Targeted fixture replacement for LLM calls in tests
   - Less intrusive testing without runtime interception

### Why Not Keep Special Execution Forever?

Keeping only HIR would lose:

1. **Composability**: Cannot use try/catch, conditionals, or loops around special constructs
2. **Interpreter Complexity**: Requires separate execution paths for each construct type
3. **Uniformity**: Different constructs have different evaluation semantics
4. **Extensibility**: Adding new effects requires modifying the interpreter core

### The Hybrid Solution

The two-tier approach provides **the best of both worlds**:
- HIR preserves semantic information for analysis and tooling
- LIR provides a uniform execution model for the interpreter
- Lowering pass bridges the two with debug info preservation

## High-Level IR (HIR)

### Purpose

HIR represents the program as written by the user, preserving all semantic information about special constructs.

### Key Types

```rust
pub enum IRExecution {
    /// LLM execution with prompt template
    LLM {
        prompt: IRNode,
        model: Option<String>,
        return_type: String,
        config: HashMap<String, Value>,
    },

    /// HTTP request execution
    HTTP {
        method: String,
        url: IRNode,
        params: HashMap<String, IRNode>,
        headers: HashMap<String, String>,
        body: Option<IRNode>,
    },

    /// SQL query execution
    SQL {
        query: String,
        tables: Vec<(String, String)>,
    },

    /// Generic expression (used post-lowering)
    Expression {
        body: IRNode,
    },
}
```

### Effect Metadata

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EffectKind {
    LLM,      // Large Language Model call
    HTTP,     // Network request
    SQL,      // Database query
    Pure,     // No side effects
}

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub file: String,
    pub line: usize,
    pub column: usize,
}
```

Function calls in HIR may include effect metadata:

```rust
IRNode::FunctionCall {
    name: String,
    args: Vec<IRNode>,
    effect_kind: Option<EffectKind>,  // Set during lowering
    source_span: Option<Span>,         // Set during lowering
}
```

### Use Cases

1. **Type Checking**: Verify that prompt templates have correct parameter types
2. **Cost Estimation**: Count LLM nodes to estimate token usage before execution
3. **Policy Enforcement**: Reject programs that make too many HTTP requests
4. **IDE Features**: Show hover info for LLM prompts with expected response types
5. **Static Analysis**: Track data flows through external services for security audits

## Low-Level IR (LIR)

### Purpose

LIR represents the program in a uniform, composable form suitable for execution. All special constructs are transformed into intrinsic function calls.

### Lowering Transformations

#### LLM Lowering

```rust
// HIR:
IRExecution::LLM {
    prompt,
    model: Some("gpt-4"),
    config: { temperature: 0.7 },
    return_type: "Person",
}

// LIR:
IRExecution::Expression {
    body: IRNode::FunctionCall {
        name: "__llm_execute",
        args: vec![
            lower_template(prompt),           // Structured template
            IRNode::Map(config),               // Model + temperature
            IRNode::String("Person"),          // Return type
        ],
        effect_kind: Some(EffectKind::LLM),
        source_span: Some(span),
    }
}
```

#### HTTP Lowering

```rust
// HIR:
IRExecution::HTTP {
    method: "POST",
    url: template_url,
    headers: { "Content-Type": "application/json" },
    body: Some(json_body),
}

// LIR:
IRExecution::Expression {
    body: IRNode::FunctionCall {
        name: "__http",
        args: vec![
            IRNode::String("POST"),
            lower_node(url),
            IRNode::Map(params),
            IRNode::Map(headers),
            lower_node(body),
        ],
        effect_kind: Some(EffectKind::HTTP),
        source_span: Some(span),
    }
}
```

#### SQL Lowering

```rust
// HIR:
IRExecution::SQL {
    query: "SELECT * FROM users WHERE age > ${min_age}",
    tables: vec![],
}

// LIR:
IRExecution::Expression {
    body: IRNode::FunctionCall {
        name: "__sql",
        args: vec![
            IRNode::TemplateString { segments },
        ],
        effect_kind: Some(EffectKind::SQL),
        source_span: Some(span),
    }
}
```

### Intrinsic Functions

Intrinsic functions are special built-in functions that are:
- Not user-callable directly (names start with `__`)
- Implemented in the interpreter's builtin registry
- Tagged with effect kinds for analysis
- Carry source span information for error reporting

Current intrinsics:
- `__llm_execute(prompt, config, return_type)` - Execute LLM call
- `__http(method, url, params, headers, body)` - Execute HTTP request
- `__sql(query)` - Execute SQL query

## The Lowering Pass

### Implementation

The lowering pass is implemented in `crates/dsl-ir/src/lowering.rs`.

```rust
pub struct Lowering {
    debug_info: DebugInfoTable,
    next_node_id: usize,
}

impl Lowering {
    /// Lower a complete HIR program to LIR
    pub fn lower_program(hir: IR) -> (IR, DebugInfoTable) {
        let mut lowering = Lowering::new();
        let lir = lowering.lower_ir(hir);
        (lir, lowering.debug_info)
    }

    /// Lower a single execution node (used by interpreter)
    pub fn lower_execution_direct(exec: &IRExecution) -> IRExecution {
        let mut lowering = Lowering::new();
        lowering.lower_execution(exec)
    }
}
```

### Debug Information

The lowering pass preserves debug information for error reporting:

```rust
pub struct DebugInfo {
    pub original_construct: String,  // "LLM function", "HTTP request"
    pub source_span: Span,
    pub function_name: String,
    pub prompt_text: Option<String>,
}

pub struct DebugInfoTable {
    entries: HashMap<NodeId, DebugInfo>,
}
```

This allows runtime errors to reference the original HIR construct:

```
Error in LLM function 'extract_user_info' at main.dsl:42:10
  Type mismatch: expected Person, got String
  Prompt: Extract name and age from: ...
  Response: John is 30 years old
```

### Template Unification

Templates are kept as structured IR nodes through lowering:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum IRTemplateSegment {
    Text(String),
    Interpolation(Box<IRNode>),  // Compiled expression, not string!
}
```

Benefits:
- Template expressions are parsed and compiled once at compile time
- Type checking can verify interpolation expressions
- Lowering pass recursively processes interpolations
- No runtime parsing overhead

## Interpreter Execution Model

### Simplified Interpreter

The interpreter only executes LIR (lowered IR):

```rust
impl Interpreter {
    fn execute_function(&mut self, ir_func: &IRFunction) -> Result<Value> {
        // Lower HIR to LIR first
        let lowered = Lowering::lower_execution_direct(&ir_func.execution);

        // Execute only Expression variant
        match lowered {
            IRExecution::Expression { body } => {
                self.eval_expression(body)
            }
            _ => unreachable!("All executions should be lowered"),
        }
    }
}
```

### Intrinsic Dispatch

Function calls check for intrinsic names:

```rust
fn eval_expression(&mut self, node: &IRNode) -> Result<Value> {
    match node {
        IRNode::FunctionCall { name, args, effect_kind, source_span } => {
            if name.starts_with("__") {
                // Route to intrinsic builtins
                let arg_values: Vec<Value> = args.iter()
                    .map(|arg| self.eval_expression(arg))
                    .collect::<Result<_>>()?;
                self.builtins.call_intrinsic_with_values(
                    name,
                    &arg_values,
                    effect_kind,
                    source_span
                )
            } else {
                // Regular function call
                self.call_user_function(name, args)
            }
        }
        // ... other cases
    }
}
```

### Intrinsic Implementation

```rust
impl BuiltinFunctions {
    pub fn call_intrinsic_with_values(
        &mut self,
        name: &str,
        args: &[Value],
        effect_kind: Option<EffectKind>,
        span: Option<Span>,
    ) -> Result<Value, InterpreterError> {
        match name {
            "__llm_execute" => {
                let prompt = &args[0];
                let config = &args[1];
                let return_type = &args[2];
                self.intrinsic_llm_execute(prompt, config, return_type, span)
            }
            "__http" => {
                let method = &args[0];
                let url = &args[1];
                let params = &args[2];
                let headers = &args[3];
                let body = &args[4];
                self.intrinsic_http(method, url, params, headers, body, span)
            }
            "__sql" => {
                let query = &args[0];
                self.intrinsic_sql(query, span)
            }
            _ => Err(InterpreterError::UnknownIntrinsic {
                name: name.to_string(),
                span,
            }),
        }
    }
}
```

## Error Handling

### Rich Error Types

The interpreter uses structured error types that include source location and context:

```rust
pub enum InterpreterError {
    LLMError {
        message: String,
        function_name: Option<String>,
        span: Option<Span>,
        prompt: Option<String>,
        response: Option<String>,
    },
    HTTPError {
        message: String,
        function_name: Option<String>,
        span: Option<Span>,
        method: String,
        url: String,
    },
    SQLError {
        message: String,
        function_name: Option<String>,
        span: Option<Span>,
        query: String,
    },
    TypeError {
        expected: String,
        got: String,
        span: Option<Span>,
    },
    // ... more variants
}
```

### Error Context Preservation

The lowering pass preserves source spans through to runtime:

1. **Parsing**: Parser records source locations in AST
2. **Compilation**: Compiler preserves spans in HIR
3. **Lowering**: Lowering pass copies spans to LIR intrinsic calls
4. **Execution**: Intrinsics include spans in error results
5. **Display**: Error messages show file, line, and column

Example error output:

```
LLM Error in function 'extract_person' at example.dsl:15:5
  Type mismatch: expected Person, got String
  Prompt: Extract name and age from: John is 30 years old
  Response: "John is 30 years old"
```

## Benefits Summary

### Immediate Benefits

1. **Simpler Interpreter**: Only executes expressions, no special cases
2. **Composability**: Can use try/catch, sequencing, conditionals around effectful operations
3. **Unified Execution**: Everything is an expression at runtime
4. **Better Testing**: Can wrap operations in user-defined retry/fallback logic

### Preserved Capabilities

1. **Static Analysis**: HIR retains explicit LLM/HTTP/SQL nodes
2. **Type Safety**: Return types tracked through lowering
3. **Error Messages**: Spans and context preserved via debug info
4. **Optimization Potential**: Can analyze HIR for batching/caching

### Future Extensibility

1. **Batching**: Analyze HIR to find identical LLM calls, batch in optimizer pass
2. **Streaming**: Add `Stream<T>` value type, mark intrinsics as streaming-capable
3. **Tool Calls**: Add `EffectKind::ToolCall`, lower to `__tool_execute` intrinsic
4. **Policy Enforcement**: Walk HIR to enforce cost limits, PII policies before execution
5. **Cost Estimation**: Calculate estimated token usage from HIR before running

## File Locations

- **HIR Types**: `crates/dsl-ir/src/ir.rs`
- **Lowering Pass**: `crates/dsl-ir/src/lowering.rs`
- **Interpreter**: `crates/dsl-interpreter/src/interpreter.rs`
- **Intrinsics**: `crates/dsl-interpreter/src/builtins.rs`
- **Error Types**: `crates/dsl-interpreter/src/error.rs`

## Related Documentation

- [Implementation Details](../12-Implementation-Details.md) - Overall compiler architecture
- [Error Handling](error-handling.md) - Error handling patterns
- [Building from Source](building.md) - Developer setup
