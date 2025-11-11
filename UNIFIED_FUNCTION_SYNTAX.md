# Unified Function Syntax

## Overview

Both named functions (`def`) and anonymous functions (`fn`) use the same `=> expr end` structure.

## Syntax Comparison

### Named Functions (def)

```dsl
def add(x, y) => x + y end

def factorial(0) => 1 end
def factorial(n) => n * factorial(n - 1) end

def process(data) =>
    data 
    |> filter(_, fn x => x.score > 50 end)
    |> map(_, fn x => x.title end)
end
```

### Anonymous Functions (fn)

```dsl
// Single parameter
fn x => x * 2 end
fn item => item.title end

// Multiple parameters  
fn x, y => x + y end
fn acc, x => acc + x end

// Used in higher-order functions
map([1, 2, 3], fn x => x * 2 end)
filter(items, fn x => x.score > 50 end)
reduce(nums, 0, fn acc, x => acc + x end)
```

## Shared Structure

Both forms share:
- `=>` for separating parameters from body
- `end` for terminating the expression
- Same expression syntax in the body

```
Named:     def name(params) => expr end
Anonymous:      fn params  => expr end
                   ^^^^^^^^    ^^^^^^^^^^
                   Only difference is name vs anonymous
```

## Benefits

1. **Easy to learn:** If you know `def`, you know `fn`
2. **Clear boundaries:** `end` shows where function ends
3. **Consistent parsing:** Both use same parsing logic
4. **Natural transformation:**
   ```dsl
   // Named helper
   def isValid(x) => x.score > 50 end
   filter(items, "isValid")
   
   // Inline it
   filter(items, fn x => x.score > 50 end)
   ```

## Pattern Matching Works the Same

```dsl
def length([]) => 0 end
def length([_, ...rest]) => 1 + length(rest) end

// Could theoretically work with fn too (future)
fn [] => 0 end
fn [_, ...rest] => 1 + length(rest) end
```

## Multi-line Bodies

With complex expressions, `end` provides clear termination:

```dsl
def processUser(user) =>
    let validated = validate(user) in
    let enriched = enrich(validated) in
    let formatted = format(enriched) in
    save(formatted)
end

map(users, fn user =>
    let validated = validate(user) in
    let enriched = enrich(validated) in
    format(enriched)
end)
```

## Migration from Old Syntax

**Old (without `end`):**
```dsl
def add(x, y) => x + y
def factorial(n) => if n == 0 then 1 else n * factorial(n - 1)
```

**New (with `end`):**
```dsl
def add(x, y) => x + y end
def factorial(n) => if n == 0 then 1 else n * factorial(n - 1) end
```

Simple mechanical transformation: append ` end` to every `def` statement.

## Complete Example

```dsl
// Named helper functions
def extractTitle(item) => item.title end
def isHighScore(item) => item.score > 50 end
def formatScore(item) => item.title + " (" + string(item.score) + ")" end

// Using string references
def withNamedFunctions(items) =>
    items
    |> filter(_, "isHighScore")
    |> map(_, "formatScore")
end

// Using inline lambdas
def withInlineLambdas(items) =>
    items
    |> filter(_, fn x => x.score > 50 end)
    |> map(_, fn x => x.title + " (" + string(x.score) + ")" end)
end

// Mixed approach
def mixed(items, threshold) =>
    items
    |> filter(_, fn x => x.score > threshold end)  // Closure captures threshold
    |> map(_, "formatScore")  // Reuse named function
    |> sortby(_, fn x => length(x) end)
end
```

## Grammar Rules

```javascript
// Named function definition
function_definition: $ => seq(
  'def',
  $.identifier,
  $.parameter_list,
  '=>',
  $._expression,
  'end'
),

// Anonymous function (lambda)
inline_lambda: $ => seq(
  'fn',
  commaSep1($.identifier),
  '=>',
  $._expression,
  'end'
),
```

## IR Representation

Both lower to similar structures:

```rust
// Named function becomes a binding
FunctionDef {
    name: "add",
    params: ["x", "y"],
    body: IRNode::BinaryOp { op: Add, left: "x", right: "y" }
}

// Anonymous function becomes CallableRef::Lambda
CallableRef::Lambda(LambdaIR {
    params: ["x", "y"],
    body: IRNode::BinaryOp { op: Add, left: "x", right: "y" }
})
```

The body uses the exact same IR node structure!

## Why This Design

1. **Consistency:** Users don't learn two different syntaxes
2. **Discoverability:** If you can write `def`, you can write `fn`
3. **Refactoring:** Easy to extract inline lambda to named function
4. **Clarity:** `end` makes multi-line functions readable
5. **Parsing:** Simpler grammar with shared rules

## Comparison to Other Languages

**JavaScript:**
```javascript
function add(x, y) { return x + y }  // Named
(x, y) => x + y                       // Anonymous
```

**Elixir:**
```elixir
def add(x, y), do: x + y              # Named
fn x, y -> x + y end                  # Anonymous
```

**Our DSL:**
```dsl
def add(x, y) => x + y end            // Named
fn x, y => x + y end                  // Anonymous
```

We're closest to Elixir's design, which users find intuitive!
