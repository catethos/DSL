# Markdown Rendering Guide

## Overview

The DSL TUI now supports rich markdown rendering with word wrapping, syntax highlighting, and aesthetic styling.

## Features

### 1. Rich Header Styling
Headers are rendered with different colors, visual prefixes, and text modifiers:

- **H1**: Cyan color, bold + underlined, `█` prefix
- **H2**: Blue color, bold, `▓` prefix
- **H3**: Magenta color, bold, `▒` prefix
- **H4-H6**: Yellow color, bold, `░` prefix

All headers automatically add a blank line after them for better readability.

### 2. Text Formatting

- **Bold text**: Rendered in yellow with bold modifier
- *Italic text*: Rendered with italic modifier
- `Inline code`: Green color with bold, surrounded by backticks

### 3. Code Blocks

Code blocks are rendered with:
- Box-drawing borders (┌─┐, │, └─┘) in dark gray
- Green colored code text
- Preserved indentation and line breaks
- `│` prefix on each line

Example:
```
┌──────────────────┐
│ fn hello() {     
│     println!("world");
│ }                
└──────────────────┘
```

### 4. Lists

- Rendered with cyan `•` bullet points
- Proper indentation for nested lists
- Automatic spacing after list completion

### 5. Word Wrapping

All text content (except code blocks) is automatically wrapped to fit the terminal width:
- Respects word boundaries
- Preserves styling across wrapped lines
- Handles long words by breaking them
- Maintains proper indentation for list items

### 6. Horizontal Rules

Rendered as full-width separator lines in dark gray.

## Usage

### Basic Usage

```rust
flow> RenderMarkdown("# Hello World\n\nThis is **bold** text")
```

### With AI Generation

```rust
flow> RenderMarkdown(Ask("give me a sample article in markdown format"))
```

### Example Output

When you run:
```
flow> RenderMarkdown(Ask("explain markdown in 3 paragraphs"))
```

You'll see:
- Styled headers in different colors
- Bold and italic text properly rendered
- Word-wrapped paragraphs
- Beautiful visual hierarchy

## Implementation Details

### Architecture

1. **Parser**: Uses `pulldown-cmark` for markdown parsing
2. **Renderer**: Custom renderer in `src/renderers/markdown.rs`
3. **Word Wrapping**: Leverages existing `wrap_text()` function from `text.rs`
4. **Styling**: Ratatui's `Style`, `Line`, and `Span` for rich formatting

### Key Components

- `markdown_to_lines()`: Main rendering function
- `flush_text_spans()`: Helper for text wrapping and line building
- Event-driven parser that accumulates text and applies styles

### Color Palette

- Headers: Cyan (H1), Blue (H2), Magenta (H3), Yellow (H4+)
- Bold text: Yellow
- Inline code: Green
- Code blocks: Green text, dark gray borders
- Bullets: Cyan
- Rules: Dark gray

## Tips

1. **Long Content**: The renderer automatically wraps text to fit your terminal width
2. **Mixed Formatting**: You can combine bold, italic, and code in the same paragraph
3. **Code Blocks**: Use triple backticks for multi-line code
4. **Lists**: Supports both ordered and unordered lists with proper nesting

## Limitations

- Syntax highlighting within code blocks uses a simple green color (not language-specific)
- Tables are not yet fully styled (displayed as-is from parser)
- Images are ignored (text-only terminal)
- Links display the link text only

## Future Enhancements

- Language-specific syntax highlighting in code blocks
- Table rendering with borders
- Configurable color schemes
- Custom styles for different heading levels
