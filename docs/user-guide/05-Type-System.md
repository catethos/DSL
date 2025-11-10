# Type System

## Overview

The DSL provides a powerful type system for defining structured data. Types are used primarily for structured LLM outputs, ensuring AI responses match your expected schema.

## Type Definitions

### Classes (Structs)

Define structured data types with named fields.

**Syntax:**
```javascript
type TypeName {
  field1: FieldType
  field2: FieldType
  field3: FieldType
}
```

**Example:**
```javascript
type Person {
  name: String
  age: Int
  email: String
}
```

### Enums

Define types with a fixed set of values.

**Syntax:**
```javascript
enum EnumName {
  Value1
  Value2
  Value3
}
```

**Example:**
```javascript
enum Status {
  Pending
  InProgress
  Completed
  Failed
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

**Syntax:**
```javascript
[ElementType]
```

**Examples:**
```javascript
type Article {
  tags: [String]           // List of strings
  scores: [Int]            // List of integers
  related: [Article]       // List of Articles
}
```

### Nested Types

**Example:**
```javascript
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

### Optional Fields (Future)

**Syntax:**
```javascript
fieldName: Type?
```

**Example:**
```javascript
type Person {
  name: String
  age: Int
  email: String?           // Optional
}
```

**Note:** Optional fields not yet fully implemented.

## Using Types

### Registering Types

Define types in the REPL or Editor:

```javascript
// In REPL
flow> type Person { name: String, age: Int }
✓ Type 'Person' registered

// In Editor (Workspace Mode)
type Person {
  name: String
  age: Int
  email: String
}

// Press Ctrl+R to register
```

### Viewing Types

**Command:** `:types`

```javascript
flow> :types
✓ Registered types:
  Person
  Address
  BlogPost
```

**Type Explorer (F3):**

Press F3 to view all types with full details:

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

### Using Types in Functions

Types are primarily used as return types for LLM functions:

```javascript
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

## Type Validation

### Automatic Validation

When a function returns a typed value, the DSL validates it:

```javascript
type Person {
  name: String
  age: Int
}

def GetPerson() -> Person {
  prompt: "Generate a person"
}

GetPerson()
// ✓ { name: "Alice", age: 30 } : Person
// Type validated automatically!
```

### Validation Rules

1. **Required fields must be present**
   ```javascript
   // Missing 'age' field
   { name: "Alice" }
   // ✗ Error: Missing required field 'age'
   ```

2. **Field types must match**
   ```javascript
   // 'age' should be Int, not String
   { name: "Alice", age: "thirty" }
   // Automatically coerced if possible, error otherwise
   ```

3. **Enum values must match**
   ```javascript
   enum Status { Pending, Completed }

   // Valid
   "Pending"
   "pending"        // Case insensitive

   // Invalid
   "Unknown"
   // ✗ Error: Not a valid Status value
   ```

## Integration with BAML

### Schema Generation

Types are converted to BAML schemas for LLM prompts:

**Your Type:**
```javascript
type BlogPost {
  title: String
  summary: String
  keywords: [String]
}
```

**Generated Schema (sent to LLM):**
```
Answer in JSON using this schema:
{
  title: string,
  summary: string,
  keywords: string[]
}
```

### Response Parsing

LLM responses are automatically parsed into your types:

**LLM Response:**
```json
{
  "title": "Understanding AI",
  "summary": "An introduction to artificial intelligence",
  "keywords": ["AI", "machine learning", "deep learning"]
}
```

**Parsed Value:**
```javascript
{
  title: "Understanding AI",
  summary: "An introduction to artificial intelligence",
  keywords: ["AI", "machine learning", "deep learning"]
} : BlogPost
```

## Complex Type Examples

### Example 1: Blog System

```javascript
enum Category {
  Technology
  Business
  Science
  Health
}

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
  category: Category
  tags: [String]
  comments: [Comment]
  published: Bool
}
```

### Example 2: E-Commerce

```javascript
enum OrderStatus {
  Pending
  Processing
  Shipped
  Delivered
  Cancelled
}

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
  status: OrderStatus
}
```

### Example 3: Data Analysis

```javascript
enum Sentiment {
  Positive
  Negative
  Neutral
}

type DataPoint {
  timestamp: String
  value: Float
  category: String
}

type Analysis {
  summary: String
  insights: [String]
  sentiment: Sentiment
  dataPoints: [DataPoint]
  confidence: Float
}
```

## Type Composition

### Building Complex Types

Start with simple types and compose them:

```javascript
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
```javascript
type Result {
  success: Bool
  message: String
  data: String
}
```

**Pattern 2: Pagination**
```javascript
type Page {
  items: [String]
  totalCount: Int
  pageNumber: Int
  hasNext: Bool
}
```

**Pattern 3: Error Information**
```javascript
type Error {
  code: Int
  message: String
  details: String
}
```

## Best Practices

### 1. Use Clear Field Names

**Good:**
```javascript
type Person {
  fullName: String
  emailAddress: String
  phoneNumber: String
}
```

**Bad:**
```javascript
type Person {
  n: String
  e: String
  p: String
}
```

### 2. Group Related Fields

```javascript
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

### 3. Use Enums for Fixed Sets

**Good:**
```javascript
enum Priority {
  Low
  Medium
  High
  Critical
}
```

**Bad:**
```javascript
type Task {
  priority: String  // Could be anything!
}
```

### 4. Model Real-World Entities

```javascript
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

## Type Inspection

### At Runtime

Check variable types:

```javascript
flow> 42 as x
✓ Bound 'x' to 42 : Int

flow> :vars
✓ Variables:
  x = 42 : Int
```

### In Type Explorer

Press **F3** to browse all registered types:

- Full type definitions
- Field names and types
- Enum values

## Working with Typed Data

### Accessing Fields

```javascript
type Person { name: String, age: Int }

def GetPerson() -> Person {
  prompt: "Generate a person"
}

GetPerson() as person
person.name              // Access field
person.age               // Access field
```

### Lists of Typed Objects

```javascript
type Person { name: String }

def GetPeople() -> [Person] {
  prompt: "Generate 3 people"
}

GetPeople() as people
people[0].name          // First person's name
people[1].name          // Second person's name
Length(people)          // Number of people
```

## Type System Limitations

### Current Limitations

1. **No Optional Fields**
   - All fields are required
   - Workaround: Use empty strings or 0 for "missing" values

2. **No Union Types**
   - Can't define "String or Int"
   - Workaround: Use separate types

3. **No Generic Types**
   - Can't define `List<T>` or `Result<T>`
   - Workaround: Define specific types

4. **No Type Aliases**
   - Can't do `type ID = Int`
   - Workaround: Use the base type directly

### Future Enhancements

Planned features:
- Optional fields (`field: Type?`)
- Union types (`Type1 | Type2`)
- Generic types (`List<T>`, `Map<K, V>`)
- Type constraints and validation
- Default field values

## Integration Examples

### Example 1: LLM with Structured Output

```javascript
type Article {
  title: String
  summary: String
  keywords: [String]
}

def AnalyzeArticle(text: String) -> Article {
  model: "gpt-4o-mini"
  prompt: "Analyze this article: ${text}"
}

"Long article text..." >> AnalyzeArticle(_) as result
result.title
result.keywords
```

### Example 2: Multiple Types in Workflow

```javascript
type Person { name: String, role: String }
type Team { teamName: String, members: [Person] }

def ExtractTeam(text: String) -> Team {
  prompt: "Extract team info: ${text}"
}

"Our team includes Alice (engineer) and Bob (designer)"
  >> ExtractTeam(_) as team
  >> team.members[0].name
```

## Next Steps

- **[06-Functions.md](06-Functions.md)** - Learn about functions
- **[08-LLM-Integration.md](08-LLM-Integration.md)** - Use types with LLMs
- **[examples/types_example.dsl](../examples/types_example.dsl)** - See type examples

## Related Documents

- **[DESIGN.md](../DESIGN.md)** - Complete type system specification
- **[EXTRACTAS_EXAMPLE.md](../EXTRACTAS_EXAMPLE.md)** - Type extraction examples
