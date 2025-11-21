# Math Operations

Mathematical functions for numerical calculations and aggregations.

## Abs

**Signature:** `(Int | Float) -> Int | Float`

Calculate the absolute value of a number.

```dsl
Abs(-42)  // 42
Abs(42)   // 42
Abs(-3.14)  // 3.14
Abs(0)    // 0
```

**Use Cases:**
```dsl
// Distance calculation
Abs(x1 - x2)

// Error magnitude
Abs(expected - actual)

// Normalize to positive
Map(numbers, Abs)
```

## Min

**Signature:** `(List) -> Int | Float`

Find the minimum value in a list of numbers. **Errors on empty list.**

```dsl
Min([3, 1, 4, 1, 5])  // 1
Min([2.5, 1.2, 3.7])  // 1.2
Min([-5, -2, -10])    // -10
Min([42])             // 42
Min([])  // Error: Cannot find min of empty list
```

**Use Cases:**
```dsl
// Find lowest score
Min(scores)

// Get minimum price
Min(Map(products, def (p) := p.price))

// Range calculation
let range = Max(values) - Min(values)
```

## Max

**Signature:** `(List) -> Int | Float`

Find the maximum value in a list of numbers. **Errors on empty list.**

```dsl
Max([3, 1, 4, 1, 5])  // 5
Max([2.5, 1.2, 3.7])  // 3.7
Max([-5, -2, -10])    // -2
Max([42])             // 42
Max([])  // Error: Cannot find max of empty list
```

**Use Cases:**
```dsl
// Find highest score
Max(scores)

// Get maximum temperature
Max(temperatures)

// Peak detection
Max(dataPoints)
```

## Sum

**Signature:** `(List) -> Int | Float`

Sum all numbers in a list. Returns `0` for empty list.

```dsl
Sum([1, 2, 3, 4])     // 10
Sum([1.5, 2.5, 3.0])  // 7.0
Sum([-5, 3, 2])       // 0
Sum([42])             // 42
Sum([])               // 0
```

**Use Cases:**
```dsl
// Total revenue
Sum(Map(orders, def (o) := o.amount))

// Count items (for 1s and 0s)
Sum(Map(items, def (i) := i.isActive ? 1 : 0))

// Aggregate values
Sum(values)
```

## Average

**Signature:** `(List) -> Float`

Calculate the average of numbers in a list. **Errors on empty list.**

```dsl
Average([1, 2, 3, 4])     // 2.5
Average([10, 20, 30])     // 20.0
Average([5])              // 5.0
Average([1, 2, 3, 4, 5])  // 3.0
Average([])  // Error: Cannot calculate average of empty list
```

**Use Cases:**
```dsl
// Mean score
Average(scores)

// Average temperature
Average(temperatures)

// Statistical analysis
let mean = Average(values)
let variance = Average(Map(values, def (x) := (x - mean) * (x - mean)))
```

## Round

**Signature:** `(Float) -> Int`

Round a float to the nearest integer.

```dsl
Round(3.7)   // 4
Round(3.2)   // 3
Round(3.5)   // 4
Round(-2.5)  // -2
Round(0.0)   // 0
```

**Rounding Rules:**
- `.5` and above rounds up
- Below `.5` rounds down
- Negative numbers round toward zero for `.5`

**Use Cases:**
```dsl
// Display rounded values
Map(prices, Round)

// Nearest integer
Round(calculation)
```

## Floor

**Signature:** `(Float) -> Int`

Round down to the nearest integer (toward negative infinity).

```dsl
Floor(3.7)   // 3
Floor(3.2)   // 3
Floor(-2.3)  // -3
Floor(-2.7)  // -3
Floor(5.0)   // 5
```

**Use Cases:**
```dsl
// Integer division
Floor(a / b)

// Bucket/bin assignment
Floor(value / bucketSize)

// Truncate decimals (positive numbers)
Floor(price)
```

## Ceil

**Signature:** `(Float) -> Int`

Round up to the nearest integer (toward positive infinity).

```dsl
Ceil(3.2)   // 4
Ceil(3.7)   // 4
Ceil(-2.3)  // -2
Ceil(-2.7)  // -2
Ceil(5.0)   // 5
```

**Use Cases:**
```dsl
// Ensure minimum value
Ceil(result)

// Pages needed
Ceil(totalItems / itemsPerPage)

// Always round up
Ceil(estimate)
```

## Type Conversion

### ToInt

**Signature:** `(String | Float) -> Int`

Convert a string or float to an integer.

```dsl
ToInt("123")   // 123
ToInt("42")    // 42
ToInt(3.7)     // 3 (truncates)
ToInt(3.2)     // 3
ToInt("-10")   // -10
```

**Error Cases:**
```dsl
ToInt("abc")  // Error: Cannot convert to Int
ToInt("12.5") // Error: Cannot convert to Int (use ToFloat first)
```

### ToFloat

**Signature:** `(String | Int) -> Float`

Convert a string or integer to a float.

```dsl
ToFloat("3.14")  // 3.14
ToFloat("42")    // 42.0
ToFloat(42)      // 42.0
ToFloat("2.5")   // 2.5
```

**Error Cases:**
```dsl
ToFloat("abc")  // Error: Cannot convert to Float
```

### ToString

**Signature:** `(Any) -> String`

Convert any value to a string representation.

```dsl
ToString(42)        // "42"
ToString(3.14)      // "3.14"
ToString(true)      // "true"
ToString([1, 2, 3]) // "[1, 2, 3]"
```

## Practical Examples

### Example 1: Statistical Summary

```dsl
def stats(numbers) := {
  min: Min(numbers),
  max: Max(numbers),
  sum: Sum(numbers),
  avg: Average(numbers),
  count: Length(numbers)
}

stats([85, 92, 78, 95, 88])
// { min: 78, max: 95, sum: 438, avg: 87.6, count: 5 }
```

### Example 2: Normalize Values

```dsl
def normalize(values) :=
  let minVal = Min(values)
  let maxVal = Max(values)
  let range = maxVal - minVal
  Map(values, def (v) := (v - minVal) / range)

normalize([10, 20, 30, 40, 50])
// [0.0, 0.25, 0.5, 0.75, 1.0]
```

### Example 3: Grade Calculator

```dsl
def calculateGrade(scores) :=
  let avg = Average(scores)
  avg >= 90 ? "A" :
  avg >= 80 ? "B" :
  avg >= 70 ? "C" :
  avg >= 60 ? "D" : "F"

calculateGrade([85, 92, 88, 95])  // "A"
```

### Example 4: Pagination Page Count

```dsl
def pageCount(totalItems, itemsPerPage) :=
  Ceil(totalItems / itemsPerPage)

pageCount(100, 10)  // 10
pageCount(95, 10)   // 10
pageCount(101, 10)  // 11
```

### Example 5: Price Calculation

```dsl
def calculateTotal(items) :=
  items
    |> Map(_, def (item) := item.price * item.quantity)
    |> Sum(_)
    |> Round(_)

let cart = [
  {price: 10.99, quantity: 2},
  {price: 5.49, quantity: 3},
  {price: 7.25, quantity: 1}
]

calculateTotal(cart)  // 46
```

### Example 6: Temperature Conversion

```dsl
def celsiusToFahrenheit(celsius) :=
  Round(celsius * 9 / 5 + 32)

def fahrenheitToCelsius(fahrenheit) :=
  Round((fahrenheit - 32) * 5 / 9)

celsiusToFahrenheit(25)   // 77
fahrenheitToCelsius(77)   // 25
```

### Example 7: Range and Variance

```dsl
def range(values) :=
  Max(values) - Min(values)

def variance(values) :=
  let mean = Average(values)
  let squaredDiffs = Map(values, def (x) := (x - mean) * (x - mean))
  Average(squaredDiffs)

let data = [10, 20, 30, 40, 50]
range(data)     // 40
variance(data)  // 200.0
```

### Example 8: Moving Average

```dsl
def movingAverage(values, windowSize) :=
  Range(0, Length(values) - windowSize + 1)
    |> Map(_, def (i) :=
         Average(Take(Skip(values, i), windowSize)))

movingAverage([1, 2, 3, 4, 5, 6, 7], 3)
// [2.0, 3.0, 4.0, 5.0, 6.0]
```

## Combining Math Operations

### Data Analysis Pipeline

```dsl
let scores = [85, 92, 78, 95, 88, 76, 91, 89]

let analysis = {
  total: Sum(scores),
  average: Average(scores),
  highest: Max(scores),
  lowest: Min(scores),
  range: Max(scores) - Min(scores),
  aboveAvg: Count(scores, def (s) := s > Average(scores)),
  roundedAvg: Round(Average(scores))
}
```

### Financial Calculations

```dsl
def calculateInvoice(lineItems) :=
  let subtotal = Sum(Map(lineItems, def (item) := item.total))
  let tax = Round(subtotal * 0.08 * 100) / 100  // 8% tax, 2 decimals
  let total = subtotal + tax
  {
    subtotal: subtotal,
    tax: tax,
    total: total
  }
```

## Best Practices

### 1. Check for Empty Lists

Always validate before using Min, Max, or Average:

```dsl
// Good
Length(values) > 0 ? Average(values) : 0

// Bad (may error)
Average(values)
```

### 2. Handle Division by Zero

```dsl
// Good
let divisor = Max(values)
divisor != 0 ? Sum(values) / divisor : 0

// Bad (may error or produce Infinity)
Sum(values) / Max(values)
```

### 3. Use Appropriate Rounding

Choose the right rounding function for your use case:

```dsl
// Round: nearest integer
Round(3.5)  // 4

// Floor: always down
Floor(3.9)  // 3

// Ceil: always up
Ceil(3.1)   // 4
```

### 4. Precision in Financial Calculations

```dsl
// Good: Round to 2 decimals
Round(price * 100) / 100

// Better: Use integer cents
ToInt(price * 100)  // Store as cents
```

## Performance Tips

### 1. Cache Expensive Calculations

```dsl
// Good: Calculate once
let avg = Average(values)
let results = Map(values, def (v) := v - avg)

// Bad: Calculate repeatedly
let results = Map(values, def (v) := v - Average(values))
```

### 2. Combine Operations

```dsl
// Better: Single pass
let stats = Reduce(values, {sum: 0, count: 0}, def (acc, val) := {
  sum: acc.sum + val,
  count: acc.count + 1
})

// Worse: Multiple passes
let sum = Sum(values)
let count = Length(values)
```

## Next Steps

- [String Operations](./strings.md) - Text manipulation
- [List Operations](./lists.md) - Collection operations
- [Functional Programming](./functional.md) - Higher-order functions
- [Utilities](./utilities.md) - Additional tools
