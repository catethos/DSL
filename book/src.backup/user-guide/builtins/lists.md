# List Operations

Functions for working with lists and collections.

## Reverse

**Signature:** `(List) -> List`

Reverse the order of elements in a list.

```dsl
Reverse([1, 2, 3])  // [3, 2, 1]
Reverse(["a", "b", "c"])  // ["c", "b", "a"]
Reverse([])  // []
```

**Use Cases:**
```dsl
// Reverse chronological order
Reverse(sortedDates)

// Process items in reverse
Map(Reverse(items), processItem)
```

## Sort

**Signature:** `(List) -> List`

Sort a list in ascending order. Works with numbers and strings.

```dsl
Sort([3, 1, 4, 1, 5])  // [1, 1, 3, 4, 5]
Sort(["c", "a", "b"])  // ["a", "b", "c"]
Sort([3.5, 1.2, 2.8])  // [1.2, 2.8, 3.5]
```

**Sorting Behavior:**
- Numbers: Ascending numerical order
- Strings: Lexicographic (alphabetical) order
- Mixed types: Type error

```dsl
// Descending order: Sort then Reverse
Reverse(Sort([3, 1, 4]))  // [4, 3, 1]
```

## Unique

**Signature:** `(List) -> List`

Remove duplicate elements from a list, preserving order of first occurrence.

```dsl
Unique([1, 2, 2, 3, 1])  // [1, 2, 3]
Unique(["a", "b", "a", "c"])  // ["a", "b", "c"]
Unique([1, 1, 1, 1])  // [1]
Unique([])  // []
```

**Use Cases:**
```dsl
// Get unique tags
Unique(allTags)

// Remove duplicate IDs
Unique(userIds)

// Count unique items
Length(Unique(items))
```

## Take

**Signature:** `(List, Int) -> List`

Take the first N elements from a list.

```dsl
Take([1, 2, 3, 4, 5], 3)  // [1, 2, 3]
Take(["a", "b", "c"], 2)  // ["a", "b"]
Take([1, 2], 5)  // [1, 2] (takes all if N > length)
Take([], 3)  // []
```

**Use Cases:**
```dsl
// Top N results
Take(Sort(scores), 5)

// Preview first items
Take(largeList, 10)

// Pagination
Take(Skip(items, pageSize * pageNum), pageSize)
```

## Skip

**Signature:** `(List, Int) -> List`

Skip the first N elements and return the rest.

```dsl
Skip([1, 2, 3, 4, 5], 2)  // [3, 4, 5]
Skip(["a", "b", "c"], 1)  // ["b", "c"]
Skip([1, 2], 5)  // [] (returns empty if N >= length)
Skip([], 3)  // []
```

**Use Cases:**
```dsl
// Remove header row
Skip(csvLines, 1)

// Pagination offset
Skip(allItems, page * pageSize)

// Drop initial elements
Skip(sequence, startIndex)
```

## First

**Signature:** `(List) -> Any`

Get the first element of a list. **Errors on empty list.**

```dsl
First([1, 2, 3])  // 1
First(["a", "b"])  // "a"
First([42])  // 42
First([])  // Error: Cannot get first element of empty list
```

**Safe Alternative:**
```dsl
// Check before accessing
Length(list) > 0 ? First(list) : defaultValue
```

## Last

**Signature:** `(List) -> Any`

Get the last element of a list. **Errors on empty list.**

```dsl
Last([1, 2, 3])  // 3
Last(["a", "b"])  // "b"
Last([42])  // 42
Last([])  // Error: Cannot get last element of empty list
```

**Safe Alternative:**
```dsl
// Check before accessing
Length(list) > 0 ? Last(list) : defaultValue
```

## Flatten

**Signature:** `(List) -> List`

Flatten nested lists by one level.

```dsl
Flatten([[1, 2], [3, 4]])  // [1, 2, 3, 4]
Flatten([[1], [2, 3], [4, 5, 6]])  // [1, 2, 3, 4, 5, 6]
Flatten([1, [2, 3], 4])  // [1, 2, 3, 4]
Flatten([[]])  // []
```

**Note:** Only flattens one level:
```dsl
Flatten([[[1, 2]], [[3, 4]]])  // [[1, 2], [3, 4]]

// Flatten multiple levels:
Flatten(Flatten([[[1, 2]], [[3, 4]]]))  // [1, 2, 3, 4]
```

**Use Cases:**
```dsl
// Combine results from parallel operations
Flatten(par(GetItems(1), GetItems(2), GetItems(3)))

// Merge sublists
Flatten(Map(categories, getCategoryItems))
```

## Practical Examples

### Example 1: Top N Sorted Items

```dsl
def topN(items, n) :=
  items
    |> Sort(_)
    |> Reverse(_)
    |> Take(_, n)

topN([85, 92, 78, 95, 88], 3)  // [95, 92, 88]
```

### Example 2: Pagination

```dsl
def paginate(items, pageNum, pageSize) :=
  items
    |> Skip(_, pageNum * pageSize)
    |> Take(_, pageSize)

let allItems = Range(1, 101)  // [1..100]
paginate(allItems, 0, 10)  // [1..10]
paginate(allItems, 1, 10)  // [11..20]
paginate(allItems, 2, 10)  // [21..30]
```

### Example 3: Unique Sorted List

```dsl
def uniqueSorted(items) :=
  items
    |> Unique(_)
    |> Sort(_)

uniqueSorted([3, 1, 4, 1, 5, 9, 2, 6, 5])
// [1, 2, 3, 4, 5, 6, 9]
```

### Example 4: Remove Empty Strings

```dsl
def removeEmpty(strings) :=
  Filter(strings, def (s) := Length(Trim(s)) > 0)

removeEmpty(["hello", "", "  ", "world"])
// ["hello", "world"]
```

### Example 5: Get Unique Tags from Articles

```dsl
def getAllTags(articles) :=
  articles
    |> Map(_, def (article) := article.tags)
    |> Flatten(_)
    |> Unique(_)
    |> Sort(_)

let articles = [
  {title: "Article 1", tags: ["tech", "ai"]},
  {title: "Article 2", tags: ["tech", "web"]},
  {title: "Article 3", tags: ["ai", "ml"]}
]

getAllTags(articles)
// ["ai", "ml", "tech", "web"]
```

### Example 6: Find Most Common Element

```dsl
def mostCommon(items) :=
  items
    |> Unique(_)
    |> Map(_, def (item) := {
         value: item,
         count: Count(items, def (x) := x == item)
       })
    |> Sort(_)  // Sort by count (requires custom sort)
    |> Last(_)
    |> _.value

// Note: This is simplified; actual implementation
// would need custom sorting by count
```

### Example 7: Sliding Window

```dsl
def windows(items, size) :=
  Range(0, Length(items) - size + 1)
    |> Map(_, def (i) := Take(Skip(items, i), size))

windows([1, 2, 3, 4, 5], 3)
// [[1, 2, 3], [2, 3, 4], [3, 4, 5]]
```

### Example 8: Interleave Two Lists

```dsl
def interleave(list1, list2) :=
  Zip(list1, list2)
    |> Flatten(_)

interleave([1, 3, 5], [2, 4, 6])
// [1, 2, 3, 4, 5, 6]
```

## Combining List Operations

### Chain Transformations

```dsl
[5, 2, 8, 2, 9, 1, 5]
  |> Unique(_)      // [5, 2, 8, 9, 1]
  |> Sort(_)        // [1, 2, 5, 8, 9]
  |> Reverse(_)     // [9, 8, 5, 2, 1]
  |> Take(_, 3)     // [9, 8, 5]
```

### With Higher-Order Functions

```dsl
[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
  |> Filter(_, def (x) := x % 2 == 0)  // [2, 4, 6, 8, 10]
  |> Map(_, def (x) := x * x)          // [4, 16, 36, 64, 100]
  |> Take(_, 3)                         // [4, 16, 36]
```

## Performance Tips

### 1. Order Operations Efficiently

```dsl
// Better: Filter first (reduces data size)
items
  |> Filter(_, predicate)
  |> Sort(_)
  |> Take(_, 10)

// Worse: Sort all data first
items
  |> Sort(_)
  |> Filter(_, predicate)
  |> Take(_, 10)
```

### 2. Use Take Early

```dsl
// Better: Take early to limit processing
largeList
  |> Take(_, 100)
  |> Map(_, expensiveOperation)

// Worse: Process everything
largeList
  |> Map(_, expensiveOperation)
  |> Take(_, 100)
```

### 3. Avoid Repeated Sorts

```dsl
// Better: Sort once, reuse
let sorted = Sort(items)
let top5 = Take(sorted, 5)
let bottom5 = Take(Reverse(sorted), 5)

// Worse: Sort multiple times
let top5 = Take(Sort(items), 5)
let bottom5 = Take(Reverse(Sort(items)), 5)
```

## Best Practices

### 1. Check for Empty Lists

Always check before using `First` or `Last`:

```dsl
// Good
Length(list) > 0 ? First(list) : defaultValue

// Bad (may error)
First(list)
```

### 2. Preserve Original Data

List operations return new lists; originals are unchanged:

```dsl
let original = [3, 1, 2]
let sorted = Sort(original)

// original is still [3, 1, 2]
// sorted is [1, 2, 3]
```

### 3. Use Meaningful Names

```dsl
// Good
let topScores = Take(Sort(Reverse(scores)), 5)

// Better
def getTopScores(scores, n) :=
  scores
    |> Sort(_)
    |> Reverse(_)
    |> Take(_, n)

let topScores = getTopScores(scores, 5)
```

## Next Steps

- [String Operations](./strings.md) - Text manipulation
- [Math Operations](./math.md) - Numerical operations
- [Functional Programming](./functional.md) - Map, Filter, Reduce
- [Utilities](./utilities.md) - Additional tools
