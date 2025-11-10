# Advanced Features

## Session Management

### Save Session

Save all variables to a file:

```javascript
42 as answer
"hello" as greeting
[1, 2, 3] as numbers

:save my_session.json
```

### Load Session

Restore variables from a file:

```javascript
:load my_session.json

answer      // 42
greeting    // "hello"
numbers     // [1, 2, 3]
```

**Note:** Only variables are saved. Types and functions must be redefined.

## Multi-line Input

### Smart Delimiter Detection

The REPL automatically detects unclosed delimiters:

```javascript
flow> {
...     sql: """
...       SELECT *
...       FROM data
...     """
... }
```

### Manual Multi-line Mode

Use **Shift+Enter** or **Alt+Enter** to continue without executing:

```javascript
flow> "This is a
...> multi-line
...> string"
```

## File Operations

### Save Editor Content

In Workspace mode (F2), save editor content:

- **Ctrl+S** - Save file

```javascript
// In editor
type Person { name: String, age: Int }
def greet(n: String) { prompt: "Greet ${n}" }

// Press Ctrl+S to save
```

### Run Editor in REPL

- **Ctrl+R** - Run entire editor content in REPL
- **Ctrl+E** - Send current line to REPL

## Debugging

### Debug Mode

Toggle debug mode to see internals:

```javascript
:debug

Ask("Hello")
// Shows: Generated prompt, LLM response, parsing details
```

### Function Inspection

List all user-defined functions:

```javascript
:funcs
```

Output shows:
- Function signatures
- Execution type (LLM, HTTP, SQL, Hybrid)
- Configuration

### Variable Inspection

```javascript
:vars
```

Shows all variables with types and values.

### Type Inspection

```javascript
:types
```

Lists all registered types and enums.

Press **F3** for detailed type explorer.

## Output Management

### Copy Results

Save last result to a file:

```javascript
SQL("SELECT * FROM 'data.csv'") as results

:copy output.txt
```

### Word Wrapping

Long output automatically wraps to fit terminal width.

### Scrolling

- **Mouse wheel** - Scroll output
- **Arrow keys** - Navigate

## Template String Features

### Variable Interpolation

```javascript
"Alice" as name
42 as age

"${name} is ${age} years old"
// "Alice is 42 years old"
```

### In Prompts

```javascript
def analyze(topic: String, criteria: [String]) {
  prompt: """
    Analyze ${topic} based on:
    ${criteria}
  """
}
```

### In URLs

```javascript
def getResource(type: String, id: Int) {
  http: "GET"
  url: "https://api.example.com/${type}/${id}"
}
```

### In SQL

```javascript
def filterData(table: Table, threshold: Float) {
  sql: "SELECT * FROM table WHERE score > ${threshold}"
}
```

## Advanced Workflows

### Iterative Refinement

```javascript
Draft(topic) as v1
  >> Review(v1) as feedback
  >> Revise(v1, feedback) as v2
  >> Review(v2) as feedback2
  >> Revise(v2, feedback2) as final
```

### Multi-Source Aggregation

```javascript
(
  FetchSource1(query) ||
  FetchSource2(query) ||
  FetchSource3(query)
) as [data1, data2, data3]
  >> Combine(data1, data2, data3) as combined
  >> Analyze(combined) as insights
```

### Data Enrichment Pipeline

```javascript
SQL("SELECT * FROM 'raw.csv'") as raw
  >> CleanData(raw) as clean
  >> (
       EnrichWithAPI(clean) ||
       EnrichWithLLM(clean) ||
       EnrichWithSQL(clean)
     ) as [api, llm, sql]
  >> MergeEnrichments(api, llm, sql) as enriched
  >> FinalAnalysis(enriched)
```

### Conditional Workflows

```javascript
// Current: Manual branching
Analyze(data) as score

// score > 0.8 case
ProcessHigh(data)

// score <= 0.8 case
ProcessLow(data)
```

**Note:** Automatic conditional operator (`?:`) not yet implemented.

## Performance Optimization

### Parallel Execution

Maximize parallelism for independent tasks:

```javascript
// Slow (sequential): 3 seconds
Task1() >> Task2() >> Task3()  // 1s + 1s + 1s

// Fast (parallel): 1 second
(Task1() || Task2() || Task3())  // max(1s, 1s, 1s)
```

### Batching

Batch similar operations:

```javascript
// Bad: Many small requests
getUser(1) >> getUser(2) >> getUser(3)

// Good: One batch request
def getBatch(ids: [Int]) {
  http: "POST"
  url: "https://api.example.com/users/batch"
  body: { "ids": ${ids} }
}

getBatch([1, 2, 3])
```

### Early Filtering

Filter data early in the pipeline:

```javascript
// Good: Filter first
SQL("SELECT * FROM 'huge.csv' WHERE date > '2024-01-01'") as filtered
  >> AnalyzeData(filtered)

// Bad: Load all then filter
SQL("SELECT * FROM 'huge.csv'") as all
  >> SQL("SELECT * FROM all WHERE date > '2024-01-01'")
  >> AnalyzeData(_)
```

## Error Recovery Patterns

### Validation Before Processing

```javascript
SQL("SELECT * FROM 'data.csv'") as raw
SQL("SELECT COUNT(*) FROM raw WHERE value IS NULL") as nulls

// Check nulls before continuing
// (Manual check - automatic validation not yet implemented)
```

### Fallback Functions

```javascript
def FetchWithFallback(source: String) {
  http: "GET"
  url: "https://primary.api.com/data?src=${source}"
}

// If primary fails, manually try secondary
// (Automatic fallback not yet implemented)
```

## Best Practices

### 1. Incremental Development

Start simple, add complexity:

```javascript
// Step 1: Basic
FetchData() as data

// Step 2: Add filtering
FetchData() >> FilterData(_) as filtered

// Step 3: Add analysis
FetchData() >> FilterData(_) >> Analyze(_) as insights
```

### 2. Name Intermediate Results

```javascript
// Good
FetchData() as raw
  >> Clean(raw) as clean
  >> Analyze(clean) as results

// Bad
FetchData() >> Clean(_) >> Analyze(_)
```

### 3. Use Types for Structure

```javascript
type Config { apiKey: String, endpoint: String }
type Result { data: [String], status: String }

def Process(config: Config) -> Result { ... }
```

### 4. Document Complex Workflows

```javascript
// Fetch user data from multiple sources
(
  FetchAPI(userId) ||      // External API
  FetchDB(userId) ||       // Database
  FetchCache(userId)       // Cache
) as [api, db, cache]

// Merge and deduplicate
  >> MergeData(api, db, cache) as merged

// Final processing
  >> Validate(merged) as validated
  >> Transform(validated) as final
```

### 5. Test Incrementally

```javascript
// Test each step
FetchData() as data
data    // Check output

Clean(data) as clean
clean   // Check output

Analyze(clean)  // Final step
```

## Keyboard Shortcuts Summary

### Global
- **F1** - REPL Mode
- **F2** - Workspace Mode
- **F3** - Type Explorer
- **Esc** / **Ctrl+C** - Quit

### REPL
- **Enter** - Execute
- **Shift+Enter** / **Alt+Enter** - Multi-line
- **Up/Down** - History
- **Ctrl+A** / **Home** - Start of line
- **Ctrl+E** / **End** - End of line

### Workspace
- **Tab** - Switch panes
- **Ctrl+R** - Run editor in REPL
- **Ctrl+E** - Send line to REPL
- **Ctrl+S** - Save file

## Command Reference

| Command | Description |
|---------|-------------|
| `:vars` | List all variables |
| `:types` | List all types |
| `:funcs` | List all functions |
| `:help` | Show help |
| `:copy <file>` | Save last result |
| `:debug` | Toggle debug mode |
| `:save <file>` | Save session |
| `:load <file>` | Load session |

## Pattern Matching ✅

**Status**: ✅ **Fully Functional** - Grammar, parser, and runtime all working!

Pattern matching is now available in the DSL, enabling powerful data destructuring, conditional logic, and elegant function definitions.

### Match Expressions

Match expressions provide a clean way to handle different cases:

```javascript
match 5 {
    0 => "zero",
    1 => "one",
    n => "other"
}
// Returns: "other"

match factorial(5) {
    120 => "correct!",
    n => "wrong: got " + n
}
// Returns: "correct!"
```

### Pattern Types

The following patterns are supported:

| Pattern | Example | Description |
|---------|---------|-------------|
| **Wildcard** | `_` | Matches anything |
| **Literal** | `0`, `"hello"`, `true` | Matches specific values |
| **Variable** | `x`, `name` | Binds to a variable |
| **Binding** | `x @ pattern` | Bind and match nested pattern |
| **Type** | `Int(x)`, `String(s)` | Type-based matching |
| **List** | `[head, ...tail]` | List destructuring |
| **Map** | `{name, age}` | Map destructuring |
| **Tuple** | `(a, b, c)` | Tuple destructuring |

### Guards

Patterns can include guards for additional conditions:

```javascript
match x {
    n if n < 0 => "negative"
    n if n > 0 => "positive"
    _ => "zero"
}
```

### List Destructuring

Extract elements from lists with rest patterns:

```javascript
match numbers {
    [] => "empty"
    [x] => "single element"
    [x, y] => "pair"
    [head, ...tail] => "head and rest"
}
```

### Map Destructuring

Extract fields from maps:

```javascript
match user {
    {name, age} => "User with name and age"
    {name} => "User with only name"
    _ => "Unknown structure"
}
```

### Expression Functions

Functions can now have expression bodies for general-purpose computation:

```javascript
// Simple expression function
function double(x) { x * 2 }
function square(x) { x * x }

// Recursive expression function
function factorial(n) {
    if n == 0 { 1 } else { n * factorial(n - 1) }
}
```

### Multi-Arm Function Definitions ✅

Pattern matching enables elegant function overloading with multiple arms:

```javascript
// Factorial using pattern matching
def factorial(0) { 1 }
def factorial(n) { n * factorial(n - 1) }

factorial(5)  // 120
factorial(0)  // 1

// FizzBuzz with patterns
def fizzbuzz(n) if n % 15 == 0 { "FizzBuzz" }
def fizzbuzz(n) if n % 3 == 0  { "Fizz" }
def fizzbuzz(n) if n % 5 == 0  { "Buzz" }
def fizzbuzz(n) { n }
```

**How it works:**
- Each `def` with the same name adds a new clause to the function
- The interpreter tries clauses **in order** until one matches
- First matching clause is executed
- If no clause matches, an error is thrown

### Implementation Status

- ✅ **IR Types**: IRPattern, IRMatchCase, IRFunctionGroup
- ✅ **Pattern Matcher**: Full pattern matching engine
- ✅ **Match Evaluation**: Match expressions work in interpreter
- ✅ **Grammar**: Pattern syntax fully integrated
- ✅ **Parser**: Parses multi-arm function definitions
- ✅ **Function Overloading**: Full runtime support with clause merging
- ✅ **TUI Integration**: REPL merges clauses correctly

**All pattern matching features are now fully functional!** 🎉

### Example Use Cases

Pattern matching enables many powerful patterns:

1. **Cleaner Conditionals** - Replace nested if/else with match
```javascript
match statusCode {
    200 => "OK",
    404 => "Not Found",
    500 => "Server Error",
    code => "Unknown: " + code
}
```

2. **Recursive Algorithms** - Elegant function definitions
```javascript
def sum([]) { 0 }
def sum([x, ...rest]) { x + sum(rest) }

def fibonacci(0) { 0 }
def fibonacci(1) { 1 }
def fibonacci(n) { fibonacci(n-1) + fibonacci(n-2) }
```

3. **Type-based Dispatch** - Different behavior for different inputs
```javascript
def process(0) { "zero special case" }
def process(n) if n < 0 { "negative: " + n }
def process(n) { "positive: " + n }
```

4. **Data Validation** - Pattern guards for constraints
```javascript
def validateAge(age) if age < 0 { "Error: negative age" }
def validateAge(age) if age > 150 { "Error: unrealistic age" }
def validateAge(age) { "Valid age: " + age }
```

5. **State Machines** - Clear state transitions
```javascript
def transition("idle", "start") { "running" }
def transition("running", "pause") { "paused" }
def transition("paused", "resume") { "running" }
def transition("running", "stop") { "idle" }
def transition(state, action) { "Invalid: " + state + " -> " + action }
```

## Next Steps

- **[12-Implementation-Details.md](12-Implementation-Details.md)** - Technical internals
- **[examples/](../examples/)** - Advanced workflow examples

## Related Documents

- **[MULTILINE_INPUT.md](../MULTILINE_INPUT.md)** - Multi-line input guide
- **[EDITOR_USAGE.md](../EDITOR_USAGE.md)** - Editor-REPL workflow
- **[SESSION_SUMMARY.md](../SESSION_SUMMARY.md)** - Session management
- **[COMPLETION_SUMMARY.md](../COMPLETION_SUMMARY.md)** - Feature completion status
