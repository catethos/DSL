# Math Operations

Mathematical functions for numerical calculations and aggregations.

## Abs

**Signature:** `(Int | Float) -> Int | Float`

Calculate the absolute value of a number.

```dsl
Abs(-42)
Abs(42)
Abs(-3.14)
Abs(0)
```

**Use Cases:**
```dsl
Abs(x1 - x2)

Abs(expected - actual)

Map(numbers, Abs)
```

## Min

**Signature:** `(List) -> Int | Float`

Find the minimum value in a list of numbers. **Errors on empty list.**

```dsl
Min([3, 1, 4, 1, 5])
Min([2.5, 1.2, 3.7])
Min([-5, -2, -10])
Min([42])
Min([])
```

**Use Cases:**
```dsl
Min(scores)

Min(map(products, fn p => p.price end))

let range = Max(values) - Min(values)
```

## Max

**Signature:** `(List) -> Int | Float`

Find the maximum value in a list of numbers. **Errors on empty list.**

```dsl
Max([3, 1, 4, 1, 5])
Max([2.5, 1.2, 3.7])
Max([-5, -2, -10])
Max([42])
Max([])
```

**Use Cases:**
```dsl
Max(scores)

Max(temperatures)

Max(dataPoints)
```

## Sum

**Signature:** `(List) -> Int | Float`

Sum all numbers in a list. Returns `0` for empty list.

```dsl
Sum([1, 2, 3, 4])
Sum([1.5, 2.5, 3.0])
Sum([-5, 3, 2])
Sum([42])
Sum([])
```

**Use Cases:**
```dsl
Sum(map(orders, fn o => o.amount end))

Sum(map(items, fn i => i.isActive ? 1 : 0 end))

Sum(values)
```

## Average

**Signature:** `(List) -> Float`

Calculate the average of numbers in a list. **Errors on empty list.**

```dsl
Average([1, 2, 3, 4])
Average([10, 20, 30])
Average([5])
Average([1, 2, 3, 4, 5])
Average([])
```

**Use Cases:**
```dsl
def main() {
    Average(scores)

    Average(temperatures)

    let mean = Average(values)
    let variance = Average(map(values, fn x => (x - mean end) * (x - mean)))
}
main()
```

## Round

**Signature:** `(Float) -> Int`

Round a float to the nearest integer.

```dsl
Round(3.7)
Round(3.2)
Round(3.5)
Round(-2.5)
Round(0.0)
```

**Rounding Rules:**
- `.5` and above rounds up
- Below `.5` rounds down
- Negative numbers round toward zero for `.5`

**Use Cases:**
```dsl
Map(prices, Round)

Round(calculation)
```

## Floor

**Signature:** `(Float) -> Int`

Round down to the nearest integer (toward negative infinity).

```dsl
Floor(3.7)
Floor(3.2)
Floor(-2.3)
Floor(-2.7)
Floor(5.0)
```

**Use Cases:**
```dsl
Floor(a / b)

Floor(value / bucketSize)

Floor(price)
```

## Ceil

**Signature:** `(Float) -> Int`

Round up to the nearest integer (toward positive infinity).

```dsl
Ceil(3.2)
Ceil(3.7)
Ceil(-2.3)
Ceil(-2.7)
Ceil(5.0)
```

**Use Cases:**
```dsl
Ceil(result)

Ceil(totalItems / itemsPerPage)

Ceil(estimate)
```

## Type Conversion

### ToInt

**Signature:** `(String | Float) -> Int`

Convert a string or float to an integer.

```dsl
ToInt("123")
ToInt("42")
ToInt(3.7)
ToInt(3.2)
ToInt("-10")
```

**Error Cases:**
```dsl
ToInt("abc")
ToInt("12.5")
```

### ToFloat

**Signature:** `(String | Int) -> Float`

Convert a string or integer to a float.

```dsl
ToFloat("3.14")
ToFloat("42")
ToFloat(42)
ToFloat("2.5")
```

**Error Cases:**
```dsl
ToFloat("abc")
```

### ToString

**Signature:** `(Any) -> String`

Convert any value to a string representation.

```dsl
ToString(42)
ToString(3.14)
ToString(true)
ToString([1, 2, 3])
```

## Practical Examples

### Example 1: Statistical Summary

```dsl
def stats(numbers) { { }
  min: Min(numbers),
  max: Max(numbers),
  sum: Sum(numbers),
  avg: Average(numbers),
  count: Length(numbers)
}

stats([85, 92, 78, 95, 88])
```

### Example 2: Normalize Values

```dsl
def normalize(values) { let minVal = Min(values) }
  let maxVal = Max(values)
  let range = maxVal - minVal
  map(values, fn v => (v - minVal end) / range)

normalize([10, 20, 30, 40, 50])
```

### Example 3: Grade Calculator

```dsl
def calculateGrade(scores) { let avg = Average(scores) }
  avg >= 90 ? "A" :
  avg >= 80 ? "B" :
  avg >= 70 ? "C" :
  avg >= 60 ? "D" : "F"

calculateGrade([85, 92, 88, 95])
```

### Example 4: Pagination Page Count

```dsl
def pageCount(totalItems, itemsPerPage) { Ceil(totalItems / itemsPerPage) }

pageCount(100, 10)
pageCount(95, 10)
pageCount(101, 10)
```

### Example 5: Price Calculation

```dsl
def calculateTotal(items) { items }
    |> map(_, fn item => item.price * item.quantity end)
    |> Sum(_)
    |> Round(_)

let cart = [
  {price: 10.99, quantity: 2},
  {price: 5.49, quantity: 3},
  {price: 7.25, quantity: 1}
]

calculateTotal(cart)
```

### Example 6: Temperature Conversion

```dsl
def celsiusToFahrenheit(celsius) { Round(celsius * 9 / 5 + 32) }

def fahrenheitToCelsius(fahrenheit) { Round((fahrenheit - 32) * 5 / 9) }

celsiusToFahrenheit(25)
fahrenheitToCelsius(77)
```

### Example 7: Range and Variance

```dsl
def range(values) { Max(values) - Min(values) }

def variance(values) { let mean = Average(values) }
  let squaredDiffs = map(values, fn x => (x - mean end) * (x - mean))
  Average(squaredDiffs)

let data = [10, 20, 30, 40, 50]
range(data)
variance(data)
```

### Example 8: Moving Average

```dsl
def movingAverage(values, windowSize) { Range(0, Length(values) - windowSize + 1) }
    |> map(_, fn i => Average(Take(Skip(values, i end), windowSize)))

movingAverage([1, 2, 3, 4, 5, 6, 7], 3)
```

## Combining Math Operations

### Data Analysis Pipeline

```dsl
def main() {
    let scores = [85, 92, 78, 95, 88, 76, 91, 89]

    let analysis = {
      total: Sum(scores),
      average: Average(scores),
      highest: Max(scores),
      lowest: Min(scores),
      range: Max(scores) - Min(scores),
      aboveAvg: Count(scores, fn s => s > Average(scores end)),
      roundedAvg: Round(Average(scores))
    }
}
main()
```

### Financial Calculations

```dsl
def calculateInvoice(lineItems) { let subtotal = Sum(map(lineItems, fn item => item.total end)) }
  let tax = Round(subtotal * 0.08 * 100) / 100
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
Length(values) > 0 ? Average(values) : 0

Average(values)
```

### 2. Handle Division by Zero

```dsl
let divisor = Max(values)
divisor != 0 ? Sum(values) / divisor : 0

Sum(values) / Max(values)
```

### 3. Use Appropriate Rounding

Choose the right rounding function for your use case:

```dsl
Round(3.5)

Floor(3.9)

Ceil(3.1)
```

### 4. Precision in Financial Calculations

```dsl
Round(price * 100) / 100

ToInt(price * 100)
```

## Performance Tips

### 1. Cache Expensive Calculations

```dsl
def main() {
    let avg = Average(values)
    let results = map(values, fn v => v - avg end)

    let results = map(values, fn v => v - Average(values end))
}
main()
```

### 2. Combine Operations

```dsl
let stats = Reduce(values, {sum: 0, count: 0}, def (acc, val) := {
  sum: acc.sum + val,
  count: acc.count + 1
})

let sum = Sum(values)
let count = Length(values)
```

## Next Steps

- [String Operations](./strings.md) - Text manipulation
- [List Operations](./lists.md) - Collection operations
- [Functional Programming](./functional.md) - Higher-order functions
- [Utilities](./utilities.md) - Additional tools
