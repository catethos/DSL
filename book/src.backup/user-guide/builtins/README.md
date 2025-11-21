# Builtin Functions Overview

DSL provides 50+ builtin functions for common operations. All builtins are dynamically sourced from metadata, ensuring consistency between implementation and autocomplete.

## Function Categories

### [String Operations](./strings.md)
Text manipulation and analysis functions.

- `Upper`, `Lower` - Case conversion
- `Trim` - Whitespace removal
- `Split`, `Join` - String/list conversion
- `Replace` - Pattern replacement
- `Contains`, `StartsWith`, `EndsWith` - String searching
- `Length` - Get string length

### [List Operations](./lists.md)
Functions for working with lists and collections.

- `Reverse`, `Sort`, `Unique` - List transformation
- `Take`, `Skip` - Subsequences
- `First`, `Last` - Element access
- `Flatten` - Nested list flattening
- `Length` - Get list length

### [Math Operations](./math.md)
Mathematical functions and calculations.

- `Abs` - Absolute value
- `Min`, `Max` - Extremes
- `Sum`, `Average` - Aggregation
- `Round`, `Floor`, `Ceil` - Rounding
- `ToInt`, `ToFloat` - Type conversion

### [Functional Programming](./functional.md)
Higher-order functions that accept user-defined functions.

- `Map` - Transform each element
- `Filter` - Keep matching elements
- `Reduce` - Accumulate result
- `Any`, `All` - Boolean predicates
- `Find` - First matching element
- `Count` - Count matching elements

### [Utilities](./utilities.md)
Miscellaneous utility functions.

- `Zip` - Combine two lists
- `Range` - Generate integer sequences
- `Repeat` - Create repeated values
- `Chunk` - Split into chunks
- `ToString` - Convert to string
- `RenderMarkdown` - Format markdown
- `Par` - Parallel execution
- `Not` - Logical negation

## Special Functions

### SQL Functions
- `SQL(query)` - Execute SQL with DuckDB
- `refresh_table(name)` - Clear table cache

### LLM Functions
- `Ask(prompt)` - Simple LLM query
- `ExtractPerson(text)` - Structured person extraction
- `ExtractAs(text, type)` - Generic structured extraction

### Chart Generation
- `GenerateBarChart(data, theme?)` - Bar charts
- `GenerateLineChart(data, theme?)` - Line charts
- `GeneratePieChart(data, theme?)` - Pie charts

## Function Properties

### Case Insensitive
Function names are case-insensitive:
```dsl
Upper("hello")  // Works
upper("hello")  // Also works
UPPER("hello")  // Also works
```

### Dynamic Metadata
All builtins are registered with metadata including:
- Function name
- Parameter types
- Return type
- Description
- Examples

This metadata powers autocomplete and documentation.

### Error Handling
- Type errors caught at runtime
- Clear error messages
- Empty list operations raise errors

## Common Patterns

### Data Processing Pipeline
```dsl
let numbers = Range(1, 11)  // [1..10]
let evens = Filter(numbers, def (x) := x % 2 == 0)
let doubled = Map(evens, def (x) := x * 2)
let sum = Sum(doubled)
```

### Text Processing
```dsl
let text = "  hello,world,foo  "
let cleaned = Trim(text)
let words = Split(cleaned, ",")
let upper = Map(words, Upper)
let result = Join(upper, " | ")
```

### Data Analysis
```dsl
let scores = [85, 92, 78, 95, 88, 76, 91]

let stats = {
  min: Min(scores),
  max: Max(scores),
  avg: Average(scores),
  count: Length(scores),
  passing: Count(scores, def (x) := x >= 80)
}
```

## Next Steps

- [String Operations](./strings.md) - Learn text manipulation
- [List Operations](./lists.md) - Work with collections
- [Math Operations](./math.md) - Perform calculations
- [Functional Programming](./functional.md) - Use higher-order functions
- [Utilities](./utilities.md) - Miscellaneous tools
