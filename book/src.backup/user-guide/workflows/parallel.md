# Parallel Workflows

Parallel workflows execute multiple operations concurrently, allowing independent tasks to run simultaneously. DSL uses the `Par()` function for parallel composition.

## The Par() Function

**Signature:** `(...expressions) -> List`

Execute multiple expressions in parallel and return results as a list.

```dsl
// Simple parallel execution
Par(5, 10, 15)  // [5, 10, 15]

// Parallel computations
Par(
  Calculate(dataA),
  Calculate(dataB),
  Calculate(dataC)
)
```

### How It Works

1. All arguments to `Par()` start executing immediately
2. Execution happens concurrently (not sequentially)
3. `Par()` waits for all operations to complete
4. Results are returned as a list in argument order

## Sequential vs Parallel

### Sequential Execution

Operations run one after another:

```dsl
// Sequential: Takes Time(A) + Time(B) + Time(C)
let resultA = FetchData("endpoint-a")
let resultB = FetchData("endpoint-b")
let resultC = FetchData("endpoint-c")
```

**Timeline:**
```
[A........] [B........] [C........]
Total: 3x time
```

### Parallel Execution

Operations run concurrently:

```dsl
// Parallel: Takes max(Time(A), Time(B), Time(C))
Par(
  FetchData("endpoint-a"),
  FetchData("endpoint-b"),
  FetchData("endpoint-c")
) as [resultA, resultB, resultC]
```

**Timeline:**
```
[A........]
[B........]
[C........]
Total: 1x time (if all take same time)
```

## Variable Binding

### Bind to List

```dsl
Par(5, 10, 15) as numbers
numbers  // [5, 10, 15]
```

### Destructure Results

```dsl
Par(
  GetUser(1),
  GetUser(2),
  GetUser(3)
) as [alice, bob, charlie]

alice.name  // "Alice"
```

### Named Results

```dsl
Par(
  FetchUsers(),
  FetchProducts(),
  FetchOrders()
) as [users, products, orders]

// Use destructured variables
users |> Length(_)
```

## Use Cases

### 1. Multiple LLM Queries

```dsl
type Summary { text: String, keywords: [String] }
type Sentiment { score: Float, label: String }
type Topics { primary: String, secondary: [String] }

let text = "Long article content..."

Par(
  Ask("Summarize: " + text),
  Ask("Sentiment analysis: " + text),
  Ask("Extract topics: " + text)
) as [summary, sentiment, topics]

{
  summary: summary,
  sentiment: sentiment,
  topics: topics
}
```

**Time Saved:**
- Sequential: 3 × 2 seconds = 6 seconds
- Parallel: max(2, 2, 2) = 2 seconds
- **Speedup: 3x**

### 2. Multiple API Requests

```dsl
def FetchUserData(userId) :=
  Par(
    HTTP("GET", "https://api.example.com/users/" + ToString(userId)),
    HTTP("GET", "https://api.example.com/users/" + ToString(userId) + "/orders"),
    HTTP("GET", "https://api.example.com/users/" + ToString(userId) + "/preferences")
  ) as [profile, orders, preferences]

  {
    profile: profile,
    orders: orders,
    preferences: preferences
  }

FetchUserData(123)
```

### 3. Data Processing

```dsl
def ProcessDatasets(files) :=
  Par(
    SQL("SELECT * FROM '" + files[0] + "'"),
    SQL("SELECT * FROM '" + files[1] + "'"),
    SQL("SELECT * FROM '" + files[2] + "'")
  ) as datasets

  // Combine results
  datasets
    |> Flatten(_)
    |> Filter(_, def (row) := row.valid)
```

### 4. Independent Calculations

```dsl
def CalculateMetrics(data) :=
  Par(
    Average(data),
    Min(data),
    Max(data),
    Sum(data)
  ) as [avg, min, max, sum]

  {
    average: avg,
    minimum: min,
    maximum: max,
    total: sum,
    range: max - min
  }
```

## Practical Examples

### Example 1: Multi-Source Data Aggregation

```dsl
type NewsArticle {
  title: String
  source: String
  content: String
}

def AggregateNews(topic) :=
  Par(
    FetchFromSource("reuters", topic),
    FetchFromSource("bbc", topic),
    FetchFromSource("cnn", topic)
  ) as sources

  sources
    |> Flatten(_)
    |> Filter(_, def (article) := Length(article.content) > 100)
    |> Take(_, 10)

def FetchFromSource(source, topic) :=
  HTTP("GET", "https://" + source + ".com/api/search?q=" + topic)
```

### Example 2: Parallel LLM Analysis

```dsl
type Article { title: String, content: String }

def AnalyzeArticle(article) :=
  Par(
    Ask("Generate 5 keywords for: " + article.content),
    Ask("Summarize in 2 sentences: " + article.content),
    Ask("What is the tone? Options: Positive, Negative, Neutral: " + article.content),
    Ask("Rate complexity 1-10: " + article.content)
  ) as [keywords, summary, tone, complexity]

  {
    title: article.title,
    keywords: Split(keywords, ","),
    summary: summary,
    tone: tone,
    complexity: ToInt(complexity)
  }
```

### Example 3: Batch User Processing

```dsl
def ProcessUserBatch(userIds) :=
  // Process up to 5 users in parallel
  let batch = Take(userIds, 5)

  Par(
    ...Map(batch, def (id) := GetUserData(id))
  ) as results

  results
    |> Filter(_, def (user) := user != null)
    |> Map(_, def (user) := {
         id: user.id,
         name: user.name,
         isActive: user.active
       })
```

### Example 4: Parallel Search

```dsl
def SearchAllSources(query) :=
  Par(
    SearchDatabase(query),
    SearchDocuments(query),
    SearchWeb(query)
  ) as [dbResults, docResults, webResults]

  {
    database: dbResults,
    documents: docResults,
    web: webResults,
    totalCount: Length(dbResults) + Length(docResults) + Length(webResults)
  }
```

### Example 5: Multi-Model LLM Ensemble

```dsl
def EnsemblePrediction(prompt) :=
  Par(
    AskModel("gpt-4", prompt),
    AskModel("claude", prompt),
    AskModel("gemini", prompt)
  ) as [gpt4, claude, gemini]

  // Combine results (majority vote, averaging, etc.)
  {
    gpt4: gpt4,
    claude: claude,
    gemini: gemini,
    consensus: FindConsensus([gpt4, claude, gemini])
  }
```

### Example 6: Parallel Data Transformation

```dsl
def TransformDatasets(data) :=
  // Split data into chunks
  let chunks = Chunk(data, 100)

  // Process first 5 chunks in parallel
  let batches = Take(chunks, 5)

  Par(
    ...Map(batches, def (chunk) :=
      chunk
        |> Filter(_, isValid)
        |> Map(_, transform)
        |> Sort(_))
  ) as processed

  // Flatten results
  Flatten(processed)
```

### Example 7: Concurrent File Processing

```dsl
def ProcessFiles(filenames) :=
  Par(
    ...Map(filenames, def (name) :=
      SQL("SELECT * FROM '" + name + "'")
        |> Filter(_, def (row) := row.valid)
        |> Map(_, def (row) := {
             id: row.id,
             value: ToFloat(row.value)
           }))
  ) as datasets

  {
    files: filenames,
    records: Flatten(datasets),
    totalRecords: Sum(Map(datasets, Length))
  }
```

### Example 8: Parallel Validation

```dsl
def ValidateData(data) :=
  Par(
    // Validation rules run in parallel
    All(data, def (item) := item.age >= 0),
    All(data, def (item) := Length(item.name) > 0),
    All(data, def (item) := Contains(item.email, "@")),
    All(data, def (item) := item.score >= 0 && item.score <= 100)
  ) as [validAge, validName, validEmail, validScore]

  {
    valid: validAge && validName && validEmail && validScore,
    checks: {
      age: validAge,
      name: validName,
      email: validEmail,
      score: validScore
    }
  }
```

## Combining Sequential and Parallel

### Pattern 1: Parallel Fetch → Sequential Process

```dsl
// Fetch data in parallel
Par(
  FetchDataA(),
  FetchDataB(),
  FetchDataC()
) as [dataA, dataB, dataC]

// Process sequentially
  |> CombineData(dataA, dataB, dataC) as combined
  |> ValidateData(combined) as validated
  |> TransformData(validated)
```

### Pattern 2: Sequential Setup → Parallel Execute

```dsl
// Setup
let config = LoadConfig()
let users = GetUsers()

// Parallel processing
Par(
  ...Map(users, def (user) := ProcessUser(user, config))
) as results

// Sequential aggregation
results
  |> Flatten(_)
  |> Filter(_, def (r) := r.success)
```

### Pattern 3: Mixed Sequential/Parallel Pipeline

```dsl
// Stage 1: Sequential setup
LoadData("input.csv") as rawData

// Stage 2: Parallel processing
  |> Chunk(_, 100) as chunks
  |> Take(chunks, 5) as batches
  |> _ as _ |> Par(
       ...Map(batches, ProcessBatch)
     ) as processed

// Stage 3: Sequential aggregation
  |> Flatten(processed)
  |> Sort(_)
  |> SaveData("output.csv")
```

### Pattern 4: Nested Parallel Operations

```dsl
def ProcessCategories(categories) :=
  // Outer parallel: Process categories
  Par(
    ...Map(categories, def (category) :=
      // Inner parallel: Process items in category
      Par(
        ...Map(category.items, ProcessItem)
      ) as items
      {
        category: category.name,
        items: items,
        count: Length(items)
      })
  ) as results

  {
    categories: results,
    totalItems: Sum(Map(results, def (r) := r.count))
  }
```

## Performance Considerations

### When to Use Parallel Execution

**Good Use Cases (I/O-bound):**

```dsl
// ✓ Network requests
Par(
  HTTP("GET", "https://api1.com/data"),
  HTTP("GET", "https://api2.com/data")
)

// ✓ LLM queries
Par(
  Ask("Question 1"),
  Ask("Question 2")
)

// ✓ Database queries
Par(
  SQL("SELECT * FROM users"),
  SQL("SELECT * FROM orders")
)

// ✓ File I/O
Par(
  LoadFile("data1.csv"),
  LoadFile("data2.csv")
)
```

**Poor Use Cases (CPU-bound):**

```dsl
// ✗ Simple calculations (overhead > benefit)
Par(
  Sum([1, 2, 3]),
  Sum([4, 5, 6])
)

// ✗ Small operations
Par(
  5 + 3,
  10 * 2
)
```

### Overhead Considerations

Parallel execution has overhead:

```dsl
// Not worth it: Operations too fast
Par(
  Length([1, 2, 3]),  // ~0.001ms
  First([1, 2, 3])    // ~0.001ms
)
// Parallel overhead: ~1ms
// Total: ~1ms (slower than sequential!)

// Worth it: Operations slow
Par(
  FetchData("url1"),  // ~500ms
  FetchData("url2")   // ~500ms
)
// Parallel overhead: ~1ms
// Total: ~501ms (sequential would be ~1000ms)
```

### Optimal Parallelism

Don't parallelize too many operations at once:

```dsl
// Bad: Too many parallel operations (may overwhelm system)
Par(
  ...Map(Range(1, 1001), def (i) := FetchData(i))
)  // 1000 concurrent requests!

// Good: Batch parallel operations
def ProcessInBatches(items, batchSize) :=
  items
    |> Chunk(_, batchSize)
    |> Map(_, def (batch) :=
         Par(...Map(batch, ProcessItem))
           |> Flatten(_))
    |> Flatten(_)

ProcessInBatches(Range(1, 1001), 10)
// 10 concurrent requests at a time
```

## Best Practices

### 1. Ensure Independence

Parallel operations must be independent:

```dsl
// Good: Independent operations
Par(
  FetchUsers(),
  FetchProducts(),
  FetchOrders()
)

// Bad: Dependent operations (use sequential)
let users = FetchUsers()
let orders = FetchOrdersForUsers(users)  // Depends on users!
```

### 2. Handle All Results

Always capture parallel results:

```dsl
// Good: Capture results
Par(
  Operation1(),
  Operation2(),
  Operation3()
) as [result1, result2, result3]

// Process results
result1 |> DoSomething(_)

// Bad: Ignoring results
Par(
  Operation1(),
  Operation2(),
  Operation3()
)
// Results are lost!
```

### 3. Keep Operations Similar

Group similar operations together:

```dsl
// Good: Similar durations
Par(
  FetchData("endpoint1"),  // ~500ms
  FetchData("endpoint2"),  // ~500ms
  FetchData("endpoint3")   // ~500ms
)
// Total: ~500ms

// Less optimal: Mixed durations
Par(
  FetchData("endpoint1"),  // ~2000ms
  QuickCalc(),            // ~10ms
  FetchData("endpoint3")   // ~500ms
)
// Total: ~2000ms (QuickCalc finishes early but must wait)
```

### 4. Limit Concurrency

Control the number of parallel operations:

```dsl
// Good: Reasonable concurrency
Par(
  FetchData("url1"),
  FetchData("url2"),
  FetchData("url3"),
  FetchData("url4"),
  FetchData("url5")
)  // 5 concurrent

// Better: Batch large datasets
let items = Range(1, 101)
items
  |> Chunk(_, 5)
  |> Map(_, def (batch) :=
       Par(...Map(batch, ProcessItem))
         |> Flatten(_))
  |> Flatten(_)
```

### 5. Use Descriptive Variable Names

```dsl
// Good: Clear names
Par(
  GetUserProfile(userId),
  GetUserOrders(userId),
  GetUserPreferences(userId)
) as [profile, orders, preferences]

// Bad: Generic names
Par(
  GetUserProfile(userId),
  GetUserOrders(userId),
  GetUserPreferences(userId)
) as [a, b, c]
```

## Error Handling

### Parallel Errors

If any parallel operation fails, the entire `Par()` fails:

```dsl
Par(
  SuccessfulOp(),
  FailingOp(),     // Throws error
  SuccessfulOp()
)
// Entire Par() fails, no results returned
```

### Defensive Parallel Execution

```dsl
// Wrap operations to handle errors
def SafeFetch(url) :=
  FetchData(url)
  // Add error handling logic here

Par(
  SafeFetch("url1"),
  SafeFetch("url2"),
  SafeFetch("url3")
) as results

// Filter out failures
results
  |> Filter(_, def (r) := r != null)
```

## Comparison Table

| Aspect | Sequential (`|>`) | Parallel (`Par()`) |
|--------|------------------|-------------------|
| **Execution** | One after another | All at once |
| **Time** | Sum of all operations | Max of all operations |
| **Use Case** | Dependent operations | Independent operations |
| **Overhead** | Minimal | Small constant |
| **Best For** | CPU-bound, dependent | I/O-bound, independent |
| **Syntax** | `a |> b |> c` | `Par(a, b, c)` |
| **Results** | Last step only | List of all results |

## Common Patterns

### Pattern 1: Fetch-Merge

```dsl
// Fetch from multiple sources in parallel
Par(
  FetchFromSourceA(),
  FetchFromSourceB(),
  FetchFromSourceC()
) as sources

// Merge sequentially
sources
  |> Flatten(_)
  |> Unique(_)
  |> Sort(_)
```

### Pattern 2: Map-Reduce

```dsl
let data = Chunk(largeDataset, 100)

// Map: Parallel processing
Par(
  ...Map(Take(data, 5), ProcessChunk)
) as results

// Reduce: Sequential aggregation
results
  |> Flatten(_)
  |> Reduce(_, 0, def (sum, item) := sum + item.value)
```

### Pattern 3: Scatter-Gather

```dsl
// Scatter: Distribute work
let tasks = [task1, task2, task3, task4, task5]

Par(
  ...Map(tasks, ExecuteTask)
) as results

// Gather: Collect and aggregate
{
  completed: Count(results, def (r) := r.success),
  failed: Count(results, def (r) := Not(r.success)),
  results: results
}
```

### Pattern 4: Pipeline with Parallel Stage

```dsl
// Stage 1: Sequential preprocessing
LoadData("input.csv") as raw
  |> CleanData(_) as cleaned
  |> ValidateData(cleaned) as validated

// Stage 2: Parallel processing
  |> Chunk(_, 50) as chunks
  |> _ as _ |> Par(
       ...Map(Take(chunks, 5), ProcessChunk)
     ) as processed

// Stage 3: Sequential postprocessing
  |> Flatten(processed)
  |> Sort(_)
  |> FormatOutput(_)
```

## Next Steps

- [Sequential Workflows](./sequential.md) - Pipeline operator and sequential composition
- [Workflow Patterns](./patterns.md) - Common patterns and anti-patterns
- [Builtin Functions](../builtins/README.md) - Functions to use in workflows
