# BAML Integration

## Overview

DSL types are automatically converted to BAML schemas for LLM prompts. This ensures LLM responses match your expected data structure without manual schema writing.

**BAML** (Basically A Markup Language) is the schema format used by the `simplify_baml` library to communicate type information to LLMs.

## How It Works

### The Flow

```
DSL Type Definition
       ↓
  BAML Schema Generation
       ↓
  Included in LLM Prompt
       ↓
  LLM Response (JSON)
       ↓
  Parsed into DSL Value
       ↓
  Type Validation
```

### Example

**Your DSL Type:**
```dsl


type BlogPost {
  title: String
  summary: String
  keywords: [String]
}
```

**Generated BAML Schema** (sent to LLM):
```
Answer in JSON using this schema:
{
  title: string,
  summary: string,
  keywords: string[]
}
```

**LLM Response** (JSON):
```json
{
  "title": "Understanding AI",
  "summary": "An introduction to artificial intelligence",
  "keywords": ["AI", "machine learning", "deep learning"]
}
```

**Parsed DSL Value:**
```dsl
{
  title: "Understanding AI",
  summary: "An introduction to artificial intelligence",
  keywords: ["AI", "machine learning", "deep learning"]
} : BlogPost
```

## Type Mapping

### Primitive Types

| DSL Type | BAML Type | JSON Type |
|----------|-----------|-----------|
| `String` | `string` | `"text"` |
| `Int` | `int` | `42` |
| `Float` | `float` | `3.14` |
| `Bool` | `bool` | `true`/`false` |

### List Types

| DSL Type | BAML Type | JSON Type |
|----------|-----------|-----------|
| `[String]` | `string[]` | `["a", "b"]` |
| `[Int]` | `int[]` | `[1, 2, 3]` |
| `[Float]` | `float[]` | `[1.5, 2.7]` |
| `[CustomType]` | `CustomType[]` | `[{...}, {...}]` |

### Nested Types

Nested types become nested objects in BAML:

**DSL:**
```dsl


type Address {
  street: String
  city: String
}

type Person {
  name: String
  address: Address
}
```

**BAML:**
```
{
  name: string,
  address: {
    street: string,
    city: string
  }
}
```

### Enum Types

Enums become constrained string values:

**DSL:**
```dsl


enum Status {
  Pending
  InProgress
  Completed
}

type Task {
  name: String
  status: Status
}
```

**BAML:**
```
{
  name: string,
  status: "Pending" | "InProgress" | "Completed"
}
```

## Defining LLM Functions with Types

### Basic Example

```dsl


type Person {
  name: String
  age: Int
  occupation: String
}

def ExtractPerson(text: String) -> Person {
  model: "gpt-4o-mini"
  prompt: "Extract person information from: ${text}"
}

ExtractPerson("Alice is a 28 year old engineer")
```

### Behind the Scenes

When you call `ExtractPerson`, the actual prompt sent to the LLM includes:

```
Extract person information from: Alice is a 28 year old engineer

Answer in JSON using this schema:
{
  name: string,
  age: int,
  occupation: string
}
```

## Complex Examples

### Example 1: Blog Post Analysis

```dsl


enum Category {
  Technology
  Business
  Science
  Health
}

enum Sentiment {
  Positive
  Negative
  Neutral
}

type Author {
  name: String
  email: String
}

type BlogPost {
  title: String
  summary: String
  category: Category
  sentiment: Sentiment
  keywords: [String]
  author: Author
  wordCount: Int
}

def AnalyzeArticle(text: String) -> BlogPost {
  model: "gpt-4o-mini"
  prompt: "Analyze this article: ${text}"
}
```

**Generated BAML Schema:**
```
{
  title: string,
  summary: string,
  category: "Technology" | "Business" | "Science" | "Health",
  sentiment: "Positive" | "Negative" | "Neutral",
  keywords: string[],
  author: {
    name: string,
    email: string
  },
  wordCount: int
}
```

### Example 2: Multi-Entity Extraction

```dsl


type Product {
  name: String
  price: Float
  category: String
}

type Review {
  rating: Int
  text: String
  helpful: Bool
}

type ProductListing {
  product: Product
  reviews: [Review]
  averageRating: Float
  totalReviews: Int
}

def ExtractListing(html: String) -> ProductListing {
  model: "gpt-4o-mini"
  prompt: "Extract product listing from HTML: ${html}"
}
```

### Example 3: Structured Data Pipeline

```dsl


type DataPoint {
  timestamp: String
  value: Float
  label: String
}

type Analysis {
  summary: String
  insights: [String]
  dataPoints: [DataPoint]
  confidence: Float
}

def AnalyzeData(csv: String) -> Analysis {
  model: "gpt-4o-mini"
  prompt: "Analyze this CSV data: ${csv}"
}

AnalyzeData("timestamp,value\n2024-01-01,100\n2024-01-02,150")
```

## Schema Generation Details

### Field Ordering

Fields are presented to the LLM in definition order:

```dsl


type Person {
  name: String
  age: Int
  email: String
}
```

### Required vs Optional

Currently, all fields are required in the generated schema:

```dsl


type Person {
  name: String
  age: Int
}
```

**Future:** Optional field syntax `field: Type?` will generate optional schema fields.

### Type Descriptions (Future)

Planned support for field descriptions:

```dsl


type Person {
  name: String
  age: Int
  email: String
}
```

These descriptions would be included in the BAML schema to guide the LLM.

## Working with LLM Responses

### Successful Response

When the LLM returns valid JSON matching the schema:

```dsl


type Person { name: String, age: Int }

def GetPerson() -> Person {
  prompt: "Generate a person"
}

GetPerson()
```

### Invalid Response

When the LLM returns malformed data:

```dsl

GetPerson()
```

### Partial Response Handling

DSL validates the entire response. Partial matches are rejected:

```dsl


type Article {
  title: String
  summary: String
  keywords: [String]
}

```

**Workaround:** Make fields optional (when feature is available) or use separate types for partial data.

## Best Practices

### 1. Design Types for LLM Capabilities

Consider what the LLM can reasonably extract or generate:

**Good:**
```dsl


type Summary {
  title: String
  keyPoints: [String]
  sentiment: String
}
```

**Problematic:**
```dsl


type Summary {
  title: String
  molecularStructure: String
  quantumState: Int
}
```

### 2. Use Enums for Constrained Outputs

Help the LLM by providing explicit choices:

**Good:**
```dsl


enum Sentiment { Positive, Negative, Neutral }

type Review {
  text: String
  sentiment: Sentiment
}
```

**Bad:**
```dsl


type Review {
  text: String
  sentiment: String
}
```

### 3. Provide Clear Prompts

Reference the type structure in your prompt:

```dsl


type Product {
  name: String
  price: Float
  category: String
}

def ExtractProduct(text: String) -> Product {
  model: "gpt-4o-mini"
  prompt: """
    Extract the product name, price, and category from:
    ${text}
  """
}
```

### 4. Keep Structures Reasonable

Don't overwhelm the LLM with huge schemas:

**Good:**
```dsl
type Article {
  title: String
  summary: String
  keywords: [String]
}
```

**Problematic:**
```dsl
type ComplexArticle {
  metadata: {
    author: {
      profile: {
      }
    }
  }
}
```

### 5. Test with Real Data

Test your typed LLM functions with actual data:

```dsl
flow> def ExtractPerson(text: String) -> Person { ... }
flow> ExtractPerson("John is a 35 year old doctor")
flow> ExtractPerson("Mary works as a teacher")
flow> ExtractPerson("Bob is 40 and likes fishing")
```

## Schema Customization (Advanced)

### Model Selection

Different models have different capabilities for structured outputs:

```dsl


type ComplexData { /* ... */ }

def Extract(text: String) -> ComplexData {
  model: "gpt-4o-mini"
  prompt: "Extract: ${text}"
}

def Extract(text: String) -> ComplexData {
  model: "gpt-3.5-turbo"
  prompt: "Extract: ${text}"
}
```

### Prompt Engineering

Include schema hints in your prompts:

```dsl


type Product {
  name: String
  price: Float
  inStock: Bool
}

def ExtractProduct(text: String) -> Product {
  model: "gpt-4o-mini"
  prompt: """
    Extract product information from the following text.
    Make sure to include:
    - The product name (as 'name')
    - The price as a number (as 'price')
    - Whether it's available (as 'inStock')

    Text: ${text}
  """
}
```

## Troubleshooting

### LLM Returns Wrong Structure

**Problem:** LLM response doesn't match schema

**Solutions:**
1. Make prompts more explicit
2. Use a more capable model (GPT-4 vs GPT-3.5)
3. Simplify the type structure
4. Add examples in the prompt

### Missing Fields

**Problem:** LLM omits required fields

**Solutions:**
1. Explicitly list all required fields in the prompt
2. Provide example responses in the prompt
3. Use follow-up calls to fill missing data

### Type Coercion Failures

**Problem:** LLM returns string when number expected

**Solutions:**
1. Specify types clearly in the prompt
2. Use post-processing to clean data
3. Accept strings and convert manually

## Integration with simplify_baml

DSL uses the `simplify_baml` library internally for:

1. **Schema Generation:** Converting DSL types to BAML
2. **Prompt Formatting:** Including schemas in LLM prompts
3. **Response Parsing:** Converting JSON to DSL values
4. **Multi-Provider Support:** Working with different LLM APIs

The integration is seamless - you define types in DSL, and `simplify_baml` handles the rest.

## Next Steps

- [Custom Types](./custom-types.md) - Define structured data types
- [Enums](./enums.md) - Define types with fixed sets of values
- [Type Validation](./type-validation.md) - Automatic runtime validation
- [LLM Integration](../../integration/llm/overview.md) - Learn more about LLM functions
