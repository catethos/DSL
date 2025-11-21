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
Upper("hello")      // "HELLO"
Lower("WORLD")      // "world"
Length("hello")     // 5
Split("a-b-c", "-") // ["a", "b", "c"]
Join(["a", "b"], "-")   // "a-b"
Replace("hello world", "world", "DSL")  // "hello DSL"
```

**Template Strings:**
```dsl
let name = "Alice"
"Hello, ${name}!"   // "Hello, Alice!"
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
10 + 5      // 15
20 - 3      // 17
4 * 5       // 20
100 / 4     // 25
2 ^ 8       // 256 (exponentiation)
```

**Math Functions:**
```dsl
Abs(-42)    // 42
Min(5, 10)  // 5
Max(5, 10)  // 10
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
3.14 * 2        // 6.28
10.5 + 2.3      // 12.8
5.0 / 2.0       // 2.5
```

**Math Functions:**
```dsl
Round(3.7)      // 4
Floor(3.7)      // 3
Ceil(3.2)       // 4
Sqrt(16)        // 4.0
```

**Auto-Promotion:**

When mixing Int and Float, Int is automatically promoted to Float:

```dsl
5 + 3.14        // 8.14 (Int → Float)
10 * 2.5        // 25.0 (Int → Float)
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
true && true        // true (AND)
true || false       // true (OR)
!true               // false (NOT)
```

**From Comparisons:**
```dsl
5 > 3               // true
10 == 10            // true
"hello" != "world"  // true
```

## Composite Types

### List

Ordered collections of values.

```dsl
[1, 2, 3, 4, 5]
["apple", "banana", "cherry"]
[true, false, true]
[[1, 2], [3, 4]]        // Nested lists
```

**Indexing:**

Lists use 0-based indexing:

```dsl
let numbers = [10, 20, 30]
numbers[0]          // 10 (first element)
numbers[2]          // 30 (third element)
numbers[-1]         // 30 (last element, if supported)
```

**List Operations:**
```dsl
Length([1, 2, 3])           // 3
Sum([1, 2, 3, 4, 5])        // 15
Avg([1, 2, 3, 4, 5])        // 3.0
Min([5, 2, 8, 1])           // 1
Max([5, 2, 8, 1])           // 8
```

**List Transformations:**
```dsl
Map([1, 2, 3], fn n => n * 2 end)        // [2, 4, 6]
Filter([1, 2, 3, 4, 5], fn n => n > 2 end)   // [3, 4, 5]
Reduce([1, 2, 3], 0, fn acc, n => acc + n end)   // 6
```

**Heterogeneous Lists:**
```dsl
[1, "hello", true, 3.14]    // Mixed types allowed
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
user.name           // "Alice"
user.age            // 30
```

**Nested Access:**
```dsl
let data = { "address": { "city": "Boston", "zip": "02101" } }
data.address.city   // "Boston"
data.address.zip    // "02101"
```

**Dynamic Construction:**
```dsl
let name = "Alice"
let age = 30
{ "name": name, "age": age }
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
value == null       // true
```

### Table

SQL query results from DuckDB.

```dsl
SQL("SELECT * FROM 'data.csv'")     // Returns Table
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
5 + 3.14        // Int → Float, result: 8.14
10 * 2.5        // Int → Float, result: 25.0
```

### No Implicit String Conversion

```admonish warning
DSL does not automatically convert numbers to strings. Use explicit conversion functions.
```

```dsl
// Error: Cannot add String and Int
"Age: " + 42        // ✗ Error

// Use template strings instead
"Age: ${42}"        // ✓ "Age: 42"
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

alice.name      // "Alice"
alice.age       // 30
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
// Valid operations
"hello" + " world"      // ✓ String concatenation
10 + 5                  // ✓ Int arithmetic
3.14 * 2                // ✓ Float arithmetic

// Invalid operations
"hello" * 42            // ✗ Cannot multiply String and Int
[1, 2] + 5              // ✗ Cannot add List and Int
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

// Transform
numbers |> Map(_, fn n => n * 2 end)    // [2, 4, 6, 8, 10]

// Filter
numbers |> Filter(_, fn n => n > 2 end) // [3, 4, 5]

// Aggregate
numbers |> Sum(_)                    // 15
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

user.name               // "Alice"
user.address.city       // "Boston"
```

### Pattern 3: Type Conversion

```dsl
// Int to String (via template)
let age = 42
"Age: ${age}"           // "Age: 42"

// String to List
Split("a,b,c", ",")     // ["a", "b", "c"]

// List to String
Join(["a", "b", "c"], ",")  // "a,b,c"
```

## What's Next?

- [Operators](operators.md) - Work with different types using operators
- [Variables](variables.md) - Store and manage typed values
- [Custom Types](../type-system/custom-types.md) - Define your own types
- [Type Validation](../type-system/type-validation.md) - Runtime type checking
