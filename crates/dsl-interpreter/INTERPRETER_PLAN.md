# Interpreter Implementation Plan

## Overview
The Interpreter is a tree-walk evaluator that executes IRNode expressions and produces runtime Values.

## File Structure

### Main Struct
```rust
pub struct Interpreter {
    runtime: Runtime,
    builtins: BuiltinFunctions,
}
```

## Core Methods

### 1. Constructor & Initialization
- `new()` - Create new interpreter with empty state
- `from_ir(ir: &IR)` - Create interpreter and load IR (types, functions)

### 2. Main Evaluation Method
```rust
pub async fn eval(&mut self, node: &IRNode) -> Result<Value, String>
```

This is the main entry point that pattern matches on all IRNode variants.

## IRNode Evaluation Plan (13 Core Types)

### Simple Values (5 types)
1. **IRNode::String** → `Value::String`
   - Direct conversion, no computation

2. **IRNode::Int** → `Value::Int`
   - Direct conversion

3. **IRNode::Float** → `Value::Float`
   - Direct conversion

4. **IRNode::Bool** → `Value::Bool`
   - Direct conversion

5. **IRNode::TemplateString** → `Value::String`
   - Call `interpolate_template()` helper
   - Iterate segments, evaluate interpolations
   - Concatenate result

### Collections (2 types)
6. **IRNode::List** → `Value::List`
   - Recursively eval each item
   - Collect into Vec

7. **IRNode::Map** → `Value::Map`
   - Recursively eval each value expression
   - Build IndexMap preserving order

### Variables & Access (3 types)
8. **IRNode::Variable** → lookup from `runtime.vars`
   - Use `runtime.get_var(name)`
   - Return error if not found

9. **IRNode::FieldAccess** → navigate map
   - Call `access_field()` helper
   - Eval base, extract field from map

10. **IRNode::IndexAccess** → array/map indexing
    - Call `access_index()` helper
    - Eval base and index
    - Handle both list[int] and map[string]

### Operations (1 type)
11. **IRNode::BinaryOp** → apply operator
    - Call `apply_binary_op()` helper
    - Eval left and right
    - Handle: +, -, *, / for Int/Float
    - Handle type coercion (Int+Float → Float)
    - Handle string concatenation

### Control Flow (2 types)
12. **IRNode::Conditional** → ternary evaluation
    - Eval condition (must be Bool)
    - Eval then_expr or else_expr based on result

13. **IRNode::Sequential** → pipe operator
    - Eval left expression
    - Store result as "_" variable
    - Apply binding if present (Single or List destructuring)
    - Eval right expression (can reference "_" or bound vars)

### Parallel & Functions
14. **IRNode::Parallel** → parallel composition
    - Currently sequential (Note: DuckDB is !Sync)
    - Eval all expressions
    - Return single value if 1 expr, else List
    - Apply binding if present

15. **IRNode::FunctionCall** → call builtin or user function
    - Check if user-defined: `runtime.functions.get(name)`
    - If user: call `call_user_function()`
    - Else: eval args and call `builtins.call(name, args)`

16. **IRNode::TypeInstantiation** → create typed value
    - TODO: Not yet implemented
    - Will need to construct Map with type checking

## Helper Methods

### Template Interpolation
```rust
async fn interpolate_template(&mut self, segments: &[IRTemplateSegment])
    -> Result<String, String>
```
- Iterate segments
- For Text: append directly
- For Interpolation: eval expression, convert to string

### Field Access
```rust
fn access_field(&self, value: &Value, field: &str)
    -> Result<Value, String>
```
- Match on Map variant
- Return field or error

### Index Access
```rust
fn access_index(&self, value: &Value, index: &Value)
    -> Result<Value, String>
```
- Match on (List, Int) → array access
- Match on (Map, String) → map access
- Return error for invalid combinations

### Binary Operations
```rust
fn apply_binary_op(&self, op: &str, left: Value, right: Value)
    -> Result<Value, String>
```
- Match on value types
- (Int, Int) → Int arithmetic
- (Float, Float) → Float arithmetic
- (Int, Float) or (Float, Int) → Float (type coercion)
- (String, String) with "+" → concatenation
- Handle division by zero

### User Function Execution
```rust
async fn call_user_function(&mut self, func: &IRFunction, args: &[IRNode])
    -> Result<Value, String>
```
- Eval all argument expressions
- Check argument count matches params
- Save current vars
- Bind params to args
- Execute based on IRExecution type:
  - **LLM**: interpolate prompt, call `builtins.ask_with_config()` or `extract_as_with_config()`
  - **HTTP**: interpolate URL/body, make HTTP request, parse response
  - **SQL**: interpolate query, call `builtins.sql()`
  - **HTTPWithLLM**: chain HTTP then LLM
- Restore vars
- Return result

### String Interpolation for Functions
```rust
fn interpolate_string_template(&self, template: &str, params: &[String],
    arg_values: &[Value]) -> Result<String, String>
```
- Replace `${param}` with arg values
- Use `value.to_prompt_string()` for full serialization

## Error Handling Strategy
- Use `Result<Value, String>` for all operations
- Provide descriptive error messages with context
- Include type information in errors
- Use `?` operator for propagation

## Async Strategy
- Use `Box::pin()` for recursive async calls (prevents stack overflow)
- All eval methods are `async` to support:
  - HTTP requests (reqwest)
  - LLM calls (simplify_baml)
  - SQL queries (potentially async in future)

## Key Design Decisions

### 1. Sequential Parallel Execution
Despite `||` suggesting parallelism, we execute sequentially because:
- DuckDB Connection is !Sync (uses RefCell)
- Can't share across threads even with Arc<Mutex<>>
- Async concurrency within single task already good for I/O

### 2. Binding Variables
Two types:
- **Single**: `expr as name` → store value
- **List**: `expr as [a, b, c]` → destructure list items

### 3. Special Variables
- `_` always stores last result (from Sequential/Parallel)
- Available in right side of pipe: `5 |> _ + 10`

### 4. Pointer Equality Check
In Sequential, check if `left == right` (pointer equality):
- Happens with simple bindings: `expr as name`
- Avoids infinite recursion
- Return left result without eval right

## Implementation Order

1. ✅ Create struct and constructor
2. ✅ Implement simple value evaluations (String, Int, Float, Bool)
3. ✅ Implement collections (List, Map)
4. ✅ Implement variable lookup
5. ✅ Implement access operations (Field, Index)
6. ✅ Implement binary operations
7. ✅ Implement control flow (Conditional)
8. ✅ Implement Sequential with binding
9. ✅ Implement Parallel with binding
10. ✅ Implement FunctionCall routing
11. ✅ Implement template interpolation
12. ✅ Implement user function execution (all 4 types)
13. ✅ Implement helper methods
14. ⏳ Add comprehensive tests

## Code Size Estimate
- Main eval() match: ~250 lines
- Helper methods: ~300 lines
- User function execution: ~200 lines
- Tests: ~200 lines
- **Total**: ~950 lines

## References to Existing Code
- Source: `crates/dsl-core/src/eval/evaluator.rs` (lines 155-600)
- Preserve exact semantics from existing evaluator
- Match error messages and behavior
