# Color Scheme & UI Improvements

## Overview

Implemented a comprehensive theme system with rich data type visualization for the DSL egui REPL interface.

## New Theme System

### Created `theme.rs` Module

A structured color palette system with four main categories:

1. **SyntaxColors** - Code editor/input highlighting
   - Keywords (vibrant purple)
   - Types (teal, sky blue)  
   - Functions (pale yellow/gold)
   - Variables (light blue)
   - Strings (warm orange/peach)
   - Numbers (soft green/cyan)
   - Comments (muted green)

2. **DataTypeColors** - Runtime value visualization
   - **Strings**: Warm peach (#E6A078)
   - **Numbers**: Aqua cyan (#78DCC8)
   - **Booleans**: 
     - `true`: Bright green (#8CE68C)
     - `false`: Soft red (#FF7878)
   - **Null**: Muted gray
   - **Object keys**: Sky blue (#82C8FF)
   - **Array indices**: Lavender (#C8A0F0)
   - **Object punctuation** `{}`: Teal (#4EC9B0)
   - **Array punctuation** `[]`: Purple (#C678DD)
   - **Function names**: Gold (#F0DC82)
   - **Special types** (Image, Markdown): Pink (#FF8CC8)

3. **UiColors** - Interface elements
   - Table headers with distinct background
   - Striped rows for readability
   - Tree indentation guides
   - Panel borders and active pane highlights

4. **StatusColors** - Feedback messages
   - Success (green)
   - Error (red) 
   - Warning (yellow/orange)
   - Info (blue)

### Light & Dark Themes

Both `Theme::dark()` and `Theme::light()` implemented with appropriate contrast ratios for accessibility.

## Enhanced Renderers

### 1. Tree Renderer (`tree.rs`)

**Before**: Plain gray text with basic indentation

**After**:
- **Indentation guides**: Subtle vertical lines (`│`) showing hierarchy
- **Color-coded keys**: 
  - Object keys in sky blue
  - Array indices in lavender
- **Type-aware values**:
  - Strings, numbers, booleans with distinct colors
  - Null in muted gray
- **Rich punctuation**:
  - `{...}` in teal for objects
  - `[...]` in purple for arrays
- **Inline comments**: Field/item counts shown in italicized gray
- **Special type icons**: 🖼️ for images, 📄 for markdown

### 2. Table Renderer (`table.rs`)

**Before**: Monochrome cells with basic selection

**After**:
- **Smart cell coloring**: Automatically detects and colors:
  - Numbers in cyan
  - Booleans (true/false) in green/red
  - Null values in gray
  - Strings in peach
  - Collapsed objects/arrays in muted gray
- **Enhanced headers**: 
  - Distinct background color
  - Larger, bolder text
- **Border frame**: Clean 1px border around table
- **Better row height**: Increased to 22px for readability

### 3. Error Renderer (`error.rs`)

**Before**: Basic red background with hardcoded colors

**After**:
- **Theme-aware backgrounds**: Uses `theme.status.error_bg` and `error_border`
- **Semantic colors**:
  - Success colors for "Expected" type
  - Error colors for "Got" type  
  - Warning colors for suggestions and expandable sections
  - Info colors for HTTP URLs, file paths
  - Number colors for line/column info
- **Consistent styling**: 4px corner radius on all frames
- **Better contrast**: All colors come from theme ensuring coherence

## Visual Examples

### Data Type Colors in Action

```javascript
// Tree view example
{
  name: "Alice",           // "name" in sky blue, "Alice" in peach
  age: 30,                 // "age" in sky blue, 30 in cyan
  active: true,            // "active" in sky blue, true in green
  settings: {...}          // braces in teal
}  // 3 fields
```

### Table with Auto-coloring

| name    | age  | active | balance |
|---------|------|--------|---------|
| Alice   | 30   | true   | 1250.50 |
| Bob     | 25   | false  | 890.25  |

- Numbers (30, 25, 1250.50, 890.25) appear in **cyan**
- Booleans (true, false) in **green/red**
- Text in **default color**

### Error Display

```
🔴 TYPE ERROR                                   // Red
Variable 'x' not found

Variable: x                                     // Gold

💡 Suggestions                                  // Yellow/orange
  • Define the variable with 'let' before using it
  • Check for typos in the variable name

📍 Source Location
  File: script.dsl                             // Blue
  Line: 42  Column: 10                         // Cyan numbers
```

## Benefits

1. **Better Readability**: Colors guide the eye to different data types
2. **Faster Comprehension**: Visual patterns emerge (e.g., all numbers are cyan)
3. **Professional Appearance**: Cohesive color palette throughout UI
4. **Accessibility**: Both light and dark themes with good contrast
5. **Consistency**: Centralized theme system ensures uniform styling
6. **Maintainability**: Easy to adjust colors globally by editing theme.rs

## Usage

All renderers automatically use the theme passed from `ReplPane`:

```rust
// In ReplPane
self.theme = Theme::dark();

// Renderers automatically receive theme
renderers::render_table(ui, columns, rows, selected, &self.theme);
renderers::render_tree(ui, root, expanded_paths, &self.theme);
renderers::render_error(ui, err, &self.theme);
```

## Future Enhancements

- [ ] Theme switcher in UI (dark/light toggle)
- [ ] Custom theme editor
- [ ] Persist theme preference
- [ ] Additional color schemes (Solarized, Monokai, etc.)
- [ ] Syntax highlighting integration with theme
- [ ] Color-blind friendly palettes
