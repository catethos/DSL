# Getting Started with the DSL TUI

## Prerequisites

### Required
- **Rust** (latest stable version)
  ```bash
  # Install Rust via rustup
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

### Optional
- **OpenAI API Key** - For LLM features
  ```bash
  export OPENAI_API_KEY="sk-..."
  ```

## Installation

### 1. Clone or Navigate to the Repository
```bash
cd /Users/catethos/workspace/DSL
```

### 2. Build the Project
```bash
# Debug build (faster compilation)
cargo build

# Release build (optimized performance)
cargo build --release
```

### 3. Run the DSL REPL
```bash
# Debug build
cargo run --bin dsl

# Release build (faster execution)
cargo run --release --bin dsl
```

### 4. Verify Installation
You should see the welcome banner:
```
┌──────────────────────────────────────────────────────────────┐
│                                                              │
│     ███████╗██╗      ██████╗ ██╗    ██╗                     │
│     ██╔════╝██║     ██╔═══██╗██║    ██║                     │
│     █████╗  ██║     ██║   ██║██║ █╗ ██║                     │
│     ██╔══╝  ██║     ██║   ██║██║███╗██║                     │
│     ██║     ███████╗╚██████╔╝╚███╔███╔╝                     │
│     ╚═╝     ╚══════╝ ╚═════╝  ╚══╝╚══╝                      │
│                                                              │
│     Agentic LLM Workflow DSL v1.0                           │
│     Interactive Terminal User Interface                     │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

## First Steps

### Tutorial 1: Basic REPL Usage (2 minutes)

The DSL starts in **Workspace Mode** with three panes. Press **F1** to switch to full REPL mode.

#### Try These Commands:

1. **Simple expressions**
   ```javascript
   flow> 42
   ✓ 42 : Int

   flow> "hello world"
   ✓ "hello world" : String

   flow> 3.14
   ✓ 3.14 : Float

   flow> true
   ✓ true : Bool
   ```

2. **Arithmetic**
   ```javascript
   flow> 10 + 5
   ✓ 15 : Int

   flow> 20 * 3
   ✓ 60 : Int

   flow> 100 / 4
   ✓ 25 : Int

   flow> 3.14 * 2
   ✓ 6.28 : Float
   ```

3. **Lists**
   ```javascript
   flow> [1, 2, 3, 4, 5]
   ✓ [1, 2, 3, 4, 5] : List

   flow> ["apple", "banana", "cherry"]
   ✓ ["apple", "banana", "cherry"] : List
   ```

### Tutorial 2: Variables and Binding (3 minutes)

4. **Variable binding with `as`**
   ```javascript
   flow> 42 as answer
   ✓ Bound 'answer' to 42 : Int

   flow> answer
   ✓ 42 : Int

   flow> answer * 2
   ✓ 84 : Int
   ```

5. **The underscore `_` variable**
   ```javascript
   flow> 100
   ✓ 100 : Int

   flow> _
   ✓ 100 : Int

   flow> _ + 50
   ✓ 150 : Int
   ```

6. **List indexing**
   ```javascript
   flow> [10, 20, 30, 40, 50] as numbers
   ✓ Bound 'numbers' to [10, 20, 30, 40, 50] : List

   flow> numbers[0]
   ✓ 10 : Int

   flow> numbers[2]
   ✓ 30 : Int
   ```

7. **Check all variables**
   ```javascript
   flow> :vars
   ✓ Variables:
     answer = 42 : Int
     numbers = [10, 20, 30, 40, 50] : List
     _ = 30 : Int
   ```

### Tutorial 3: Built-in Functions (3 minutes)

8. **String operations**
   ```javascript
   flow> Upper("hello world")
   ✓ "HELLO WORLD" : String

   flow> Lower("HELLO WORLD")
   ✓ "hello world" : String

   flow> Length("hello")
   ✓ 5 : Int

   flow> Join(["a", "b", "c"], "-")
   ✓ "a-b-c" : String
   ```

### Tutorial 4: Sequential Composition (3 minutes)

9. **Chain operations with `>>`**
   ```javascript
   flow> "hello" >> Upper(_)
   ✓ "HELLO" : String

   flow> "hello" >> Upper(_) >> Length(_)
   ✓ 5 : Int

   flow> [1, 2, 3, 4, 5] >> Length(_)
   ✓ 5 : Int
   ```

10. **Sequential with binding**
    ```javascript
    flow> ["a", "b", "c"] >> Join(_, "-") as joined >> Upper(joined)
    ✓ "A-B-C" : String

    flow> 5 >> _ * 2 >> _ + 3
    ✓ 13 : Int
    ```

### Tutorial 5: Parallel Execution (2 minutes)

11. **Parallel with `||`**
    ```javascript
    flow> (5 || 10 || 15)
    ✓ [5, 10, 15] : List

    flow> (5 || 10 || 15) as numbers
    ✓ Bound 'numbers' to [5, 10, 15] : List
    ```

12. **Parallel destructuring**
    ```javascript
    flow> (10 || 20 || 30) as [a, b, c]
    ✓ Bound 'a' to 10 : Int
    ✓ Bound 'b' to 20 : Int
    ✓ Bound 'c' to 30 : Int

    flow> a + b + c
    ✓ 60 : Int
    ```

### Tutorial 6: LLM Integration (requires API key)

13. **Set up OpenAI API key** (if you haven't already)
    ```bash
    # In your terminal (outside the REPL)
    export OPENAI_API_KEY="sk-..."
    ```

14. **Simple LLM call**
    ```javascript
    flow> Ask("What is the capital of France?")
    ✓ "The capital of France is Paris." : String
    ```

15. **Chain with LLM**
    ```javascript
    flow> "Explain Rust in one sentence" >> Ask(_)
    ✓ "Rust is a systems programming language..." : String
    ```

## UI Modes

The DSL has three main modes you can switch between:

### 1. REPL Mode (F1)
Full-screen interactive REPL for quick experimentation.

**Best for:**
- Testing expressions
- Exploring features
- Quick calculations

**Key bindings:**
- **Enter** - Execute command
- **Shift+Enter** - Multi-line input
- **Up/Down** - History navigation
- **Esc** - Quit

### 2. Workspace Mode (F2)
Three-pane layout: Editor + REPL + Preview

**Best for:**
- Writing multi-line workflows
- Developing functions and types
- Seeing execution preview

**Key bindings:**
- **Tab** - Switch between panes
- **Ctrl+R** - Run editor content in REPL
- **Ctrl+S** - Save file
- **Esc** - Quit

### 3. Type Explorer (F3)
Browse all registered types and their fields.

**Best for:**
- Viewing defined types
- Understanding type structure
- Quick reference

## Special Commands

The REPL supports several special commands:

| Command | Description | Example |
|---------|-------------|---------|
| `:vars` | List all variables | `:vars` |
| `:types` | List all type definitions | `:types` |
| `:funcs` | List all user-defined functions | `:funcs` |
| `:help` | Show help information | `:help` |
| `:copy` | Save last result to file | `:copy output.txt` |
| `:debug` | Toggle debug mode | `:debug` |
| `:save` | Save session to file | `:save session.json` |
| `:load` | Load session from file | `:load session.json` |

## Common Workflows

### Workflow 1: REPL Exploration
```
1. Start REPL (F1)
2. Type expressions and see results
3. Use :vars to check variables
4. Use :help for quick reference
```

### Workflow 2: Function Development
```
1. Switch to Workspace (F2)
2. Write function in Editor pane
3. Press Ctrl+R to load into REPL
4. Switch to REPL pane (Tab) to test
5. Iterate: edit → Ctrl+R → test
```

### Workflow 3: Type Definition
```
1. Switch to Workspace (F2)
2. Define types in Editor
3. Press Ctrl+R to register types
4. Press F3 to view in Type Explorer
5. Use types in REPL or functions
```

## Quick Reference Card

### Operators
- `+`, `-`, `*`, `/` - Arithmetic
- `>>` - Sequential composition
- `||` - Parallel execution
- `as` - Variable binding

### Data Types
- `Int` - Integer numbers
- `Float` - Floating point numbers
- `String` - Text
- `Bool` - true/false
- `List` - Collections
- `Map` - Key-value pairs

### Key Bindings
- **F1** - REPL Mode
- **F2** - Workspace Mode
- **F3** - Type Explorer
- **Tab** - Switch panes (Workspace)
- **Ctrl+R** - Run editor content
- **Ctrl+S** - Save file
- **Esc** - Quit

### Built-in Functions
- `Upper(str)` - Convert to uppercase
- `Lower(str)` - Convert to lowercase
- `Length(str/list)` - Get length
- `Join(list, sep)` - Join strings
- `Ask(prompt)` - LLM query

## Troubleshooting

### Issue: "OpenAI API key not found"
**Solution:** Set your API key in the environment:
```bash
export OPENAI_API_KEY="sk-..."
```

### Issue: Can't see the cursor in REPL
**Solution:** This is normal. The cursor is shown at the prompt line.

### Issue: Multi-line input not working
**Solution:** Use **Shift+Enter** or **Alt+Enter** to add new lines.

### Issue: "Type not found" error
**Solution:** Define the type first using `type TypeName { ... }` syntax.

### Issue: Function not defined
**Solution:** Define the function first using `def functionName() { ... }` syntax.

## Next Steps

Now that you've completed the tutorials, explore more features:

1. **[03-TUI-Interface.md](03-TUI-Interface.md)** - Learn about all UI modes in detail
2. **[04-Language-Features.md](04-Language-Features.md)** - Master the language syntax
3. **[05-Type-System.md](05-Type-System.md)** - Create custom types
4. **[06-Functions.md](06-Functions.md)** - Define your own functions
5. **[examples/](../examples/)** - Try the example workflows

## Resources

- **Examples**: See `examples/` directory for complete workflows
- **Design**: Read `DESIGN.md` for language specification
- **Progress**: Check `PROGRESS.md` for implementation status
- **Main README**: See `README.md` for project overview

## Tips for Success

1. **Start Simple** - Begin with basic expressions before complex workflows
2. **Use :vars** - Check variables frequently to understand state
3. **Experiment** - The REPL is safe, try things out
4. **Save Sessions** - Use `:save` to preserve your work
5. **Read Examples** - The `examples/` directory has working code
6. **Use the Editor** - For multi-line code, use Workspace mode (F2)
7. **Check Types** - Use `:types` and F3 to view type definitions

Happy coding!
