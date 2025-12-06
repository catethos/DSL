# Implementation Plan: `explode` Function

## Overview

Implement an `explode` function that takes a table (list of maps) and a column name containing nested maps, then expands the keys of those nested maps into new columns in the parent table.

### Example

**Input:**
```
| name  | metadata                          |
|-------|-----------------------------------|
| Alice | {"age": 30, "city": "NYC"}        |
| Bob   | {"age": 25, "city": "LA"}         |
```

**After `explode(table, "metadata")`:**
```
| name  | metadata_age | metadata_city |
|-------|--------------|---------------|
| Alice | 30           | NYC           |
| Bob   | 25           | LA            |
```

---

## Implementation Steps

### 1. Grammar (grammar.pest)

Add `explode_expr` to the grammar:

**Location:** `crates/lattice/src/syntax/grammar.pest`

```pest
// Add to primary_expr (around line 218)
primary_expr = {
    ...
    | explode_expr      // NEW
    | map_column_expr
    ...
}

// Add new rule (after map_row_expr, around line 291)
/// Explode: explode(table, "column") or explode(table, "column", "prefix")
/// Or for pipe: data |> explode("column") or data |> explode("column", "prefix")
explode_expr = {
    "explode" ~ "(" ~ expression ~ "," ~ expression ~ ("," ~ expression)? ~ ")"
    | "explode" ~ "(" ~ expression ~ ("," ~ expression)? ~ ")"
}
```

Also add `explode` to keywords (line 424-426).

---

### 2. AST (ast.rs)

Add the `Explode` variant to the `Expr` enum:

**Location:** `crates/lattice/src/syntax/ast.rs` (around line 351)

```rust
/// Explode nested map column into separate columns
Explode {
    table: Box<Expr>,
    column: Box<Expr>,
    prefix: Option<Box<Expr>>,  // Optional prefix for new column names
}
```

---

### 3. Parser (parser.rs)

Add parsing logic for `explode_expr`:

**Location:** `crates/lattice/src/syntax/parser.rs`

Add a new function `parse_explode_expr` following the pattern of `parse_map_column_expr`:

```rust
fn parse_explode_expr(pair: Pair<'_, Rule>) -> Result<Expr, ParseError> {
    let line = pair.line_col().0;
    let mut inner = pair.into_inner();

    // Parse arguments based on count
    let args: Vec<_> = inner.collect();

    match args.len() {
        // Pipe form: explode("column") or explode("column", "prefix")
        1 => {
            let column = parse_expression(args[0].clone())?;
            Ok(Expr::Explode {
                table: Box::new(Expr::PipeTarget),  // Placeholder for pipe
                column: Box::new(column),
                prefix: None,
            })
        }
        2 => {
            // Could be: explode(table, "column") OR explode("column", "prefix") in pipe
            // Need to disambiguate based on context (handled in pipe resolution)
            let first = parse_expression(args[0].clone())?;
            let second = parse_expression(args[1].clone())?;
            Ok(Expr::Explode {
                table: Box::new(first),
                column: Box::new(second),
                prefix: None,
            })
        }
        3 => {
            let table = parse_expression(args[0].clone())?;
            let column = parse_expression(args[1].clone())?;
            let prefix = parse_expression(args[2].clone())?;
            Ok(Expr::Explode {
                table: Box::new(table),
                column: Box::new(column),
                prefix: Some(Box::new(prefix)),
            })
        }
        _ => Err(ParseError::InvalidExplodeArgs { line })
    }
}
```

Add case in `parse_primary_expr` to handle `Rule::explode_expr`.

---

### 4. Bytecode (bytecode.rs)

Add the `Explode` opcode:

**Location:** `crates/lattice/src/vm/bytecode.rs` (around line 170)

```rust
/// Explode nested map column into separate columns
/// Stack: [table, column_name, prefix_or_null] -> [result_table]
Explode,
```

---

### 5. Compiler (compiler/mod.rs)

Add compilation logic for `Explode`:

**Location:** `crates/lattice/src/compiler/mod.rs`

```rust
fn compile_explode(&mut self, table: &Expr, column: &Expr, prefix: &Option<Box<Expr>>, line: usize) -> Result<()> {
    // Compile table expression
    self.compile_expr(table)?;

    // Compile column name
    self.compile_expr(column)?;

    // Compile prefix (or push null if not provided)
    if let Some(prefix_expr) = prefix {
        self.compile_expr(prefix_expr)?;
    } else {
        self.emit(OpCode::LoadNull, line);
    }

    self.emit(OpCode::Explode, line);
    Ok(())
}
```

Add case in `compile_expr` match:

```rust
Expr::Explode { table, column, prefix } => {
    self.compile_explode(table, column, prefix, line)?;
}
```

---

### 6. VM (machine.rs)

Implement the `Explode` operation:

**Location:** `crates/lattice/src/vm/machine.rs` (add new function around line 2450)

```rust
fn op_explode(&mut self) -> Result<(), RuntimeError> {
    // Pop arguments from stack
    let prefix_val = self.pop()?;
    let column_val = self.pop()?;
    let table_val = self.pop()?;

    // Extract column name
    let column_name = match &column_val {
        Value::String(s) => s.to_string(),
        _ => return Err(RuntimeError::TypeError {
            expected: "String".to_string(),
            found: column_val.type_name().to_string(),
            context: "explode column name".to_string(),
        }),
    };

    // Extract prefix (default to column_name + "_")
    let prefix = match &prefix_val {
        Value::Null => format!("{}_", column_name),
        Value::String(s) => s.to_string(),
        _ => return Err(RuntimeError::TypeError {
            expected: "String or null".to_string(),
            found: prefix_val.type_name().to_string(),
            context: "explode prefix".to_string(),
        }),
    };

    // Get the table (list of maps)
    let rows = match &table_val {
        Value::List(list) => list.clone(),
        _ => return Err(RuntimeError::TypeError {
            expected: "List".to_string(),
            found: table_val.type_name().to_string(),
            context: "explode table".to_string(),
        }),
    };

    // First pass: collect all keys from the nested maps
    let mut all_nested_keys: std::collections::HashSet<String> = std::collections::HashSet::new();
    for row in rows.iter() {
        if let Value::Map(row_map) = row {
            if let Some(nested_val) = row_map.get(&column_name) {
                if let Value::Map(nested_map) = nested_val {
                    for key in nested_map.keys() {
                        all_nested_keys.insert(key.clone());
                    }
                }
            }
        }
    }

    // Sort keys for consistent column ordering
    let mut sorted_keys: Vec<_> = all_nested_keys.into_iter().collect();
    sorted_keys.sort();

    // Second pass: create new rows with exploded columns
    let mut result_rows = Vec::with_capacity(rows.len());

    for row in rows.iter() {
        let row_map = match row {
            Value::Map(m) => m,
            _ => return Err(RuntimeError::TypeError {
                expected: "Map".to_string(),
                found: row.type_name().to_string(),
                context: "explode row".to_string(),
            }),
        };

        // Start with existing columns (except the one being exploded)
        let mut new_row: HashMap<String, Value> = HashMap::new();
        for (key, value) in row_map.iter() {
            if key != &column_name {
                new_row.insert(key.clone(), value.clone());
            }
        }

        // Get the nested map (if it exists)
        let nested_map = row_map
            .get(&column_name)
            .and_then(|v| v.as_map());

        // Add exploded columns
        for nested_key in &sorted_keys {
            let new_col_name = format!("{}{}", prefix, nested_key);
            let value = nested_map
                .and_then(|m| m.get(nested_key))
                .cloned()
                .unwrap_or(Value::Null);
            new_row.insert(new_col_name, value);
        }

        result_rows.push(Value::map(new_row));
    }

    self.push(Value::list(result_rows));
    Ok(())
}
```

Add case in the main VM dispatch loop:

```rust
OpCode::Explode => self.op_explode()?,
```

---

### 7. Type Checker (optional enhancement)

**Location:** `crates/lattice/src/types/checker.rs`

For now, explode can return a dynamically typed table since the nested map structure may not be known at compile time. The type would be `[Map<String, Any>]`.

---

## Design Decisions

### 1. Column Naming Strategy
- Default: `{original_column}_{nested_key}` (e.g., `metadata_age`)
- Optional custom prefix via third argument

### 2. Handling Missing Keys
- If a row's nested map doesn't have a key that appears in other rows, use `Value::Null`

### 3. Non-Map Values in Column
- If the target column contains non-map values, treat as error or skip row
- **Recommendation:** Return error with clear message

### 4. Empty Tables
- Return empty table (preserve semantics)

### 5. Pipe Support
- Support both forms:
  - `explode(table, "column")`
  - `table |> explode("column")`

---

## Files to Modify

| File | Changes |
|------|---------|
| `crates/lattice/src/syntax/grammar.pest` | Add `explode_expr` rule, add to `primary_expr`, add keyword |
| `crates/lattice/src/syntax/ast.rs` | Add `Explode` variant to `Expr` enum |
| `crates/lattice/src/syntax/parser.rs` | Add `parse_explode_expr` function |
| `crates/lattice/src/vm/bytecode.rs` | Add `Explode` opcode |
| `crates/lattice/src/compiler/mod.rs` | Add `compile_explode` function |
| `crates/lattice/src/vm/machine.rs` | Add `op_explode` implementation |

---

## Test Cases

Create `example/explode_test.lat`:

```lattice
// Basic explode test
let data = [
    {"name": "Alice", "info": {"age": 30, "city": "NYC"}},
    {"name": "Bob", "info": {"age": 25, "city": "LA"}}
]

let exploded = explode(data, "info")
print(exploded)
// Expected: name, info_age, info_city columns

// With custom prefix
let custom = explode(data, "info", "user_")
print(custom)
// Expected: name, user_age, user_city columns

// Pipe syntax
let piped = data |> explode("info")
print(piped)

// SQL result explode
let sql_data = SQL("SELECT name, metadata FROM users")
let result = sql_data |> explode("metadata")
```

---

## Future Enhancements

1. **Nested explode:** Support exploding multiple levels deep
2. **Keep original column:** Option to preserve the original nested map column
3. **Selective keys:** Only explode specific keys from the nested map
4. **Type inference:** Infer result type when struct types are known
