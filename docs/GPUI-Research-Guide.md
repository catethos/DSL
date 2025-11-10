# GPUI Research Guide: Comprehensive Migration Reference

**Last Updated**: 2025-11-07
**Version**: 1.1
**Purpose**: Complete reference for GPUI migration from Ratatui TUI

**Update Notes (v1.1)**:
- Updated with correct Adabraka UI information from official repository
- Added 73+ component details and examples
- Included theme system documentation
- Added password input and form handling examples
- Corrected installation and setup instructions

---

## Table of Contents

- [1. Overview](#1-overview)
- [2. GPUI Fundamentals](#2-gpui-fundamentals)
- [3. Setup and Installation](#3-setup-and-installation)
- [4. Core Concepts](#4-core-concepts)
- [5. Layout System](#5-layout-system)
- [6. Event Handling](#6-event-handling)
- [7. State Management and Reactivity](#7-state-management-and-reactivity)
- [8. Text Styling and Formatting](#8-text-styling-and-formatting)
- [9. Scrolling and Dynamic Content](#9-scrolling-and-dynamic-content)
- [10. Interactive Components](#10-interactive-components)
- [11. Autocomplete and Popups](#11-autocomplete-and-popups)
- [12. Clipboard Integration](#12-clipboard-integration)
- [13. Tree-sitter Syntax Highlighting](#13-tree-sitter-syntax-highlighting)
- [14. Animations and Transitions](#14-animations-and-transitions)
- [14.5. Theme System (Adabraka UI)](#145-theme-system-adabraka-ui)
- [15. Image Rendering](#15-image-rendering)
- [16. Code Examples](#16-code-examples)
- [17. Learning Resources](#17-learning-resources)

---

## 1. Overview

### What is GPUI?

GPUI is a **hybrid immediate and retained mode, GPU-accelerated UI framework for Rust** created by the makers of the Zed code editor. It leverages graphics processors to render interfaces in parallel, addressing performance limitations of CPU-based rendering used by traditional desktop frameworks.

### Key Features

- **GPU-Accelerated**: Uses the GPU for parallel rendering, achieving 120+ FPS
- **Rust Native**: Written entirely in Rust with strong type safety
- **Flexbox & Grid**: Modern layout algorithms similar to CSS
- **Tailwind-like API**: Chainable method syntax for styling
- **Reactive State**: Entity-based state management with observers
- **Pre-1.0**: Still in active development, with breaking changes between versions

### Why GPUI over Ratatui?

| Feature | Ratatui | GPUI |
|---------|---------|------|
| Rendering | Terminal characters | GPU-accelerated pixels |
| Colors | 256 colors | Full RGB + transparency |
| Layout | Terminal grid | Flexbox + Grid |
| Performance | ~30-60 FPS | 120+ FPS |
| Mouse Input | Limited | Full support |
| Animations | None | Native support |
| Images | Limited (ASCII art) | Native image rendering |

---

## 2. GPUI Fundamentals

### Application Lifecycle

Every GPUI application starts with the `App` object, which coordinates all activities:

```rust
use gpui::*;

fn main() {
    Application::new().run(move |cx: &mut App| {
        // Application setup happens here
        cx.open_window(window_options, |window, cx| {
            // Window initialization
            cx.new_view(|cx| MyView::new(cx))
        });
    });
}
```

### Three Core Registers

GPUI organizes UI architecture into three registers:

#### 1. State Management with Entities

Entities are framework-owned data structures accessed through smart pointers (similar to `Rc`). They serve as the central state containers for your application.

```rust
// Creating an entity
let model = cx.new_model(|cx| MyModel::new());

// Accessing entity state
model.read(cx).some_field;

// Updating entity state
model.update(cx, |model, cx| {
    model.some_field = new_value;
    cx.notify(); // Trigger observers
});
```

#### 2. High-Level UI with Views

Views are entities that implement the `Render` trait. The framework calls `render()` on the root view each frame to build the element tree.

```rust
struct MyView {
    state: String,
}

impl Render for MyView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .child(&self.state)
    }
}
```

#### 3. Low-Level UI with Elements

Elements provide imperative control for custom rendering needs, including efficient list virtualization and specialized layouts. They offer maximum flexibility over rendering behavior.

---

## 3. Setup and Installation

### Prerequisites

**Rust Nightly Required**:
```bash
rustup install nightly
rustup default nightly
```

Or use `rust-toolchain.toml` in your project:
```toml
[toolchain]
channel = "nightly"
```

### Platform Requirements

**macOS**:
- Xcode from App Store or Apple Developer site
- Xcode command line tools
- Proper Xcode path configuration

**Linux**:
- Standard development tools
- X11 or Wayland support

**Windows**:
- MSVC toolchain

### Cargo.toml Setup

```toml
[package]
name = "my-gpui-app"
version = "0.1.0"
edition = "2024"

[dependencies]
gpui = { git = "https://github.com/zed-industries/zed" }

# Alternative: Use published version (when available)
# gpui = "0.2.2"
```

### Adabraka UI Component Library

Adabraka UI is a comprehensive UI component library for GPUI with **73+ polished, accessible components** inspired by shadcn/ui. It provides professional components for building desktop applications.

**Features**:
- 73+ components with full styling trait coverage
- Complete theme system (light/dark modes with semantic tokens)
- Responsive layout utilities (VStack, HStack, Grid)
- Professional animations with cubic-bezier easing
- Built-in code editor with syntax highlighting
- Full keyboard navigation and accessibility
- Type-safe Rust implementations
- High-performance optimized rendering

**Installation**:
```toml
[dependencies]
adabraka-ui = "0.2.2"
gpui = "0.2.0"
```

**Initialize Adabraka UI**:
```rust
use adabraka_ui::prelude::*;
use gpui::*;

fn main() {
    Application::new().run(|cx: &mut App| {
        adabraka_ui::init(cx); // Must be first line
        install_theme(cx, Theme::dark()); // or Theme::light()

        // ... rest of setup
    });
}
```

**Icon Setup** (Optional):
Icon assets require separate setup. Choose from:
- Lucide icons
- Heroicons
- Feather icons
- Phosphor icons

Refer to Adabraka UI documentation for icon configuration.

### First Build

Initial compilation takes several minutes as Cargo downloads and compiles dependencies. Subsequent builds are much faster due to caching.

```bash
cargo run
```

---

## 4. Core Concepts

### Context Types

GPUI uses context parameters (typically named `cx`) to provide access to application state and services.

#### App

The foundational context granting access to global application state. It "owns all entities' data and can be used to read or update the data referenced by an Entity&lt;T&gt;."

**Key Methods**:
- `cx.new_model()` - Create a model entity
- `cx.new_view()` - Create a view entity
- `cx.open_window()` - Open a new window
- `cx.quit()` - Quit the application

#### Context&lt;T&gt;

A specialized context provided when interacting with specific entities. It provides "additional methods related to that specific entity such as notifying observers and emitting events."

**Important**: Context&lt;T&gt; dereferences into App, meaning functions accepting App can also accept Context&lt;T&gt;.

**Key Methods**:
- `cx.notify()` - Notify observers of state changes
- `cx.emit(event)` - Emit typed events
- `cx.observe()` - Observe entity changes
- `cx.subscribe()` - Subscribe to entity events

#### Window

Provides access to individual window state, including its root view entity. Requires passing mutable App references or derived contexts to interact with global state.

#### AsyncApp and AsyncWindowContext

Enable asynchronous operations across await points. These have **static lifetimes** and can be created by calling `to_async()` on existing context references.

**Critical Distinction**: "Calls become fallible, because the context may outlive the window or even the app itself."

```rust
let async_cx = cx.to_async();

spawn(async move {
    // Can use async_cx across await points
    async_cx.update(|cx| {
        // Update app state
    }).ok()?;
});
```

#### TestAppContext

Purpose-built for testing scenarios. These contexts "panic if you attempt to access a non-existent app or window" and include test-specific features.

### The Render Trait

The `Render` trait has exactly one method: `render()`. This method takes the current state of your view and returns a fresh description of what should appear on screen.

**Method Signature**:
```rust
fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement
```

**Parameters**:
- `&mut self` - Mutable access to view's state
- `_window: &mut Window` - Access to window properties
- `_cx: &mut Context<Self>` - Context for GPUI systems

**Returns**: `impl IntoElement` - Built using `div()` and chainable styling methods

### Element System

Elements are temporary objects describing visual components. They use a builder pattern where chained methods create new element descriptions.

**Base Element**:
```rust
div()
    .flex()              // Enable flexbox
    .flex_col()          // Stack vertically
    .gap_3()             // Gap between children
    .child("Content")    // Add child element
```

**Method Categories**:
- **Layout**: `flex()`, `flex_col()`, `flex_row()`, `gap_*()`
- **Sizing**: `size()`, `w_*()`, `h_*()`, `size_full()`
- **Styling**: `bg()`, `border_*()`, `shadow_*()`, `text_color()`
- **Content**: `child()`, `children()`
- **Conditional**: `when(condition, |el| el.style())`
- **Events**: `on_click()`, `on_key_down()`, `on_scroll_wheel()`

---

## 5. Layout System

### Flexbox Layout

GPUI uses flexbox for layout, similar to CSS Flexbox.

#### Basic Flex Container

```rust
div()
    .flex()                 // Enable flexbox
    .flex_col()             // Vertical stacking (default: row)
    .justify_center()       // Main axis alignment
    .items_center()         // Cross axis alignment
    .gap_3()                // Gap between children
    .child(/* ... */)
```

#### Flex Direction

```rust
// Horizontal (row)
div().flex().flex_row()
    .child("Item 1")
    .child("Item 2")

// Vertical (column)
div().flex().flex_col()
    .child("Item 1")
    .child("Item 2")
```

#### Sizing

```rust
div()
    .size_full()            // 100% width and height
    .w_1_2()                // 50% width
    .h_px(200.0)            // Fixed pixel height
    .flex_1()               // Flex grow: 1
```

#### 50/50 Split Layout Example

```rust
div()
    .flex()
    .flex_row()
    .size_full()
    .child(
        div()
            .w_1_2()        // 50% width
            .bg(rgb(0x1e1e1e))
            .child("Left Pane")
    )
    .child(
        div()
            .w_1_2()        // 50% width
            .bg(rgb(0x2e2e2e))
            .child("Right Pane")
    )
```

### Grid Layout

GPUI also supports Grid layout for more complex layouts:

```rust
div()
    .grid()
    .grid_cols(3)           // 3 columns
    .gap_2()                // Gap between cells
    .children(items)
```

### Positioning

```rust
div()
    .absolute()             // Absolute positioning
    .top(px(10.0))
    .left(px(20.0))
```

### Color Helpers

```rust
rgb(0x1e1e1e)               // Hex color
gpui::red()                 // Named colors
gpui::green()
gpui::blue()
```

---

## 6. Event Handling

### Keyboard Events

#### Focus System

GPUI uses a focus system to determine which UI element receives keyboard input. The `FocusHandle` acts as a unique identifier.

**Setting Up Focus**:

```rust
struct MyView {
    focus_handle: FocusHandle,
}

impl MyView {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }
}

// Implement Focusable trait
impl Focusable for MyView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

// In render():
div()
    .track_focus(&self.focus_handle)
    .on_key_down(cx.listener(Self::handle_key_event))
```

#### Actions

Actions represent user intentions rather than raw input events. They can be triggered by keyboard shortcuts or mouse interactions.

**Define Actions**:
```rust
actions!(counter, [Increment, Reset]);
```

**Register Actions**:
```rust
impl Render for MyView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::handle_increment))
            .on_action(cx.listener(Self::handle_reset))
            .child("Content")
    }
}

impl MyView {
    fn handle_increment(&mut self, _: &Increment, _window: &mut Window, cx: &mut Context<Self>) {
        self.count += 1;
        cx.notify(); // Trigger re-render
    }

    fn handle_reset(&mut self, _: &Reset, _window: &mut Window, cx: &mut Context<Self>) {
        self.count = 0;
        cx.notify();
    }
}
```

**Bind Keys**:
```rust
cx.bind_keys([
    KeyBinding::new("space", Increment, None),
    KeyBinding::new("r", Reset, None),
]);
```

#### Direct Key Event Handling

For more direct control:

```rust
div()
    .on_key_down(cx.listener(|this, event: &KeyDownEvent, cx| {
        match event.keystroke.key.as_str() {
            "enter" => {
                // Handle Enter
                cx.notify();
            }
            "backspace" => {
                // Handle Backspace
                cx.notify();
            }
            _ => {
                // Handle character input
                if let Some(c) = event.keystroke.key.chars().next() {
                    this.insert_char(c);
                    cx.notify();
                }
            }
        }
    }))
```

#### Modifier Keys

```rust
match (event.keystroke.key.as_str(), &event.keystroke.modifiers) {
    ("c", mods) if mods.control => {
        // Ctrl+C
    }
    ("s", mods) if mods.control => {
        // Ctrl+S
    }
    ("tab", mods) if mods.shift => {
        // Shift+Tab
    }
    _ => {}
}
```

### Mouse Events

Mouse event handlers work regardless of focus state and respond to direct mouse interaction on specific elements.

#### Click Events

```rust
div()
    .on_mouse_down(MouseButton::Left, cx.listener(|this, event, cx| {
        // Handle mouse down
        cx.notify();
    }))
    .on_mouse_up(MouseButton::Left, cx.listener(|this, event, cx| {
        // Handle mouse up (preferred for clicks)
        cx.notify();
    }))
    .on_click(cx.listener(|this, event, cx| {
        // Handle click
        cx.notify();
    }))
```

**Best Practice**: Use `on_mouse_up` instead of `on_mouse_down` to prevent accidental triggers when users press the mouse button on an element but release elsewhere.

#### Hover Effects

```rust
div()
    .hover(|style| {
        style.bg(rgb(0x3e3e3e))
    })
    .child("Hover me")
```

#### Drag Events

```rust
struct MyView {
    is_dragging: bool,
    drag_start: Option<Point>,
}

impl Render for MyView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_drag_start))
            .on_mouse_move(cx.listener(Self::on_drag_move))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_drag_end))
    }
}

impl MyView {
    fn on_drag_start(&mut self, event: &MouseDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.is_dragging = true;
        self.drag_start = Some(event.position);
        cx.notify();
    }

    fn on_drag_move(&mut self, event: &MouseMoveEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.is_dragging {
            // Handle drag
            cx.notify();
        }
    }

    fn on_drag_end(&mut self, event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.is_dragging = false;
        cx.notify();
    }
}
```

#### Scroll Events

```rust
div()
    .overflow_y_scroll()
    .on_scroll_wheel(cx.listener(|this, event: &ScrollWheelEvent, cx| {
        this.scroll_offset += event.delta.y;
        cx.notify();
    }))
```

---

## 7. State Management and Reactivity

### Centralized Ownership

GPUI uses a centralized ownership system where the `AppContext` is the single top-level owner of all application state. When creating models or views, you pass ownership to the application.

### Model Handles

Rather than direct state ownership, work with model handles—lightweight identifiers that carry type information. These are "merely an inert identifier plus a compile-time type tag" that maintain reference counts.

**Handles only grant state access when paired with an AppContext reference.**

### Creating Entities

```rust
// Create a model
let model: Entity<MyModel> = cx.new_model(|cx| MyModel::new());

// Create a view
let view: Entity<MyView> = cx.new_view(|cx| MyView::new(cx));
```

### Updating State

To update state, call the `update()` method on a handle:

```rust
model.update(cx, |model, cx| {
    model.value = new_value;
    cx.notify(); // Notify observers
});
```

### Reactivity Through Effects

GPUI implements a "run-to-completion" event system. When `emit()` or `notify()` is called, data queues as effects rather than triggering handlers synchronously. The system flushes these effects at each update cycle's end.

**This prevents reentrancy bugs common in traditional event emitters.**

### Observation Patterns

#### observe() - General State Changes

Watches for general state changes via `notify()`:

```rust
cx.observe(&model, |view, model, cx| {
    // Called whenever model.notify() is called
    view.update_from_model(model.read(cx));
    cx.notify();
})
```

#### subscribe() - Typed Events

Listens for typed events from objects implementing `EventEmitter`:

```rust
cx.subscribe(&model, |view, model, event: &MyEvent, cx| {
    // Called when model emits MyEvent
    view.handle_event(event);
    cx.notify();
})
```

#### Subscription Management

Both return `Subscription` objects that can be `detach()`ed or dropped to control handler lifecycle:

```rust
let subscription = cx.observe(&model, |view, model, cx| { /* ... */ });

// Later: stop observing
subscription.detach();
```

### Triggering Re-renders

**Always call `cx.notify()` after state changes** to inform GPUI that state has changed and trigger a re-render on the next frame.

```rust
impl MyView {
    fn update_value(&mut self, new_value: i32, cx: &mut Context<Self>) {
        self.value = new_value;
        cx.notify(); // Critical!
    }
}
```

---

## 8. Text Styling and Formatting

### Basic Text

```rust
div()
    .child("Hello, World!")
```

### Text Color

```rust
div()
    .text_color(rgb(0xffffff))
    .child("White text")
```

### Text Size

```rust
div()
    .text_xl()              // Extra large
    .child("Big text")
```

### Font Family

```rust
div()
    .font_family("monospace")
    .child("Monospace text")
```

### Styled Text Spans

For more complex text with multiple styles:

```rust
div()
    .flex()
    .child(
        span()
            .text_color(rgb(0xff0000))
            .child("Red ")
    )
    .child(
        span()
            .text_color(rgb(0x00ff00))
            .child("Green ")
    )
    .child(
        span()
            .text_color(rgb(0x0000ff))
            .child("Blue")
    )
```

### Error Message Styling

```rust
div()
    .text_color(rgb(0xff0000))
    .child(format!("Error: {}", error_message))
```

### Text Formatting Methods

```rust
div()
    .text_xs()              // Extra small
    .text_sm()              // Small
    .text_base()            // Base (default)
    .text_lg()              // Large
    .text_xl()              // Extra large
    .text_2xl()             // 2x extra large
    // ... and more
```

---

## 9. Scrolling and Dynamic Content

### Scrollable Containers

```rust
div()
    .overflow_y_scroll()    // Vertical scrolling
    .overflow_x_scroll()    // Horizontal scrolling
    .overflow_scroll()      // Both directions
```

### Scroll State Management

```rust
struct OutputDisplay {
    items: Vec<OutputItem>,
    scroll_offset: f32,
    auto_scroll: bool,
}

impl OutputDisplay {
    fn scroll_up(&mut self, amount: f32, cx: &mut Context<Self>) {
        self.auto_scroll = false;
        self.scroll_offset = (self.scroll_offset - amount).max(0.0);
        cx.notify();
    }

    fn scroll_to_bottom(&mut self, cx: &mut Context<Self>) {
        self.scroll_offset = f32::MAX; // Will be clamped
        cx.notify();
    }
}
```

### Handling Scroll Events

```rust
div()
    .overflow_y_scroll()
    .on_scroll_wheel(cx.listener(|this, event: &ScrollWheelEvent, cx| {
        this.scroll_up(event.delta.y, cx);
    }))
    .children(self.items.iter().map(|item| {
        item.render()
    }))
```

### Dynamic Lists

Use Adabraka UI's VStack for vertical layouts:

```rust
use adabraka_ui::prelude::*;

VStack::new()
    .gap(px(4.))
    .children(self.items.iter().map(|item| {
        div().child(item.render())
    }))
```

### Layout Utilities Example

```rust
use adabraka_ui::prelude::*;

// Vertical stack
VStack::new()
    .p(px(32.0))
    .gap(px(16.0))
    .child(h1("Welcome"))
    .child(p("This is a paragraph"))

// Horizontal stack
HStack::new()
    .gap(px(8.0))
    .child(Button::new("btn1", "Button 1"))
    .child(Button::new("btn2", "Button 2"))

// Grid layout
Grid::new()
    .cols(3)
    .gap(px(16.0))
    .children(items.iter().map(|item| {
        Card::new().child(item.render())
    }))
```

### Auto-scroll Behavior

```rust
impl OutputDisplay {
    fn add_item(&mut self, item: OutputItem, cx: &mut Context<Self>) {
        self.items.push(item);
        if self.auto_scroll {
            self.scroll_to_bottom(cx);
        }
        cx.notify();
    }
}
```

### Performance: List Virtualization

For large lists (1000+ items), consider virtualization. GPUI provides low-level element APIs for efficient list rendering. See Zed's editor implementation for examples.

---

## 10. Interactive Components

### Adabraka UI Components

Adabraka UI provides **73+ professional components** across multiple categories:

**Input & Forms**: Button, IconButton, Input, Textarea, Checkbox, Toggle, Select, SearchInput, Code Editor, SQL Editor, Radio, RadioGroup, ColorPicker, DatePicker, Calendar, Combobox

**Navigation**: Sidebar, MenuBar, Tabs, Breadcrumbs, Tree, Toolbar, StatusBar, Command Palette, Context Menu

**Data Display**: Table, DataTable, Card, Badge, Accordion, Progress, Spinner, Text, Avatar, AvatarGroup

**Overlays**: Dialog, Modal, Popover, Tooltip, Toast, Alert, AlertDialog

**Layout**: VStack, HStack, Grid, Scrollable, Resizable Panels

### Button Example

```rust
use adabraka_ui::prelude::*;

Button::new("click-me", "Click Me!")
    .variant(ButtonVariant::Default)
    .size(ButtonSize::Md)
    .on_click(|_event, _window, _cx| {
        println!("Button clicked!");
    })
```

### Styling Components

All 54 Adabraka UI components implement the `Styled` trait for complete customization:

```rust
Button::new("styled-button", "Styled")
    .variant(ButtonVariant::Primary)
    .bg(rgb(0x007acc))           // Background color
    .px_6()                       // Horizontal padding
    .py_4()                       // Vertical padding
    .rounded_lg()                 // Border radius
    .shadow_lg()                  // Shadow
    .text_color(rgb(0xffffff))    // Text color
```

**Available Styling Methods**:
- **Backgrounds/Colors**: `.bg()`, `.text_color()`, `.border_color()`
- **Spacing**: `.p_4()`, `.px_6()`, `.m_4()`, `.mx_auto()`
- **Borders/Radius**: `.border_2()`, `.rounded_lg()`, `.rounded_xl()`
- **Sizing**: `.w_full()`, `.h_full()`, `.w()`, `.h()`
- **Effects**: `.shadow_sm()`, `.shadow_lg()`, `.opacity()`

### Input Example

```rust
use adabraka_ui::prelude::*;

Input::new("username")
    .placeholder("Enter username")
    .size(InputSize::Md)
    .on_change(|value, _window, _cx| {
        println!("Input changed: {}", value);
    })
```

### Password Input

Adabraka UI v0.2.2+ includes password input with toggle visibility:

```rust
use adabraka_ui::prelude::*;

PasswordInput::new("password")
    .placeholder("Enter password")
    .on_change(|value, _window, _cx| {
        println!("Password: {}", value);
    })
```

**Features**:
- Eye icon toggle for revealing/hiding passwords
- Immediate state updates
- Full keyboard navigation (Tab/Shift-Tab between form inputs)

### Icon Example

```rust
use adabraka_ui::prelude::*;

Icon::new("search")
    .size(IconSize::Large)
    .p_2()
    .text_color(rgb(0x007acc))
```

### Text Component

```rust
use adabraka_ui::prelude::*;

// Typography helpers
h1("Main Title")
h2("Subtitle")
p("This is a paragraph")
code("inline code")

// Custom text
Text::new("Custom text")
    .variant(TextVariant::Body)
    .text_color(rgb(0xcccccc))
```

### Toggle/Checkbox

```rust
struct MyView {
    is_checked: bool,
}

impl Render for MyView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .child(
                div()
                    .bg(if self.is_checked {
                        rgb(0x007acc)
                    } else {
                        rgb(0x3e3e3e)
                    })
                    .child(if self.is_checked { "☑" } else { "☐" })
                    .on_click(cx.listener(|this, _event, cx| {
                        this.is_checked = !this.is_checked;
                        cx.notify();
                    }))
            )
    }
}
```

### Tree View with Expand/Collapse

```rust
use std::collections::HashMap;

struct TreeView {
    root: TreeNode,
    expanded_paths: HashMap<String, bool>,
}

impl TreeView {
    fn render_node(&self, node: &TreeNode, depth: usize, cx: &mut Context<Self>) -> impl IntoElement {
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
                        if has_children {
                            span().child(if is_expanded { "▼" } else { "▶" })
                        } else {
                            span().child("  ")
                        }
                    )
                    .child(&node.key)
                    .on_click(cx.listener(move |this, _event, cx| {
                        let current = this.expanded_paths.get(&node.path).copied().unwrap_or(false);
                        this.expanded_paths.insert(node.path.clone(), !current);
                        cx.notify();
                    }))
                    .hover(|style| style.bg(rgb(0x3e3e3e)))
            )
            .when(is_expanded, |div| {
                div.children(node.children.iter().map(|child| {
                    self.render_node(child, depth + 1, cx)
                }))
            })
    }
}
```

### Table Renderer

```rust
struct TableRenderer {
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl TableRenderer {
    fn render(&self) -> impl IntoElement {
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

---

## 11. Autocomplete and Popups

### Positioned Popup

```rust
struct AutocompletePopup {
    suggestions: Vec<Suggestion>,
    selected_index: usize,
    cursor_position: Point<Pixels>,
}

impl Render for AutocompletePopup {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
                            .hover(|style| style.bg(rgb(0x3e3e3e)))
                    }))
            )
    }
}
```

### Autocomplete Navigation

```rust
impl AutocompletePopup {
    fn move_selection_up(&mut self, cx: &mut Context<Self>) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            cx.notify();
        }
    }

    fn move_selection_down(&mut self, cx: &mut Context<Self>) {
        if self.selected_index < self.suggestions.len() - 1 {
            self.selected_index += 1;
            cx.notify();
        }
    }

    fn accept_suggestion(&mut self) -> Option<String> {
        self.suggestions.get(self.selected_index)
            .map(|s| s.label.clone())
    }
}
```

### Zed's Autocomplete Architecture

From the Zed codebase research:

- **Two phases**: Request (ask language server) and Filtering (fuzzy sort locally)
- **Continuous re-request**: As user types, re-request from server
- **While waiting**: Re-filter existing list for responsiveness
- **Manual trigger**: Ctrl+Space or `editor::ShowCompletions` action
- **Acceptance**: Tab or Enter

**Key files in Zed**:
- `crates/editor/src/editor.rs` - Main editor with completion handling
- `crates/languages/src/c.rs` - Completion label formatting examples

---

## 12. Clipboard Integration

### GPUI's Built-in Clipboard

GPUI has built-in clipboard functionality that works across platforms. X11 clipboard functionality is integrated directly (stripped from 1Password/arboard).

### Platform Differences

**Linux**: Multiple clipboards (Primary, Clipboard, Secondary)
**macOS/Windows**: Single clipboard

### Basic Clipboard Operations

```rust
// Write to clipboard
cx.write_to_clipboard(ClipboardItem::new(text));

// Read from clipboard
if let Some(item) = cx.read_from_clipboard() {
    let text = item.text();
}
```

### Linux-Specific Clipboard

```rust
// Write to primary selection (Linux)
cx.write_to_primary(ClipboardItem::new(text));

// Read from primary selection (Linux)
if let Some(item) = cx.read_from_primary() {
    let text = item.text();
}
```

### Using arboard Directly

If you need more control, you can use `arboard` crate directly:

```toml
[dependencies]
arboard = "3.4"
```

```rust
use arboard::Clipboard;

fn copy_to_clipboard(&self, text: &str) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| format!("{}", e))?;
    clipboard.set_text(text).map_err(|e| format!("{}", e))?;
    Ok(())
}

fn paste_from_clipboard(&self) -> Result<String, String> {
    let mut clipboard = Clipboard::new().map_err(|e| format!("{}", e))?;
    clipboard.get_text().map_err(|e| format!("{}", e))
}
```

### Text Selection and Copy

```rust
struct OutputDisplay {
    selection_start: Option<usize>,
    selection_end: Option<usize>,
    is_selecting: bool,
}

impl OutputDisplay {
    fn handle_mouse_down(&mut self, line: usize, cx: &mut Context<Self>) {
        self.selection_start = Some(line);
        self.selection_end = Some(line);
        self.is_selecting = true;
        cx.notify();
    }

    fn handle_mouse_drag(&mut self, line: usize, cx: &mut Context<Self>) {
        if self.is_selecting {
            self.selection_end = Some(line);
            cx.notify();
        }
    }

    fn copy_selection(&self, cx: &mut Context<Self>) -> Result<(), String> {
        if let Some(text) = self.get_selected_text() {
            cx.write_to_clipboard(ClipboardItem::new(text));
            Ok(())
        } else {
            Err("No text selected".to_string())
        }
    }
}
```

---

## 13. Tree-sitter Syntax Highlighting

### GPUI Component Integration

GPUI Component features syntax highlighting via Tree-sitter, with a high-performance code editor supporting 200K+ lines with LSP integration.

### Key Features

- **Lazy/batched parsing**: Parse syntax only for visible cells or in priority batches
- **Partial repaint**: Only re-render changed regions
- **Direct optimization**: Tight coupling with renderer for performance

### Basic Integration

```rust
use tree_sitter::{Parser, Language};
use tree_sitter_highlight::{Highlighter, HighlightConfiguration, HighlightEvent};

struct EditorPane {
    content: String,
    highlighter: Highlighter,
    config: HighlightConfiguration,
}

impl EditorPane {
    fn new(cx: &mut Context<Self>) -> Self {
        let mut highlighter = Highlighter::new();
        let language = tree_sitter_dsl::language();

        let mut config = HighlightConfiguration::new(
            language,
            tree_sitter_dsl::HIGHLIGHT_QUERY,
            "",
            "",
        ).unwrap();

        Self {
            content: String::new(),
            highlighter,
            config,
        }
    }

    fn highlight_content(&self) -> Vec<(String, Hsla)> {
        let highlights = self.highlighter
            .highlight(&self.config, self.content.as_bytes(), None, |_| None)
            .unwrap();

        let mut result = Vec::new();
        let mut current_pos = 0;

        for event in highlights {
            match event.unwrap() {
                HighlightEvent::Source { start, end } => {
                    let text = &self.content[start..end];
                    result.push((text.to_string(), rgb(0xcccccc)));
                    current_pos = end;
                }
                HighlightEvent::HighlightStart(highlight) => {
                    // Map highlight to color
                }
                HighlightEvent::HighlightEnd => {
                    // Reset color
                }
            }
        }

        result
    }
}
```

### Rendering Highlighted Text

```rust
impl Render for EditorPane {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let highlighted = self.highlight_content();

        div()
            .flex()
            .flex_col()
            .font_family("monospace")
            .children(highlighted.into_iter().map(|(text, color)| {
                span()
                    .text_color(color)
                    .child(text)
            }))
    }
}
```

### Color Mapping

Map Tree-sitter highlight types to colors:

```rust
fn highlight_to_color(highlight: &str) -> Hsla {
    match highlight {
        "keyword" => rgb(0xc586c0),
        "function" => rgb(0xdcdcaa),
        "string" => rgb(0xce9178),
        "number" => rgb(0xb5cea8),
        "comment" => rgb(0x6a9955),
        "operator" => rgb(0xd4d4d4),
        "variable" => rgb(0x9cdcfe),
        "type" => rgb(0x4ec9b0),
        _ => rgb(0xd4d4d4),
    }
}
```

### Reusing from TUI

Your existing `crates/dsl-tui/src/ui/highlight.rs` can be adapted for GPUI by changing the output format from Ratatui spans to GPUI styled elements.

---

## 14. Animations and Transitions

### Animation API

GPUI provides built-in animation support through the `Animation` struct.

### Basic Animation

```rust
use gpui::{Animation, AnimationExt};
use std::time::Duration;

Animation::new(Duration::from_secs(2))
    .repeat()
    .with_easing(bounce(ease_in_out))
```

### Applying Animations

```rust
div()
    .with_animation(
        "my_animation",
        Animation::new(Duration::from_millis(200)),
        |element, delta| {
            // delta: 0.0 to 1.0 animation progress
            element.opacity(delta)
        }
    )
```

### Rotation Animation

```rust
div()
    .with_animation(
        "rotation",
        Animation::new(Duration::from_secs(2)).repeat(),
        |element, delta| {
            element.transform(Transformation::rotate(percentage(delta)))
        }
    )
    .child(/* content */)
```

### Fade-in Animation

```rust
// Autocomplete popup with fade-in
div()
    .absolute()
    .with_animation(
        "fade_in",
        Animation::new(Duration::from_millis(200)),
        |element, delta| {
            element.opacity(delta)
        }
    )
    .child(/* popup content */)
```

### Slide Animation

```rust
// Tree node expand with slide
div()
    .overflow_hidden()
    .with_animation(
        "slide_down",
        Animation::new(Duration::from_millis(150)),
        |element, delta| {
            element.h(px(delta * 100.0)) // Slide from 0 to 100px
        }
    )
    .children(/* child nodes */)
```

### Easing Functions

Available easing functions:
- `linear`
- `ease_in`
- `ease_out`
- `ease_in_out`
- `bounce`
- Custom cubic bezier curves

```rust
Animation::new(Duration::from_millis(300))
    .with_easing(bounce(ease_in_out))
```

### Transform Types

```rust
// Rotation
Transformation::rotate(angle)

// Scale
Transformation::scale(scale_factor)

// Translate
Transformation::translate(x, y)
```

### Animation Example from Zed

See `crates/gpui/examples/animation.rs` in the Zed repository for a complete working example with SVG rotation.

---

## 14.5. Theme System (Adabraka UI)

### Adabraka UI Theme System

Adabraka UI includes a complete theme system with semantic color tokens and light/dark mode support.

### Installing Themes

```rust
use adabraka_ui::prelude::*;

fn main() {
    Application::new().run(|cx: &mut App| {
        adabraka_ui::init(cx);

        // Install dark theme
        install_theme(cx, Theme::dark());

        // Or install light theme
        // install_theme(cx, Theme::light());

        // ... rest of setup
    });
}
```

### Available Themes

- `Theme::dark()` - Dark theme with proper contrast
- `Theme::light()` - Clean light theme

### Semantic Color Tokens

Themes provide semantic color tokens that you can use throughout your application:

- **Background colors**: Primary, secondary, tertiary backgrounds
- **Text colors**: Primary, secondary, muted text
- **Interactive colors**:
  - Primary (main actions)
  - Secondary (alternative actions)
  - Accent (highlights)
  - Destructive (dangerous actions, errors)
- **UI elements**: Borders, shadows, overlays
- **Spacing and sizing**: Consistent spacing tokens

### Using Theme Colors

```rust
use adabraka_ui::prelude::*;

// Components automatically use theme colors
Button::new("themed-button", "Click Me!")
    .variant(ButtonVariant::Primary) // Uses theme's primary color

// Access theme colors directly
div()
    .bg(theme.bg_primary())
    .text_color(theme.text_primary())
    .border_color(theme.border_default())
```

### Theme Switching

```rust
struct MyApp {
    current_theme: ThemeMode,
}

impl MyApp {
    fn toggle_theme(&mut self, cx: &mut Context<Self>) {
        self.current_theme = match self.current_theme {
            ThemeMode::Dark => {
                install_theme(cx, Theme::light());
                ThemeMode::Light
            }
            ThemeMode::Light => {
                install_theme(cx, Theme::dark());
                ThemeMode::Dark
            }
        };
        cx.notify();
    }
}
```

### Philosophy

Adabraka UI components ship with sensible defaults that respect the active theme, but you can completely override any styling using the `Styled` trait.

---

## 15. Image Rendering

### Image Support in GPUI

GPUI provides native image rendering support with various image formats.

### Loading Images

```rust
use gpui::*;
use image::DynamicImage;
use std::sync::Arc;

struct ImageRenderer {
    path: String,
    data: Option<Arc<DynamicImage>>,
}

impl ImageRenderer {
    fn new(path: String) -> Self {
        let data = image::open(&path).ok().map(Arc::new);
        Self { path, data }
    }
}
```

### Rendering Images

```rust
impl ImageRenderer {
    fn render(&self) -> impl IntoElement {
        if let Some(img_data) = &self.data {
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
            div().child(format!("[Image not found: {}]", self.path))
        }
    }
}
```

### SVG Rendering

GPUI has SVG support built-in:

```rust
svg()
    .path("path/to/icon.svg")
    .size(px(24.))
    .text_color(rgb(0xffffff))
```

### Image Sizing

```rust
img()
    .w(px(400.))           // Fixed width
    .h(px(300.))           // Fixed height
    .max_w(px(800.))       // Maximum width
    .max_h(px(600.))       // Maximum height
    .object_fit(ObjectFit::Contain)  // Fit behavior
```

### Object Fit Options

- `ObjectFit::Contain` - Maintain aspect ratio, fit within bounds
- `ObjectFit::Cover` - Cover entire area, may crop
- `ObjectFit::Fill` - Stretch to fill
- `ObjectFit::None` - Original size
- `ObjectFit::ScaleDown` - Scale down if larger

### Dynamic Image Loading

```rust
struct ImageView {
    images: Vec<String>,
    loaded_images: HashMap<String, Arc<DynamicImage>>,
}

impl ImageView {
    fn load_image(&mut self, path: &str, cx: &mut Context<Self>) {
        let path = path.to_string();

        cx.spawn(|this, mut cx| async move {
            if let Ok(img) = image::open(&path) {
                this.update(&mut cx, |this, cx| {
                    this.loaded_images.insert(path, Arc::new(img));
                    cx.notify();
                }).ok();
            }
        }).detach();
    }
}
```

### Clipboard Image Support

GPUI supports copying images to/from clipboard on all platforms (including X11).

---

## 16. Code Examples

### Complete Hello World

```rust
use gpui::{
    div, prelude::*, px, rgb, size, App, Application,
    Bounds, Context, SharedString, Window,
    WindowBounds, WindowOptions,
};

struct HelloWorld {
    text: SharedString,
}

impl HelloWorld {
    fn new() -> Self {
        Self {
            text: "GPUI World".into(),
        }
    }
}

impl Render for HelloWorld {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .bg(rgb(0x505050))
            .size(px(500.0))
            .justify_center()
            .items_center()
            .shadow_lg()
            .border_1()
            .border_color(rgb(0x0000ff))
            .text_xl()
            .text_color(rgb(0xffffff))
            .child(format!("Hello, {}!", &self.text))
    }
}

fn main() {
    Application::new().run(move |cx: &mut App| {
        cx.open_window(
            WindowOptions {
                bounds: WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(500.0), px(500.0)),
                    cx,
                )),
                ..Default::default()
            },
            |window, cx| cx.new_view(|cx| HelloWorld::new()),
        );
    });
}
```

### Counter with Actions

```rust
use gpui::*;

actions!(counter, [Increment, Decrement, Reset]);

struct Counter {
    count: i32,
    focus_handle: FocusHandle,
}

impl Counter {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            count: 0,
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Focusable for Counter {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Counter {
    fn increment(&mut self, _: &Increment, _window: &mut Window, cx: &mut Context<Self>) {
        self.count += 1;
        cx.notify();
    }

    fn decrement(&mut self, _: &Decrement, _window: &mut Window, cx: &mut Context<Self>) {
        self.count -= 1;
        cx.notify();
    }

    fn reset(&mut self, _: &Reset, _window: &mut Window, cx: &mut Context<Self>) {
        self.count = 0;
        cx.notify();
    }
}

impl Render for Counter {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_4()
            .size_full()
            .justify_center()
            .items_center()
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::increment))
            .on_action(cx.listener(Self::decrement))
            .on_action(cx.listener(Self::reset))
            .child(
                div()
                    .text_2xl()
                    .child(format!("Count: {}", self.count))
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        div()
                            .px(px(12.))
                            .py(px(6.))
                            .bg(rgb(0x007acc))
                            .child("-")
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.decrement(&Decrement, _window, cx);
                            }))
                    )
                    .child(
                        div()
                            .px(px(12.))
                            .py(px(6.))
                            .bg(rgb(0x007acc))
                            .child("+")
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.increment(&Increment, _window, cx);
                            }))
                    )
            )
    }
}

fn main() {
    Application::new().run(move |cx: &mut App| {
        cx.bind_keys([
            KeyBinding::new("space", Increment, None),
            KeyBinding::new("up", Increment, None),
            KeyBinding::new("down", Decrement, None),
            KeyBinding::new("r", Reset, None),
        ]);

        cx.open_window(WindowOptions::default(), |window, cx| {
            cx.new_view(|cx| Counter::new(cx))
        });
    });
}
```

---

## 17. Learning Resources

### Official Documentation

1. **GPUI Website**: https://www.gpui.rs/
   - Getting started guide
   - Code examples

2. **Zed GPUI README**: https://github.com/zed-industries/zed/blob/main/crates/gpui/README.md
   - Architecture overview
   - Core concepts
   - Best practices

3. **GPUI Context Documentation**: https://github.com/zed-industries/zed/blob/main/crates/gpui/docs/contexts.md
   - Detailed context type explanations
   - Usage patterns

### Tutorials and Guides

1. **GPUI Hello World Tutorial**: https://blog.0xshadow.dev/posts/learning-gpui/gpui-hello-world-tutorial/
   - Step-by-step beginner guide
   - Complete code examples

2. **GPUI Interactivity Tutorial**: https://blog.0xshadow.dev/posts/learning-gpui/gpui-interactivity/
   - Counter app example
   - Event handling patterns
   - Focus system

3. **GPUI Book**: https://matinaniss.github.io/gpui-book/getting-started/
   - Comprehensive guide
   - Manual setup instructions

4. **GPUI Tutorial Repository**: https://github.com/hedge-ops/gpui-tutorial
   - Beginner-friendly tutorial
   - Framework basics

### Component Libraries

1. **Adabraka UI**: https://github.com/Augani/adabraka-ui
   - 73+ polished, accessible components
   - Inspired by shadcn/ui
   - Documentation: https://augani.github.io/adabraka-ui/
   - Examples in repository
   - Complete theme system with light/dark modes
   - Professional animations and accessibility features
   - License: MIT

2. **GPUI Component (Alternative)**: https://github.com/longbridge/gpui-component
   - Another component library option
   - 60+ components
   - Documentation: https://longbridge.github.io/gpui-component/
   - Note: Different from Adabraka UI, choose based on your needs

3. **Awesome GPUI**: https://github.com/zed-industries/awesome-gpui
   - Community projects
   - Example applications
   - Curated list of GPUI resources

### Zed Source Code

**Essential files to study**:

1. **Editor**: `crates/editor/src/editor.rs`
   - Text editing implementation
   - Multi-line input handling
   - Selection management

2. **Completions**: `crates/editor/src/completions.rs`
   - Autocomplete implementation
   - LSP integration

3. **Workspace**: `crates/workspace/src/workspace.rs`
   - Window layout
   - Keyboard shortcuts
   - Pane management

4. **Theme**: `crates/theme/src/`
   - Theme system implementation
   - Color management

5. **GPUI Examples**: `crates/gpui/examples/`
   - `animation.rs` - Animation examples
   - `image.rs` - Image rendering
   - `text.rs` - Text handling
   - `window.rs` - Window management

### API Documentation

1. **GPUI docs.rs**: https://docs.rs/gpui/latest/gpui/
   - Full GPUI API reference
   - Type documentation
   - Core framework docs

2. **Adabraka UI docs.rs**: https://docs.rs/adabraka-ui/
   - Component library API
   - Complete component reference
   - Styling trait documentation

3. **Adabraka UI Web Docs**: https://augani.github.io/adabraka-ui/
   - Component examples
   - Usage guides
   - Visual component gallery

### Community Resources

1. **Zed Discord**: Join for questions and discussions
   - Active community
   - Direct help from GPUI developers

2. **GitHub Discussions**: https://github.com/zed-industries/zed/discussions
   - Feature discussions
   - Implementation help

### Additional Projects

1. **Loungy**: Application launcher built with GPUI
   - Real-world example

2. **gpui-plot**: Plotting library for GPUI
   - Graphics and visualization

3. **gpui-todos**: Todo app example
   - State management patterns

### Blog Posts

1. **GPUI 2 is now in production**: https://zed.dev/blog/gpui-2-on-preview
   - Latest version features

2. **Ownership and data flow in GPUI**: https://zed.dev/blog/gpui-ownership
   - Deep dive into state management
   - Reactivity patterns

3. **Leveraging Rust and the GPU to render user interfaces at 120 FPS**: https://zed.dev/blog/videogame
   - Performance deep dive
   - Technical architecture

4. **GPUI Technical Overview**: https://beckmoulton.medium.com/gpui-a-technical-overview-of-the-high-performance-rust-ui-framework-powering-zed-ac65975cda9f
   - Comprehensive overview
   - Architecture analysis

### Search Strategy

When searching for GPUI information:

1. **Conceptual**: "GPUI [feature] explained"
2. **API**: "GPUI [feature] API reference"
3. **Examples**: "GPUI [feature] example"
4. **Zed source**: "site:github.com/zed-industries/zed [feature]"

### Best Learning Path

**Week 1: Fundamentals**
- Read GPUI documentation cover-to-cover
- Complete Hello World tutorial
- Study Zed's main application structure
- Build 3-5 simple examples

**Week 2-3: Core Features**
- Text input examples
- Layout system practice
- Event handling patterns
- Build a counter or calculator app

**Week 4-5: Advanced Topics**
- Editor implementation study
- State management patterns
- Tree-sitter integration
- Build a text editor prototype

**Week 6-7: Polish**
- Theme system
- Animations
- Performance optimization
- Build a complete application

---

## Appendix: Quick Reference

### Common Method Chains

```rust
// Container with flexbox
div().flex().flex_col().gap_4()

// Full size container
div().size_full()

// Centered content
div().flex().justify_center().items_center()

// Styled box
div().bg(rgb(0x2e2e2e)).border_1().shadow_lg()

// Text styling
div().text_xl().text_color(rgb(0xffffff))

// Interactive element
div().on_click(cx.listener(handler)).hover(|s| s.bg(rgb(0x3e3e3e)))

// Scrollable
div().overflow_y_scroll()

// Positioned
div().absolute().top(px(10.)).left(px(20.))
```

### Common Patterns

**State Update**:
```rust
self.value = new_value;
cx.notify();
```

**Entity Update**:
```rust
entity.update(cx, |entity, cx| {
    entity.value = new_value;
    cx.notify();
});
```

**Async Spawn**:
```rust
cx.spawn(|this, mut cx| async move {
    // async work
    this.update(&mut cx, |this, cx| {
        // update state
        cx.notify();
    }).ok();
}).detach();
```

**Observation**:
```rust
cx.observe(&entity, |this, entity, cx| {
    // react to changes
    cx.notify();
})
```

---

## Conclusion

This guide provides comprehensive coverage of GPUI concepts and patterns needed for the Ratatui to GPUI migration. Refer to specific sections as you implement each phase of the migration plan.

For the latest information, always check:
- The official Zed repository
- The GPUI documentation
- The Zed Discord community

**Remember**: GPUI is pre-1.0 and evolving. The Zed source code is your best reference for current best practices.

---

**Document End**
