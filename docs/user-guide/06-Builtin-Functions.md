# Builtin Functions Reference

This document provides a comprehensive reference for all builtin functions available in the DSL.

## Table of Contents

1. [String Functions](#string-functions)
2. [List Functions](#list-functions)
3. [Math Functions](#math-functions)
4. [Type Conversion Functions](#type-conversion-functions)
5. [Functional Programming Functions](#functional-programming-functions)
6. [Utility Functions](#utility-functions)
7. [LLM Functions](#llm-functions)
8. [SQL Functions](#sql-functions)
9. [Rendering Functions](#rendering-functions)
10. [Concurrency Functions](#concurrency-functions)
11. [Logic Functions](#logic-functions)
12. [Chart Generation Functions](#chart-generation-functions)

---

## String Functions

### Upper
**Signature:** `(String) -> String`

Convert a string to uppercase.

```javascript
Upper("hello")  # "HELLO"
Upper("Hello World")  # "HELLO WORLD"
```

### Lower
**Signature:** `(String) -> String`

Convert a string to lowercase.

```javascript
Lower("HELLO")  # "hello"
Lower("Hello World")  # "hello world"
```

### Length
**Signature:** `(String | List) -> Int`

Get the length of a string or list.

```javascript
Length("hello")  # 5
Length([1, 2, 3])  # 3
Length("")  # 0
```

### Trim
**Signature:** `(String) -> String`

Remove whitespace from both ends of a string.

```javascript
Trim("  hello  ")  # "hello"
Trim("\n  world\t")  # "world"
```

### Split
**Signature:** `(String, String) -> List`

Split a string by a separator into a list of strings.

```javascript
Split("a,b,c", ",")  # ["a", "b", "c"]
Split("hello world", " ")  # ["hello", "world"]
Split("a::b::c", "::")  # ["a", "b", "c"]
```

### Replace
**Signature:** `(String, String, String) -> String`

Replace all occurrences of a pattern with a replacement string.

```javascript
Replace("hello world", "world", "DSL")  # "hello DSL"
Replace("foo bar foo", "foo", "baz")  # "baz bar baz"
```

### Contains
**Signature:** `(String, String) -> Bool`

Check if a string contains a substring.

```javascript
Contains("hello world", "world")  # true
Contains("hello", "xyz")  # false
```

### StartsWith
**Signature:** `(String, String) -> Bool`

Check if a string starts with a prefix.

```javascript
StartsWith("hello", "hel")  # true
StartsWith("hello", "lo")  # false
```

### EndsWith
**Signature:** `(String, String) -> Bool`

Check if a string ends with a suffix.

```javascript
EndsWith("hello", "lo")  # true
EndsWith("hello", "hel")  # false
```

### Join
**Signature:** `(List, String) -> String`

Join list elements into a string with a separator.

```javascript
Join(["a", "b", "c"], ", ")  # "a, b, c"
Join([1, 2, 3], "-")  # "1-2-3"
```

---

## List Functions

### Reverse
**Signature:** `(List) -> List`

Reverse the order of elements in a list.

```javascript
Reverse([1, 2, 3])  # [3, 2, 1]
Reverse(["a", "b", "c"])  # ["c", "b", "a"]
```

### Sort
**Signature:** `(List) -> List`

Sort a list in ascending order. Works with numbers and strings.

```javascript
Sort([3, 1, 4, 1, 5])  # [1, 1, 3, 4, 5]
Sort(["c", "a", "b"])  # ["a", "b", "c"]
Sort([3.5, 1.2, 2.8])  # [1.2, 2.8, 3.5]
```

### Unique
**Signature:** `(List) -> List`

Remove duplicate elements from a list, preserving order.

```javascript
Unique([1, 2, 2, 3, 1])  # [1, 2, 3]
Unique(["a", "b", "a", "c"])  # ["a", "b", "c"]
```

### Take
**Signature:** `(List, Int) -> List`

Take the first N elements from a list.

```javascript
Take([1, 2, 3, 4, 5], 3)  # [1, 2, 3]
Take(["a", "b", "c"], 2)  # ["a", "b"]
```

### Skip
**Signature:** `(List, Int) -> List`

Skip the first N elements and return the rest.

```javascript
Skip([1, 2, 3, 4, 5], 2)  # [3, 4, 5]
Skip(["a", "b", "c"], 1)  # ["b", "c"]
```

### First
**Signature:** `(List) -> Any`

Get the first element of a list. Errors on empty list.

```javascript
First([1, 2, 3])  # 1
First(["a", "b"])  # "a"
```

### Last
**Signature:** `(List) -> Any`

Get the last element of a list. Errors on empty list.

```javascript
Last([1, 2, 3])  # 3
Last(["a", "b"])  # "b"
```

### Flatten
**Signature:** `(List) -> List`

Flatten nested lists by one level.

```javascript
Flatten([[1, 2], [3, 4]])  # [1, 2, 3, 4]
Flatten([[1], [2, 3], [4, 5, 6]])  # [1, 2, 3, 4, 5, 6]
Flatten([1, [2, 3], 4])  # [1, 2, 3, 4]
```

---

## Math Functions

### Abs
**Signature:** `(Int | Float) -> Int | Float`

Calculate the absolute value of a number.

```javascript
Abs(-42)  # 42
Abs(3.14)  # 3.14
Abs(-2.5)  # 2.5
```

### Min
**Signature:** `(List) -> Int | Float`

Find the minimum value in a list of numbers.

```javascript
Min([3, 1, 4, 1, 5])  # 1
Min([2.5, 1.2, 3.7])  # 1.2
```

### Max
**Signature:** `(List) -> Int | Float`

Find the maximum value in a list of numbers.

```javascript
Max([3, 1, 4, 1, 5])  # 5
Max([2.5, 1.2, 3.7])  # 3.7
```

### Sum
**Signature:** `(List) -> Int | Float`

Sum all numbers in a list.

```javascript
Sum([1, 2, 3, 4])  # 10
Sum([1.5, 2.5, 3.0])  # 7.0
```

### Average
**Signature:** `(List) -> Float`

Calculate the average of numbers in a list.

```javascript
Average([1, 2, 3, 4])  # 2.5
Average([10, 20, 30])  # 20.0
```

### Round
**Signature:** `(Float) -> Int`

Round a float to the nearest integer.

```javascript
Round(3.7)  # 4
Round(3.2)  # 3
Round(3.5)  # 4
```

### Floor
**Signature:** `(Float) -> Int`

Round down to the nearest integer.

```javascript
Floor(3.7)  # 3
Floor(3.2)  # 3
Floor(-2.7)  # -3
```

### Ceil
**Signature:** `(Float) -> Int`

Round up to the nearest integer.

```javascript
Ceil(3.2)  # 4
Ceil(3.7)  # 4
Ceil(-2.3)  # -2
```

---

## Type Conversion Functions

### ToString
**Signature:** `(Any) -> String`

Convert any value to a string representation.

```javascript
ToString(42)  # "42"
ToString(3.14)  # "3.14"
ToString(true)  # "true"
ToString([1, 2, 3])  # "[1, 2, 3]"
```

### ToInt
**Signature:** `(String | Float) -> Int`

Convert a string or float to an integer.

```javascript
ToInt("123")  # 123
ToInt(3.7)  # 3
ToInt("42")  # 42
```

### ToFloat
**Signature:** `(String | Int) -> Float`

Convert a string or integer to a float.

```javascript
ToFloat("3.14")  # 3.14
ToFloat(42)  # 42.0
ToFloat("2.5")  # 2.5
```

---

## Functional Programming Functions

These functions accept user-defined functions as arguments, enabling powerful functional programming patterns.

### Map
**Signature:** `(List, Function) -> List`

Apply a function to each element of a list.

```javascript
def double(x) := x * 2
Map([1, 2, 3], double)  # [2, 4, 6]

def toUpper(s) := Upper(s)
Map(["hello", "world"], toUpper)  # ["HELLO", "WORLD"]
```

### Filter
**Signature:** `(List, Function) -> List`

Keep only elements that match a predicate function.

```javascript
def isEven(x) := x % 2 == 0
Filter([1, 2, 3, 4, 5], isEven)  # [2, 4]

def isLong(s) := Length(s) > 3
Filter(["hi", "hello", "hey", "world"], isLong)  # ["hello", "world"]
```

### Reduce
**Signature:** `(List, Any, Function) -> Any`

Accumulate a result by applying a function to each element with an accumulator.

```javascript
def add(a, b) := a + b
Reduce([1, 2, 3, 4], 0, add)  # 10

def multiply(a, b) := a * b
Reduce([1, 2, 3, 4], 1, multiply)  # 24

def concat(a, b) := a + "-" + b
Reduce(["a", "b", "c"], "", concat)  # "-a-b-c"
```

### Any
**Signature:** `(List, Function) -> Bool`

Check if any element matches a predicate function.

```javascript
def isPositive(x) := x > 0
Any([1, -2, 3], isPositive)  # true
Any([-1, -2, -3], isPositive)  # false
```

### All
**Signature:** `(List, Function) -> Bool`

Check if all elements match a predicate function.

```javascript
def isPositive(x) := x > 0
All([1, 2, 3], isPositive)  # true
All([1, -2, 3], isPositive)  # false
```

### Find
**Signature:** `(List, Function) -> Any`

Find the first element that matches a predicate. Returns null if not found.

```javascript
def isEven(x) := x % 2 == 0
Find([1, 3, 4, 5, 6], isEven)  # 4
Find([1, 3, 5], isEven)  # null
```

### Count
**Signature:** `(List, Function) -> Int`

Count how many elements match a predicate function.

```javascript
def isEven(x) := x % 2 == 0
Count([1, 2, 3, 4, 5, 6], isEven)  # 3

def isLong(s) := Length(s) > 4
Count(["hi", "hello", "hey", "world"], isLong)  # 2
```

---

## Utility Functions

### Zip
**Signature:** `(List, List) -> List`

Combine two lists into a list of pairs.

```javascript
Zip([1, 2, 3], ["a", "b", "c"])  # [[1, "a"], [2, "b"], [3, "c"]]
Zip([1, 2], [10, 20])  # [[1, 10], [2, 20]]
```

### Range
**Signature:** `(Int, Int) -> List`

Generate a range of integers from start (inclusive) to end (exclusive).

```javascript
Range(1, 5)  # [1, 2, 3, 4]
Range(0, 3)  # [0, 1, 2]
Range(5, 10)  # [5, 6, 7, 8, 9]
```

### Repeat
**Signature:** `(Any, Int) -> List`

Create a list with a value repeated N times.

```javascript
Repeat("x", 3)  # ["x", "x", "x"]
Repeat(0, 5)  # [0, 0, 0, 0, 0]
Repeat([1, 2], 2)  # [[1, 2], [1, 2]]
```

### Chunk
**Signature:** `(List, Int) -> List`

Split a list into chunks of a given size.

```javascript
Chunk([1, 2, 3, 4, 5], 2)  # [[1, 2], [3, 4], [5]]
Chunk(["a", "b", "c", "d"], 3)  # [["a", "b", "c"], ["d"]]
```

---

## LLM Functions

### Ask
**Signature:** `(String) -> String`

Send a prompt to an LLM and get a text response.

```javascript
Ask("What is the capital of France?")  # "Paris"
Ask("Translate 'hello' to Spanish")  # "hola"
```

**Configuration:** Requires `OPENAI_API_KEY` environment variable.

### ExtractPerson
**Signature:** `(String) -> Person`

Extract person information from text into a structured Person object.

```javascript
ExtractPerson("John Doe is 30 years old and works as a software engineer")
# { name: "John Doe", age: 30, occupation: "software engineer" }
```

**Configuration:** Requires `OPENAI_API_KEY` environment variable.

### ExtractAs
**Signature:** `(String, Type) -> Type`

Extract structured data of a given type from text.

```javascript
type Product = {
  name: String,
  price: Float,
  category: String
}

ExtractAs("iPhone 15 costs $999 in electronics", "Product")
# { name: "iPhone 15", price: 999.0, category: "electronics" }
```

**Configuration:** Requires `OPENAI_API_KEY` environment variable.

---

## SQL Functions

### SQL
**Signature:** `(String) -> Table`

Execute a SQL query using DuckDB with automatic table registration.

**Auto-registration with $variable:**
Use `$variable` syntax to automatically register DSL variables as tables:

```javascript
users = [
  {name: "Alice", age: 30},
  {name: "Bob", age: 25}
]

SQL("SELECT * FROM $users WHERE age > 26")
# Automatically registers 'users' as a table, then queries it

# Multiple tables
orders = [{user_id: 1, amount: 100}]
SQL("SELECT * FROM $users JOIN $orders ON users.id = $orders.user_id")
```

**Requirements for auto-registered tables:**
- Must be a List of Maps
- All maps must have the same keys (consistent schema)
- No nested structures (Map/List inside Map)
- Cannot be empty

**Table caching:**
Tables are cached after first registration. Subsequent queries reuse the cached table for better performance.

```javascript
users = [{name: "Alice", age: 25}]
SQL("SELECT * FROM $users")  # Registers and caches
SQL("SELECT COUNT(*) FROM $users")  # Uses cache (fast!)
```

See also: [refresh_table()](#refresh_table)

### refresh_table
**Signature:** `(String) -> Null`

Clear a cached SQL table, forcing it to be re-registered on next use.

```javascript
users = [{name: "Alice", age: 25}]
SQL("SELECT * FROM $users")  # Registers and caches

# Update data
users = [{name: "Bob", age: 30}]

# Force refresh
refresh_table("users")
SQL("SELECT * FROM $users")  # Uses fresh data
```

**Use when:**
- Data variables change and you need fresh results
- Freeing memory from large cached tables
- Debugging data issues

---

## Rendering Functions

### RenderMarkdown
**Signature:** `(String) -> Markdown`

Render a string as formatted markdown in the TUI.

```javascript
RenderMarkdown("# Hello\n\nThis is **bold** text")
# Displays formatted markdown in the output
```

---

## Concurrency Functions

### Par
**Signature:** `(...) -> List`

Execute multiple expressions in parallel and return results as a list.

```javascript
let [result1, result2, result3] = Par(
  Ask("What is 2+2?"),
  Ask("What is 3+3?"),
  Ask("What is 4+4?")
)
# All three LLM calls execute in parallel
```

---

## Logic Functions

### Not
**Signature:** `(Bool) -> Bool`

Logical NOT operation.

```javascript
Not(true)  # false
Not(false)  # true
Not(5 > 3)  # false
```

---

## Chart Generation Functions

### GenerateBarChart
**Signature:** `(List, String?) -> Image`

Generate a bar chart from data. Optional theme parameter.

```javascript
let data = [
  {label: "A", value: 10},
  {label: "B", value: 25},
  {label: "C", value: 15}
]

GenerateBarChart(data)  # Default blue theme
GenerateBarChart(data, "green")  # Green theme
```

**Available themes:** `"blue"`, `"green"`, `"red"`, `"purple"`, `"orange"`, `"dark"`

### GenerateLineChart
**Signature:** `(List, String?) -> Image`

Generate a line chart from data. Optional theme parameter.

```javascript
let data = [
  {label: "Jan", value: 10},
  {label: "Feb", value: 20},
  {label: "Mar", value: 15}
]

GenerateLineChart(data)
GenerateLineChart(data, "purple")
```

### GeneratePieChart
**Signature:** `(List, String?) -> Image`

Generate a pie chart from data. Optional theme parameter.

```javascript
let data = [
  {label: "Product A", value: 30},
  {label: "Product B", value: 45},
  {label: "Product C", value: 25}
]

GeneratePieChart(data)
GeneratePieChart(data, "dark")
```

---

## Advanced Examples

### Combining Functions

```javascript
# Data processing pipeline
let numbers = Range(1, 11)  # [1..10]
let evens = Filter(numbers, def (x) := x % 2 == 0)
let doubled = Map(evens, def (x) := x * 2)
let sum = Sum(doubled)  # Sum of doubled even numbers
```

### Text Processing

```javascript
let text = "  hello,world,foo  "
let cleaned = Trim(text)  # "hello,world,foo"
let words = Split(cleaned, ",")  # ["hello", "world", "foo"]
let upper = Map(words, Upper)  # ["HELLO", "WORLD", "FOO"]
let result = Join(upper, " | ")  # "HELLO | WORLD | FOO"
```

### Data Analysis

```javascript
let scores = [85, 92, 78, 95, 88, 76, 91]

let stats = {
  min: Min(scores),
  max: Max(scores),
  avg: Average(scores),
  count: Length(scores),
  passing: Count(scores, def (x) := x >= 80)
}
# { min: 76, max: 95, avg: 86.43, count: 7, passing: 5 }
```

---

## Notes

- All builtin functions are **dynamically sourced** from metadata, ensuring consistency between implementation and autocomplete.
- Function names are **case-insensitive** (you can use `upper`, `Upper`, or `UPPER`).
- Type errors are caught at runtime with clear error messages.
- Most list functions preserve element order unless specifically noted (like `Sort`).
- Empty list operations (like `First`, `Last`, `Min`, `Max`, `Average`) will raise errors.

For more examples, see the [Language Features](04-Language-Features.md) documentation.
