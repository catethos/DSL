# Utilities

Miscellaneous utility functions for common programming tasks.

## Zip

**Signature:** `(List, List) -> List`

Combine two lists into a list of pairs (2-element lists).

```dsl
Zip([1, 2, 3], ["a", "b", "c"])

Zip([1, 2], [10, 20])
```

**Behavior:**
- Stops at the shorter list length
- Returns empty list if either input is empty

```dsl
Zip([1, 2, 3], ["a", "b"])
Zip([], [1, 2, 3])
```

**Use Cases:**
```dsl
def main() {
    let keys = ["name", "age", "email"]
    let values = ["Alice", 30, "alice@example.com"]
    Zip(keys, values)

    let names = ["Alice", "Bob"]
    let scores = [85, 92]
    Map(Zip(names, scores), fn pair => {
      name: pair[0],
      score: pair[1]
    } end)
}
main()
```

## Range

**Signature:** `(Int, Int) -> List`

Generate a list of integers from start (inclusive) to end (exclusive).

```dsl
Range(1, 5)
Range(0, 3)
Range(5, 10)
Range(3, 3)
```

**Use Cases:**
```dsl
Range(1, 11)

Map(Range(0, 5), fn i => i * 2 end)

let items = ["a", "b", "c"]
Range(0, Length(items))

Map(Range(0, 10), fn _ => fetchData( end))
```

## Repeat

**Signature:** `(Any, Int) -> List`

Create a list with a value repeated N times.

```dsl
Repeat("x", 3)
Repeat(0, 5)
Repeat([1, 2], 2)
Repeat("hello", 0)
```

**Use Cases:**
```dsl
Repeat(0, 10)

Repeat({id: 0, value: ""}, 5)

Join([...Repeat(" ", 5), "text"], "")
```

## Chunk

**Signature:** `(List, Int) -> List`

Split a list into chunks of a given size. Last chunk may be smaller.

```dsl
Chunk([1, 2, 3, 4, 5], 2)

Chunk(["a", "b", "c", "d"], 3)

Chunk([1, 2, 3], 5)
```

**Use Cases:**
```dsl
def main() {
    let items = Range(1, 101)
    let batches = Chunk(items, 10)
    Map(batches, processBatch)

    Chunk(data, 3)

    Chunk(allItems, pageSize)
}
main()
```

## RenderMarkdown

**Signature:** `(String) -> Markdown`

Render a string as formatted markdown in the TUI.

```dsl
RenderMarkdown("# Hello\n\nThis is **bold** text")

RenderMarkdown("""

- Point 1
- Point 2

**Total**: 100
""")
```

**Use Cases:**
```dsl
let response = Ask("Write a summary")
RenderMarkdown(response)

RenderMarkdown(generateReport(data))

RenderMarkdown(docString)
```

## Par

**Signature:** `(...expressions) -> List`

Execute multiple expressions in parallel and return results as a list.

```dsl
Par(5, 10, 15)

Par(5 + 1, 10 + 2, 15 + 3)
```

**With Variable Binding:**
```dsl
Par(5, 10, 15) as numbers
numbers

Par(10, 20, 30) as [a, b, c]
a + b + c
```

**Parallel Operations:**
```dsl
Par(
  fetchUsers(),
  fetchProducts(),
  fetchOrders()
) as [users, products, orders]

def GetUser(id) { fetchFromAPI("users/" + ToString(id)) }

Par(GetUser(1), GetUser(2), GetUser(3)) as users
```

**Use Cases:**
```dsl
Par(
  Ask("What is the capital of France?"),
  Ask("What is the capital of Germany?"),
  Ask("What is the capital of Italy?")
) as [france, germany, italy]

Par(
  SQL("SELECT * FROM users"),
  SQL("SELECT * FROM orders"),
  SQL("SELECT * FROM products")
) as [users, orders, products]

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
Not(true)
Not(false)
Not(5 > 3)
Not(5 < 3)
```

**Use Cases:**
```dsl
Not(isValid)

filter(items, fn item => Not(item.deleted end))

Not(a) && b
```

## ToString

**Signature:** `(Any) -> String`

Convert any value to its string representation.

```dsl
ToString(42)
ToString(3.14)
ToString(true)
ToString([1, 2, 3])
ToString({a: 1, b: 2})
```

**Use Cases:**
```dsl
Map(numbers, ToString)

"Value: " + ToString(value)

Log(ToString(debugData))

buildURL("id=" + ToString(userId))
```

## Practical Examples

### Example 1: Parallel Data Processing

```dsl
def processInParallel(urls) {
    Par(
    ...map(urls, fn url => fetchData(url end))
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
def processBatches(items, batchSize) { items }
    |> Chunk(_, batchSize)
    |> map(_, fn batch => {
         processed: Map(batch, processItem end),
         count: Length(batch)
       })

processBatches(Range(1, 101), 10)
```

### Example 3: Create Index Mapping

```dsl
def createIndex(items) { let indices = Range(0, Length(items)) }
  Zip(indices, items)

createIndex(["a", "b", "c"])

def toIndexMap(items) {
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
def analyzeText(text) {
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

def getSummary(text) { Ask("Summarize: " + text) }
def extractKeywords(text) { Ask("Extract keywords: " + text) }
def getSentiment(text) { Ask("Sentiment: " + text) }
def getTopics(text) { Ask("Topics: " + text) }
```

### Example 5: Matrix Operations with Zip

```dsl
def dotProduct(vec1, vec2) { Zip(vec1, vec2) }
    |> map(_, fn pair => pair[0] * pair[1] end)
    |> Sum(_)

dotProduct([1, 2, 3], [4, 5, 6])

def addVectors(vec1, vec2) { Zip(vec1, vec2) }
    |> map(_, fn pair => pair[0] + pair[1] end)

addVectors([1, 2, 3], [4, 5, 6])
```

### Example 6: Generate Test Data

```dsl
def generateTestUsers(count) { Range(1, count + 1) }
    |> map(_, fn i => {
         id: i,
         name: "User" + ToString(i end),
         email: "user" + ToString(i) + "@test.com",
         active: i % 2 == 0
       })

generateTestUsers(5)
```

### Example 7: Repeat with Variation

```dsl
def createGrid(rows, cols, initValue) { Range(0, rows) }
    |> map(_, fn row => Range(0, cols end)
           |> map(_, fn col => initValue end))

createGrid(3, 3, 0)

def createStaircase(height) { Range(1, height + 1) }
    |> map(_, fn i => Repeat("*", i end))

createStaircase(5)
```

### Example 8: Combining Utilities

```dsl
def processDataPipeline(data) {
  let batches = Chunk(data, 10)

  let results = Par(
    ...map(batches, fn batch => Map(batch, processItem end))
  )

  results
    |> Flatten(_)
    |> filter(_, fn r => Not(r.hasError end))
    |> map(_, fn r => r.value end)
```

## Combining Utilities

### Range + Map for Iteration

```dsl
Range(0, 10)
  |> map(_, fn i => performOperation(i end))

Range(1, 6)
  |> map(_, fn i => i * i end)
```

### Zip + Reduce for Dictionaries

```dsl
def zipToMap(keys, values) { Zip(keys, values) }
    |> Reduce(_, {}, def (map, pair) := {
         ...map,
         [pair[0]]: pair[1]
       })

zipToMap(["a", "b", "c"], [1, 2, 3])
```

### Chunk + Par for Parallel Batches

```dsl
def processInParallelBatches(items, batchSize) { items }
    |> Chunk(_, batchSize)
    |> Take(_, 5)
    |> _ as batches |> Par(
         ...Map(batches, processBatch)
       )
    |> Flatten(_)
```

## Best Practices

### 1. Use Range for Counted Loops

```dsl
Map(Range(0, 10), fn _ => generateRandomNumber( end))

Repeat(null, 10) |> map(_, fn _ => generateRandomNumber( end))
```

### 2. Parallel Operations for Independence

```dsl
def main() {
    Par(fetchA(), fetchB(), fetchC())

    let a = fetchA()
    let b = fetchB(a)
    let c = fetchC(b)
}
main()
```

### 3. Chunk for Memory Efficiency

```dsl
largeData
  |> Chunk(_, 100)
  |> Map(_, processBatch)

Map(largeData, processItem)
```

### 4. Zip for Parallel Lists

```dsl
let users = Zip(names, emails)
  |> map(_, fn pair => {name: pair[0], email: pair[1]} end)

Map(Range(0, Length(names)), fn i => {
  name: names[i],
  email: emails[i]
} end)
```

## Performance Tips

### 1. Use Par for I/O Operations

Parallel execution is most beneficial for I/O-bound operations:

```dsl
Par(
  fetchFromAPI("endpoint1"),
  fetchFromAPI("endpoint2"),
  fetchFromAPI("endpoint3")
)

Par(
  Sum([1, 2, 3]),
  Sum([4, 5, 6]),
  Sum([7, 8, 9])
)
```

### 2. Batch Size Optimization

```dsl
Chunk(items, 1)

Chunk(items, Length(items))

Chunk(items, 50)
```

### 3. Minimize Zip Operations

```dsl
let pairs = Zip(list1, list2)
Map(pairs, process)

Map(Range(0, Length(list1)), fn i => {
  a: list1[i],
  b: list2[i]
} end)
```

## Next Steps

- [String Operations](./strings.md) - Text manipulation
- [List Operations](./lists.md) - Collection operations
- [Math Operations](./math.md) - Numerical functions
- [Functional Programming](./functional.md) - Map, Filter, Reduce
