# Test Rich Error Display in DSL GUI
# Run with: cargo run --bin dsl-gui
# Then load this file or copy individual examples into the REPL

## 1. Unknown Variable Error
# Type this in REPL:
undefined_variable

## 2. Unknown Function Error
# Type this in REPL:
nonexistent_function(42)

## 3. Type Mismatch Errors (when type checking is enabled)
# These would show "Expected: X, Got: Y"
let x = 5
upper(x)  # upper expects string, gets int

## 4. Invalid Arguments Error
# Call a function with wrong number of arguments
let greet = |name| "Hello " + name
greet()  # Missing required argument
greet("Alice", "Bob")  # Too many arguments

## 5. Runtime Errors
# Division by zero or other runtime issues
let zero = 0
let result = 100 / zero

# Accessing non-existent map key
let person = {"name": "Alice"}
person.age  # Key doesn't exist

## 6. Parse Errors
# Invalid syntax
let x =
let y = 5 +

## 7. Pattern Matching Errors
# Non-exhaustive pattern match
let value = Some(42)
match value {
    None => "nothing"
    # Missing Some case
}

## 8. LLM Errors (requires API setup)
# Uncomment these if you have LLM API configured:
# function test_llm() -> String {
#     prompt: "This will fail if API key is missing"
#     model: "gpt-4"
# }
# test_llm()

## 9. HTTP Errors
# Uncomment to test HTTP errors:
# function test_http() -> String {
#     http: GET "https://invalid-url-that-doesnt-exist-12345.com"
# }
# test_http()

## 10. Scope/Binding Errors
let x = 10
{
    let y = 20
}
y  # y is out of scope

## 11. Command Errors (in REPL)
# Type invalid commands:
:invalidcommand
:this_doesnt_exist

## 12. Complex Nested Errors
# Errors in nested expressions
let calculate = |x| x * 2
let data = [1, 2, 3]
calculate(data)  # Can't multiply list by number

## 13. String Operation Errors
let num = 42
len(num)  # len expects string or list, not int

## 14. List/Map Type Errors
let items = [1, 2, 3]
items.name  # Can't access property on list

let person = {"name": "Bob"}
person[0]  # Can't use numeric index on map
