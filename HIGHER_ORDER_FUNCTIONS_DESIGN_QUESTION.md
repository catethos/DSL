# Higher-Order Functions Design Question

## Context

I'm building a DSL for data processing workflows (with LLM, HTTP, SQL, and RSS capabilities). The language currently has:

1. **Pattern matching with function overloading:**
   ```
   def factorial(0) => 1
   def factorial(n) => n * factorial(n - 1)
   ```

2. **Sequential composition with bindings:**
   ```
   rss("https://example.com/feed") |> _.items |> pluck(_, "title")
   ```

3. **Value types:** String, Int, Float, Bool, List, Map, Null, Markdown, Image
   - **No Function type yet**

4. **Current "higher-order" functions** that only work with field names:
   - `pluck(list, "field_name")` - extract field from maps
   - `filter(list, "field_name", value)` - filter by field equality
   - `sortby(list, "field_name")` - sort by field
   - `groupby(list, "field_name")` - group by field value

## Current Architecture

```rust
// Values (in dsl-ir crate)
pub enum Value {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    List(Vec<Value>),
    Map(IndexMap<String, Value>),
    Null,
    Markdown(String),
    Image(String),
    // No Function variant!
}

// Functions (in dsl-core crate)
pub struct FunctionDef {
    pub name: String,
    pub params: Vec<String>,
    pub return_type: Option<FieldType>,
    pub execution: FunctionExecution,
}

pub enum FunctionExecution {
    Expression { body: Box<Expr> },
    LLM { prompt: String, model: Option<String>, ... },
    HTTP { method: String, url: String, ... },
    SQL { query: String },
}

// Runtime (in dsl-interpreter crate)
pub struct Runtime {
    pub functions: HashMap<String, IRFunction>,
    pub function_groups: HashMap<String, IRFunctionGroup>, // overloaded
    pub types: TypeRegistry,
    // ... variable scopes
}

// Builtins don't have access to Runtime/Interpreter
pub struct BuiltinFunctions {
    runtime: Option<BamlRuntime>,    // for LLM
    llm_client: Option<LLMClient>,
    sql_executor: Option<SQLExecutor>,
    // NO access to user-defined functions!
}
```

## The Problem

Users want true higher-order functions:
```
// Desired syntax (not yet possible):
map(items, fn(x) { x.title })
filter(items, fn(x) { x.score > 50 })
reduce(numbers, 0, fn(acc, x) { acc + x })
```

## Design Questions

### 1. **First-Class Functions: How to Represent?**

**Option A: Add Function Value Type**
```rust
pub enum Value {
    // ... existing types
    Function(FunctionValue),
}

pub struct FunctionValue {
    name: String,
    params: Vec<String>,
    body: Box<IRNode>,  // Compiled IR
    closure: HashMap<String, Value>,  // Captured variables
}
```
- **Pros:** Clean, enables closures, functional programming idioms
- **Cons:** Complexity, memory overhead, serialization issues

**Option B: String-Based Function References**
```dsl
// Pass function name as string
map(items, "extractTitle")  // calls user-defined extractTitle(item)
```
- **Pros:** Simple, no new types, easy serialization
- **Cons:** No closures, no inline lambdas, runtime errors

**Option C: Intrinsic Approach (like `__http`, `__llm`)**
```rust
// Compiler generates special IR for higher-order operations
IRNode::Map { list: Box<IRNode>, function_name: String }
// Interpreter handles specially, can access runtime functions
```
- **Pros:** No Value changes, interpreter has full context
- **Cons:** Grammar changes, limited to predefined operations

**Which approach fits best for a data-processing DSL? Are there hybrid approaches?**

---

### 2. **Builtin Access to User Functions**

Current builtins (in `BuiltinFunctions` struct) can't call user-defined functions. To implement `map`:

**Option A: Move Higher-Order Functions to Interpreter**
```rust
impl Interpreter {
    async fn builtin_map(&mut self, list: &[IRNode], func_name: &str) -> Result<Value> {
        // Can access self.runtime.functions
        let func = self.runtime.functions.get(func_name)?;
        // Can call self.call_user_function()
    }
}
```

**Option B: Pass Interpreter/Runtime to Builtins**
```rust
pub struct BuiltinFunctions<'a> {
    runtime_ref: Option<&'a Runtime>,
    // ...
}
```

**Option C: Callback Pattern**
```rust
pub struct BuiltinFunctions {
    function_caller: Box<dyn Fn(&str, Vec<Value>) -> Result<Value>>,
}
```

**Which pattern is most maintainable? How do other interpreted languages handle this?**

---

### 3. **Anonymous Functions / Lambdas**

If we support lambdas, what syntax?

**Option A: Inline Functions**
```dsl
map(items, fn(x) { x.title + "!" })
```

**Option B: Arrow Syntax**
```dsl
map(items, x => x.title + "!")
```

**Option C: Block Syntax**
```dsl
map(items, { _.title + "!" })  // _ is the implicit parameter
```

**Given the existing grammar (arrow functions for `def`), which fits best?**

---

### 4. **Performance & Memory Considerations**

For data processing (RSS feeds, SQL results, etc.):
- Lists can have 100-1000+ items
- Functions might be called millions of times
- Users might chain operations: `items |> filter(...) |> map(...) |> sortby(...)`

**Questions:**
- Should we optimize for iterator-style lazy evaluation?
- Should closures be reference-counted or cloned?
- Should we compile lambdas once or re-parse each time?

---

### 5. **Type Safety**

Current DSL has optional type annotations but runtime type checking:
```dsl
def process(x: String) -> Int => length(x)
```

With higher-order functions:
```dsl
map(items, fn(x: Item) -> String { x.title })
```

**Questions:**
- Should lambda parameters be typed?
- Should `map`'s return type be inferred?
- How much type safety vs flexibility for a data DSL?

---

### 6. **Serialization & Remote Execution**

The DSL supports:
- HTTP execution: `method: POST, url: "...", body: "..."`
- LLM execution: `prompt: "..."`

**Question:** If we add Function values, do we need to serialize them?
- For caching HTTP responses?
- For passing to LLM as context?
- For storing in database query results?

---

### 7. **Scope & Practical Tradeoffs**

**Current workaround** (using pattern matching):
```dsl
def extractTitle([]) => []
def extractTitle([item, ...rest]) => [item.title, ...extractTitle(rest)]

items |> extractTitle(_)
```

**Questions:**
- Is the pattern-matching approach "good enough" for a data DSL?
- Should we prioritize other features (async/await, error handling, modules)?
- What's the minimum viable higher-order function support?

---

## What I'm Looking For

1. **Architectural guidance:** Which approach fits best given the existing architecture?
2. **Prior art:** How do similar DSLs (Elixir, Lua, Dhall, Jsonnet) handle this?
3. **Pragmatic advice:** What's the simplest path that covers 80% of use cases?
4. **Pitfalls:** What mistakes should I avoid based on your experience?
5. **Incremental path:** Can I ship something useful without full closures?

## Additional Context

- **Target users:** Data analysts, not necessarily programmers
- **Primary use case:** RSS/API data → transform → LLM → output
- **Performance:** Currently synchronous async execution (no true parallelism yet)
- **Deployment:** Compiled to binary, runs locally (not VM/bytecode)

---

**What would you recommend as the best path forward?**
