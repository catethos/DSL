# List Operations

Functions for working with lists and collections.

## Reverse

**Signature:** `(List) -> List`

Reverse the order of elements in a list.

```dsl
Reverse([1, 2, 3])
Reverse(["a", "b", "c"])
Reverse([])
```

**Use Cases:**
```dsl
Reverse(sortedDates)

Map(Reverse(items), processItem)
```

## Sort

**Signature:** `(List) -> List`

Sort a list in ascending order. Works with numbers and strings.

```dsl
Sort([3, 1, 4, 1, 5])
Sort(["c", "a", "b"])
Sort([3.5, 1.2, 2.8])
```

**Sorting Behavior:**
- Numbers: Ascending numerical order
- Strings: Lexicographic (alphabetical) order
- Mixed types: Type error

```dsl
Reverse(Sort([3, 1, 4]))
```

## Unique

**Signature:** `(List) -> List`

Remove duplicate elements from a list, preserving order of first occurrence.

```dsl
Unique([1, 2, 2, 3, 1])
Unique(["a", "b", "a", "c"])
Unique([1, 1, 1, 1])
Unique([])
```

**Use Cases:**
```dsl
Unique(allTags)

Unique(userIds)

Length(Unique(items))
```

## Take

**Signature:** `(List, Int) -> List`

Take the first N elements from a list.

```dsl
Take([1, 2, 3, 4, 5], 3)
Take(["a", "b", "c"], 2)
Take([1, 2], 5)
Take([], 3)
```

**Use Cases:**
```dsl
Take(Sort(scores), 5)

Take(largeList, 10)

Take(Skip(items, pageSize * pageNum), pageSize)
```

## Skip

**Signature:** `(List, Int) -> List`

Skip the first N elements and return the rest.

```dsl
Skip([1, 2, 3, 4, 5], 2)
Skip(["a", "b", "c"], 1)
Skip([1, 2], 5)
Skip([], 3)
```

**Use Cases:**
```dsl
Skip(csvLines, 1)

Skip(allItems, page * pageSize)

Skip(sequence, startIndex)
```

## First

**Signature:** `(List) -> Any`

Get the first element of a list. **Errors on empty list.**

```dsl
First([1, 2, 3])
First(["a", "b"])
First([42])
First([])
```

**Safe Alternative:**
```dsl
Length(list) > 0 ? First(list) : defaultValue
```

## Last

**Signature:** `(List) -> Any`

Get the last element of a list. **Errors on empty list.**

```dsl
Last([1, 2, 3])
Last(["a", "b"])
Last([42])
Last([])
```

**Safe Alternative:**
```dsl
Length(list) > 0 ? Last(list) : defaultValue
```

## Flatten

**Signature:** `(List) -> List`

Flatten nested lists by one level.

```dsl
Flatten([[1, 2], [3, 4]])
Flatten([[1], [2, 3], [4, 5, 6]])
Flatten([1, [2, 3], 4])
Flatten([[]])
```

**Note:** Only flattens one level:
```dsl
Flatten([[[1, 2]], [[3, 4]]])

Flatten(Flatten([[[1, 2]], [[3, 4]]]))
```

**Use Cases:**
```dsl
Flatten(par(GetItems(1), GetItems(2), GetItems(3)))

Flatten(Map(categories, getCategoryItems))
```

## Practical Examples

### Example 1: Top N Sorted Items

```dsl
def topN(items, n) { items }
    |> Sort(_)
    |> Reverse(_)
    |> Take(_, n)

topN([85, 92, 78, 95, 88], 3)
```

### Example 2: Pagination

```dsl
def paginate(items, pageNum, pageSize) { items }
    |> Skip(_, pageNum * pageSize)
    |> Take(_, pageSize)

let allItems = Range(1, 101)
paginate(allItems, 0, 10)
paginate(allItems, 1, 10)
paginate(allItems, 2, 10)
```

### Example 3: Unique Sorted List

```dsl
def uniqueSorted(items) { items }
    |> Unique(_)
    |> Sort(_)

uniqueSorted([3, 1, 4, 1, 5, 9, 2, 6, 5])
```

### Example 4: Remove Empty Strings

```dsl
def removeEmpty(strings) { filter(strings, fn s => Length(Trim(s end)) > 0) }

removeEmpty(["hello", "", "  ", "world"])
```

### Example 5: Get Unique Tags from Articles

```dsl
def getAllTags(articles) { articles }
    |> map(_, fn article => article.tags end)
    |> Flatten(_)
    |> Unique(_)
    |> Sort(_)

let articles = [
  {title: "Article 1", tags: ["tech", "ai"]},
  {title: "Article 2", tags: ["tech", "web"]},
  {title: "Article 3", tags: ["ai", "ml"]}
]

getAllTags(articles)
```

### Example 6: Find Most Common Element

```dsl
def mostCommon(items) { items }
    |> Unique(_)
    |> map(_, fn item => {
         value: item,
         count: Count(items, fn x => x == item end)
       })
    |> Sort(_)
    |> Last(_)
    |> _.value

```

### Example 7: Sliding Window

```dsl
def windows(items, size) { Range(0, Length(items) - size + 1) }
    |> map(_, fn i => Take(Skip(items, i end), size))

windows([1, 2, 3, 4, 5], 3)
```

### Example 8: Interleave Two Lists

```dsl
def interleave(list1, list2) { Zip(list1, list2) }
    |> Flatten(_)

interleave([1, 3, 5], [2, 4, 6])
```

## Combining List Operations

### Chain Transformations

```dsl
[5, 2, 8, 2, 9, 1, 5]
  |> Unique(_)
  |> Sort(_)
  |> Reverse(_)
  |> Take(_, 3)
```

### With Higher-Order Functions

```dsl
[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
  |> filter(_, fn x => x % 2 == 0 end)
  |> map(_, fn x => x * x end)
  |> Take(_, 3)
```

## Performance Tips

### 1. Order Operations Efficiently

```dsl
items
  |> Filter(_, predicate)
  |> Sort(_)
  |> Take(_, 10)

items
  |> Sort(_)
  |> Filter(_, predicate)
  |> Take(_, 10)
```

### 2. Use Take Early

```dsl
largeList
  |> Take(_, 100)
  |> Map(_, expensiveOperation)

largeList
  |> Map(_, expensiveOperation)
  |> Take(_, 100)
```

### 3. Avoid Repeated Sorts

```dsl
def main() {
    let sorted = Sort(items)
    let top5 = Take(sorted, 5)
    let bottom5 = Take(Reverse(sorted), 5)

    let top5 = Take(Sort(items), 5)
    let bottom5 = Take(Reverse(Sort(items)), 5)
}
main()
```

## Best Practices

### 1. Check for Empty Lists

Always check before using `First` or `Last`:

```dsl
Length(list) > 0 ? First(list) : defaultValue

First(list)
```

### 2. Preserve Original Data

List operations return new lists; originals are unchanged:

```dsl
def main() {
    let original = [3, 1, 2]
    let sorted = Sort(original)

}
main()
```

### 3. Use Meaningful Names

```dsl
let topScores = Take(Sort(Reverse(scores)), 5)

def getTopScores(scores, n) { scores }
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
