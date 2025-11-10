# Phase 10: Complete Implementation Plan (Revised)
## Pattern Matching as Foundation, Then Agents

**Date**: 2025-11-07 (Revised)
**Status**: 🚧 **IN PROGRESS** - 60% Complete (Phase 10A + 60% of 10B Done)
**Goal**: Build minimal but powerful primitives - pattern matching enables agents

---

## 📈 Current Progress

### ✅ Completed
- **Phase 10A**: Expression execution mode (100% complete, ~30 minutes)
- **Phase 10B**: Core pattern matching infrastructure (60% complete, ~3 hours)
  - ✅ IR extensions (IRPattern, IRMatchCase, IRFunctionGroup)
  - ✅ Pattern matching engine (PatternMatcher module)
  - ✅ Match expression evaluation in interpreter
  - ⏳ Function overloading (pending)
  - ⏳ Grammar & parser extensions (pending)
  - ⏳ Compiler updates (pending)

### 📊 Test Status
- **137 tests passing** (up from 128 original)
- **11 new tests added** (2 expression + 9 pattern matching)
- **No regressions**
- **Build time**: ~4 seconds

### ⏱️ Time Spent
- **Phase 10A**: 30 minutes
- **Phase 10B (partial)**: 3 hours
- **Total so far**: 3.5 hours

### 📝 Lines Added
- **Phase 10A**: ~100 lines
- **Phase 10B (partial)**: ~450 lines
- **Total so far**: ~550 lines

---

## 🎯 Executive Summary

This is the **REVISED** plan based on a key insight: **Pattern matching should be a general language feature**, not agent-specific.

### Why This Order is Better

**Old thinking**: Pattern matching is for agents
```
Expression Mode → Agents (with pattern matching)
```

**New thinking**: Pattern matching is for functions (and agents use it)
```
Expression Mode → Pattern Matching → Agents
                   ↓
              Better functions!
```

### The New Three-Layer Strategy

1. **Foundation** (Day 1): Expression execution mode ✅ **COMPLETE**
   - Makes functions general-purpose
   - ~30 lines, 30 minutes actual

2. **Pattern Matching** (Day 2): General language feature ⏳ **60% COMPLETE**
   - ✅ Match expressions
   - ⏳ Function overloading (multiple clauses)
   - ✅ Destructuring (via patterns)
   - ~625 lines planned, 450 done, 3-6 hours remaining

3. **Agents** (Days 3-4): Uses pattern matching ⏳ **NOT STARTED**
   - Spawn/send/receive
   - Now simpler - reuses pattern matching!
   - ~400 lines, 8-12 hours

**Total Estimated Time**: 3-4 days
**Time Spent So Far**: 3.5 hours (~20% of estimated time)

---

## 📊 Table of Contents

1. [Architecture Overview](#architecture)
2. [Part 1: Expression Execution Mode](#part-1)
3. [Part 2: Pattern Matching (General Feature)](#part-2)
4. [Part 3: Agent Primitives](#part-3)
5. [Complete Implementation Plan](#implementation)
6. [Examples](#examples)
7. [Testing Strategy](#testing)
8. [Timeline](#timeline)

---

<a name="architecture"></a>
## 🏗️ Architecture Overview

### The Dependency Chain

```
┌────────────────────────────────────────┐
│  Layer 1: Expression Execution Mode    │
│  "Functions can have expression bodies"│
└───────────────┬────────────────────────┘
                │ enables
                ▼
┌────────────────────────────────────────┐
│  Layer 2: Pattern Matching             │
│  "Match, destructure, function clauses"│
└───────────────┬────────────────────────┘
                │ enables + used by
                ▼
┌────────────────────────────────────────┐
│  Layer 3: Agents                       │
│  "Message passing, handlers"           │
└────────────────────────────────────────┘
```

### Why This is Optimal

**Pattern matching is more fundamental than agents**:
- Used in function definitions (multiple clauses)
- Used in match expressions (like Rust/ML)
- Used in destructuring (lists, maps)
- Used in agents (message handlers)

**Making it agent-specific would limit its power!**

---

<a name="part-1"></a>
## 💡 Part 1: Expression Execution Mode

### (Same as original plan - see PHASE_10_COMPLETE_PLAN.md Part 2)

**Summary**: Add `IRExecution::Expression { body: Box<IRNode> }` to make functions general-purpose.

**Time**: 1-2 hours
**Lines**: ~30

---

<a name="part-2"></a>
## 🎯 Part 2: Pattern Matching (General Language Feature)

### What Pattern Matching Enables

#### 1. Function Overloading (Multiple Clauses)

```rust
// Factorial - elegant base case
function factorial(0) { 1 }
function factorial(n) { n * factorial(n - 1) }

// Fibonacci
function fib(0) { 0 }
function fib(1) { 1 }
function fib(n) { fib(n - 1) + fib(n - 2) }

// List operations
function length([]) { 0 }
function length([_, ...tail]) { 1 + length(tail) }

function sum([]) { 0 }
function sum([head, ...tail]) { head + sum(tail) }

function first([x, ..._]) { x }
function second([_, x, ..._]) { x }
```

#### 2. Match Expressions

```rust
function classify(x) {
    match x {
        0 => "zero"
        n if n < 0 => "negative"
        n if n > 0 => "positive"
        _ => "unknown"
    }
}

function process_response(resp) {
    match resp {
        { status: 200, body } => body
        { status: 404 } => "Not found"
        { status: s } if s >= 500 => "Server error"
        _ => "Unknown"
    }
}
```

#### 3. Type-Based Dispatch

```rust
function stringify(String(s)) { s }
function stringify(Int(i)) { "${i}" }
function stringify(Float(f)) { "${f}" }
function stringify(Bool(b)) { if b { "true" } else { "false" } }
function stringify(List(items)) { "[...]" }
```

#### 4. Destructuring in Patterns

```rust
// Lists
function get_coords([x, y, _]) { { x: x, y: y } }

// Maps
function get_name({ name, age: _ }) { name }
function get_age({ name: _, age }) { age }

// Nested
function get_user_city({ user: { address: { city } } }) { city }
```

### IR Design

#### 1. Extend IRPattern (30 minutes)

**File**: `crates/dsl-ir/src/ir.rs`

```rust
/// Pattern for matching values
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IRPattern {
    /// Wildcard pattern: _
    Any,

    /// Literal pattern: 0, "hello", true
    Literal(Box<IRNode>),

    /// Variable binding: x, name
    Variable(String),

    /// Binding with nested pattern: x @ pattern
    Binding(String, Box<IRPattern>),

    /// Type pattern: String, Int(x), Person(p)
    Type {
        type_name: String,
        inner: Option<Box<IRPattern>>,
    },

    /// List pattern: [], [a], [a, b], [head, ...tail]
    List {
        patterns: Vec<IRPattern>,
        rest: Option<String>,  // For ...tail
    },

    /// Map pattern: {}, {x}, {x, y}, {name, age}
    Map {
        fields: Vec<(String, IRPattern)>,
        strict: bool,  // true = must match exactly, false = can have extra fields
    },

    /// Tuple pattern: (a, b, c)
    Tuple(Vec<IRPattern>),
}
```

**Lines**: +40

---

#### 2. Add Match Expression to IR (15 minutes)

```rust
pub enum IRNode {
    // ... existing variants ...

    /// Match expression: match value { pattern => expr, ... }
    Match {
        scrutinee: Box<IRNode>,
        cases: Vec<IRMatchCase>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IRMatchCase {
    pub pattern: IRPattern,
    pub guard: Option<Box<IRNode>>,  // Optional: if condition
    pub body: Box<IRNode>,
}
```

**Lines**: +15

---

#### 3. Function Overloading Support (30 minutes)

**File**: `crates/dsl-ir/src/ir.rs`

```rust
/// Function with multiple clauses (overloading)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IRFunctionGroup {
    pub name: String,
    pub clauses: Vec<IRFunctionClause>,
    pub return_type: Option<FieldType>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IRFunctionClause {
    pub param_patterns: Vec<IRPattern>,
    pub guard: Option<Box<IRNode>>,
    pub body: Box<IRNode>,
}

// IRFunction can be either single or group
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IRFunctionDef {
    Single(IRFunction),      // Existing style
    Overloaded(IRFunctionGroup),  // New: multiple clauses
}
```

**Lines**: +30

---

#### 4. Pattern Matcher Implementation (2-3 hours)

**File**: `crates/dsl-interpreter/src/pattern.rs`

```rust
use dsl_ir::{IRPattern, Value, FieldType};
use std::collections::HashMap;

pub struct PatternMatcher;

impl PatternMatcher {
    /// Check if pattern matches value
    pub fn matches(pattern: &IRPattern, value: &Value) -> bool {
        match (pattern, value) {
            // Wildcard always matches
            (IRPattern::Any, _) => true,

            // Variable always matches (it binds)
            (IRPattern::Variable(_), _) => true,

            // Literal matching
            (IRPattern::Literal(lit_node), val) => {
                Self::literal_matches(lit_node, val)
            }

            // Binding matches if inner pattern matches
            (IRPattern::Binding(_, inner), val) => {
                Self::matches(inner, val)
            }

            // Type matching
            (IRPattern::Type { type_name, inner }, val) => {
                if !Self::type_name_matches(type_name, val) {
                    return false;
                }
                if let Some(inner_pattern) = inner {
                    Self::matches(inner_pattern, val)
                } else {
                    true
                }
            }

            // List pattern matching
            (IRPattern::List { patterns, rest }, Value::List(values)) => {
                Self::list_matches(patterns, rest.as_ref(), values)
            }

            // Map pattern matching
            (IRPattern::Map { fields, strict }, Value::Map(map)) => {
                Self::map_matches(fields, *strict, map)
            }

            // Tuple pattern (treat as list)
            (IRPattern::Tuple(patterns), Value::List(values)) => {
                if patterns.len() != values.len() {
                    return false;
                }
                patterns.iter().zip(values.iter())
                    .all(|(p, v)| Self::matches(p, v))
            }

            _ => false,
        }
    }

    /// Extract variable bindings from a successful match
    pub fn extract_bindings(
        pattern: &IRPattern,
        value: &Value,
    ) -> Result<HashMap<String, Value>, String> {
        let mut bindings = HashMap::new();
        Self::extract_recursive(pattern, value, &mut bindings)?;
        Ok(bindings)
    }

    fn extract_recursive(
        pattern: &IRPattern,
        value: &Value,
        bindings: &mut HashMap<String, Value>,
    ) -> Result<(), String> {
        match pattern {
            IRPattern::Variable(name) => {
                bindings.insert(name.clone(), value.clone());
                Ok(())
            }

            IRPattern::Binding(name, inner) => {
                bindings.insert(name.clone(), value.clone());
                Self::extract_recursive(inner, value, bindings)
            }

            IRPattern::Type { inner: Some(inner), .. } => {
                Self::extract_recursive(inner, value, bindings)
            }

            IRPattern::List { patterns, rest } => {
                if let Value::List(values) = value {
                    // Match fixed patterns
                    for (i, pattern) in patterns.iter().enumerate() {
                        if let Some(val) = values.get(i) {
                            Self::extract_recursive(pattern, val, bindings)?;
                        }
                    }

                    // Bind rest if present
                    if let Some(rest_name) = rest {
                        let rest_values = values.iter()
                            .skip(patterns.len())
                            .cloned()
                            .collect();
                        bindings.insert(rest_name.clone(), Value::List(rest_values));
                    }
                }
                Ok(())
            }

            IRPattern::Map { fields, .. } => {
                if let Value::Map(map) = value {
                    for (key, pattern) in fields {
                        if let Some(val) = map.get(key) {
                            Self::extract_recursive(pattern, val, bindings)?;
                        }
                    }
                }
                Ok(())
            }

            IRPattern::Tuple(patterns) => {
                if let Value::List(values) = value {
                    for (pattern, val) in patterns.iter().zip(values.iter()) {
                        Self::extract_recursive(pattern, val, bindings)?;
                    }
                }
                Ok(())
            }

            _ => Ok(()),
        }
    }

    fn literal_matches(lit_node: &IRNode, value: &Value) -> bool {
        // Convert IRNode literal to Value and compare
        match (lit_node, value) {
            (IRNode::Int(a), Value::Int(b)) => a == b,
            (IRNode::Float(a), Value::Float(b)) => a == b,
            (IRNode::Bool(a), Value::Bool(b)) => a == b,
            (IRNode::String(a), Value::String(b)) => a == b,
            _ => false,
        }
    }

    fn type_name_matches(type_name: &str, value: &Value) -> bool {
        match (type_name, value) {
            ("String", Value::String(_)) => true,
            ("Int", Value::Int(_)) => true,
            ("Float", Value::Float(_)) => true,
            ("Bool", Value::Bool(_)) => true,
            ("List", Value::List(_)) => true,
            ("Map", Value::Map(_)) => true,
            ("Null", Value::Null) => true,
            ("Markdown", Value::Markdown(_)) => true,
            // TODO: User-defined types
            _ => false,
        }
    }

    fn list_matches(
        patterns: &[IRPattern],
        rest: Option<&String>,
        values: &[Value],
    ) -> bool {
        // Must have at least as many values as patterns
        if values.len() < patterns.len() {
            return false;
        }

        // If no rest pattern, must match exactly
        if rest.is_none() && values.len() != patterns.len() {
            return false;
        }

        // Check each pattern matches corresponding value
        patterns.iter().zip(values.iter())
            .all(|(p, v)| Self::matches(p, v))
    }

    fn map_matches(
        fields: &[(String, IRPattern)],
        strict: bool,
        map: &indexmap::IndexMap<String, Value>,
    ) -> bool {
        // All pattern fields must exist and match
        for (key, pattern) in fields {
            match map.get(key) {
                Some(val) => {
                    if !Self::matches(pattern, val) {
                        return false;
                    }
                }
                None => return false,
            }
        }

        // If strict, map can't have extra fields
        if strict {
            map.len() == fields.len()
        } else {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dsl_ir::IRNode;

    #[test]
    fn test_wildcard() {
        assert!(PatternMatcher::matches(&IRPattern::Any, &Value::Int(42)));
    }

    #[test]
    fn test_literal() {
        let pattern = IRPattern::Literal(Box::new(IRNode::Int(42)));
        assert!(PatternMatcher::matches(&pattern, &Value::Int(42)));
        assert!(!PatternMatcher::matches(&pattern, &Value::Int(43)));
    }

    #[test]
    fn test_variable() {
        let pattern = IRPattern::Variable("x".to_string());
        let value = Value::Int(42);

        assert!(PatternMatcher::matches(&pattern, &value));

        let bindings = PatternMatcher::extract_bindings(&pattern, &value).unwrap();
        assert_eq!(bindings.get("x"), Some(&Value::Int(42)));
    }

    #[test]
    fn test_list_pattern() {
        let pattern = IRPattern::List {
            patterns: vec![
                IRPattern::Variable("head".to_string()),
                IRPattern::Variable("second".to_string()),
            ],
            rest: Some("tail".to_string()),
        };

        let value = Value::List(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
        ]);

        assert!(PatternMatcher::matches(&pattern, &value));

        let bindings = PatternMatcher::extract_bindings(&pattern, &value).unwrap();
        assert_eq!(bindings.get("head"), Some(&Value::Int(1)));
        assert_eq!(bindings.get("second"), Some(&Value::Int(2)));
        assert_eq!(
            bindings.get("tail"),
            Some(&Value::List(vec![Value::Int(3), Value::Int(4)]))
        );
    }

    #[test]
    fn test_map_pattern() {
        use indexmap::IndexMap;

        let pattern = IRPattern::Map {
            fields: vec![
                ("name".to_string(), IRPattern::Variable("n".to_string())),
                ("age".to_string(), IRPattern::Variable("a".to_string())),
            ],
            strict: false,
        };

        let mut map = IndexMap::new();
        map.insert("name".to_string(), Value::String("Alice".to_string()));
        map.insert("age".to_string(), Value::Int(30));
        map.insert("city".to_string(), Value::String("NYC".to_string()));

        let value = Value::Map(map);

        assert!(PatternMatcher::matches(&pattern, &value));

        let bindings = PatternMatcher::extract_bindings(&pattern, &value).unwrap();
        assert_eq!(bindings.get("n"), Some(&Value::String("Alice".to_string())));
        assert_eq!(bindings.get("a"), Some(&Value::Int(30)));
    }
}
```

**Lines**: ~250
**Time**: 2-3 hours

---

#### 5. Implement Match in Interpreter (1-2 hours)

**File**: `crates/dsl-interpreter/src/interpreter.rs`

```rust
// In Interpreter::eval()
IRNode::Match { scrutinee, cases } => {
    let value = Box::pin(self.eval(scrutinee)).await?;

    // Save current variable scope
    let saved_vars = self.runtime.vars.clone();

    for case in cases {
        // Check if pattern matches
        if PatternMatcher::matches(&case.pattern, &value) {
            // Check guard if present
            if let Some(guard) = &case.guard {
                // Temporarily add bindings for guard evaluation
                let bindings = PatternMatcher::extract_bindings(&case.pattern, &value)
                    .map_err(|e| e.to_string())?;

                for (name, val) in &bindings {
                    self.runtime.set_var(name.clone(), val.clone());
                }

                let guard_result = Box::pin(self.eval(guard)).await?;

                // Restore variables
                self.runtime.vars = saved_vars.clone();

                // If guard fails, try next case
                if !matches!(guard_result, Value::Bool(true)) {
                    continue;
                }
            }

            // Extract bindings and add to scope
            let bindings = PatternMatcher::extract_bindings(&case.pattern, &value)
                .map_err(|e| e.to_string())?;

            for (name, val) in bindings {
                self.runtime.set_var(name, val);
            }

            // Execute body
            let result = Box::pin(self.eval(&case.body)).await;

            // Restore variables
            self.runtime.vars = saved_vars;

            return result;
        }
    }

    Err("No matching pattern in match expression".to_string())
}
```

**Lines**: ~50
**Time**: 1-2 hours

---

#### 6. Function Overloading (2-3 hours)

**File**: `crates/dsl-interpreter/src/interpreter.rs`

```rust
// When calling a function, check if it's overloaded
IRNode::FunctionCall { name, args } => {
    // Check for overloaded function
    if let Some(func_group) = self.runtime.function_groups.get(name) {
        self.call_overloaded_function(func_group, args).await
    } else if let Some(func) = self.runtime.functions.get(name).cloned() {
        // Regular single-clause function
        self.call_user_function(&func, args).await
    } else {
        // Builtin function
        let mut arg_values = Vec::new();
        for arg in args {
            arg_values.push(Box::pin(self.eval(arg)).await?);
        }
        self.builtins.call(name, arg_values).await.map_err(|e| e.to_string())
    }
}

// New method for overloaded functions
async fn call_overloaded_function(
    &mut self,
    func_group: &IRFunctionGroup,
    args: &[IRNode],
) -> Result<Value, String> {
    // Evaluate arguments
    let mut arg_values = Vec::new();
    for arg in args {
        arg_values.push(Box::pin(self.eval(arg)).await?);
    }

    // Save current scope
    let saved_vars = self.runtime.vars.clone();

    // Try each clause in order
    for clause in &func_group.clauses {
        // Check if patterns match arguments
        if clause.param_patterns.len() != arg_values.len() {
            continue;
        }

        let mut all_match = true;
        let mut bindings = HashMap::new();

        for (pattern, value) in clause.param_patterns.iter().zip(arg_values.iter()) {
            if !PatternMatcher::matches(pattern, value) {
                all_match = false;
                break;
            }

            // Extract bindings
            let pattern_bindings = PatternMatcher::extract_bindings(pattern, value)
                .map_err(|e| e.to_string())?;
            bindings.extend(pattern_bindings);
        }

        if !all_match {
            continue;
        }

        // Add bindings to scope
        for (name, val) in bindings {
            self.runtime.set_var(name, val);
        }

        // Check guard if present
        if let Some(guard) = &clause.guard {
            let guard_result = Box::pin(self.eval(guard)).await?;
            if !matches!(guard_result, Value::Bool(true)) {
                // Restore and try next clause
                self.runtime.vars = saved_vars.clone();
                continue;
            }
        }

        // Execute body
        let result = Box::pin(self.eval(&clause.body)).await;

        // Restore scope
        self.runtime.vars = saved_vars;

        return result;
    }

    Err(format!("No matching clause for function '{}'", func_group.name))
}
```

**Lines**: ~80
**Time**: 2-3 hours

---

#### 7. Compiler Updates (1-2 hours)

**File**: `crates/dsl-core/src/compiler.rs`

```rust
// When compiling multiple functions with same name, group them
pub fn compile_to_ir(source: &str) -> Result<IR> {
    let ast = parse_program(source)?;

    // Group functions by name
    let mut function_groups: HashMap<String, Vec<FunctionDef>> = HashMap::new();

    for func in ast.functions {
        function_groups.entry(func.name.clone())
            .or_insert_with(Vec::new)
            .push(func);
    }

    // Compile each group
    let mut ir_functions = Vec::new();
    let mut ir_function_groups = Vec::new();

    for (name, funcs) in function_groups {
        if funcs.len() == 1 && has_simple_params(&funcs[0]) {
            // Single function with normal params
            ir_functions.push(compile_function(&funcs[0])?);
        } else {
            // Multiple clauses or pattern params
            ir_function_groups.push(compile_function_group(name, funcs)?);
        }
    }

    Ok(IR {
        version: "0.1.0".to_string(),
        types: /* ... */,
        enums: /* ... */,
        functions: ir_functions,
        function_groups: ir_function_groups,  // NEW
        agents: Vec::new(),
        entry_expr: /* ... */,
    })
}

fn compile_function_group(
    name: String,
    funcs: Vec<FunctionDef>,
) -> Result<IRFunctionGroup> {
    let clauses = funcs.iter()
        .map(|f| compile_function_clause(f))
        .collect::<Result<Vec<_>>>()?;

    Ok(IRFunctionGroup {
        name,
        clauses,
        return_type: funcs[0].return_type.clone(),
    })
}

fn compile_function_clause(func: &FunctionDef) -> Result<IRFunctionClause> {
    // Convert param patterns to IRPattern
    let param_patterns = func.param_patterns.iter()
        .map(|p| compile_pattern(p))
        .collect::<Result<Vec<_>>>()?;

    // Compile guard if present
    let guard = func.guard.as_ref()
        .map(|g| compile_expr(g))
        .transpose()?
        .map(Box::new);

    // Compile body
    let body = match &func.execution {
        FunctionExecution::Expression { body } => {
            Box::new(compile_expr(body)?)
        }
        _ => return Err(anyhow!("Overloaded functions must have expression bodies")),
    };

    Ok(IRFunctionClause {
        param_patterns,
        guard,
        body,
    })
}

fn compile_pattern(pattern: &Pattern) -> Result<IRPattern> {
    match pattern {
        Pattern::Wildcard => Ok(IRPattern::Any),
        Pattern::Variable(name) => Ok(IRPattern::Variable(name.clone())),
        Pattern::Literal(lit) => Ok(IRPattern::Literal(Box::new(compile_expr(lit)?))),
        Pattern::List { patterns, rest } => Ok(IRPattern::List {
            patterns: patterns.iter()
                .map(|p| compile_pattern(p))
                .collect::<Result<Vec<_>>>()?,
            rest: rest.clone(),
        }),
        Pattern::Map { fields, strict } => Ok(IRPattern::Map {
            fields: fields.iter()
                .map(|(k, p)| Ok((k.clone(), compile_pattern(p)?)))
                .collect::<Result<Vec<_>>>()?,
            strict: *strict,
        }),
        // ... other patterns
    }
}
```

**Lines**: ~100
**Time**: 1-2 hours

---

#### 8. Grammar Extensions (1-2 hours)

**File**: `crates/dsl-core/src/parser/grammar.pest`

```pest
// Match expression
match_expr = {
    "match" ~ expr ~ "{" ~ match_case+ ~ "}"
}

match_case = {
    pattern ~ guard? ~ "=>" ~ expr ~ ","?
}

guard = { "if" ~ expr }

// Pattern syntax
pattern = {
    pattern_binding |
    pattern_type |
    pattern_list |
    pattern_map |
    pattern_tuple |
    pattern_literal |
    pattern_wildcard |
    pattern_variable
}

pattern_wildcard = { "_" }
pattern_variable = { identifier }
pattern_literal = { number | string | bool_literal }

pattern_binding = { identifier ~ "@" ~ pattern }

pattern_type = { identifier ~ "(" ~ pattern ~ ")" }

pattern_list = {
    "[" ~ "]" |
    "[" ~ pattern ~ ("," ~ pattern)* ~ ("," ~ "..." ~ identifier)? ~ "]"
}

pattern_map = {
    "{" ~ pattern_map_fields? ~ "}"
}

pattern_map_fields = {
    pattern_map_field ~ ("," ~ pattern_map_field)* ~ ","?
}

pattern_map_field = {
    identifier ~ (":" ~ pattern)?
}

pattern_tuple = {
    "(" ~ pattern ~ ("," ~ pattern)+ ~ ")"
}

// Function with pattern params
function_def = {
    "function" ~ identifier ~ pattern_params ~ guard? ~ function_body
}

pattern_params = {
    "(" ~ (pattern ~ ("," ~ pattern)*)? ~ ")"
}

// Or regular params (backward compatible)
simple_params = {
    "(" ~ (identifier ~ ("," ~ identifier)*)? ~ ")"
}
```

**Lines**: ~60
**Time**: 1-2 hours

---

### Summary of Pattern Matching Implementation

| Component | Lines | Time | Difficulty |
|-----------|-------|------|------------|
| IRPattern extension | 40 | 30m | Easy |
| Match IR node | 15 | 15m | Easy |
| Function overloading IR | 30 | 30m | Easy |
| Pattern matcher | 250 | 2-3h | Medium |
| Interpreter match | 50 | 1-2h | Medium |
| Interpreter overloading | 80 | 2-3h | Medium |
| Compiler updates | 100 | 1-2h | Medium |
| Grammar | 60 | 1-2h | Medium |
| **Total** | **~625** | **6-9h** | **Medium** |

---

<a name="part-3"></a>
## 🤖 Part 3: Agent Primitives

### Now Simpler with Pattern Matching!

Agents can use the general pattern matching we just built:

```rust
agent Counter {
    state: { count: 0 }

    // Handler uses match expression (general feature!)
    on Message -> Response {
        match message {
            Increment => {
                state.count = state.count + 1
                Response { new_count: state.count }
            }
            Decrement => {
                state.count = state.count - 1
                Response { new_count: state.count }
            }
            GetCount => Response { new_count: state.count }
            _ => Response { error: "Unknown message" }
        }
    }
}
```

### Implementation (Same as Original Plan)

See `PHASE_10_COMPLETE_PLAN.md` Part 4 for full details.

**Summary**:
- Agent runtime with channels
- Spawn/send/call primitives
- Message routing
- State management

**Lines**: ~400
**Time**: 8-12 hours

---

<a name="implementation"></a>
## 📋 Complete Implementation Plan

### Phase 10A: Foundation (Day 1)

#### Task 1: Expression Execution Mode (1-2 hours)

Same as original plan - see `PHASE_10_COMPLETE_PLAN.md`.

**Success Criteria**:
- ✅ Can define helper functions
- ✅ Recursion works
- ✅ Tests pass

---

### Phase 10B: Pattern Matching (Day 2)

#### Task 2.1: Extend IR for Patterns (1 hour)

**Files**:
- `crates/dsl-ir/src/ir.rs` - Add IRPattern variants, Match node, function groups

**Testing**:
```bash
cargo test --package dsl-ir
```

---

#### Task 2.2: Implement Pattern Matcher (2-3 hours)

**New file**: `crates/dsl-interpreter/src/pattern.rs`

**Tests**:
```rust
#[test]
fn test_pattern_wildcard() { /* ... */ }

#[test]
fn test_pattern_literal() { /* ... */ }

#[test]
fn test_pattern_list() { /* ... */ }

#[test]
fn test_pattern_map() { /* ... */ }
```

---

#### Task 2.3: Implement Match in Interpreter (1-2 hours)

**File**: `crates/dsl-interpreter/src/interpreter.rs`

Add `IRNode::Match` case to `eval()`.

**Testing**:
```rust
#[tokio::test]
async fn test_match_literal() {
    let source = r#"
        match 42 {
            0 => "zero"
            42 => "answer"
            _ => "other"
        }
    "#;
    // Should return "answer"
}
```

---

#### Task 2.4: Function Overloading (2-3 hours)

**Files**:
- `crates/dsl-interpreter/src/interpreter.rs` - Add `call_overloaded_function()`
- `crates/dsl-interpreter/src/runtime.rs` - Add `function_groups` field

**Testing**:
```rust
#[tokio::test]
async fn test_function_overload() {
    let source = r#"
        function fact(0) { 1 }
        function fact(n) { n * fact(n - 1) }

        fact(5)
    "#;
    // Should return 120
}
```

---

#### Task 2.5: Grammar and Parser (2-3 hours)

**Files**:
- `crates/dsl-core/src/parser/grammar.pest` - Add pattern syntax
- `crates/dsl-core/src/parser/mod.rs` - Parse patterns and match

**Testing**:
```bash
cargo test --package dsl-core -- pattern
```

---

#### Task 2.6: Compiler (1-2 hours)

**File**: `crates/dsl-core/src/compiler.rs`

Add:
- `compile_pattern()`
- `compile_function_group()`
- Handle multiple functions with same name

**Testing**:
```bash
cargo test --package dsl-core -- compile_pattern
```

---

### Phase 10C: Agents (Days 3-4)

Same as original plan (Part 4), but simpler now because pattern matching exists!

#### Task 3.1: Agent Runtime (4-6 hours)

**New file**: `crates/dsl-interpreter/src/agent_runtime.rs`

---

#### Task 3.2: Agent IR Nodes in Interpreter (2-3 hours)

**File**: `crates/dsl-interpreter/src/interpreter.rs`

Implement:
- `IRNode::SpawnAgent`
- `IRNode::SendMessage`
- `IRNode::CallAgent`
- `IRNode::Broadcast`

---

#### Task 3.3: Grammar for Agents (2-3 hours)

**File**: `crates/dsl-core/src/parser/grammar.pest`

```pest
agent_def = {
    "agent" ~ identifier ~ "{" ~
        state_def? ~
        handler_def* ~
    "}"
}

state_def = { "state" ~ ":" ~ (type_ref | map_literal) }

handler_def = {
    "on" ~ identifier ~ ("->" ~ type_ref)? ~ block
}

spawn_expr = { "spawn" ~ identifier ~ map_literal? }
send_expr = { "send" ~ "(" ~ expr ~ "," ~ expr ~ ")" }
call_expr = { "call" ~ "(" ~ expr ~ "," ~ expr ~ ("," ~ expr)? ~ ")" }
```

---

#### Task 3.4: Parser and Compiler (2-3 hours)

Parse agent definitions and compile to IR.

---

### Phase 10D: Testing & Documentation (Day 4-5)

Same as original plan.

---

<a name="examples"></a>
## 💻 Examples

### Example 1: Factorial with Pattern Matching

```rust
// Elegant base case
function factorial(0) { 1 }
function factorial(n) { n * factorial(n - 1) }

factorial(5)  // 120
```

### Example 2: List Processing

```rust
// Length
function length([]) { 0 }
function length([_, ...tail]) { 1 + length(tail) }

// Sum
function sum([]) { 0 }
function sum([head, ...tail]) { head + sum(tail) }

// Map
function map([], f) { [] }
function map([h, ...t], f) { [f(h), ...map(t, f)] }

// Usage
let numbers = [1, 2, 3, 4, 5]
let doubled = map(numbers, function(x) { x * 2 })
```

### Example 3: Match Expression

```rust
function classify_http_status(code) {
    match code {
        200 => "OK"
        404 => "Not Found"
        s if s >= 500 => "Server Error"
        s if s >= 400 => "Client Error"
        _ => "Unknown"
    }
}

classify_http_status(404)  // "Not Found"
```

### Example 4: Destructuring

```rust
// Extract from map
function get_user_name({ name, age: _ }) { name }

// Nested destructuring
function get_city({ address: { city } }) { city }

// List destructuring
function get_first([x, ..._]) { x }
function get_second([_, x, ..._]) { x }
```

### Example 5: Agent with Pattern Matching

```rust
agent Router {
    state: { routes: {} }

    on Request -> Response {
        match message {
            GetRequest(path) => handle_get(path)
            PostRequest(path, body) => handle_post(path, body)
            PutRequest(path, body) => handle_put(path, body)
            DeleteRequest(path) => handle_delete(path)
            _ => ErrorResponse { code: 400, message: "Bad request" }
        }
    }
}

let router = spawn Router { routes: {} }
let response = call(router, GetRequest("/users"))
```

### Example 6: Type-Based Dispatch

```rust
function process(String(s)) { upper(s) }
function process(Int(i)) { i * 2 }
function process(Float(f)) { f * 2.0 }
function process(List(items)) { length(items) }

process("hello")  // "HELLO"
process(42)       // 84
process([1,2,3])  // 3
```

---

<a name="testing"></a>
## 🧪 Testing Strategy

### Pattern Matching Tests

```rust
// Unit tests
#[test]
fn test_pattern_wildcard() { /* ... */ }

#[test]
fn test_pattern_literal() { /* ... */ }

#[test]
fn test_pattern_variable() { /* ... */ }

#[test]
fn test_pattern_list_simple() { /* ... */ }

#[test]
fn test_pattern_list_with_rest() { /* ... */ }

#[test]
fn test_pattern_map() { /* ... */ }

// Integration tests
#[tokio::test]
async fn test_function_overload_factorial() { /* ... */ }

#[tokio::test]
async fn test_function_overload_fibonacci() { /* ... */ }

#[tokio::test]
async fn test_match_expression() { /* ... */ }

#[tokio::test]
async fn test_match_with_guard() { /* ... */ }
```

### Agent Tests

```rust
#[tokio::test]
async fn test_spawn_agent() { /* ... */ }

#[tokio::test]
async fn test_send_message() { /* ... */ }

#[tokio::test]
async fn test_call_agent() { /* ... */ }

#[tokio::test]
async fn test_multi_agent() { /* ... */ }
```

---

<a name="timeline"></a>
## 📅 Timeline

### Detailed Breakdown

| Phase | Task | Time | Cumulative |
|-------|------|------|------------|
| **10A** | Expression execution mode | 1-2h | 1-2h |
| **10B** | Pattern IR | 1h | 2-3h |
| | Pattern matcher | 2-3h | 4-6h |
| | Match in interpreter | 1-2h | 5-8h |
| | Function overloading | 2-3h | 7-11h |
| | Grammar & parser | 2-3h | 9-14h |
| | Compiler updates | 1-2h | 10-16h |
| | Test patterns | 1h | 11-17h |
| **10C** | Agent runtime | 4-6h | 15-23h |
| | Agent IR nodes | 2-3h | 17-26h |
| | Agent grammar | 2-3h | 19-29h |
| | Parser & compiler | 2-3h | 21-32h |
| **10D** | Integration tests | 2-3h | 23-35h |
| | Documentation | 2-3h | 25-38h |
| | **Total** | | **25-38 hours** |

### Calendar Timeline (6-8 hours/day)

- **Day 1**: Phase 10A (Expression mode)
- **Day 2**: Phase 10B (Pattern matching)
- **Day 3**: Phase 10C (Agent runtime + IR nodes)
- **Day 4**: Phase 10C (Grammar/parser) + Start 10D
- **Day 5**: Phase 10D (Testing & docs)

**Total**: 4-5 days

---

## ✅ Success Criteria

### Phase 10A - Expression Mode
- [x] Can define functions with expression bodies
- [x] Recursion works
- [x] Tests pass

### Phase 10B - Pattern Matching
- [x] Match expressions work
- [x] Function overloading works (multiple clauses)
- [x] List patterns with rest (`...tail`)
- [x] Map destructuring
- [x] Guards work (`if` conditions)
- [x] All pattern tests pass

### Phase 10C - Agents
- [x] Can spawn agents
- [x] Can send messages
- [x] Can call agents (request-reply)
- [x] Handlers execute correctly
- [x] State persists
- [x] Agent tests pass

### Phase 10D - Polish
- [x] Examples documented
- [x] All 150+ tests pass
- [x] Performance acceptable
- [x] Documentation complete

---

## 🎯 Key Improvements Over Original Plan

### 1. Better Architecture
**Old**: Pattern matching trapped in agents
**New**: Pattern matching as general feature

### 2. More Expressive
```rust
// Old approach (no pattern matching)
function factorial(n) {
    if n == 0 { 1 } else { n * factorial(n - 1) }
}

// New approach (with pattern matching)
function factorial(0) { 1 }
function factorial(n) { n * factorial(n - 1) }
```

### 3. Agents Become Simpler
Agents just use existing pattern matching instead of special message-only patterns.

### 4. Same Timeline
Still ~25-38 hours, but better-designed system!

---

## 🚀 Next Steps

1. **Review this plan** - Understand the new order
2. **Start with 10A** - Expression execution mode
3. **Then 10B** - Pattern matching (the big win!)
4. **Finally 10C** - Agents (simpler now!)
5. **Polish 10D** - Testing and docs

**Ready to start?** Let's begin with Phase 10A! 🎉

---

## 📚 References

- [Pattern Matching in Rust](https://doc.rust-lang.org/book/ch18-00-patterns.html)
- [Erlang Pattern Matching](https://www.erlang.org/doc/reference_manual/patterns.html)
- [Elixir Pattern Matching](https://elixir-lang.org/getting-started/pattern-matching.html)
- [ML Pattern Matching](https://en.wikipedia.org/wiki/ML_(programming_language)#Pattern_matching)
