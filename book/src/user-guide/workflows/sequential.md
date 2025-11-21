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
LoadFile("file.txt") |> Filter(_) |> Sort(_) |> Unique(_)
```

### Basic Usage

```dsl
"hello" |> Upper(_)

"hello" |> Upper(_) |> Length(_)

5 |> _ * 2 |> _ + 3
```

## The Underscore Variable (`_`)

The `_` variable always contains the result of the previous step in a pipeline:

```dsl
[1, 2, 3, 4, 5] |> Length(_)

"test" |> Upper(_) |> Length(_)

[1, 2, 3] |> map(_, fn x => x * 2 end) |> Sum(_)
```

### Explicit Access

```dsl
[1, 2, 3, 4] |> filter(_, fn x => x % 2 == 0 end)

10 |> _ * 2 |> _ + 5 |> _ / 3
```

## Variable Binding with `as`

Bind intermediate results to named variables for clarity and reuse:

```dsl
["a", "b", "c"] |> Join(_, "-") as joined |> Upper(joined)

5 |> _ * 2 as doubled |> doubled + 3

[1, 2, 3, 4, 5]
  |> filter(_, fn x => x % 2 == 0 end) as evens
  |> map(evens, fn x => x * x end) as squares
  |> Sum(squares)
```

### Naming Improves Readability

```dsl
data
  |> cleanData(_) as cleaned
  |> validateData(cleaned) as validated
  |> transformData(validated) as transformed
  |> saveData(transformed)

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
def processUserData(users) {
  users
    |> filter(_, fn u => u.active end) as activeUsers
    |> map(activeUsers, fn u => {
         id: u.id,
         name: Trim(u.name end),
         email: Lower(Trim(u.email))
       } end) as normalized
    |> Sort(normalized) as sorted
    |> Take(sorted, 10)
}

processUserData(rawUsers)
```

### Example 2: Score Analysis

```dsl
def analyzeScores(scores) {
  scores
    |> filter(_, fn s => s > 0 end) as valid
    |> Sort(valid) as sorted
    |> Reverse(sorted) as descending
    |> Take(descending, 5) as top5
    |> {
         topScores: top5,
         average: Average(top5),
         highest: First(top5)
       }
}

analyzeScores([85, 92, 78, 95, 88, 76, 91, 89])
```

### Example 3: Content Generation

```dsl


type Outline { title: String, sections: [String] }
type Article { title: String, content: String, wordCount: Int }

def generateArticle(topic) { topic }
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
def fetchAndProcess(userId) { userId }
    |> ToString(_) as userIdStr
    |> fetchUserData(userIdStr) as userData
    |> userData.orders as orders
    |> map(orders, fn o => {
         id: o.id,
         total: o.amount * (1 + o.tax end)
       }) as processedOrders
    |> Sort(processedOrders) as sortedOrders
    |> {
         userId: userId,
         orderCount: Length(sortedOrders),
         totalSpent: Sum(map(sortedOrders, fn o => o.total end))
       }
```

### Example 5: CSV Processing

```dsl
def processCSV(filename) { SQL("SELECT * FROM '" + filename + "'") as raw }
    |> SQL("SELECT * FROM raw WHERE valid = true") as valid
    |> map(valid, fn row => {
         name: Trim(row.name end),
         value: ToFloat(row.value),
         category: Lower(row.category)
       }) as normalized
    |> filter(normalized, fn r => r.value > 0 end) as filtered
    |> {
         count: Length(filtered),
         total: Sum(map(filtered, fn r => r.value end)),
         categories: Unique(map(filtered, fn r => r.category end))
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
fetchDataA() as dataA
fetchDataB() as dataB

combineData(dataA, dataB) as combined

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
users
  |> Filter(_, isActive) as activeUsers
  |> Map(activeUsers, extractEmail) as emails

users
  |> Filter(_, isActive) as x
  |> Map(x, extractEmail) as y
```

### 4. Keep Functions Pure

```dsl
def double(x) { x * 2 }

data |> Map(_, double)

data |> map(_, fn x => {
  Log("Processing: " + ToString(x end))
  x * 2
})
```

### 5. Handle Errors Early

```dsl
data
  |> validateInput(_) as validated
  |> processData(validated) as processed
  |> formatOutput(processed)

data
  |> processData(_) as processed
  |> validateOutput(processed)
```

## Performance Considerations

### Sequential Execution Time

Operations execute one after another:

```dsl
slowOperation1()
  |> slowOperation2(_)
  |> slowOperation3(_)

```

### Optimization Tips

#### 1. Filter Early

```dsl
largeData
  |> Filter(_, isValid)
  |> Map(_, expensiveTransform)

largeData
  |> Map(_, expensiveTransform)
  |> Filter(_, isValid)
```

#### 2. Minimize Data Transfer

```dsl
hugeDataset
  |> aggregateByKey(_) as summary
  |> processSmallSummary(summary)

hugeDataset
  |> transferAllData(_)
  |> aggregateRemotely(_)
```

#### 3. Cache Expensive Computations

```dsl
let average = Average(scores)
scores
  |> map(_, fn s => s - average end)

scores
  |> map(_, fn s => s - Average(scores end))
```

## Common Patterns

### Pattern 1: Extract-Transform-Load (ETL)

```dsl
source
  |> extractData(_) as rawData

  |> cleanData(rawData) as cleanedData
  |> validateData(cleanedData) as validatedData
  |> transformData(validatedData) as transformedData

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

```

### Defensive Programming

```dsl
data
  |> _ as input
  |> Length(input) > 0
     ? processData(input)
     : defaultResult
```

## Debugging Pipelines

### Inspect Intermediate Values

```dsl
data
  |> step1(_) as result1
  |> step2(result1) as result2
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
