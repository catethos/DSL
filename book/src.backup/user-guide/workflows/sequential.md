# Sequential Workflows

Sequential workflows chain operations together, where the output of each step becomes the input to the next. DSL uses the pipeline operator `|>` for sequential composition.

## The Pipeline Operator (`|>`)

### Concept

The `|>` operator chains operations, passing the result of each step to the next. Think of it like Unix pipes:

```bash
# Unix pipes
cat file.txt | grep "keyword" | sort | uniq
```

```dsl
# DSL equivalent
LoadFile("file.txt") |> Filter(_) |> Sort(_) |> Unique(_)
```

### Basic Usage

```dsl
// Simple chain
"hello" |> Upper(_)
// "HELLO"

// Multi-step
"hello" |> Upper(_) |> Length(_)
// 5

// With numbers
5 |> _ * 2 |> _ + 3
// 13 (5 * 2 = 10, 10 + 3 = 13)
```

## The Underscore Variable (`_`)

The `_` variable always contains the result of the previous step in a pipeline:

```dsl
[1, 2, 3, 4, 5] |> Length(_)
// 5

"test" |> Upper(_) |> Length(_)
// 4 ("TEST" has 4 characters)

[1, 2, 3] |> Map(_, def (x) := x * 2) |> Sum(_)
// 12
```

### Explicit Access

```dsl
// Using _ to access the previous result
[1, 2, 3, 4] |> Filter(_, def (x) := x % 2 == 0)
// [2, 4]

// Mathematical operations on _
10 |> _ * 2 |> _ + 5 |> _ / 3
// 8.33... ((10 * 2 = 20) + 5 = 25) / 3
```

## Variable Binding with `as`

Bind intermediate results to named variables for clarity and reuse:

```dsl
["a", "b", "c"] |> Join(_, "-") as joined |> Upper(joined)
// "A-B-C"

5 |> _ * 2 as doubled |> doubled + 3
// 13

// Multiple bindings
[1, 2, 3, 4, 5]
  |> Filter(_, def (x) := x % 2 == 0) as evens
  |> Map(evens, def (x) := x * x) as squares
  |> Sum(squares)
// 20
```

### Naming Improves Readability

```dsl
// Good: Named intermediate results
data
  |> cleanData(_) as cleaned
  |> validateData(cleaned) as validated
  |> transformData(validated) as transformed
  |> saveData(transformed)

// Less clear: No names
data
  |> cleanData(_)
  |> validateData(_)
  |> transformData(_)
  |> saveData(_)
```

## Complex Workflows

### Data Processing Pipeline

```dsl
SQL("SELECT * FROM 'data.csv'") as raw
  |> SQL("SELECT * FROM raw WHERE score > 0.8") as filtered
  |> SQL("SELECT AVG(score) FROM filtered") as avg
  |> "Average score: " + ToString(avg)
```

### Text Processing

```dsl
"  hello,world,test  "
  |> Trim(_) as cleaned
  |> Split(cleaned, ",") as words
  |> Map(words, Upper) as uppercased
  |> Join(uppercased, " | ")
// "HELLO | WORLD | TEST"
```

### LLM Workflow

```dsl
type Article {
  title: String
  summary: String
  keywords: [String]
}

"AI Ethics" as topic
  |> Research(topic) as research
  |> CreateOutline(research, topic) as outline
  |> WriteArticle(outline) as draft
  |> ReviewArticle(draft) as feedback
  |> ReviseArticle(draft, feedback) as final
```

## Practical Examples

### Example 1: Data Transformation

```dsl
def processUserData(users) :=
  users
    |> Filter(_, def (u) := u.active) as activeUsers
    |> Map(activeUsers, def (u) := {
         id: u.id,
         name: Trim(u.name),
         email: Lower(Trim(u.email))
       }) as normalized
    |> Sort(normalized) as sorted
    |> Take(sorted, 10)

processUserData(rawUsers)
```

### Example 2: Score Analysis

```dsl
def analyzeScores(scores) :=
  scores
    |> Filter(_, def (s) := s > 0) as valid
    |> Sort(valid) as sorted
    |> Reverse(sorted) as descending
    |> Take(descending, 5) as top5
    |> {
         topScores: top5,
         average: Average(top5),
         highest: First(top5)
       }

analyzeScores([85, 92, 78, 95, 88, 76, 91, 89])
```

### Example 3: Content Generation

```dsl
type Outline { title: String, sections: [String] }
type Article { title: String, content: String, wordCount: Int }

def generateArticle(topic) :=
  topic
    |> Research(_) as research
    |> CreateOutline(research, topic) as outline
    |> WriteArticle(outline) as draft
    |> ReviewArticle(draft) as review
    |> review.score > 0.8
       ? draft
       : ReviseArticle(draft, review.suggestions)

generateArticle("Machine Learning Basics")
```

### Example 4: API Data Pipeline

```dsl
def fetchAndProcess(userId) :=
  userId
    |> ToString(_) as userIdStr
    |> fetchUserData(userIdStr) as userData
    |> userData.orders as orders
    |> Map(orders, def (o) := {
         id: o.id,
         total: o.amount * (1 + o.tax)
       }) as processedOrders
    |> Sort(processedOrders) as sortedOrders
    |> {
         userId: userId,
         orderCount: Length(sortedOrders),
         totalSpent: Sum(Map(sortedOrders, def (o) := o.total))
       }
```

### Example 5: CSV Processing

```dsl
def processCSV(filename) :=
  SQL("SELECT * FROM '" + filename + "'") as raw
    |> SQL("SELECT * FROM raw WHERE valid = true") as valid
    |> Map(valid, def (row) := {
         name: Trim(row.name),
         value: ToFloat(row.value),
         category: Lower(row.category)
       }) as normalized
    |> Filter(normalized, def (r) := r.value > 0) as filtered
    |> {
         count: Length(filtered),
         total: Sum(Map(filtered, def (r) := r.value)),
         categories: Unique(Map(filtered, def (r) := r.category))
       }

processCSV("data.csv")
```

## Best Practices

### 1. Name Intermediate Results

**Good:**
```dsl
data
  |> cleanData(_) as cleaned
  |> validateData(cleaned) as validated
  |> transformData(validated) as result
```

**Bad:**
```dsl
data
  |> cleanData(_)
  |> validateData(_)
  |> transformData(_)
```

### 2. Break Long Pipelines

**Good:**
```dsl
// Stage 1: Data Collection
fetchDataA() as dataA
fetchDataB() as dataB

// Stage 2: Processing
combineData(dataA, dataB) as combined

// Stage 3: Analysis
combined
  |> analyzeData(_) as analysis
  |> formatResults(analysis)
```

**Bad:**
```dsl
fetchDataA() |> combineWith(fetchDataB()) |> analyze(_) |> format(_) |> validate(_) |> save(_)
```

### 3. Use Descriptive Variable Names

```dsl
// Good
users
  |> Filter(_, isActive) as activeUsers
  |> Map(activeUsers, extractEmail) as emails

// Bad
users
  |> Filter(_, isActive) as x
  |> Map(x, extractEmail) as y
```

### 4. Keep Functions Pure

```dsl
// Good: Pure function (no side effects)
def double(x) := x * 2

data |> Map(_, double)

// Avoid: Side effects in pipeline
data |> Map(_, def (x) := {
  Log("Processing: " + ToString(x))  // Side effect
  x * 2
})
```

### 5. Handle Errors Early

```dsl
// Good: Validate early
data
  |> validateInput(_) as validated
  |> processData(validated) as processed
  |> formatOutput(processed)

// Bad: Process then validate
data
  |> processData(_) as processed
  |> validateOutput(processed)  // Too late!
```

## Performance Considerations

### Sequential Execution Time

Operations execute one after another:

```dsl
slowOperation1()
  |> slowOperation2(_)
  |> slowOperation3(_)

// Total time = Time(op1) + Time(op2) + Time(op3)
```

### Optimization Tips

#### 1. Filter Early

```dsl
// Good: Filter first (reduces data size)
largeData
  |> Filter(_, isValid)
  |> Map(_, expensiveTransform)

// Bad: Process everything first
largeData
  |> Map(_, expensiveTransform)
  |> Filter(_, isValid)
```

#### 2. Minimize Data Transfer

```dsl
// Good: Summarize early
hugeDataset
  |> aggregateByKey(_) as summary
  |> processSmallSummary(summary)

// Bad: Transfer all data
hugeDataset
  |> transferAllData(_)
  |> aggregateRemotely(_)
```

#### 3. Cache Expensive Computations

```dsl
// Good: Calculate once
let average = Average(scores)
scores
  |> Map(_, def (s) := s - average)

// Bad: Calculate repeatedly
scores
  |> Map(_, def (s) := s - Average(scores))
```

## Common Patterns

### Pattern 1: Extract-Transform-Load (ETL)

```dsl
// Extract
source
  |> extractData(_) as rawData

// Transform
  |> cleanData(rawData) as cleanedData
  |> validateData(cleanedData) as validatedData
  |> transformData(validatedData) as transformedData

// Load
  |> saveData(transformedData)
```

### Pattern 2: Validate-Process-Format

```dsl
input
  |> validateInput(_) as validated
  |> processData(validated) as processed
  |> formatOutput(processed)
```

### Pattern 3: Fetch-Parse-Aggregate

```dsl
url
  |> fetchData(_) as response
  |> parseJSON(response) as data
  |> extractRecords(data) as records
  |> aggregateResults(records)
```

### Pattern 4: Filter-Map-Reduce

```dsl
data
  |> Filter(_, isValid) as validData
  |> Map(validData, transform) as transformedData
  |> Reduce(transformedData, initialValue, combine)
```

## Error Handling

### Error Propagation

Errors stop execution and propagate:

```dsl
step1()
  |> step2(_)
  |> step3(_)

// If step2 fails, step3 doesn't execute
```

### Defensive Programming

```dsl
// Check before processing
data
  |> _ as input
  |> Length(input) > 0
     ? processData(input)
     : defaultResult
```

## Debugging Pipelines

### Inspect Intermediate Values

```dsl
// Add intermediate bindings to inspect
data
  |> step1(_) as result1  // Can examine result1
  |> step2(result1) as result2  // Can examine result2
  |> step3(result2)
```

### Log Pipeline Steps

```dsl
data
  |> cleanData(_) as cleaned
  |> _ as _ |> Log("Cleaned: " + ToString(Length(cleaned)))
  |> validateData(cleaned) as validated
  |> _ as _ |> Log("Validated: " + ToString(Length(validated)))
  |> processData(validated)
```

## Next Steps

- [Parallel Workflows](./parallel.md) - Execute operations concurrently
- [Workflow Patterns](./patterns.md) - Common patterns and anti-patterns
- [Builtin Functions](../builtins/README.md) - Functions to use in pipelines
