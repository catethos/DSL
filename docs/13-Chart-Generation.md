# Chart Generation with LLM Integration

This document describes how to generate charts in the DSL using the built-in chart generation functions.

## Overview

The DSL now supports generating chart images using the `plotters` library. Charts are saved as PNG files in the system's temporary directory and can be displayed in terminals that support graphics protocols (via ratatui-image).

## Available Chart Functions

### 1. `generateBarChart(data, theme?)`

Generates a bar chart from structured data.

**Parameters:**
- `data`: List of objects with `{label: String, value: Number}` structure
- `theme` (optional): Color theme string - one of: `"blue"`, `"green"`, `"red"`, `"purple"`, `"orange"`, `"dark"`. Default: `"blue"`

**Returns:** `Image` value containing the path to the generated PNG file

**Example:**
```dsl
let salesData = [
    {label: "Q1", value: 100},
    {label: "Q2", value: 150},
    {label: "Q3", value: 120},
    {label: "Q4", value: 180}
]

// Default blue theme
let chart = generateBarChart(salesData)

// With green theme
let chartGreen = generateBarChart(salesData, "green")
chart
```

### 2. `generateLineChart(data, theme?)`

Generates a line chart from a list of numbers.

**Parameters:**
- `data`: List of numbers
- `theme` (optional): Color theme string - one of: `"blue"`, `"green"`, `"red"`, `"purple"`, `"orange"`, `"dark"`. Default: `"blue"`

**Returns:** `Image` value containing the path to the generated PNG file

**Example:**
```dsl
let temperatures = [20, 22, 25, 23, 21, 19, 18, 20, 24, 26, 25, 23]

// Default blue theme
let chart = generateLineChart(temperatures)

// With red theme
let chartRed = generateLineChart(temperatures, "red")
chart
```

### 3. `generatePieChart(data, theme?)`

Generates a pie chart from structured data.

**Parameters:**
- `data`: List of objects with `{label: String, value: Number}` structure
- `theme` (optional): Color theme string - one of: `"default"`, `"green"`, `"red"`, `"purple"`, `"orange"`, `"dark"`. Default: `"default"`

**Returns:** `Image` value containing the path to the generated PNG file

**Example:**
```dsl
let marketShare = [
    {label: "Product A", value: 35},
    {label: "Product B", value: 25},
    {label: "Product C", value: 20},
    {label: "Product D", value: 15},
    {label: "Other", value: 5}
]

// Default colorful theme
let chart = generatePieChart(marketShare)

// With dark theme
let chartDark = generatePieChart(marketShare, "dark")
chart
```

## Color Themes

All chart functions support color themes via an optional second parameter. This allows you to customize the appearance of your charts.

### Available Themes

| Theme | Description | Background | Primary Color |
|-------|-------------|------------|---------------|
| `"blue"` | Classic blue (default) | White | Blue (#0077B6) |
| `"green"` | Material green | White | Green (#4CAF50) |
| `"red"` | Material red | White | Red (#F44336) |
| `"purple"` | Material purple | White | Purple (#9C27B0) |
| `"orange"` | Material orange | White | Orange (#FF9800) |
| `"dark"` | Dark mode | Dark Gray (#1E1E1E) | Light Blue (#42A5F5) |

### Theme Examples

**Bar Chart with Different Themes:**
```dsl
let data = [{label: "A", value: 10}, {label: "B", value: 20}]

generateBarChart(data, "blue")    // Classic blue bars
generateBarChart(data, "green")   // Green bars
generateBarChart(data, "dark")    // Light blue bars on dark background
```

**Pie Chart with Themed Color Palettes:**
```dsl
let data = [
    {label: "Slice 1", value: 30},
    {label: "Slice 2", value: 25},
    {label: "Slice 3", value: 20},
    {label: "Slice 4", value: 15},
    {label: "Slice 5", value: 10}
]

generatePieChart(data, "purple")  // 8 shades of purple
generatePieChart(data, "orange")  // 8 shades of orange
generatePieChart(data, "dark")    // Vibrant colors on dark background
```

**Line Chart with Themes:**
```dsl
let temps = [18, 20, 22, 24, 23, 21, 19]

generateLineChart(temps, "red")     // Red line and points
generateLineChart(temps, "green")   // Green line and points
generateLineChart(temps, "dark")    // Light blue on dark background
```

## LLM Integration

You can combine LLM functions with chart generation to extract data from text and visualize it.

### Example: Extract and Visualize Sales Data

```dsl
// Define a type for chart data
type ChartData {
    label: String,
    value: Int
}

// Sample text with sales data
let salesText = """
Our company had a great year!
- In Q1, we made $100,000 in revenue
- Q2 was even better with $150,000
- Q3 saw a slight dip to $120,000
- But Q4 was amazing with $180,000 in sales!
"""

// Extract data using LLM (requires OPENAI_API_KEY)
let data = extractAs(salesText, "List<ChartData>")

// Generate bar chart from extracted data
let chart = generateBarChart(data)
chart

// Optional: Get LLM analysis
let analysis = ask("""
Based on this sales data:
Q1: $100k, Q2: $150k, Q3: $120k, Q4: $180k

Provide a brief 2-sentence analysis of the trend.
""")

renderMarkdown(analysis)
```

## Image Display

### Inline Display in TUI

The TUI supports **high-quality image display** using terminal graphics protocols (Kitty, iTerm2, Sixel) via ratatui-image:

**Features:**
- ✅ **Full PNG quality display** in supported terminals (Kitty, iTerm2, WezTerm)
- ✅ Split-pane view: text output (40%) + image (60%)
- ✅ Automatic protocol detection and fallback
- ✅ **Toggle images on/off** with `Ctrl+I` for better performance
- ✅ Cached rendering for smooth interaction

**Controls:**
- `Ctrl+I` - Toggle image display on/off
- When images are hidden, the TUI runs much faster and you can still access the PNG files

**Display Modes:**
1. **Graphics Protocol Mode** (default when supported):
   - Displays actual PNG with full 800x600 resolution
   - Image shown in bottom 60% of screen
   - Text history in top 40%

2. **Text-Only Mode** (press `Ctrl+I` to toggle):
   - Shows only text output
   - Much faster rendering
   - Image files still saved to `/tmp/`

**Note:** For maximum performance when working with multiple charts, toggle images off with `Ctrl+I` and open them externally when needed.

### Viewing Generated Charts

Charts are saved in your system's temporary directory:
- macOS/Linux: `/tmp/chart_*.png`
- Windows: `%TEMP%\chart_*.png`

You can:
1. Copy the path and open it in an image viewer
2. Use the TUI in a graphics-capable terminal for inline display
3. Access the files programmatically in your scripts

## Examples

See the following example files:
- `examples/charts_basic.dsl` - Chart generation without LLM
- `examples/llm_charts.dsl` - LLM + chart generation workflow
- `examples/test_chart.dsl` - Simple test script

## Technical Details

### Chart Specifications

- **Resolution**: 800x600 pixels
- **Format**: PNG
- **Colors**: 6 built-in color themes (blue, green, red, purple, orange, dark)
- **Font**: Sans-serif, various sizes for labels and titles
- **Themes**:
  - Light themes: white background with Material Design colors
  - Dark theme: dark gray background (#1E1E1E) with vibrant accent colors

### Value Type

Charts return a new `Value::Image(String)` type containing the file path to the generated image.

### Dependencies

The chart generation feature requires:
- `plotters` - Chart generation library
- `image` - Image processing
- `ratatui-image` - Terminal image rendering (for TUI)

## Limitations

1. Charts are static PNG images (not interactive)
2. High-quality image display in terminal requires graphics protocol support (Kitty, iTerm2, Sixel)
3. Temporary files are not automatically cleaned up
4. Limited customization options (6 preset themes, fixed sizes and fonts)
5. Image rendering may impact TUI performance (use `Ctrl+I` to toggle off when not needed)

## Future Enhancements

Planned improvements:
- Additional color themes and custom color palettes
- More chart types (scatter plots, histograms, heatmaps, area charts)
- Customizable chart dimensions and font sizes
- Interactive charts in capable terminals
- Export to different formats (SVG, PDF)
- Direct integration with data analysis functions
- Chart legends and annotations
- Multi-series line/bar charts
