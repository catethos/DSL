# Enums

## Overview

Enums allow you to define a type with a fixed set of possible values. They are useful for representing categorical data where only specific values are valid.

## Defining Enums

### Basic Syntax

```javascript
enum EnumName {
  Value1
  Value2
  Value3
}
```

### Simple Example

```dsl
enum Status {
  Pending
  InProgress
  Completed
  Failed
}
```

## Common Use Cases

### 1. Status Values

```dsl
enum OrderStatus {
  Pending
  Processing
  Shipped
  Delivered
  Cancelled
}
```

### 2. Priority Levels

```dsl
enum Priority {
  Low
  Medium
  High
  Critical
}
```

### 3. Categories

```dsl
enum Category {
  Technology
  Business
  Science
  Health
  Entertainment
}
```

### 4. Sentiment Analysis

```dsl
enum Sentiment {
  Positive
  Negative
  Neutral
}
```

## Using Enums with Types

Enums are most commonly used as fields in custom types:

```dsl
enum Category {
  Technology
  Business
  Science
  Health
}

type Article {
  title: String
  content: String
  category: Category
  views: Int
}
```

### Complex Example

```dsl
enum OrderStatus {
  Pending
  Processing
  Shipped
  Delivered
  Cancelled
}

enum Priority {
  Low
  Medium
  High
  Critical
}

type Product {
  id: Int
  name: String
  price: Float
  inStock: Bool
}

type Order {
  orderId: Int
  product: Product
  status: OrderStatus
  priority: Priority
  customerEmail: String
}
```

## Registering Enums

### In REPL

```dsl
flow> enum Status { Pending, InProgress, Completed }
✓ Enum 'Status' registered
```

### In Editor

1. Define the enum:
   ```dsl
   enum Status {
     Pending
     InProgress
     Completed
     Failed
   }
   ```
2. Press **Ctrl+R** to register
3. Use in type definitions or functions

## Viewing Registered Enums

### List Command

```dsl
flow> :types
✓ Registered types:
  Person
  Status (enum)
  Priority (enum)
```

### Type Explorer (F3)

Press **F3** to view all enums with their values:

```
enum Status
  Pending
  InProgress
  Completed
  Failed

enum Priority
  Low
  Medium
  High
  Critical
```

## Enum Values

### Case Sensitivity

Enum values are **case insensitive** when matching:

```dsl
enum Status {
  Pending
  Completed
}

// All of these are valid:
"Pending"
"pending"
"PENDING"
"PeNdInG"
```

The DSL normalizes enum values to match the defined casing.

### String Representation

Enums are represented as strings in DSL:

```dsl
enum Status { Pending, Completed }

type Task {
  name: String
  status: Status
}

// Valid task:
{
  name: "Build feature",
  status: "Pending"
}
```

## Using Enums with LLMs

Enums are particularly useful for structured LLM outputs:

```dsl
enum Sentiment {
  Positive
  Negative
  Neutral
}

type Review {
  text: String
  sentiment: Sentiment
  score: Float
}

def AnalyzeReview(text: String) -> Review {
  model: "gpt-4o-mini"
  prompt: "Analyze this review: ${text}"
}

AnalyzeReview("This product is amazing!")
// Returns: { text: "...", sentiment: "Positive", score: 0.95 }
```

The LLM will receive the enum values as part of the schema and must respond with one of the valid values.

## Validation

### Automatic Validation

When a function returns a typed value with an enum field, the DSL validates the enum value:

```dsl
enum Status { Pending, Completed }

type Task { name: String, status: Status }

def GetTask() -> Task {
  prompt: "Generate a task"
}

GetTask()
// ✓ { name: "Task", status: "Pending" }  // Valid
// ✗ Error if status is "Unknown"         // Invalid
```

### Validation Rules

1. **Value must match one of the enum variants**
   ```dsl
   enum Status { Pending, Completed }

   // Valid
   "Pending"
   "Completed"

   // Invalid
   "Unknown"
   // ✗ Error: Not a valid Status value
   ```

2. **Case insensitive matching**
   ```dsl
   enum Status { Pending, Completed }

   // All valid (normalized to "Pending")
   "pending"
   "PENDING"
   "PeNdInG"
   ```

## Best Practices

### 1. Use Enums for Fixed Sets

**Good:**
```dsl
enum Priority {
  Low
  Medium
  High
  Critical
}

type Task {
  name: String
  priority: Priority  // Type-safe
}
```

**Bad:**
```dsl
type Task {
  name: String
  priority: String     // Could be anything!
}
```

### 2. Keep Values Descriptive

**Good:**
```dsl
enum PaymentMethod {
  CreditCard
  DebitCard
  PayPal
  BankTransfer
}
```

**Bad:**
```dsl
enum PaymentMethod {
  A
  B
  C
  D
}
```

### 3. Logical Grouping

**Good:**
```dsl
// Separate enums for different concepts
enum OrderStatus {
  Pending
  Shipped
  Delivered
}

enum PaymentStatus {
  Unpaid
  Paid
  Refunded
}
```

**Bad:**
```dsl
// Mixed concepts in one enum
enum Status {
  OrderPending
  OrderShipped
  PaymentUnpaid
  PaymentPaid
}
```

### 4. Consistent Naming

Use **PascalCase** for enum values:

**Good:**
```dsl
enum Status {
  InProgress
  OnHold
  Completed
}
```

**Bad:**
```dsl
enum Status {
  in_progress
  on-hold
  COMPLETED
}
```

## Real-World Examples

### Example 1: Content Management

```dsl
enum ContentType {
  Article
  Video
  Podcast
  Infographic
}

enum PublishStatus {
  Draft
  Scheduled
  Published
  Archived
}

type Content {
  id: Int
  title: String
  type: ContentType
  status: PublishStatus
  author: String
  publishDate: String
}
```

### Example 2: Task Management

```dsl
enum TaskStatus {
  Todo
  InProgress
  Review
  Done
}

enum TaskPriority {
  Low
  Medium
  High
  Urgent
}

type Task {
  id: Int
  title: String
  description: String
  status: TaskStatus
  priority: TaskPriority
  assignee: String
  dueDate: String
}
```

### Example 3: E-Commerce

```dsl
enum ProductCategory {
  Electronics
  Clothing
  Books
  Home
  Sports
}

enum ProductCondition {
  New
  LikeNew
  Good
  Fair
  Poor
}

type Product {
  id: Int
  name: String
  category: ProductCategory
  condition: ProductCondition
  price: Float
  inStock: Bool
}
```

## Common Patterns

### Pattern 1: Workflow States

```dsl
enum WorkflowState {
  Submitted
  UnderReview
  Approved
  Rejected
  Completed
}
```

### Pattern 2: Log Levels

```dsl
enum LogLevel {
  Debug
  Info
  Warning
  Error
  Critical
}
```

### Pattern 3: User Roles

```dsl
enum UserRole {
  Guest
  User
  Moderator
  Admin
  SuperAdmin
}
```

## Limitations

### No Enum Values with Data

Enums in DSL are simple value enums. They cannot carry additional data:

```dsl
// Not supported
enum Result {
  Success(String)
  Error(Int, String)
}
```

**Workaround:** Use separate types:

```dsl
enum ResultType {
  Success
  Error
}

type Result {
  type: ResultType
  message: String
  errorCode: Int
}
```

### No Default Values

Enum fields in types are always required:

```dsl
enum Status { Pending, Completed }

type Task {
  name: String
  status: Status  // Cannot have default value
}
```

**Workaround:** Set the value explicitly when creating instances

## Next Steps

- [Custom Types](./custom-types.md) - Define structured data types
- [Type Validation](./type-validation.md) - Automatic runtime validation
- [BAML Integration](./baml-integration.md) - Schema generation for LLMs
