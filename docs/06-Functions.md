# Functions

## Overview

Functions are the building blocks of workflows in the DSL. They encapsulate logic, enable reusability, and provide integration with external services (LLMs, HTTP APIs, SQL databases).

## Built-in Functions

### String Operations

#### Upper(str: String) -> String
Convert string to uppercase.

```javascript
Upper("hello")          // "HELLO"
Upper("world") as result
```

#### Lower(str: String) -> String
Convert string to lowercase.

```javascript
Lower("HELLO")          // "hello"
Lower("WORLD") as result
```

#### Length(str: String | List) -> Int
Get length of string or list.

```javascript
Length("hello")         // 5
Length([1, 2, 3])       // 3
```

#### Join(list: [String], separator: String) -> String
Join list elements with separator.

```javascript
Join(["a", "b", "c"], "-")      // "a-b-c"
Join(["hello", "world"], " ")   // "hello world"
```

### LLM Functions

#### Ask(prompt: String) -> String
Simple LLM query for unstructured responses.

```javascript
Ask("What is the capital of France?")
// "The capital of France is Paris."

Ask("Explain ${topic} in one sentence") as explanation
```

**Configuration:**
- Uses OpenAI GPT-4o-mini by default
- Requires OPENAI_API_KEY environment variable
- Returns plain text string

## User-Defined Functions

### Function Definition Syntax

```javascript
def functionName(param1: Type1, param2: Type2) -> ReturnType {
  // Function properties
  property: value

  // Execution block (one of: prompt, sql, http)
  prompt: "..."  // LLM function
  // OR
  sql: "..."     // SQL function
  // OR
  http: "GET"    // HTTP function
}
```

### LLM Functions

**Purpose:** Call LLMs with custom prompts and structured outputs.

**Basic Example:**
```javascript
def Greet(name: String) {
  model: "gpt-4o-mini"
  temperature: 0.7
  prompt: "Generate a friendly greeting for ${name}"
}

Greet("Alice")
// "Hello Alice! Great to meet you!"
```

**Structured Output Example:**
```javascript
type Person {
  name: String
  age: Int
  occupation: String
}

def ExtractPerson(text: String) -> Person {
  model: "gpt-4o-mini"
  temperature: 0.3
  prompt: "Extract person information from: ${text}"
}

ExtractPerson("Alice is a 28 year old engineer")
// { name: "Alice", age: 28, occupation: "engineer" }
```

**Using OpenRouter for Multiple Providers:**
```javascript
// Use Claude via OpenRouter with custom API key env var
def AnalyzeWithClaude(text: String) {
  base_url: "https://openrouter.ai/api/v1"
  model: "anthropic/claude-3.5-sonnet"
  api_key_env: "OPENROUTER_API_KEY"
  temperature: 0.7
  prompt: "Analyze the following text: ${text}"
}

// Use Gemini via OpenRouter
def AnalyzeWithGemini(text: String) {
  base_url: "https://openrouter.ai/api/v1"
  model: "google/gemini-pro"
  api_key_env: "OPENROUTER_API_KEY"
  prompt: "Analyze: ${text}"
}

// Use Llama via OpenRouter
def AnalyzeWithLlama(text: String) {
  base_url: "https://openrouter.ai/api/v1"
  model: "meta-llama/llama-3.1-70b-instruct"
  api_key_env: "OPENROUTER_API_KEY"
  prompt: "Analyze: ${text}"
}

// Use direct OpenAI (no base_url, defaults to OPENAI_API_KEY)
def AnalyzeWithGPT(text: String) {
  model: "gpt-4o-mini"
  prompt: "Analyze: ${text}"
}

// Use native Anthropic API with separate key
def AnalyzeWithNativeAnthropic(text: String) {
  base_url: "https://api.anthropic.com/v1"
  model: "claude-3-5-sonnet-20241022"
  api_key_env: "ANTHROPIC_API_KEY"
  prompt: "Analyze: ${text}"
}
```

**Properties:**
- `model` - LLM model name (e.g., "gpt-4o-mini", "anthropic/claude-3.5-sonnet")
- `base_url` - Optional API base URL (for OpenRouter or custom endpoints)
- `api_key_env` - Optional environment variable name for API key (defaults to "OPENAI_API_KEY")
- `temperature` - Randomness (0.0-2.0, default 0.7)
- `max_tokens` - Max response length
- `prompt` - Jinja2 template with `${variable}` interpolation

**Supported Providers:**
- **OpenAI** (default) - Requires `OPENAI_API_KEY` (or custom env var via `api_key_env`)
- **OpenRouter** - Recommended to use `OPENROUTER_API_KEY` with `api_key_env: "OPENROUTER_API_KEY"`
  - Access to 100+ models from Anthropic, Google, Meta, and more
  - Single API key for all providers
  - Get your key at: https://openrouter.ai/keys
  - Use `base_url: "https://openrouter.ai/api/v1"`
- **Anthropic** - Use `ANTHROPIC_API_KEY` with `api_key_env: "ANTHROPIC_API_KEY"`
  - Use `base_url: "https://api.anthropic.com/v1"`
- **Custom** - Any OpenAI-compatible endpoint

**Environment Variable Setup:**
```bash
# For OpenAI (default)
export OPENAI_API_KEY="sk-..."

# For OpenRouter
export OPENROUTER_API_KEY="sk-or-v1-..."

# For Anthropic
export ANTHROPIC_API_KEY="sk-ant-..."

# Or use all three simultaneously
export OPENAI_API_KEY="sk-..."
export OPENROUTER_API_KEY="sk-or-v1-..."
export ANTHROPIC_API_KEY="sk-ant-..."
```

### SQL Functions

**Purpose:** Transform data using SQL queries.

**Example:**
```javascript
def FilterHighScores(data: Table, threshold: Float) -> Table {
  sql: """
    SELECT *
    FROM data
    WHERE score >= ${threshold}
    ORDER BY score DESC
  """
}

SQL("SELECT * FROM 'data.csv'") as raw
  >> FilterHighScores(raw, 0.8) as filtered
```

**Features:**
- Uses DuckDB in-memory database
- Full SQL support (joins, aggregations, window functions)
- Template variable interpolation with `${variable}`
- Automatic table registration

### HTTP Functions

**Purpose:** Make HTTP API requests.

**GET Example:**
```javascript
def GetUser(id: Int) {
  http: "GET"
  url: "https://api.artic.edu/api/v1/artworks/${id}"
}

GetUser(129884) as user
user.config
user.config.website_url
```

**POST Example:**
```javascript
def CreatePost(title: String, content: String) {
  http: "POST"
  url: "https://api.example.com/posts"
  headers: {
    "Content-Type": "application/json"
  }
  body: {
    "title": "${title}",
    "content": "${content}"
  }
}

CreatePost("My Title", "My content") as result
```

**Properties:**
- `http` - Method: GET, POST, PUT, DELETE, PATCH, HEAD
- `url` - URL with `${variable}` interpolation
- `headers` - Optional request headers
- `params` - Optional query parameters
- `body` - Optional request body (JSON)

**Features:**
- Automatic JSON parsing
- 30-second timeout
- Error handling
- Parameter interpolation

### Hybrid Functions

Combine HTTP and LLM processing:

```javascript
def AnalyzeUserProfile(userId: Int) -> Analysis {
  http: "GET"
  url: "https://api.example.com/users/${userId}"

  model: "gpt-4o-mini"
  prompt: "Analyze this user profile and provide insights"
}

// Fetches user data, then sends to LLM for analysis
AnalyzeUserProfile(123)
```

**With OpenRouter:**
```javascript
def AnalyzeArtwork(artworkId: Int) -> Analysis {
  http: "GET"
  url: "https://api.artic.edu/api/v1/artworks/${artworkId}"

  base_url: "https://openrouter.ai/api/v1"
  model: "anthropic/claude-3.5-sonnet"
  api_key_env: "OPENROUTER_API_KEY"
  temperature: 0.8
  prompt: "Analyze this artwork data and provide insights about the artist, style, and historical context"
}

// Setup: export OPENROUTER_API_KEY="sk-or-v1-..."
// Fetches artwork data from API, then analyzes with Claude via OpenRouter
AnalyzeArtwork(129884)
```

## Function Calls

### Calling Functions

**Syntax:**
```javascript
functionName(arg1, arg2, ...)
```

**Examples:**
```javascript
// Built-in
Upper("hello")
Length([1, 2, 3])
Ask("What is AI?")

// User-defined
Greet("Alice")
ExtractPerson("Bob is 30")
GetUser(123)
```

### With Sequential Operator

```javascript
"hello" >> Upper(_) >> Length(_)
// 5

GetUser(1)
  >> ExtractInfo(_) as info
  >> Analyze(info)
```

### With Parallel Operator

```javascript
(GetUser(1) || GetUser(2) || GetUser(3)) as users

(
  Ask("Name a color") ||
  Ask("Name a fruit") ||
  Ask("Name an animal")
) as [color, fruit, animal]
```

## Template Strings

### Variable Interpolation

Use `${variable}` to inject values:

```javascript
"Alice" as name
"Hello, ${name}!"
// "Hello, Alice!"

def Greet(person: String) {
  prompt: "Greet ${person} warmly"
}
```

### In Prompts

```javascript
def Analyze(text: String, criteria: [String]) {
  prompt: """
    Analyze the following text:
    ${text}

    Criteria to consider:
    ${criteria}
  """
}
```

### In URLs

```javascript
def GetResource(id: Int, category: String) {
  http: "GET"
  url: "https://api.example.com/${category}/${id}"
}
```

## Function Management

### List Functions

```javascript
:funcs
```

**Output:**
```
User-defined functions:
  def Greet(name: String) {
    execution: LLM (gpt-4o-mini)
  }

  def GetUser(id: Int) {
    execution: HTTP GET https://api.example.com/users/${id}
  }
```

### Function Inspection

Functions show their execution type:
- **LLM** - Uses language model
- **HTTP** - Makes HTTP request
- **SQL** - Executes SQL query
- **Hybrid** - Combines multiple types

## Best Practices

### 1. Clear Function Names

**Good:**
```javascript
def ExtractPersonInfo(text: String) -> Person
def AnalyzeUserBehavior(userId: Int) -> Analysis
```

**Bad:**
```javascript
def func1(x: String) -> Person
def do_stuff(id: Int) -> Analysis
```

### 2. Typed Return Values

**Good:**
```javascript
def GetArticle(id: Int) -> Article {
  http: "GET"
  url: "https://api.example.com/articles/${id}"
}
```

**Bad:**
```javascript
def GetArticle(id: Int) {
  http: "GET"
  url: "https://api.example.com/articles/${id}"
}
// Returns untyped Map
```

### 3. Descriptive Prompts

**Good:**
```javascript
def Summarize(text: String, maxWords: Int) -> String {
  prompt: """
    Summarize the following text in approximately ${maxWords} words.
    Focus on the main points and key takeaways.

    Text: ${text}
  """
}
```

**Bad:**
```javascript
def Summarize(text: String) {
  prompt: "Summarize: ${text}"
}
```

### 4. Error Handling

Functions automatically handle errors:

```javascript
GetUser(999999)
// ✗ Error: HTTP 404 Not Found

Ask("")
// ✗ Error: Empty prompt
```

## Common Patterns

### Pattern 1: Data Pipeline

```javascript
def LoadData(file: String) -> Table {
  sql: "SELECT * FROM '${file}'"
}

def CleanData(data: Table) -> Table {
  sql: """
    SELECT * FROM data
    WHERE value IS NOT NULL
  """
}

def AnalyzeData(data: Table) -> Report {
  prompt: "Analyze this data and provide insights: ${data}"
}

LoadData("data.csv")
  >> CleanData(_) as clean
  >> AnalyzeData(clean)
```

### Pattern 2: Multi-Source Aggregation

```javascript
def FetchWeather(city: String) {
  http: "GET"
  url: "https://api.weather.com/v1/current?q=${city}"
}

def FetchNews(city: String) {
  http: "GET"
  url: "https://api.news.com/v1/local?q=${city}"
}

def CombineInfo(weather: Map, news: Map) -> CityReport {
  prompt: """
    Create a city report combining:
    Weather: ${weather}
    News: ${news}
  """
}

(FetchWeather("NYC") || FetchNews("NYC")) as [w, n]
  >> CombineInfo(w, n)
```

### Pattern 3: Iterative Refinement

```javascript
def Draft(topic: String) -> Article {
  prompt: "Write a draft article about: ${topic}"
}

def Review(article: Article) -> Feedback {
  prompt: "Review this article and provide feedback: ${article}"
}

def Revise(article: Article, feedback: Feedback) -> Article {
  prompt: """
    Revise this article based on feedback:

    Article: ${article}
    Feedback: ${feedback}
  """
}

Draft("AI Ethics")
  >> Review(_) as feedback
  >> Revise(_, feedback)
```

## Advanced Features

### Nested Function Calls

```javascript
Upper(Join(["a", "b", "c"], "-"))
// "A-B-C"

Analyze(ExtractInfo(GetUser(123)))
```

### Function Composition

```javascript
def ProcessUser(id: Int) -> Report {
  prompt: "Process user data"
}

GetUser(123) >> ProcessUser(_) as report
```

### Conditional Execution (Future)

```javascript
GetScore(data) as score
  >> score > 0.8
     ? ProcessHigh(data)
     : ProcessLow(data)
```

## Limitations

### Current Limitations

1. **No Recursion** - Functions cannot call themselves
2. **No Closures** - Functions cannot capture outer scope
3. **No Higher-Order Functions** - Cannot pass functions as arguments
4. **No Overloading** - One function per name

### Workarounds

**Instead of recursion:**
```javascript
// Use SQL or iteration in prompt
SQL("WITH RECURSIVE ...")
```

**Instead of closures:**
```javascript
// Pass all needed data as parameters
def Process(data: String, context: String) { ... }
```

## Next Steps

- **[07-Workflow-Constructs.md](07-Workflow-Constructs.md)** - Combine functions into workflows
- **[08-LLM-Integration.md](08-LLM-Integration.md)** - Deep dive into LLM features
- **[09-SQL-DuckDB.md](09-SQL-DuckDB.md)** - SQL function details
- **[10-HTTP-Client.md](10-HTTP-Client.md)** - HTTP function details
- **[examples/](../examples/)** - Function examples

## Related Documents

- **[EXAMPLES.md](../EXAMPLES.md)** - Function examples
- **[HTTP_QUICK_START.md](../HTTP_QUICK_START.md)** - HTTP quick reference
- **[DESIGN.md](../DESIGN.md)** - Function design specification
