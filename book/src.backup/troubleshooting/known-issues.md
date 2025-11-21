# Known Issues

This page documents known limitations and issues in the current version of the DSL.

## Language Features

### Comments Not Fully Implemented

**Issue:** Comments are partially parsed but not fully supported.

**Impact:**
- Single-line comments (`//` and `#`) may be parsed but could cause unexpected behavior
- Multi-line comments (`/* */`) will cause parser errors
- Comments in code may lead to runtime issues

**Workaround:**
- Avoid using comments in DSL code until this feature is fully implemented
- Use descriptive variable names instead of inline comments
- Document code in external files or markdown

**Status:** Planned for future release

### If/Else Blocks Not Supported

**Issue:** Traditional `if/else` block syntax does not exist in the DSL.

**Impact:**
- Code using `if condition { } else { }` syntax will fail to parse

**Workaround:**
- Use ternary operator: `condition ? then_expr : else_expr`
- Use match expressions for complex conditionals
- See [Control Flow](../user-guide/language/control-flow.md) for correct syntax

**Status:** Ternary and match expressions are fully supported

## Syntax Requirements

### Pipeline Operator Requires Underscore

**Issue:** Pipeline operator `|>` requires explicit underscore placeholder.

**Incorrect:**
```dsl
value |> Function
```

**Correct:**
```dsl
value |> Function(_)
```

**Status:** Working as designed

### Lambda Syntax

**Issue:** Only `fn x => expr end` syntax is supported for lambdas.

**Incorrect:**
```dsl
fun(x) { x * 2 }
```

**Correct:**
```dsl
fn x => x * 2 end
```

**Status:** Working as designed
