# TUI Interface Guide

## Overview

The DSL TUI (Terminal User Interface) provides three distinct modes for different workflows:

1. **REPL Mode (F1)** - Full-screen interactive REPL
2. **Workspace Mode (F2)** - Split view with Editor, REPL, and Preview
3. **Type Explorer (F3)** - Browse registered types and enums

Each mode is optimized for specific tasks and can be switched at any time.

## Mode 1: REPL Mode (F1)

### Overview
Full-screen REPL for interactive programming and quick experimentation.

### Layout
```
┌─────────────────────────────────────────────────────────────┐
│ Flow DSL v1.0                                    [REPL]     │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Output Area (scrollable)                                   │
│  - Shows command history                                    │
│  - Shows results                                            │
│  - Shows error messages                                     │
│  - Automatic word wrapping                                  │
│  - Mouse wheel scrolling                                    │
│                                                             │
│                                                             │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│ flow> [Your input here]_                                    │
└─────────────────────────────────────────────────────────────┘
 F1: REPL | F2: Workspace | F3: Types | Esc: Quit
```

### Features

#### 1. **Command History**
- **Up Arrow** - Previous command
- **Down Arrow** - Next command
- Persistent across sessions (saved in `.dsl_history`)
- Unlimited history size

#### 2. **Multi-line Input**
- **Shift+Enter** or **Alt+Enter** - Add new line without executing
- Automatic delimiter detection:
  - Unclosed `{`, `[`, `(` → Multi-line mode
  - Closed delimiters → Single line
- Visual indicator shows multi-line mode

Example:
```javascript
flow> {                    // Shift+Enter
...     sql: """          // Shift+Enter
...       SELECT *        // Shift+Enter
...       FROM data       // Shift+Enter
...     """               // Shift+Enter
... }                      // Enter to execute
```

#### 3. **Output Display**
- **Success** - Green checkmark `✓`
- **Error** - Red cross `✗`
- **Loading** - Spinner `⏳`
- **Type annotations** - Shows value type
- **Word wrapping** - Long lines automatically wrap
- **Scrolling** - Mouse wheel or arrow keys

#### 4. **Special Commands**
All special commands start with `:`:

| Command | Description | Example |
|---------|-------------|---------|
| `:vars` | List all variables | `:vars` |
| `:types` | List all registered types | `:types` |
| `:funcs` | List all user-defined functions | `:funcs` |
| `:help` | Show help message | `:help` |
| `:copy <file>` | Save last result to file | `:copy output.txt` |
| `:debug` | Toggle debug mode | `:debug` |
| `:save <file>` | Save session variables | `:save session.json` |
| `:load <file>` | Load session variables | `:load session.json` |

#### 5. **Cursor Movement**
- **Left/Right Arrow** - Move cursor
- **Home** - Start of line
- **End** - End of line
- **Ctrl+A** - Start of line
- **Ctrl+E** - End of line
- **Backspace** - Delete character
- **Delete** - Delete character forward

### Use Cases
- Quick calculations
- Testing expressions
- Interactive data exploration
- Function testing
- Learning the language

### Tips
1. Use **Up Arrow** to recall previous commands
2. Use **:vars** frequently to check state
3. The `_` variable always holds the last result
4. Scroll output with mouse wheel
5. **Shift+Enter** for multi-line input

---

## Mode 2: Workspace Mode (F2)

### Overview
Three-pane layout for developing workflows with editor, REPL, and preview.

### Layout
```
┌─────────────────────────────────────────────────────────────┐
│ Flow DSL v1.0                                  [WORKSPACE]  │
├─────────────────────────────────┬───────────────────────────┤
│  Editor          (40%)          │  REPL         (60%)       │
│ ┌─────────────────────────────┐ │ ┌─────────────────────────┤
│ │ 1 │ type Person {           │ │ │ flow> 42 as answer      │
│ │ 2 │   name: String          │ │ │ ✓ Bound 'answer' to 42  │
│ │ 3 │   age: Int              │ │ │                         │
│ │ 4 │ }                       │ │ │ flow> answer * 2        │
│ │ 5 │                         │ │ │ ✓ 84 : Int              │
│ │ 6 │ def greet(name) {       │ │ │                         │
│ │ 7 │   model: "gpt-4o-mini"  │ │ │ flow> greet("Alice")    │
│ │ 8 │   prompt: "Greet ${name│ │ │ ⏳ Loading...            │
│ │ 9 │ }                       │ │ │                         │
│ │10 │                         │ │ │                         │
│ └─────────────────────────────┘ │ └─────────────────────────┤
├─────────────────────────────────┴───────────────────────────┤
│  Preview / Type Explorer                                    │
│ ┌──────────────────────────────────────────────────────────┐│
│ │ Execution Steps:                                         ││
│ │ ✓ Step 1: type Person defined [0.01s]                    ││
│ │ ✓ Step 2: def greet defined [0.01s]                      ││
│ │ ⏳ Step 3: greet("Alice") [executing...]                  ││
│ └──────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
 Tab: Switch pane | Ctrl+R: Run | Ctrl+S: Save | Esc: Quit
```

### Features

#### 1. **Editor Pane** (Left, 40%)

**Capabilities:**
- Multi-line text editing
- Syntax highlighting
- Line numbers
- Cursor positioning
- Vertical scrolling
- File save/load

**Syntax Highlighting Colors:**
- **Keywords** (`type`, `enum`, `def`, `as`) - Magenta
- **Types** (`String`, `Int`, `Float`, `Bool`) - Blue
- **Operators** (`>>`, `||`, `?:`, `->`) - Cyan
- **Strings** - Green
- **Numbers** - Yellow
- **Comments** - Dark Gray

**Key Bindings:**
| Key | Action |
|-----|--------|
| **Arrow Keys** | Navigate cursor |
| **Home** | Start of line |
| **End** | End of line |
| **Enter** | New line |
| **Backspace** | Delete character |
| **Ctrl+S** | Save file |
| **Ctrl+R** | Run all content in REPL |
| **Ctrl+E** | Send current line to REPL |

**Use Cases:**
- Writing type definitions
- Defining functions
- Creating workflows
- Developing multi-line code
- Saving reusable code

#### 2. **REPL Pane** (Right, 60%)

**Same as REPL Mode**, but in a split view.

**Key Bindings:**
- Same as REPL Mode
- **Tab** - Switch to Editor pane

**Integration with Editor:**
- **Ctrl+R** in Editor - Run entire editor content
- **Ctrl+E** in Editor - Send current line to REPL
- Results appear in REPL pane
- Variables persist across panes

#### 3. **Preview Pane** (Bottom)

**Shows:**
- Execution steps with status
- Step timing
- Input/output for each step
- Error messages
- Registered types (when idle)

**Status Indicators:**
- **○** - Pending (not started)
- **⏳** - Running (in progress)
- **✓** - Complete (success)
- **✗** - Error (failed)

**Example:**
```
Execution Steps:
✓ Step 1: 42 as answer [0.01s]
  Output: 42 : Int
✓ Step 2: answer * 2 [0.01s]
  Output: 84 : Int
✓ Step 3: Length("hello") [0.01s]
  Output: 5 : Int
```

### Pane Navigation

**Switch Between Panes:**
- **Tab** - Cycle through: Editor → REPL → Preview → Editor
- Active pane has highlighted border

**Keyboard Focus:**
- Only the active pane receives keyboard input
- Editor pane: typing edits text
- REPL pane: typing enters commands
- Preview pane: read-only (no input)

### Workflow Patterns

#### Pattern 1: Type-Driven Development
```
1. Define types in Editor
2. Press Ctrl+R to register
3. View types in Preview pane
4. Test in REPL pane
5. Iterate
```

#### Pattern 2: Function Development
```
1. Write function in Editor
2. Press Ctrl+R to load
3. Test with different inputs in REPL
4. Edit function in Editor
5. Reload with Ctrl+R
6. Test again
```

#### Pattern 3: Interactive Workflow
```
1. Write workflow skeleton in Editor
2. Send to REPL with Ctrl+R
3. Test individual steps in REPL
4. Refine in Editor
5. Run complete workflow
```

### Use Cases
- Developing complex workflows
- Creating reusable functions
- Iterative development
- Learning with preview
- Professional development

---

## Mode 3: Type Explorer (F3)

### Overview
Browse all registered types, enums, and their fields.

### Layout
```
┌─────────────────────────────────────────────────────────────┐
│ Flow DSL v1.0                              [TYPE EXPLORER]  │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Registered Types                                           │
│                                                             │
│  type Person                                                │
│    name: String                                             │
│    age: Int                                                 │
│    email: String                                            │
│                                                             │
│  type BlogPost                                              │
│    title: String                                            │
│    content: String                                          │
│    tags: [String]                                           │
│    author: Person                                           │
│                                                             │
│  enum Status                                                │
│    Pending                                                  │
│    InProgress                                               │
│    Completed                                                │
│    Failed                                                   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
 F1: REPL | F2: Workspace | F3: Types | Esc: Quit
```

### Features

**Display:**
- All registered type definitions
- Field names and types
- Enum variants
- Scrollable list
- Clear formatting

**Navigation:**
- **Mouse Wheel** - Scroll through types
- **Arrow Keys** - Navigate (future)

### Use Cases
- Quick type reference
- Understanding data structures
- Verifying type definitions
- Learning type system

---

## Global Key Bindings

These work in **all modes**:

| Key | Action | Description |
|-----|--------|-------------|
| **F1** | REPL Mode | Switch to full REPL |
| **F2** | Workspace Mode | Switch to split view |
| **F3** | Type Explorer | Browse types |
| **Esc** | Quit | Exit application |
| **Ctrl+C** | Quit | Exit application |

---

## UI Elements

### Status Bar (Bottom)
Shows available key bindings for current mode.

**REPL Mode:**
```
F1: REPL | F2: Workspace | F3: Types | Esc: Quit
```

**Workspace Mode:**
```
Tab: Switch pane | Ctrl+R: Run | Ctrl+S: Save | Esc: Quit
```

**Type Explorer:**
```
F1: REPL | F2: Workspace | F3: Types | Esc: Quit
```

### Title Bar (Top)
Shows current mode and version.

**Format:**
```
Flow DSL v1.0                                [MODE NAME]
```

### Loading Indicator
Shown during async operations (LLM calls, HTTP requests).

**Indicator:** `⏳ Loading...` in title bar

---

## Visual Design

### Color Scheme

**Borders:**
- Active pane: Bright/highlighted
- Inactive pane: Dim/gray

**Text:**
- Normal text: White
- Success: Green
- Error: Red
- Loading: Yellow
- Type annotations: Cyan

**Syntax Highlighting:**
- Keywords: Magenta
- Types: Blue
- Operators: Cyan
- Strings: Green
- Numbers: Yellow
- Comments: Dark Gray

### Spacing and Layout

**Proportions (Workspace Mode):**
- Editor: 40% width
- REPL: 60% width
- Preview: 30% height (when visible)

**Padding:**
- 1 character padding inside borders
- Line numbers: Right-aligned with spacing

---

## Mouse Support

### Supported Actions

**REPL Output Area:**
- **Scroll Wheel Up** - Scroll up (3 lines)
- **Scroll Wheel Down** - Scroll down (3 lines)

**Future Enhancements:**
- Click to position cursor
- Click to select pane
- Drag to resize panes
- Right-click context menu

---

## Accessibility

### Visual
- High contrast colors
- Clear status indicators
- Consistent layout
- Readable fonts (depends on terminal)

### Keyboard
- All features accessible via keyboard
- No mouse required
- Standard key bindings
- Discoverable shortcuts (shown in status bar)

---

## Tips and Tricks

### REPL Mode
1. **Use history** - Up/Down arrows save time
2. **Scroll output** - Mouse wheel to review results
3. **Save work** - Use `:save` before experimenting
4. **Multi-line input** - Shift+Enter for complex expressions

### Workspace Mode
1. **Active pane** - Check border to see which pane is active
2. **Quick run** - Ctrl+R runs entire editor
3. **Line-by-line** - Ctrl+E for testing single lines
4. **Save often** - Ctrl+S to save work
5. **Use preview** - Watch execution in real-time

### Type Explorer
1. **Quick reference** - Press F3 anytime to check types
2. **Copy to editor** - Manually copy type definitions for reuse
3. **Verify** - Check if types registered correctly

---

## Troubleshooting

### Issue: Can't see cursor
**Solution:** In REPL, cursor is at the prompt. In Editor, look for blinking cursor.

### Issue: Text not appearing
**Solution:** Check which pane is active (highlighted border). Press Tab to switch.

### Issue: Can't scroll output
**Solution:** Make sure mouse is over the output area. Use arrow keys as alternative.

### Issue: Syntax highlighting not working
**Solution:** Syntax highlighting only works in Editor pane (Workspace mode).

### Issue: Preview pane empty
**Solution:** Preview shows execution steps. Run code with Ctrl+R to see steps.

---

## Next Steps

- **[04-Language-Features.md](04-Language-Features.md)** - Learn the language syntax
- **[05-Type-System.md](05-Type-System.md)** - Define custom types
- **[06-Functions.md](06-Functions.md)** - Create functions
- **[examples/](../examples/)** - Try example workflows

---

## Related Documents

- **[EDITOR_USAGE.md](../EDITOR_USAGE.md)** - Editor-specific guide
- **[README.md](../README.md)** - Project overview
- **[PROGRESS.md](../PROGRESS.md)** - Implementation status
