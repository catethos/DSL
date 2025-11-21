# Your First DSL Program

Let's build a complete workflow from scratch. This tutorial walks you through creating a simple data processing pipeline.

```admonish warning "Note: Comments Not Fully Supported"
The examples below use `#` and `//` for documentation purposes. Comments are **not yet fully implemented** and may cause errors. When writing actual DSL code, avoid using comments until this feature is completed.
```

## The Goal

We'll create a program that:
1. Creates a list of words
2. Converts them to uppercase
3. Joins them with a delimiter
4. Gets the length of the result

## Method 1: Step-by-Step in REPL

Press **F1** to enter REPL mode, then type:

### Step 1: Create Data

```dsl
flow> ["hello", "world", "from", "dsl"] as words
✓ Bound 'words' to ["hello", "world", "from", "dsl"] : List
```

### Step 2: Transform Data

```dsl
flow> Map(words, Upper) as uppercase
✓ Bound 'uppercase' to ["HELLO", "WORLD", "FROM", "DSL"] : List
```

### Step 3: Join Results

```dsl
flow> Join(uppercase, " ") as message
✓ Bound 'message' to "HELLO WORLD FROM DSL" : String
```

### Step 4: Get Length

```dsl
flow> Length(message)
✓ 23 : Int
```

Success! You've created your first multi-step workflow.

## Method 2: Using the Pipeline Operator

The pipeline operator `|>` makes this more elegant:

```dsl
flow> ["hello", "world", "from", "dsl"]
    |> Map(_, Upper)
    |> Join(_, " ")
    |> Length(_)
✓ 23 : Int
```

Much better! The data flows left-to-right through transformations.

## Method 3: Using Workspace Mode

For longer programs, use Workspace mode (**F2**):

1. Press **F2** to enter Workspace mode
2. In the Editor pane, write:

```dsl
def main() {
    let words = ["hello", "world", "from", "dsl"]
    let uppercase = Map(words, Upper)
    let message = Join(uppercase, " ")
    let length = Length(message)

    message
    length
}
main()
```

3. Press **Ctrl+R** to run the program
4. Switch to REPL pane (**Tab**) to see results

## Adding Functions

Let's create a reusable function:

```dsl
def ProcessWords(wordList) {
    wordList
    |> Map(_, Upper)
    |> Join(_, " ")
}

let result = ProcessWords(["hello", "world"])
result
```

```admonish tip
Functions make your code reusable and easier to understand.
```

## Adding Types

For structured data, define custom types:

```dsl
class Message {
    content: String
    length: Int
}

let msg = Message {
    content: "HELLO WORLD",
    length: 11
}

msg.content
msg.length
```

## Complete Example: Data Analysis

Here's a more practical example:

```dsl
def main() {
    class Person {
        name: String
        age: Int
        city: String
    }

    let people = [
        Person { name: "Alice", age: 30, city: "NYC" },
        Person { name: "Bob", age: 25, city: "SF" },
        Person { name: "Charlie", age: 35, city: "NYC" }
    ]

    let nycPeople = filter(people, fn p => p.city == "NYC" end)

    let names = map(nycPeople, fn p => p.name end)

    let result = Join(names, " and ")
    result
}
main()
```

```admonish note
This example uses higher-order functions like `Filter` and `Map` with anonymous functions.
```

## Common Workflows

### Workflow 1: String Processing

```dsl
"hello world"
|> Upper(_)
|> Split(_, " ")
|> map(_, fn w => w + "!" end)
|> Join(_, " ")
```

### Workflow 2: List Operations

```dsl
[1, 2, 3, 4, 5]
|> filter(_, fn n => n > 2 end)
|> map(_, fn n => n * 2 end)
|> Sum(_)
```

### Workflow 3: Conditional Logic

```dsl
def main() {
    let age = 25
    let status = age >= 18 ? "adult" : "minor"
    status
}
main()
```

## Saving Your Work

To save your program:

1. In Workspace mode, write your code in the Editor
2. Press **Ctrl+S**
3. Enter a filename: `my-program.dsl`

To load it later:

```dsl
flow> :load my-program.dsl
✓ Loaded my-program.dsl
```

## Debugging Tips

### Check Variables

```dsl
flow> :vars
✓ Variables:
  words = ["hello", "world", "from", "dsl"] : List
  uppercase = ["HELLO", "WORLD", "FROM", "DSL"] : List
  message = "HELLO WORLD FROM DSL" : String
```

### Check Types

```dsl
flow> :types
✓ Type definitions:
  Message { content: String, length: Int }
  Person { name: String, age: Int, city: String }
```

### Output Variables for Debugging

```dsl
def main() {
    let x = 10
    "x = ${x}"
    let y = x * 2
    "y = ${y}"
}
main()
```

## Common Mistakes

### Mistake 1: Forgetting Variable Binding

❌ **Don't do this:**
```dsl
["hello", "world"]
Map(_, Upper)
```

✅ **Do this instead:**
```dsl
["hello", "world"] |> Map(_, Upper)
["hello", "world"] as words
Map(words, Upper)
```

### Mistake 2: Wrong Function Arguments

❌ **Don't do this:**
```dsl
Join(" ", ["a", "b"])
```

✅ **Do this instead:**
```dsl
Join(["a", "b"], " ")
```

### Mistake 3: Missing Pipeline

❌ **Don't do this:**
```dsl
"hello" Upper
```

✅ **Do this instead:**
```dsl
"hello" |> Upper(_)
Upper("hello")
```

## What's Next?

You've created your first DSL programs! Continue learning:

- [REPL Tour](repl-tour.md) - Master the REPL
- [Language Syntax](../user-guide/language/syntax.md) - Complete syntax reference
- [Built-in Functions](../user-guide/builtins/README.md) - Explore all functions
- [Type System](../user-guide/type-system/custom-types.md) - Advanced types

## Practice Exercises

Try these on your own:

1. **Exercise 1**: Create a list of numbers and calculate their average
2. **Exercise 2**: Define a `Book` type with title and author, create a list of books
3. **Exercise 3**: Write a pipeline that processes a sentence (split, filter, transform, join)
4. **Exercise 4**: Create a function that takes a list and returns its sum

Solutions are in the [Cookbook section](../cookbook/data-pipelines.md).
