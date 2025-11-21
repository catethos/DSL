# Desktop GUI (egui)

The DSL Desktop GUI provides a modern, visually rich interface for developing DSL applications. Built with egui, it offers a polished workspace with syntax highlighting, autocomplete, rich output rendering, and a customizable split-pane layout.

## Overview

The egui interface combines the power of an IDE with the interactivity of a REPL:

- **Visual Editor** with syntax highlighting and Tree-sitter integration
- **Interactive REPL** for immediate feedback
- **Rich Output Rendering** (tables, trees, images, markdown)
- **Resizable Split Panes** for flexible workspace layout
- **Modern Theme** with smooth animations and intuitive controls

## Getting Started

### Launching the GUI

```bash
# Start the desktop GUI
dsl gui

# Or run dsl-egui directly
dsl-egui
```

### First Impressions

When you launch the GUI:

1. Split workspace with Editor (left) and REPL (right)
2. Modern dark theme with accent colors
3. Resizable panes (drag the divider)
4. Menu bar with File, Edit, View, Help options
5. Status bar showing keyboard hints

## Workspace Layout

### Split-Pane Interface

The GUI uses a horizontal split layout:

```
┌─────────────────────────────┬─────────────────────────────┐
│                             │                             │
│         Editor Pane         │         REPL Pane           │
│     (Write Scripts)         │   (Interactive Testing)     │
│                             │                             │
│  • Syntax highlighting      │  • Immediate evaluation     │
│  • Multi-file tabs          │  • Command history          │
│  • File operations          │  • Rich output              │
│  • Line numbers             │  • Autocomplete             │
│                             │                             │
└─────────────────────────────┴─────────────────────────────┘
```

**Adjusting Layout:**
- Drag the center divider to resize panes
- Double-click divider to reset to 50/50
- Hide Editor: drag all the way left
- Hide REPL: drag all the way right

### Active Pane Focus

The active pane is highlighted with a colored border:

- **Blue border**: Active pane (receives keyboard input)
- Click to switch focus
- Or use **Ctrl+Tab** to toggle between panes

## Editor Pane

The Editor provides a full-featured code editor for writing DSL scripts.

### Features

**Syntax Highlighting:**
- Tree-sitter powered
- Real-time as you type
- Semantic highlighting (types, functions, variables)
- Error underlining

**Code Navigation:**
- Line numbers
- Goto line (Ctrl+G)
- Find (Ctrl+F)
- Replace (Ctrl+H)

**Editing:**
- Multi-line selection
- Block indent/outdent (Tab/Shift+Tab)
- Comment toggle (Ctrl+/)
- Auto-indent
- Bracket matching

**File Operations:**
- New file (Ctrl+N)
- Open file (Ctrl+O)
- Save (Ctrl+S)
- Save As (Ctrl+Shift+S)
- Recent files list

### Example Workflow

```dsl
// Write a complete program in the editor

type User {
  id: Int
  name: String
  email: String
  active: Bool
}

def getActiveUsers(users) :=
  users
    |> Filter(_, def (u) := u.active)
    |> Map(_, def (u) := {
         id: u.id,
         name: u.name,
         email: u.email
       })
    |> Sort(_)

// Save to file: Ctrl+S
// Test in REPL: copy/paste or load file
```

### Multi-Tab Support

Work on multiple files simultaneously:

- **New Tab**: Ctrl+T
- **Close Tab**: Ctrl+W
- **Switch Tabs**: Ctrl+1, Ctrl+2, etc.
- Tab bar shows all open files
- Unsaved files marked with *

### Syntax Highlighting Colors

The modern theme uses semantic colors:

| Element | Color | Example |
|---------|-------|---------|
| **Keywords** | Blue | `def`, `type`, `let` |
| **Strings** | Green | `"hello world"` |
| **Numbers** | Orange | `42`, `3.14` |
| **Comments** | Gray | `// comment` |
| **Functions** | Yellow | `Map`, `Filter` |
| **Types** | Cyan | `Int`, `String`, `User` |
| **Operators** | White | `|>`, `+`, `==` |
| **Variables** | White | `x`, `users` |

## REPL Pane

The REPL (Read-Eval-Print Loop) provides immediate, interactive evaluation.

### Features

**Input Area:**
- Multi-line input support
- Syntax highlighting as you type
- Auto-completion (Tab)
- History navigation (Up/Down)
- Smart indentation

**Output Area:**
- Rich rendering (tables, trees, markdown)
- Syntax-highlighted values
- Image display
- Scrollable history
- Selectable text (for copying)

**Evaluation:**
- Press Enter to evaluate (single-line)
- Shift+Enter for multi-line
- Ctrl+Enter to execute multi-line input
- Clear button to reset output

### Example Session

```dsl
// Simple arithmetic
5 + 3
→ 8

// Variable binding
let users = [
  {id: 1, name: "Alice", active: true},
  {id: 2, name: "Bob", active: false},
  {id: 3, name: "Charlie", active: true}
]
→ ✓ Bound 'users' : List

// Use defined function from Editor
getActiveUsers(users)
→ [
    {id: 1, name: "Alice", email: "alice@example.com"},
    {id: 3, name: "Charlie", email: "charlie@example.com"}
  ]
```

### Rich Output Rendering

The REPL renders different data types with appropriate formatting:

**Lists:**
```dsl
[1, 2, 3, 4, 5]
```
Renders as:
```
[1, 2, 3, 4, 5]
```

**Maps/Objects:**
```dsl
{name: "Alice", age: 30, active: true}
```
Renders as formatted JSON:
```json
{
  "name": "Alice",
  "age": 30,
  "active": true
}
```

**Tables:**
Large lists of objects render as tables:
```dsl
users
```
Renders as:
```
┌────┬──────────┬──────────┐
│ id │ name     │ active   │
├────┼──────────┼──────────┤
│ 1  │ Alice    │ true     │
│ 2  │ Bob      │ false    │
│ 3  │ Charlie  │ true     │
└────┴──────────┴──────────┘
```

**Markdown:**
```dsl
RenderMarkdown("# Hello\n\nThis is **bold**")
```
Renders with formatting (headers, bold, lists, etc.)

**Images:**
```dsl
LoadImage("diagram.png")
```
Displays the image inline in the output

### Autocomplete

Intelligent suggestions as you type:

**Trigger:**
- Automatic after typing `.` or first letter
- Manual with **Tab** key

**Suggestions Include:**
- Builtin functions (`Map`, `Filter`, `Sum`)
- Defined functions (`getActiveUsers`)
- Variables (`users`, `data`)
- Type names (`User`, `Status`)
- REPL commands (`:help`, `:vars`)

**Usage:**
1. Start typing: `Fil`
2. Suggestions appear in popup
3. Arrow keys to navigate
4. Enter to accept
5. Esc to dismiss

### History Navigation

Access previously executed commands:

- **Up Arrow**: Previous command
- **Down Arrow**: Next command
- **Ctrl+R**: Search history (type to filter)
- History persists between sessions

## Menu Bar

### File Menu

- **New** (Ctrl+N): Create new file
- **Open** (Ctrl+O): Open existing file
- **Save** (Ctrl+S): Save current file
- **Save As** (Ctrl+Shift+S): Save with new name
- **Recent Files**: Quick access to recently opened
- **Exit** (Ctrl+Q): Quit application

### Edit Menu

- **Undo** (Ctrl+Z): Undo last edit
- **Redo** (Ctrl+Y): Redo undone edit
- **Cut** (Ctrl+X): Cut selection
- **Copy** (Ctrl+C): Copy selection
- **Paste** (Ctrl+V): Paste from clipboard
- **Select All** (Ctrl+A): Select entire editor
- **Find** (Ctrl+F): Search in editor
- **Replace** (Ctrl+H): Find and replace

### View Menu

- **Toggle Theme**: Switch light/dark mode
- **Increase Font Size** (Ctrl++): Larger text
- **Decrease Font Size** (Ctrl+-): Smaller text
- **Reset Font Size** (Ctrl+0): Default size
- **Toggle Line Numbers**: Show/hide in editor
- **Toggle Minimap**: Code overview (if enabled)

### REPL Menu

- **Clear Output** (Ctrl+L): Clear REPL output
- **Clear Variables**: Reset interpreter state
- **Show Variables** (:vars): List all variables
- **Show Types** (:types): List defined types
- **Show Functions** (:funcs): List defined functions

### Help Menu

- **Documentation**: Open user guide
- **Keyboard Shortcuts**: Shortcut cheat sheet
- **About**: Version and credits

## Keyboard Shortcuts

### Global

| Shortcut | Action |
|----------|--------|
| **Ctrl+Tab** | Switch between Editor/REPL |
| **Ctrl+Q** | Quit application |
| **F11** | Toggle fullscreen |

### Editor

| Shortcut | Action |
|----------|--------|
| **Ctrl+N** | New file |
| **Ctrl+O** | Open file |
| **Ctrl+S** | Save file |
| **Ctrl+Shift+S** | Save As |
| **Ctrl+W** | Close tab |
| **Ctrl+Z** | Undo |
| **Ctrl+Y** | Redo |
| **Ctrl+F** | Find |
| **Ctrl+H** | Replace |
| **Ctrl+G** | Go to line |
| **Ctrl+/** | Toggle comment |
| **Tab** | Indent selection |
| **Shift+Tab** | Outdent selection |

### REPL

| Shortcut | Action |
|----------|--------|
| **Enter** | Execute (single-line) |
| **Shift+Enter** | New line |
| **Ctrl+Enter** | Execute (multi-line) |
| **Up/Down** | History navigation |
| **Ctrl+R** | Search history |
| **Tab** | Show autocomplete |
| **Esc** | Close autocomplete |
| **Ctrl+L** | Clear output |
| **Ctrl+C** | Copy selection (from output) |

### Text Editing

| Shortcut | Action |
|----------|--------|
| **Ctrl+A** | Select all |
| **Ctrl+X** | Cut |
| **Ctrl+C** | Copy |
| **Ctrl+V** | Paste |
| **Ctrl+Left/Right** | Jump by word |
| **Home** | Start of line |
| **End** | End of line |
| **Ctrl+Home** | Start of file |
| **Ctrl+End** | End of file |

## Features

### Persistent Workspace

Your workspace state is saved automatically:

- Open files and tabs
- Split pane ratio
- REPL history
- Recent files list
- Theme preference
- Font size

**Location:** `~/.dsl_workspace.json`

### Drag and Drop

Convenient file operations:

- Drag `.dsl` file into Editor to open
- Drag multiple files to open in tabs
- Drag text from external apps into Editor

### Status Bar

Bottom status bar shows:

- **Left**: File path, line:column position
- **Center**: Current mode (Insert/Command)
- **Right**: File encoding, type, modified status

### Error Highlighting

Real-time error detection:

- Red underlines for syntax errors
- Yellow for warnings
- Hover to see error message
- Error list panel (if enabled)

### Themes

Choose between light and dark themes:

**Dark Theme (default):**
- Modern dark background
- High contrast
- Blue accents
- Easy on eyes for long sessions

**Light Theme:**
- Clean white background
- Black text
- Blue accents
- Better for bright environments

**Toggle:** View → Toggle Theme or Ctrl+Shift+T

### Font Customization

Adjust text size for comfort:

- **Ctrl++**: Increase size
- **Ctrl+-**: Decrease size
- **Ctrl+0**: Reset to default (16pt)
- Font: JetBrains Mono (monospace with ligatures)

### Animation and Polish

Smooth, responsive UI:

- Animated pane resizing
- Fade-in autocomplete
- Smooth scrolling
- Ripple effects on buttons
- Loading spinners for async operations

## Advanced Features

### Multi-File Projects

Organize complex projects:

1. Create project directory
2. Open files in separate tabs
3. Reference types/functions across files
4. Use Editor for definitions, REPL for testing

**Example Structure:**
```
my_project/
  ├── types.dsl       # Type definitions
  ├── utils.dsl       # Utility functions
  └── workflow.dsl    # Main workflow
```

Load in REPL:
```dsl
:load types.dsl
:load utils.dsl
:load workflow.dsl
```

### Session Management

Save and restore work sessions:

**Auto-Save:**
- Workspace state saves on exit
- Unsaved files prompt before quit
- Crash recovery (auto-save every 30s)

**Manual Save:**
```dsl
// In REPL
:save session.dsl

// Later
:load session.dsl
```

### Integration with Terminal

Seamless workflow between GUI and CLI:

1. Develop in GUI (visual, interactive)
2. Save to `.dsl` file
3. Run from terminal: `dsl run script.dsl`
4. Or execute in REPL: `:load script.dsl`

### Performance Optimization

The GUI is optimized for responsiveness:

- Incremental parsing (Tree-sitter)
- Syntax highlighting on visible lines only
- Lazy rendering for large outputs
- Async evaluation (non-blocking UI)
- Efficient re-renders (egui immediate mode)

## Tips and Tricks

### Rapid Prototyping

Use split panes effectively:

1. **Editor**: Write complete functions/types
2. **REPL**: Test immediately without saving
3. Iterate quickly without switching windows

### Code Organization

Keep editor organized:

- One file per major component (types, functions, workflows)
- Use tabs to switch between related files
- Save frequently (Ctrl+S becomes muscle memory)

### REPL-Driven Development

1. Experiment in REPL first
2. Once satisfied, move to Editor
3. Refine and document in Editor
4. Verify in REPL one more time

### Keyboard Efficiency

Master these workflows:

- **Ctrl+O → Ctrl+Tab → Enter**: Open file, test in REPL
- **Ctrl+C → Ctrl+Tab → Ctrl+V**: Copy from Editor to REPL
- **Ctrl+L**: Clear REPL clutter frequently

### Large Datasets

When working with large data:

1. Load in REPL once: `let data = SQL("SELECT * FROM 'big.csv'")`
2. Experiment with transformations
3. Reuse `data` variable (no re-loading)
4. Use `:clear` to clean output, not variables

### Visual Debugging

Use rich output rendering:

```dsl
// See structure clearly
data
  |> Take(_, 5)  // Preview first 5 items
  |> _ as sample

// Renders as table - easy to spot issues
```

## Common Workflows

### 1. Building a Data Pipeline

```dsl
// Editor: Define pipeline
def processSales(filename) :=
  SQL("SELECT * FROM '" + filename + "'")
    |> Filter(_, def (row) := row.amount > 100)
    |> Map(_, def (row) := {
         category: row.category,
         total: row.amount * (1 + row.tax)
       })
    |> GroupBy(_, "category")

// REPL: Test with sample data
processSales("sales_sample.csv")
// See table output

// REPL: Refine if needed
// Editor: Update function
// REPL: Test again
```

### 2. LLM Workflow Development

```dsl
// Editor: Define multi-step workflow
def analyzeText(text) :=
  Par(
    Ask("Summarize: " + text),
    Ask("Keywords: " + text),
    Ask("Sentiment: " + text)
  ) as [summary, keywords, sentiment]

  {
    summary: summary,
    keywords: Split(keywords, ","),
    sentiment: sentiment
  }

// REPL: Test with sample text
let sampleText = "AI is transforming..."
analyzeText(sampleText)
// See formatted output

// Iterate on prompts in Editor based on results
```

### 3. Type-Driven Development

```dsl
// Editor: Define types first
type User {
  id: Int
  name: String
  email: String
}

type UserProfile {
  user: User
  posts: [Post]
  followers: Int
}

// REPL: Create test instances
let testUser = {
  id: 1,
  name: "Alice",
  email: "alice@example.com"
}

// Editor: Write functions using types
def enrichUser(user) :=
  {
    user: user,
    posts: fetchPosts(user.id),
    followers: countFollowers(user.id)
  }

// REPL: Test with mock data
enrichUser(testUser)
```

## Troubleshooting

### Application Won't Launch

**Problem:** GUI window doesn't appear

**Solutions:**
- Check GPU drivers are up to date
- Try software rendering: `dsl gui --renderer software`
- Verify egui dependencies are installed

### Slow Performance

**Problem:** UI is laggy or unresponsive

**Solutions:**
- Close unused tabs (Ctrl+W)
- Clear REPL output (Ctrl+L)
- Reduce font size (Ctrl+-)
- Disable animations in settings
- Check for infinite loops in code

### Autocomplete Not Working

**Problem:** Tab doesn't show suggestions

**Solutions:**
- Ensure cursor is at end of word
- Type at least one character
- Check if symbols are defined (`:vars`, `:funcs`)
- Restart application to rebuild symbol cache

### Syntax Highlighting Off

**Problem:** Code appears unstyled

**Solutions:**
- Check file has `.dsl` extension
- Restart application
- Verify Tree-sitter grammar is installed
- Check theme settings (View → Toggle Theme)

### Files Not Saving

**Problem:** Changes not persisting

**Solutions:**
- Check file permissions
- Verify disk space available
- Use "Save As" to different location
- Check status bar for save confirmation

## Next Steps

- [Terminal UI (TUI)](./tui.md) - Command-line interface alternative
- [CLI Reference](./cli.md) - Batch processing and automation
- [Workflows](../workflows/sequential.md) - Building complex workflows
- [Type System](../type-system/custom-types.md) - Defining structured data
