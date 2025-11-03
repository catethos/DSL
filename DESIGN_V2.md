# Flow Language Design Specification v2.0

**A Gleam-inspired functional language for AI-powered data workflows**

---

## Table of Contents

1. [Overview](#overview)
2. [Design Philosophy](#design-philosophy)
3. [Type System](#type-system)
4. [Pattern Matching](#pattern-matching)
5. [Functions](#functions)
6. [First-Class Effects](#first-class-effects)
7. [Error Handling](#error-handling)
8. [Control Flow](#control-flow)
9. [Operators](#operators)
10. [Grammar Specification](#grammar-specification)
11. [AST Specification](#ast-specification)
12. [Evaluation Semantics](#evaluation-semantics)
13. [Integration with simplify_baml](#integration-with-simplify_baml)
14. [Migration from v1](#migration-from-v1)
15. [Implementation Roadmap](#implementation-roadmap)

---

## 1. Overview

Flow is a functional programming language designed specifically for building AI-powered data workflows. It combines the elegance of Gleam's syntax with first-class support for LLM calls, SQL queries, and HTTP requests.

### Key Features

- **Algebraic Data Types (ADTs)** - Product types (records) and sum types (variants)
- **Pattern Matching** - Core control flow mechanism with guards
- **First-class Effects** - LLM, SQL, and HTTP as language primitives
- **Explicit Error Handling** - Result type for all fallible operations
- **Pipeline Operator** - Functional composition with `|>`
- **Immutable by Default** - Pure functional programming
- **Type-safe LLM Integration** - Structured outputs with field descriptions
- **REPL-first** - Interactive development experience

### Design Goals

1. **Make LLM/SQL/HTTP calls as natural as function calls**
2. **Prevent runtime errors through strong typing**
3. **Enable clear data transformations through pipelines**
4. **Provide helpful error messages**
5. **Integrate seamlessly with simplify_baml**

---

## 2. Design Philosophy

### 2.1 Rationale

#### Why Redesign?

The v1 DSL accumulated technical debt that made adding features (like pattern matching) increasingly difficult:

- **Weak type system** - Classes and enums existed but lacked proper ADT support
- **Control flow confusion** - Ternary operators, sequential composition, but no unified approach
- **Grammar complexity** - Ad-hoc additions without systematic design
- **Pattern matching bolted on** - Should have been core from the start

#### Why Gleam-inspired?

Gleam provides an excellent foundation for a data-oriented language:

1. **Expression-based** - Everything returns a value
2. **Pattern matching core** - Single, powerful control flow mechanism
3. **Clean syntax** - Readable and minimal
4. **Strong typing** - Catches errors early
5. **Functional** - Immutable, composable

We adapt Gleam's principles while adding unique features for AI workflows.

### 2.2 Core Principles

#### Everything is an Expression

Unlike languages with statements, Flow treats everything as an expression that returns a value:

```gleam
// If/else doesn't exist - use case
let status = case user.age {
  a if a < 18 -> "Minor"
  _ -> "Adult"
}

// Blocks return their last expression
let result = {
  let x = 10
  let y = 20
  x + y  // Returns 30
}
```

**Rationale**: Expressions compose better than statements and lead to more predictable code.

#### Pattern Matching is Core Control Flow

Flow has exactly **one** control flow construct: `case` expressions.

```gleam
case value {
  pattern1 -> result1
  pattern2 -> result2
  _ -> default
}
```

**Rationale**:
- Reduces cognitive load (one construct to learn)
- Forces exhaustive handling of cases
- More powerful than if/else chains
- Natural fit for ADTs

#### Immutable by Default

All bindings are immutable. There is no mutation or reassignment.

```gleam
let x = 10
// x = 20  // Error! Cannot reassign

// Instead, create new bindings
let x = x + 10  // Shadows previous x
```

**Rationale**:
- Easier to reason about code
- No accidental mutations
- Enables better optimizations
- Standard in functional languages

#### Explicit Error Handling

All fallible operations return `Result(value, error)`. No exceptions.

```gleam
// LLM calls can fail
fn ask(question: String) -> Result(String, String) {
  llm(question)
}

// Must handle both cases
case ask("What is Rust?") {
  Ok(answer) -> answer
  Error(msg) -> "Failed: " + msg
}
```

**Rationale**:
- Forces consideration of error cases
- Makes error paths visible in types
- Prevents unexpected crashes
- Standard in Rust, Gleam, Elm

---

## 3. Type System

### 3.1 Primitive Types

Flow has five primitive types:

```gleam
String  // "hello", "world"
Int     // 42, -10, 0
Float   // 3.14, -0.5, 2.0
Bool    // true, false
Nil     // Unit type, similar to void
```

**Design Decision**: We use `Nil` instead of `Null` to match Gleam and avoid confusion with nullable types.

### 3.2 Product Types (Records)

Product types combine multiple named fields:

```gleam
// Simple record type
type Point {
  x: Int
  y: Int
}

// With field descriptions for LLM guidance
type User {
  name: String "the full name of the user"
  email: String "valid email address"
  age: Int "age in years, use -1 if unknown"
}

// With optional fields
type Profile {
  username: String
  bio: String?
  avatar_url: String?
}

// Nested types
type Company {
  name: String
  ceo: User
  employees: List(User)
}
```

#### Field Descriptions

Field descriptions are **critical** for LLM integration. They guide the LLM in extracting structured data:

```gleam
type Resume {
  name: String "candidate's full name"
  experience_years: Int "total years of experience, -1 if unclear"
}
```

When this type is used as an LLM return type, simplify_baml generates:

```json
{
  name: string, // candidate's full name
  experience_years: int, // total years of experience, -1 if unclear
}
```

**Syntax Options**:

1. **Single-line**: `field: Type "description"`
2. **Multi-line**:
```gleam
field: Type """
  Long description
  spanning multiple lines
"""
```

**Grammar**:
```pest
field = { identifier ~ optional? ~ ":" ~ type_ref ~ description? }
description = { string_literal | triple_quoted_string }
optional = { "?" }
```

**AST Representation**:
```rust
pub struct Field {
    pub name: String,
    pub field_type: FieldType,
    pub optional: bool,
    pub description: Option<String>,
}
```

### 3.3 Sum Types (Variants)

Sum types represent values that can be one of several alternatives:

```gleam
// Simple enum
type Status {
  Pending
  Active
  Completed
}

// Variants with data
type Result(value, error) {
  Ok(value)
  Error(error)
}

type Option(value) {
  Some(value)
  None
}

// Complex variants
type HttpResponse {
  Success(data: String, status: Int)
  Error(message: String, code: Int)
  Timeout
}

// With descriptions
type UserStatus {
  Active "user account is active and verified"
  Suspended "account temporarily suspended"
  Banned "permanently banned from platform"
}
```

**Generic Types**: Sum types can have type parameters (like `Result(value, error)`).

**Variant Data**: Variants can carry:
- No data: `Pending`
- Anonymous data: `Some(value)`
- Named fields: `Success(data: String, status: Int)`

**Grammar**:
```pest
type_decl = {
  "type" ~ type_name ~ type_params? ~ "{" ~ variant ~ ("," ~ variant)* ~ ","? ~ "}"
}

type_params = { "(" ~ identifier ~ ("," ~ identifier)* ~ ")" }

variant = {
  variant_name ~ variant_data? ~ description?
}

variant_data = {
  "(" ~ variant_field ~ ("," ~ variant_field)* ~ ")"
}

variant_field = {
  (identifier ~ ":")? ~ type_ref
}
```

**AST Representation**:
```rust
pub struct TypeDecl {
    pub name: String,
    pub type_params: Vec<String>,
    pub variants: Vec<Variant>,
    pub description: Option<String>,
}

pub struct Variant {
    pub name: String,
    pub fields: Vec<VariantField>,
    pub description: Option<String>,
}

pub struct VariantField {
    pub name: Option<String>,  // None for anonymous fields
    pub field_type: FieldType,
}
```

### 3.4 Collection Types

#### Lists

Homogeneous, ordered collections:

```gleam
type List(element) {
  // Built-in, not user-defined
}

// Syntax
[1, 2, 3, 4, 5]
["hello", "world"]
[]  // empty list

// List of specific type
List(Int)
List(String)
List(User)
```

#### Maps

Not yet implemented. Use records for structured data.

**Rationale**: Records are more type-safe. We may add maps later for dynamic data.

### 3.5 Function Types

Functions have types that describe their inputs and outputs:

```gleam
// Function type syntax
fn(Int, String) -> Bool

// Example function
fn is_adult(age: Int, country: String) -> Bool {
  case country {
    "US" -> age >= 18
    "Japan" -> age >= 20
    _ -> age >= 18
  }
}
```

**Higher-order Functions**:
```gleam
// Functions can take functions as arguments
fn map(list: List(a), f: fn(a) -> b) -> List(b) {
  // Implementation
}

// Usage
[1, 2, 3] |> map(fn(x) { x * 2 })
```

### 3.6 Type Inference

Flow performs **local type inference** within function bodies:

```gleam
// Types can be inferred from context
let x = 42  // x: Int
let y = "hello"  // y: String
let z = [1, 2, 3]  // z: List(Int)

// But function signatures must be explicit
fn add(a: Int, b: Int) -> Int {
  a + b
}
```

**Rationale**: Explicit function signatures serve as documentation and prevent type errors from propagating.

---

## 4. Pattern Matching

### 4.1 Overview

Pattern matching is the **only** control flow mechanism in Flow. It replaces:
- if/else statements
- switch/case statements
- Null checks
- Type checks
- Destructuring

**Syntax**:
```gleam
case value {
  pattern1 if guard1 -> expression1
  pattern2 -> expression2
  _ -> default
}
```

### 4.2 Pattern Types

#### 4.2.1 Wildcard Pattern

Matches anything, discards value:

```gleam
case x {
  _ -> "matches everything"
}
```

#### 4.2.2 Literal Pattern

Matches exact values:

```gleam
case status {
  "pending" -> "Not started"
  "active" -> "In progress"
  42 -> "The answer"
  true -> "Yes"
  _ -> "Other"
}
```

#### 4.2.3 Binding Pattern

Binds value to a variable:

```gleam
case user {
  u -> u.name  // u is bound to user
}
```

#### 4.2.4 Variant Pattern

Matches sum type variants:

```gleam
case result {
  Ok(value) -> value
  Error(msg) -> "Failed: " + msg
}

// With named fields
case response {
  Success(data: d, status: 200) -> d
  Success(data: d, status: s) -> "Got: " + s
  Error(message: m, code: c) -> "Error " + c + ": " + m
  Timeout -> "Request timed out"
}
```

#### 4.2.5 Record Pattern

Matches and destructures records:

```gleam
case user {
  User(name: "admin", ..) -> "Admin user"
  User(name: n, age: a, ..) -> n + " is " + a + " years old"
}
```

The `..` syntax ignores remaining fields.

#### 4.2.6 List Pattern

Matches list structure:

```gleam
case items {
  [] -> "Empty"
  [x] -> "Single: " + x
  [first, second] -> "Two items"
  [first, ..rest] -> "Multiple, first: " + first
}
```

#### 4.2.7 Type Pattern

Matches based on runtime type (for dynamic scenarios):

```gleam
case value {
  x: Int -> "Number: " + x
  x: String -> "Text: " + x
  x: Bool -> "Boolean"
  _ -> "Unknown type"
}
```

**Note**: Only useful when dealing with untyped external data.

### 4.3 Guards

Guards add conditional checks to patterns:

```gleam
case user {
  User(age: a, ..) if a < 13 -> "Child"
  User(age: a, ..) if a < 20 -> "Teen"
  User(age: a, ..) if a < 60 -> "Adult"
  _ -> "Senior"
}
```

**Guard Restrictions**:

Guards can only contain:
- Comparisons: `==`, `!=`, `>`, `<`, `>=`, `<=`
- Boolean operators: `&&`, `||`, `!`
- Field access: `user.age`, `data.count`
- Variables: `a`, `x`, `result`
- Literals: `42`, `"hello"`, `true`

Guards **cannot** contain:
- Function calls
- Case expressions
- Blocks

**Rationale**: Keeps guards simple and predictable. Complex logic should go in the body.

**Grammar**:
```pest
guard = { "if" ~ guard_expr }

guard_expr = {
  comparison
  | boolean_op
  | field_access
  | variable
  | literal
}

comparison = {
  guard_expr ~ ("==" | "!=" | ">" | "<" | ">=" | "<=") ~ guard_expr
}

boolean_op = {
  guard_expr ~ ("&&" | "||") ~ guard_expr
  | "!" ~ guard_expr
}
```

### 4.4 Alternate Patterns

Multiple patterns can share the same arm using `|`:

```gleam
case status {
  "pending" | "waiting" | "queued" -> "Not started"
  "active" | "running" -> "In progress"
  "done" | "complete" -> "Finished"
  _ -> "Unknown"
}
```

**Grammar**:
```pest
pattern_list = { pattern ~ ("|" ~ pattern)* }
```

### 4.5 Exhaustiveness Checking

The compiler should check that all cases are covered:

```gleam
type Status {
  Pending
  Active
  Done
}

// Error: Missing case for Done
case status {
  Pending -> "pending"
  Active -> "active"
}

// OK: All cases covered
case status {
  Pending -> "pending"
  Active -> "active"
  Done -> "done"
}

// OK: Wildcard covers remaining
case status {
  Pending -> "pending"
  _ -> "other"
}
```

**Implementation**: Build decision tree and check coverage.

### 4.6 Pattern Matching Semantics

1. **Top-to-bottom evaluation**: Patterns are tried in order
2. **First match wins**: Once a pattern matches, evaluation stops
3. **Bindings scope to arm**: Variables bound in patterns are only available in that arm
4. **Guards evaluated after pattern**: Pattern must match, then guard is checked
5. **Runtime error on no match**: If no pattern matches and no wildcard, error

**Example Evaluation**:
```gleam
case [1, 2, 3] {
  [] -> "empty"           // Try: No match
  [x] -> "single"         // Try: No match
  [a, b] -> "pair"        // Try: No match
  [first, ..rest] -> "many"  // Try: Match! Bind first=1, rest=[2,3]
}
// Returns "many"
```

---

## 5. Functions

### 5.1 Function Definitions

Functions are first-class values:

```gleam
// Basic function
fn double(x: Int) -> Int {
  x * 2
}

// Multiple parameters
fn add(a: Int, b: Int) -> Int {
  a + b
}

// With pattern matching
fn describe_age(age: Int) -> String {
  case age {
    a if a < 13 -> "Child"
    a if a < 20 -> "Teen"
    a if a < 60 -> "Adult"
    _ -> "Senior"
  }
}

// Generic function
fn first(list: List(a)) -> Option(a) {
  case list {
    [x, ..] -> Some(x)
    [] -> None
  }
}
```

**Grammar**:
```pest
fn_decl = {
  "fn" ~ fn_name ~ type_params? ~ "(" ~ params ~ ")" ~ "->" ~ type_ref ~ block
}

type_params = { "(" ~ identifier ~ ("," ~ identifier)* ~ ")" }

params = { (param ~ ("," ~ param)*)? }

param = { identifier ~ ":" ~ type_ref }

block = { "{" ~ expr ~ "}" }
```

**AST**:
```rust
pub struct FunctionDecl {
    pub name: String,
    pub type_params: Vec<String>,
    pub params: Vec<Param>,
    pub return_type: FieldType,
    pub body: Box<Expr>,
}

pub struct Param {
    pub name: String,
    pub param_type: FieldType,
}
```

### 5.2 Anonymous Functions

Create functions inline:

```gleam
// Lambda syntax
fn(x) { x * 2 }
fn(a, b) { a + b }

// Usage with higher-order functions
[1, 2, 3] |> map(fn(x) { x * 2 })

// Multi-line
users |> filter(fn(user) {
  case user.age {
    a if a >= 18 -> true
    _ -> false
  }
})
```

**Grammar**:
```pest
fn_expr = {
  "fn" ~ "(" ~ params ~ ")" ~ block
}
```

### 5.3 Function Application

```gleam
// Direct call
double(5)
add(10, 20)

// With pipeline
5 |> double
```

### 5.4 Partial Application & Currying

Not yet implemented. May add in future.

---

## 6. First-Class Effects

### 6.1 Overview

Flow treats LLM calls, SQL queries, and HTTP requests as **first-class language primitives**. They are not library functions—they are part of the language itself.

**Design Rationale**:

1. **These are core to AI workflows** - Making them first-class emphasizes their importance
2. **Better error messages** - The compiler understands their structure
3. **Syntax optimization** - Custom syntax for common patterns
4. **Type integration** - Return types are checked at compile time
5. **Tooling support** - IDEs can provide better assistance

### 6.2 LLM Effect

#### Basic Syntax

```gleam
// Simple prompt
llm("What is the capital of France?")
// Returns: Result(String, String)

// Block syntax
llm {
  model: "gpt-4"
  prompt: "Explain quantum computing"
}
// Returns: Result(String, String)

// With all options
llm {
  model: "gpt-4"
  base_url: "https://api.openai.com/v1"
  api_key_env: "OPENAI_API_KEY"
  temperature: 0.7
  prompt: "Write a haiku about Rust"
}
```

#### Structured Output

The killer feature: type-safe structured extraction:

```gleam
type User {
  name: String "full name"
  age: Int "age in years, -1 if unknown"
  email: String "email address"
}

// Extract structured data
llm {
  model: "gpt-4"
  return_type: User
  prompt: """
    Extract user info from this text:
    John Doe, 30 years old, john@example.com
  """
}
// Returns: Result(User, String)
```

**How It Works**:

1. Compiler sees `return_type: User`
2. Looks up `User` type definition
3. Generates JSON schema with field descriptions
4. Passes to simplify_baml runtime
5. Runtime appends schema to prompt:
```
Extract user info from this text:
John Doe, 30 years old, john@example.com

Answer in JSON using this schema:
{
  name: string, // full name
  age: int, // age in years, -1 if unknown
  email: string, // email address
}
```
6. Parses LLM response into `User` type
7. Returns `Ok(user)` or `Error(msg)`

#### Grammar

```pest
llm_expr = {
  "llm" ~ "(" ~ expr ~ ")"
  | "llm" ~ "{" ~ llm_prop ~ ("," ~ llm_prop)* ~ ","? ~ "}"
}

llm_prop = {
  "model" ~ ":" ~ expr
  | "base_url" ~ ":" ~ expr
  | "api_key_env" ~ ":" ~ expr
  | "temperature" ~ ":" ~ expr
  | "prompt" ~ ":" ~ expr
  | "return_type" ~ ":" ~ type_ref
}
```

#### AST

```rust
pub enum LLMExpr {
    Simple(Box<Expr>),  // llm("prompt")
    Block {
        model: Option<String>,
        base_url: Option<String>,
        api_key_env: Option<String>,
        temperature: Option<f64>,
        prompt: Box<Expr>,
        return_type: Option<FieldType>,
    },
}
```

#### Evaluation Semantics

```rust
async fn eval_llm(&mut self, llm_expr: &LLMExpr) -> Result<Value, String> {
    match llm_expr {
        LLMExpr::Simple(prompt_expr) => {
            let prompt = self.eval_expr(prompt_expr).await?;
            let prompt_str = prompt.to_string();

            self.runtime.call_llm(
                prompt_str,
                None,  // default model
                None,  // return string
            ).await
        }

        LLMExpr::Block { model, base_url, api_key_env, temperature, prompt, return_type } => {
            let prompt_str = self.eval_expr(prompt).await?.to_string();

            if let Some(ret_type) = return_type {
                // Structured output
                self.runtime.call_llm_structured(
                    prompt_str,
                    model.clone(),
                    ret_type.clone(),
                    base_url.clone(),
                    api_key_env.clone(),
                    *temperature,
                ).await
            } else {
                // String output
                self.runtime.call_llm(
                    prompt_str,
                    model.clone(),
                    None,
                ).await
            }
        }
    }
}
```

### 6.3 SQL Effect

#### Basic Syntax

```gleam
// Simple query
sql("SELECT * FROM users WHERE age > 18")
// Returns: Result(List(Map), String)

// With parameters (safe interpolation)
let min_age = 18
sql("SELECT * FROM users WHERE age > ${min_age}")

// Create table from data
sql_create_table("temp_users", users)
```

#### Type-Safe Results

If the SQL query returns data matching a type, it can be automatically parsed:

```gleam
type User {
  name: String
  email: String
  age: Int
}

// Returns List(User) if columns match
sql("SELECT name, email, age FROM users")
  |> result.map(fn(rows) {
    // rows is List(Map)
    // Can be converted to List(User)
  })
```

#### Grammar

```pest
sql_expr = {
  "sql" ~ "(" ~ expr ~ ")"
  | "sql_create_table" ~ "(" ~ expr ~ "," ~ expr ~ ")"
}
```

#### AST

```rust
pub enum SQLExpr {
    Query(Box<Expr>),
    CreateTable {
        table_name: Box<Expr>,
        data: Box<Expr>,
    },
}
```

### 6.4 HTTP Effect

#### Basic Syntax

```gleam
// Simple GET
http.get("https://api.example.com/users")
// Returns: Result(String, String)

// Other methods
http.post(url, body)
http.put(url, body)
http.delete(url)
http.patch(url, body)

// Full configuration
http {
  method: "POST"
  url: "https://api.example.com/users"
  headers: {
    "Content-Type": "application/json"
    "Authorization": "Bearer ${token}"
  }
  body: """{"name": "Alice"}"""
}
```

#### JSON Parsing

HTTP responses are strings, but can be parsed:

```gleam
http.get("https://api.example.com/user/123")
  |> result.then(fn(response) {
    json_parse(response)  // Parse JSON string
  })
  |> result.map(fn(json) {
    // Use json data
  })
```

#### Grammar

```pest
http_expr = {
  "http" ~ "." ~ http_method ~ "(" ~ expr ~ ("," ~ expr)? ~ ")"
  | "http" ~ "{" ~ http_prop ~ ("," ~ http_prop)* ~ ","? ~ "}"
}

http_method = { "get" | "post" | "put" | "delete" | "patch" | "head" }

http_prop = {
  "method" ~ ":" ~ expr
  | "url" ~ ":" ~ expr
  | "headers" ~ ":" ~ record_expr
  | "body" ~ ":" ~ expr
}
```

#### AST

```rust
pub enum HTTPExpr {
    Method {
        method: HTTPMethod,
        url: Box<Expr>,
        body: Option<Box<Expr>>,
    },
    Block {
        method: String,
        url: Box<Expr>,
        headers: Option<RecordExpr>,
        body: Option<Box<Expr>>,
    },
}

pub enum HTTPMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
}
```

---

## 7. Error Handling

### 7.1 The Result Type

All fallible operations return `Result(value, error)`:

```gleam
type Result(value, error) {
  Ok(value)
  Error(error)
}
```

This is a built-in sum type.

### 7.2 Handling Results

Use pattern matching:

```gleam
case http.get(url) {
  Ok(data) -> data
  Error(msg) -> "Failed: " + msg
}
```

### 7.3 Result Combinators

Standard library provides helpers:

```gleam
// Map over success value
result.map(Ok(5), fn(x) { x * 2 })
// -> Ok(10)

// Chain operations
result.then(Ok(5), fn(x) {
  if x > 0 {
    Ok(x * 2)
  } else {
    Error("Negative")
  }
})
// -> Ok(10)

// Map error
result.map_error(Error("failed"), fn(e) {
  "Error: " + e
})
// -> Error("Error: failed")

// Unwrap with default
result.unwrap_or(Error("failed"), "default")
// -> "default"
```

### 7.4 The try Keyword

Syntactic sugar for early returns:

```gleam
fn workflow() -> Result(String, String) {
  let users = try get_users()  // If Error, return early
  let analysis = try analyze(users)
  Ok(analysis)
}

// Desugars to:
fn workflow() -> Result(String, String) {
  case get_users() {
    Ok(users) -> case analyze(users) {
      Ok(analysis) -> Ok(analysis)
      Error(e) -> Error(e)
    }
    Error(e) -> Error(e)
  }
}
```

**Grammar**:
```pest
try_expr = { "try" ~ expr }
```

**AST**:
```rust
pub enum Expr {
    Try(Box<Expr>),
    // ...
}
```

**Evaluation**:
```rust
async fn eval_expr(&mut self, expr: &Expr) -> Result<Value, String> {
    match expr {
        Expr::Try(inner) => {
            let result = self.eval_expr(inner).await?;
            match result {
                Value::Variant("Ok", values) => Ok(values[0].clone()),
                Value::Variant("Error", values) => Err(values[0].to_string()),
                _ => Err("try expects Result type".to_string()),
            }
        }
        // ...
    }
}
```

### 7.5 The Option Type

For values that may be absent:

```gleam
type Option(value) {
  Some(value)
  None
}

// Usage
fn first(list: List(a)) -> Option(a) {
  case list {
    [x, ..] -> Some(x)
    [] -> None
  }
}
```

### 7.6 Panic (Discouraged)

For unrecoverable errors:

```gleam
panic("This should never happen")
// Terminates execution
```

**Use sparingly**. Prefer Result for recoverable errors.

---

## 8. Control Flow

### 8.1 Case Expressions

The only control flow construct (see [Pattern Matching](#4-pattern-matching)).

### 8.2 No If/Else

Flow deliberately omits if/else. Use case instead:

```gleam
// Instead of:
// if condition {
//   true_branch
// } else {
//   false_branch
// }

// Use:
case condition {
  true -> true_branch
  false -> false_branch
}
```

**Rationale**:
- One less construct to learn
- Forces consistency
- Case is more powerful anyway

### 8.3 Loops

No loops. Use recursion or higher-order functions:

```gleam
// Recursion
fn sum(list: List(Int)) -> Int {
  case list {
    [] -> 0
    [x, ..rest] -> x + sum(rest)
  }
}

// Higher-order functions (preferred)
fn sum(list: List(Int)) -> Int {
  list |> fold(0, fn(acc, x) { acc + x })
}
```

**Rationale**:
- Immutability makes loops awkward
- Recursion and HOFs are idiomatic
- Tail-call optimization handles performance

---

## 9. Operators

### 9.1 Arithmetic Operators

```gleam
+ - * /   // Int and Float
%         // Modulo (Int only)
```

**Type Rules**:
- `Int + Int -> Int`
- `Float + Float -> Float`
- `Int + Float -> Float` (automatic promotion)

### 9.2 Comparison Operators

```gleam
== != < > <= >=
```

**Type Rules**:
- Can compare same types
- Returns Bool

### 9.3 Boolean Operators

```gleam
&& || !
```

**Type Rules**:
- `Bool && Bool -> Bool`
- Short-circuiting evaluation

### 9.4 String Operators

```gleam
+   // Concatenation
```

**Type Rules**:
- `String + String -> String`

### 9.5 Pipeline Operator

The most important operator in Flow:

```gleam
|>  // Pipe forward
```

**Semantics**:
```gleam
value |> function(arg1, arg2)
// Becomes:
function(value, arg1, arg2)
```

**Example**:
```gleam
"hello"
|> string.uppercase
|> string.reverse
|> string.length
// -> 5
```

**Multi-line Pipelines**:
```gleam
users
|> filter(fn(u) { u.age > 18 })
|> map(fn(u) { u.name })
|> sort()
|> take(10)
```

**With Results**:
```gleam
http.get(url)
|> result.map(parse_json)
|> result.then(validate)
|> result.map(process)
|> result.unwrap_or(default)
```

**Grammar**:
```pest
pipeline_expr = {
  primary ~ ("|>" ~ call_expr)*
}
```

### 9.6 Operator Precedence

From highest to lowest:

1. Function call: `f(x)`
2. Field access: `x.field`
3. Unary: `!x`, `-x`
4. Multiplicative: `*`, `/`, `%`
5. Additive: `+`, `-`
6. Comparison: `==`, `!=`, `<`, `>`, `<=`, `>=`
7. Boolean AND: `&&`
8. Boolean OR: `||`
9. Pipeline: `|>`

---

## 10. Grammar Specification

Complete Pest grammar for Flow:

```pest
// ============================================
// Program Structure
// ============================================

program = { SOI ~ (declaration | expr)* ~ EOI }

declaration = {
  type_decl
  | fn_decl
}

// ============================================
// Type Declarations
// ============================================

type_decl = {
  type_description? ~ "type" ~ type_name ~ type_params? ~ "{" ~ variant ~ ("," ~ variant)* ~ ","? ~ "}"
}

type_description = { string_literal | triple_quoted_string }

type_params = { "(" ~ identifier ~ ("," ~ identifier)* ~ ")" }

variant = {
  variant_name ~ variant_data? ~ description?
}

variant_data = {
  "(" ~ variant_field ~ ("," ~ variant_field)* ~ ")"
}

variant_field = {
  (identifier ~ ":")? ~ type_ref
}

description = { string_literal | triple_quoted_string }

// Type references
type_ref = {
  type_name ~ type_args?
  | list_type
  | fn_type
}

type_args = { "(" ~ type_ref ~ ("," ~ type_ref)* ~ ")" }

list_type = { "[" ~ type_ref ~ "]" }

fn_type = { "fn" ~ "(" ~ (type_ref ~ ("," ~ type_ref)*)? ~ ")" ~ "->" ~ type_ref }

// ============================================
// Function Declarations
// ============================================

fn_decl = {
  "fn" ~ fn_name ~ type_params? ~ "(" ~ params ~ ")" ~ "->" ~ type_ref ~ block
}

params = { (param ~ ("," ~ param)*)? }

param = { identifier ~ ":" ~ type_ref }

block = { "{" ~ expr ~ "}" }

// ============================================
// Expressions
// ============================================

expr = {
  case_expr
  | let_expr
  | try_expr
  | pipeline_expr
}

// Case expression
case_expr = {
  "case" ~ expr ~ "{" ~ case_arm ~ ("," ~ case_arm)* ~ ","? ~ "}"
}

case_arm = {
  pattern_list ~ guard? ~ "->" ~ expr
}

pattern_list = {
  pattern ~ ("|" ~ pattern)*
}

guard = { "if" ~ guard_expr }

guard_expr = {
  comparison
  | boolean_op
  | field_access_expr
  | variable
  | literal
}

comparison = {
  guard_expr ~ ("==" | "!=" | ">=" | "<=" | ">" | "<") ~ guard_expr
}

boolean_op = {
  guard_expr ~ ("&&" | "||") ~ guard_expr
  | "!" ~ guard_expr
}

// Patterns
pattern = {
  variant_pattern
  | record_pattern
  | list_pattern
  | type_pattern
  | literal_pattern
  | binding_pattern
  | wildcard_pattern
}

wildcard_pattern = { "_" }

binding_pattern = { identifier }

literal_pattern = { literal }

type_pattern = { identifier ~ ":" ~ type_name }

variant_pattern = {
  variant_name ~ "(" ~ pattern_field ~ ("," ~ pattern_field)* ~ ("," ~ "..")? ~ ")"
}

pattern_field = {
  (identifier ~ ":")? ~ pattern
}

record_pattern = {
  type_name ~ "{" ~ record_pattern_field ~ ("," ~ record_pattern_field)* ~ ("," ~ "..")? ~ "}"
}

record_pattern_field = {
  identifier ~ ":" ~ pattern
}

list_pattern = {
  "[" ~ "]"
  | "[" ~ pattern ~ ("," ~ pattern)* ~ ("," ~ ".." ~ identifier?)? ~ "]"
}

// Let binding
let_expr = {
  "let" ~ pattern ~ "=" ~ expr ~ expr
}

// Try expression
try_expr = { "try" ~ expr }

// Pipeline
pipeline_expr = {
  or_expr ~ ("|>" ~ call_expr)*
}

// Boolean OR
or_expr = {
  and_expr ~ ("||" ~ and_expr)*
}

// Boolean AND
and_expr = {
  comparison_expr ~ ("&&" ~ comparison_expr)*
}

// Comparison
comparison_expr = {
  additive_expr ~ (("==" | "!=" | ">=" | "<=" | ">" | "<") ~ additive_expr)?
}

// Arithmetic
additive_expr = {
  multiplicative_expr ~ (("+"|"-") ~ multiplicative_expr)*
}

multiplicative_expr = {
  unary_expr ~ (("*"|"/"|"%") ~ unary_expr)*
}

unary_expr = {
  ("!"|"-") ~ unary_expr
  | postfix_expr
}

postfix_expr = {
  primary ~ (field_access | index_access | call_args)*
}

field_access = { "." ~ identifier }

index_access = { "[" ~ expr ~ "]" }

call_args = { "(" ~ args ~ ")" }

args = { (expr ~ ("," ~ expr)*)? }

// Primary expressions
primary = {
  llm_expr
  | sql_expr
  | http_expr
  | fn_expr
  | record_expr
  | list_expr
  | literal
  | identifier
  | paren_expr
}

// LLM expression
llm_expr = {
  "llm" ~ "(" ~ expr ~ ")"
  | "llm" ~ "{" ~ llm_prop ~ ("," ~ llm_prop)* ~ ","? ~ "}"
}

llm_prop = {
  "model" ~ ":" ~ expr
  | "base_url" ~ ":" ~ expr
  | "api_key_env" ~ ":" ~ expr
  | "temperature" ~ ":" ~ expr
  | "prompt" ~ ":" ~ expr
  | "return_type" ~ ":" ~ type_ref
}

// SQL expression
sql_expr = {
  "sql" ~ "(" ~ expr ~ ")"
  | "sql_create_table" ~ "(" ~ expr ~ "," ~ expr ~ ")"
}

// HTTP expression
http_expr = {
  "http" ~ "." ~ http_method ~ "(" ~ expr ~ ("," ~ expr)? ~ ")"
  | "http" ~ "{" ~ http_prop ~ ("," ~ http_prop)* ~ ","? ~ "}"
}

http_method = { "get" | "post" | "put" | "delete" | "patch" | "head" }

http_prop = {
  "method" ~ ":" ~ expr
  | "url" ~ ":" ~ expr
  | "headers" ~ ":" ~ record_expr
  | "body" ~ ":" ~ expr
}

// Function expression
fn_expr = {
  "fn" ~ "(" ~ params ~ ")" ~ block
}

// Record expression
record_expr = {
  type_name ~ "{" ~ record_field ~ ("," ~ record_field)* ~ ","? ~ "}"
}

record_field = {
  identifier ~ ":" ~ expr
}

// List expression
list_expr = {
  "[" ~ (expr ~ ("," ~ expr)*)? ~ ","? ~ "]"
}

// Parenthesized expression
paren_expr = { "(" ~ expr ~ ")" }

// ============================================
// Literals
// ============================================

literal = {
  string_literal
  | triple_quoted_string
  | float
  | integer
  | boolean
}

string_literal = ${
  "\"" ~ string_inner ~ "\""
}

string_inner = @{
  string_char*
}

string_char = {
  !("\"" | "\\") ~ ANY
  | "\\" ~ ("\"" | "\\" | "/" | "b" | "f" | "n" | "r" | "t")
  | "\\" ~ ("u" ~ ASCII_HEX_DIGIT{4})
}

triple_quoted_string = ${
  "\"\"\"" ~ triple_string_inner ~ "\"\"\""
}

triple_string_inner = @{
  (!"\"\"\"" ~ ANY)*
}

integer = @{
  "-"? ~ ASCII_DIGIT+
}

float = @{
  "-"? ~ ASCII_DIGIT+ ~ "." ~ ASCII_DIGIT+
}

boolean = { "true" | "false" }

// ============================================
// Identifiers
// ============================================

identifier = @{
  (ASCII_ALPHA_LOWER | "_") ~ (ASCII_ALPHANUMERIC | "_")*
}

type_name = @{
  ASCII_ALPHA_UPPER ~ (ASCII_ALPHANUMERIC | "_")*
}

variant_name = @{
  ASCII_ALPHA_UPPER ~ (ASCII_ALPHANUMERIC | "_")*
}

fn_name = @{
  (ASCII_ALPHA_LOWER | "_") ~ (ASCII_ALPHANUMERIC | "_")*
}

// ============================================
// Whitespace & Comments
// ============================================

WHITESPACE = _{ " " | "\t" | "\r" | "\n" }

COMMENT = _{
  "//" ~ (!"\n" ~ ANY)* ~ "\n"
  | "/*" ~ (!"*/" ~ ANY)* ~ "*/"
}
```

---

## 11. AST Specification

Complete AST in Rust:

```rust
use std::collections::HashMap;

// ============================================
// Program
// ============================================

pub struct Program {
    pub declarations: Vec<Declaration>,
    pub expressions: Vec<Expr>,
}

pub enum Declaration {
    Type(TypeDecl),
    Function(FunctionDecl),
}

// ============================================
// Type Declarations
// ============================================

pub struct TypeDecl {
    pub name: String,
    pub type_params: Vec<String>,
    pub variants: Vec<Variant>,
    pub description: Option<String>,
}

pub struct Variant {
    pub name: String,
    pub fields: Vec<VariantField>,
    pub description: Option<String>,
}

pub struct VariantField {
    pub name: Option<String>,
    pub field_type: FieldType,
}

pub enum FieldType {
    // Primitives
    String,
    Int,
    Float,
    Bool,
    Nil,

    // Named types
    Named(String),

    // Generic instantiation
    Generic {
        name: String,
        args: Vec<FieldType>,
    },

    // Collections
    List(Box<FieldType>),

    // Functions
    Function {
        params: Vec<FieldType>,
        return_type: Box<FieldType>,
    },

    // Type variable
    TypeVar(String),
}

// ============================================
// Function Declarations
// ============================================

pub struct FunctionDecl {
    pub name: String,
    pub type_params: Vec<String>,
    pub params: Vec<Param>,
    pub return_type: FieldType,
    pub body: Box<Expr>,
}

pub struct Param {
    pub name: String,
    pub param_type: FieldType,
}

// ============================================
// Expressions
// ============================================

pub enum Expr {
    // Literals
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),

    // Variables
    Variable(String),

    // Collections
    List(Vec<Expr>),
    Record {
        type_name: String,
        fields: Vec<(String, Expr)>,
    },

    // Control flow
    Case {
        scrutinee: Box<Expr>,
        arms: Vec<CaseArm>,
    },

    // Bindings
    Let {
        pattern: Pattern,
        value: Box<Expr>,
        body: Box<Expr>,
    },

    // Error handling
    Try(Box<Expr>),

    // Functions
    Function {
        params: Vec<Param>,
        body: Box<Expr>,
    },

    Call {
        function: Box<Expr>,
        args: Vec<Expr>,
    },

    // Field access
    FieldAccess {
        base: Box<Expr>,
        field: String,
    },

    // Index access
    IndexAccess {
        base: Box<Expr>,
        index: Box<Expr>,
    },

    // Operators
    BinaryOp {
        op: BinaryOperator,
        left: Box<Expr>,
        right: Box<Expr>,
    },

    UnaryOp {
        op: UnaryOperator,
        operand: Box<Expr>,
    },

    // Pipeline
    Pipeline {
        value: Box<Expr>,
        function: Box<Expr>,
    },

    // Effects
    LLM(LLMExpr),
    SQL(SQLExpr),
    HTTP(HTTPExpr),

    // Block
    Block(Vec<Expr>),
}

// ============================================
// Case Expressions
// ============================================

pub struct CaseArm {
    pub patterns: Vec<Pattern>,
    pub guard: Option<Box<Expr>>,
    pub body: Box<Expr>,
}

pub enum Pattern {
    Wildcard,

    Binding(String),

    Literal(Literal),

    Type {
        binding: String,
        type_name: String,
    },

    Variant {
        name: String,
        fields: Vec<PatternField>,
        ignore_rest: bool,
    },

    Record {
        type_name: String,
        fields: Vec<(String, Pattern)>,
        ignore_rest: bool,
    },

    List {
        elements: Vec<Pattern>,
        rest: Option<String>,
    },
}

pub struct PatternField {
    pub name: Option<String>,
    pub pattern: Pattern,
}

pub enum Literal {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

// ============================================
// Operators
// ============================================

pub enum BinaryOperator {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,

    // Comparison
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,

    // Boolean
    And,
    Or,
}

pub enum UnaryOperator {
    Negate,
    Not,
}

// ============================================
// Effects
// ============================================

pub enum LLMExpr {
    Simple(Box<Expr>),
    Block {
        model: Option<String>,
        base_url: Option<String>,
        api_key_env: Option<String>,
        temperature: Option<f64>,
        prompt: Box<Expr>,
        return_type: Option<FieldType>,
    },
}

pub enum SQLExpr {
    Query(Box<Expr>),
    CreateTable {
        table_name: Box<Expr>,
        data: Box<Expr>,
    },
}

pub enum HTTPExpr {
    Method {
        method: HTTPMethod,
        url: Box<Expr>,
        body: Option<Box<Expr>>,
    },
    Block {
        method: String,
        url: Box<Expr>,
        headers: Option<HashMap<String, Expr>>,
        body: Option<Box<Expr>>,
    },
}

pub enum HTTPMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
}
```

---

## 12. Evaluation Semantics

### 12.1 Value Representation

Runtime values:

```rust
use indexmap::IndexMap;

pub enum Value {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Nil,
    List(Vec<Value>),
    Record {
        type_name: String,
        fields: IndexMap<String, Value>,
    },
    Variant {
        type_name: String,
        variant: String,
        values: Vec<Value>,
    },
    Function {
        params: Vec<String>,
        body: Expr,
        env: Environment,
    },
}
```

### 12.2 Environment

```rust
use std::collections::HashMap;

pub struct Environment {
    scopes: Vec<HashMap<String, Value>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn bind(&mut self, name: String, value: Value) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, value);
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value);
            }
        }
        None
    }
}
```

### 12.3 Evaluator

```rust
pub struct Evaluator {
    env: Environment,
    types: TypeRegistry,
    runtime: BamlRuntime,
}

impl Evaluator {
    pub async fn eval(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::String(s) => Ok(Value::String(s.clone())),
            Expr::Int(n) => Ok(Value::Int(*n)),
            Expr::Float(f) => Ok(Value::Float(*f)),
            Expr::Bool(b) => Ok(Value::Bool(*b)),

            Expr::Variable(name) => {
                self.env.lookup(name)
                    .cloned()
                    .ok_or_else(|| format!("Variable '{}' not found", name))
            }

            Expr::List(items) => {
                let mut values = Vec::new();
                for item in items {
                    values.push(self.eval(item).await?);
                }
                Ok(Value::List(values))
            }

            Expr::Record { type_name, fields } => {
                let mut field_values = IndexMap::new();
                for (name, expr) in fields {
                    field_values.insert(name.clone(), self.eval(expr).await?);
                }
                Ok(Value::Record {
                    type_name: type_name.clone(),
                    fields: field_values,
                })
            }

            Expr::Case { scrutinee, arms } => {
                let value = self.eval(scrutinee).await?;
                self.eval_case(&value, arms).await
            }

            Expr::Let { pattern, value, body } => {
                let val = self.eval(value).await?;
                self.env.push_scope();
                self.bind_pattern(pattern, &val)?;
                let result = self.eval(body).await?;
                self.env.pop_scope();
                Ok(result)
            }

            Expr::Try(inner) => {
                let result = self.eval(inner).await?;
                match result {
                    Value::Variant { variant, values, .. } if variant == "Ok" => {
                        Ok(values[0].clone())
                    }
                    Value::Variant { variant, values, .. } if variant == "Error" => {
                        Err(values[0].to_string())
                    }
                    _ => Err("try expects Result type".to_string()),
                }
            }

            Expr::Call { function, args } => {
                let func_val = self.eval(function).await?;
                let arg_vals: Vec<Value> = {
                    let mut vals = Vec::new();
                    for arg in args {
                        vals.push(self.eval(arg).await?);
                    }
                    vals
                };
                self.apply_function(func_val, arg_vals).await
            }

            Expr::FieldAccess { base, field } => {
                let base_val = self.eval(base).await?;
                match base_val {
                    Value::Record { fields, .. } => {
                        fields.get(field)
                            .cloned()
                            .ok_or_else(|| format!("Field '{}' not found", field))
                    }
                    _ => Err("Cannot access field on non-record".to_string()),
                }
            }

            Expr::BinaryOp { op, left, right } => {
                let left_val = self.eval(left).await?;
                let right_val = self.eval(right).await?;
                self.apply_binary_op(op, left_val, right_val)
            }

            Expr::Pipeline { value, function } => {
                let val = self.eval(value).await?;

                // Function must be a call expression
                if let Expr::Call { function: f, args } = function.as_ref() {
                    let func_val = self.eval(f).await?;
                    let mut arg_vals = vec![val];
                    for arg in args {
                        arg_vals.push(self.eval(arg).await?);
                    }
                    self.apply_function(func_val, arg_vals).await
                } else {
                    Err("Pipeline right-hand side must be a call".to_string())
                }
            }

            Expr::LLM(llm_expr) => self.eval_llm(llm_expr).await,
            Expr::SQL(sql_expr) => self.eval_sql(sql_expr).await,
            Expr::HTTP(http_expr) => self.eval_http(http_expr).await,

            _ => todo!("Implement remaining expressions"),
        }
    }

    async fn eval_case(&mut self, value: &Value, arms: &[CaseArm]) -> Result<Value, String> {
        for arm in arms {
            for pattern in &arm.patterns {
                if let Some(bindings) = self.try_match_pattern(pattern, value)? {
                    // Apply bindings
                    self.env.push_scope();
                    for (name, val) in bindings {
                        self.env.bind(name, val);
                    }

                    // Check guard if present
                    let guard_passes = if let Some(guard) = &arm.guard {
                        let guard_val = self.eval(guard).await?;
                        match guard_val {
                            Value::Bool(b) => b,
                            _ => return Err("Guard must evaluate to Bool".to_string()),
                        }
                    } else {
                        true
                    };

                    if guard_passes {
                        let result = self.eval(&arm.body).await?;
                        self.env.pop_scope();
                        return Ok(result);
                    }

                    self.env.pop_scope();
                }
            }
        }

        Err("No pattern matched".to_string())
    }

    fn try_match_pattern(
        &self,
        pattern: &Pattern,
        value: &Value,
    ) -> Result<Option<Vec<(String, Value)>>, String> {
        match pattern {
            Pattern::Wildcard => Ok(Some(Vec::new())),

            Pattern::Binding(name) => {
                Ok(Some(vec![(name.clone(), value.clone())]))
            }

            Pattern::Literal(lit) => {
                let matches = match (lit, value) {
                    (Literal::String(s1), Value::String(s2)) => s1 == s2,
                    (Literal::Int(i1), Value::Int(i2)) => i1 == i2,
                    (Literal::Float(f1), Value::Float(f2)) => (f1 - f2).abs() < f64::EPSILON,
                    (Literal::Bool(b1), Value::Bool(b2)) => b1 == b2,
                    _ => false,
                };
                Ok(if matches { Some(Vec::new()) } else { None })
            }

            Pattern::Variant { name, fields, ignore_rest } => {
                if let Value::Variant { variant, values, .. } = value {
                    if variant == name {
                        if !ignore_rest && values.len() != fields.len() {
                            return Ok(None);
                        }

                        let mut bindings = Vec::new();
                        for (i, field) in fields.iter().enumerate() {
                            if let Some(val) = values.get(i) {
                                if let Some(field_bindings) =
                                    self.try_match_pattern(&field.pattern, val)?
                                {
                                    bindings.extend(field_bindings);
                                } else {
                                    return Ok(None);
                                }
                            } else {
                                return Ok(None);
                            }
                        }
                        Ok(Some(bindings))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }

            Pattern::List { elements, rest } => {
                if let Value::List(items) = value {
                    if items.len() < elements.len() {
                        return Ok(None);
                    }

                    if rest.is_none() && items.len() != elements.len() {
                        return Ok(None);
                    }

                    let mut bindings = Vec::new();

                    for (i, pat) in elements.iter().enumerate() {
                        if let Some(item_bindings) =
                            self.try_match_pattern(pat, &items[i])?
                        {
                            bindings.extend(item_bindings);
                        } else {
                            return Ok(None);
                        }
                    }

                    if let Some(rest_name) = rest {
                        let rest_items = items[elements.len()..].to_vec();
                        bindings.push((rest_name.clone(), Value::List(rest_items)));
                    }

                    Ok(Some(bindings))
                } else {
                    Ok(None)
                }
            }

            _ => todo!("Implement remaining patterns"),
        }
    }
}
```

---

## 13. Integration with simplify_baml

### 13.1 Type System Mapping

Flow types map directly to BAML IR:

```rust
use simplify_baml::{IR, Class, Field, FieldType as BamlFieldType, Enum};

pub struct TypeRegistry {
    types: HashMap<String, TypeDecl>,
}

impl TypeRegistry {
    pub fn to_baml_ir(&self) -> IR {
        let mut ir = IR::new();

        for (name, type_decl) in &self.types {
            // Convert to BAML Class
            if type_decl.is_record_type() {
                let variant = &type_decl.variants[0];

                let fields: Vec<Field> = variant.fields.iter().map(|f| {
                    Field {
                        name: f.name.clone().unwrap_or_default(),
                        field_type: self.convert_field_type(&f.field_type),
                        optional: false,  // TODO: handle optional
                        description: f.description.clone(),
                    }
                }).collect();

                ir.classes.push(Class {
                    name: name.clone(),
                    description: type_decl.description.clone(),
                    fields,
                });
            }

            // Convert to BAML Enum
            if type_decl.is_enum_type() {
                let values: Vec<String> = type_decl.variants.iter()
                    .map(|v| v.name.clone())
                    .collect();

                ir.enums.push(Enum {
                    name: name.clone(),
                    description: type_decl.description.clone(),
                    values,
                });
            }
        }

        ir
    }

    fn convert_field_type(&self, ft: &FieldType) -> BamlFieldType {
        match ft {
            FieldType::String => BamlFieldType::String,
            FieldType::Int => BamlFieldType::Int,
            FieldType::Float => BamlFieldType::Float,
            FieldType::Bool => BamlFieldType::Bool,
            FieldType::List(inner) => {
                BamlFieldType::List(Box::new(self.convert_field_type(inner)))
            }
            FieldType::Named(name) => {
                // Check if it's a class or enum
                if let Some(type_decl) = self.types.get(name) {
                    if type_decl.is_record_type() {
                        BamlFieldType::Class(name.clone())
                    } else if type_decl.is_enum_type() {
                        BamlFieldType::Enum(name.clone())
                    } else {
                        panic!("Unknown type: {}", name)
                    }
                } else {
                    panic!("Type not found: {}", name)
                }
            }
            _ => panic!("Unsupported field type"),
        }
    }
}
```

### 13.2 LLM Evaluation

```rust
async fn eval_llm(&mut self, llm_expr: &LLMExpr) -> Result<Value, String> {
    match llm_expr {
        LLMExpr::Simple(prompt_expr) => {
            let prompt = self.eval(prompt_expr).await?;
            let prompt_str = prompt.to_string();

            let result = self.runtime.call_llm_simple(prompt_str).await
                .map_err(|e| e.to_string())?;

            Ok(Value::Variant {
                type_name: "Result".to_string(),
                variant: "Ok".to_string(),
                values: vec![Value::String(result)],
            })
        }

        LLMExpr::Block { model, base_url, api_key_env, temperature, prompt, return_type } => {
            let prompt_str = self.eval(prompt).await?.to_string();

            if let Some(ret_type) = return_type {
                // Structured output
                let ir = self.types.to_baml_ir();
                let type_name = ret_type.get_name();

                let result = self.runtime.call_llm_structured(
                    prompt_str,
                    type_name,
                    &ir,
                    model.clone(),
                    base_url.clone(),
                    api_key_env.clone(),
                    *temperature,
                ).await.map_err(|e| e.to_string())?;

                // Convert BamlValue to Flow Value
                let value = self.baml_value_to_flow_value(result);

                Ok(Value::Variant {
                    type_name: "Result".to_string(),
                    variant: "Ok".to_string(),
                    values: vec![value],
                })
            } else {
                // String output
                let result = self.runtime.call_llm_simple(prompt_str).await
                    .map_err(|e| e.to_string())?;

                Ok(Value::Variant {
                    type_name: "Result".to_string(),
                    variant: "Ok".to_string(),
                    values: vec![Value::String(result)],
                })
            }
        }
    }
}

fn baml_value_to_flow_value(&self, baml_val: BamlValue) -> Value {
    match baml_val {
        BamlValue::String(s) => Value::String(s),
        BamlValue::Int(i) => Value::Int(i),
        BamlValue::Float(f) => Value::Float(f),
        BamlValue::Bool(b) => Value::Bool(b),
        BamlValue::List(items) => {
            Value::List(items.into_iter().map(|v| self.baml_value_to_flow_value(v)).collect())
        }
        BamlValue::Map(map) => {
            let mut fields = IndexMap::new();
            for (k, v) in map {
                fields.insert(k, self.baml_value_to_flow_value(v));
            }
            Value::Record {
                type_name: "Unknown".to_string(),  // TODO: infer from context
                fields,
            }
        }
        BamlValue::Null => Value::Nil,
    }
}
```

---

## 14. Migration from v1

### 14.1 Syntax Changes

| v1 Syntax | v2 Syntax | Notes |
|-----------|-----------|-------|
| `x ? a : b` | `case x { true -> a, false -> b }` | Ternary removed |
| `x >> f(_)` | `x \|> f()` | Pipeline operator changed |
| `a \|\| b` (parallel) | `parallel { let x = a, let y = b, [x, y] }` | Parallel is explicit |
| `type Person { name: string }` | `type Person { name: String }` | Type names capitalized |
| `def func(x)` | `fn func(x: Int) -> Int` | Explicit types required |
| `Ask("question")` | `llm("question")` | Builtin renamed |

### 14.2 Type System Changes

```gleam
// v1: Weak enums
enum Status { Pending, Active, Done }

// v2: Proper ADTs
type Status {
  Pending
  Active
  Done
}

// v1: No variant data
// (not possible)

// v2: Variants with data
type Result(value, error) {
  Ok(value)
  Error(error)
}
```

### 14.3 Error Handling Changes

```gleam
// v1: No structured error handling
Ask("question")  // Could fail silently or throw

// v2: Explicit Result type
case llm("question") {
  Ok(answer) -> answer
  Error(msg) -> "Failed: " + msg
}
```

### 14.4 Migration Strategy

1. **Add type annotations** to all function signatures
2. **Replace ternary operators** with case expressions
3. **Update parallel syntax** to use explicit parallel blocks
4. **Handle Result types** from all LLM/SQL/HTTP calls
5. **Capitalize type names** (String, Int, etc.)
6. **Add field descriptions** for better LLM extraction

---

## 15. Implementation Roadmap

### Phase 1: Core Language (Weeks 1-2)

**Week 1: Parser & AST**
- [ ] Implement new Pest grammar
- [ ] Build AST types
- [ ] Parser for type declarations
- [ ] Parser for function declarations
- [ ] Parser for expressions
- [ ] Tests for parser

**Week 2: Type System**
- [ ] Type registry
- [ ] Type checking (basic)
- [ ] Generic type instantiation
- [ ] Type inference (local)
- [ ] Tests for type system

### Phase 2: Evaluator (Weeks 3-4)

**Week 3: Basic Evaluation**
- [ ] Environment implementation
- [ ] Literal evaluation
- [ ] Variable lookup
- [ ] Function calls
- [ ] Binary/unary operators
- [ ] Tests for evaluator

**Week 4: Pattern Matching**
- [ ] Pattern matching algorithm
- [ ] Wildcard, binding, literal patterns
- [ ] Variant patterns
- [ ] List patterns
- [ ] Guards
- [ ] Exhaustiveness checking
- [ ] Tests for pattern matching

### Phase 3: Effects (Weeks 5-6)

**Week 5: LLM Integration**
- [ ] LLM expression evaluation
- [ ] Integration with simplify_baml
- [ ] Type-to-IR conversion
- [ ] Structured output parsing
- [ ] Error handling
- [ ] Tests for LLM calls

**Week 6: SQL & HTTP**
- [ ] SQL expression evaluation
- [ ] DuckDB integration
- [ ] HTTP expression evaluation
- [ ] Request/response handling
- [ ] Tests for SQL/HTTP

### Phase 4: Advanced Features (Weeks 7-8)

**Week 7: Pipeline & Standard Library**
- [ ] Pipeline operator
- [ ] Result combinators
- [ ] List functions (map, filter, fold)
- [ ] String functions
- [ ] Tests for stdlib

**Week 8: Error Handling & Try**
- [ ] Result type implementation
- [ ] Try expression
- [ ] Error propagation
- [ ] Error messages
- [ ] Tests for error handling

### Phase 5: Tooling (Weeks 9-10)

**Week 9: REPL**
- [ ] Update REPL for v2 syntax
- [ ] Command handling
- [ ] History
- [ ] Auto-completion
- [ ] Tests for REPL

**Week 10: Editor & UI**
- [ ] Syntax highlighting for v2
- [ ] Error display
- [ ] Type explorer
- [ ] Workflow preview
- [ ] Tests for UI

### Phase 6: Documentation & Polish (Week 11+)

- [ ] Language tutorial
- [ ] API documentation
- [ ] Example workflows
- [ ] Migration guide
- [ ] Performance optimization
- [ ] Bug fixes

---

## Appendix A: Standard Library

### A.1 Result Module

```gleam
// Map over Ok value
fn map(result: Result(a, e), f: fn(a) -> b) -> Result(b, e)

// Chain results
fn then(result: Result(a, e), f: fn(a) -> Result(b, e)) -> Result(b, e)

// Map over Error value
fn map_error(result: Result(a, e), f: fn(e) -> f) -> Result(a, f)

// Unwrap with default
fn unwrap_or(result: Result(a, e), default: a) -> a

// Convert List(Result) to Result(List)
fn sequence(results: List(Result(a, e))) -> Result(List(a), e)
```

### A.2 List Module

```gleam
// Map over list
fn map(list: List(a), f: fn(a) -> b) -> List(b)

// Filter list
fn filter(list: List(a), f: fn(a) -> Bool) -> List(a)

// Fold/reduce
fn fold(list: List(a), init: b, f: fn(b, a) -> b) -> b

// Length
fn length(list: List(a)) -> Int

// Take first n
fn take(list: List(a), n: Int) -> List(a)

// Drop first n
fn drop(list: List(a), n: Int) -> List(a)

// Find first matching
fn find(list: List(a), f: fn(a) -> Bool) -> Option(a)

// Check if any matches
fn any(list: List(a), f: fn(a) -> Bool) -> Bool

// Check if all match
fn all(list: List(a), f: fn(a) -> Bool) -> Bool

// Sort
fn sort(list: List(a)) -> List(a)

// Reverse
fn reverse(list: List(a)) -> List(a)
```

### A.3 String Module

```gleam
// Uppercase
fn uppercase(s: String) -> String

// Lowercase
fn lowercase(s: String) -> String

// Length
fn length(s: String) -> Int

// Concatenate
fn concat(strings: List(String)) -> String

// Split
fn split(s: String, delimiter: String) -> List(String)

// Join
fn join(strings: List(String), separator: String) -> String

// Trim
fn trim(s: String) -> String

// Contains
fn contains(s: String, substring: String) -> Bool

// Replace
fn replace(s: String, pattern: String, replacement: String) -> String
```

---

## Appendix B: Example Programs

### B.1 Resume Parser

```gleam
type Experience {
  company: String "company name"
  role: String "job title/role"
  years: Int "duration in years, round up if months > 6"
}

type Resume {
  name: String "candidate full name"
  email: String "email address, empty if not found"
  experience: List(Experience) "work history, most recent first"
}

fn parse_resume(text: String) -> Result(Resume, String) {
  llm {
    model: "gpt-4"
    return_type: Resume
    temperature: 0.1
    prompt: """
    Extract structured information from this resume.
    Follow the schema exactly.

    ${text}
    """
  }
}

fn validate_resume(resume: Resume) -> Result(Resume, String) {
  case resume {
    Resume(email: "", ..) -> Error("Missing email")
    Resume(experience: [], ..) -> Error("No experience")
    r -> Ok(r)
  }
}

fn main(resume_text: String) -> Result(Resume, String) {
  resume_text
  |> parse_resume
  |> result.then(validate_resume)
}
```

### B.2 Data Pipeline

```gleam
type User {
  id: Int
  name: String
  age: Int
  email: String
}

fn get_users() -> Result(List(User), String) {
  sql("SELECT id, name, age, email FROM users")
}

fn analyze_user(user: User) -> Result(String, String) {
  llm {
    model: "gpt-4"
    prompt: "Analyze this user: ${user.name}, age ${user.age}"
  }
}

fn main() -> Result(List(String), String) {
  let users = try get_users()

  let adults = users
    |> list.filter(fn(u) { u.age >= 18 })
    |> list.take(10)

  // Process in parallel
  adults
  |> list.map(analyze_user)
  |> result.sequence
}
```

### B.3 API Integration

```gleam
type ApiResponse {
  Success(data: String, status: Int)
  Error(message: String, code: Int)
  Timeout
}

fn fetch_with_retry(url: String, retries: Int) -> Result(String, String) {
  case http.get(url) {
    Ok(data) -> Ok(data)
    Error(msg) -> case retries {
      0 -> Error("Max retries: " + msg)
      n -> fetch_with_retry(url, n - 1)
    }
  }
}

fn parse_json(json: String) -> Result(User, String) {
  llm {
    model: "gpt-4"
    return_type: User
    prompt: "Parse this JSON into a User: ${json}"
  }
}

fn main(url: String) -> Result(User, String) {
  url
  |> fetch_with_retry(3)
  |> result.then(parse_json)
}
```

---

## Conclusion

This design represents a complete overhaul of the DSL, incorporating modern functional programming principles while maintaining focus on AI workflows. The result is a language that is:

- **Type-safe** - Strong typing prevents errors
- **Expressive** - Pattern matching and pipelines enable clear code
- **Focused** - LLM/SQL/HTTP are first-class citizens
- **Gleam-inspired** - Proven syntax and semantics
- **Production-ready** - Designed for real workflows

The implementation roadmap provides a clear path forward, and the integration with simplify_baml ensures that LLM features work seamlessly.
