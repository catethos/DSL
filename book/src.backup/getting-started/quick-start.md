# Quick Start

Get up and running with DSL in 5 minutes.

## Launch the REPL

After [installation](installation.md), start the DSL:

```bash
cargo run --release --bin dsl
```

## UI Modes

The DSL has three main modes:

| Key | Mode | Description |
|-----|------|-------------|
| **F1** | REPL Mode | Full-screen interactive REPL |
| **F2** | Workspace Mode | Editor + REPL + Preview (default) |
| **F3** | Type Explorer | Browse type definitions |

```admonish tip
Press **F1** to switch to full REPL mode for quick experimentation.
```

## Your First Commands

### 1. Simple Expressions (30 seconds)

```dsl
flow> 42
✓ 42 : Int

flow> "hello world"
✓ "hello world" : String

flow> [1, 2, 3, 4, 5]
✓ [1, 2, 3, 4, 5] : List
```

### 2. Arithmetic (30 seconds)

```dsl
flow> 10 + 5
✓ 15 : Int

flow> 20 * 3
✓ 60 : Int

flow> 100 / 4
✓ 25 : Int
```

### 3. Variables (1 minute)

```dsl
flow> 42 as answer
✓ Bound 'answer' to 42 : Int

flow> answer * 2
✓ 84 : Int

flow> [10, 20, 30] as numbers
✓ Bound 'numbers' to [10, 20, 30] : List

flow> numbers[1]
✓ 20 : Int
```

### 4. Built-in Functions (1 minute)

```dsl
flow> Upper("hello world")
✓ "HELLO WORLD" : String

flow> Length("hello")
✓ 5 : Int

flow> Join(["a", "b", "c"], "-")
✓ "a-b-c" : String
```

### 5. Pipeline Operator (1 minute)

The `|>` operator chains operations left-to-right:

```dsl
flow> "hello" |> Upper(_)
✓ "HELLO" : String

flow> "hello" |> Upper(_) |> Length(_)
✓ 5 : Int

flow> [1, 2, 3, 4, 5] |> Length(_)
✓ 5 : Int
```

### 6. Special Commands (1 minute)

```dsl
flow> :vars
✓ Variables:
  answer = 42 : Int
  numbers = [10, 20, 30] : List

flow> :help
✓ Available commands:
  :vars  - List all variables
  :types - List type definitions
  :help  - Show this help
```

## Key Bindings Quick Reference

### Navigation
- **Enter** - Execute command
- **Shift+Enter** - Multi-line input
- **Up/Down** - History navigation
- **Tab** - Switch panes (Workspace mode)
- **Esc** - Quit

### Modes
- **F1** - REPL Mode (full screen)
- **F2** - Workspace Mode (editor + REPL + preview)
- **F3** - Type Explorer

### Workspace Mode
- **Ctrl+R** - Run editor content in REPL
- **Ctrl+S** - Save file

## What's Next?

You now know the basics! Explore more:

- [First Program](first-program.md) - Build a complete workflow
- [REPL Tour](repl-tour.md) - Comprehensive REPL walkthrough
- [Language Basics](../user-guide/language/syntax.md) - Master the syntax
- [Built-in Functions](../user-guide/builtins/README.md) - Explore 50+ functions

## Quick Tips

```admonish tip "Use the underscore variable"
The `_` variable always holds the last result:

flow> 100
✓ 100 : Int

flow> _ + 50
✓ 150 : Int
```

```admonish tip "Check variables frequently"
Use `:vars` to see what's in scope:

flow> :vars
✓ Variables:
  answer = 42 : Int
  _ = 150 : Int
```

```admonish tip "Save your work"
Use `:save` to preserve your session:

flow> :save my-session.json
✓ Session saved to my-session.json
```
