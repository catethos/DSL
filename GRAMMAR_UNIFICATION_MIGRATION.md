# Grammar Unification: Adding `end` to `def`

## Motivation

Unify syntax between named functions and anonymous lambdas:

**Before:**
```dsl
def factorial(n) => if n == 0 then 1 else n * factorial(n - 1)
```

**After:**
```dsl
def factorial(n) => if n == 0 then 1 else n * factorial(n - 1) end
```

This creates perfect symmetry:
- `def name(params) => expr end` - Named function
- `fn params => expr end` - Anonymous function

## Benefits

1. **Consistency:** Both use identical `=> expr end` structure
2. **Clearer boundaries:** `end` marks where function body terminates
3. **Shared parsing:** Both can use same expression parsing logic
4. **Natural for multi-line:** When expressions get complex, `end` is clearer
5. **Pattern matching compatibility:**
   ```dsl
   def factorial(0) => 1 end
   def factorial(n) => n * factorial(n - 1) end
   ```

## Implementation Strategy

### Phase 0: Make `end` Optional (Backward Compatible)

Update grammar to accept both forms temporarily:

```javascript
function_definition: $ => seq(
  'def',
  $.identifier,
  $.parameter_list,
  '=>',
  $._expression,
  optional('end')  // Make it optional during migration
),
```

### Phase 1: Update All Examples and Tests

Update all existing code to use `end`:
- Update README examples
- Update test files
- Update documentation

### Phase 2: Make `end` Required

Remove `optional()` from grammar:

```javascript
function_definition: $ => seq(
  'def',
  $.identifier,
  $.parameter_list,
  '=>',
  $._expression,
  'end'  // Now required
),
```

## Migration Impact

### Files to Update

```bash
# Find all def statements
rg "def \w+\([^)]*\) =>" --glob "*.dsl" --glob "examples/*"

# Find all test files
rg "def " --glob "**/*test*.dsl"

# Find all examples
rg "def " --glob "examples/**/*.dsl"
```

### Automated Migration

```bash
# Regex replacement (approximate - test first!)
# Pattern: def name(params) => expr
# Replace: def name(params) => expr end

# For simple single-line defs without existing 'end'
sed -i.bak 's/^\(def [^=]*=>[^e]*\)$/\1 end/' examples/*.dsl
```

### Parser Changes

```rust
// In parser.rs
fn parse_function_definition(&mut self) -> Result<FunctionDef> {
    self.expect(Token::Def)?;
    let name = self.expect_identifier()?;
    
    // Parse parameter list (or pattern)
    let params = self.parse_params()?;
    
    self.expect(Token::Arrow)?;  // =>
    
    // Parse body expression
    let body = self.parse_expr()?;
    
    // NEW: Expect 'end' keyword
    self.expect(Token::End)?;
    
    Ok(FunctionDef { name, params, body })
}
```

## Timeline

**If doing before HOFs:**
- Phase 0: Make `end` optional (30 min)
- Phase 1: Update all examples/tests (1-2 hours)
- Phase 2: Make `end` required (15 min)
- **Total: 2-3 hours**

**If doing alongside HOFs:**
- Include in Phase 1 of HOF implementation
- Update grammar for both `def` and `fn` simultaneously
- **No additional time needed**

## Recommendation

✅ **Do this as part of HOF Phase 1** since you're already updating the grammar for `fn`.

Update both at once:
```javascript
// Named function
function_definition: $ => seq(
  'def',
  $.identifier,
  $.parameter_list,
  '=>',
  $._expression,
  'end'
),

// Anonymous function (new)
inline_lambda: $ => seq(
  'fn',
  commaSep1($.identifier),
  '=>',
  $._expression,
  'end'
),
```

## Pattern Matching Consideration

Your DSL supports pattern matching in function definitions:

```dsl
def factorial(0) => 1 end
def factorial(n) => n * factorial(n - 1) end
```

Ensure the pattern matching lowering handles `end` correctly when there are multiple overloads.

## Breaking Change Communication

If you have users, communicate:

1. **Why:** Consistency with new lambda syntax
2. **Migration:** Simple - add `end` to all `def` statements
3. **Timeline:** Optional in v1.x, required in v2.0
4. **Tool:** Provide migration script if needed

## Example Before/After

**Before:**
```dsl
def add(x, y) => x + y
def factorial(0) => 1
def factorial(n) => n * factorial(n - 1)
def process(data) => 
    data |> filter(data, "isValid") |> map(data, "transform")
```

**After:**
```dsl
def add(x, y) => x + y end
def factorial(0) => 1 end
def factorial(n) => n * factorial(n - 1) end
def process(data) => 
    data |> filter(data, "isValid") |> map(data, "transform") end
```

Clean and consistent! ✨
