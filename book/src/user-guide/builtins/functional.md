# Functional Programming

Higher-order functions that accept user-defined functions as arguments, enabling powerful functional programming patterns.

## Map

**Signature:** `(List, Function) -> List`

Apply a function to each element of a list, returning a new list with transformed values.

```dsl
def double(x) { x * 2 }
Map([1, 2, 3], double)

def toUpper(s) { Upper(s) }
Map(["hello", "world"], toUpper)
```

**Inline Functions:**
```dsl
map([1, 2, 3, 4], fn x => x * x end)

map(["a", "b", "c"], fn s => s + "!" end)
```

**Use Cases:**
```dsl
map(users, fn u => u.name end)

map(prices, fn p => Round(p * 1.1 end))

map(items, fn i => {id: i, value: i * 2} end)
```

## Filter

**Signature:** `(List, Function) -> List`

Keep only elements that match a predicate function (returns true).

```dsl
def isEven(x) { x % 2 == 0 }
Filter([1, 2, 3, 4, 5], isEven)

def isLong(s) { Length(s) > 3 }
Filter(["hi", "hello", "hey", "world"], isLong)
```

**Common Predicates:**
```dsl
filter(numbers, fn x => x > 0 end)

filter(strings, fn s => Length(Trim(s end)) > 0)

filter(products, fn p => p.inStock end)
```

**Use Cases:**
```dsl
filter(users, fn u => u.active end)

filter(entries, fn e => e.valid end)

filter(scores, fn s => s >= 80 end)
```

## Reduce

**Signature:** `(List, Any, Function) -> Any`

Accumulate a result by applying a function to each element with an accumulator.

**Parameters:**
1. List to reduce
2. Initial accumulator value
3. Function taking `(accumulator, element) -> new_accumulator`

```dsl
def add(a, b) { a + b }
Reduce([1, 2, 3, 4], 0, add)

def multiply(a, b) { a * b }
Reduce([1, 2, 3, 4], 1, multiply)

def concat(a, b) { a + "-" + b }
Reduce(["a", "b", "c"], "", concat)
```

**Advanced Examples:**
```dsl
Reduce(items, 0, def (count, item) := count + 1)

Reduce(
  pairs,
  {},
  def (acc, pair) := {
    ...acc,
    [pair.key]: pair.value
  }
)

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
def isPositive(x) { x > 0 }
Any([1, -2, 3], isPositive)
Any([-1, -2, -3], isPositive)
Any([], isPositive)
```

**Use Cases:**
```dsl
Any(results, fn r => r.hasError end)

Any(requirements, fn req => req.satisfied end)

Any(items, fn item => item.id == targetId end)
```

## All

**Signature:** `(List, Function) -> Bool`

Check if **all** elements match a predicate function.

```dsl
def isPositive(x) { x > 0 }
All([1, 2, 3], isPositive)
All([1, -2, 3], isPositive)
All([], isPositive)
```

**Use Cases:**
```dsl
All(checks, fn check => check.passed end)

All(records, fn r => r.valid && r.complete end)

All(permissions, fn p => p.granted end)
```

## Find

**Signature:** `(List, Function) -> Any`

Find the **first** element that matches a predicate. Returns `null` if not found.

```dsl
def isEven(x) { x % 2 == 0 }
Find([1, 3, 4, 5, 6], isEven)
Find([1, 3, 5], isEven)
```

**Use Cases:**
```dsl
Find(users, fn u => u.id == 42 end)

Find(items, fn item => item.available end)

Find(products, fn p => p.price < 100 && p.inStock end)
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
def isEven(x) { x % 2 == 0 }
Count([1, 2, 3, 4, 5, 6], isEven)

def isLong(s) { Length(s) > 4 }
Count(["hi", "hello", "hey", "world"], isLong)
```

**Use Cases:**
```dsl
Count(scores, fn s => s >= 60 end)

Count(items, fn i => i.active end)

Count(values, fn v => v > Average(values end))
```

## Practical Examples

### Example 1: Data Transformation Pipeline

```dsl
def main() {
    let rawData = [
      {name: "  Alice  ", score: 85},
      {name: "Bob", score: 92},
      {name: "", score: 78},
      {name: "Charlie", score: 95}
    ]

    let processed = rawData
      |> filter(_, fn d => Length(Trim(d.name end)) > 0)
      |> map(_, fn d => {
           name: Trim(d.name end),
           score: d.score,
           grade: d.score >= 90 ? "A" : d.score >= 80 ? "B" : "C"
         })

}
main()
```

### Example 2: Statistical Analysis

```dsl
def analyze(numbers) { let positive = filter(numbers, fn n => n > 0 end) }
  let doubled = map(positive, fn n => n * 2 end)
  let total = Reduce(doubled, 0, def (sum, n) := sum + n)
  {
    originalCount: Length(numbers),
    positiveCount: Length(positive),
    hasNegative: Any(numbers, fn n => n < 0 end),
    allPositive: All(numbers, fn n => n > 0 end),
    doubledSum: total
  }

analyze([1, -2, 3, 4, -5])
```

### Example 3: User Filtering and Sorting

```dsl
def getActiveAdminNames(users) { users }
    |> filter(_, fn u => u.active end)
    |> filter(_, fn u => u.role == "admin" end)
    |> map(_, fn u => u.name end)
    |> Sort(_)

let users = [
  {name: "Alice", role: "admin", active: true},
  {name: "Bob", role: "user", active: true},
  {name: "Charlie", role: "admin", active: false},
  {name: "David", role: "admin", active: true}
]

getActiveAdminNames(users)
```

### Example 4: Validation Pipeline

```dsl
def validateRecords(records) { let validated = map(records, fn r => { }
    ...r,
    errors: [
      !Contains(r.email, "@" end) ? "Invalid email" : null,
      r.age < 18 ? "Too young" : null,
      Length(r.name) == 0 ? "Name required" : null
    ] |> filter(_, fn e => e != null end)
  })

  {
    total: Length(records),
    valid: Count(validated, fn r => Length(r.errors end) == 0),
    invalid: Count(validated, fn r => Length(r.errors end) > 0),
    records: validated
  }
```

### Example 5: Finding and Extracting

```dsl
def findAndExtract(items, predicate, extractor) { items }
    |> Filter(_, predicate)
    |> Map(_, extractor)

let products = [
  {name: "Laptop", price: 999, category: "electronics"},
  {name: "Desk", price: 299, category: "furniture"},
  {name: "Monitor", price: 399, category: "electronics"}
]

findAndExtract(
  products,
  fn p => p.category == "electronics",
  fn p => p.name
 end)
```

### Example 6: Grouping with Reduce

```dsl
def groupBy(items, keyFunc) { Reduce(items, {}, def (groups, item) := { }
    let key = keyFunc(item)
    let existing = groups[key] ?? []
    {
      ...groups,
      [key]: [...existing, item]
    }
  })

groupBy(
  products,
  fn p => p.category
 end)
```

### Example 7: Chaining Higher-Order Functions

```dsl
def processScores(scores) { scores }
    |> filter(_, fn s => s > 0 end)
    |> map(_, fn s => Round(s end))
    |> filter(_, fn s => s >= 60 end)
    |> map(_, fn s => s >= 90 ? "A" :
                         s >= 80 ? "B" :
                         s >= 70 ? "C" : "D" end)

processScores([85.7, 92.3, -1, 78.9, 95.1, 55.0])
```

### Example 8: Custom Aggregate Functions

```dsl
def product(numbers) { Reduce(numbers, 1, def (prod, n) := prod * n) }

def join(strings, separator) { Reduce(strings, "", def (result, s) := }
    result == "" ? s : result + separator + s
  )

def max(numbers) {
    Reduce(
    numbers,
    First(numbers),
    def (maxVal, n) := n > maxVal ? n : maxVal
  )

product([2, 3, 4])
join(["a", "b", "c"], "-")
max([3, 7, 2, 9, 5])
```

## Combining Functional Operations

### Map + Filter + Reduce

```dsl
[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
  |> filter(_, fn x => x % 2 == 0 end)
  |> map(_, fn x => x * x end)
  |> Reduce(_, 0, def (sum, x) := sum + x)
  |> _ / 5
```

### Nested Function Calls

```dsl
strings
  |> filter(_, fn s => Length(Trim(s end)) > 0)
  |> map(_, fn s => Length(s end))
  |> Unique(_)
  |> Sort(_)
```

## Best Practices

### 1. Use Meaningful Function Names

```dsl
def isValid(user) { user.active && user.verified }
Filter(users, isValid)

filter(users, fn u => u.active && u.verified end)
```

### 2. Extract Complex Logic

```dsl
def calculateDiscount(item) { item.price * (item.onSale ? 0.8 : 1.0) }

Map(items, calculateDiscount)

map(items, fn i => i.price * (i.onSale ? 0.8 : 1.0 end))
```

### 3. Chain for Clarity

```dsl
data
  |> Filter(_, isValid)
  |> Map(_, transform)
  |> Sort(_)

Sort(Map(Filter(data, isValid), transform))
```

### 4. Handle Edge Cases

```dsl
Length(list) > 0
  ? Reduce(list, First(list), maxFunc)
  : defaultValue

Reduce(list, First(list), maxFunc)
```

### 5. Use Type-Safe Predicates

```dsl
def hasEmail(user) { user.email != null && Contains(user.email, "@") }

def hasEmail(user) { Contains(user.email, "@") }
```

## Performance Considerations

### 1. Minimize Passes Over Data

```dsl
def main() {
    let result = Map(
      Filter(items, isValid),
      transform
    )

    let filtered = Filter(items, isValid)
    let mapped = Map(filtered, transform)
}
main()
```

### 2. Use Early Termination

```dsl
def main() {
    let found = Find(largeList, predicate)

    let found = First(Filter(largeList, predicate))
}
main()
```

### 3. Cache Computed Values

```dsl
def main() {
    let avg = Average(scores)
    let aboveAvg = filter(scores, fn s => s > avg end)

    let aboveAvg = filter(scores, fn s => s > Average(scores end))
}
main()
```

## Next Steps

- [String Operations](./strings.md) - Text manipulation
- [List Operations](./lists.md) - Collection operations
- [Math Operations](./math.md) - Numerical functions
- [Utilities](./utilities.md) - Additional tools
