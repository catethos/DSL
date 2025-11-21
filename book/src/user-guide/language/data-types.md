# Data Types

DSL provides a rich type system with primitive types, composite types, and special types for different use cases.

## Primitive Types

### String

Text data enclosed in quotes.

```dsl
"hello world"
'single quotes also work'
```

**String Operations:**
```dsl
Upper("hello")
Lower("WORLD")
Length("hello")
Split("a-b-c", "-")
Join(["a", "b"], "-")
Replace("hello world", "world", "DSL")
```

**Template Strings:**
```dsl
let name = "Alice"
"Hello, ${name}!"
```

### Int

Integer numbers (whole numbers).

```dsl
42
-10
0
1000
```

**Int Operations:**
```dsl
10 + 5
20 - 3
4 * 5
100 / 4
2 ^ 8
```

**Math Functions:**
```dsl
Abs(-42)
Min(5, 10)
Max(5, 10)
```

### Float

Floating-point numbers (decimals).

```dsl
3.14
-0.5
2.71828
0.0
```

**Float Operations:**
```dsl
3.14 * 2
10.5 + 2.3
5.0 / 2.0
```

**Math Functions:**
```dsl
Round(3.7)
Floor(3.7)
Ceil(3.2)
Sqrt(16)
```

**Auto-Promotion:**

When mixing Int and Float, Int is automatically promoted to Float:

```dsl
5 + 3.14
10 * 2.5
```

```admonish tip
The DSL automatically promotes integers to floats when needed, so you don't need to manually cast.
```

### Bool

Boolean values representing true or false.

```dsl
true
false
```

**Boolean Operations:**
```dsl
true && true
true || false
!true
```

**From Comparisons:**
```dsl
5 > 3
10 == 10
"hello" != "world"
```

## Composite Types

### List

Ordered collections of values.

```dsl
[1, 2, 3, 4, 5]
["apple", "banana", "cherry"]
[true, false, true]
[[1, 2], [3, 4]]
```

**Indexing:**

Lists use 0-based indexing:

```dsl
let numbers = [10, 20, 30]
numbers[0]
numbers[2]
numbers[-1]
```

**List Operations:**
```dsl
Length([1, 2, 3])
Sum([1, 2, 3, 4, 5])
Avg([1, 2, 3, 4, 5])
Min([5, 2, 8, 1])
Max([5, 2, 8, 1])
```

**List Transformations:**
```dsl
map([1, 2, 3], fn n => n * 2 end)
filter([1, 2, 3, 4, 5], fn n => n > 2 end)
reduce([1, 2, 3], 0, fn acc, n => acc + n end)
```

**Heterogeneous Lists:**
```dsl
[1, "hello", true, 3.14]
```

### Map

Key-value pairs (objects/dictionaries).

```dsl
{ "name": "Alice", "age": 30 }
{ "x": 10, "y": 20 }
{ "nested": { "key": "value" } }
```

**Field Access:**

Use dot notation to access fields:

```dsl
let user = { "name": "Alice", "age": 30 }
user.name
user.age
```

**Nested Access:**
```dsl
let data = { "address": { "city": "Boston", "zip": "02101" } }
data.address.city
data.address.zip
```

**Dynamic Construction:**
```dsl
def main() {
    let name = "Alice"
    let age = 30
    { "name": name, "age": age }
}
main()
```

## Special Types

### Null

Represents the absence of a value.

```dsl
null
```

**Use Cases:**
- Optional values
- Uninitialized data
- API responses with missing data

**Checking for Null:**
```dsl
let value = null
value == null
```

### Table

SQL query results from DuckDB.

```dsl
SQL("SELECT * FROM 'data.csv'")
```

**Table Properties:**
- Rows and columns
- Can be queried with SQL
- Can be converted to lists

**Working with Tables:**
```dsl
SQL("SELECT * FROM 'users.csv'") as users
SQL("SELECT name, age FROM users WHERE age > 25")
```

## Type Annotations

Variables carry runtime type information:

```dsl
flow> 42 as x
✓ Bound 'x' to 42 : Int

flow> "hello" as greeting
✓ Bound 'greeting' to "hello" : String

flow> [1, 2, 3] as numbers
✓ Bound 'numbers' to [1, 2, 3] : List
```

**Check Variable Types:**
```dsl
flow> :vars
✓ Variables:
  x = 42 : Int
  greeting = "hello" : String
  numbers = [1, 2, 3] : List
```

## Type Coercion

### Automatic Coercion

Some types are automatically converted:

```dsl
5 + 3.14
10 * 2.5
```

### No Implicit String Conversion

```admonish warning
DSL does not automatically convert numbers to strings. Use explicit conversion functions.
```

```dsl
"Age: " + 42

"Age: ${42}"
```

## Custom Types

Define your own types using the `class` keyword:

```dsl
class Person {
    name: String
    age: Int
    email: String
}

let alice = Person {
    name: "Alice",
    age: 30,
    email: "alice@example.com"
}

alice.name
alice.age
```

See [Custom Types](../type-system/custom-types.md) for more details.

## Type Checking

### Runtime Type Validation

Types are checked at runtime:

```dsl
flow> "hello" + 42
✗ Error: Cannot add String and Int

flow> Upper(42)
✗ Error: Upper expects String, got Int
```

### Type Safety

The DSL enforces type safety for operations:

```dsl
"hello" + " world"
10 + 5
3.14 * 2

"hello" * 42
[1, 2] + 5
```

## Type Compatibility Table

| Operation | Int | Float | String | Bool | List | Map | Null |
|-----------|-----|-------|--------|------|------|-----|------|
| **Arithmetic (+, -, *, /)** | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ |
| **Comparison (==, !=)** | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| **Logical (&&, \|\|)** | ✗ | ✗ | ✗ | ✓ | ✗ | ✗ | ✗ |
| **Index Access ([])** | ✗ | ✗ | ✗ | ✗ | ✓ | ✗ | ✗ |
| **Field Access (.)** | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ | ✗ |

## Common Type Patterns

### Pattern 1: Working with Lists

```dsl
let numbers = [1, 2, 3, 4, 5]

numbers |> map(_, fn n => n * 2 end)

numbers |> filter(_, fn n => n > 2 end)

numbers |> Sum(_)
```

### Pattern 2: Working with Maps

```dsl
let user = {
    "name": "Alice",
    "age": 30,
    "address": {
        "city": "Boston"
    }
}

user.name
user.address.city
```

### Pattern 3: Type Conversion

```dsl
let age = 42
"Age: ${age}"

Split("a,b,c", ",")

Join(["a", "b", "c"], ",")
```

## What's Next?

- [Operators](operators.md) - Work with different types using operators
- [Variables](variables.md) - Store and manage typed values
- [Custom Types](../type-system/custom-types.md) - Define your own types
- [Type Validation](../type-system/type-validation.md) - Runtime type checking
