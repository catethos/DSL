# Workflow Patterns

Common patterns for building robust, maintainable workflows in DSL. Learn when to use sequential, parallel, or mixed approaches for different problem types.

## Data Pipeline Patterns

### Extract-Transform-Load (ETL)

Classic data processing pattern: extract from source, transform data, load to destination.

```dsl
def ETLPipeline(sourceFile, destFile) :=
  // Extract
  SQL("SELECT * FROM '" + sourceFile + "'") as rawData

  // Transform
    |> Filter(_, def (row) := row.valid) as validRows
    |> Map(validRows, def (row) := {
         id: row.id,
         value: ToFloat(row.value),
         category: Lower(Trim(row.category)),
         timestamp: row.created_at
       }) as transformed

  // Load
    |> SaveData(destFile, transformed)

ETLPipeline("input.csv", "output.csv")
```

**When to Use:**
- Batch data processing
- Data migration
- Report generation
- Data warehouse loading

### Streaming Pipeline

Process data continuously in small batches:

```dsl
def StreamProcess(dataStream) :=
  dataStream
    |> Chunk(_, 10) as batches
    |> Map(batches, def (batch) :=
         batch
           |> Filter(_, isValid)
           |> Map(_, transform)
           |> ProcessBatch(_))
    |> Flatten(_)
```

**When to Use:**
- Real-time data processing
- Large datasets (won't fit in memory)
- Continuous data flow

### Map-Reduce

Parallel processing with aggregation:

```dsl
def MapReduce(data, mapFunc, reduceFunc, initialValue) :=
  // Map phase: Parallel processing
  data
    |> Chunk(_, 100) as chunks
    |> Take(chunks, 5) as batches
    |> _ as _ |> Par(
         ...Map(batches, def (chunk) :=
           Map(chunk, mapFunc))
       ) as mapped

  // Reduce phase: Sequential aggregation
    |> Flatten(mapped)
    |> Reduce(_, initialValue, reduceFunc)

// Example: Word count
MapReduce(
  documents,
  def (doc) := Length(Split(doc.text, " ")),
  def (sum, count) := sum + count,
  0
)
```

**When to Use:**
- Large-scale data processing
- Aggregations over big datasets
- Distributed computation

### Fan-Out/Fan-In

Distribute work, then collect results:

```dsl
def FanOutFanIn(input, workers) :=
  // Fan-out: Distribute to workers
  Par(
    ...Map(workers, def (worker) := ProcessWithWorker(input, worker))
  ) as results

  // Fan-in: Collect and merge
  results
    |> Filter(_, def (r) := r.success)
    |> Map(_, def (r) := r.data)
    |> Flatten(_)
```

**When to Use:**
- Parallel processing with different strategies
- Multiple API endpoints
- Ensemble methods

## LLM Workflow Patterns

### Multi-Step Reasoning

Chain LLM calls where each step builds on the previous:

```dsl
type Analysis { insights: [String], confidence: Float }

def MultiStepAnalysis(text) :=
  text
    |> Ask("Identify key themes in: " + _) as themes
    |> Ask("Analyze sentiment of themes: " + themes) as sentiment
    |> Ask("Generate insights from: " + sentiment) as insights
    |> {
         themes: themes,
         sentiment: sentiment,
         insights: insights
       }
```

**When to Use:**
- Complex reasoning tasks
- Dependent analysis steps
- Building context progressively

### Parallel Analysis

Multiple independent LLM queries:

```dsl
def ParallelAnalysis(text) :=
  Par(
    Ask("Summarize: " + text),
    Ask("Extract keywords: " + text),
    Ask("Sentiment: " + text),
    Ask("Category: " + text)
  ) as [summary, keywords, sentiment, category]

  {
    summary: summary,
    keywords: Split(keywords, ","),
    sentiment: sentiment,
    category: category
  }
```

**When to Use:**
- Independent analyses
- Extracting multiple aspects
- Time-critical processing

### Critique and Refine

Generate, review, and improve:

```dsl
type Draft { content: String, issues: [String] }
type FinalDocument { content: String, quality: Float }

def CritiqueAndRefine(topic) :=
  // Generate initial draft
  Ask("Write an article about: " + topic) as draft

  // Critique
    |> Ask("What are issues with: " + draft) as critique

  // Refine based on critique
    |> Ask("Improve based on feedback. Original: " + draft + " Feedback: " + critique) as refined

  {
    draft: draft,
    critique: critique,
    final: refined
  }
```

**When to Use:**
- Content generation
- Quality improvement loops
- Self-verification

### Branching Logic

Conditional LLM paths:

```dsl
def BranchingWorkflow(input) :=
  // Classify input
  Ask("Classify as: technical, business, or personal: " + input) as classification

  // Branch based on classification
  classification == "technical" ? HandleTechnical(input) :
  classification == "business" ? HandleBusiness(input) :
  HandlePersonal(input)

def HandleTechnical(input) :=
  Ask("Provide technical analysis: " + input)

def HandleBusiness(input) :=
  Ask("Provide business analysis: " + input)

def HandlePersonal(input) :=
  Ask("Provide personal advice: " + input)
```

**When to Use:**
- Different handling for different inputs
- Specialized processing paths
- Context-dependent logic

### Ensemble Methods

Combine multiple LLM responses:

```dsl
def Ensemble(prompt) :=
  Par(
    Ask(prompt + " (Approach: analytical)"),
    Ask(prompt + " (Approach: creative)"),
    Ask(prompt + " (Approach: practical)")
  ) as [analytical, creative, practical]

  // Synthesize responses
  Ask("Synthesize these perspectives: 1) " + analytical + " 2) " + creative + " 3) " + practical)
```

**When to Use:**
- High-stakes decisions
- Multiple perspectives needed
- Bias reduction

## Validation Patterns

### Validate-Process-Format

Ensure data quality throughout:

```dsl
def ValidateProcessFormat(input) :=
  // Validate input
  input
    |> ValidateInput(_) as validated
    |> _ as _ |> Length(validated) == 0
       ? Error("No valid input data")
       : validated

  // Process
    |> Map(_, transform) as processed

  // Validate output
    |> ValidateOutput(_) as validOutput

  // Format
    |> FormatResults(validOutput)

def ValidateInput(data) :=
  Filter(data, def (item) :=
    item.id != null &&
    Length(item.name) > 0 &&
    item.value >= 0)

def ValidateOutput(data) :=
  Filter(data, def (item) :=
    item.result != null &&
    item.status == "success")
```

**When to Use:**
- Data quality critical
- Multiple processing steps
- Need to catch errors early

### Progressive Validation

Validate incrementally:

```dsl
def ProgressiveValidation(data) :=
  data
    // Level 1: Basic structure
    |> Filter(_, def (item) := item.id != null) as level1
    |> _ as _ |> Log("Passed basic validation: " + ToString(Length(level1)))

    // Level 2: Required fields
    |> Filter(_, def (item) :=
         item.name != null &&
         item.email != null) as level2
    |> _ as _ |> Log("Passed required fields: " + ToString(Length(level2)))

    // Level 3: Business rules
    |> Filter(_, def (item) :=
         item.age >= 18 &&
         Contains(item.email, "@")) as level3
    |> _ as _ |> Log("Passed business rules: " + ToString(Length(level3)))
```

**When to Use:**
- Complex validation rules
- Debugging data issues
- Progress tracking

## Error Handling Patterns

### Early Termination

Check conditions before expensive operations:

```dsl
def SafeProcess(data) :=
  // Check preconditions
  Length(data) == 0 ? Error("No data to process") :
  Length(data) > 10000 ? Error("Too much data") :

  // Proceed with processing
  data
    |> Filter(_, isValid)
    |> Map(_, expensiveTransform)
```

**When to Use:**
- Expensive operations
- Resource constraints
- Clear failure conditions

### Defensive Processing

Handle errors gracefully:

```dsl
def DefensiveProcess(data) :=
  data
    |> Map(_, def (item) := {
         ...item,
         processed: ProcessItem(item) ?? defaultValue,
         error: ProcessItem(item) == null ? "Failed" : null
       }) as results

  {
    successful: Filter(results, def (r) := r.error == null),
    failed: Filter(results, def (r) := r.error != null),
    stats: {
      total: Length(results),
      success: Count(results, def (r) := r.error == null),
      failure: Count(results, def (r) := r.error != null)
    }
  }
```

**When to Use:**
- Unreliable operations
- Batch processing
- Need to continue despite errors

## Optimization Patterns

### Filter Early

Reduce data size as soon as possible:

```dsl
// Good: Filter first
largeDataset
  |> Filter(_, isValid)              // Reduces size
  |> Map(_, expensiveTransform)      // Fewer items
  |> Sort(_)                          // Smaller sort

// Bad: Filter late
largeDataset
  |> Map(_, expensiveTransform)      // All items
  |> Sort(_)                          // All items
  |> Filter(_, isValid)              // Finally reduced
```

### Cache Expensive Computations

Calculate once, reuse many times:

```dsl
// Good: Calculate once
let average = Average(values)
let standardDev = CalculateStdDev(values, average)

values
  |> Map(_, def (v) := {
       value: v,
       zScore: (v - average) / standardDev
     })

// Bad: Calculate repeatedly
values
  |> Map(_, def (v) := {
       value: v,
       zScore: (v - Average(values)) / CalculateStdDev(values, Average(values))
     })
```

### Batch Processing

Process in manageable chunks:

```dsl
def BatchProcess(items, batchSize) :=
  items
    |> Chunk(_, batchSize) as batches
    |> Map(batches, def (batch) :=
         Par(
           ...Map(Take(batch, 5), ProcessItem)
         )
         |> Flatten(_))
    |> Flatten(_)

BatchProcess(largeDataset, 100)
```

## Anti-Patterns

### Anti-Pattern 1: Unnecessary Parallelization

**Bad:**
```dsl
// Operations too fast to benefit from parallelization
Par(
  Length([1, 2, 3]),
  Sum([1, 2, 3]),
  First([1, 2, 3])
)
```

**Good:**
```dsl
// Sequential is faster for simple operations
let len = Length([1, 2, 3])
let sum = Sum([1, 2, 3])
let first = First([1, 2, 3])
```

**Why Bad:** Parallel overhead exceeds time saved.

### Anti-Pattern 2: Sequential Dependent Data

**Bad:**
```dsl
// Using sequential when operations are dependent
let dataA = FetchData("a")
let dataB = FetchData("b")
let dataC = CombineData(dataA, dataB)  // Depends on A and B
```

**Good:**
```dsl
// Parallelize independent fetches
Par(
  FetchData("a"),
  FetchData("b")
) as [dataA, dataB]

let dataC = CombineData(dataA, dataB)
```

**Why Bad:** Wastes time when operations could run in parallel.

### Anti-Pattern 3: Nested Pipeline Chaos

**Bad:**
```dsl
// Hard to read nested pipelines
ProcessData(
  TransformData(
    FilterData(
      LoadData("file.csv")
    )
  )
)
```

**Good:**
```dsl
// Clear sequential pipeline
LoadData("file.csv")
  |> FilterData(_)
  |> TransformData(_)
  |> ProcessData(_)
```

**Why Bad:** Hard to read and debug.

### Anti-Pattern 4: Ignored Intermediate Results

**Bad:**
```dsl
data
  |> Process1(_)
  |> Process2(_)
  |> Process3(_)
  // What happened at each step?
```

**Good:**
```dsl
data
  |> Process1(_) as step1
  |> _ as _ |> Log("After step 1: " + ToString(Length(step1)))
  |> Process2(step1) as step2
  |> _ as _ |> Log("After step 2: " + ToString(Length(step2)))
  |> Process3(step2)
```

**Why Bad:** Hard to debug when something goes wrong.

### Anti-Pattern 5: Over-Parallelization

**Bad:**
```dsl
// Too many concurrent operations
Par(
  ...Map(Range(1, 10001), FetchData)
)  // 10,000 concurrent requests!
```

**Good:**
```dsl
// Controlled concurrency
def ProcessInBatches(ids, batchSize) :=
  ids
    |> Chunk(_, batchSize)
    |> Map(_, def (batch) :=
         Par(...Map(batch, FetchData))
           |> Flatten(_))
    |> Flatten(_)

ProcessInBatches(Range(1, 10001), 10)
```

**Why Bad:** Overwhelms system resources.

### Anti-Pattern 6: No Error Handling

**Bad:**
```dsl
// Assumes everything succeeds
data
  |> Map(_, riskyOperation)
  |> Process(_)
```

**Good:**
```dsl
// Handles failures gracefully
data
  |> Map(_, def (item) := {
       ...item,
       result: riskyOperation(item) ?? null,
       failed: riskyOperation(item) == null
     })
  |> Filter(_, def (item) := Not(item.failed))
  |> Map(_, def (item) := item.result)
  |> Process(_)
```

**Why Bad:** One failure breaks entire pipeline.

### Anti-Pattern 7: Repeated Expensive Calls

**Bad:**
```dsl
// Calls expensive function multiple times
users
  |> Filter(_, def (u) := IsValidUser(u, GetConfig()))
  |> Map(_, def (u) := ProcessUser(u, GetConfig()))
  |> Sort(_, def (u) := ScoreUser(u, GetConfig()))
```

**Good:**
```dsl
// Call once, reuse
let config = GetConfig()

users
  |> Filter(_, def (u) := IsValidUser(u, config))
  |> Map(_, def (u) := ProcessUser(u, config))
  |> Sort(_, def (u) := ScoreUser(u, config))
```

**Why Bad:** Unnecessary repeated work.

### Anti-Pattern 8: Monolithic Functions

**Bad:**
```dsl
def ProcessEverything(data) :=
  // 200 lines of complex logic
  // Hard to test, maintain, or understand
  ...
```

**Good:**
```dsl
def ProcessEverything(data) :=
  data
    |> Validate(_) as validated
    |> Transform(validated) as transformed
    |> Enrich(transformed) as enriched
    |> Format(enriched)

def Validate(data) := ...
def Transform(data) := ...
def Enrich(data) := ...
def Format(data) := ...
```

**Why Bad:** Hard to maintain and test.

## Composition Patterns

### Pattern Composition

Combine patterns for complex workflows:

```dsl
def ComplexWorkflow(input) :=
  // Pattern 1: ETL
  input
    |> Extract(_) as raw
    |> Transform(raw) as transformed

  // Pattern 2: Fan-Out (Parallel Processing)
    |> Chunk(transformed, 100) as chunks
    |> _ as _ |> Par(
         ...Map(Take(chunks, 5), ProcessChunk)
       ) as processed

  // Pattern 3: Map-Reduce
    |> Flatten(processed) as flattened
    |> Reduce(flattened, initialState, aggregateFunc) as aggregated

  // Pattern 4: Validate-Format
    |> ValidateOutput(aggregated) as validated
    |> FormatOutput(validated)
```

### Layered Architecture

Separate concerns into layers:

```dsl
// Layer 1: Data Access
def LoadUsers() := SQL("SELECT * FROM users")
def LoadOrders() := SQL("SELECT * FROM orders")

// Layer 2: Business Logic
def ProcessUsers(users) :=
  users
    |> Filter(_, isActive)
    |> Map(_, enrichUser)

def ProcessOrders(orders, users) :=
  orders
    |> Filter(_, isValid)
    |> Map(_, def (o) := AttachUser(o, users))

// Layer 3: Application
def FullPipeline() :=
  Par(
    LoadUsers(),
    LoadOrders()
  ) as [users, orders]

  let processedUsers = ProcessUsers(users)
  let processedOrders = ProcessOrders(orders, processedUsers)

  {
    users: processedUsers,
    orders: processedOrders
  }
```

## Best Practices Summary

### 1. Choose the Right Pattern

- **Sequential** for dependent operations
- **Parallel** for independent I/O operations
- **Batch** for large datasets
- **Streaming** for continuous data

### 2. Name Intermediate Results

```dsl
// Good
data
  |> Process1(_) as step1
  |> Process2(step1) as step2
  |> Process3(step2)

// Bad
data
  |> Process1(_)
  |> Process2(_)
  |> Process3(_)
```

### 3. Validate Early and Often

```dsl
input
  |> ValidateInput(_)
  |> Process(_)
  |> ValidateOutput(_)
```

### 4. Handle Errors Gracefully

```dsl
data
  |> Map(_, safeTransform)
  |> Filter(_, isNotNull)
```

### 5. Optimize for Readability

```dsl
// Clear, self-documenting code
LoadData("input.csv")
  |> CleanData(_) as cleaned
  |> ValidateData(cleaned) as validated
  |> TransformData(validated) as transformed
  |> SaveData("output.csv", transformed)
```

## Next Steps

- [Sequential Workflows](./sequential.md) - Pipeline operator and sequential composition
- [Parallel Workflows](./parallel.md) - Concurrent execution with Par()
- [Builtin Functions](../builtins/README.md) - Functions to use in workflows
- [Type System](../type-system/custom-types.md) - Defining structured data
