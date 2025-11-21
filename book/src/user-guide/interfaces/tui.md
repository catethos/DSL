# Terminal UI (TUI)

The DSL Terminal User Interface provides a powerful, interactive environment for developing and testing DSL code directly in your terminal. Built with Ratatui, it features a split-pane workspace, syntax highlighting, autocomplete, and image rendering support.

## Overview

The TUI offers three main components:

1. **REPL** - Interactive Read-Eval-Print Loop for quick experimentation
2. **Editor** - Full-featured text editor for writing DSL scripts
3. **Preview** - Visual execution tracker showing workflow steps

## Getting Started

### Launching the TUI

```bash
# Start the interactive TUI
dsl

# Or explicitly run TUI mode
dsl tui
```

### First Steps

When you launch the TUI:

1. You'll see a banner with version info and quick tips
2. The REPL pane is active by default
3. Type DSL expressions and press Enter to evaluate
4. Use `:help` to see available commands

### Basic Usage

```dsl
5 + 3

let x = 10

Upper("hello")

[1, 2, 3, 4, 5]
  |> filter(_, fn x => x % 2 == 0 end)
  |> map(_, fn x => x * x end)
```

## Workspace Panes

### REPL Pane

The **Read-Eval-Print Loop** is the primary interface for interactive development.

**Features:**
- Immediate evaluation of expressions
- Command history (Up/Down arrows)
- Autocomplete (Tab)
- Multiline input support
- Syntax error reporting
- Variable persistence between evaluations

**Example Session:**

```dsl
type User {
  name: String
  age: Int
}

let user = {name: "Alice", age: 30}

user.name

def greet(u) { "Hello, " + u.name }
greet(user)
```

### Editor Pane

The **Editor** provides a full-featured text editor for writing longer scripts.

**Features:**
- Syntax highlighting with Tree-sitter
- File loading and saving
- Multi-line editing
- Standard editor keybindings

**Workflow:**

1. Switch to Editor pane (Ctrl+E)
2. Write your DSL script
3. Save to file (`:save filename.dsl`)
4. Execute in REPL or run as script

**Example:**

```dsl

type Article {
  title: String
  content: String
}

def analyzeArticle(article) {
    Par(
    Ask("Summarize: " + article.content),
    Ask("Extract keywords: " + article.content),
    Ask("Sentiment: " + article.content)
  ) as [summary, keywords, sentiment]

  {
    title: article.title,
    summary: summary,
    keywords: Split(keywords, ","),
    sentiment: sentiment
  }

analyzeArticle({
  title: "AI Ethics",
  content: "Long article text..."
})
```

### Preview Pane

The **Preview** shows execution steps for workflows, useful for debugging.

**Features:**
- Step-by-step execution tracking
- Duration timing for each step
- Status indicators (Pending, Running, Complete, Error)
- Output display for each step

**Use Case:**

When running complex workflows, switch to Preview (Ctrl+P) to see:

```
Step 1: FetchData("api/users")      [Complete] 523ms
Step 2: Filter(data, isActive)      [Complete] 12ms
Step 3: Map(data, transform)        [Complete] 45ms
Step 4: Par(process1, process2)     [Running]  ...
```

## REPL Commands

Commands start with `:` and provide meta-operations.

### Information Commands

#### `:help`

Show help message with keybindings and available commands.

```
:help
```

#### `:vars`

List all currently bound variables with their types.

```
:vars

Variables:
  x: Int = 10
  name: String = "Alice"
  users: List = [...]
```

#### `:types`

Show all defined custom types.

```
:types

Types:
  User { name: String, age: Int }
  Article { title: String, content: String }
  Status (enum): Pending | InProgress | Completed
```

#### `:funcs`

List all defined functions with signatures.

```
:funcs

Functions:
  greet(user) -> String
  analyzeArticle(article) -> Map
  processData(data) -> List
```

#### `:globals`

Show global scope variables only.

```
:globals
```

#### `:scopes` (Debug)

Show the entire scope stack for debugging.

```
:scopes
```

### Session Management

#### `:save [filename]`

Save current session (variables, types, functions) to a file.

```
:save my_session.dsl
```

The session file can be loaded later or executed as a script.

#### `:load [filename]`

Load a session file, restoring all definitions.

```
:load my_session.dsl
```

### Utility Commands

#### `:clear`

Clear the output pane (doesn't reset variables).

```
:clear
```

#### `:copy`

Copy the last evaluation result to clipboard.

```
:copy
```

#### `:debug`

Toggle debug mode for verbose output.

```
:debug
```

#### `:quit` or `:q`

Exit the TUI.

```
:quit
```

## Keyboard Shortcuts

### Navigation

| Key | Action |
|-----|--------|
| **Ctrl+E** | Switch to Editor pane |
| **Ctrl+R** | Switch to REPL pane |
| **Ctrl+P** | Switch to Preview pane |
| **Tab** | Cycle between panes |

### REPL Input

| Key | Action |
|-----|--------|
| **Enter** | Execute input (single-line mode) |
| **Shift+Enter** | New line (multiline mode) |
| **Ctrl+Enter** | Execute (multiline mode) |
| **Tab** | Show autocomplete |
| **Esc** | Close autocomplete |
| **Up/Down** | Navigate history |
| **Ctrl+C** | Clear input |
| **Ctrl+U** | Clear line |

### Editor

| Key | Action |
|-----|--------|
| **Ctrl+S** | Save file |
| **Ctrl+O** | Open file |
| **Ctrl+K** | Cut line |
| **Ctrl+Y** | Paste |
| **Ctrl+Z** | Undo |
| **Ctrl+Y** | Redo |
| **Home** | Start of line |
| **End** | End of line |

### Output

| Key | Action |
|-----|--------|
| **PgUp/PgDn** | Scroll output |
| **Ctrl+Up/Down** | Scroll output |
| **Mouse Wheel** | Scroll output |
| **Mouse Drag** | Select text |
| **Ctrl+C** | Copy selection |

### General

| Key | Action |
|-----|--------|
| **Ctrl+L** | Clear screen |
| **Ctrl+D** | Quit TUI |
| **Ctrl+I** | Toggle images |

## Features

### Autocomplete

The TUI provides intelligent autocomplete for:
- REPL commands (`:help`, `:vars`, etc.)
- Variable names
- Function names
- Type names
- Builtin functions

**Usage:**

1. Type prefix (e.g., `Fil`)
2. Press **Tab**
3. Select from suggestions with **Up/Down**
4. Press **Enter** to accept or **Esc** to cancel

**Example:**

```dsl
Filter
FilterData

:help
:vars
:types
:clear
...
```

### Multiline Mode

For complex expressions, the REPL automatically enters multiline mode when it detects unclosed delimiters.

**Automatic Multiline:**

```dsl
[1, 2, 3]
  |> map(_, fn x => {
       value: x,
       squared: x * x
     } end)
  |> filter(_, fn item => item.squared > 5 end)
```

**Manual Multiline:**

Use **Shift+Enter** to explicitly add new lines:

```dsl
let result =
  FetchData("api/users")
    |> Filter(_, isActive)
    |> Map(_, transform)
```

### Command History

The REPL maintains a history of executed commands.

**Navigation:**
- **Up Arrow**: Previous command
- **Down Arrow**: Next command
- History persists between sessions

**Tips:**
- Edit historical commands before re-executing
- Search history by typing prefix, then Up
- History includes both expressions and commands

### Syntax Highlighting

Tree-sitter provides real-time syntax highlighting for:
- Keywords (`def`, `type`, `let`)
- Operators (`|>`, `+`, `==`)
- Literals (strings, numbers, booleans)
- Comments
- Function calls
- Type annotations

### Image Rendering

The TUI can render images directly in the output pane.

**Supported:**
- PNG, JPEG, GIF
- Terminal graphics protocols (iTerm2, Kitty, Sixel)
- Fallback to half-block rendering

**Example:**

```dsl
let imagePath = "diagram.png"

LoadImage(imagePath)
```

**Toggle Images:**
- Press **Ctrl+I** to show/hide images (performance)

### Variable Persistence

Variables, types, and functions persist across evaluations:

```dsl
let x = 10
def double(n) { n * 2 }

double(x)
```

**Scoping:**
- REPL uses global scope
- Variables shadow previous definitions
- Use `:vars` to check what's defined

### Error Reporting

Clear error messages with context:

```dsl
5 + "hello"

unknownVar

[1, 2, 3
```

## Tips and Tricks

### Rapid Prototyping

Use REPL for quick experiments:

```dsl
[1, 2, 3, 4, 5]
  |> filter(_, fn x => x > 2 end)
  |> map(_, fn x => x * 2 end)

```

### Debugging Workflows

Add intermediate bindings to inspect values:

```dsl
data
  |> Process1(_) as step1
  |> _ as _ |> step1
  |> Process2(step1) as step2
  |> _ as _ |> step2
```

### Using the Editor

1. Write complete programs in Editor
2. Save to file (`:save script.dsl`)
3. Load in REPL to test functions (`:load script.dsl`)
4. Iterate between Editor and REPL

### Exploratory Data Analysis

```dsl
SQL("SELECT * FROM 'data.csv'") as data

Length(data)
First(data)
data[0]

filter(data, fn row => row.value > 100 end)
  |> map(_, fn row => row.category end)
  |> Unique(_)
```

### Quick Documentation

Use builtin functions to explore:

```dsl
:funcs

map([1, 2, 3], fn x => x * 2 end)
filter([1, 2, 3, 4], fn x => x % 2 == 0 end)
```

### Saving Work

Regularly save sessions:

```dsl
:save progress.dsl

:load progress.dsl
```

### Performance Tips

1. **Disable images** for large outputs: **Ctrl+I**
2. **Clear output** frequently: `:clear` or **Ctrl+L**
3. **Use variables** instead of recomputing:
   ```dsl
   let expensive = ComputeExpensive(data)
   ```

### Keyboard Efficiency

Master these shortcuts:
- **Ctrl+C**: Clear input without executing
- **Ctrl+L**: Clear screen
- **Up/Down**: Navigate history
- **Tab**: Autocomplete
- **Ctrl+Enter**: Execute multiline

## Common Workflows

### 1. Interactive Development

```dsl
type User { name: String, age: Int }

let testUser = {name: "Alice", age: 30}

def greet(user) { "Hello, " + user.name }

greet(testUser)

```

### 2. Data Pipeline Development

```dsl
SQL("SELECT * FROM 'sales.csv'") as sales

sales |> filter(_, fn s => s.amount > 100 end) as filtered
filtered |> map(_, fn s => s.category end) as categories
categories |> Unique(_) as uniqueCategories

let result = sales
  |> filter(_, fn s => s.amount > 100 end)
  |> map(_, fn s => s.category end)
  |> Unique(_)
```

### 3. LLM Workflow Testing

```dsl
Ask("What is the capital of France?")

let topic = "AI Ethics"
let research = Ask("Research " + topic)
let outline = Ask("Create outline for: " + research)
let draft = Ask("Write article: " + outline)

def generateArticle(topic) { Ask("Research " + topic) }
    |> Ask("Create outline for: " + _)
    |> Ask("Write article: " + _)
```

### 4. Type-Driven Development

```dsl
type Request {
  method: String
  url: String
  body: String
}

type Response {
  status: Int
  body: String
}

def makeRequest(req) { ... }

let testReq = {
  method: "GET",
  url: "https://api.example.com",
  body: ""
}

makeRequest(testReq)
```

## Troubleshooting

### Input Not Working

**Problem:** Typing doesn't show characters

**Solution:**
- Ensure REPL pane is active (Ctrl+R)
- Check terminal supports raw mode
- Try restarting TUI

### Autocomplete Not Showing

**Problem:** Tab doesn't show suggestions

**Solution:**
- Type at least one character
- Ensure cursor is at end of word
- Check if suggestions exist for prefix

### Images Not Rendering

**Problem:** Images show as placeholders

**Solution:**
- Check terminal supports graphics (iTerm2, Kitty, Sixel)
- Verify image file exists and is valid
- Try toggling images: Ctrl+I

### History Not Persisting

**Problem:** Command history lost between sessions

**Solution:**
- History saves to `~/.dsl_history`
- Check file permissions
- Ensure TUI exits cleanly (`:quit`, not kill)

### Slow Performance

**Problem:** TUI becomes sluggish

**Solution:**
- Clear output: `:clear`
- Disable images: Ctrl+I
- Reduce output size
- Check for infinite loops

## Next Steps

- [Desktop GUI (egui)](./egui.md) - Visual interface with graphical editor
- [CLI Reference](./cli.md) - Command-line usage and batch processing
- [Workflows](../workflows/sequential.md) - Building complex workflows
- [Builtin Functions](../builtins/README.md) - Available functions
