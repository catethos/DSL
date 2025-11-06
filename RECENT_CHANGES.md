# Recent Changes

## Build Cache Optimization (2025-11-07)

### Summary
Implemented persistent build cache for compiled binaries, achieving 16x faster builds.

### Performance
- **Cold cache**: ~36 seconds (first build)
- **Warm cache**: ~2 seconds (subsequent builds)
- **Speedup**: 16-18x faster

### Changes
- Modified `dsl-compiler` to use persistent cache directory at `/tmp/dsl_compiler_cache`
- Added `clean` command to manually clear cache
- Cache automatically reuses compiled dependencies across builds

### Usage
```bash
# Normal build (uses cache automatically)
dsl-compiler build program.dsl -o program

# Clean cache to free space (~500MB)
dsl-compiler clean
```

### Documentation
- [Compiling to Binary](docs/13-Compiling-to-Binary.md)

---

## Command-Line Arguments (2025-11-07)

### Summary
Added support for command-line arguments in compiled binaries through pre-defined variables.

### Available Variables
- `args` - List of all arguments
- `arg1`, `arg2`, `arg3`, etc. - Individual positional arguments

### Usage Example
```dsl
let name = length(args) > 0 ? arg1 : "World"
upper("Hello, ${name}!")
```

```bash
./greet Alice
# Output: "HELLO, ALICE!"
```

### Documentation
- [Compiling to Binary - Command-Line Arguments](docs/13-Compiling-to-Binary.md#command-line-arguments)

---

## Let Syntax Addition (2025-11-07)

### Summary
Added `let x = expr` syntax as an alternative to the existing `expr as x` syntax for variable binding.

### Changes
- **Parser**: Added `let_binding` grammar rule and parsing logic
- **Autocomplete**: Added `let` keyword to suggestions
- **Help**: Updated REPL help text and banner to show both syntaxes
- **Syntax Highlighting**: Added `let` to keyword highlighting
- **Documentation**: Updated `docs/04-Language-Features.md` with let syntax examples

### Usage
Both syntaxes are now supported and completely equivalent:

```javascript
// Traditional style
let x = 5
let result = calculate(10)
let [a, b, c] = [1, 2, 3]

// Pipeline style (original)
5 as x
calculate(10) as result
[1, 2, 3] as [a, b, c]
```

### Technical Details
- No IR changes required - both syntaxes compile to identical IR
- Zero performance overhead
- All 109 tests pass
- Fully backward compatible

### Files Modified
1. `crates/dsl-core/src/parser/grammar.pest`
2. `crates/dsl-core/src/parser/mod.rs`
3. `crates/dsl-autocomplete/src/providers/keyword.rs`
4. `crates/dsl-repl/src/banner.rs`
5. `tree-sitter-dsl/queries/highlights.scm`
6. `docs/04-Language-Features.md`
7. `docs/00-Documentation-Summary.md`

### See Also
- [Language Features Documentation](docs/04-Language-Features.md#variables-and-binding)
