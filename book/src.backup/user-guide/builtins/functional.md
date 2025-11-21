# Functional Programming

Higher-order functions that accept user-defined functions as arguments, enabling powerful functional programming patterns.

## Map

**Signature:** `(List, Function) -> List`

Apply a function to each element of a list, returning a new list with transformed values.

```dsl
def double(x) := x * 2
Map([1, 2, 3], double)  // [2, 4, 6]

def toUpper(s) := Upper(s)
Map(["hello", "world"], toUpper)  // ["HELLO", "WORLD"]
```

**Inline Functions:**
```dsl
Map([1, 2, 3, 4], def (x) := x * x)
// [1, 4, 9, 16]

Map(["a", "b", "c"], def (s) := s + "!")
// ["a!", "b!", "c!"]
```

**Use Cases:**
```dsl
// Extract field from objects
Map(users, def (u) := u.name)

// Transform data
Map(prices, def (p) := Round(p * 1.1))

// Create new objects
Map(items, def (i) := {id: i, value: i * 2})
```

## Filter

**Signature:** `(List, Function) -> List`

Keep only elements that match a predicate function (returns true).

```dsl
def isEven(x) := x % 2 == 0
Filter([1, 2, 3, 4, 5], isEven)  // [2, 4]

def isLong(s) := Length(s) > 3
Filter(["hi", "hello", "hey", "world"], isLong)
// ["hello", "world"]
```

**Common Predicates:**
```dsl
// Filter positive numbers
Filter(numbers, def (x) := x > 0)

// Filter non-empty strings
Filter(strings, def (s) := Length(Trim(s)) > 0)

// Filter by property
Filter(products, def (p) := p.inStock)
```

**Use Cases:**
```dsl
// Get active users
Filter(users, def (u) := u.active)

// Remove invalid entries
Filter(entries, def (e) := e.valid)

// Select by condition
Filter(scores, def (s) := s >= 80)
```

## Reduce

**Signature:** `(List, Any, Function) -> Any`

Accumulate a result by applying a function to each element with an accumulator.

**Parameters:**
1. List to reduce
2. Initial accumulator value
3. Function taking `(accumulator, element) -> new_accumulator`

```dsl
def add(a, b) := a + b
Reduce([1, 2, 3, 4], 0, add)  // 10

def multiply(a, b) := a * b
Reduce([1, 2, 3, 4], 1, multiply)  // 24

def concat(a, b) := a + "-" + b
Reduce(["a", "b", "c"], "", concat)  // "-a-b-c"
```

**Advanced Examples:**
```dsl
// Count occurrences
Reduce(items, 0, def (count, item) := count + 1)

// Build object
Reduce(
  pairs,
  {},
  def (acc, pair) := {
    ...acc,
    [pair.key]: pair.value
  }
)

// Find maximum
Reduce(
  numbers,
  First(numbers),
  def (max, n) := n > max ? n : max
)
```

## Any

**Signature:** `(List, Function) -> Bool`

Check if **any** element matches a predicate function.

```dsl
def isPositive(x) := x > 0
Any([1, -2, 3], isPositive)  // true
Any([-1, -2, -3], isPositive)  // false
Any([], isPositive)  // false
```

**Use Cases:**
```dsl
// Check if any errors exist
Any(results, def (r) := r.hasError)

// Validate at least one condition
Any(requirements, def (req) := req.satisfied)

// Search for match
Any(items, def (item) := item.id == targetId)
```

## All

**Signature:** `(List, Function) -> Bool`

Check if **all** elements match a predicate function.

```dsl
def isPositive(x) := x > 0
All([1, 2, 3], isPositive)  // true
All([1, -2, 3], isPositive)  // false
All([], isPositive)  // true (vacuous truth)
```

**Use Cases:**
```dsl
// Validate all requirements met
All(checks, def (check) := check.passed)

// Ensure data quality
All(records, def (r) := r.valid && r.complete)

// Check permissions
All(permissions, def (p) := p.granted)
```

## Find

**Signature:** `(List, Function) -> Any`

Find the **first** element that matches a predicate. Returns `null` if not found.

```dsl
def isEven(x) := x % 2 == 0
Find([1, 3, 4, 5, 6], isEven)  // 4
Find([1, 3, 5], isEven)  // null
```

**Use Cases:**
```dsl
// Find by ID
Find(users, def (u) := u.id == 42)

// Get first match
Find(items, def (item) := item.available)

// Search with condition
Find(products, def (p) := p.price < 100 && p.inStock)
```

**Safe Usage:**
```dsl
let result = Find(items, predicate)
result != null ? result.name : "Not found"
```

## Count

**Signature:** `(List, Function) -> Int`

Count how many elements match a predicate function.

```dsl
def isEven(x) := x % 2 == 0
Count([1, 2, 3, 4, 5, 6], isEven)  // 3

def isLong(s) := Length(s) > 4
Count(["hi", "hello", "hey", "world"], isLong)  // 2
```

**Use Cases:**
```dsl
// Count passing grades
Count(scores, def (s) := s >= 60)

// Count active items
Count(items, def (i) := i.active)

// Statistical counting
Count(values, def (v) := v > Average(values))
```

## Practical Examples

### Example 1: Data Transformation Pipeline

```dsl
let rawData = [
  {name: "  Alice  ", score: 85},
  {name: "Bob", score: 92},
  {name: "", score: 78},
  {name: "Charlie", score: 95}
]

let processed = rawData
  |> Filter(_, def (d) := Length(Trim(d.name)) > 0)
  |> Map(_, def (d) := {
       name: Trim(d.name),
       score: d.score,
       grade: d.score >= 90 ? "A" : d.score >= 80 ? "B" : "C"
     })

// [
//   {name: "Alice", score: 85, grade: "B"},
//   {name: "Bob", score: 92, grade: "A"},
//   {name: "Charlie", score: 95, grade: "A"}
// ]
```

### Example 2: Statistical Analysis

```dsl
def analyze(numbers) :=
  let positive = Filter(numbers, def (n) := n > 0)
  let doubled = Map(positive, def (n) := n * 2)
  let total = Reduce(doubled, 0, def (sum, n) := sum + n)
  {
    originalCount: Length(numbers),
    positiveCount: Length(positive),
    hasNegative: Any(numbers, def (n) := n < 0),
    allPositive: All(numbers, def (n) := n > 0),
    doubledSum: total
  }

analyze([1, -2, 3, 4, -5])
// {
//   originalCount: 5,
//   positiveCount: 3,
//   hasNegative: true,
//   allPositive: false,
//   doubledSum: 16
// }
```

### Example 3: User Filtering and Sorting

```dsl
def getActiveAdminNames(users) :=
  users
    |> Filter(_, def (u) := u.active)
    |> Filter(_, def (u) := u.role == "admin")
    |> Map(_, def (u) := u.name)
    |> Sort(_)

let users = [
  {name: "Alice", role: "admin", active: true},
  {name: "Bob", role: "user", active: true},
  {name: "Charlie", role: "admin", active: false},
  {name: "David", role: "admin", active: true}
]

getActiveAdminNames(users)  // ["Alice", "David"]
```

### Example 4: Validation Pipeline

```dsl
def validateRecords(records) :=
  let validated = Map(records, def (r) := {
    ...r,
    errors: [
      !Contains(r.email, "@") ? "Invalid email" : null,
      r.age < 18 ? "Too young" : null,
      Length(r.name) == 0 ? "Name required" : null
    ] |> Filter(_, def (e) := e != null)
  })

  {
    total: Length(records),
    valid: Count(validated, def (r) := Length(r.errors) == 0),
    invalid: Count(validated, def (r) := Length(r.errors) > 0),
    records: validated
  }
```

### Example 5: Finding and Extracting

```dsl
def findAndExtract(items, predicate, extractor) :=
  items
    |> Filter(_, predicate)
    |> Map(_, extractor)

let products = [
  {name: "Laptop", price: 999, category: "electronics"},
  {name: "Desk", price: 299, category: "furniture"},
  {name: "Monitor", price: 399, category: "electronics"}
]

findAndExtract(
  products,
  def (p) := p.category == "electronics",
  def (p) := p.name
)
// ["Laptop", "Monitor"]
```

### Example 6: Grouping with Reduce

```dsl
def groupBy(items, keyFunc) :=
  Reduce(items, {}, def (groups, item) := {
    let key = keyFunc(item)
    let existing = groups[key] ?? []
    {
      ...groups,
      [key]: [...existing, item]
    }
  })

groupBy(
  products,
  def (p) := p.category
)
// {
//   electronics: [laptop, monitor],
//   furniture: [desk]
// }
```

### Example 7: Chaining Higher-Order Functions

```dsl
def processScores(scores) :=
  scores
    |> Filter(_, def (s) := s > 0)              // Remove invalid
    |> Map(_, def (s) := Round(s))              // Round values
    |> Filter(_, def (s) := s >= 60)            // Passing only
    |> Map(_, def (s) := s >= 90 ? "A" :        // Assign grades
                         s >= 80 ? "B" :
                         s >= 70 ? "C" : "D")

processScores([85.7, 92.3, -1, 78.9, 95.1, 55.0])
// ["B", "A", "C", "A"]
```

### Example 8: Custom Aggregate Functions

```dsl
def product(numbers) :=
  Reduce(numbers, 1, def (prod, n) := prod * n)

def join(strings, separator) :=
  Reduce(strings, "", def (result, s) :=
    result == "" ? s : result + separator + s
  )

def max(numbers) :=
  Reduce(
    numbers,
    First(numbers),
    def (maxVal, n) := n > maxVal ? n : maxVal
  )

product([2, 3, 4])  // 24
join(["a", "b", "c"], "-")  // "a-b-c"
max([3, 7, 2, 9, 5])  // 9
```

## Combining Functional Operations

### Map + Filter + Reduce

```dsl
// Calculate average of even squares
[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
  |> Filter(_, def (x) := x % 2 == 0)        // [2, 4, 6, 8, 10]
  |> Map(_, def (x) := x * x)                // [4, 16, 36, 64, 100]
  |> Reduce(_, 0, def (sum, x) := sum + x)   // 220
  |> _ / 5                                    // 44.0 (average)
```

### Nested Function Calls

```dsl
// Get unique lengths of non-empty strings
strings
  |> Filter(_, def (s) := Length(Trim(s)) > 0)
  |> Map(_, def (s) := Length(s))
  |> Unique(_)
  |> Sort(_)
```

## Best Practices

### 1. Use Meaningful Function Names

```dsl
// Good
def isValid(user) := user.active && user.verified
Filter(users, isValid)

// Bad
Filter(users, def (u) := u.active && u.verified)  // Inline with no context
```

### 2. Extract Complex Logic

```dsl
// Good
def calculateDiscount(item) :=
  item.price * (item.onSale ? 0.8 : 1.0)

Map(items, calculateDiscount)

// Bad (hard to read)
Map(items, def (i) := i.price * (i.onSale ? 0.8 : 1.0))
```

### 3. Chain for Clarity

```dsl
// Good
data
  |> Filter(_, isValid)
  |> Map(_, transform)
  |> Sort(_)

// Bad (nested)
Sort(Map(Filter(data, isValid), transform))
```

### 4. Handle Edge Cases

```dsl
// Good
Length(list) > 0
  ? Reduce(list, First(list), maxFunc)
  : defaultValue

// Bad (may error on empty list)
Reduce(list, First(list), maxFunc)
```

### 5. Use Type-Safe Predicates

```dsl
// Good
def hasEmail(user) :=
  user.email != null && Contains(user.email, "@")

// Bad (may error)
def hasEmail(user) := Contains(user.email, "@")
```

## Performance Considerations

### 1. Minimize Passes Over Data

```dsl
// Better: Single pass
let result = Map(
  Filter(items, isValid),
  transform
)

// Worse: Could be optimized
let filtered = Filter(items, isValid)
let mapped = Map(filtered, transform)
```

### 2. Use Early Termination

```dsl
// Better: Find stops at first match
let found = Find(largeList, predicate)

// Worse: Filter processes entire list
let found = First(Filter(largeList, predicate))
```

### 3. Cache Computed Values

```dsl
// Good
let avg = Average(scores)
let aboveAvg = Filter(scores, def (s) := s > avg)

// Bad: Recalculates average for each element
let aboveAvg = Filter(scores, def (s) := s > Average(scores))
```

## Next Steps

- [String Operations](./strings.md) - Text manipulation
- [List Operations](./lists.md) - Collection operations
- [Math Operations](./math.md) - Numerical functions
- [Utilities](./utilities.md) - Additional tools
