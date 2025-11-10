# DSL Editor Formatting Features

This document describes the code formatting features available in the DSL egui editor.

## Features Implemented

### 1. **Live Syntax Highlighting** ✓

Syntax highlighting is applied **directly in the editor** as you type - no separate preview pane needed!

- Keywords: `def`, `type`, `enum`, `let`, `match`, etc.
- Types: Custom types, built-in types
- Functions: Function names and calls
- Strings: String literals and template strings
- Numbers: Integers and floats
- Comments: Line and block comments
- Operators and punctuation

**Color schemes:**
- Dark mode (default)
- Light mode

### 2. **Auto-Formatting** ✓

Format your entire document with proper indentation and spacing.

**Keyboard shortcut:** `Cmd+Shift+F` (Mac) or `Ctrl+Shift+F` (Windows/Linux)

**What it does:**
- Adds proper indentation inside `{}`, `[]`, `()` blocks
- Normalizes spacing around operators
- Preserves string literals and comments
- Handles nested structures correctly

**Example:**

Before formatting:
```dsl
def unformatted(x){let y=x*2
let z=y+1
z}
```

After formatting (`Cmd+Shift+F`):
```dsl
def unformatted(x){
  let y=x*2
  let z=y+1
  z
}
```

### 3. **Code Editor Mode** ✓

The editor uses egui's `.code_editor()` mode which provides:
- Tab key inserts spaces (configurable)
- Line numbers (optional)
- Better text selection behavior
- Code-optimized scrolling

### 4. **Bracket Matching** ✓

The formatter includes logic to find matching brackets/braces/parentheses.

**Supported pairs:**
- `{` and `}`
- `[` and `]`
- `(` and `)`

## Configuration

The formatter can be configured via `FormatterConfig`:

```rust
pub struct FormatterConfig {
    pub indent_size: usize,      // Default: 2 spaces
    pub use_spaces: bool,         // Default: true (use spaces, not tabs)
    pub auto_close_brackets: bool, // Default: true
}
```

### Changing Indent Size

To use 4 spaces instead of 2:

```rust
editor.formatter_config_mut().indent_size = 4;
```

### Using Tabs Instead of Spaces

```rust
editor.formatter_config_mut().use_spaces = false;
```

## Formatting Rules

The formatter follows these rules based on the DSL grammar:

### Indentation

1. **Function bodies:** Indent after `def name(params) {`
   ```dsl
   def example(x) {
     let y = x * 2  # Indented
     y
   }
   ```

2. **Type definitions:** Indent fields inside type bodies
   ```dsl
   type Person {
     name: String,
     age: Int
   }
   ```

3. **Blocks:** Indent after any `{`
   ```dsl
   let result = {
     let x = 10
     let y = 20
     x + y
   }
   ```

4. **Match expressions:** Indent cases
   ```dsl
   match value {
     0 => "zero",
     n => "other"
   }
   ```

5. **Nested structures:** Each level adds one indent
   ```dsl
   def nested() {
     let outer = {
       inner: {
         value: 42
       }
     }
     outer
   }
   ```

### String Handling

- String literals are preserved exactly as written
- Template strings with `${}` are preserved
- Triple-quoted strings `"""..."""` are preserved
- Escape sequences are maintained

### Comments

- Line comments `#` are preserved
- Block comments `/* */` are preserved
- Comments don't affect indentation

## Implementation Details

### Modules

- **`formatter.rs`**: Core formatting logic
  - `calculate_indent_level()`: Determines indent depth
  - `format_document()`: Formats entire file
  - `find_matching_bracket()`: Finds bracket pairs
  - `handle_tab()`: Tab key behavior
  - `handle_untab()`: Shift+Tab behavior
  - `handle_enter()`: Smart newline with auto-indent
  - `handle_open_bracket()`: Auto-close brackets

- **`editor.rs`**: Integration with egui
  - Uses `.layouter()` for syntax highlighting
  - Keyboard shortcut handling
  - Format on demand

### Testing

Run tests with:
```bash
cargo test -p dsl-egui formatter
```

Test coverage includes:
- Indent level calculation
- Bracket matching
- Document formatting
- Nested structures

## Future Enhancements

Potential features to add:

1. **Format on Save** - Auto-format when saving file
2. **Format on Paste** - Auto-format pasted code
3. **Smart Enter** - Auto-indent new lines based on context
4. **Auto-close Brackets** - Automatically insert closing `}`, `]`, `)`
5. **Alignment** - Align `=`, `:`, `=>` in consecutive lines
6. **Trailing Comma** - Auto-add/remove trailing commas
7. **Line Wrapping** - Wrap long lines intelligently
8. **Import Sorting** - Sort type/function declarations
9. **Custom Rules** - User-configurable formatting rules

## Examples

See `examples/formatting_test.dsl` for examples of code before and after formatting.

## Troubleshooting

**Q: Formatting doesn't work**
- Make sure you're pressing `Cmd+Shift+F` (Mac) or `Ctrl+Shift+F` (Windows/Linux)
- Check the status bar for "Document formatted" message

**Q: Indentation is wrong**
- The formatter uses brace counting; ensure braces are balanced
- Strings containing braces won't confuse the formatter (they're handled correctly)

**Q: Want different indent size**
- Modify `formatter_config.indent_size` (default is 2)

**Q: Prefer tabs over spaces**
- Set `formatter_config.use_spaces = false`
