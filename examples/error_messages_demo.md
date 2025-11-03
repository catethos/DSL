# Improved Error Messages for Type Instantiation

## Problem
When there's a syntax error in type instantiation (e.g., missing comma), the error message was misleading:

```
flow> Person { name: "Alice", age: 30 email: "ada@gmail.com"}
Variable 'Person' not found
```

This happens because:
1. The parser fails to parse the type instantiation due to syntax error
2. It backtracks and successfully parses just `Person` as a variable
3. The evaluator tries to look up the variable and fails

## Solution
Added context-aware error detection in the evaluator that:
1. Detects when a variable lookup fails but the input looks like type instantiation
2. Checks if the name is a registered type
3. Provides a helpful error message

## Improved Error Messages

### Case 1: Syntax Error in Known Type
```dsl
type Person { name: string, age: int, email?: string }

# Missing comma before 'email'
Person { name: "Alice", age: 30 email: "ada@gmail.com"}
```

**New Error:**
```
Syntax error in type instantiation for 'Person'. 
Check for missing commas between fields or other syntax errors.
Hint: Type instantiation syntax is: Person { field: value, field2: value2 }
```

### Case 2: Undefined Type
```dsl
# Trying to instantiate a type that doesn't exist
UndefinedType { field: "value" }
```

**New Error:**
```
Type 'UndefinedType' not found. Did you mean to define it first?
Example: type UndefinedType { field1: string, field2: int }
```

## Implementation

The solution works by:

1. **Pattern Detection**: When a variable lookup fails, check if:
   - The expression is a simple variable reference
   - The input starts with that variable name followed by `{`

2. **Type Registry Check**: If pattern matches:
   - If type exists → syntax error in instantiation
   - If type doesn't exist → undefined type error

3. **Helpful Hints**: Provide:
   - Clear indication of what went wrong
   - Syntax examples
   - Actionable suggestions

## Files Modified

- `evaluator.rs:108-120`: Added error enhancement in `eval()` method
- `evaluator.rs:523-562`: Added `enhance_parse_error()` and `enhance_eval_error()` helper methods
- Added tests to verify error messages are helpful
