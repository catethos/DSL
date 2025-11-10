# DSL egui - Desktop GUI with egui

This crate contains the egui-based desktop UI for the DSL interpreter, replacing the terminal-based TUI (Ratatui) with an immediate-mode GUI framework.

## Why egui?

- **Mature**: Well-documented with extensive examples
- **Easy to use**: Immediate-mode API is intuitive
- **Batteries included**: Built-in widgets (TextEdit, ScrollArea, Table, etc.)
- **Cross-platform**: Runs on desktop and web (via WASM)
- **Performance**: Efficient rendering and updates
- **Rich ecosystem**: Many community extensions

## Building

```bash
cargo build -p dsl-egui
```

## Running

```bash
cargo run -p dsl-egui
```

## Architecture

```
crates/dsl-egui/
├── src/
│   ├── main.rs           # egui application entry
│   ├── lib.rs            # Library interface
│   ├── app.rs            # Main application state
│   ├── repl.rs           # REPL pane component
│   ├── editor.rs         # Editor pane component
│   ├── output_item.rs    # Output types (reuse from TUI)
│   ├── autocomplete.rs   # Autocomplete popup
│   └── renderers/        # Output renderers
│       ├── mod.rs
│       ├── text.rs
│       ├── table.rs
│       ├── tree.rs
│       ├── markdown.rs
│       └── image.rs
├── Cargo.toml
└── README.md
```

## Features

- **Workspace Layout**: Resizable split between REPL and Editor
- **REPL**: Multi-line input, history, autocomplete
- **Editor**: Syntax highlighting, file operations
- **Output**: Text, Tables, Trees, Markdown, Images
- **Interactive**: Mouse and keyboard support
- **Clipboard**: Copy/paste functionality

## Migration Status

- [x] Create crate structure
- [x] Set up dependencies
- [ ] Basic egui application
- [ ] Workspace layout
- [ ] REPL input
- [ ] Output display
- [ ] Output renderers
- [ ] Editor pane
- [ ] Autocomplete
- [ ] Full feature parity with TUI
