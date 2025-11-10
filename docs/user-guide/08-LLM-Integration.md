# LLM Integration

## Overview

The DSL integrates with Large Language Models (LLMs) via **simplify_baml**, providing both unstructured and structured outputs.

## Configuration

### API Key Setup

```bash
export OPENAI_API_KEY="sk-..."
```

Currently supports OpenAI models. Default: `gpt-4o-mini`

## Built-in LLM Functions

### Ask() - Simple Queries

Unstructured LLM queries returning plain text.

```javascript
Ask("What is the capital of France?")
// "The capital of France is Paris."

Ask("Explain ${topic} in one sentence") as explanation
```

## User-Defined LLM Functions

### Basic LLM Function

```javascript
def Greet(name: String) {
  model: "gpt-4o-mini"
  temperature: 0.7
  prompt: "Generate a friendly greeting for ${name}"
}

Greet("Alice")
// "Hello Alice! Great to meet you!"
```

### With Structured Output

Define a type, then use as return type:

```javascript
type Person {
  name: String
  age: Int
  occupation: String
  location: String
}

def ExtractPerson(text: String) -> Person {
  model: "gpt-4o-mini"
  temperature: 0.3
  prompt: "Extract person information from: ${text}"
}

ExtractPerson("Alice is a 28 year old engineer in Boston")
// { name: "Alice", age: 28, occupation: "engineer", location: "Boston" }
```

## Function Properties

### Model Selection

```javascript
def MyFunction() {
  model: "gpt-4o-mini"          // Default, fast and cheap
  // model: "gpt-4"              // More capable, slower, expensive
  // model: "gpt-3.5-turbo"      // Fast, less capable
}
```

### Temperature Control

Controls randomness (0.0 = deterministic, 2.0 = very random):

```javascript
def ExtractData(text: String) -> Data {
  temperature: 0.1              // Low = factual, consistent
  prompt: "Extract data from: ${text}"
}

def CreativeWriting(topic: String) {
  temperature: 1.5              // High = creative, varied
  prompt: "Write creatively about: ${topic}"
}
```

### Max Tokens

Limit response length:

```javascript
def Summarize(text: String) {
  max_tokens: 100               // Short summary
  prompt: "Summarize: ${text}"
}
```

## Prompting Best Practices

### 1. Clear Instructions

```javascript
def ExtractInfo(text: String) -> Info {
  prompt: """
    Extract the following information from the text:
    - Name of person
    - Age
    - Occupation

    Text: ${text}

    Be precise and only extract information explicitly stated.
  """
}
```

### 2. Provide Context

```javascript
def AnalyzeArticle(article: String, criteria: [String]) -> Analysis {
  prompt: """
    Analyze this article based on the following criteria:
    ${criteria}

    Article:
    ${article}

    Provide detailed analysis for each criterion.
  """
}
```

### 3. Use Examples (Few-Shot)

```javascript
def Classify(text: String) -> Category {
  prompt: """
    Classify the following text into a category.

    Examples:
    - "Stock prices rise" → Business
    - "New vaccine approved" → Health
    - "Team wins championship" → Sports

    Text: ${text}
  """
}
```

### 4. Specify Output Format

```javascript
def GenerateList(topic: String) -> [String] {
  prompt: """
    Generate a list of 5 items related to: ${topic}

    Format your response as a JSON array of strings.
  """
}
```

## Type-Driven Outputs

### How It Works

1. Define your type
2. BAML generates a schema
3. Schema appended to prompt
4. LLM responds in JSON
5. Response validated and parsed

**Your Type:**
```javascript
type BlogPost {
  title: String
  summary: String
  keywords: [String]
  category: String
}
```

**Generated Prompt Addition:**
```
Answer in JSON using this schema:
{
  title: string,
  summary: string,
  keywords: string[],
  category: string
}
```

## Complex Type Examples

### Nested Types

```javascript
type Address {
  street: String
  city: String
  country: String
}

type Person {
  name: String
  age: Int
  address: Address
}

def ExtractPerson(text: String) -> Person {
  prompt: "Extract person details including address from: ${text}"
}
```

### Lists of Objects

```javascript
type Article {
  title: String
  author: String
  summary: String
}

def ExtractArticles(text: String) -> [Article] {
  prompt: "Extract all article mentions from: ${text}"
}

ExtractArticles("...")
// [
//   { title: "...", author: "...", summary: "..." },
//   { title: "...", author: "...", summary: "..." }
// ]
```

### Enums

```javascript
enum Sentiment {
  Positive
  Negative
  Neutral
}

type Analysis {
  text: String
  sentiment: Sentiment
  confidence: Float
}

def AnalyzeSentiment(text: String) -> Analysis {
  prompt: "Analyze sentiment of: ${text}"
}
```

## Workflow Integration

### Sequential LLM Calls

```javascript
Draft(topic) as v1
  >> Review(v1) as feedback
  >> Revise(v1, feedback) as v2
  >> Finalize(v2)
```

### Parallel LLM Calls

```javascript
(
  Summarize(article, 50) ||
  ExtractKeywords(article) ||
  AnalyzeSentiment(article)
) as [summary, keywords, sentiment]
```

### LLM + HTTP

```javascript
def AnalyzeWebPage(url: String) -> Analysis {
  http: "GET"
  url: "${url}"

  model: "gpt-4o-mini"
  prompt: "Analyze this webpage content"
}

// Fetches page, then analyzes with LLM
AnalyzeWebPage("https://example.com")
```

### LLM + SQL

```javascript
SQL("SELECT * FROM 'data.csv'") as data
  >> AnalyzeData(data) as insights
  >> GenerateReport(insights)
```

## Error Handling

### Missing API Key

```javascript
Ask("Hello")
// ✗ Error: OpenAI API key not found
// Solution: export OPENAI_API_KEY="sk-..."
```

### Invalid Response

```javascript
ExtractPerson("No person mentioned here")
// ✗ Error: Could not parse response into Person type
// LLM couldn't find required fields
```

### Type Mismatch

```javascript
type StrictType { age: Int }

ExtractData("Age is thirty")
// BAML attempts coercion "thirty" → 30
// Or returns error if coercion fails
```

## Performance Tips

### 1. Use Parallel for Independent Queries

```javascript
// Slow (3 LLM calls sequentially)
Ask("Question 1") >> Ask("Question 2") >> Ask("Question 3")

// Fast (3 LLM calls in parallel)
(Ask("Q1") || Ask("Q2") || Ask("Q3"))
```

### 2. Batch Related Queries

```javascript
// Slow (3 separate calls)
ExtractA(text) >> ExtractB(text) >> ExtractC(text)

// Fast (1 call)
def ExtractAll(text: String) -> AllData {
  prompt: "Extract A, B, and C from: ${text}"
}
```

### 3. Use Lower Temperature for Factual

```javascript
def Extract(text: String) -> Data {
  temperature: 0.1        // Fast, deterministic, factual
}
```

### 4. Set Reasonable max_tokens

```javascript
def Summarize(text: String) {
  max_tokens: 150         // Limits response size
}
```

## Debugging

### Debug Mode

```javascript
:debug
```

Shows generated prompts and raw LLM responses.

### Inspect Prompts

Use `:funcs` to see function configuration:

```javascript
:funcs
```

## Limitations

1. **OpenAI Only** - Currently only OpenAI models supported
2. **No Streaming UI** - Responses appear when complete
3. **No Context Memory** - Each call is independent
4. **No Function Calling** - Standard prompting only

## Next Steps

- **[05-Type-System.md](05-Type-System.md)** - Define types for structured outputs
- **[06-Functions.md](06-Functions.md)** - Create LLM functions
- **[07-Workflow-Constructs.md](07-Workflow-Constructs.md)** - Build LLM workflows
- **[examples/](../examples/)** - LLM workflow examples

## Related Documents

- **[EXTRACTAS_EXAMPLE.md](../EXTRACTAS_EXAMPLE.md)** - Structured extraction examples
- **[EXAMPLES.md](../EXAMPLES.md)** - LLM function examples
- **[DESIGN.md](../DESIGN.md)** - LLM integration design
