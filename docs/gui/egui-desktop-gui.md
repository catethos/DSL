# egui Desktop GUI

The DSL provides a modern desktop GUI built with egui, offering a rich interactive development environment with advanced features like charts, tables, markdown rendering, and more.

> **Note:** Both the terminal-based TUI (Ratatui) and the desktop GUI (egui) are actively maintained. This document focuses on the egui desktop application.

## Table of Contents

- [Why egui?](#why-egui)
- [Quick Start](#quick-start)
- [Features](#features)
- [User Interface](#user-interface)
- [Keyboard Shortcuts](#keyboard-shortcuts)
- [Output Rendering](#output-rendering)
- [Error Display](#error-display)
- [File Operations](#file-operations)
- [Tips and Tricks](#tips-and-tricks)
- [Troubleshooting](#troubleshooting)

---

## Why egui?

egui was chosen for several key reasons:

### Advantages

1. **Mature Ecosystem**
   - Well-documented with extensive examples
   - Large community and active development
   - Many third-party extensions and widgets

2. **Immediate Mode Simplicity**
   - Easy to learn and use
   - UI code reads like a description of what you want
   - No complex state management required

3. **Batteries Included**
   - Built-in widgets: TextEdit, ScrollArea, Table, etc.
   - Rich styling system
   - Automatic layout

4. **Cross-Platform**
   - Native desktop (macOS, Linux, Windows)
   - Web support via WASM
   - Consistent API across platforms

5. **Performance**
   - Efficient immediate-mode rendering
   - Only redraws when needed
   - Handles large UIs well

---

## Quick Start

### Installation

```bash
# From workspace root
cargo build --release -p dsl-egui

# First build takes longer (downloading/compiling dependencies)
# Subsequent builds are much faster
```

### Running

```bash
# Run from workspace root
cargo run --release -p dsl-egui

# Or from the crate directory
cd crates/dsl-egui
cargo run --release
```

### First Use

When you launch the application, you'll see:

```
┌────────────────────────────────────────────────────────┐
│ File  Edit  View                                       │
├──────────────────┬─────────────────────────────────────┤
│      REPL        │           Editor                    │
│                  │                                     │
│ Welcome to DSL   │                                     │
│ Interactive      │                                     │
│ Environment!     │  (empty)                            │
│                  │                                     │
│ Type ':help'     │                                     │
│ for help...      │                                     │
│                  │                                     │
│ flow> _          │                                     │
├──────────────────┴─────────────────────────────────────┤
│ Active: Repl | Shift+Tab: Switch pane | Ctrl+E: Send  │
└────────────────────────────────────────────────────────┘
```

---

## Features

### Completed Features ✅

**Core UI:**
- Resizable split-pane workspace (REPL left, Editor right)
- Menu bar with File, Edit, View menus
- Status bar with color-coded messages
- Mouse click to switch active pane

**REPL Pane:**
- Multi-line input with Enter to submit
- Command history navigation (Up/Down arrows)
- History persistence across sessions
- Special commands (`:help`, `:clear`, `:quit`)
- Scrollable output with mouse wheel

**Editor Pane:**
- Multi-line text editing
- Syntax highlighting with Tree-sitter
- File operations (Open, Save, Save As)
- Recent files menu (up to 10 files)

**Output Rendering:**
- Text output with syntax highlighting
- Rich error messages with expandable sections (Phase 1 complete)
- Table renderer with automatic detection
- Tree renderer with expand/collapse
- Markdown renderer with full formatting
- Image display with smart sizing
- Chart rendering (line, bar, scatter plots)

**Autocomplete:**
- Popup UI with keyboard navigation
- Context-aware suggestions
- Integration with dsl-autocomplete engine

**Performance Optimizations:**
- Output history limit (max 1000 items)
- Lazy image loading
- Visibility-based culling for large outputs
- Optimized rendering (no continuous redraws)

### Future Enhancements 📋

- Mouse text selection and clipboard (Phase 2)
- Rich error display enhancement (Phase 2)
- Theme customization
- Table column sorting
- Pie chart support
- Image zoom/pan controls

---

## User Interface

### Layout

The application uses a split-pane layout:

```
┌───────────────────────────────────────────────┐
│              Menu Bar                         │
├─────────────────────┬─────────────────────────┤
│                     │                         │
│     REPL Pane       │     Editor Pane         │
│   (Interactive)     │   (Multi-line code)     │
│                     │                         │
│  • Input area       │  • Syntax highlighting  │
│  • Output display   │  • File operations      │
│  • History          │  • Preview on run       │
│                     │                         │
├─────────────────────┴─────────────────────────┤
│              Status Bar                       │
└───────────────────────────────────────────────┘
```

### Menu Bar

**File Menu:**
- **New** - Clear editor content
- **Open** - Open .dsl file (native file picker)
- **Save** - Save current file
- **Save As** - Save with new filename
- **Recent Files** - Quick access to last 10 files
- **Quit** - Exit application

**Edit Menu:**
- **Copy** - Copy selected text
- **Paste** - Paste from clipboard
- **Clear Output** - Clear REPL output

**View Menu:**
- **Reset Layout** - Reset pane sizes to 50/50

### Resizing Panes

- Move mouse to vertical separator between panes
- When cursor changes to resize cursor, click and drag
- Release to set new size
- Or use **View > Reset Layout** to restore 50/50

---

## Keyboard Shortcuts

### Global Shortcuts

| Key | Action |
|-----|--------|
| **Shift+Tab** | Switch between REPL and Editor |
| **Cmd+Q** (macOS) / **Ctrl+Q** (Linux/Win) | Quit application |

### REPL Pane

| Key | Action |
|-----|--------|
| **Enter** | Submit input for evaluation |
| **Up ↑** | Previous command in history |
| **Down ↓** | Next command in history |
| **Tab** | Trigger autocomplete |
| **Esc** | Dismiss autocomplete popup |

### Editor Pane

| Key | Action |
|-----|--------|
| **Cmd+S** / **Ctrl+S** | Save file |
| **Cmd+E** / **Ctrl+E** | Send current line to REPL |
| **Cmd+R** / **Ctrl+R** | Run all editor content |
| **Cmd+O** / **Ctrl+O** | Open file |

### Autocomplete Popup

| Key | Action |
|-----|--------|
| **Up ↑** / **Down ↓** | Navigate suggestions |
| **Enter** / **Tab** | Accept suggestion |
| **Esc** | Dismiss popup |

---

## Output Rendering

The egui GUI supports rich output rendering with automatic type detection:

### Text Output

Simple text values are displayed with optional syntax highlighting:

```
flow> "Hello, World!"
"Hello, World!"

flow> 42
42
```

### Tables

Lists of maps are automatically rendered as tables:

```
flow> [{name: "Alice", age: 30}, {name: "Bob", age: 25}]

┌───────┬─────┐
│ name  │ age │
├───────┼─────┤
│ Alice │ 30  │
│ Bob   │ 25  │
└───────┴─────┘
```

### Trees

Nested structures are rendered as expandable trees:

```
flow> {user: {name: "Alice", address: {city: "NYC"}}}

▶ user
  ▶ name: "Alice"
  ▶ address
    • city: "NYC"
```

Click the **▶** arrow to expand/collapse sections.

### Markdown

Strings containing markdown are automatically rendered with formatting:

```
flow> "# Title\n\n- Item 1\n- Item 2\n\n**Bold** and *italic*"

# Title

- Item 1
- Item 2

**Bold** and *italic*
```

### Images

Image paths are automatically detected and displayed:

```
flow> "path/to/image.png"

[Image display with smart sizing]
```

Images are lazily loaded for performance.

### Charts

Chart data is automatically visualized:

```
flow> Chart("line", [{x: 1, y: 2}, {x: 2, y: 4}])

[Interactive line chart display]
```

Supported chart types:
- **Line charts** - Trends and time series
- **Bar charts** - Categorical comparisons
- **Scatter plots** - Data distributions

---

## Error Display

The egui GUI provides rich error display with expandable sections.

### Phase 1: Completed ✅

**Features:**
- Color-coded error types (red for errors)
- Expandable sections with ▶/▼ arrows
- Smart error pattern recognition
- Contextual suggestions

**Example Error Display:**

```
┌─────────────────────────────────────────────────┐
│ 🔴 UNKNOWN VARIABLE                             │
├─────────────────────────────────────────────────┤
│ Runtime error: Variable 'x' not found           │
│                                                 │
│ Variable: x                                     │
├─────────────────────────────────────────────────┤
│ ▶ 💡 Suggestions                                │
│   • Define the variable with 'let' before use  │
│   • Check for typos in the variable name       │
│   • Ensure the variable is in scope            │
└─────────────────────────────────────────────────┘
```

Click the **▶** arrow to expand the suggestions section.

### Recognized Error Types

- **Unknown Variable** - Variable not defined
- **Unknown Function** - Function not found
- **Parse Error** - Syntax errors
- **Type Error** - Type mismatches
- **Runtime Error** - Execution errors
- **LLM Error** - AI/LLM call failures
- **HTTP Error** - Network request failures
- **SQL Error** - Database query errors

### Phase 2: Planned 📋

Future enhancements include:
- Clickable source locations (file:line:column)
- Source code context with highlighting
- Stack traces for complex errors
- Error filtering and search

---

## File Operations

### Opening Files

**Method 1: Menu**
1. Click **File > Open**
2. Select .dsl file in native file picker
3. File content loads into editor

**Method 2: Recent Files**
1. Click **File > Recent Files**
2. Select from last 10 opened files

**Method 3: Keyboard Shortcut**
- Press **Cmd+O** (macOS) or **Ctrl+O** (Linux/Windows)

### Saving Files

**Save Current File:**
- Click **File > Save** or press **Cmd+S** / **Ctrl+S**
- Saves to current filename

**Save As New File:**
- Click **File > Save As**
- Choose location and filename in file picker
- Becomes current file for future saves

### Recent Files

The application tracks your last 10 opened files for quick access:
- Accessible via **File > Recent Files** menu
- Files are listed with full paths
- List persists across application restarts

---

## Tips and Tricks

### 1. Long Output

The output area is scrollable:
- Use mouse wheel to scroll
- Drag scrollbar for quick navigation
- Output auto-scrolls to bottom for new results

### 2. Focus Indicators

The active pane has a colored border:
- **REPL active:** Green/blue border
- **Editor active:** Green/blue border
- Click pane or use **Shift+Tab** to switch

### 3. Command History

REPL history is persistent:
- Saved to `~/.flow_history`
- Up to 1000 recent commands
- Available across sessions
- Use Up/Down arrows to navigate

### 4. Autocomplete

Trigger autocomplete with **Tab**:
- Shows keywords, functions, variables
- Navigate with Up/Down
- Accept with Enter/Tab
- Dismiss with Esc

### 5. Multi-line Input

In REPL, press **Enter** to submit:
- For multi-line expressions, use the Editor pane
- Or wrap in delimiters: `{ }`, `[ ]`, `( )`

### 6. Syntax Highlighting

Tree-sitter provides real-time syntax highlighting:
- Keywords in magenta
- Types in blue
- Strings in green
- Numbers in yellow
- Operators in cyan

### 7. Performance

For large outputs:
- Output history limited to 1000 items
- Older items automatically removed
- Images load lazily (only when visible)
- Charts optimized for smooth scrolling

---

## Troubleshooting

### Common Issues

#### "Command not found: cargo"

Install Rust from https://rustup.rs/

#### Build Errors

Install required build tools:
- **macOS:** `xcode-select --install`
- **Linux:** `sudo apt install build-essential`
- **Windows:** Install Visual Studio with C++ tools

#### Window Doesn't Appear

- Don't run in SSH session
- Ensure you have a desktop environment
- Check for graphics driver issues

#### High CPU Usage

- Normal during active use
- egui optimizes redraws automatically
- Close unused applications if necessary

#### Slow Rendering

- Check output history size (max 1000 items)
- Clear output with `:clear` command
- Restart application if needed

#### Autocomplete Not Working

- Ensure you're in REPL pane
- Press **Tab** to trigger
- Check for parse errors in current input

---

## Comparison with TUI

Both interfaces are maintained. Here's how they compare:

| Feature | TUI (Terminal) | GUI (egui) |
|---------|----------------|------------|
| **Window** | Terminal emulator | Native window |
| **Mouse** | Limited support | Full support |
| **Resizing** | Terminal-dependent | Smooth dragging |
| **Colors** | 256 colors | Full RGB |
| **Fonts** | Terminal font | System fonts |
| **Copy/Paste** | Terminal shortcuts | Standard shortcuts |
| **File Picker** | Text input | Native dialog |
| **Charts** | ASCII/Basic | Rich interactive |
| **Images** | Limited/ASCII | Full display |
| **Tables** | ASCII borders | Clean rendering |
| **Performance** | Very fast | Fast |
| **Resource Usage** | Very low | Low |

**Choose TUI if:**
- You work primarily in terminal
- You want minimal resource usage
- You need SSH access
- You prefer keyboard-only workflow

**Choose GUI if:**
- You want rich visualizations
- You work with charts and images
- You prefer mouse interaction
- You want modern UI/UX

---

## Getting Help

### In the Application

- Type `:help` in REPL for quick reference
- Check status bar for keyboard shortcuts
- View menu for feature access

### Documentation

- See [User Guide](../user-guide/01-Overview.md) for DSL language features
- See [Implementation Details](../developer/egui-implementation.md) for architecture
- See main [README](../../README.md) for project overview

### Reporting Issues

If you find bugs or have suggestions:
1. Check if it's a known issue
2. Note error message and steps to reproduce
3. Report in GitHub issues with:
   - OS and version
   - Steps to reproduce
   - Expected vs actual behavior
   - Screenshots if applicable

---

**Version:** 2.0
**Last Updated:** 2025-11-10
**Status:** Production Ready (with ongoing enhancements)

Enjoy using the DSL Desktop GUI! 🎉
