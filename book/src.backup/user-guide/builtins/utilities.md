# Utilities

Miscellaneous utility functions for common programming tasks.

## Zip

**Signature:** `(List, List) -> List`

Combine two lists into a list of pairs (2-element lists).

```dsl
Zip([1, 2, 3], ["a", "b", "c"])
// [[1, "a"], [2, "b"], [3, "c"]]

Zip([1, 2], [10, 20])
// [[1, 10], [2, 20]]
```

**Behavior:**
- Stops at the shorter list length
- Returns empty list if either input is empty

```dsl
Zip([1, 2, 3], ["a", "b"])  // [[1, "a"], [2, "b"]]
Zip([], [1, 2, 3])           // []
```

**Use Cases:**
```dsl
// Create key-value pairs
let keys = ["name", "age", "email"]
let values = ["Alice", 30, "alice@example.com"]
Zip(keys, values)

// Parallel iteration
let names = ["Alice", "Bob"]
let scores = [85, 92]
Map(Zip(names, scores), def (pair) := {
  name: pair[0],
  score: pair[1]
})
```

## Range

**Signature:** `(Int, Int) -> List`

Generate a list of integers from start (inclusive) to end (exclusive).

```dsl
Range(1, 5)   // [1, 2, 3, 4]
Range(0, 3)   // [0, 1, 2]
Range(5, 10)  // [5, 6, 7, 8, 9]
Range(3, 3)   // []
```

**Use Cases:**
```dsl
// Generate sequence
Range(1, 11)  // [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

// Iteration
Map(Range(0, 5), def (i) := i * 2)
// [0, 2, 4, 6, 8]

// Indices
let items = ["a", "b", "c"]
Range(0, Length(items))  // [0, 1, 2]

// Repeat operation N times
Map(Range(0, 10), def (_) := fetchData())
```

## Repeat

**Signature:** `(Any, Int) -> List`

Create a list with a value repeated N times.

```dsl
Repeat("x", 3)      // ["x", "x", "x"]
Repeat(0, 5)        // [0, 0, 0, 0, 0]
Repeat([1, 2], 2)   // [[1, 2], [1, 2]]
Repeat("hello", 0)  // []
```

**Use Cases:**
```dsl
// Initialize list
Repeat(0, 10)  // Ten zeros

// Create template
Repeat({id: 0, value: ""}, 5)

// Padding
Join([...Repeat(" ", 5), "text"], "")  // "     text"
```

## Chunk

**Signature:** `(List, Int) -> List`

Split a list into chunks of a given size. Last chunk may be smaller.

```dsl
Chunk([1, 2, 3, 4, 5], 2)
// [[1, 2], [3, 4], [5]]

Chunk(["a", "b", "c", "d"], 3)
// [["a", "b", "c"], ["d"]]

Chunk([1, 2, 3], 5)
// [[1, 2, 3]]
```

**Use Cases:**
```dsl
// Batch processing
let items = Range(1, 101)
let batches = Chunk(items, 10)
Map(batches, processBatch)

// Create rows
Chunk(data, 3)  // 3 columns per row

// Pagination
Chunk(allItems, pageSize)
```

## RenderMarkdown

**Signature:** `(String) -> Markdown`

Render a string as formatted markdown in the TUI.

```dsl
RenderMarkdown("# Hello\n\nThis is **bold** text")
// Displays formatted markdown in output

RenderMarkdown("""
# Report

## Summary
- Point 1
- Point 2

**Total**: 100
""")
```

**Use Cases:**
```dsl
// Format LLM responses
let response = Ask("Write a summary")
RenderMarkdown(response)

// Display reports
RenderMarkdown(generateReport(data))

// Pretty print documentation
RenderMarkdown(docString)
```

## Par

**Signature:** `(...expressions) -> List`

Execute multiple expressions in parallel and return results as a list.

```dsl
// Simple parallel execution
Par(5, 10, 15)  // [5, 10, 15]

// Parallel computations
Par(5 + 1, 10 + 2, 15 + 3)  // [6, 12, 18]
```

**With Variable Binding:**
```dsl
// Bind to list
Par(5, 10, 15) as numbers
numbers  // [5, 10, 15]

// Destructure results
Par(10, 20, 30) as [a, b, c]
a + b + c  // 60
```

**Parallel Operations:**
```dsl
// Parallel function calls
Par(
  fetchUsers(),
  fetchProducts(),
  fetchOrders()
) as [users, products, orders]

// Parallel API requests
def GetUser(id) := fetchFromAPI("users/" + ToString(id))

Par(GetUser(1), GetUser(2), GetUser(3)) as users
```

**Use Cases:**
```dsl
// Multiple LLM queries
Par(
  Ask("What is the capital of France?"),
  Ask("What is the capital of Germany?"),
  Ask("What is the capital of Italy?")
) as [france, germany, italy]

// Concurrent data fetching
Par(
  SQL("SELECT * FROM users"),
  SQL("SELECT * FROM orders"),
  SQL("SELECT * FROM products")
) as [users, orders, products]

// Independent computations
Par(
  processA(data),
  processB(data),
  processC(data)
) as results
```

## Not

**Signature:** `(Bool) -> Bool`

Logical NOT operation.

```dsl
Not(true)   // false
Not(false)  // true
Not(5 > 3)  // false
Not(5 < 3)  // true
```

**Use Cases:**
```dsl
// Negate condition
Not(isValid)

// Filter negation
Filter(items, def (item) := Not(item.deleted))

// Boolean logic
Not(a) && b  // !a && b
```

## ToString

**Signature:** `(Any) -> String`

Convert any value to its string representation.

```dsl
ToString(42)          // "42"
ToString(3.14)        // "3.14"
ToString(true)        // "true"
ToString([1, 2, 3])   // "[1, 2, 3]"
ToString({a: 1, b: 2}) // "{a: 1, b: 2}"
```

**Use Cases:**
```dsl
// Format for display
Map(numbers, ToString)

// Concatenation
"Value: " + ToString(value)

// Logging
Log(ToString(debugData))

// API integration
buildURL("id=" + ToString(userId))
```

## Practical Examples

### Example 1: Parallel Data Processing

```dsl
def processInParallel(urls) :=
  Par(
    ...Map(urls, def (url) := fetchData(url))
  ) as results
  Flatten(results)

let urls = [
  "https://api.example.com/data1",
  "https://api.example.com/data2",
  "https://api.example.com/data3"
]

processInParallel(urls)
```

### Example 2: Batch Processing with Chunks

```dsl
def processBatches(items, batchSize) :=
  items
    |> Chunk(_, batchSize)
    |> Map(_, def (batch) := {
         processed: Map(batch, processItem),
         count: Length(batch)
       })

processBatches(Range(1, 101), 10)
// 10 batches of 10 items each
```

### Example 3: Create Index Mapping

```dsl
def createIndex(items) :=
  let indices = Range(0, Length(items))
  Zip(indices, items)

createIndex(["a", "b", "c"])
// [[0, "a"], [1, "b"], [2, "c"]]

// Convert to map
def toIndexMap(items) :=
  Reduce(
    Zip(Range(0, Length(items)), items),
    {},
    def (map, pair) := {
      ...map,
      [ToString(pair[0])]: pair[1]
    }
  )
```

### Example 4: Parallel LLM Analysis

```dsl
def analyzeText(text) :=
  Par(
    getSummary(text),
    extractKeywords(text),
    getSentiment(text),
    getTopics(text)
  ) as [summary, keywords, sentiment, topics]

  {
    summary: summary,
    keywords: keywords,
    sentiment: sentiment,
    topics: topics
  }

def getSummary(text) := Ask("Summarize: " + text)
def extractKeywords(text) := Ask("Extract keywords: " + text)
def getSentiment(text) := Ask("Sentiment: " + text)
def getTopics(text) := Ask("Topics: " + text)
```

### Example 5: Matrix Operations with Zip

```dsl
def dotProduct(vec1, vec2) :=
  Zip(vec1, vec2)
    |> Map(_, def (pair) := pair[0] * pair[1])
    |> Sum(_)

dotProduct([1, 2, 3], [4, 5, 6])
// 1*4 + 2*5 + 3*6 = 32

def addVectors(vec1, vec2) :=
  Zip(vec1, vec2)
    |> Map(_, def (pair) := pair[0] + pair[1])

addVectors([1, 2, 3], [4, 5, 6])
// [5, 7, 9]
```

### Example 6: Generate Test Data

```dsl
def generateTestUsers(count) :=
  Range(1, count + 1)
    |> Map(_, def (i) := {
         id: i,
         name: "User" + ToString(i),
         email: "user" + ToString(i) + "@test.com",
         active: i % 2 == 0
       })

generateTestUsers(5)
// [
//   {id: 1, name: "User1", email: "user1@test.com", active: false},
//   {id: 2, name: "User2", email: "user2@test.com", active: true},
//   ...
// ]
```

### Example 7: Repeat with Variation

```dsl
def createGrid(rows, cols, initValue) :=
  Range(0, rows)
    |> Map(_, def (row) :=
         Range(0, cols)
           |> Map(_, def (col) := initValue))

createGrid(3, 3, 0)
// [[0, 0, 0], [0, 0, 0], [0, 0, 0]]

def createStaircase(height) :=
  Range(1, height + 1)
    |> Map(_, def (i) := Repeat("*", i))

createStaircase(5)
// [["*"], ["*", "*"], ["*", "*", "*"], ...]
```

### Example 8: Combining Utilities

```dsl
def processDataPipeline(data) :=
  // Chunk data for batch processing
  let batches = Chunk(data, 10)

  // Process batches in parallel
  let results = Par(
    ...Map(batches, def (batch) :=
      Map(batch, processItem))
  )

  // Flatten and filter results
  results
    |> Flatten(_)
    |> Filter(_, def (r) := Not(r.hasError))
    |> Map(_, def (r) := r.value)
```

## Combining Utilities

### Range + Map for Iteration

```dsl
// Execute N times
Range(0, 10)
  |> Map(_, def (i) := performOperation(i))

// Generate sequence
Range(1, 6)
  |> Map(_, def (i) := i * i)
// [1, 4, 9, 16, 25]
```

### Zip + Reduce for Dictionaries

```dsl
def zipToMap(keys, values) :=
  Zip(keys, values)
    |> Reduce(_, {}, def (map, pair) := {
         ...map,
         [pair[0]]: pair[1]
       })

zipToMap(["a", "b", "c"], [1, 2, 3])
// {a: 1, b: 2, c: 3}
```

### Chunk + Par for Parallel Batches

```dsl
def processInParallelBatches(items, batchSize) :=
  items
    |> Chunk(_, batchSize)
    |> Take(_, 5)  // Limit to 5 batches at a time
    |> _ as batches |> Par(
         ...Map(batches, processBatch)
       )
    |> Flatten(_)
```

## Best Practices

### 1. Use Range for Counted Loops

```dsl
// Good
Map(Range(0, 10), def (_) := generateRandomNumber())

// Bad
Repeat(null, 10) |> Map(_, def (_) := generateRandomNumber())
```

### 2. Parallel Operations for Independence

```dsl
// Good: Independent operations
Par(fetchA(), fetchB(), fetchC())

// Bad: Dependent operations (use sequential)
let a = fetchA()
let b = fetchB(a)  // Depends on a
let c = fetchC(b)  // Depends on b
```

### 3. Chunk for Memory Efficiency

```dsl
// Good: Process in chunks
largeData
  |> Chunk(_, 100)
  |> Map(_, processBatch)

// Bad: Process all at once (high memory)
Map(largeData, processItem)
```

### 4. Zip for Parallel Lists

```dsl
// Good: Zip related data
let users = Zip(names, emails)
  |> Map(_, def (pair) := {name: pair[0], email: pair[1]})

// Bad: Separate iteration
Map(Range(0, Length(names)), def (i) := {
  name: names[i],
  email: emails[i]
})
```

## Performance Tips

### 1. Use Par for I/O Operations

Parallel execution is most beneficial for I/O-bound operations:

```dsl
// Good: Parallel I/O
Par(
  fetchFromAPI("endpoint1"),
  fetchFromAPI("endpoint2"),
  fetchFromAPI("endpoint3")
)

// Less beneficial: CPU-bound (already fast)
Par(
  Sum([1, 2, 3]),
  Sum([4, 5, 6]),
  Sum([7, 8, 9])
)
```

### 2. Batch Size Optimization

```dsl
// Too small: High overhead
Chunk(items, 1)

// Too large: No benefit
Chunk(items, Length(items))

// Good: Balanced batches
Chunk(items, 50)  // Adjust based on item complexity
```

### 3. Minimize Zip Operations

```dsl
// Better: Single zip
let pairs = Zip(list1, list2)
Map(pairs, process)

// Worse: Multiple zips
Map(Range(0, Length(list1)), def (i) := {
  a: list1[i],
  b: list2[i]
})
```

## Next Steps

- [String Operations](./strings.md) - Text manipulation
- [List Operations](./lists.md) - Collection operations
- [Math Operations](./math.md) - Numerical functions
- [Functional Programming](./functional.md) - Map, Filter, Reduce
