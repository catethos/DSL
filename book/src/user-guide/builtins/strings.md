# String Operations

String manipulation functions for text processing and analysis.

## Upper

**Signature:** `(String) -> String`

Convert a string to uppercase.

```dsl
Upper("hello")
Upper("Hello World")
Upper("test123")
```

## Lower

**Signature:** `(String) -> String`

Convert a string to lowercase.

```dsl
Lower("HELLO")
Lower("Hello World")
Lower("TEST123")
```

## Length

**Signature:** `(String | List) -> Int`

Get the length of a string or list.

**With Strings:**
```dsl
Length("hello")
Length("")
Length("Hello World")
```

**With Lists:**
```dsl
Length([1, 2, 3])
Length([])
```

## Trim

**Signature:** `(String) -> String`

Remove whitespace from both ends of a string.

```dsl
Trim("  hello  ")
Trim("\n  world\t")
Trim("  spaces  everywhere  ")
```

**Note:** Only leading and trailing whitespace is removed. Internal whitespace is preserved.

## Split

**Signature:** `(String, String) -> List`

Split a string by a separator into a list of strings.

```dsl
Split("a,b,c", ",")
Split("hello world", " ")
Split("a::b::c", "::")
Split("one,two,three,four", ",")
```

**Common Use Cases:**
```dsl
Split("Alice,30,Engineer", ",")

Split("/usr/local/bin", "/")

Split("The quick brown fox", " ")
```

## Join

**Signature:** `(List, String) -> String`

Join list elements into a string with a separator.

```dsl
Join(["a", "b", "c"], ", ")
Join([1, 2, 3], "-")
Join(["hello", "world"], " ")
```

**Common Use Cases:**
```dsl
Join(["Alice", "30", "Engineer"], ",")

Join(["apple", "banana", "cherry"], ", ")

Join(["usr", "local", "bin"], "/")
```

## Replace

**Signature:** `(String, String, String) -> String`

Replace all occurrences of a pattern with a replacement string.

```dsl
Replace("hello world", "world", "DSL")
Replace("foo bar foo", "foo", "baz")
Replace("test-file-name.txt", "-", "_")
```

**Multiple Replacements:**
```dsl
Replace("aaa", "a", "b")

"hello world"
  |> Replace(_, "hello", "hi")
  |> Replace(_, "world", "there")
```

## Contains

**Signature:** `(String, String) -> Bool`

Check if a string contains a substring.

```dsl
Contains("hello world", "world")
Contains("hello", "xyz")
Contains("The quick brown fox", "quick")
Contains("test", "TEST")
```

**Use Cases:**
```dsl
def hasKeyword(text, keyword) { Contains(text, keyword) }

Contains(email, "@") ? "Valid" : "Invalid"
```

## StartsWith

**Signature:** `(String, String) -> Bool`

Check if a string starts with a prefix.

```dsl
StartsWith("hello", "hel")
StartsWith("hello", "lo")
StartsWith("https://example.com", "https://")  // true
StartsWith("test", "TEST")
```

**Use Cases:**
```dsl
StartsWith(url, "https://") ? "Secure" : "Insecure"

StartsWith(filename, "test_")
```

## EndsWith

**Signature:** `(String, String) -> Bool`

Check if a string ends with a suffix.

```dsl
EndsWith("hello", "lo")
EndsWith("hello", "hel")
EndsWith("document.pdf", ".pdf")
EndsWith("test", "TEST")
```

**Use Cases:**
```dsl
EndsWith(filename, ".txt") ? "Text file" : "Other"

EndsWith(email, "@example.com")
```

## Practical Examples

### Example 1: Email Validation

```dsl
def isValidEmail(email) { Contains(email, "@") && }
  Contains(email, ".") &&
  Length(email) > 5

isValidEmail("user@example.com")
isValidEmail("invalid")
```

### Example 2: Text Cleanup

```dsl
def cleanText(text) { text }
    |> Trim(_)
    |> Replace(_, "  ", " ")
    |> Replace(_, "\n\n", "\n")

cleanText("  hello   world  \n\n\ntest  ")
```

### Example 3: CSV Processing

```dsl
def main() {
    let csvLine = "Alice,30,Engineer,New York"
    let fields = Split(csvLine, ",")
    let person = {
      name: fields[0],
      age: ToInt(fields[1]),
      job: fields[2],
      city: fields[3]
    }
}
main()
```

### Example 4: Word Count

```dsl
def wordCount(text) { text }
    |> Trim(_)
    |> Split(_, " ")
    |> filter(_, fn word => Length(word end) > 0)
    |> Length(_)

wordCount("The quick brown fox")
```

### Example 5: String Transformation Pipeline

```dsl
def formatName(name) { name }
    |> Trim(_)
    |> Lower(_)
    |> Split(_, " ")
    |> map(_, fn word => Upper(word[0] end) + Lower(word[1:]))
    |> Join(_, " ")

formatName("  alice   smith  ")
```

### Example 6: Path Manipulation

```dsl
def getFilename(path) { path }
    |> Split(_, "/")
    |> Last(_)

getFilename("/usr/local/bin/dsl")

def getExtension(filename) { filename }
    |> Split(_, ".")
    |> Last(_)

getExtension("document.pdf")
```

## Best Practices

### 1. Use Pipeline Operator

Chain string operations for clarity:

```dsl
text
  |> Trim(_)
  |> Lower(_)
  |> Replace(_, " ", "_")

Replace(Lower(Trim(text)), " ", "_")
```

### 2. Check Before Processing

Validate input before operations:

```dsl
def processEmail(email) { Contains(email, "@") }
    ? Lower(Trim(email))
    : error("Invalid email")

def processEmail(email) { Lower(Trim(email)) }
```

### 3. Combine with Filter/Map

Use string functions with higher-order functions:

```dsl
filter(strings, fn s => Length(Trim(s end)) > 0)

map(strings, fn s => Trim(Lower(s end)))
```

## Next Steps

- [List Operations](./lists.md) - Work with collections
- [Functional Programming](./functional.md) - Use Map/Filter with strings
- [Utilities](./utilities.md) - Additional text tools
