# Parallel Workflows

Parallel workflows execute multiple operations concurrently, allowing independent tasks to run simultaneously. DSL uses the `Par()` function for parallel composition.

## The Par() Function

**Signature:** `(...expressions) -> List`

Execute multiple expressions in parallel and return results as a list.

```dsl
Par(5, 10, 15)

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
def main() {
    let resultA = FetchData("endpoint-a")
    let resultB = FetchData("endpoint-b")
    let resultC = FetchData("endpoint-c")
}
main()
```

**Timeline:**
```
[A........] [B........] [C........]
Total: 3x time
```

### Parallel Execution

Operations run concurrently:

```dsl
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
numbers
```

### Destructure Results

```dsl
Par(
  GetUser(1),
  GetUser(2),
  GetUser(3)
) as [alice, bob, charlie]

alice.name
```

### Named Results

```dsl
Par(
  FetchUsers(),
  FetchProducts(),
  FetchOrders()
) as [users, products, orders]

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
def FetchUserData(userId) {
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
}

FetchUserData(123)
```

### 3. Data Processing

```dsl
def ProcessDatasets(files) {
    Par(
    SQL("SELECT * FROM '" + files[0] + "'"),
    SQL("SELECT * FROM '" + files[1] + "'"),
    SQL("SELECT * FROM '" + files[2] + "'")
  ) as datasets

  datasets
    |> Flatten(_)
    |> filter(_, fn row => row.valid end)
```

### 4. Independent Calculations

```dsl
def CalculateMetrics(data) {
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

def AggregateNews(topic) {
    Par(
    FetchFromSource("reuters", topic),
    FetchFromSource("bbc", topic),
    FetchFromSource("cnn", topic)
  ) as sources

  sources
    |> Flatten(_)
    |> filter(_, fn article => Length(article.content end) > 100)
    |> Take(_, 10)

def FetchFromSource(source, topic) { HTTP("GET", "https://" + source + ".com/api/search?q=" + topic) }
```

### Example 2: Parallel LLM Analysis

```dsl


type Article { title: String, content: String }

def AnalyzeArticle(article) {
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
def ProcessUserBatch(userIds) {
  let batch = Take(userIds, 5)

  Par(
    ...map(batch, fn id => GetUserData(id end))
  ) as results

  results
    |> filter(_, fn user => user != null end)
    |> map(_, fn user => {
         id: user.id,
         name: user.name,
         isActive: user.active
       } end)
```

### Example 4: Parallel Search

```dsl
def SearchAllSources(query) {
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
def EnsemblePrediction(prompt) {
    Par(
    AskModel("gpt-4", prompt),
    AskModel("claude", prompt),
    AskModel("gemini", prompt)
  ) as [gpt4, claude, gemini]

  {
    gpt4: gpt4,
    claude: claude,
    gemini: gemini,
    consensus: FindConsensus([gpt4, claude, gemini])
  }
```

### Example 6: Parallel Data Transformation

```dsl
def TransformDatasets(data) {
  let chunks = Chunk(data, 100)

  let batches = Take(chunks, 5)

  Par(
    ...map(batches, fn chunk => chunk
        |> Filter(_, isValid end)
        |> Map(_, transform)
        |> Sort(_))
  ) as processed

  Flatten(processed)
```

### Example 7: Concurrent File Processing

```dsl
def ProcessFiles(filenames) {
    Par(
    ...map(filenames, fn name => SQL("SELECT * FROM '" + name + "'" end)
        |> filter(_, fn row => row.valid end)
        |> map(_, fn row => {
             id: row.id,
             value: ToFloat(row.value end)
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
def ValidateData(data) {
    Par(
    All(data, fn item => item.age >= 0 end),
    All(data, fn item => Length(item.name end) > 0),
    All(data, fn item => Contains(item.email, "@" end)),
    All(data, fn item => item.score >= 0 && item.score <= 100 end)
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
Par(
  FetchDataA(),
  FetchDataB(),
  FetchDataC()
) as [dataA, dataB, dataC]

  |> CombineData(dataA, dataB, dataC) as combined
  |> ValidateData(combined) as validated
  |> TransformData(validated)
```

### Pattern 2: Sequential Setup → Parallel Execute

```dsl
def main() {
    let config = LoadConfig()
    let users = GetUsers()

    Par(
      ...map(users, fn user => ProcessUser(user, config end))
    ) as results

    results
      |> Flatten(_)
      |> filter(_, fn r => r.success end)
}
main()
```

### Pattern 3: Mixed Sequential/Parallel Pipeline

```dsl
LoadData("input.csv") as rawData

  |> Chunk(_, 100) as chunks
  |> Take(chunks, 5) as batches
  |> _ as _ |> Par(
       ...Map(batches, ProcessBatch)
     ) as processed

  |> Flatten(processed)
  |> Sort(_)
  |> SaveData("output.csv")
```

### Pattern 4: Nested Parallel Operations

```dsl
def ProcessCategories(categories) {
  Par(
    ...map(categories, fn category =>
      Par(
        ...Map(category.items, ProcessItem end)
      ) as items
      {
        category: category.name,
        items: items,
        count: Length(items)
      })
  ) as results

  {
    categories: results,
    totalItems: Sum(map(results, fn r => r.count end))
  }
```

## Performance Considerations

### When to Use Parallel Execution

**Good Use Cases (I/O-bound):**

```dsl
Par(
  HTTP("GET", "https://api1.com/data"),
  HTTP("GET", "https://api2.com/data")
)

Par(
  Ask("Question 1"),
  Ask("Question 2")
)

Par(
  SQL("SELECT * FROM users"),
  SQL("SELECT * FROM orders")
)

Par(
  LoadFile("data1.csv"),
  LoadFile("data2.csv")
)
```

**Poor Use Cases (CPU-bound):**

```dsl
Par(
  Sum([1, 2, 3]),
  Sum([4, 5, 6])
)

Par(
  5 + 3,
  10 * 2
)
```

### Overhead Considerations

Parallel execution has overhead:

```dsl
Par(
  Length([1, 2, 3]),
  First([1, 2, 3])
)

Par(
  FetchData("url1"),
  FetchData("url2")
)
```

### Optimal Parallelism

Don't parallelize too many operations at once:

```dsl
Par(
  ...Map(Range(1, 1001), fn i => FetchData(i end))
)

def ProcessInBatches(items, batchSize) { items }
    |> Chunk(_, batchSize)
    |> map(_, fn batch => Par(...Map(batch, ProcessItem end))
           |> Flatten(_))
    |> Flatten(_)

ProcessInBatches(Range(1, 1001), 10)
```

## Best Practices

### 1. Ensure Independence

Parallel operations must be independent:

```dsl
def main() {
    Par(
      FetchUsers(),
      FetchProducts(),
      FetchOrders()
    )

    let users = FetchUsers()
    let orders = FetchOrdersForUsers(users)
}
main()
```

### 2. Handle All Results

Always capture parallel results:

```dsl
Par(
  Operation1(),
  Operation2(),
  Operation3()
) as [result1, result2, result3]

result1 |> DoSomething(_)

Par(
  Operation1(),
  Operation2(),
  Operation3()
)
```

### 3. Keep Operations Similar

Group similar operations together:

```dsl
Par(
  FetchData("endpoint1"),
  FetchData("endpoint2"),
  FetchData("endpoint3")
)

Par(
  FetchData("endpoint1"),
  QuickCalc(),
  FetchData("endpoint3")
)
```

### 4. Limit Concurrency

Control the number of parallel operations:

```dsl
Par(
  FetchData("url1"),
  FetchData("url2"),
  FetchData("url3"),
  FetchData("url4"),
  FetchData("url5")
)

let items = Range(1, 101)
items
  |> Chunk(_, 5)
  |> map(_, fn batch => Par(...Map(batch, ProcessItem end))
         |> Flatten(_))
  |> Flatten(_)
```

### 5. Use Descriptive Variable Names

```dsl
Par(
  GetUserProfile(userId),
  GetUserOrders(userId),
  GetUserPreferences(userId)
) as [profile, orders, preferences]

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
  FailingOp(),
  SuccessfulOp()
)
```

### Defensive Parallel Execution

```dsl
def SafeFetch(url) { FetchData(url) }

Par(
  SafeFetch("url1"),
  SafeFetch("url2"),
  SafeFetch("url3")
) as results

results
  |> filter(_, fn r => r != null end)
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
Par(
  FetchFromSourceA(),
  FetchFromSourceB(),
  FetchFromSourceC()
) as sources

sources
  |> Flatten(_)
  |> Unique(_)
  |> Sort(_)
```

### Pattern 2: Map-Reduce

```dsl
let data = Chunk(largeDataset, 100)

Par(
  ...Map(Take(data, 5), ProcessChunk)
) as results

results
  |> Flatten(_)
  |> Reduce(_, 0, def (sum, item) := sum + item.value)
```

### Pattern 3: Scatter-Gather

```dsl
let tasks = [task1, task2, task3, task4, task5]

Par(
  ...Map(tasks, ExecuteTask)
) as results

{
  completed: Count(results, fn r => r.success end),
  failed: Count(results, fn r => Not(r.success end)),
  results: results
}
```

### Pattern 4: Pipeline with Parallel Stage

```dsl
LoadData("input.csv") as raw
  |> CleanData(_) as cleaned
  |> ValidateData(cleaned) as validated

  |> Chunk(_, 50) as chunks
  |> _ as _ |> Par(
       ...Map(Take(chunks, 5), ProcessChunk)
     ) as processed

  |> Flatten(processed)
  |> Sort(_)
  |> FormatOutput(_)
```

## Next Steps

- [Sequential Workflows](./sequential.md) - Pipeline operator and sequential composition
- [Workflow Patterns](./patterns.md) - Common patterns and anti-patterns
- [Builtin Functions](../builtins/README.md) - Functions to use in workflows
