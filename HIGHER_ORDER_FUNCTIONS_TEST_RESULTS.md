# Higher-Order Functions Test Results

## Phase 1: Map and Filter Implementation - COMPLETE ✅

### Implementation Summary

**What was implemented:**
- Grammar: `fn params => expr end` syntax for inline lambdas
- IR: `Lambda(LambdaIR)` node type
- Parser: Converts `inline_lambda` grammar rule to AST
- Compiler: AST → IR transformation for lambdas
- Interpreter: 
  - `map(list, callable)` - supports both lambdas and string refs
  - `filter(list, predicate)` - supports both lambdas and string refs
  - Lambda evaluation with proper scoping
  - String reference resolution to user-defined functions

### Test Coverage

#### ✅ Basic Map Tests
- [x] Map with simple arithmetic: `map([1,2,3], fn x => x * 2 end)` → `[2,4,6]`
- [x] Map with complex expressions: `map([1,2,3], fn x => x * x + 1 end)` → `[2,5,10]`
- [x] Map on empty list: `map([], fn x => x * 2 end)` → `[]`
- [x] Map with string concatenation: `map(["hello","world"], fn s => s + "!" end)` → `["hello!","world!"]`
- [x] Map with division: `map([10,20,30], fn x => x / 2 end)` → `[5,10,15]`

#### ✅ Basic Filter Tests
- [x] Filter with simple predicate: `filter([1,2,3,4,5], fn x => x > 3 end)` → `[4,5]`
- [x] Filter on empty list: `filter([], fn x => x > 0 end)` → `[]`
- [x] Filter that matches nothing: `filter([1,2,3], fn x => x > 10 end)` → `[]`
- [x] Filter that matches everything: `filter([1,2,3], fn x => x > 0 end)` → `[1,2,3]`
- [x] Filter with compound conditions: `filter([1..6], fn x => x > 2 && x < 6 end)` → `[3,4,5]`
- [x] Filter with OR conditions: `filter([1..10], fn x => x < 3 || x > 8 end)` → `[1,2,9,10]`

#### ✅ Chaining Tests
- [x] Filter then map: `[1,2,3,4,5] |> filter(_, fn x => x > 2 end) |> map(_, fn x => x * 2 end)` → `[6,8,10]`
- [x] Map then filter: `[1,2,3,4,5] |> map(_, fn x => x * 2 end) |> filter(_, fn x => x > 5 end)` → `[6,8,10]`
- [x] Multiple filters: Sequential filtering works correctly
- [x] Multiple maps: Sequential mapping works correctly
- [x] Long chains: 3+ operations in a pipeline

#### ✅ Map/Object Tests
- [x] Extract field from maps: `map([{name:"Alice"},{name:"Bob"}], fn x => x.name end)` → `["Alice","Bob"]`
- [x] Filter maps by field: `filter([...], fn x => x.age > 28 end)` works
- [x] Access multiple fields: `map([{x:1,y:2},...], fn m => m.x + m.y end)` → `[3,7]`
- [x] Filter and extract pipeline: Works correctly

#### ✅ Edge Cases
- [x] Single element lists
- [x] Boolean values in map/filter
- [x] Arithmetic operations in lambdas: `+`, `-`, `*`, `/`
- [x] Comparison operators: `>`, `<`, `==`, `>=`, `<=`
- [x] Logical operators: `&&`, `||`, `!`
- [x] Field access in lambdas
- [x] Nested lambda calls (map inside map)

### Limitations Discovered

**Not yet supported:**
- ❌ Modulo operator `%` in expressions
- ❌ `if-then-else` inside lambda expressions
- ❌ Map literals inside lambda body (parser limitation)
- ❌ Pattern-matched functions as string references in map/filter
- ❌ Multi-parameter lambdas (only single-param tested so far)

**Error handling verified:**
- ✅ Type errors: Filter predicate must return Bool
- ✅ Arity errors: Lambda parameter count must match usage
- ✅ Clear error messages with operation and index context

### Performance Notes

- Lambdas are evaluated eagerly (not lazy)
- Vec preallocated with `with_capacity` for map results
- Keys not precomputed yet (that's for sortby/groupby)
- Acceptable for 100-1000 item lists

### Example Usage

```dsl
// Basic usage
map([1, 2, 3], fn x => x * 2 end)

// Chaining
[1, 2, 3, 4, 5, 6]
  |> filter(_, fn x => x > 2 end)
  |> map(_, fn x => x * 2 end)

// Working with objects
[{name: "Alice", age: 30}, {name: "Bob", age: 25}]
  |> filter(_, fn x => x.age > 28 end)
  |> map(_, fn x => x.name end)

// String references
def double(x) { x * 2 }
map([1, 2, 3], "double")
```

### Test Files Created

1. `examples/test_hof_basic.txt` - Basic map/filter tests
2. `examples/test_hof_chaining.txt` - Pipeline chaining tests
3. `examples/test_hof_edge_cases.txt` - Edge cases and Boolean tests
4. `examples/test_hof_maps.txt` - Map/object data tests
5. `test_hof.sh` - Test runner script

### Run Tests

```bash
# Individual test
echo "map([1,2,3], fn x => x * 2 end)" | cargo run --bin dsl -- -c

# All tests
./test_hof.sh
```

## Next Steps (Phase 2)

- [ ] Implement `reduce(list, init, fn acc, x => ... end)`
- [ ] Implement `sortby(list, fn x => x.field end)`
- [ ] Implement `groupby(list, fn x => x.category end)`
- [ ] Add support for multi-param lambdas (needed for reduce)
- [ ] Optimize sortby/groupby with key precomputation
- [ ] Add more comprehensive error messages

## Known Issues

1. **File parser doesn't work with comments** - use REPL stdin instead
2. **No modulo operator** - can't test even/odd filtering
3. **No if-then-else in lambdas** - grammar conflict to resolve
4. **Map literals in lambda body cause parse errors** - needs grammar fix

## Conclusion

Phase 1 implementation is **fully functional** for the core use cases:
- ✅ Map and filter with inline lambdas
- ✅ String references to user functions
- ✅ Chaining via pipe operator
- ✅ Working with map/object data
- ✅ Proper error handling and scoping

The foundation is solid for Phase 2 (reduce/sortby/groupby).
