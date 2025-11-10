# GPUI Migration Plan: Ratatui to GPUI + Adabraka UI

This document outlines the complete migration strategy for replacing the terminal-based TUI (Ratatui) with a GPU-accelerated desktop UI using GPUI and Adabraka UI.

## Table of Contents

- [Overview](#overview)
- [Current Architecture](#current-architecture)
- [Target Architecture](#target-architecture)
- [Phase 0: Prerequisites & Setup](#phase-0-prerequisites--setup-week-1)
- [Phase 1: Basic Window & Layout](#phase-1-basic-window--layout-week-1-2)
- [Phase 2: Text Input & REPL Prompt](#phase-2-text-input--repl-prompt-week-2)
- [Phase 3: Scrollable Output Display](#phase-3-scrollable-output-display-week-3)
- [Phase 4: Output Item Renderers](#phase-4-output-item-renderers-week-3-4)
- [Phase 5: Syntax-Highlighted Editor](#phase-5-syntax-highlighted-editor-week-4-5)
- [Phase 6: Interactive Elements - Tree View](#phase-6-interactive-elements---tree-view-week-5)
- [Phase 7: Autocomplete Popup](#phase-7-autocomplete-popup-week-6)
- [Phase 8: Mouse Selection & Clipboard](#phase-8-mouse-selection--clipboard-week-6)
- [Phase 9: Keyboard Shortcuts & Focus Management](#phase-9-keyboard-shortcuts--focus-management-week-7)
- [Phase 10: History & Commands](#phase-10-history--commands-week-7)
- [Phase 11: Image Display](#phase-11-image-display-week-8)
- [Phase 12: Advanced Features - Table & Markdown](#phase-12-advanced-features---table--markdown-week-8)
- [Phase 13: Polish & Theme](#phase-13-polish--theme-week-9)
- [Phase 14: Testing & Migration Completion](#phase-14-testing--migration-completion-week-10)
- [Dependencies](#dependencies)
- [Learning Resources](#learning-resources)

---

## Overview

### Why Migrate?

- **Performance**: GPU-accelerated rendering vs terminal character grid
- **Richer UI**: Full color, transparency, shadows, animations
- **Better UX**: Native desktop features, smoother interactions
- **Modern**: GPUI is actively developed for Zed editor

### Timeline

**Estimated Duration**: 10 weeks for full migration

---

## Current Architecture

### Key Components (Ratatui-based)

```
crates/dsl-tui/
├── src/
│   ├── lib.rs              # Main TUI entry, event loop
│   ├── app.rs              # Application state (1139 lines)
│   ├── ui/
│   │   ├── render.rs       # Main rendering logic
│   │   ├── highlight.rs    # Tree-sitter syntax highlighting
│   │   ├── autocomplete.rs # Autocomplete popup rendering
│   │   └── banner.rs       # ASCII art banner
│   ├── editor.rs           # Editor pane rendering
│   ├── autocomplete.rs     # Autocomplete state management
│   ├── output_item.rs      # Output types (Text, Table, Tree, etc.)
│   └── renderers/          # Specialized renderers
│       ├── text.rs
│       ├── table.rs
│       ├── tree.rs
│       ├── markdown.rs
│       └── image.rs
```

### Features to Migrate

1. **Workspace Layout**: 50/50 split (REPL left, Editor right)
2. **REPL**:
   - Multi-line input with smart delimiters
   - History navigation
   - Scrollable output with mouse wheel
   - Multiple output types (Text, Tables, Trees, Markdown, Images)
   - Text selection with mouse
   - Clipboard operations
   - Autocomplete popup
3. **Editor**:
   - Tree-sitter syntax highlighting
   - File operations (load/save)
   - Multi-line editing
4. **Interactive Elements**:
   - Tree expand/collapse on click
   - Scrollbar
   - Status bar with keybindings
5. **Visual Features**:
   - Color-coded output
   - ASCII art banner
   - Image rendering

---

## Target Architecture

### New Structure (GPUI-based)

```
crates/dsl-gpui/
├── src/
│   ├── main.rs             # GPUI application entry
│   ├── workspace.rs        # Workspace component (split layout)
│   ├── repl/
│   │   ├── mod.rs          # REPL pane component
│   │   ├── input.rs        # Input field component
│   │   └── output.rs       # Scrollable output display
│   ├── editor/
│   │   ├── mod.rs          # Editor pane component
│   │   └── highlight.rs    # Syntax highlighting (reuse from TUI)
│   ├── autocomplete.rs     # Autocomplete popup component
│   ├── output_item.rs      # Output types (reuse structure)
│   ├── renderers/          # GPUI-based renderers
│   │   ├── text.rs
│   │   ├── table.rs
│   │   ├── tree.rs
│   │   ├── markdown.rs
│   │   └── image.rs
│   └── theme.rs            # Theme management
└── Cargo.toml
```

---

## Phase 0: Prerequisites & Setup (Week 1)

### Knowledge Required

1. **GPUI Fundamentals**
   - Component model & Render trait
   - Element system & method chaining
   - Context types (AppContext, WindowContext, ViewContext)
   - State management patterns
   - Event handling system

2. **Adabraka UI Basics**
   - Component library structure
   - Theme system & color tokens
   - Layout components (VStack, HStack)
   - Styled trait usage

### Web Search Queries

```
1. "GPUI getting started tutorial"
2. "GPUI Render trait example"
3. "GPUI hello world application"
4. "GPUI context types explained"
5. "GPUI state management patterns"
6. "Adabraka UI installation setup"
7. "Adabraka UI VStack HStack examples"
8. "Adabraka UI theme system usage"
9. "GPUI vs Ratatui differences"
10. "GPUI window creation example"
```

### Action Items

- [ ] Create new crate `crates/dsl-gpui`
- [ ] Add GPUI and Adabraka dependencies to Cargo.toml
- [ ] Build "Hello World" GPUI app to verify setup
- [ ] Experiment with basic window creation
- [ ] Test VStack/HStack layout components

### Reference Files

- **Existing**: `crates/dsl-tui/Cargo.toml` (lines 19-23)
- **New**: `crates/dsl-gpui/Cargo.toml`

### Expected Output

A basic GPUI application window with "Hello, World!" text.

---

## Phase 1: Basic Window & Layout (Week 1-2)

### Knowledge Required

1. **Window Management**
   - Creating application windows
   - Setting window bounds and options
   - Window lifecycle management

2. **Layout System**
   - Flexbox concepts in GPUI
   - Size constraints (percentages, fixed sizes)
   - div() element basics
   - Gap and spacing utilities

### Web Search Queries

```
1. "GPUI window creation API"
2. "GPUI window bounds and positioning"
3. "GPUI flexbox layout tutorial"
4. "GPUI div element examples"
5. "GPUI flex row column layout"
6. "GPUI percentage sizing constraints"
7. "GPUI gap spacing utilities"
8. "GPUI split pane layout example"
9. "Adabraka UI layout components guide"
10. "GPUI responsive layout patterns"
```

### Action Items

- [ ] Create main window with basic layout
- [ ] Implement 50/50 horizontal split (REPL left, Editor right)
- [ ] Add placeholder containers for each pane
- [ ] Implement status bar at bottom
- [ ] Test window resizing behavior

### Migration Target

**Replace**: `crates/dsl-tui/src/ui/render.rs:19-63` (draw_workspace function)

### Code Structure

```rust
// crates/dsl-gpui/src/workspace.rs
use gpui::*;

#[derive(Clone, Copy, PartialEq)]
pub enum WorkspacePane {
    Editor,
    Repl,
}

pub struct Workspace {
    active_pane: WorkspacePane,
}

impl Workspace {
    pub fn new(cx: &mut ViewContext<Self>) -> Self {
        Self {
            active_pane: WorkspacePane::Repl,
        }
    }
}

impl Render for Workspace {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .child(
                // Main content area with split panes
                div()
                    .flex()
                    .flex_row()
                    .flex_grow()
                    .child(
                        // REPL pane (left, 50%)
                        div()
                            .flex()
                            .flex_col()
                            .w_1_2()
                            .bg(rgb(0x1e1e1e))
                            .child("REPL Pane")
                    )
                    .child(
                        // Editor pane (right, 50%)
                        div()
                            .flex()
                            .flex_col()
                            .w_1_2()
                            .bg(rgb(0x2e2e2e))
                            .child("Editor Pane")
                    )
            )
            .child(
                // Status bar at bottom
                div()
                    .h(px(24.))
                    .bg(rgb(0x007acc))
                    .child("Status Bar")
            )
    }
}
```

### Expected Output

A window with two side-by-side panes and a status bar at the bottom.

---

## Phase 2: Text Input & REPL Prompt (Week 2)

### Knowledge Required

1. **Text Input Handling**
   - GPUI text input fields
   - Keyboard event handling
   - Cursor positioning
   - Multi-line input support

2. **State Updates**
   - Mutable state in components
   - Triggering re-renders
   - Event handlers and callbacks

### Web Search Queries

```
1. "GPUI text input field example"
2. "GPUI keyboard event handling"
3. "GPUI key press event dispatch"
4. "GPUI cursor positioning"
5. "GPUI multi-line text input"
6. "GPUI state update re-render"
7. "Adabraka UI Input component"
8. "GPUI on_key_down handler"
9. "GPUI text editing widget"
10. "GPUI input focus management"
11. "GPUI Enter key handling"
12. "GPUI character insertion text field"
```

### Action Items

- [ ] Implement REPL input prompt component
- [ ] Add keyboard event handlers (Enter, Backspace, Arrows)
- [ ] Handle cursor position tracking
- [ ] Implement character insertion at cursor
- [ ] Add multi-line input support (Ctrl+J for newline)
- [ ] Test input with various Unicode characters

### Migration Target

**Replace**: `crates/dsl-tui/src/lib.rs:246-368` (handle_repl_input function)

### State Structure

```rust
// crates/dsl-gpui/src/repl/input.rs
use gpui::*;

pub struct ReplInput {
    pub input: String,
    pub cursor_position: usize,
    pub multiline_mode: bool,
}

impl ReplInput {
    pub fn insert_char(&mut self, c: char, cx: &mut ViewContext<Self>) {
        self.input.insert(self.cursor_position, c);
        self.cursor_position += c.len_utf8();
        cx.notify(); // Trigger re-render
    }

    pub fn delete_char(&mut self, cx: &mut ViewContext<Self>) {
        if self.cursor_position > 0 {
            let mut char_start = self.cursor_position - 1;
            while char_start > 0 && !self.input.is_char_boundary(char_start) {
                char_start -= 1;
            }
            self.input.remove(char_start);
            self.cursor_position = char_start;
            cx.notify();
        }
    }
}

impl Render for ReplInput {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .child("flow> ")
            .child(&self.input)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, cx| {
                match event.keystroke.key.as_str() {
                    "enter" => { /* submit */ }
                    "backspace" => this.delete_char(cx),
                    _ => {
                        if let Some(c) = event.keystroke.key.chars().next() {
                            this.insert_char(c, cx);
                        }
                    }
                }
            }))
    }
}
```

### Expected Output

A functional input field in the REPL pane that accepts text input and handles basic editing.

---

## Phase 3: Scrollable Output Display (Week 3)

### Knowledge Required

1. **Scrolling Containers**
   - Creating scrollable views in GPUI
   - Scroll offset management
   - Mouse wheel events
   - Auto-scroll behavior

2. **Dynamic Content Rendering**
   - Rendering lists of elements
   - VStack for vertical content
   - Text wrapping and line breaks

### Web Search Queries

```
1. "GPUI scrollable container example"
2. "GPUI scroll view tutorial"
3. "GPUI mouse wheel event handling"
4. "GPUI scroll offset state"
5. "GPUI auto scroll to bottom"
6. "Adabraka UI VStack scrollable"
7. "GPUI dynamic list rendering"
8. "GPUI overflow scroll"
9. "GPUI scrollbar implementation"
10. "GPUI content height calculation"
11. "GPUI PageUp PageDown scroll"
```

### Action Items

- [ ] Create scrollable output container
- [ ] Implement scroll offset tracking
- [ ] Add mouse wheel scroll support
- [ ] Implement keyboard scroll (PageUp/PageDown)
- [ ] Add auto-scroll when new output arrives
- [ ] Create text line renderer for output

### Migration Target

**Replace**: `crates/dsl-tui/src/ui/render.rs:99-263` (draw_repl_pane function)

### Component Structure

```rust
// crates/dsl-gpui/src/repl/output.rs
use gpui::*;
use crate::output_item::OutputItem;

pub struct OutputDisplay {
    items: Vec<OutputItem>,
    scroll_offset: f32,
    auto_scroll: bool,
}

impl OutputDisplay {
    pub fn add_item(&mut self, item: OutputItem, cx: &mut ViewContext<Self>) {
        self.items.push(item);
        if self.auto_scroll {
            // Scroll to bottom
            self.scroll_offset = f32::MAX; // Will be clamped by scroll container
        }
        cx.notify();
    }

    pub fn scroll_up(&mut self, amount: f32, cx: &mut ViewContext<Self>) {
        self.auto_scroll = false;
        self.scroll_offset = (self.scroll_offset - amount).max(0.0);
        cx.notify();
    }
}

impl Render for OutputDisplay {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .overflow_y_scroll()
            .child(
                // Using Adabraka VStack for vertical layout
                VStack::new()
                    .gap(px(4.))
                    .children(self.items.iter().map(|item| {
                        item.render() // Render each output item
                    }))
            )
            .on_scroll_wheel(cx.listener(|this, event: &ScrollWheelEvent, cx| {
                this.scroll_up(event.delta.y, cx);
            }))
    }
}
```

### Expected Output

A scrollable list of output items in the REPL pane that responds to mouse wheel events.

---

## Phase 4: Output Item Renderers (Week 3-4)

### Knowledge Required

1. **Text Styling**
   - Color and background colors
   - Text formatting
   - Styled spans/runs

2. **Component Composition**
   - Building complex components from primitives
   - Conditional rendering
   - Child elements

### Web Search Queries

```
1. "GPUI text color styling"
2. "GPUI background color"
3. "GPUI styled text spans"
4. "GPUI conditional rendering"
5. "GPUI text formatting bold italic"
6. "GPUI error message styling"
7. "Adabraka UI theme colors"
8. "GPUI component composition patterns"
9. "GPUI text wrapping long lines"
10. "GPUI monospace font rendering"
```

### Action Items

- [ ] Implement Text output renderer
- [ ] Implement Error output renderer (red styling)
- [ ] Implement basic Table renderer
- [ ] Add markdown rendering support
- [ ] Test different output types

### Migration Target

**Replace**: `crates/dsl-tui/src/output_item.rs:75-294`

### Renderer Pattern

```rust
// crates/dsl-gpui/src/renderers/text.rs
use gpui::*;
use crate::output_item::OutputItem;

impl OutputItem {
    pub fn render(&self) -> impl IntoElement {
        match self {
            OutputItem::Text(s) => {
                div()
                    .text_color(rgb(0xcccccc))
                    .child(s.clone())
            }
            OutputItem::Error(e) => {
                div()
                    .text_color(rgb(0xff0000))
                    .child(format!("Error: {}", e))
            }
            OutputItem::Table { columns, rows, .. } => {
                // Use grid layout for table
                div()
                    .flex()
                    .flex_col()
                    .child(/* render header */)
                    .children(rows.iter().map(|row| {
                        /* render row */
                    }))
            }
            // ... other types
        }
    }
}
```

### Expected Output

Different types of output rendered with appropriate styling in the REPL pane.

---

## Phase 5: Syntax-Highlighted Editor (Week 4-5)

### Knowledge Required

1. **Tree-sitter Integration**
   - Using existing Tree-sitter code in GPUI
   - Mapping Tree-sitter tokens to colors
   - Performance optimization for highlighting

2. **Text Editor Widget**
   - Multi-line editing
   - Line numbering (optional)
   - Cursor rendering
   - Selection handling

### Web Search Queries

```
1. "GPUI text editor component"
2. "GPUI Tree-sitter syntax highlighting"
3. "GPUI multi-line text editor"
4. "GPUI cursor rendering"
5. "GPUI text selection"
6. "GPUI line numbers display"
7. "GPUI code editor widget"
8. "Zed editor GPUI text editor"
9. "GPUI monospace text rendering"
10. "GPUI syntax highlight spans"
11. "GPUI TextElement usage"
```

### Action Items

- [ ] Create multi-line editor component
- [ ] Integrate existing Tree-sitter highlighting
- [ ] Implement cursor position rendering
- [ ] Add line-by-line rendering with syntax colors
- [ ] Test with large files

### Migration Target

**Replace**:
- `crates/dsl-tui/src/editor.rs`
- `crates/dsl-tui/src/ui/render.rs:65-97`

### Reuse Existing Code

- `crates/dsl-tui/src/ui/highlight.rs` - Tree-sitter integration
  - Function: `highlight_text()` - can be adapted for GPUI
  - Color mapping logic can be reused

### Component Structure

```rust
// crates/dsl-gpui/src/editor/mod.rs
use gpui::*;
use crate::editor::highlight::highlight_text;

pub struct EditorPane {
    content: String,
    cursor_row: usize,
    cursor_col: usize,
    file_path: Option<String>,
}

impl Render for EditorPane {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let highlighted_spans = highlight_text(&self.content);

        div()
            .flex()
            .flex_col()
            .font_family("monospace")
            .child(
                // Render each line with syntax highlighting
                VStack::new()
                    .children(highlighted_spans.into_iter().map(|line_spans| {
                        div()
                            .children(line_spans.into_iter().map(|(text, color)| {
                                span()
                                    .text_color(color)
                                    .child(text)
                            }))
                    }))
            )
            .child(
                // Render cursor
                div()
                    .absolute()
                    .top(px(self.cursor_row as f32 * 20.0))
                    .left(px(self.cursor_col as f32 * 10.0))
                    .w(px(2.))
                    .h(px(20.))
                    .bg(rgb(0xffffff))
            )
    }
}
```

### Expected Output

A syntax-highlighted editor pane with cursor position and multi-line editing support.

---

## Phase 6: Interactive Elements - Tree View (Week 5)

### Knowledge Required

1. **Click Event Handling**
   - Mouse click events in GPUI
   - Click position detection
   - Interactive elements

2. **Stateful Components**
   - Managing expansion state
   - Re-rendering on state change

### Web Search Queries

```
1. "GPUI mouse click event handling"
2. "GPUI on_click handler"
3. "GPUI interactive elements"
4. "GPUI stateful component"
5. "GPUI tree view component"
6. "GPUI expandable list"
7. "GPUI toggle state on click"
8. "GPUI indented tree rendering"
9. "GPUI click coordinates"
10. "GPUI hover effects"
```

### Action Items

- [ ] Implement TreeNode renderer with indentation
- [ ] Add expand/collapse icons (▶/▼)
- [ ] Handle click events on tree nodes
- [ ] Update expansion state in HashMap
- [ ] Trigger re-render after state change
- [ ] Add hover effects for interactive elements

### Migration Target

**Replace**:
- `crates/dsl-tui/src/renderers/tree.rs`
- Click handling in `crates/dsl-tui/src/app.rs:432-497`

### Component Structure

```rust
// crates/dsl-gpui/src/renderers/tree.rs
use gpui::*;
use std::collections::HashMap;

pub struct TreeView {
    root: TreeNode,
    expanded_paths: HashMap<String, bool>,
}

impl TreeView {
    fn render_node(&self, node: &TreeNode, depth: usize) -> impl IntoElement {
        let is_expanded = self.expanded_paths.get(&node.path).copied().unwrap_or(false);
        let has_children = !node.children.is_empty();

        div()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex()
                    .pl(px((depth * 20) as f32))
                    .child(
                        // Expand/collapse icon
                        if has_children {
                            span().child(if is_expanded { "▼" } else { "▶" })
                        } else {
                            span().child("  ")
                        }
                    )
                    .child(node.key.as_ref().unwrap_or(&String::from("root")))
                    .on_click(cx.listener(|this, _event, cx| {
                        // Toggle expansion
                        let current = this.expanded_paths.get(&node.path).copied().unwrap_or(false);
                        this.expanded_paths.insert(node.path.clone(), !current);
                        cx.notify();
                    }))
                    .hover(|style| style.bg(rgb(0x3e3e3e)))
            )
            .when(is_expanded, |div| {
                div.children(node.children.iter().map(|child| {
                    self.render_node(child, depth + 1)
                }))
            })
    }
}
```

### Expected Output

An interactive tree view in the REPL output that expands/collapses on click.

---

## Phase 7: Autocomplete Popup (Week 6)

### Knowledge Required

1. **Positioned Elements**
   - Absolute positioning in GPUI
   - Popup/overlay rendering
   - Z-index/layering

2. **Filtered Lists**
   - Rendering suggestion lists
   - Highlighting selected item
   - Dynamic filtering

### Web Search Queries

```
1. "GPUI popup positioned element"
2. "GPUI absolute positioning"
3. "GPUI overlay rendering"
4. "GPUI z-index layering"
5. "GPUI dropdown menu example"
6. "GPUI list with selection"
7. "GPUI highlight selected item"
8. "GPUI autocomplete component"
9. "GPUI popup near cursor"
10. "GPUI KeyDown event filtering"
```

### Action Items

- [ ] Create positioned popup component
- [ ] Render suggestion list with VStack
- [ ] Highlight selected suggestion
- [ ] Position popup relative to cursor
- [ ] Add keyboard navigation (Up/Down/Tab)
- [ ] Implement suggestion acceptance

### Migration Target

**Replace**:
- `crates/dsl-tui/src/autocomplete.rs`
- `crates/dsl-tui/src/ui/autocomplete.rs`

### Reuse Existing Code

- **Keep as-is**: `crates/dsl-autocomplete` crate (just change UI rendering)

### Component Structure

```rust
// crates/dsl-gpui/src/autocomplete.rs
use gpui::*;
use dsl_autocomplete::Suggestion;

pub struct AutocompletePopup {
    suggestions: Vec<Suggestion>,
    selected_index: usize,
    cursor_position: Point<Pixels>,
}

impl Render for AutocompletePopup {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .absolute()
            .top(self.cursor_position.y + px(20.))
            .left(self.cursor_position.x)
            .bg(rgb(0x2e2e2e))
            .border_1()
            .border_color(rgb(0x3e3e3e))
            .shadow_lg()
            .child(
                VStack::new()
                    .children(self.suggestions.iter().enumerate().map(|(idx, suggestion)| {
                        div()
                            .px(px(8.))
                            .py(px(4.))
                            .when(idx == self.selected_index, |div| {
                                div.bg(rgb(0x094771))
                            })
                            .child(&suggestion.label)
                            .when(suggestion.detail.is_some(), |div| {
                                div.child(
                                    span()
                                        .text_color(rgb(0x888888))
                                        .child(suggestion.detail.as_ref().unwrap())
                                )
                            })
                    }))
            )
    }
}
```

### Expected Output

A popup menu near the cursor showing autocomplete suggestions with keyboard navigation.

---

## Phase 8: Mouse Selection & Clipboard (Week 6)

### Knowledge Required

1. **Mouse Event Tracking**
   - Mouse down/up/drag events
   - Selection range calculation
   - Visual selection feedback

2. **Clipboard Integration**
   - System clipboard access in GPUI
   - Copy/paste operations

### Web Search Queries

```
1. "GPUI mouse drag event"
2. "GPUI mouse down up events"
3. "GPUI text selection"
4. "GPUI selection highlighting"
5. "GPUI clipboard copy paste"
6. "GPUI system clipboard access"
7. "GPUI mouse event coordinates"
8. "GPUI drag tracking"
9. "GPUI selection range visual"
10. "Rust clipboard crate GPUI"
```

### Action Items

- [ ] Implement mouse down/drag/up tracking
- [ ] Calculate selection line range
- [ ] Add visual selection highlight
- [ ] Integrate clipboard operations (may reuse `arboard` crate)
- [ ] Add Ctrl+Shift+C shortcut

### Migration Target

**Replace**: `crates/dsl-tui/src/app.rs:499-592` (selection methods)

### Component Structure

```rust
// crates/dsl-gpui/src/repl/output.rs (extend)
pub struct OutputDisplay {
    // ... existing fields
    selection_start: Option<usize>,
    selection_end: Option<usize>,
    is_selecting: bool,
}

impl OutputDisplay {
    fn handle_mouse_down(&mut self, line: usize, cx: &mut ViewContext<Self>) {
        self.selection_start = Some(line);
        self.selection_end = Some(line);
        self.is_selecting = true;
        cx.notify();
    }

    fn handle_mouse_drag(&mut self, line: usize, cx: &mut ViewContext<Self>) {
        if self.is_selecting {
            self.selection_end = Some(line);
            cx.notify();
        }
    }

    fn copy_to_clipboard(&self) -> Result<(), String> {
        use arboard::Clipboard;
        if let Some(text) = self.get_selected_text() {
            let mut clipboard = Clipboard::new().map_err(|e| format!("{}", e))?;
            clipboard.set_text(text).map_err(|e| format!("{}", e))?;
            Ok(())
        } else {
            Err("No text selected".to_string())
        }
    }
}
```

### Expected Output

Text selection with mouse drag and clipboard copy functionality.

---

## Phase 9: Keyboard Shortcuts & Focus Management (Week 7)

### Knowledge Required

1. **Key Event Routing**
   - Global keyboard shortcuts
   - Focus management between panes
   - Modifier keys (Ctrl, Shift, Alt)

2. **Focus System**
   - Setting focus on elements
   - Tab navigation
   - Visual focus indicators

### Web Search Queries

```
1. "GPUI keyboard shortcuts"
2. "GPUI key modifiers Ctrl Shift"
3. "GPUI focus management"
4. "GPUI tab navigation"
5. "GPUI global key bindings"
6. "GPUI focus indicator styling"
7. "GPUI on_action handler"
8. "GPUI KeyBinding system"
9. "GPUI prevent default key event"
10. "GPUI focus switching between views"
```

### Action Items

- [ ] Implement global keyboard shortcuts (Ctrl+C quit, etc.)
- [ ] Add pane switching (Shift+Tab)
- [ ] Implement Ctrl+E (send line to REPL)
- [ ] Implement Ctrl+R (run all)
- [ ] Implement Ctrl+S (save file)
- [ ] Add visual focus indicators
- [ ] Test all keyboard shortcuts

### Migration Target

**Replace**: `crates/dsl-tui/src/lib.rs:370-422` (handle_workspace_input)

### Keyboard Shortcuts to Implement

| Shortcut | Action |
|----------|--------|
| Ctrl+C | Quit application |
| Ctrl+Shift+C | Copy selection to clipboard |
| Shift+Tab | Switch active pane |
| Tab | Trigger autocomplete |
| Ctrl+E | Send current line to REPL |
| Ctrl+R | Run all editor content |
| Ctrl+S | Save file |
| Ctrl+I | Toggle image display |
| Up/Down | Navigate history (REPL) / Autocomplete |
| PageUp/PageDown | Scroll output |

### Component Structure

```rust
// crates/dsl-gpui/src/workspace.rs (extend)
impl Workspace {
    fn handle_key_event(&mut self, event: &KeyDownEvent, cx: &mut ViewContext<Self>) {
        match (event.keystroke.key.as_str(), &event.keystroke.modifiers) {
            ("c", mods) if mods.control => {
                // Quit application
                cx.quit();
            }
            ("tab", mods) if mods.shift => {
                // Switch panes
                self.next_pane(cx);
            }
            ("e", mods) if mods.control => {
                // Send current line to REPL
                self.send_current_line_to_repl(cx);
            }
            // ... other shortcuts
            _ => {}
        }
    }
}

impl Render for Workspace {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .on_key_down(cx.listener(Self::handle_key_event))
            // ... rest of layout
    }
}
```

### Expected Output

All keyboard shortcuts working as expected with visual focus indicators.

---

## Phase 10: History & Commands (Week 7)

### Knowledge Required

1. **State Persistence**
   - Saving/loading history to disk
   - Session management

### Web Search Queries

```
1. "GPUI file operations"
2. "Rust save load history file"
3. "GPUI command palette"
4. "GPUI special command handling"
```

### Action Items

- [ ] Implement history navigation (Up/Down arrows)
- [ ] Save history to `~/.flow_history`
- [ ] Load history on startup
- [ ] Implement special commands (:help, :clear, :quit, etc.)

### Migration Target

**Replace**: `crates/dsl-tui/src/app.rs:595-673` (history methods)

### Special Commands to Implement

| Command | Description |
|---------|-------------|
| `:help` | Show help text |
| `:clear` | Clear output |
| `:q` / `:quit` | Quit application |
| `:vars` | List all variables |
| `:types` | List all types |
| `:funcs` | List all functions |
| `:save <file>` | Save session |
| `:load <file>` | Load session |
| `:debug` | Show debug info |

### Component Structure

```rust
// crates/dsl-gpui/src/repl/history.rs
use std::fs;
use std::path::PathBuf;

pub struct History {
    entries: Vec<String>,
    index: Option<usize>,
    temp: String,
}

impl History {
    pub fn load() -> Self {
        let entries = Self::load_from_file().unwrap_or_default();
        Self {
            entries,
            index: None,
            temp: String::new(),
        }
    }

    fn history_file_path() -> Option<PathBuf> {
        if let Some(home) = std::env::var_os("HOME") {
            let mut path = PathBuf::from(home);
            path.push(".flow_history");
            Some(path)
        } else {
            None
        }
    }

    fn load_from_file() -> Option<Vec<String>> {
        let path = Self::history_file_path()?;
        let content = fs::read_to_string(&path).ok()?;
        Some(content.lines().map(String::from).collect())
    }

    pub fn save(&self) {
        if let Some(path) = Self::history_file_path() {
            let start = self.entries.len().saturating_sub(1000);
            let content = self.entries[start..].join("\n");
            let _ = fs::write(&path, content);
        }
    }

    pub fn add(&mut self, entry: String) {
        if self.entries.last() != Some(&entry) {
            self.entries.push(entry);
            self.save();
        }
    }

    pub fn navigate_up(&mut self, current_input: &str) -> Option<String> {
        if self.entries.is_empty() {
            return None;
        }

        match self.index {
            None => {
                self.temp = current_input.to_string();
                self.index = Some(self.entries.len() - 1);
                Some(self.entries[self.entries.len() - 1].clone())
            }
            Some(idx) if idx > 0 => {
                self.index = Some(idx - 1);
                Some(self.entries[idx - 1].clone())
            }
            _ => None,
        }
    }

    pub fn navigate_down(&mut self) -> Option<String> {
        match self.index {
            Some(idx) if idx + 1 < self.entries.len() => {
                self.index = Some(idx + 1);
                Some(self.entries[idx + 1].clone())
            }
            Some(_) => {
                self.index = None;
                Some(self.temp.clone())
            }
            None => None,
        }
    }
}
```

### Expected Output

History navigation working with Up/Down arrows, and special commands functioning correctly.

---

## Phase 11: Image Display (Week 8)

### Knowledge Required

1. **Image Rendering**
   - Loading images in GPUI
   - Image display components
   - SVG support

### Web Search Queries

```
1. "GPUI image rendering"
2. "GPUI image element"
3. "GPUI load image file"
4. "GPUI image resize fit"
5. "GPUI SVG rendering"
6. "GPUI dynamic image display"
7. "GPUI image from path"
```

### Action Items

- [ ] Implement image loading from file path
- [ ] Create Image output renderer
- [ ] Add resize/fit logic
- [ ] Test with various image formats
- [ ] Add toggle for image display (Ctrl+I)

### Migration Target

**Replace**:
- `crates/dsl-tui/src/renderers/image.rs`
- Remove dependency: `ratatui-image`

### Component Structure

```rust
// crates/dsl-gpui/src/renderers/image.rs
use gpui::*;
use image::DynamicImage;
use std::sync::Arc;

pub struct ImageRenderer {
    path: String,
    data: Option<Arc<DynamicImage>>,
}

impl ImageRenderer {
    pub fn new(path: String) -> Self {
        let data = image::open(&path).ok().map(Arc::new);
        Self { path, data }
    }

    pub fn render(&self) -> impl IntoElement {
        if let Some(img_data) = &self.data {
            // GPUI has native image support
            div()
                .flex()
                .flex_col()
                .child(
                    img()
                        .source(/* convert DynamicImage to GPUI format */)
                        .max_w(px(800.))
                        .max_h(px(600.))
                        .object_fit(ObjectFit::Contain)
                )
        } else {
            div()
                .child(format!("[Image not found: {}]", self.path))
        }
    }
}
```

### Expected Output

Images displayed in the REPL output with proper sizing and fit.

---

## Phase 12: Advanced Features - Table & Markdown (Week 8)

### Knowledge Required

1. **Table Layout**
   - Grid layout for tables
   - Column sizing
   - Row rendering

2. **Markdown Rendering**
   - Markdown parsing (reuse pulldown-cmark)
   - Styled text from markdown

### Web Search Queries

```
1. "GPUI table component"
2. "GPUI grid layout"
3. "GPUI column sizing table"
4. "Adabraka UI Grid component"
5. "GPUI markdown rendering"
6. "GPUI rich text formatting"
7. "pulldown-cmark GPUI integration"
```

### Action Items

- [ ] Implement table renderer with columns/rows
- [ ] Add markdown parsing and rendering
- [ ] Test with complex tables and markdown

### Migration Target

**Replace**:
- `crates/dsl-tui/src/renderers/table.rs`
- `crates/dsl-tui/src/renderers/markdown.rs`

### Table Renderer Structure

```rust
// crates/dsl-gpui/src/renderers/table.rs
use gpui::*;

pub struct TableRenderer {
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl TableRenderer {
    pub fn render(&self) -> impl IntoElement {
        let column_count = self.columns.len();

        div()
            .flex()
            .flex_col()
            .border_1()
            .border_color(rgb(0x3e3e3e))
            .child(
                // Header row
                div()
                    .flex()
                    .flex_row()
                    .bg(rgb(0x2e2e2e))
                    .children(self.columns.iter().map(|col| {
                        div()
                            .flex_1()
                            .px(px(8.))
                            .py(px(4.))
                            .child(col.as_str())
                    }))
            )
            .children(self.rows.iter().map(|row| {
                div()
                    .flex()
                    .flex_row()
                    .border_t_1()
                    .border_color(rgb(0x3e3e3e))
                    .children(row.iter().map(|cell| {
                        div()
                            .flex_1()
                            .px(px(8.))
                            .py(px(4.))
                            .child(cell.as_str())
                    }))
            }))
    }
}
```

### Markdown Renderer Structure

```rust
// crates/dsl-gpui/src/renderers/markdown.rs
use gpui::*;
use pulldown_cmark::{Parser, Event, Tag};

pub fn render_markdown(markdown: &str) -> impl IntoElement {
    let parser = Parser::new(markdown);
    let mut elements = Vec::new();

    for event in parser {
        match event {
            Event::Start(Tag::Heading(level)) => {
                // Add heading with appropriate size
            }
            Event::Start(Tag::Paragraph) => {
                // Start paragraph
            }
            Event::Text(text) => {
                elements.push(div().child(text.to_string()));
            }
            Event::Code(code) => {
                elements.push(
                    div()
                        .bg(rgb(0x2e2e2e))
                        .px(px(4.))
                        .font_family("monospace")
                        .child(code.to_string())
                );
            }
            // ... handle other markdown elements
            _ => {}
        }
    }

    div()
        .flex()
        .flex_col()
        .gap(px(8.))
        .children(elements)
}
```

### Expected Output

Tables with proper column alignment and markdown with formatted text elements.

---

## Phase 13: Polish & Theme (Week 9)

### Knowledge Required

1. **Theme System**
   - Adabraka theme tokens
   - Light/dark mode switching
   - Custom color schemes

2. **Animations**
   - Transition animations
   - Easing functions

### Web Search Queries

```
1. "Adabraka UI theme customization"
2. "GPUI light dark theme switch"
3. "GPUI animations tutorial"
4. "GPUI transition effects"
5. "GPUI cubic bezier easing"
6. "GPUI theme colors semantic"
7. "GPUI border shadow styling"
```

### Action Items

- [ ] Implement theme switching
- [ ] Add animations for autocomplete popup
- [ ] Add animations for tree expand/collapse
- [ ] Polish visual styling (borders, shadows, spacing)
- [ ] Add hover effects
- [ ] Test both light and dark themes

### Theme Structure

```rust
// crates/dsl-gpui/src/theme.rs
use gpui::*;
use adabraka_ui::theme::{Theme, ThemeColor};

#[derive(Clone, Copy, PartialEq)]
pub enum AppTheme {
    Dark,
    Light,
}

impl AppTheme {
    pub fn background(&self) -> Hsla {
        match self {
            Self::Dark => rgb(0x1e1e1e),
            Self::Light => rgb(0xffffff),
        }
    }

    pub fn foreground(&self) -> Hsla {
        match self {
            Self::Dark => rgb(0xcccccc),
            Self::Light => rgb(0x000000),
        }
    }

    pub fn accent(&self) -> Hsla {
        match self {
            Self::Dark => rgb(0x007acc),
            Self::Light => rgb(0x0066cc),
        }
    }

    pub fn error(&self) -> Hsla {
        rgb(0xff0000)
    }

    // ... more theme colors
}

pub struct ThemeManager {
    current_theme: AppTheme,
}

impl ThemeManager {
    pub fn toggle(&mut self, cx: &mut ViewContext<impl Render>) {
        self.current_theme = match self.current_theme {
            AppTheme::Dark => AppTheme::Light,
            AppTheme::Light => AppTheme::Dark,
        };
        cx.notify();
    }
}
```

### Animation Examples

```rust
// Autocomplete popup with fade-in animation
div()
    .absolute()
    .opacity(0.0)
    .animate(Duration::from_millis(200))
    .opacity(1.0)
    .child(/* popup content */)

// Tree node expand with slide animation
div()
    .overflow_hidden()
    .h(px(0.))
    .animate(Duration::from_millis(150))
    .h(auto())
    .children(/* child nodes */)
```

### Expected Output

A polished UI with smooth animations and consistent theming.

---

## Phase 14: Testing & Migration Completion (Week 10)

### Action Items

#### Testing

- [ ] Create integration tests for key workflows
  - [ ] Test input submission and output display
  - [ ] Test editor save/load operations
  - [ ] Test autocomplete functionality
  - [ ] Test tree expand/collapse
  - [ ] Test history navigation
  - [ ] Test special commands

- [ ] Performance testing
  - [ ] Test with large output (1000+ lines)
  - [ ] Test with large files in editor
  - [ ] Test with many autocomplete suggestions
  - [ ] Monitor memory usage
  - [ ] Profile rendering performance

- [ ] Interaction testing
  - [ ] Test all keyboard shortcuts
  - [ ] Test mouse scroll in output
  - [ ] Test mouse selection and drag
  - [ ] Test clipboard copy/paste
  - [ ] Test window resizing
  - [ ] Test pane switching

#### Bug Fixes

- [ ] Fix any remaining visual glitches
- [ ] Fix edge cases in text editing
- [ ] Fix scroll position edge cases
- [ ] Fix autocomplete positioning edge cases

#### Documentation

- [ ] Update README with GPUI version instructions
- [ ] Document new build requirements
- [ ] Create migration guide for users
- [ ] Document keyboard shortcuts
- [ ] Add screenshots of new UI

#### Integration

- [ ] Update `crates/dsl-repl/src/main.rs` to use `dsl-gpui`
- [ ] Remove or deprecate `dsl-tui` crate
- [ ] Update workspace `Cargo.toml`
- [ ] Update CI/CD for new dependencies
- [ ] Test on multiple platforms (macOS, Linux, Windows)

### Migration Checklist

```markdown
# Feature Parity Checklist

## Layout & Navigation
- [ ] 50/50 split workspace layout
- [ ] Pane switching (Shift+Tab)
- [ ] Visual focus indicators
- [ ] Status bar with keybindings
- [ ] Window resizing

## REPL Features
- [ ] Text input with cursor positioning
- [ ] Multi-line input support
- [ ] Character insertion/deletion
- [ ] Unicode support (CJK characters, emojis)
- [ ] Enter to submit
- [ ] Ctrl+J for newline in input
- [ ] History navigation (Up/Down)
- [ ] History persistence (~/.flow_history)
- [ ] Special commands (:help, :clear, :quit, etc.)

## Output Display
- [ ] Scrollable output container
- [ ] Mouse wheel scrolling
- [ ] PageUp/PageDown scrolling
- [ ] Auto-scroll on new output
- [ ] Text output rendering
- [ ] Error output (red color)
- [ ] Table output with columns/rows
- [ ] Tree output with expand/collapse
- [ ] Markdown rendering
- [ ] Image rendering
- [ ] Banner/artwork display

## Editor Features
- [ ] Multi-line text editing
- [ ] Cursor positioning
- [ ] Tree-sitter syntax highlighting
- [ ] File load/save operations
- [ ] Ctrl+E (send current line to REPL)
- [ ] Ctrl+R (run all lines)
- [ ] Ctrl+S (save file)

## Interactive Features
- [ ] Autocomplete popup
- [ ] Autocomplete trigger (Tab)
- [ ] Autocomplete navigation (Up/Down)
- [ ] Autocomplete acceptance (Enter/Tab)
- [ ] Tree node click to expand/collapse
- [ ] Mouse text selection (Shift+Click or Right-Click drag)
- [ ] Clipboard copy (Ctrl+Shift+C)
- [ ] Selection highlighting

## Polish
- [ ] Theme support (dark/light)
- [ ] Animations (autocomplete, tree expand)
- [ ] Hover effects
- [ ] Proper color scheme
- [ ] Consistent spacing and padding
- [ ] Borders and shadows

## Performance
- [ ] Fast rendering with large outputs
- [ ] Efficient syntax highlighting
- [ ] Smooth scrolling
- [ ] Responsive interactions
```

### Final Steps

1. **Code cleanup**
   - Remove debug logging
   - Remove commented-out code
   - Format code consistently
   - Add documentation comments

2. **Release preparation**
   - Update version numbers
   - Write changelog
   - Tag release
   - Build release binaries

3. **User communication**
   - Announce migration
   - Share new features
   - Provide migration guide
   - Gather feedback

---

## Dependencies

### Required Cargo.toml

```toml
# crates/dsl-gpui/Cargo.toml
[package]
name = "dsl-gpui"
version = "1.0.0"
edition = "2021"

[[bin]]
name = "dsl-gui"
path = "src/main.rs"

[dependencies]
# GPUI framework
gpui = "0.2.0"

# Adabraka UI component library
adabraka-ui = "0.2.2"

# Core DSL engine
dsl-core = { path = "../dsl-core" }
dsl-ir = { path = "../dsl-ir" }
dsl-interpreter = { path = "../dsl-interpreter" }
dsl-autocomplete = { path = "../dsl-autocomplete" }

# Tree-sitter syntax highlighting
tree-sitter = { workspace = true }
tree-sitter-highlight = { workspace = true }
tree-sitter-language = { workspace = true }
tree-sitter-dsl = { path = "../../tree-sitter-dsl" }

# Async runtime
tokio = { workspace = true }

# Clipboard support
arboard = "3.4"

# Unicode width calculation
unicode-width = "0.1"

# Markdown rendering
pulldown-cmark = "0.13"

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }

# IndexMap for ordered maps
indexmap = { workspace = true }

# Image loading
image = { workspace = true }
```

### Build Requirements

```bash
# Requires Rust nightly
rustup install nightly
rustup default nightly

# Platform-specific dependencies may be required for GPUI
# Check GPUI documentation for details
```

---

## Learning Resources

### Essential Documentation

1. **GPUI Official Resources**
   - Official documentation: https://www.gpui.rs/
   - GitHub repository: https://github.com/zed-industries/gpui
   - Examples directory in GPUI repo

2. **Adabraka UI Resources**
   - GitHub repository: https://github.com/Augani/adabraka-ui
   - Component examples
   - Theme documentation

3. **Zed Editor Source Code**
   - GitHub: https://github.com/zed-industries/zed
   - Excellent examples of GPUI usage
   - Text editor implementation
   - Autocomplete implementation
   - Theme system

### Web Search Strategy

#### Phase-Specific Searches

For each phase, use the provided search queries in this order:

1. **Conceptual understanding** (e.g., "GPUI Render trait explained")
2. **API documentation** (e.g., "GPUI div element API")
3. **Code examples** (e.g., "GPUI text input example")
4. **Zed source code** (e.g., "Zed editor GPUI autocomplete implementation")

#### General Search Patterns

```
# Official documentation
"GPUI [feature] documentation"
"GPUI [feature] API reference"

# Examples
"GPUI [feature] example"
"GPUI [feature] tutorial"
"GPUI how to [task]"

# Zed source code (best examples)
"site:github.com/zed-industries/zed [feature]"
"Zed editor [feature] implementation"

# Community
"GPUI [feature] discussion"
"GPUI [feature] forum"
```

### Recommended Study Order

1. **Week 1**: Focus on GPUI fundamentals
   - Read entire GPUI documentation
   - Study Zed's main application structure
   - Build 3-5 simple GPUI examples

2. **Week 2-3**: Study specific components
   - Text input in Zed
   - Scrolling in Zed
   - Layout system examples

3. **Week 4-5**: Advanced topics
   - Editor implementation in Zed
   - Event handling patterns
   - State management

4. **Week 6-7**: Polish and refinement
   - Theme system
   - Animations
   - Performance optimization

### Code References from Zed

Search Zed's codebase for these implementations:

- **Text editing**: `crates/editor/src/editor.rs`
- **Autocomplete**: `crates/editor/src/completions.rs`
- **Scrolling**: Look for `scroll` implementations
- **Keyboard shortcuts**: `crates/workspace/src/workspace.rs`
- **Theme**: `crates/theme/src/`

---

## Migration Timeline Summary

| Week | Phase | Focus | Deliverable |
|------|-------|-------|-------------|
| 1 | 0-1 | Setup & Layout | Basic window with split panes |
| 2 | 2 | Input Handling | Functional REPL input |
| 3 | 3 | Scrollable Output | Scrollable output display |
| 3-4 | 4 | Output Renderers | Text, Error, Table rendering |
| 4-5 | 5 | Editor | Syntax-highlighted editor |
| 5 | 6 | Tree View | Interactive tree component |
| 6 | 7-8 | Interactions | Autocomplete & selection |
| 7 | 9-10 | Shortcuts & History | All keyboard shortcuts |
| 8 | 11-12 | Advanced Features | Images, Tables, Markdown |
| 9 | 13 | Polish | Themes & animations |
| 10 | 14 | Testing | Complete & tested |

---

## Risk Mitigation

### Potential Challenges

1. **GPUI Learning Curve**
   - **Risk**: GPUI is less documented than Ratatui
   - **Mitigation**: Study Zed source code extensively, start with simple examples

2. **Performance Issues**
   - **Risk**: Large outputs may be slow
   - **Mitigation**: Implement virtualization for long lists, profile early

3. **Platform Compatibility**
   - **Risk**: GPUI may have platform-specific issues
   - **Mitigation**: Test on all target platforms early

4. **Missing Features**
   - **Risk**: Some TUI features may be hard to replicate
   - **Mitigation**: Identify blockers early, have fallback plans

### Contingency Plans

- Keep `dsl-tui` crate as fallback
- Implement features incrementally
- Get user feedback early (after Phase 7)
- Consider hybrid approach if needed

---

## Success Criteria

The migration is complete when:

- [ ] All features from Ratatui version are implemented
- [ ] Performance is equal or better
- [ ] UI is polished and professional
- [ ] All tests pass
- [ ] Documentation is complete
- [ ] Users can switch seamlessly

---

## Next Steps

1. **Start with Phase 0**: Set up the project and verify GPUI installation
2. **Build incrementally**: Complete each phase before moving to the next
3. **Test continuously**: Don't wait until the end to test
4. **Get feedback early**: Share progress after Phase 7 for user feedback
5. **Document as you go**: Write docs alongside code

---

**Last Updated**: 2025-11-07
**Version**: 1.0
