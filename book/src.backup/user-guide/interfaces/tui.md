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
// Simple expressions
5 + 3
// 8

// Variable binding
let x = 10
// ✓ Bound 'x' : Int

// Function calls
Upper("hello")
// "HELLO"

// Complex workflows
[1, 2, 3, 4, 5]
  |> Filter(_, def (x) := x % 2 == 0)
  |> Map(_, def (x) := x * x)
// [4, 16]
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
// Define a type
type User {
  name: String
  age: Int
}

// Create instance
let user = {name: "Alice", age: 30}
// ✓ Bound 'user' : Map

// Access fields
user.name
// "Alice"

// Use in function
def greet(u) := "Hello, " + u.name
greet(user)
// "Hello, Alice"
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
// In editor: write a complete program

type Article {
  title: String
  content: String
}

def analyzeArticle(article) :=
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

// Switch to REPL to test
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
// Type "Fil" then Tab
Filter    // Builtin function
FilterData // User function (if defined)

// Type ":" then Tab
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
// Opens multiline automatically
[1, 2, 3]
  |> Map(_, def (x) := {
       value: x,
       squared: x * x
     })
  |> Filter(_, def (item) := item.squared > 5)
// Press Ctrl+Enter to execute
```

**Manual Multiline:**

Use **Shift+Enter** to explicitly add new lines:

```dsl
let result =
  FetchData("api/users")
    |> Filter(_, isActive)
    |> Map(_, transform)
// Shift+Enter adds lines
// Ctrl+Enter executes
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
// Generate or load image
let imagePath = "diagram.png"

// Display in TUI
LoadImage(imagePath)
// Image renders in output pane
```

**Toggle Images:**
- Press **Ctrl+I** to show/hide images (performance)

### Variable Persistence

Variables, types, and functions persist across evaluations:

```dsl
// Session 1
let x = 10
def double(n) := n * 2

// Session 2 (later)
double(x)
// 20 (x and double still available)
```

**Scoping:**
- REPL uses global scope
- Variables shadow previous definitions
- Use `:vars` to check what's defined

### Error Reporting

Clear error messages with context:

```dsl
// Type error
5 + "hello"
// Error: Type mismatch: Cannot add Int and String

// Undefined variable
unknownVar
// Error: Undefined variable: unknownVar

// Parse error
[1, 2, 3
// Error: Unclosed bracket
```

## Tips and Tricks

### Rapid Prototyping

Use REPL for quick experiments:

```dsl
// Test data transformations
[1, 2, 3, 4, 5]
  |> Filter(_, def (x) := x > 2)
  |> Map(_, def (x) := x * 2)

// Verify output, refine, then move to Editor
```

### Debugging Workflows

Add intermediate bindings to inspect values:

```dsl
data
  |> Process1(_) as step1
  |> _ as _ |> step1  // Inspect step1
  |> Process2(step1) as step2
  |> _ as _ |> step2  // Inspect step2
```

### Using the Editor

1. Write complete programs in Editor
2. Save to file (`:save script.dsl`)
3. Load in REPL to test functions (`:load script.dsl`)
4. Iterate between Editor and REPL

### Exploratory Data Analysis

```dsl
// Load data
SQL("SELECT * FROM 'data.csv'") as data

// Explore interactively
Length(data)
First(data)
data[0]

// Find patterns
Filter(data, def (row) := row.value > 100)
  |> Map(_, def (row) := row.category)
  |> Unique(_)
```

### Quick Documentation

Use builtin functions to explore:

```dsl
// List all builtins (if defined)
:funcs

// Test function behavior
Map([1, 2, 3], def (x) := x * 2)
Filter([1, 2, 3, 4], def (x) := x % 2 == 0)
```

### Saving Work

Regularly save sessions:

```dsl
// After defining types and functions
:save progress.dsl

// Later, resume work
:load progress.dsl
```

### Performance Tips

1. **Disable images** for large outputs: **Ctrl+I**
2. **Clear output** frequently: `:clear` or **Ctrl+L**
3. **Use variables** instead of recomputing:
   ```dsl
   let expensive = ComputeExpensive(data)
   // Reuse 'expensive' instead of recomputing
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
// Define types
type User { name: String, age: Int }

// Create test data
let testUser = {name: "Alice", age: 30}

// Define function
def greet(user) := "Hello, " + user.name

// Test immediately
greet(testUser)

// Refine and iterate
```

### 2. Data Pipeline Development

```dsl
// Load data
SQL("SELECT * FROM 'sales.csv'") as sales

// Build pipeline incrementally
sales |> Filter(_, def (s) := s.amount > 100) as filtered
filtered |> Map(_, def (s) := s.category) as categories
categories |> Unique(_) as uniqueCategories

// Combine when satisfied
let result = sales
  |> Filter(_, def (s) := s.amount > 100)
  |> Map(_, def (s) := s.category)
  |> Unique(_)
```

### 3. LLM Workflow Testing

```dsl
// Test single query
Ask("What is the capital of France?")

// Build multi-step workflow
let topic = "AI Ethics"
let research = Ask("Research " + topic)
let outline = Ask("Create outline for: " + research)
let draft = Ask("Write article: " + outline)

// Package as function
def generateArticle(topic) :=
  Ask("Research " + topic)
    |> Ask("Create outline for: " + _)
    |> Ask("Write article: " + _)
```

### 4. Type-Driven Development

```dsl
// Define types first
type Request {
  method: String
  url: String
  body: String
}

type Response {
  status: Int
  body: String
}

// Write function signature
def makeRequest(req) := ...

// Test with mock data
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
