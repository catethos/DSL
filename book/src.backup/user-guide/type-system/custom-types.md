# Custom Types (Classes)

## Overview

Custom types in DSL allow you to define structured data with named fields. They are primarily used for structured LLM outputs, ensuring AI responses match your expected schema.

## Defining Classes

### Basic Syntax

```javascript
type TypeName {
  field1: FieldType
  field2: FieldType
  field3: FieldType
}
```

### Simple Example

```dsl
type Person {
  name: String
  age: Int
  email: String
}
```

## Field Types

### Primitive Types

| Type | Description | Example Values |
|------|-------------|----------------|
| `String` | Text data | `"hello"`, `"world"` |
| `Int` | Integer numbers | `42`, `-10`, `0` |
| `Float` | Floating point | `3.14`, `-0.5` |
| `Bool` | Boolean | `true`, `false` |

### List Types

Lists are defined with square brackets around the element type:

```dsl
type Article {
  tags: [String]           // List of strings
  scores: [Int]            // List of integers
  related: [Article]       // List of Articles
}
```

### Nested Types

Types can contain other custom types as fields:

```dsl
type Address {
  street: String
  city: String
  zipCode: String
}

type Person {
  name: String
  age: Int
  address: Address         // Nested type
}
```

## Registering Types

### In REPL

```dsl
flow> type Person { name: String, age: Int }
✓ Type 'Person' registered
```

### In Editor (Workspace Mode)

1. Define the type in the editor:
   ```dsl
   type Person {
     name: String
     age: Int
     email: String
   }
   ```
2. Press **Ctrl+R** to register the type
3. The type is now available for use

## Viewing Registered Types

### List All Types

```dsl
flow> :types
✓ Registered types:
  Person
  Address
  BlogPost
```

### Type Explorer (F3)

Press **F3** in the TUI to view all types with full details:

```
type Person
  name: String
  age: Int
  email: String

type Address
  street: String
  city: String
  zipCode: String
```

## Using Types in Functions

Types are primarily used as return types for LLM functions:

```dsl
type Person {
  name: String
  age: Int
  occupation: String
}

def ExtractPerson(text: String) -> Person {
  model: "gpt-4o-mini"
  prompt: "Extract person info from: ${text}"
}

ExtractPerson("Alice is a 28 year old engineer")
// Returns: { name: "Alice", age: 28, occupation: "engineer" }
```

## Complex Type Examples

### Example 1: Blog System

```dsl
type Author {
  name: String
  email: String
  bio: String
}

type Comment {
  author: String
  content: String
  upvotes: Int
}

type BlogPost {
  title: String
  content: String
  author: Author
  tags: [String]
  comments: [Comment]
  published: Bool
}
```

### Example 2: E-Commerce

```dsl
type Product {
  id: Int
  name: String
  price: Float
  inStock: Bool
}

type OrderItem {
  product: Product
  quantity: Int
  subtotal: Float
}

type Order {
  orderId: Int
  items: [OrderItem]
  total: Float
}
```

### Example 3: Data Analysis

```dsl
type DataPoint {
  timestamp: String
  value: Float
  category: String
}

type Analysis {
  summary: String
  insights: [String]
  dataPoints: [DataPoint]
  confidence: Float
}
```

## Type Composition

### Building Complex Types

Start with simple types and compose them into larger structures:

```dsl
// Step 1: Basic types
type Address {
  street: String
  city: String
  country: String
}

type ContactInfo {
  email: String
  phone: String
  address: Address
}

// Step 2: Compose into larger types
type Person {
  name: String
  age: Int
  contact: ContactInfo
}

// Step 3: Use in collections
type Company {
  name: String
  employees: [Person]
  headquarters: Address
}
```

### Reusable Type Patterns

**Pattern 1: Result Types**
```dsl
type Result {
  success: Bool
  message: String
  data: String
}
```

**Pattern 2: Pagination**
```dsl
type Page {
  items: [String]
  totalCount: Int
  pageNumber: Int
  hasNext: Bool
}
```

**Pattern 3: Error Information**
```dsl
type Error {
  code: Int
  message: String
  details: String
}
```

## Working with Typed Data

### Accessing Fields

```dsl
type Person { name: String, age: Int }

def GetPerson() -> Person {
  prompt: "Generate a person"
}

GetPerson() as person
person.name              // Access field
person.age               // Access field
```

### Lists of Typed Objects

```dsl
type Person { name: String }

def GetPeople() -> [Person] {
  prompt: "Generate 3 people"
}

GetPeople() as people
people[0].name          // First person's name
people[1].name          // Second person's name
Length(people)          // Number of people
```

## Best Practices

### 1. Use Clear Field Names

**Good:**
```dsl
type Person {
  fullName: String
  emailAddress: String
  phoneNumber: String
}
```

**Bad:**
```dsl
type Person {
  n: String
  e: String
  p: String
}
```

### 2. Group Related Fields

```dsl
type User {
  // Identity
  id: Int
  username: String

  // Contact
  email: String
  phone: String

  // Profile
  bio: String
  avatar: String
}
```

### 3. Model Real-World Entities

```dsl
type Invoice {
  invoiceNumber: String
  date: String
  customer: Customer
  lineItems: [LineItem]
  subtotal: Float
  tax: Float
  total: Float
}
```

## Current Limitations

### Optional Fields (Not Yet Implemented)

```dsl
type Person {
  name: String
  age: Int
  email: String?           // Optional (not yet supported)
}
```

**Workaround:** Use empty strings or 0 for "missing" values

### No Union Types

Cannot define "String or Int" fields.

**Workaround:** Use separate types or convert to a common type

### No Generic Types

Cannot define `List<T>` or `Result<T>`.

**Workaround:** Define specific types for each use case

### No Type Aliases

Cannot do `type ID = Int`.

**Workaround:** Use the base type directly

## Next Steps

- [Enums](./enums.md) - Define types with fixed sets of values
- [Type Validation](./type-validation.md) - Automatic runtime validation
- [BAML Integration](./baml-integration.md) - Schema generation for LLMs
