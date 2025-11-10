# DSL egui - Quick Start Guide

## Installation

No installation needed if you have the source code. Just build and run!

## Building

```bash
# From workspace root
cargo build -p dsl-egui

# First build takes longer (downloading/compiling dependencies)
# Subsequent builds are much faster
```

## Running

```bash
# Run from workspace root
cargo run -p dsl-egui

# Or from the crate directory
cd crates/dsl-egui
cargo run
```

## First Use

When you launch the application, you'll see:

```
┌────────────────────────────────────────────────────────┐
│ File  Edit  View                                       │
├──────────────────┬─────────────────────────────────────┤
│      REPL        │           Editor                    │
│                  │                                     │
│ Welcome to DSL   │                                     │
│ Interactive      │  (empty)                            │
│ Environment!     │                                     │
│                  │                                     │
│ Type ':help'     │                                     │
│ for help...      │                                     │
│                  │                                     │
│ flow> _          │                                     │
├──────────────────┴─────────────────────────────────────┤
│ Active: Repl | Shift+Tab: Switch pane | Ctrl+E: Send  │
└────────────────────────────────────────────────────────┘
```

## Basic Usage

### 1. Evaluate Expressions

Type in the REPL and press **Enter**:

```
flow> 1 + 2
3

flow> "Hello, " + "World!"
"Hello, World!"

flow> [1, 2, 3, 4, 5]
[1, 2, 3, 4, 5]
```

### 2. Work with Data Structures

```
flow> {name: "Alice", age: 30, city: "NYC"}
{
  name: "Alice"
  age: 30
  city: "NYC"
}

flow> [1, 2, 3] | map(x -> x * 2)
[2, 4, 6]
```

### 3. Use History

- Press **Up arrow** to recall previous commands
- Press **Down arrow** to go forward in history
- History is preserved across sessions

### 4. Special Commands

Commands start with `:`:

```
flow> :help
Available commands:
  :help     - Show this help
  :clear    - Clear output
  :quit     - Quit application

flow> :clear
(output cleared)
```

### 5. Switch to Editor

- Press **Shift+Tab** to switch to the editor pane
- Type DSL code in the editor
- Press **Shift+Tab** again to switch back to REPL

### 6. Use the Menu

Click on the menu bar at the top:

- **File > Open**: Open a .dsl file (TODO)
- **File > Save**: Save editor content (TODO)
- **File > Quit**: Close the application
- **Edit > Copy/Paste**: Clipboard operations (TODO)
- **View > Reset Layout**: Reset pane sizes to 50/50

### 7. Resize Panes

- Move mouse to the vertical separator between REPL and Editor
- When cursor changes to resize cursor, click and drag
- Release to set new size

## Examples to Try

### Math Operations
```
flow> 10 + 20
30

flow> 100 / 4
25

flow> 2 ^ 8
256
```

### Lists and Transformations
```
flow> [1, 2, 3, 4, 5] | filter(x -> x > 2)
[3, 4, 5]

flow> [1, 2, 3] | map(x -> x * x)
[1, 4, 9]

flow> [1, 2, 3, 4] | fold(0, acc x -> acc + x)
10
```

### Strings
```
flow> "hello" | upper()
"HELLO"

flow> "  spaces  " | trim()
"spaces"

flow> "a,b,c" | split(",")
["a", "b", "c"]
```

### Maps/Objects
```
flow> person = {name: "Bob", age: 25}
{name: "Bob", age: 25}

flow> person.name
"Bob"

flow> person | keys()
["name", "age"]
```

### Tables
```
flow> data = [{name: "Alice", score: 95}, {name: "Bob", score: 87}]
┌───────┬───────┐
│ name  │ score │
├───────┼───────┤
│ Alice │ 95    │
│ Bob   │ 87    │
└───────┴───────┘
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| Enter | Submit REPL input |
| Up ↑ | Previous command in history |
| Down ↓ | Next command in history |
| Shift+Tab | Switch between REPL and Editor |
| Ctrl+C (in menu) | Quit application |

**Coming soon**:
- Ctrl+E: Send current line to REPL
- Ctrl+R: Run all editor content
- Ctrl+S: Save file
- Ctrl+Shift+C: Copy selected text

## Tips

1. **Long output**: The output area is scrollable - use mouse wheel or scrollbar
2. **Error messages**: Shown in red color
3. **Auto-scroll**: Output automatically scrolls to bottom for new results
4. **Focus**: The active pane has a colored border
5. **Tables**: Automatically formatted when you output list of objects

## Common Issues

### "Command not found: cargo"
- Install Rust: https://rustup.rs/

### "error: failed to run custom build command"
- Install C compiler (needed for tree-sitter):
  - macOS: `xcode-select --install`
  - Linux: `sudo apt install build-essential`
  - Windows: Install Visual Studio

### Window doesn't appear
- Don't run in SSH session
- Make sure you have a display/desktop environment

### High CPU usage
- This is normal during active use
- egui optimizes redraws automatically

## Getting Help

- Type `:help` in the REPL
- See full documentation: `docs/08-egui-Migration-Guide.md`
- Check README: `crates/dsl-egui/README.md`

## What's Different from TUI?

If you used the TUI (terminal) version:

| Feature | TUI | GUI |
|---------|-----|-----|
| Window | Terminal | Native window |
| Mouse | Limited | Full support |
| Resizing | Terminal-dependent | Smooth dragging |
| Colors | 256 colors | Full RGB |
| Fonts | Terminal font | System fonts |
| Copy/Paste | Terminal shortcuts | Standard Ctrl+C/V |
| File picker | Text input | Native dialog (coming) |

## Next Steps

After you're comfortable with the basics:

1. **Write complex expressions** in the editor
2. **Try DSL features** like pipes, maps, filters
3. **Explore** the menu options
4. **Customize** by resizing panes to your preference
5. **Check** the full migration guide for advanced features

## Reporting Issues

If you find bugs or have suggestions:

1. Note the error message
2. Try to reproduce with minimal example
3. Report in GitHub issues (if applicable)
4. Include:
   - OS and version
   - Steps to reproduce
   - Expected vs actual behavior

## Have Fun!

Enjoy using the DSL Interactive Environment! 🎉

---

**Version**: 1.0
**Date**: 2025-11-08
