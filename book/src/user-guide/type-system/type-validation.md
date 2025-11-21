# Type Validation

## Overview

DSL provides automatic type validation for structured data. When functions return typed values, the runtime validates that the data matches the expected schema.

## Automatic Validation

### How It Works

When a function declares a return type, DSL automatically validates the result:

```dsl


type Person {
  name: String
  age: Int
}

def GetPerson() -> Person {
  prompt: "Generate a person"
}

GetPerson()
```

### Validation Process

1. **Function executes** and returns a value
2. **Validator checks** the value against the declared type
3. **Success**: Value passes through with type annotation
4. **Failure**: Runtime error with details about what's wrong

## Validation Rules

### Rule 1: Required Fields Must Be Present

All fields defined in a type must be present in the value:

```dsl


type Person {
  name: String
  age: Int
}

{ name: "Alice", age: 30 }

{ name: "Alice" }
```

### Rule 2: Field Types Must Match

Each field's value must match its declared type:

```dsl


type Person {
  name: String
  age: Int
}

{ name: "Alice", age: 30 }

{ name: "Alice", age: "thirty" }
```

### Rule 3: Enum Values Must Be Valid

Enum fields must contain one of the defined enum values:

```dsl


enum Status {
  Pending
  Completed
}

type Task {
  name: String
  status: Status
}

{ name: "Task 1", status: "Pending" }
{ name: "Task 2", status: "pending" }

{ name: "Task 3", status: "Unknown" }
```

### Rule 4: List Elements Must Match Type

For list fields, all elements must match the element type:

```dsl


type Article {
  tags: [String]
  scores: [Int]
}

{ tags: ["tech", "ai"], scores: [85, 92, 78] }

{ tags: ["tech"], scores: [85, "ninety"] }
```

### Rule 5: Nested Types Must Be Valid

Nested type fields must themselves be valid instances:

```dsl


type Address {
  street: String
  city: String
}

type Person {
  name: String
  address: Address
}

{
  name: "Alice",
  address: { street: "123 Main St", city: "NYC" }
}

{
  name: "Alice",
  address: { street: "123 Main St" }
}
```

## Type Coercion

### Automatic Coercion

DSL attempts to coerce values to the expected type when possible:

```dsl


type Person {
  name: String
  age: Int
}

{ name: "Alice", age: "30" }

type Product { price: Float }
{ price: 10 }
```

### When Coercion Fails

If coercion is not possible, validation fails:

```dsl


type Person {
  name: String
  age: Int
}

{ name: "Alice", age: "thirty" }
```

## Validation in Practice

### Example 1: LLM Function

```dsl


type BlogPost {
  title: String
  summary: String
  keywords: [String]
}

def AnalyzeArticle(text: String) -> BlogPost {
  model: "gpt-4o-mini"
  prompt: "Analyze this article: ${text}"
}

AnalyzeArticle("Long article text...")
```

### Example 2: Multi-Level Validation

```dsl


enum Priority { Low, Medium, High }

type Task {
  name: String
  priority: Priority
}

type Project {
  name: String
  tasks: [Task]
}

def GetProject() -> Project {
  prompt: "Generate a project with tasks"
}

GetProject()
```

### Example 3: Complex Nested Types

```dsl


type Address {
  street: String
  city: String
  zipCode: String
}

type ContactInfo {
  email: String
  phone: String
  address: Address
}

type Person {
  name: String
  age: Int
  contact: ContactInfo
}

def GetPerson() -> Person {
  prompt: "Generate person info"
}

GetPerson()
```

## Error Messages

### Clear Error Reporting

Validation errors include detailed information about what went wrong:

```dsl


type Person {
  name: String
  age: Int
  email: String
}

{ name: "Alice", age: 30 }

{ name: "Alice", age: "thirty", email: "alice@example.com" }

enum Status { Pending, Completed }
type Task { name: String, status: Status }
{ name: "Task", status: "Unknown" }
```

### Field Path in Nested Errors

For nested types, errors include the full path to the problematic field:

```dsl


type Address { street: String, city: String }
type Person { name: String, address: Address }

{
  name: "Alice",
  address: { street: "123 Main St" }
}
```

## Runtime Type Checking

### Inspecting Types at Runtime

Check variable types using the REPL:

```dsl
flow> 42 as x
✓ Bound 'x' to 42 : Int

flow> :vars
✓ Variables:
  x = 42 : Int
```

### Type Annotations

Values returned from typed functions show their type:

```dsl


type Person { name: String, age: Int }

def GetPerson() -> Person {
  prompt: "Generate a person"
}

GetPerson()
```

## Validation Performance

### When Validation Occurs

Validation happens **only** when:
1. A function with a return type completes
2. A value is explicitly cast to a type (future feature)

Validation does **not** occur for:
- Intermediate computations
- Variable assignments without type annotations
- Plain data structures

### Performance Characteristics

- **Fast**: Validation is a single-pass check
- **Lazy**: Only validates when necessary
- **Comprehensive**: Checks entire nested structure in one pass

## Best Practices

### 1. Use Types for External Data

Apply types to data from external sources:

```dsl


type ApiResponse {
  status: Int
  data: String
  timestamp: String
}

def FetchData(url: String) -> ApiResponse {
  http: "GET"
  url: url
}

FetchData("https://api.example.com/data")
```

### 2. Validate LLM Outputs

Always use return types for LLM functions:

```dsl


type Summary {
  title: String
  keyPoints: [String]
  sentiment: String
}

def Summarize(text: String) -> Summary {
  model: "gpt-4o-mini"
  prompt: "Summarize: ${text}"
}

Summarize("Long article...")
```

### 3. Use Enums for Constrained Values

Prefer enums over strings for validation:

**Good:**
```dsl


enum Status { Pending, Completed }

type Task {
  name: String
  status: Status
}
```

**Bad:**
```dsl


type Task {
  name: String
  status: String
}
```

### 4. Start Simple, Add Complexity

Build types incrementally:

```dsl
type Person {
  name: String
  age: Int
}

type Address { street: String, city: String }

type Person {
  name: String
  age: Int
  address: Address
}

enum Status { Active, Inactive }

type Person {
  name: String
  age: Int
  address: Address
  status: Status
  tags: [String]
}
```

## Testing Type Validation

### In REPL

Test your types interactively:

```dsl
flow> type Person { name: String, age: Int }
✓ Type 'Person' registered

flow> def MakePerson() -> Person { prompt: "Generate person" }
✓ Function 'MakePerson' registered

flow> MakePerson()
⏳ Calling LLM...
✓ { name: "Alice", age: 30 } : Person

flow> def BadPerson() -> Person { prompt: "Return just a name" }
flow> BadPerson()
✗ Error: Missing required field 'age'
```

### In Editor (Workspace Mode)

1. Define types and functions in editor
2. Press **Ctrl+R** to load
3. Test in REPL pane
4. Observe validation errors
5. Fix and retry

## Current Limitations

### No Optional Fields

All fields are required. Planned for future:

```dsl
type Person {
  name: String
  age: Int
  email: String?
}
```

**Current Workaround:** Use empty strings or 0 for missing values

### No Custom Validators

Cannot define custom validation logic:

```dsl
type Person {
  age: Int where age >= 0 && age <= 150
}
```

**Workaround:** Validate manually after type validation

### No Partial Validation

Must validate entire structure at once:

```dsl
```

## Next Steps

- [Custom Types](./custom-types.md) - Define structured data types
- [Enums](./enums.md) - Define types with fixed sets of values
- [BAML Integration](./baml-integration.md) - Schema generation for LLMs
