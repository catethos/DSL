# Markdown Rendering Demo

This example demonstrates the new markdown rendering capability in the DSL TUI.

## Features Supported

The markdown renderer supports:

- **Bold text**
- *Italic text*
- `Inline code`
- Lists (ordered and unordered)
- Headers (multiple levels)
- Code blocks with syntax highlighting

### Code Example

```rust
fn main() {
    println!("Hello, markdown world!");
}
```

### Lists

1. First item
2. Second item
3. Third item

Unordered list:
- Item A
- Item B
  - Nested item
- Item C

### Inline Formatting

You can combine **bold**, *italic*, and `code` formatting in the same line.

---

## Usage in DSL

To use markdown rendering in your DSL application:

```rust
use dsl_tui::output_item::OutputItem;

let markdown = r#"
# My Title
Some **bold** text
"#;

app.output.push(OutputItem::markdown(markdown));
```

The markdown will be rendered with proper styling and syntax highlighting!
