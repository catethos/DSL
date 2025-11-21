# Interactive REPL Tour

This comprehensive tour covers all REPL features with hands-on examples.

## Starting the REPL

```bash
cargo run --release --bin dsl
```

Press **F1** to enter full-screen REPL mode.

## Tutorial 1: Basic REPL Usage (2 minutes)

### Simple Expressions

```dsl
flow> 42
✓ 42 : Int

flow> "hello world"
✓ "hello world" : String

flow> 3.14
✓ 3.14 : Float

flow> true
✓ true : Bool

flow> [1, 2, 3, 4, 5]
✓ [1, 2, 3, 4, 5] : List
```

### Arithmetic Operations

```dsl
flow> 10 + 5
✓ 15 : Int

flow> 20 * 3
✓ 60 : Int

flow> 100 / 4
✓ 25 : Int

flow> 3.14 * 2
✓ 6.28 : Float

flow> 2 ^ 8
✓ 256 : Int
```

## Tutorial 2: Variables and Binding (3 minutes)

### Variable Binding with `as`

```dsl
flow> 42 as answer
✓ Bound 'answer' to 42 : Int

flow> answer
✓ 42 : Int

flow> answer * 2
✓ 84 : Int
```

### The Underscore `_` Variable

The `_` variable always holds the last result:

```dsl
flow> 100
✓ 100 : Int

flow> _
✓ 100 : Int

flow> _ + 50
✓ 150 : Int

flow> _ * 2
✓ 300 : Int
```

```admonish tip
The `_` variable is perfect for quick calculations without naming intermediate results.
```

### List Indexing

```dsl
flow> [10, 20, 30, 40, 50] as numbers
✓ Bound 'numbers' to [10, 20, 30, 40, 50] : List

flow> numbers[0]
✓ 10 : Int

flow> numbers[2]
✓ 30 : Int

flow> numbers[-1]
✓ 50 : Int
```

### Check All Variables

```dsl
flow> :vars
✓ Variables:
  answer = 42 : Int
  numbers = [10, 20, 30, 40, 50] : List
  _ = 50 : Int
```

## Tutorial 3: Built-in Functions (3 minutes)

### String Operations

```dsl
flow> Upper("hello world")
✓ "HELLO WORLD" : String

flow> Lower("HELLO WORLD")
✓ "hello world" : String

flow> Length("hello")
✓ 5 : Int

flow> Join(["a", "b", "c"], "-")
✓ "a-b-c" : String

flow> Split("a-b-c", "-")
✓ ["a", "b", "c"] : List

flow> Replace("hello world", "world", "DSL")
✓ "hello DSL" : String
```

### List Operations

```dsl
flow> [1, 2, 3, 4, 5] as nums
✓ Bound 'nums' to [1, 2, 3, 4, 5] : List

flow> Length(nums)
✓ 5 : Int

flow> Sum(nums)
✓ 15 : Int

flow> Avg(nums)
✓ 3.0 : Float

flow> Min(nums)
✓ 1 : Int

flow> Max(nums)
✓ 5 : Int
```

### Math Operations

```dsl
flow> Abs(-42)
✓ 42 : Int

flow> Round(3.7)
✓ 4 : Int

flow> Floor(3.7)
✓ 3 : Int

flow> Ceil(3.2)
✓ 4 : Int

flow> Sqrt(16)
✓ 4.0 : Float
```

## Tutorial 4: Pipeline Operator (3 minutes)

The `|>` operator chains operations left-to-right:

### Basic Pipelines

```dsl
flow> "hello" |> Upper(_)
✓ "HELLO" : String

flow> "hello" |> Upper(_) |> Length(_)
✓ 5 : Int

flow> [1, 2, 3, 4, 5] |> Length(_)
✓ 5 : Int

flow> [1, 2, 3, 4, 5] |> Sum(_)
✓ 15 : Int
```

### Pipeline with Binding

```dsl
flow> ["a", "b", "c"] |> Join(_, "-") as joined
✓ Bound 'joined' to "a-b-c" : String

flow> joined |> Upper(_)
✓ "A-B-C" : String
```

### Multi-Step Pipelines

```dsl
flow> "hello world"
    |> Upper(_)
    |> Split(_, " ")
    |> Join(_, "-")
✓ "HELLO-WORLD" : String

flow> [1, 2, 3, 4, 5]
    |> map(_, fn n => n * 2 end)
    |> filter(_, fn n => n > 5 end)
    |> Sum(_)
✓ 24 : Int
```

```admonish tip
Multi-line pipelines make complex transformations readable. Use **Shift+Enter** for multi-line input.
```

## Tutorial 5: Higher-Order Functions (4 minutes)

### Map

Transform each element in a list:

```dsl
flow> map([1, 2, 3], fn n => n * 2 end)
✓ [2, 4, 6] : List

flow> Map(["a", "b", "c"], Upper)
✓ ["A", "B", "C"] : List
```

### Filter

Keep only elements that match a condition:

```dsl
flow> filter([1, 2, 3, 4, 5], fn n => n > 2 end)
✓ [3, 4, 5] : List

flow> filter([1, 2, 3, 4, 5], fn n => n % 2 == 0 end)
✓ [2, 4] : List
```

### Reduce

Combine all elements into a single value:

```dsl
flow> reduce([1, 2, 3, 4, 5], 0, fn acc, n => acc + n end)
✓ 15 : Int

flow> reduce(["a", "b", "c"], "", fn acc, s => acc + s end)
✓ "abc" : String
```

### Combining Higher-Order Functions

```dsl
flow> [1, 2, 3, 4, 5]
    |> filter(_, fn n => n > 2 end)
    |> map(_, fn n => n * 2 end)
    |> reduce(_, 0, fn acc, n => acc + n end)
✓ 24 : Int
```

## Tutorial 6: Parallel Execution (2 minutes)

Use `par()` to run operations concurrently:

### Basic Parallel

```dsl
flow> par(5, 10, 15)
✓ [5, 10, 15] : List

flow> par(5, 10, 15) as numbers
✓ Bound 'numbers' to [5, 10, 15] : List
```

### Parallel Destructuring

```dsl
flow> par(10, 20, 30) as [a, b, c]
✓ Bound 'a' to 10 : Int
✓ Bound 'b' to 20 : Int
✓ Bound 'c' to 30 : Int

flow> a + b + c
✓ 60 : Int
```

### Parallel Function Calls

```dsl
flow> par(
    Sum([1, 2, 3]),
    Sum([4, 5, 6]),
    Sum([7, 8, 9])
) as results
✓ Bound 'results' to [6, 15, 24] : List

flow> Sum(results)
✓ 45 : Int
```

## Tutorial 7: LLM Integration (Requires API Key)

```admonish warning
You need an OpenAI API key for this tutorial:
`export OPENAI_API_KEY="sk-..."`
```

### Simple LLM Call

```dsl
flow> Ask("What is the capital of France?")
✓ "The capital of France is Paris." : String
```

### LLM with Pipeline

```dsl
flow> "Explain Rust in one sentence" |> Ask(_)
✓ "Rust is a systems programming language..." : String
```

### LLM with Variables

```dsl
flow> "quantum computing" as topic
✓ Bound 'topic' to "quantum computing" : String

flow> "Explain ${topic} in simple terms" |> Ask(_)
✓ "Quantum computing uses quantum mechanics..." : String
```

## Special REPL Commands

### :vars - List Variables

```dsl
flow> :vars
✓ Variables:
  answer = 42 : Int
  numbers = [10, 20, 30, 40, 50] : List
  topic = "quantum computing" : String
  _ = "Quantum computing uses..." : String
```

### :types - List Type Definitions

```dsl
flow> :types
✓ Type definitions:
  (no types defined yet)
```

### :funcs - List User Functions

```dsl
flow> :funcs
✓ User-defined functions:
  (no functions defined yet)
```

### :help - Show Help

```dsl
flow> :help
✓ DSL REPL Help
  Commands:
    :vars  - List all variables
    :types - List type definitions
    :funcs - List user functions
    :help  - Show this help
    :copy  - Save result to file
    :save  - Save session
    :load  - Load session
    :debug - Toggle debug mode
```

### :copy - Save Result to File

```dsl
flow> "Hello World" |> Upper(_)
✓ "HELLO WORLD" : String

flow> :copy output.txt
✓ Result saved to output.txt
```

### :save/:load - Session Management

```dsl
flow> :save my-session.json
✓ Session saved to my-session.json

flow> :load my-session.json
✓ Session loaded from my-session.json
```

## UI Modes

### REPL Mode (F1)

Full-screen interactive REPL.

**Best for:**
- Quick experimentation
- Testing expressions
- Learning the language

**Key bindings:**
- **Enter** - Execute command
- **Shift+Enter** - Multi-line input
- **Up/Down** - History navigation
- **Esc** - Quit

### Workspace Mode (F2)

Three-pane layout: Editor + REPL + Preview.

**Best for:**
- Writing multi-line programs
- Developing functions and types
- Seeing execution results

**Key bindings:**
- **Tab** - Switch between panes
- **Ctrl+R** - Run editor content in REPL
- **Ctrl+S** - Save file
- **Esc** - Quit

### Type Explorer (F3)

Browse all registered types and their fields.

**Best for:**
- Viewing type definitions
- Understanding data structures
- Quick reference

## Tips and Tricks

### Tip 1: Use History

```admonish tip
Press **Up** to recall previous commands. Press **Down** to navigate forward through history.
```

### Tip 2: Multi-line Input

```admonish tip
Use **Shift+Enter** to add new lines without executing:

flow> [1, 2, 3, 4, 5]
    |> filter(_, fn n => n > 2 end)
    |> Sum(_)
```

### Tip 3: Quick Calculations

```admonish tip
Use `_` for chained calculations:

flow> 100
flow> _ + 50
flow> _ * 2
flow> _ / 3
```

### Tip 4: Template Strings

```admonish tip
Use `${}` for variable interpolation:

flow> "Alice" as name
flow> "Hello, ${name}!"
✓ "Hello, Alice!" : String
```

### Tip 5: Save Frequently

```admonish tip
Use `:save session.json` to save your work. The REPL doesn't auto-save!
```

## Common REPL Workflows

### Workflow 1: Exploration

```
1. Start with simple expressions
2. Build complexity gradually
3. Use :vars to check state
4. Save useful snippets with :copy
```

### Workflow 2: Development

```
1. Switch to Workspace (F2)
2. Write code in Editor
3. Press Ctrl+R to run
4. Switch to REPL (Tab) to test
5. Iterate: edit → run → test
```

### Workflow 3: Debugging

```
1. Output expressions directly (no Print needed)
2. Check :vars frequently
3. Test expressions in isolation
4. Build up complexity step by step
```

## Next Steps

You've mastered the REPL! Continue learning:

- [Language Syntax](../user-guide/language/syntax.md) - Complete syntax guide
- [Data Types](../user-guide/language/data-types.md) - Type system details
- [Built-in Functions](../user-guide/builtins/README.md) - All 50+ functions
- [Workflows](../user-guide/workflows/sequential.md) - Advanced patterns
