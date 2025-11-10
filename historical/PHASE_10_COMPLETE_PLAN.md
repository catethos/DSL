# Phase 10: Complete Implementation Plan
## From IR Foundation to Agent Primitives

**Date**: 2025-11-07
**Status**: 📋 Planning
**Goal**: Implement minimal but powerful agent system with general-purpose functions

---

## 🎯 Executive Summary

This document outlines a **minimal, theoretically-sound approach** to implementing Phase 10 (Agent Primitives). The key insight is that we need **very few new primitives** if we design them correctly.

### The Three-Layer Strategy

1. **Foundation Layer** (Day 1): Add `Expression` execution mode to functions
   - Makes functions general-purpose
   - Enables loop compilation to recursion
   - ~25 lines of code, 1-2 hours

2. **Control Flow Layer** (Optional): Defer or minimize
   - Agents don't need loops in handlers (event-driven)
   - Can compile to recursion if needed later
   - Skip for Phase 10A

3. **Agent Layer** (Days 2-4): Core agent primitives
   - Spawn agents
   - Send/receive messages
   - Pattern matching
   - ~500 lines of code, 16-24 hours

**Total Estimated Time**: 3-4 days of focused work

---

## 📊 Table of Contents

1. [The Problem: Current Function Limitations](#problem)
2. [Solution 1: Expression Execution Mode](#solution-1)
3. [Solution 2: Minimal Control Flow](#solution-2)
4. [Solution 3: Agent Primitives](#solution-3)
5. [Solution 4: Pattern Matching](#solution-4)
6. [Complete Implementation Plan](#implementation)
7. [Examples and Use Cases](#examples)
8. [Testing Strategy](#testing)
9. [Timeline and Estimates](#timeline)

---

<a name="problem"></a>
## 🔍 Part 1: The Problem

### Current Function Limitation

**Current IR** (`dsl-ir/src/ir.rs`):
```rust
pub enum IRExecution {
    LLM { prompt: String, ... },      // Special-purpose only
    HTTP { method: String, ... },     // Special-purpose only
    SQL { query: String },            // Special-purpose only
    HTTPWithLLM { ... },              // Special-purpose only
    // ❌ No way to define general functions!
}
```

### What We Can't Do

```rust
// ❌ Can't define helper functions
function add(a, b) {
    a + b  // No execution mode for arbitrary expressions!
}

// ❌ Can't compile loops to recursive functions
// while x < 10 { body }
// Would need: function __loop() { if x < 10 { body; __loop() } }
//             But functions can't have expression bodies!

// ❌ Can't define agent message handlers
agent Counter {
    on Increment -> Int {
        state.count + 1  // No way to execute this!
    }
}
```

### Why This Is Critical

Without general-purpose functions:
- Can't implement loop → recursion compilation
- Can't implement agent handlers
- Can't have user-defined helpers
- IR remains limited to LLM/HTTP/SQL only

**This is the blocker for everything else.**

---

<a name="solution-1"></a>
## 💡 Part 2: Solution 1 - Expression Execution Mode

### Design

Add **one new variant** to `IRExecution`:

```rust
pub enum IRExecution {
    // Existing special-purpose modes
    LLM { prompt: String, ... },
    HTTP { method: String, ... },
    SQL { query: String },
    HTTPWithLLM { ... },

    // NEW: General-purpose expression body
    Expression {
        body: Box<IRNode>,  // ✅ Any IR expression!
    },
}
```

### What This Enables

```rust
// ✅ Helper functions
function add(a, b) {
    a + b
}
// Compiles to: IRExecution::Expression { body: BinaryOp { ... } }

// ✅ Recursive functions
function factorial(n) {
    if n <= 1 { 1 } else { n * factorial(n - 1) }
}
// Compiles to: IRExecution::Expression { body: Conditional { ... } }

// ✅ Agent handlers
agent Counter {
    on Increment -> Int {
        state.count + 1
    }
}
// Handler compiles to: IRExecution::Expression { body: BinaryOp { ... } }

// ✅ Loop compilation (if we want it)
// while x < 10 { body }
// Compiles to recursive function with Expression body
```

### Implementation Changes

#### 1. IR Definition (5 minutes)

**File**: `crates/dsl-ir/src/ir.rs` (line 151)

```rust
pub enum IRExecution {
    LLM { /* ... */ },
    HTTP { /* ... */ },
    SQL { /* ... */ },
    HTTPWithLLM { /* ... */ },

    /// NEW: Expression-bodied function
    Expression {
        /// The function body as an IR expression
        body: Box<IRNode>,
    },
}
```

**Impact**: +4 lines

---

#### 2. Compiler (15 minutes)

**File**: `crates/dsl-core/src/compiler.rs` (line 152)

**Add to `compile_function()`**:
```rust
pub fn compile_function(func: &FunctionDef) -> Result<IRFunction> {
    let execution = match &func.execution {
        // Existing modes...
        FunctionExecution::LLM { ... } => IRExecution::LLM { ... },
        FunctionExecution::HTTP { ... } => IRExecution::HTTP { ... },
        FunctionExecution::SQL { ... } => IRExecution::SQL { ... },
        FunctionExecution::HTTPWithLLM { ... } => IRExecution::HTTPWithLLM { ... },

        // NEW: Expression-bodied function
        FunctionExecution::Expression { body } => {
            IRExecution::Expression {
                body: Box::new(compile_expr(body)?),
            }
        },
    };

    Ok(IRFunction { /* ... */ })
}
```

**Impact**: +5 lines

---

#### 3. Interpreter (10 minutes)

**File**: `crates/dsl-interpreter/src/interpreter.rs` (line 452)

**Add to `call_user_function()`**:
```rust
let result = match &func.execution {
    // Existing modes...
    IRExecution::LLM { ... } => self.execute_llm_function(...).await?,
    IRExecution::HTTP { ... } => self.execute_http_function(...).await?,
    IRExecution::SQL { ... } => self.execute_sql_function(...).await?,
    IRExecution::HTTPWithLLM { ... } => { /* ... */ },

    // NEW: Expression-bodied function
    IRExecution::Expression { body } => {
        // Just evaluate the body expression!
        Box::pin(self.eval(body)).await?
    },
};
```

**Impact**: +4 lines

---

#### 4. Parser (30 minutes)

**File**: `crates/dsl-core/src/parser/mod.rs`

**Add to `FunctionExecution` enum**:
```rust
pub enum FunctionExecution {
    LLM { /* ... */ },
    HTTP { /* ... */ },
    SQL { /* ... */ },
    HTTPWithLLM { /* ... */ },

    // NEW
    Expression {
        body: Box<Expr>,
    },
}
```

**Update parsing logic** to detect expression-bodied functions:
```rust
// In parse_function_def():
let execution = if let Some(via_clause) = /* check for "via" */ {
    // Parse LLM/HTTP/SQL modes
    match via_clause {
        "llm" => parse_llm_execution(...),
        "http" => parse_http_execution(...),
        "sql" => parse_sql_execution(...),
    }
} else {
    // NEW: No "via" clause means expression body
    let body = parse_block(pairs)?;
    FunctionExecution::Expression {
        body: Box::new(body),
    }
};
```

**Impact**: +15 lines

---

#### 5. Grammar (5 minutes)

**File**: `crates/dsl-core/src/parser/grammar.pest`

**Update `function_def` rule**:
```pest
function_def = {
    "function" ~ identifier ~ param_list ~ return_type? ~ function_body
}

function_body = {
    via_clause ~ "{" ~ execution_config ~ "}" |  // LLM/HTTP/SQL
    block                                         // NEW: Expression body
}

via_clause = { "via" ~ ("llm" | "http" | "sql") }

block = { "{" ~ expr ~ "}" }
```

**Impact**: +3 lines

---

### Testing

```rust
#[tokio::test]
async fn test_expression_function() {
    let source = r#"
        function add(a, b) {
            a + b
        }
        add(2, 3)
    "#;

    let ir = compile_to_ir(source).unwrap();
    let mut interpreter = Interpreter::from_ir(&ir).unwrap();
    let result = interpreter.eval(&ir.entry_expr).await.unwrap();

    assert_eq!(result, Value::Int(5));
}

#[tokio::test]
async fn test_recursive_function() {
    let source = r#"
        function factorial(n) {
            if n <= 1 { 1 } else { n * factorial(n - 1) }
        }
        factorial(5)
    "#;

    let ir = compile_to_ir(source).unwrap();
    let mut interpreter = Interpreter::from_ir(&ir).unwrap();
    let result = interpreter.eval(&ir.entry_expr).await.unwrap();

    assert_eq!(result, Value::Int(120));
}
```

### Summary

**Total Changes**: ~31 lines of code
**Time Estimate**: 1-2 hours
**Complexity**: Low
**Impact**: 🚀 **Massive** - Makes functions general-purpose!

---

<a name="solution-2"></a>
## 🔄 Part 3: Solution 2 - Minimal Control Flow

### Theoretical Analysis

**Question**: What control flow primitives are **truly necessary**?

**Answer**: For Phase 10 agents - **NONE!**

### Why Agents Don't Need Loops

Agents are **event-driven** (reactive), not **loop-driven** (imperative):

```rust
// Agent pattern: No loops needed!
agent Counter {
    state: { count: 0 }

    on Increment -> Int {
        // Handle ONE message
        // No loops - runtime loops for you!
        state.count + 1
    }

    on GetCount -> Int {
        state.count
    }
}

// The runtime does:
loop {  // ← This loop is in the runtime, not user code!
    let msg = receive();
    match msg {
        Increment => handle_increment(),
        GetCount => handle_get_count(),
    }
}
```

### What If Users Really Need Loops?

Two options:

#### Option A: Compile to Recursion (Elegant)

```rust
// DSL: while x < 10 { body }

// Compiler generates synthetic function:
function __while_123() {
    if x < 10 {
        body
        __while_123()  // Tail recursion
    } else {
        null
    }
}
__while_123()
```

**Requirements**: Just need `Expression` execution mode ✅ (already adding!)

**IR Nodes Needed**: Zero (uses existing Conditional + FunctionCall)

---

#### Option B: Add Minimal Loop Primitive (Pragmatic)

If recursion proves problematic (stack overflow, debugging):

```rust
// Add to IRNode:
Loop {
    body: Box<IRNode>,
}

// All DSL loops compile to this one IR construct:
// while cond { body } → Loop { if !cond { break }; body }
// for x in list { body } → Loop { if i >= len { break }; ... }
```

**Requirements**: Add Loop + Break to IR (~150 lines)

**Decision**: **Defer to Phase 10.5** - Not needed for agents

---

### Recommendation for Phase 10

**Skip all loop constructs:**
- No `Loop`, `While`, `For`, `Break`, `Continue` in IR
- No loop syntax in grammar
- If users need iteration, use recursion or defer to Phase 11

**Rationale**:
1. Agents don't need loops (event-driven)
2. Can add later if real use case emerges
3. Saves 6-8 hours of implementation time
4. Keeps IR minimal and clean

---

<a name="solution-3"></a>
## 🤖 Part 4: Solution 3 - Agent Primitives

### What We Need

From the IR (already defined in Phase 2):

```rust
// Already in IR! (Phase 2)
IRNode::SpawnAgent { agent_type, init_state }
IRNode::SendMessage { target, message }
IRNode::CallAgent { target, message, timeout_ms }
IRNode::ReceiveMessage { pattern }
IRNode::Broadcast { targets, message }  // Optional
```

**Status**: ✅ IR types exist, need interpreter implementation

### Agent Runtime Architecture

```
┌─────────────────────────────────────────────────┐
│               Agent Runtime                     │
│                                                 │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐     │
│  │ Agent A  │  │ Agent B  │  │ Agent C  │     │
│  │          │  │          │  │          │     │
│  │ State: X │  │ State: Y │  │ State: Z │     │
│  │          │  │          │  │          │     │
│  │ Handlers │  │ Handlers │  │ Handlers │     │
│  │  • on M1 │  │  • on M2 │  │  • on M3 │     │
│  │  • on M2 │  │  • on M3 │  │  • on M1 │     │
│  │          │  │          │  │          │     │
│  │ Channel  │  │ Channel  │  │ Channel  │     │
│  │   ↓      │  │   ↓      │  │   ↓      │     │
│  └───┼──────┘  └───┼──────┘  └───┼──────┘     │
│      │             │              │            │
│      └─────────────┴──────────────┘            │
│                    │                           │
│         ┌──────────▼──────────┐               │
│         │   Message Router    │               │
│         │  (Tokio channels)   │               │
│         └─────────────────────┘               │
└─────────────────────────────────────────────────┘
```

### Implementation Plan

#### 1. Create Agent Runtime Module (4-6 hours)

**New File**: `crates/dsl-interpreter/src/agent_runtime.rs`

```rust
use tokio::sync::mpsc;
use std::collections::HashMap;
use dsl_ir::{Value, IRAgent, IRMessageHandler, IRPattern};

/// A message sent between agents
#[derive(Debug, Clone)]
pub struct Message {
    pub sender: String,
    pub payload: Value,
    pub reply_to: Option<mpsc::Sender<Value>>,
}

/// Handle to a running agent
#[derive(Clone)]
pub struct AgentHandle {
    pub name: String,
    pub tx: mpsc::Sender<Message>,
}

/// The agent runtime manages all agents
pub struct AgentRuntime {
    agents: HashMap<String, AgentHandle>,
}

impl AgentRuntime {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
        }
    }

    /// Spawn a new agent
    pub async fn spawn_agent(
        &mut self,
        agent_def: IRAgent,
        init_state: Value,
        interpreter: &Interpreter,
    ) -> Result<AgentHandle, String> {
        let (tx, mut rx) = mpsc::channel::<Message>(100);

        let agent_name = agent_def.name.clone();
        let handlers = agent_def.handlers.clone();

        // Clone interpreter for the agent's task
        let mut agent_interpreter = interpreter.clone();

        // Spawn agent task
        tokio::spawn(async move {
            let mut state = init_state;

            while let Some(msg) = rx.recv().await {
                // Find matching handler
                let handler = handlers.iter()
                    .find(|h| pattern_matches(&h.message_type, &msg.payload))
                    .ok_or_else(|| format!("No handler for message type"));

                if let Ok(handler) = handler {
                    // Set up state in interpreter
                    agent_interpreter.runtime.set_var("state".to_string(), state.clone());
                    agent_interpreter.runtime.set_var("message".to_string(), msg.payload.clone());

                    // Execute handler body
                    let result = agent_interpreter.eval(&handler.body).await;

                    // Update state from interpreter
                    if let Ok(new_state) = agent_interpreter.runtime.get_var("state") {
                        state = new_state;
                    }

                    // Send reply if needed
                    if let Some(reply_tx) = msg.reply_to {
                        if let Ok(value) = result {
                            let _ = reply_tx.send(value).await;
                        }
                    }
                }
            }
        });

        let handle = AgentHandle {
            name: agent_name.clone(),
            tx,
        };

        self.agents.insert(agent_name, handle.clone());
        Ok(handle)
    }

    /// Send a message to an agent (fire-and-forget)
    pub async fn send_message(
        &self,
        target: &str,
        message: Value,
    ) -> Result<(), String> {
        let agent = self.agents.get(target)
            .ok_or_else(|| format!("Agent '{}' not found", target))?;

        agent.tx.send(Message {
            sender: "system".to_string(),
            payload: message,
            reply_to: None,
        }).await.map_err(|e| e.to_string())
    }

    /// Call an agent and wait for reply
    pub async fn call_agent(
        &self,
        target: &str,
        message: Value,
        timeout_ms: Option<u32>,
    ) -> Result<Value, String> {
        let agent = self.agents.get(target)
            .ok_or_else(|| format!("Agent '{}' not found", target))?;

        let (reply_tx, mut reply_rx) = mpsc::channel(1);

        agent.tx.send(Message {
            sender: "system".to_string(),
            payload: message,
            reply_to: Some(reply_tx),
        }).await.map_err(|e| e.to_string())?;

        // Wait for reply with timeout
        let timeout = std::time::Duration::from_millis(
            timeout_ms.unwrap_or(5000) as u64
        );

        tokio::time::timeout(timeout, reply_rx.recv())
            .await
            .map_err(|_| "Agent call timeout".to_string())?
            .ok_or_else(|| "Agent reply channel closed".to_string())
    }

    /// Broadcast to multiple agents
    pub async fn broadcast(
        &self,
        targets: &[String],
        message: Value,
    ) -> Result<(), String> {
        for target in targets {
            self.send_message(target, message.clone()).await?;
        }
        Ok(())
    }
}

/// Check if a pattern matches a value
fn pattern_matches(pattern: &IRPattern, value: &Value) -> bool {
    match pattern {
        IRPattern::Any => true,
        IRPattern::Type(field_type) => {
            // Check if value matches the expected type
            match (field_type, value) {
                (FieldType::String, Value::String(_)) => true,
                (FieldType::Int, Value::Int(_)) => true,
                (FieldType::Float, Value::Float(_)) => true,
                (FieldType::Bool, Value::Bool(_)) => true,
                // TODO: Class/Enum matching
                _ => false,
            }
        },
        IRPattern::Binding(_, inner) => pattern_matches(inner, value),
    }
}
```

**Lines**: ~200
**Time**: 4-6 hours

---

#### 2. Add Agent Runtime to Interpreter (1-2 hours)

**File**: `crates/dsl-interpreter/src/runtime.rs`

```rust
use crate::agent_runtime::AgentRuntime;

pub struct Runtime {
    pub vars: HashMap<String, Value>,
    pub types: TypeRegistry,
    pub functions: HashMap<String, IRFunction>,
    pub agents: AgentRuntime,  // NEW
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            types: TypeRegistry::new(),
            functions: HashMap::new(),
            agents: AgentRuntime::new(),  // NEW
        }
    }
}
```

---

#### 3. Implement Agent IR Nodes in Interpreter (2-3 hours)

**File**: `crates/dsl-interpreter/src/interpreter.rs`

**Add to `eval()` match**:

```rust
// In Interpreter::eval()
match node {
    // ... existing cases ...

    IRNode::SpawnAgent { agent_type, init_state } => {
        // Get agent definition
        let agent_def = self.runtime.agents.get_agent_def(agent_type)
            .ok_or_else(|| format!("Agent type '{}' not found", agent_type))?;

        // Evaluate init state
        let state = Box::pin(self.eval(init_state)).await?;

        // Spawn agent
        let handle = self.runtime.agents.spawn_agent(
            agent_def.clone(),
            state,
            &self
        ).await?;

        // Return handle name as string
        Ok(Value::String(handle.name))
    }

    IRNode::SendMessage { target, message } => {
        let target_str = Box::pin(self.eval(target)).await?
            .as_string()
            .ok_or_else(|| "Target must be string")?;

        let msg_value = Box::pin(self.eval(message)).await?;

        self.runtime.agents.send_message(&target_str, msg_value).await?;

        Ok(Value::Null)
    }

    IRNode::CallAgent { target, message, timeout_ms } => {
        let target_str = Box::pin(self.eval(target)).await?
            .as_string()
            .ok_or_else(|| "Target must be string")?;

        let msg_value = Box::pin(self.eval(message)).await?;

        let result = self.runtime.agents.call_agent(
            &target_str,
            msg_value,
            *timeout_ms
        ).await?;

        Ok(result)
    }

    IRNode::Broadcast { targets, message } => {
        let target_names: Vec<String> = targets.iter()
            .map(|t| t.clone())  // Assume strings for now
            .collect();

        let msg_value = Box::pin(self.eval(message)).await?;

        self.runtime.agents.broadcast(&target_names, msg_value).await?;

        Ok(Value::Null)
    }

    IRNode::ReceiveMessage { pattern } => {
        // This is typically not used directly
        // Receiving is handled by agent runtime
        Err("ReceiveMessage should be used in agent context only".to_string())
    }
}
```

**Lines**: ~80
**Time**: 2-3 hours

---

#### 4. Store Agent Definitions in IR (30 minutes)

**File**: `crates/dsl-ir/src/ir.rs` (already has this!)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IR {
    pub version: String,
    pub types: Vec<Class>,
    pub enums: Vec<Enum>,
    pub functions: Vec<IRFunction>,
    pub agents: Vec<IRAgent>,  // ✅ Already exists!
    pub entry_expr: IRNode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IRAgent {
    pub name: String,
    pub description: Option<String>,
    pub state_type: Option<Class>,
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

**Status**: ✅ Already defined in Phase 2!

---

#### 5. Load Agent Definitions (15 minutes)

**File**: `crates/dsl-interpreter/src/agent_runtime.rs`

```rust
impl AgentRuntime {
    // Store agent definitions
    agent_defs: HashMap<String, IRAgent>,

    pub fn register_agent_def(&mut self, agent: IRAgent) {
        self.agent_defs.insert(agent.name.clone(), agent);
    }

    pub fn get_agent_def(&self, name: &str) -> Option<&IRAgent> {
        self.agent_defs.get(name)
    }
}

// In Interpreter::from_ir():
for agent in &ir.agents {
    interpreter.runtime.agents.register_agent_def(agent.clone());
}
```

---

### Agent Examples

```rust
// Simple counter agent
agent Counter {
    state: { count: 0 }

    on Increment -> Int {
        state.count = state.count + 1
        state.count
    }

    on GetCount -> Int {
        state.count
    }
}

// Spawn and use
let counter = spawn Counter { count: 0 }
send(counter, Increment)
let count = call(counter, GetCount)
```

### Summary

**Total Changes**: ~400 lines
**Time Estimate**: 8-12 hours
**Complexity**: Medium
**Impact**: 🚀 **Core feature** - Enables multi-agent systems!

---

<a name="solution-4"></a>
## 🎭 Part 5: Solution 4 - Pattern Matching

### Requirements

Pattern matching is needed for:
1. Message reception in agents
2. Selecting correct handler based on message type

### Design

**Already in IR** (from Phase 2):

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IRPattern {
    /// Match any value
    Any,
    /// Match specific type
    Type(FieldType),
    /// Bind to variable and match inner pattern
    Binding(String, Box<IRPattern>),
}
```

### Implementation

#### Simple Pattern Matcher (2-3 hours)

**File**: `crates/dsl-interpreter/src/pattern.rs` (new)

```rust
use dsl_ir::{IRPattern, Value, FieldType};
use std::collections::HashMap;

pub struct PatternMatcher;

impl PatternMatcher {
    /// Check if a pattern matches a value
    pub fn matches(pattern: &IRPattern, value: &Value) -> bool {
        match pattern {
            IRPattern::Any => true,

            IRPattern::Type(field_type) => {
                Self::type_matches(field_type, value)
            }

            IRPattern::Binding(_, inner) => {
                // Binding doesn't affect matching, just captures
                Self::matches(inner, value)
            }
        }
    }

    /// Extract bindings from a pattern match
    pub fn extract_bindings(
        pattern: &IRPattern,
        value: &Value
    ) -> HashMap<String, Value> {
        let mut bindings = HashMap::new();
        Self::extract_bindings_recursive(pattern, value, &mut bindings);
        bindings
    }

    fn extract_bindings_recursive(
        pattern: &IRPattern,
        value: &Value,
        bindings: &mut HashMap<String, Value>,
    ) {
        match pattern {
            IRPattern::Binding(name, inner) => {
                bindings.insert(name.clone(), value.clone());
                Self::extract_bindings_recursive(inner, value, bindings);
            }
            _ => {}
        }
    }

    fn type_matches(field_type: &FieldType, value: &Value) -> bool {
        match (field_type, value) {
            (FieldType::String, Value::String(_)) => true,
            (FieldType::Int, Value::Int(_)) => true,
            (FieldType::Float, Value::Float(_)) => true,
            (FieldType::Bool, Value::Bool(_)) => true,
            (FieldType::List(_), Value::List(_)) => true,
            (FieldType::Map(_), Value::Map(_)) => true,
            // TODO: Class/Enum matching with structural typing
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_any() {
        let pattern = IRPattern::Any;
        assert!(PatternMatcher::matches(&pattern, &Value::Int(42)));
        assert!(PatternMatcher::matches(&pattern, &Value::String("hello".to_string())));
    }

    #[test]
    fn test_pattern_type() {
        let pattern = IRPattern::Type(FieldType::Int);
        assert!(PatternMatcher::matches(&pattern, &Value::Int(42)));
        assert!(!PatternMatcher::matches(&pattern, &Value::String("hello".to_string())));
    }

    #[test]
    fn test_pattern_binding() {
        let pattern = IRPattern::Binding(
            "x".to_string(),
            Box::new(IRPattern::Type(FieldType::Int))
        );

        let value = Value::Int(42);
        assert!(PatternMatcher::matches(&pattern, &value));

        let bindings = PatternMatcher::extract_bindings(&pattern, &value);
        assert_eq!(bindings.get("x"), Some(&Value::Int(42)));
    }
}
```

**Lines**: ~100
**Time**: 2-3 hours
**Complexity**: Low

---

<a name="implementation"></a>
## 📋 Part 6: Complete Implementation Plan

### Phase 10A: Foundation (Day 1)

#### Task 1.1: Add Expression Execution Mode (1-2 hours)

**Files to modify**:
1. `crates/dsl-ir/src/ir.rs` - Add variant
2. `crates/dsl-core/src/compiler.rs` - Compile it
3. `crates/dsl-interpreter/src/interpreter.rs` - Execute it
4. `crates/dsl-core/src/parser/mod.rs` - Parse it
5. `crates/dsl-core/src/parser/grammar.pest` - Grammar

**Testing**:
```bash
# Test helper function
echo 'function add(a,b){a+b} add(2,3)' | cargo run --bin dsl-repl

# Test recursive function
echo 'function fact(n){if n<=1{1}else{n*fact(n-1)}} fact(5)' | cargo run --bin dsl-repl
```

**Success Criteria**:
- ✅ Can define functions with expression bodies
- ✅ Recursion works
- ✅ All tests pass

---

### Phase 10B: Agent Runtime (Days 2-3)

#### Task 2.1: Create Agent Runtime Module (4-6 hours)

**New file**: `crates/dsl-interpreter/src/agent_runtime.rs`

**Implement**:
- `Message` struct
- `AgentHandle` struct
- `AgentRuntime` with spawn/send/call/broadcast

**Testing**:
```rust
#[tokio::test]
async fn test_spawn_agent() {
    let runtime = AgentRuntime::new();
    // Test agent spawning
}
```

---

#### Task 2.2: Integrate with Interpreter (2-3 hours)

**Files to modify**:
- `crates/dsl-interpreter/src/runtime.rs` - Add agents field
- `crates/dsl-interpreter/src/interpreter.rs` - Implement IR nodes

**Testing**:
```rust
#[tokio::test]
async fn test_spawn_and_send() {
    let source = r#"
        agent Echo {
            on Message -> String { message }
        }

        let agent = spawn Echo {}
        send(agent, "hello")
    "#;
    // Test execution
}
```

---

#### Task 2.3: Pattern Matching (2-3 hours)

**New file**: `crates/dsl-interpreter/src/pattern.rs`

**Implement**:
- `PatternMatcher::matches()`
- `PatternMatcher::extract_bindings()`

**Testing**:
```rust
#[test]
fn test_pattern_matching() {
    // Test all pattern types
}
```

---

### Phase 10C: Grammar & Parser (Day 4)

#### Task 3.1: Extend Grammar (2-3 hours)

**File**: `crates/dsl-core/src/parser/grammar.pest`

**Add**:
```pest
// Agent definition
agent_def = {
    "agent" ~ identifier ~ "{" ~
        state_def? ~
        handler_def* ~
    "}"
}

state_def = { "state" ~ ":" ~ type_or_map }

handler_def = {
    "on" ~ identifier ~ "->" ~ identifier? ~ block
}

// Agent primitives
spawn_expr = { "spawn" ~ identifier ~ map_literal? }
send_expr = { "send" ~ "(" ~ expr ~ "," ~ expr ~ ")" }
call_expr = { "call" ~ "(" ~ expr ~ "," ~ expr ~ ("," ~ expr)? ~ ")" }
```

---

#### Task 3.2: Update Parser (3-4 hours)

**File**: `crates/dsl-core/src/parser/mod.rs`

**Add AST nodes**:
```rust
pub enum Expr {
    // ... existing ...

    SpawnAgent { agent_type: String, init_state: Box<Expr> },
    SendMessage { target: Box<Expr>, message: Box<Expr> },
    CallAgent { target: Box<Expr>, message: Box<Expr>, timeout: Option<Box<Expr>> },
}

pub struct AgentDef {
    pub name: String,
    pub state_type: Option<Expr>,
    pub handlers: Vec<MessageHandler>,
}

pub struct MessageHandler {
    pub message_type: String,
    pub reply_type: Option<String>,
    pub body: Box<Expr>,
}
```

**Implement parsing**:
```rust
fn parse_agent_def(pair: Pair<Rule>) -> Result<AgentDef> {
    // Parse agent definition
}

fn parse_handler_def(pair: Pair<Rule>) -> Result<MessageHandler> {
    // Parse message handler
}
```

---

#### Task 3.3: Update Compiler (2-3 hours)

**File**: `crates/dsl-core/src/compiler.rs`

**Add**:
```rust
fn compile_agent(agent: &AgentDef) -> Result<IRAgent> {
    let handlers = agent.handlers.iter()
        .map(|h| compile_handler(h))
        .collect::<Result<Vec<_>>>()?;

    Ok(IRAgent {
        name: agent.name.clone(),
        state_type: /* ... */,
        handlers,
        // ...
    })
}

fn compile_handler(handler: &MessageHandler) -> Result<IRMessageHandler> {
    Ok(IRMessageHandler {
        message_type: /* parse type */,
        reply_type: /* parse type */,
        body: compile_expr(&handler.body)?,
    })
}
```

---

### Phase 10D: Integration Testing (Day 4-5)

#### Task 4.1: End-to-End Examples (2-3 hours)

**Example 1: Counter Agent**
```rust
agent Counter {
    state: { count: 0 }

    on Increment -> Int {
        state.count = state.count + 1
        state.count
    }

    on GetCount -> Int {
        state.count
    }
}

let counter = spawn Counter { count: 0 }
send(counter, Increment)
send(counter, Increment)
let count = call(counter, GetCount)
// count should be 2
```

**Example 2: Echo Agent**
```rust
agent Echo {
    on Message -> String {
        "Echo: " + message
    }
}

let echo = spawn Echo {}
let response = call(echo, "hello")
// response should be "Echo: hello"
```

**Example 3: Multi-Agent System**
```rust
agent Logger {
    on Log -> Null {
        print("LOG: " + message)
        null
    }
}

agent Worker {
    on DoWork -> String {
        send(logger, "Starting work")
        let result = "work done"
        send(logger, "Finished work")
        result
    }
}

let logger = spawn Logger {}
let worker = spawn Worker {}
call(worker, DoWork)
```

---

#### Task 4.2: Performance Testing (1-2 hours)

**Test**: Spawn 100 agents, send 1000 messages

```rust
#[tokio::test]
async fn test_many_agents() {
    // Spawn 100 counter agents
    let agents = (0..100).map(|i| {
        spawn_agent(&counter_def, Value::Map(...))
    }).collect();

    // Send messages concurrently
    for agent in agents {
        send_message(agent, Increment).await;
    }
}
```

---

#### Task 4.3: Documentation (2-3 hours)

Create:
- `docs/AGENTS.md` - Agent system guide
- `examples/counter_agent.dsl`
- `examples/chat_agents.dsl`
- `examples/workflow_agents.dsl`

---

<a name="examples"></a>
## 💻 Part 7: Examples and Use Cases

### Example 1: Simple Counter

```rust
// Define agent
agent Counter {
    state: { count: 0 }

    on Increment -> Int {
        state.count = state.count + 1
        state.count
    }

    on Decrement -> Int {
        state.count = state.count - 1
        state.count
    }

    on Reset -> Int {
        state.count = 0
        0
    }

    on GetCount -> Int {
        state.count
    }
}

// Use it
let counter = spawn Counter { count: 0 }

send(counter, Increment)  // Fire-and-forget
send(counter, Increment)
let count = call(counter, GetCount)  // Request-reply
print(count)  // Should be 2
```

---

### Example 2: Chat System

```rust
agent ChatRoom {
    state: {
        messages: [],
        users: []
    }

    on Join -> String {
        state.users = state.users + [message.username]
        "Welcome " + message.username
    }

    on PostMessage -> Null {
        state.messages = state.messages + [message]

        // Broadcast to all users
        for user in state.users {
            send(user, message)
        }

        null
    }

    on GetMessages -> List {
        state.messages
    }
}

agent User {
    on ReceiveMessage -> Null {
        print("New message: " + message.text)
        null
    }
}

// Set up chat
let room = spawn ChatRoom { messages: [], users: [] }
let alice = spawn User {}
let bob = spawn User {}

call(room, Join { username: "alice", agent: alice })
call(room, Join { username: "bob", agent: bob })

send(room, PostMessage {
    from: "alice",
    text: "Hello everyone!"
})
```

---

### Example 3: Data Pipeline

```rust
agent DataFetcher {
    on Fetch -> Data {
        // Fetch data from API
        let data = http("https://api.example.com/data")
        data
    }
}

agent DataProcessor {
    on Process -> ProcessedData {
        // Process data
        let processed = transform(message.data)
        processed
    }
}

agent DataSaver {
    on Save -> Null {
        // Save to database
        sql("INSERT INTO results VALUES (${message.data})")
        null
    }
}

// Set up pipeline
let fetcher = spawn DataFetcher {}
let processor = spawn DataProcessor {}
let saver = spawn DataSaver {}

// Execute pipeline
let data = call(fetcher, Fetch)
let processed = call(processor, Process { data: data })
send(saver, Save { data: processed })
```

---

<a name="testing"></a>
## 🧪 Part 8: Testing Strategy

### Unit Tests

```rust
// Test 1: Expression functions
#[tokio::test]
async fn test_expression_function() { /* ... */ }

// Test 2: Agent spawning
#[tokio::test]
async fn test_spawn_agent() { /* ... */ }

// Test 3: Message sending
#[tokio::test]
async fn test_send_message() { /* ... */ }

// Test 4: Agent calls
#[tokio::test]
async fn test_call_agent() { /* ... */ }

// Test 5: Pattern matching
#[test]
fn test_pattern_matching() { /* ... */ }
```

### Integration Tests

```rust
// Test 6: Counter agent
#[tokio::test]
async fn test_counter_agent() { /* ... */ }

// Test 7: Multi-agent communication
#[tokio::test]
async fn test_multi_agent() { /* ... */ }

// Test 8: Concurrent agents
#[tokio::test]
async fn test_concurrent_agents() { /* ... */ }
```

### Performance Tests

```rust
// Test 9: 100 agents
#[tokio::test]
async fn test_many_agents() { /* ... */ }

// Test 10: 1000 messages
#[tokio::test]
async fn test_many_messages() { /* ... */ }
```

---

<a name="timeline"></a>
## 📅 Part 9: Timeline and Estimates

### Detailed Breakdown

| Phase | Task | Time | Cumulative |
|-------|------|------|------------|
| **10A** | Expression execution mode | 1-2h | 1-2h |
| | Test expression functions | 30m | 1.5-2.5h |
| **10B** | Agent runtime module | 4-6h | 5.5-8.5h |
| | Integrate with interpreter | 2-3h | 7.5-11.5h |
| | Pattern matching | 2-3h | 9.5-14.5h |
| | Test agent basics | 1h | 10.5-15.5h |
| **10C** | Grammar extension | 2-3h | 12.5-18.5h |
| | Parser updates | 3-4h | 15.5-22.5h |
| | Compiler updates | 2-3h | 17.5-25.5h |
| **10D** | Integration tests | 2-3h | 19.5-28.5h |
| | Performance tests | 1-2h | 20.5-30.5h |
| | Documentation | 2-3h | 22.5-33.5h |
| | **Total** | | **22.5-33.5 hours** |

### Calendar Timeline

**If working 6-8 hours/day**:
- **Day 1**: Phase 10A (Expression mode) + Start 10B
- **Day 2**: Phase 10B (Agent runtime)
- **Day 3**: Phase 10B (Complete) + Start 10C
- **Day 4**: Phase 10C (Grammar/Parser) + Start 10D
- **Day 5**: Phase 10D (Testing & Docs)

**Total**: 4-5 days of focused work

---

## ✅ Success Criteria

### Must Have (Phase 10A-C)

- [x] Expression-bodied functions work
- [x] Can define recursive functions
- [x] Can spawn agents
- [x] Can send messages to agents
- [x] Can call agents and get replies
- [x] Pattern matching works
- [x] Agent handlers execute correctly
- [x] State persists between messages
- [x] All tests pass (150+ tests)

### Should Have (Phase 10D)

- [x] Examples documented
- [x] Performance tests pass
- [x] Multi-agent examples work
- [x] Documentation complete

### Could Have (Future)

- [ ] Broadcast (can defer - just use multiple sends)
- [ ] Loop constructs (can defer - use recursion)
- [ ] Error handling (can defer - use Result types)
- [ ] Agent supervision/restart
- [ ] Agent discovery/registry

---

## 🎯 Decision Summary

### What We're Adding (Minimal Set)

1. **Expression Execution Mode** ✅
   - Critical foundation
   - ~30 lines of code
   - 1-2 hours

2. **Agent Runtime** ✅
   - Core feature
   - ~400 lines of code
   - 8-12 hours

3. **Pattern Matching** ✅
   - Needed for handlers
   - ~100 lines of code
   - 2-3 hours

4. **Grammar/Parser for Agents** ✅
   - User-facing syntax
   - ~200 lines of code
   - 5-7 hours

### What We're Deferring

1. **Loop Constructs** ❌
   - Not needed for agents
   - Can use recursion
   - Defer to Phase 11

2. **Broadcast** ❌
   - Can use multiple sends
   - Nice-to-have
   - Defer to Phase 10.5

3. **Error Handling** ❌
   - Can use Result types
   - Not critical for MVP
   - Defer to Phase 11

### Total Effort

**Code**: ~730 lines
**Time**: 22.5-33.5 hours (3-5 days)
**Complexity**: Medium
**Impact**: 🚀 **Huge** - Enables multi-agent systems!

---

## 🚀 Getting Started

### Step 1: Review This Plan

- Read through all sections
- Ask questions about unclear parts
- Confirm timeline is acceptable

### Step 2: Set Up Environment

```bash
# Ensure all tests pass currently
cargo test --workspace

# Create feature branch
git checkout -b phase-10-agents
```

### Step 3: Start with Phase 10A

Begin with Expression execution mode:
1. Modify IR
2. Update compiler
3. Update interpreter
4. Test

### Step 4: Iterate

Complete each phase sequentially, testing as you go.

---

## 📚 References

- [IR_MIGRATION_PLAN.md](IR_MIGRATION_PLAN.md) - Original plan
- [PHASE_9_SUMMARY.md](PHASE_9_SUMMARY.md) - Current state
- [Actor Model](https://en.wikipedia.org/wiki/Actor_model) - Theoretical basis
- [Erlang OTP](https://www.erlang.org/doc/design_principles/des_princ.html) - Similar system

---

**Ready to begin? Let's start with Phase 10A!** 🚀
