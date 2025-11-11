# Higher-Order Functions Implementation Plan

## Executive Summary

**Approach:** Hybrid intrinsic IR nodes + string references + inline lambdas  
**Timeline:** 3-5 days total across 3 phases  
**Philosophy:** Ship incrementally, prioritize analyst ergonomics, avoid over-engineering  
**Grammar unification:** Both `def` and `fn` use `=> expr end` syntax for consistency

---

## Architecture Overview

### What We're Building

```dsl
// Phase 1: String references to user functions
map(items, "extractTitle")
filter(items, "isValid")

// Phase 1: Inline lambdas (reusing existing => syntax!)
map(items, fn x => x.title + "!" end)
filter(items, fn x => x.score > 50 end)

// Phase 2: Multi-arg lambdas and more HOFs
reduce(numbers, 0, fn acc, x => acc + x end)
sortby(items, fn x => x.date end)
groupby(items, fn x => x.category end)
```

### What We're NOT Building (Yet)

- `Value::Function` variant (first-class functions)
- Escaping closures (storing/returning lambdas)
- Lazy evaluation / iterator pipelines
- Concurrent map operations
- Lambda serialization

---

## Phase 1: Foundation with Inline Lambdas (1-2 days)

### Goal
Map and filter working with both string references AND inline `fn` lambdas.

### Grammar Unification (Do First!)

Update `def` syntax to include `end` for consistency with `fn`:

**Before:**
```dsl
def factorial(n) => n * factorial(n - 1)
```

**After:**
```dsl
def factorial(n) => n * factorial(n - 1) end
```

**Why:** Creates perfect symmetry with lambdas:
- `def name(params) => expr end` - Named function  
- `fn params => expr end` - Anonymous function

**Impact:** Breaking change - requires updating all existing `.dsl` files. See `GRAMMAR_UNIFICATION_MIGRATION.md` for migration guide.

### IR Changes (`crates/dsl-ir/src/lib.rs`)

```rust
// Add new IR node variants
#[derive(Debug, Clone, PartialEq)]
pub enum IRNode {
    // ... existing variants
    
    Map {
        list: Box<IRNode>,
        callable: CallableRef,
    },
    Filter {
        list: Box<IRNode>,
        callable: CallableRef,
    },
}

// New types
#[derive(Debug, Clone, PartialEq)]
pub enum CallableRef {
    FunctionName(String),
    Lambda(LambdaIR),  // For Phase 2
}

#[derive(Debug, Clone, PartialEq)]
pub struct LambdaIR {
    pub params: Vec<String>,
    pub body: Box<IRNode>,
}
```

### Grammar Changes (`tree-sitter-dsl/grammar.js`)

```javascript
// Update function definition to require 'end'
function_definition: $ => seq(
  'def',
  $.identifier,
  $.parameter_list,  // or pattern for pattern matching
  '=>',
  $._expression,
  'end'  // Now required!
),

// Add inline lambda (same structure as def, but anonymous)
inline_lambda: $ => seq(
  'fn',
  commaSep1($.identifier),  // Parameters
  '=>',
  $._expression,
  'end'
),

// Update primary expression to include inline lambdas
primary_expression: $ => choice(
  // ... existing choices
  $.inline_lambda,
),
```

### Parser Changes (`crates/dsl-parser/src/parser.rs`)

```rust
// Update function call parsing to recognize HOF intrinsics
fn parse_function_call(&mut self, name: String) -> Result<Expr> {
    match name.as_str() {
        "map" | "filter" => self.parse_hof_call(name),
        _ => self.parse_normal_call(name),
    }
}

fn parse_hof_call(&mut self, name: String) -> Result<Expr> {
    self.expect(Token::LeftParen)?;
    let list = self.parse_expr()?;
    self.expect(Token::Comma)?;
    
    let callable = match self.current_token() {
        // String reference to user function
        Token::String(func_name) => {
            self.advance();
            CallableRef::FunctionName(func_name.clone())
        }
        
        // Inline lambda: fn x => expr end
        Token::Fn => {
            self.parse_inline_lambda()?
        }
        
        _ => return Err("Expected function name or 'fn' lambda".into()),
    };
    
    self.expect(Token::RightParen)?;
    
    Ok(match name.as_str() {
        "map" => Expr::Map { list: Box::new(list), callable },
        "filter" => Expr::Filter { list: Box::new(list), callable },
        _ => unreachable!(),
    })
}

fn parse_inline_lambda(&mut self) -> Result<CallableRef> {
    self.expect(Token::Fn)?;
    
    // Parse parameter list (comma-separated identifiers)
    let mut params = vec![];
    loop {
        let param = self.expect_identifier()?;
        params.push(param);
        
        if self.peek() == Token::Arrow {
            break;
        }
        
        self.expect(Token::Comma)?;
    }
    
    self.expect(Token::Arrow)?;  // =>
    
    // Parse body expression
    let body = self.parse_expr()?;
    
    self.expect(Token::End)?;
    
    Ok(CallableRef::Lambda(LambdaIR {
        params,
        body: Box::new(body),
    }))
}
```

### Lowering Changes (`crates/dsl-core/src/lowering.rs`)

```rust
// Add to lower_expr
fn lower_expr(&mut self, expr: &Expr) -> Result<IRNode> {
    match expr {
        // ... existing cases
        
        Expr::Map { list, callable } => {
            Ok(IRNode::Map {
                list: Box::new(self.lower_expr(list)?),
                callable: self.lower_callable(callable)?,
            })
        }
        
        Expr::Filter { list, callable } => {
            Ok(IRNode::Filter {
                list: Box::new(self.lower_expr(list)?),
                callable: self.lower_callable(callable)?,
            })
        }
    }
}

fn lower_callable(&mut self, callable: &CallableRef) -> Result<CallableRef> {
    match callable {
        CallableRef::FunctionName(name) => {
            Ok(CallableRef::FunctionName(name.clone()))
        }
        CallableRef::Lambda(lambda) => {
            // Phase 2
            todo!()
        }
    }
}
```

### Interpreter Changes (`crates/dsl-interpreter/src/interpreter.rs`)

```rust
impl Interpreter {
    pub async fn eval(&mut self, node: &IRNode) -> Result<Value> {
        match node {
            // ... existing cases
            
            IRNode::Map { list, callable } => {
                self.eval_map(list, callable).await
            }
            
            IRNode::Filter { list, callable } => {
                self.eval_filter(list, callable).await
            }
            
            // ... rest
        }
    }
    
    async fn eval_map(&mut self, list_node: &IRNode, callable: &CallableRef) -> Result<Value> {
        let list_value = self.eval(list_node).await?;
        let items = list_value.as_list()
            .ok_or_else(|| format!("map: expected List, got {:?}", list_value.type_name()))?;
        
        let mut results = Vec::with_capacity(items.len());
        
        for (idx, item) in items.iter().enumerate() {
            let result = self.call_callable(callable, vec![item.clone()], idx, "map").await
                .map_err(|e| format!("map failed at index {}: {}", idx, e))?;
            results.push(result);
        }
        
        Ok(Value::List(results))
    }
    
    async fn eval_filter(&mut self, list_node: &IRNode, callable: &CallableRef) -> Result<Value> {
        let list_value = self.eval(list_node).await?;
        let items = list_value.as_list()
            .ok_or_else(|| format!("filter: expected List, got {:?}", list_value.type_name()))?;
        
        let mut results = Vec::new();
        
        for (idx, item) in items.iter().enumerate() {
            let result = self.call_callable(callable, vec![item.clone()], idx, "filter").await
                .map_err(|e| format!("filter failed at index {}: {}", idx, e))?;
            
            let keep = result.as_bool()
                .ok_or_else(|| format!("filter: expected Bool at index {}, got {:?}", idx, result.type_name()))?;
            
            if keep {
                results.push(item.clone());
            }
        }
        
        Ok(Value::List(results))
    }
    
    async fn call_callable(
        &mut self,
        callable: &CallableRef,
        args: Vec<Value>,
        idx: usize,
        context: &str,
    ) -> Result<Value> {
        match callable {
            CallableRef::FunctionName(name) => {
                self.call_function_by_name(name, args).await
            }
            CallableRef::Lambda(_lambda) => {
                // Phase 2
                todo!()
            }
        }
    }
    
    async fn call_function_by_name(&mut self, name: &str, args: Vec<Value>) -> Result<Value> {
        // Check function_groups for matching arity
        if let Some(group) = self.runtime.function_groups.get(name) {
            // Find function with matching arity
            let matching = group.functions.iter()
                .find(|f| f.params.len() == args.len())
                .ok_or_else(|| format!("No function '{}' with {} arguments", name, args.len()))?;
            
            return self.call_user_function(matching, args).await;
        }
        
        Err(format!("Unknown function: {}", name).into())
    }
    
    async fn call_user_function(&mut self, func: &IRFunction, args: Vec<Value>) -> Result<Value> {
        // Push new scope
        self.push_scope();
        
        // Bind parameters
        for (param, arg) in func.params.iter().zip(args.iter()) {
            self.set_variable(param, arg.clone());
        }
        
        // Evaluate body
        let result = self.eval(&func.body).await;
        
        // Pop scope
        self.pop_scope();
        
        result
    }
}
```

### Testing

```dsl
// test_map_filter.dsl

// Define helper functions for string ref tests
def extractTitle(item) => item.title
def isHighScore(item) => item.score > 50

// Test data
def testData() => [
    {title: "First", score: 60},
    {title: "Second", score: 30},
    {title: "Third", score: 80}
]

// Test map with string ref
def testMapString() => 
    testData() |> map(_, "extractTitle")
// Expected: ["First", "Second", "Third"]

// Test map with inline lambda
def testMapLambda() =>
    testData() |> map(_, fn item => item.title end)
// Expected: ["First", "Second", "Third"]

// Test map with expression in lambda
def testMapExpr() =>
    testData() |> map(_, fn x => x.title + "!" end)
// Expected: ["First!", "Second!", "Third!"]

// Test filter with string ref
def testFilterString() => 
    testData() |> filter(_, "isHighScore")
// Expected: [{title: "First", score: 60}, {title: "Third", score: 80}]

// Test filter with inline lambda
def testFilterLambda() =>
    testData() |> filter(_, fn x => x.score > 50 end)
// Expected: [{title: "First", score: 60}, {title: "Third", score: 80}]

// Test lexical capture in lambda
def testCapture() =>
    let threshold = 50 in
    testData() |> filter(_, fn x => x.score > threshold end)
// Expected: [{title: "First", score: 60}, {title: "Third", score: 80}]

// Test chaining
def testChain() =>
    testData() 
    |> filter(_, fn x => x.score > 40 end)
    |> map(_, fn x => x.title end)
// Expected: ["First", "Third"]
```

### Acceptance Criteria

- [ ] `map(list, "funcName")` compiles to `IRNode::Map`
- [ ] `filter(list, "funcName")` compiles to `IRNode::Filter`
- [ ] `fn x => expr end` syntax compiles to `CallableRef::Lambda`
- [ ] Single-arg lambdas work: `fn x => x.field end`
- [ ] Multi-arg lambdas work: `fn x, y => x + y end`
- [ ] Runtime resolves function by arity
- [ ] Error messages include index and context
- [ ] Arity mismatch produces clear error
- [ ] Unknown function produces clear error
- [ ] Filter validates Bool return type
- [ ] Lambda can access outer scope (lexical capture)
- [ ] Map/filter can chain with pipes
- [ ] Complex expressions in lambda body work

---

## Phase 2: SortBy, GroupBy, Reduce (1-2 days)

### Goal
Extend HOF support to sorting, grouping, and reduction using same lambda syntax.

### Parser Changes

```rust
// Extend parse_hof_call to handle sortby/groupby/reduce
fn parse_hof_call(&mut self, name: String) -> Result<Expr> {
    match name.as_str() {
        "sortby" | "groupby" => {
            // sortby(list, key_fn)
            self.expect(Token::LeftParen)?;
            let list = self.parse_expr()?;
            self.expect(Token::Comma)?;
            let callable = self.parse_callable()?;
            self.expect(Token::RightParen)?;
            
            Ok(match name.as_str() {
                "sortby" => Expr::SortBy { list: Box::new(list), key_fn: callable },
                "groupby" => Expr::GroupBy { list: Box::new(list), key_fn: callable },
                _ => unreachable!(),
            })
        }
        
        "reduce" => {
            // reduce(list, init, reducer)
            self.expect(Token::LeftParen)?;
            let list = self.parse_expr()?;
            self.expect(Token::Comma)?;
            let init = self.parse_expr()?;
            self.expect(Token::Comma)?;
            let callable = self.parse_callable()?;
            self.expect(Token::RightParen)?;
            
            Ok(Expr::Reduce {
                list: Box::new(list),
                init: Box::new(init),
                reducer: callable,
            })
        }
        
        _ => { /* ... existing map/filter ... */ }
    }
}

// Helper to parse callable (string ref or lambda)
fn parse_callable(&mut self) -> Result<CallableRef> {
    match self.current_token() {
        Token::String(name) => {
            self.advance();
            Ok(CallableRef::FunctionName(name.clone()))
        }
        Token::Fn => self.parse_inline_lambda(),
        _ => Err("Expected function name or 'fn' lambda".into()),
    }
}
```

### IR Changes

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum IRNode {
    // ... existing
    
    SortBy {
        list: Box<IRNode>,
        key_fn: CallableRef,
    },
    
    GroupBy {
        list: Box<IRNode>,
        key_fn: CallableRef,
    },
    
    Reduce {
        list: Box<IRNode>,
        init: Box<IRNode>,
        reducer: CallableRef,  // Must be arity-2
    },
}
```

### Interpreter Implementation

```rust
async fn eval_sortby(&mut self, list_node: &IRNode, key_fn: &CallableRef) -> Result<Value> {
    let list_value = self.eval(list_node).await?;
    let items = list_value.as_list()?;
    
    // Phase 1: Precompute keys ONCE (critical for performance)
    let mut keyed_items: Vec<(Value, Value)> = Vec::with_capacity(items.len());
    
    for (idx, item) in items.iter().enumerate() {
        let key = self.call_callable(key_fn, vec![item.clone()], idx, "sortby").await
            .map_err(|e| format!("sortby key computation at index {}: {}", idx, e))?;
        
        // Validate key is sortable (not Float with NaN for now)
        self.validate_sortable_key(&key)?;
        
        keyed_items.push((key, item.clone()));
    }
    
    // Phase 2: Stable sort by precomputed keys
    keyed_items.sort_by(|(k1, _), (k2, _)| {
        self.compare_values(k1, k2).expect("Validated sortable keys")
    });
    
    // Phase 3: Extract sorted items
    let sorted = keyed_items.into_iter()
        .map(|(_, item)| item)
        .collect();
    
    Ok(Value::List(sorted))
}

async fn eval_groupby(&mut self, list_node: &IRNode, key_fn: &CallableRef) -> Result<Value> {
    let list_value = self.eval(list_node).await?;
    let items = list_value.as_list()?;
    
    // Use IndexMap to preserve insertion order
    let mut groups: IndexMap<Value, Vec<Value>> = IndexMap::new();
    
    for (idx, item) in items.iter().enumerate() {
        let key = self.call_callable(key_fn, vec![item.clone()], idx, "groupby").await
            .map_err(|e| format!("groupby key computation at index {}: {}", idx, e))?;
        
        // Validate key is hashable (disallow Float for now)
        self.validate_hashable_key(&key)?;
        
        groups.entry(key)
            .or_insert_with(Vec::new)
            .push(item.clone());
    }
    
    // Convert to Map of Lists
    let result = groups.into_iter()
        .map(|(k, v)| (self.value_to_map_key(k), Value::List(v)))
        .collect();
    
    Ok(Value::Map(result))
}

async fn eval_reduce(&mut self, list_node: &IRNode, init_node: &IRNode, reducer: &CallableRef) -> Result<Value> {
    let list_value = self.eval(list_node).await?;
    let items = list_value.as_list()?;
    
    let mut accumulator = self.eval(init_node).await?;
    
    for (idx, item) in items.iter().enumerate() {
        accumulator = self.call_callable(
            reducer,
            vec![accumulator, item.clone()],
            idx,
            "reduce"
        ).await.map_err(|e| format!("reduce at index {}: {}", idx, e))?;
    }
    
    Ok(accumulator)
}

// Helper: Compare values for sorting
fn compare_values(&self, a: &Value, b: &Value) -> Result<std::cmp::Ordering> {
    use std::cmp::Ordering;
    
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => Ok(x.cmp(y)),
        (Value::Float(x), Value::Float(y)) => {
            // Handle NaN: treat as error for now
            if x.is_nan() || y.is_nan() {
                return Err("Cannot sort: NaN values not supported".into());
            }
            Ok(x.partial_cmp(y).unwrap())
        }
        (Value::String(x), Value::String(y)) => Ok(x.cmp(y)),
        (Value::Bool(x), Value::Bool(y)) => Ok(x.cmp(y)),
        _ => Err(format!("Cannot compare {:?} and {:?}", a.type_name(), b.type_name()).into())
    }
}

fn validate_sortable_key(&self, key: &Value) -> Result<()> {
    match key {
        Value::Int(_) | Value::String(_) | Value::Bool(_) => Ok(()),
        Value::Float(f) if !f.is_nan() => Ok(()),
        Value::Float(_) => Err("Cannot use NaN as sort key".into()),
        _ => Err(format!("Cannot sort by {:?}", key.type_name()).into())
    }
}

fn validate_hashable_key(&self, key: &Value) -> Result<()> {
    match key {
        Value::Int(_) | Value::String(_) | Value::Bool(_) | Value::Null => Ok(()),
        Value::Float(_) => Err("Cannot use Float as groupby key (use Int or String)".into()),
        _ => Err(format!("Cannot group by {:?}", key.type_name()).into())
    }
}
```

### Testing

```dsl
// test_sortby_groupby_reduce.dsl

def testData() => [
    {name: "Alice", age: 30, dept: "Eng"},
    {name: "Bob", age: 25, dept: "Sales"},
    {name: "Charlie", age: 35, dept: "Eng"},
    {name: "Diana", age: 28, dept: "Sales"}
]

// SortBy
def testSortByAge() =>
    testData() |> sortby(_, { _.age })
// Expected: Bob(25), Diana(28), Alice(30), Charlie(35)

def testSortByName() =>
    testData() |> sortby(_, { _.name })
// Expected: Alice, Bob, Charlie, Diana

// GroupBy
def testGroupByDept() =>
    testData() |> groupby(_, { _.dept })
// Expected: {"Eng": [Alice, Charlie], "Sales": [Bob, Diana]}

// Reduce
def testSum() =>
    [1, 2, 3, 4, 5] |> reduce(_, 0, fn acc, x => acc + x end)
// Expected: 15

def testProduct() =>
    [1, 2, 3, 4] |> reduce(_, 1, fn acc, x => acc * x end)
// Expected: 24

def testConcat() =>
    ["a", "b", "c"] |> reduce(_, "", fn acc, x => acc + x end)
// Expected: "abc"

// Complex pipeline
def testPipeline() =>
    testData()
    |> filter(_, { _.age >= 28 })
    |> sortby(_, { _.age })
    |> map(_, { _.name })
    |> reduce(_, "", {|acc, name| acc + name + ", "})
// Expected: "Diana, Alice, Charlie, "
```

### Acceptance Criteria

- [x] `sortby` precomputes keys only once
- [x] `sortby` is stable (preserves order of equal elements)
- [x] `sortby` validates sortable key types
- [x] `groupby` preserves insertion order (IndexMap)
- [x] `groupby` validates hashable keys (no Float)
- [x] `reduce` validates arity-2 callable
- [x] All HOFs work with both string refs and lambdas
- [x] Error messages include operation and index
- [x] Complex pipelines combining all HOFs work

---

## Phase 4: Polish & Edge Cases (0.5-1 day)

### Goals
- Comprehensive error messages
- Edge case handling
- Documentation
- Performance validation

### Error Message Improvements

```rust
// Context-rich errors
pub struct EvalError {
    pub message: String,
    pub operation: Option<String>,  // "map", "filter", "reduce"
    pub index: Option<usize>,
    pub function_name: Option<String>,
}

impl EvalError {
    pub fn hof_error(op: &str, idx: usize, func: &str, msg: String) -> Self {
        EvalError {
            message: format!("{} '{}' failed at index {}: {}", op, func, idx, msg),
            operation: Some(op.to_string()),
            index: Some(idx),
            function_name: Some(func.to_string()),
        }
    }
}
```

### Edge Cases to Handle

```rust
// Empty lists
assert_eq!(map([], { _.x }), []);
assert_eq!(filter([], { true }), []);
assert_eq!(reduce([], 0, {|a, x| a + x}), 0);

// Type mismatches
filter([1, 2, "oops"], { true })  // OK: filter doesn't care about item type
filter([1, 2, 3], { _ })          // ERROR: expected Bool, got Int

// Arity mismatches
map([1, 2], "twoArgFunc")         // ERROR: function expects 2 args
reduce([1, 2], 0, "oneArgFunc")   // ERROR: reducer expects 2 args

// Missing functions
map([1, 2], "doesNotExist")       // ERROR: unknown function

// Nested HOFs
map([[1, 2], [3, 4]], { map(_, {|x| x * 2}) })
// Expected: [[2, 4], [6, 8]]

// Closure capture
let multiplier = 3 in
map([1, 2, 3], { _ * multiplier })
// Expected: [3, 6, 9]
```

### Performance Validation

```rust
// Benchmark script
def largeFeed() => 
    rss("https://news.ycombinator.com/rss") |> _.items  // ~30 items

def benchmark() =>
    largeFeed()
    |> filter(_, { length(_.title) > 20 })
    |> map(_, { {title: _.title, len: length(_.title)} })
    |> sortby(_, { _.len })
    |> groupby(_, { _.len / 10 })

// Validate:
// - Keys computed only once in sortby
// - No unnecessary clones
// - Vec preallocated with capacity
```

### Documentation

Create `docs/higher_order_functions.md`:

```markdown
# Higher-Order Functions

## Available Functions

### map(list, function)
Apply a function to each element.

**Examples:**
```dsl
map([1, 2, 3], fn x => x * 2 end)      // [2, 4, 6]
map(items, "extractTitle")              // Call user function
map(items, fn x => x.title + "!" end)   // Inline lambda
```

### filter(list, predicate)
Keep elements where predicate returns true.

**Examples:**
```dsl
filter([1, 2, 3, 4], fn x => x > 2 end)  // [3, 4]
filter(items, fn x => x.score >= 50 end) // High scores only
```

### sortby(list, key_function)
Sort by computed key (stable sort).

**Examples:**
```dsl
sortby(items, fn x => x.date end)        // Sort by date
sortby(items, fn x => length(x.title) end) // Sort by title length
```

### groupby(list, key_function)
Group into map by computed key.

**Examples:**
```dsl
groupby(items, fn x => x.category end)   // Map of category -> items
groupby(items, fn x => x.score / 10 end) // Group by score ranges
```

### reduce(list, initial, reducer)
Reduce list to single value.

**Examples:**
```dsl
reduce([1,2,3], 0, fn acc, x => acc + x end)  // 6 (sum)
reduce(items, [], fn acc, item => [item.title, ...acc] end)  // Collect titles
```

## Lambda Syntax

**Inline lambdas** (reuses existing `=>` syntax from `def`):

```dsl
// Single parameter
map(items, fn x => x.field end)
filter(items, fn x => x.score > 50 end)

// Multiple parameters
reduce(nums, 0, fn acc, x => acc + x end)
```

**String references** to user-defined functions:

```dsl
def extractTitle(item) => item.title
def isHighScore(item) => item.score > 50

map(items, "extractTitle")
filter(items, "isHighScore")
```

## Limitations

- Lambdas cannot be stored or returned (non-escaping)
- No concurrent execution (I/O in lambdas runs sequentially)
- Float values cannot be used as groupby keys
- NaN values cannot be sorted

## Error Messages

All HOFs include context in errors:
```
map 'extractTitle' failed at index 12: expected Map, got String
filter: expected Bool at index 5, got Int
```
```

### Acceptance Criteria

- [x] All edge cases handled with clear errors
- [x] Empty list handling correct
- [x] Type validation comprehensive
- [x] Arity validation comprehensive
- [x] Documentation complete
- [x] Performance validated on 100+ item lists

---

## Implementation Checklist

### Phase 1: Map/Filter with String Refs + Inline Lambdas (1-2d)

**Step 0: Grammar Unification**
- [ ] Update grammar: `def` now requires `end`
- [ ] Add `fn` keyword to lexer
- [ ] Ensure `end` keyword exists in lexer
- [ ] Update all existing `.dsl` examples to use `end`
- [ ] Update all tests to use `end`
- [ ] Verify existing code still works

**Step 1: IR Changes**
- [ ] Add `IRNode::Map` and `IRNode::Filter`
- [ ] Add `CallableRef` enum with `FunctionName` and `Lambda` variants
- [ ] Add `LambdaIR` struct to IR

**Step 2: Grammar & Parser**
- [ ] Update grammar for `fn params => expr end` syntax
- [ ] Parse `map(list, "funcName")` syntax
- [ ] Parse `map(list, fn x => expr end)` syntax
- [ ] Handle single param: `fn x => expr end`
- [ ] Handle multi-param: `fn x, y => expr end`

**Step 3: Lowering**
- [ ] Lower both string refs and lambdas to IR
- [ ] Implement `eval_map` in interpreter
- [ ] Implement `eval_filter` in interpreter
- [ ] Implement `call_function_by_name` with arity resolution
- [ ] Implement `eval_lambda` with scope management
- [ ] Implement lexical scope capture (walk scope chain)
- [ ] Add arity validation for lambdas
- [ ] Add error messages with index and context
- [ ] Write tests for string refs
- [ ] Write tests for single-param lambdas
- [ ] Write tests for multi-param lambdas
- [ ] Write tests for lexical capture
- [ ] Write tests for complex expressions in lambda body
- [ ] Write tests for chaining map/filter
- [ ] Validate all tests pass

### Phase 2: SortBy/GroupBy/Reduce (1-2d)
- [ ] Add `IRNode::SortBy`, `IRNode::GroupBy`, `IRNode::Reduce`
- [ ] Parse sortby/groupby/reduce syntax
- [ ] Implement `eval_sortby` with key precomputation
- [ ] Implement stable sort
- [ ] Implement `compare_values` helper
- [ ] Implement `validate_sortable_key`
- [ ] Implement `eval_groupby` with IndexMap
- [ ] Implement `validate_hashable_key`
- [ ] Implement `eval_reduce` with arity-2 validation
- [ ] Write tests for sortby
- [ ] Write tests for groupby
- [ ] Write tests for reduce
- [ ] Write tests for complex pipelines
- [ ] Validate performance on 100+ items
- [ ] Validate all tests pass

### Phase 3: Polish (0.5-1d)
- [ ] Implement rich `EvalError` type
- [ ] Add context to all error messages
- [ ] Test all edge cases (empty lists, type mismatches, etc.)
- [ ] Write documentation
- [ ] Performance validation with real RSS feeds
- [ ] Add examples to docs
- [ ] Update README with HOF section
- [ ] Final integration testing

---

## Testing Strategy

### Unit Tests (Per Phase)
- Test each HOF in isolation
- Test error cases explicitly
- Test edge cases (empty, single item, large lists)

### Integration Tests
```dsl
// Real-world example: RSS processing
def topTechStories() =>
    rss("https://hnrss.org/newest")
    |> _.items
    |> filter(_, { contains(lower(_.title), "ai") or contains(lower(_.title), "llm") })
    |> map(_, { {
        title: _.title,
        score: length(_.description),
        link: _.link
    }})
    |> sortby(_, { _.score })
    |> reverse(_)
    |> take(_, 10)
```

### Performance Tests
- 1000-item list map/filter
- Nested HOFs
- Complex lambda expressions
- Verify key precomputation (add debug logging if needed)

---

## Risks & Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| Parser conflicts with existing grammar | High | Test incrementally; use bounded contexts for lambda blocks |
| Scope chain complexity | Medium | Keep scope stack simple (Vec of HashMaps); document clearly |
| Performance regression | Medium | Benchmark before/after; optimize key precomputation |
| Breaking existing code | High | Keep all existing builtins working; add HOFs alongside |
| Error message quality | Medium | Add comprehensive error context from day 1 |

---

## Future Enhancements (Post-MVP)

### Phase 5: Advanced Features (Optional)
- [ ] `map_concurrent(list, lambda, concurrency: Int)` for parallel LLM/HTTP
- [ ] Iterator pipelines for lazy evaluation
- [ ] `Value::Function` for first-class functions
- [ ] Comprehensions: `[x.title for x in items if x.score > 50]`
- [ ] Partial application: `map(_, add(5))` where `add` is curried

### When to Consider
- **Concurrent map:** User reports slow LLM processing in loops
- **Lazy evaluation:** User processes 10k+ item datasets
- **First-class functions:** User wants to return/store functions
- **Comprehensions:** User feedback shows lambda syntax too verbose

---

## Success Metrics

**Definition of Done:**
1. All 4 phases complete with tests passing
2. Documentation written and reviewed
3. Performance validated (no regression on existing code)
4. Real RSS workflow example works end-to-end
5. Error messages are clear and actionable

**Timeline:** 3-5 days total
- Phase 1: Map/Filter with string refs + lambdas (1-2 days)
- Phase 2: SortBy/GroupBy/Reduce (1-2 days)
- Phase 3: Polish & documentation (0.5-1 day)

**Ready to ship when:**
- [ ] Can process HN RSS feed with map/filter/sortby/groupby
- [ ] Errors include context and suggestions
- [ ] Performance acceptable for 100-1000 items
- [ ] Documentation clear for non-programmers
