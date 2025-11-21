# String Operations

String manipulation functions for text processing and analysis.

## Upper

**Signature:** `(String) -> String`

Convert a string to uppercase.

```dsl
Upper("hello")  // "HELLO"
Upper("Hello World")  // "HELLO WORLD"
Upper("test123")  // "TEST123"
```

## Lower

**Signature:** `(String) -> String`

Convert a string to lowercase.

```dsl
Lower("HELLO")  // "hello"
Lower("Hello World")  // "hello world"
Lower("TEST123")  // "test123"
```

## Length

**Signature:** `(String | List) -> Int`

Get the length of a string or list.

**With Strings:**
```dsl
Length("hello")  // 5
Length("")  // 0
Length("Hello World")  // 11
```

**With Lists:**
```dsl
Length([1, 2, 3])  // 3
Length([])  // 0
```

## Trim

**Signature:** `(String) -> String`

Remove whitespace from both ends of a string.

```dsl
Trim("  hello  ")  // "hello"
Trim("\n  world\t")  // "world"
Trim("  spaces  everywhere  ")  // "spaces  everywhere"
```

**Note:** Only leading and trailing whitespace is removed. Internal whitespace is preserved.

## Split

**Signature:** `(String, String) -> List`

Split a string by a separator into a list of strings.

```dsl
Split("a,b,c", ",")  // ["a", "b", "c"]
Split("hello world", " ")  // ["hello", "world"]
Split("a::b::c", "::")  // ["a", "b", "c"]
Split("one,two,three,four", ",")  // ["one", "two", "three", "four"]
```

**Common Use Cases:**
```dsl
// CSV parsing
Split("Alice,30,Engineer", ",")  // ["Alice", "30", "Engineer"]

// Path splitting
Split("/usr/local/bin", "/")  // ["", "usr", "local", "bin"]

// Word tokenization
Split("The quick brown fox", " ")  // ["The", "quick", "brown", "fox"]
```

## Join

**Signature:** `(List, String) -> String`

Join list elements into a string with a separator.

```dsl
Join(["a", "b", "c"], ", ")  // "a, b, c"
Join([1, 2, 3], "-")  // "1-2-3"
Join(["hello", "world"], " ")  // "hello world"
```

**Common Use Cases:**
```dsl
// Create CSV line
Join(["Alice", "30", "Engineer"], ",")  // "Alice,30,Engineer"

// Format list for display
Join(["apple", "banana", "cherry"], ", ")  // "apple, banana, cherry"

// Build path
Join(["usr", "local", "bin"], "/")  // "usr/local/bin"
```

## Replace

**Signature:** `(String, String, String) -> String`

Replace all occurrences of a pattern with a replacement string.

```dsl
Replace("hello world", "world", "DSL")  // "hello DSL"
Replace("foo bar foo", "foo", "baz")  // "baz bar baz"
Replace("test-file-name.txt", "-", "_")  // "test_file_name.txt"
```

**Multiple Replacements:**
```dsl
// Replace multiple occurrences
Replace("aaa", "a", "b")  // "bbb"

// Chain replacements
"hello world"
  |> Replace(_, "hello", "hi")
  |> Replace(_, "world", "there")
// "hi there"
```

## Contains

**Signature:** `(String, String) -> Bool`

Check if a string contains a substring.

```dsl
Contains("hello world", "world")  // true
Contains("hello", "xyz")  // false
Contains("The quick brown fox", "quick")  // true
Contains("test", "TEST")  // false (case sensitive)
```

**Use Cases:**
```dsl
// Filter based on content
def hasKeyword(text, keyword) := Contains(text, keyword)

// Conditional logic
Contains(email, "@") ? "Valid" : "Invalid"
```

## StartsWith

**Signature:** `(String, String) -> Bool`

Check if a string starts with a prefix.

```dsl
StartsWith("hello", "hel")  // true
StartsWith("hello", "lo")  // false
StartsWith("https://example.com", "https://")  // true
StartsWith("test", "TEST")  // false (case sensitive)
```

**Use Cases:**
```dsl
// URL validation
StartsWith(url, "https://") ? "Secure" : "Insecure"

// Filter files by extension
StartsWith(filename, "test_")  // Check for test files
```

## EndsWith

**Signature:** `(String, String) -> Bool`

Check if a string ends with a suffix.

```dsl
EndsWith("hello", "lo")  // true
EndsWith("hello", "hel")  // false
EndsWith("document.pdf", ".pdf")  // true
EndsWith("test", "TEST")  // false (case sensitive)
```

**Use Cases:**
```dsl
// File type detection
EndsWith(filename, ".txt") ? "Text file" : "Other"

// Domain checking
EndsWith(email, "@example.com")
```

## Practical Examples

### Example 1: Email Validation

```dsl
def isValidEmail(email) :=
  Contains(email, "@") &&
  Contains(email, ".") &&
  Length(email) > 5

isValidEmail("user@example.com")  // true
isValidEmail("invalid")  // false
```

### Example 2: Text Cleanup

```dsl
def cleanText(text) :=
  text
    |> Trim(_)
    |> Replace(_, "  ", " ")  // Remove double spaces
    |> Replace(_, "\n\n", "\n")  // Remove double newlines

cleanText("  hello   world  \n\n\ntest  ")
// "hello world\ntest"
```

### Example 3: CSV Processing

```dsl
let csvLine = "Alice,30,Engineer,New York"
let fields = Split(csvLine, ",")
let person = {
  name: fields[0],
  age: ToInt(fields[1]),
  job: fields[2],
  city: fields[3]
}
```

### Example 4: Word Count

```dsl
def wordCount(text) :=
  text
    |> Trim(_)
    |> Split(_, " ")
    |> Filter(_, def (word) := Length(word) > 0)
    |> Length(_)

wordCount("The quick brown fox")  // 4
```

### Example 5: String Transformation Pipeline

```dsl
def formatName(name) :=
  name
    |> Trim(_)
    |> Lower(_)
    |> Split(_, " ")
    |> Map(_, def (word) := Upper(word[0]) + Lower(word[1:]))
    |> Join(_, " ")

formatName("  alice   smith  ")  // "Alice Smith"
```

### Example 6: Path Manipulation

```dsl
def getFilename(path) :=
  path
    |> Split(_, "/")
    |> Last(_)

getFilename("/usr/local/bin/dsl")  // "dsl"

def getExtension(filename) :=
  filename
    |> Split(_, ".")
    |> Last(_)

getExtension("document.pdf")  // "pdf"
```

## Best Practices

### 1. Use Pipeline Operator

Chain string operations for clarity:

```dsl
// Good
text
  |> Trim(_)
  |> Lower(_)
  |> Replace(_, " ", "_")

// Less clear
Replace(Lower(Trim(text)), " ", "_")
```

### 2. Check Before Processing

Validate input before operations:

```dsl
// Good
def processEmail(email) :=
  Contains(email, "@")
    ? Lower(Trim(email))
    : error("Invalid email")

// Risky
def processEmail(email) := Lower(Trim(email))
```

### 3. Combine with Filter/Map

Use string functions with higher-order functions:

```dsl
// Filter non-empty strings
Filter(strings, def (s) := Length(Trim(s)) > 0)

// Normalize all strings
Map(strings, def (s) := Trim(Lower(s)))
```

## Next Steps

- [List Operations](./lists.md) - Work with collections
- [Functional Programming](./functional.md) - Use Map/Filter with strings
- [Utilities](./utilities.md) - Additional text tools
