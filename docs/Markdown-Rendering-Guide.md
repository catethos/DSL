# Markdown Rendering Guide

## Overview

The DSL GUI (egui) uses `egui_commonmark` for rich markdown rendering with full CommonMark support plus GitHub-flavored extensions.

## Migration from Custom Renderer

**Previous Implementation (TUI):**
- Custom renderer using `pulldown-cmark` directly
- ~300 lines of manual rendering code
- Limited features (headings, bold, italic, code blocks, lists)

**Current Implementation (egui):**
- Uses `egui_commonmark` library (v0.20)
- ~50 lines of integration code
- Full CommonMark + GitHub extensions support

## Features

### 1. Standard Markdown

- **Headings** (H1-H6) with styled text and sizing
- **Paragraphs** with automatic text wrapping
- **Bold** and *italic* text formatting
- `Inline code` with monospace font
- Code blocks with optional syntax highlighting
- Horizontal rules (separators)

### 2. Lists

- Unordered lists with bullet points
- Ordered lists with numbers
- Nested lists
- Task lists with checkboxes `- [ ]` and `- [x]`

### 3. Extended Features (GitHub Flavored Markdown)

- **Tables** - Automatic column alignment and borders
- **Strikethrough** - ~~crossed out text~~
- **Footnotes** - Reference-style notes
- **Task Lists** - Interactive checkboxes

### 4. Links and Images

- **Clickable links** with URL tooltips
- **Images** - Embedded (base64) and remote (URLs)
- Supported formats: PNG, JPEG, WebP, SVG

### 5. Advanced Formatting

- Blockquotes with left border and background
- Nested blockquotes
- Mixed inline formatting (bold + italic + code)

## Usage

### Basic Usage

```rust
// In DSL code
RenderMarkdown("# Hello World\n\nThis is **bold** text")
```

### With AI Generation

```rust
RenderMarkdown(Ask("give me a sample article in markdown format"))
```

### Tables

```markdown
| Feature | Supported |
|---------|-----------|
| Tables  | ✅        |
| Images  | ✅        |
| Links   | ✅        |
```

### Code Blocks with Syntax Highlighting

````markdown
```rust
fn main() {
    println!("Hello, world!");
}
```
````

### Task Lists

```markdown
- [x] Migrate to egui_commonmark
- [x] Fix version compatibility
- [ ] Add custom themes
```

## Implementation Details

### Architecture

1. **Library**: Uses `egui_commonmark` (v0.20 for egui 0.31)
2. **Cache**: `CommonMarkCache` for efficient re-rendering
3. **Integration**: Simple wrapper in `src/renderers/markdown.rs`

### Key Components

**`markdown.rs` (crates/dsl-egui/src/renderers/markdown.rs):**

```rust
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

pub fn render_markdown(ui: &mut egui::Ui, cache: &mut CommonMarkCache, markdown: &str) {
    CommonMarkViewer::new().show(ui, cache, markdown);
}
```

**Integration in ReplPane:**

```rust
pub struct ReplPane {
    // ... other fields
    markdown_cache: CommonMarkCache,
}

// Usage
OutputItem::Markdown(md) => {
    renderers::render_markdown(ui, &mut self.markdown_cache, md);
}
```

### Dependencies

```toml
[dependencies]
egui_commonmark = "0.20"  # Compatible with egui 0.31
```

**Version Compatibility:**
- egui 0.31 → `egui_commonmark = "0.20"`
- egui 0.32 → `egui_commonmark = "0.21"`
- egui 0.33 → `egui_commonmark = "0.22"`

See [egui Migration Guide](08-egui-Migration-Guide.md#version-compatibility-notes) for details.

## Styling

### Default Styles

`egui_commonmark` uses egui's default styling which includes:

- **Headings**: Larger font sizes, bold text, styled colors
- **Code blocks**: Dark background, monospace font
- **Links**: Underlined, blue color, clickable
- **Blockquotes**: Left border, subtle background
- **Tables**: Grid layout with borders

### Customization

The renderer respects egui's global style settings. To customize:

```rust
// In your app.rs
ctx.set_visuals(egui::Visuals::dark()); // or light()

// Or customize specific colors
let mut style = (*ctx.style()).clone();
style.visuals.hyperlink_color = egui::Color32::from_rgb(100, 200, 255);
ctx.set_style(style);
```

## Features Comparison

| Feature | TUI (Old) | egui (New) |
|---------|-----------|------------|
| Headings | ✅ | ✅ |
| Bold/Italic | ✅ | ✅ |
| Code Blocks | ✅ | ✅ (with syntax highlighting) |
| Lists | ✅ | ✅ (+ nested) |
| Tables | ❌ | ✅ |
| Links | Text only | ✅ Clickable |
| Images | ❌ | ✅ |
| Strikethrough | ❌ | ✅ |
| Task Lists | ❌ | ✅ |
| Footnotes | ❌ | ✅ |
| Code Size | ~300 lines | ~50 lines |

## Performance

- **Caching**: `CommonMarkCache` caches parsed markdown for fast re-renders
- **Lazy Rendering**: Only visible content is rendered (egui handles this)
- **Memory**: Cache is stored per-REPL instance, cleared on restart

## Tips

1. **Long Documents**: The renderer automatically handles scrolling and text wrapping
2. **Mixed Formatting**: Combine bold, italic, and code freely
3. **Tables**: Use pipe `|` syntax for clean table formatting
4. **Images**: Use relative paths or URLs for images
5. **Syntax Highlighting**: Enable via `syntect` feature (optional)

## Limitations

- **Heading Size Hierarchy**: Version 0.20 may render all heading levels (H1-H6) with the same font size
  - This is a limitation of egui_commonmark 0.20
  - All headings use `TextStyle::Heading` which has a single size
  - **Workaround**: Upgrade to egui 0.33 + egui_commonmark 0.22 for better heading hierarchy
  - **Current behavior**: All headings are styled/bold but appear the same size
- **Syntax Highlighting**: Requires additional feature flag (not enabled by default)
- **Custom Themes**: Limited to egui's style system
- **Math**: LaTeX/MathJax not supported
- **Mermaid Diagrams**: Not supported

## Future Enhancements

- [ ] Optional syntax highlighting for code blocks
- [ ] Custom theme support
- [ ] Export markdown to HTML/PDF
- [ ] Live preview for editor
- [ ] Markdown toolbar for editor

## Troubleshooting

### Markdown Not Rendering

**Issue**: Markdown displays as plain text

**Solution**: Ensure the value is wrapped in `OutputItem::Markdown()`:
```rust
// Correct
OutputItem::Markdown(markdown_string)

// Wrong - will display as plain text
OutputItem::Text(markdown_string)
```

### Version Conflicts

**Issue**: Compilation errors about `egui` type mismatches

**Solution**: Ensure `egui_commonmark` version matches your `egui` version:
- egui 0.31 → egui_commonmark 0.20
- See [Version Compatibility Notes](08-egui-Migration-Guide.md#version-compatibility-notes)

### Images Not Loading

**Issue**: Image markdown syntax doesn't display images

**Solution**:
1. Check image path is correct (relative to working directory)
2. Ensure image format is supported (PNG, JPEG, WebP, SVG)
3. For remote images, check network connectivity

## References

- [egui_commonmark Documentation](https://docs.rs/egui_commonmark)
- [CommonMark Specification](https://commonmark.org/)
- [GitHub Flavored Markdown](https://github.github.com/gfm/)
- [egui Migration Guide](08-egui-Migration-Guide.md)
