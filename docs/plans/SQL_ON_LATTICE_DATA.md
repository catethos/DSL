# Plan: SQL Queries on Lattice Data Structures

## Executive Summary

Enable Lattice users to query in-memory data structures (Lists, Maps) using SQL syntax, with automatic support across all language bindings (Python, Node.js, Elixir) without any binding-specific code changes.

---

## 1. Context & Motivation

### 1.1 Current State

Lattice currently supports SQL queries via the `SQL()` expression:

```lattice
SQL("SELECT * FROM users WHERE age > 21")
```

However, this **only works with**:
- External data sources (CSV, Parquet files via DuckDB's native readers)
- Manually created tables (`CREATE TABLE` + `INSERT`)
- Pre-existing database tables

**It does NOT support** querying Lattice's own data structures:

```lattice
let users = [
    {id: 1, name: "Alice", age: 30},
    {id: 2, name: "Bob", age: 18}
]

// THIS DOES NOT WORK TODAY:
SQL("SELECT * FROM users WHERE age > 21")
```

### 1.2 Why This Matters

1. **Data Pipeline Use Case**: Users often load/transform data in Lattice, then want to filter/aggregate it with SQL's familiar syntax.

2. **Host Language Integration**: When embedding Lattice in Python/Node/Elixir, users pass data from their application. They expect to query it directly.

3. **Competitive Parity**: DuckDB's Python API allows `SELECT * FROM my_dataframe` - Lattice should match this ergonomics.

4. **Unified Query Model**: Users shouldn't need to choose between Lattice's list comprehensions and SQL - both should work on the same data.

### 1.3 Why Core-Level Implementation

Lattice has three language bindings that share identical architecture:

```
Host Language → LatticeValue (FFI type) → Core Runtime → SqlProvider
```

All bindings:
- Convert host values to `LatticeValue` before calling core
- Use the same `RuntimeBuilder` for configuration
- Share the same `SqlProvider` trait implementation

**Implication**: Implementing this feature in the Lattice core automatically provides it to all three bindings with zero additional work.

**Note**: Bindings must be rebuilt with `sql-arrow` feature enabled, but no binding source code changes are required.

---

## 2. Goals & Non-Goals

### 2.1 Goals

1. **Query Lattice variables with SQL**: `SQL("SELECT * FROM my_list")` where `my_list` is a Lattice `List<Map>`.

2. **Zero binding changes**: Python, Node.js, and Elixir bindings work automatically.

3. **Efficient execution**: Use Arrow-based zero-copy when possible, avoid unnecessary data duplication.

4. **Clean syntax**: No special registration ceremony - just use variable names as table names.

5. **Type preservation**: Query results maintain proper Lattice types.

6. **Clear error messages**: Validate inputs early with actionable error messages before hitting DuckDB.

### 2.2 Non-Goals

1. **Querying arbitrary types**: Only `List<Map>` (tabular data) will be queryable. Not nested structures or primitives.

2. **Bi-directional sync**: Changes via SQL (`UPDATE`, `DELETE`) won't modify the original Lattice variable.

3. **Persistent registration**: Tables are registered per-query, not persisted in DuckDB catalog.

4. **Host-language callback providers**: We won't implement FFI callbacks for custom SqlProvider implementations.

5. **Complex SQL constructs**: Subqueries, CTEs, and derived tables are explicitly unsupported in v1.

---

## 3. Technical Design

### 3.1 Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         SQL Query Flow                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  1. User writes: SQL("SELECT * FROM data WHERE x > 10")         │
│                              │                                   │
│                              ▼                                   │
│  2. Compiler emits: OpCode::SqlQuery                            │
│                              │                                   │
│                              ▼                                   │
│  3. VM executes op_sql_query():                                 │
│     ┌────────────────────────────────────────────────────┐      │
│     │ a. Parse SQL with sqlparser-rs, extract table refs │      │
│     │ b. Validate: reject unsupported constructs (CTEs)  │      │
│     │ c. For each table, check VM globals                │      │
│     │ d. Validate: ensure List<Map> with string keys     │      │
│     │ e. Convert to Arrow RecordBatch                    │      │
│     │ f. Register as DuckDB virtual table (quoted ident) │      │
│     │ g. Execute SQL query                               │      │
│     │ h. Convert results back to LatticeValue            │      │
│     │ i. Cleanup: unregister temporary tables (always)   │      │
│     └────────────────────────────────────────────────────┘      │
│                              │                                   │
│                              ▼                                   │
│  4. Result pushed to stack as List<Map>                         │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 Key Components

#### Component 1: SQL Table Reference Extractor (AST-based)

Extract table names from SQL queries using `sqlparser-rs` for robust parsing.

```rust
// src/sql/refs.rs (new file)

use sqlparser::dialect::DuckDbDialect;
use sqlparser::parser::Parser;

/// Extract table references from a SQL query using AST parsing
pub fn extract_table_references(sql: &str) -> Result<Vec<String>, SqlError> {
    let dialect = DuckDbDialect {};
    let ast = Parser::parse_sql(&dialect, sql)?;
    
    let mut tables = Vec::new();
    for statement in ast {
        extract_tables_from_statement(&statement, &mut tables)?;
    }
    Ok(tables)
}

/// Validates that the query doesn't use unsupported constructs
fn extract_tables_from_statement(stmt: &Statement, tables: &mut Vec<String>) -> Result<(), SqlError> {
    // Extract from TableFactor::Table { name, .. }
    // Extract from TableWithJoins base tables
    // 
    // REJECT with clear error:
    // - CTEs (WITH clauses): "SQL on Lattice data does not support CTEs"
    // - Derived tables/subqueries: "SQL on Lattice data does not support subqueries"
    // - Table functions: skip (not Lattice variables)
}
```

**Why AST over regex**: Regex-based extraction is brittle and will misidentify names in strings, comments, subqueries, and table functions. AST parsing is a one-time complexity cost that prevents subtle bugs.

#### Component 2: LatticeValue to Arrow Conversion

Convert Lattice data to Arrow's columnar format for efficient querying.

```rust
// src/sql/arrow.rs (new file)

/// Convert List<Map> to Arrow RecordBatch
/// Validates that all elements are Maps with string keys
pub fn lattice_list_to_recordbatch(list: &[Value]) -> Result<RecordBatch, SqlError> {
    // 1. Validate all elements are Map
    for (i, elem) in list.iter().enumerate() {
        if !matches!(elem, Value::Map(_)) {
            return Err(SqlError::InvalidRowType {
                index: i,
                expected: "Map",
                found: elem.type_name(),
            });
        }
    }
    
    // 2. Infer schema from first N rows
    let schema = infer_schema_from_list(list)?;
    
    // 3. Build columnar arrays
    // ...
}

/// Infer Arrow schema from first N rows (default: 100)
fn infer_schema_from_list(list: &[Value]) -> Result<Schema, SqlError>;

/// Build Arrow array for a single column
fn build_arrow_array(values: &[Value], field: &Field) -> Result<ArrayRef, SqlError>;
```

**Type mapping**:
| Lattice Type | Arrow Type |
|--------------|------------|
| Int | Int64 |
| Float | Float64 |
| String | Utf8 |
| Bool | Boolean |
| Null | Null |
| List | List (experimental) |
| Map | Struct (experimental) |

**Type promotion rules**:
| Types Present | Result |
|---------------|--------|
| Null + X | X (with nulls) |
| Int + Float | Float64 |
| Int + Null | Int64 |
| String + Int | **Error** |
| Bool + Int | **Error** |

**Column presence**: Union of keys over first N rows. Missing entries become null. Keys appearing after inference window cause an error: "Column 'x' appears after schema inference window".

#### Component 3: Identifier Handling

Safe identifier quoting and collision detection.

```rust
// src/sql/ident.rs (new file)

/// Quote an identifier for safe use in SQL
/// Prevents SQL injection and handles special characters
pub fn quote_ident(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

/// Check if a Lattice variable name is a valid unquoted SQL identifier
pub fn is_valid_unquoted_ident(name: &str) -> bool {
    // [a-zA-Z_][a-zA-Z0-9_]* and not a reserved word
}
```

**Identifier semantics**:
- Only variables with valid unquoted SQL identifier names are auto-registered
- Variables with special characters (e.g., `user-data`) require quoted identifiers in SQL: `SELECT * FROM "user-data"`
- Lattice variables are case-sensitive; SQL lookup uses case-insensitive matching against globals

#### Component 4: Extended SqlArrowProvider Trait

Separate trait for Arrow-specific operations to avoid Arrow type dependencies in base trait.

```rust
// src/runtime/providers/sql.rs (modify existing)

pub trait SqlProvider: Send + Sync {
    // Existing methods
    fn query(&self, sql: &str) -> Result<SqlResult, SqlError>;
    fn execute(&self, sql: &str) -> Result<usize, SqlError>;
    fn table_exists(&self, name: &str) -> Result<bool, SqlError>;
}

// NEW: Separate trait for Arrow integration (behind sql-arrow feature)
#[cfg(feature = "sql-arrow")]
pub trait SqlArrowProvider: SqlProvider {
    /// Register in-memory Arrow data as a queryable table
    fn register_arrow_table(&self, name: &str, data: Arc<RecordBatch>) -> Result<(), SqlError>;
    
    /// Remove a registered table
    fn unregister_table(&self, name: &str) -> Result<(), SqlError>;
}
```

**Why separate trait**: Avoids Arrow type dependencies when `sql-arrow` feature is disabled. `NoSqlProvider` compiles cleanly regardless of Arrow/DuckDB features.

#### Component 5: DuckDB Provider Implementation

Implement the new trait methods using DuckDB's Arrow virtual table.

```rust
// src/runtime/providers/sql.rs (extend DuckDbProvider)

#[cfg(feature = "sql-arrow")]
impl SqlArrowProvider for DuckDbProvider {
    fn register_arrow_table(&self, name: &str, batch: Arc<RecordBatch>) -> Result<(), SqlError> {
        // Store batch in internal map to ensure lifetime
        self.registered_tables.insert(name.to_string(), batch.clone());
        
        self.with_connection(|conn| {
            let quoted = quote_ident(name);
            // Register using DuckDB's Arrow interface
            // ...
            Ok(())
        })
    }

    fn unregister_table(&self, name: &str) -> Result<(), SqlError> {
        self.registered_tables.remove(name);
        self.with_connection(|conn| {
            let quoted = quote_ident(name);
            conn.execute(&format!("DROP VIEW IF EXISTS {}", quoted), [])?;
            Ok(())
        })
    }
}
```

**Memory management**: `DuckDbProvider` holds `Arc<RecordBatch>` in an internal map while table is registered. This ensures the Arrow data lives for the duration of the query.

#### Component 6: VM Integration

Modify the SQL opcode handler with proper validation and cleanup.

```rust
// src/vm/machine.rs (modify op_sql_query)

fn op_sql_query(&mut self) -> Result<()> {
    let query_str = self.pop_string()?;

    // 1. Parse SQL and extract table references (validates syntax)
    let table_refs = extract_table_references(&query_str)?;
    
    let mut registered_tables = Vec::new();

    // 2. Pre-validate all tables BEFORE any registration
    for table_name in &table_refs {
        // Skip if it's an existing DuckDB table
        if self.sql_provider.table_exists(table_name)? {
            // Emit debug warning if Lattice variable with same name exists
            if self.has_global(table_name) {
                log::debug!("Lattice variable '{}' shadowed by existing DuckDB table", table_name);
            }
            continue;
        }

        // Check if it's a Lattice variable
        match self.get_global(table_name) {
            None => {
                return Err(LatticeError::SqlTableNotFound {
                    name: table_name.clone(),
                    hint: "Not found as Lattice variable or database table",
                });
            }
            Some(value) => {
                if !matches!(value, Value::List(_)) {
                    return Err(LatticeError::SqlWrongType {
                        name: table_name.clone(),
                        expected: "List<Map>",
                        found: value.type_name(),
                    });
                }
            }
        }
    }

    // 3. Convert and register tables
    for table_name in &table_refs {
        if self.sql_provider.table_exists(table_name)? {
            continue;
        }
        
        if let Some(Value::List(list)) = self.get_global(table_name) {
            let batch = lattice_list_to_recordbatch(&list)?;
            self.sql_provider.register_arrow_table(table_name, Arc::new(batch))?;
            registered_tables.push(table_name.clone());
        }
    }

    // 4. Execute query (capture result before cleanup)
    let result = self.sql_provider.query(&query_str);

    // 5. ALWAYS cleanup temporary tables, even on error
    for name in &registered_tables {
        let _ = self.sql_provider.unregister_table(name);
    }

    // 6. Now propagate any query error
    let sql_result = result?;
    
    // 7. Convert and push result
    let lattice_result = sql_result_to_lattice_value(sql_result)?;
    self.push(lattice_result);
    Ok(())
}
```

**Key improvements**:
- Pre-validates all tables before any registration (fail fast)
- Cleanup runs even when query fails
- Clear error messages with hints
- Debug logging for shadowed variables

---

## 4. Implementation Plan

### Phase 1: Core Infrastructure

#### Step 1.1: Add Dependencies

**File**: `crates/lattice/Cargo.toml`

```toml
[dependencies]
sqlparser = "0.43"  # For SQL parsing

[features]
sql-arrow = ["arrow", "duckdb/vtab-arrow"]
```

#### Step 1.2: Create SQL Reference Extractor

**File**: `crates/lattice/src/sql/refs.rs` (new)

Implement AST-based extraction using `sqlparser-rs`:
- Parse with DuckDB dialect
- Walk AST to find `TableFactor::Table`
- Reject unsupported constructs with clear errors

**Test cases**:
- Simple SELECT
- Multiple tables (JOIN)
- Table aliases (`FROM users u`)
- Case insensitivity
- **Reject**: CTEs, subqueries, derived tables
- **Ignore**: Table functions, schema-qualified names

#### Step 1.3: Create Identifier Utilities

**File**: `crates/lattice/src/sql/ident.rs` (new)

Implement:
- `quote_ident()` - Safe identifier quoting
- `is_valid_unquoted_ident()` - Validation

**Test cases**:
- Normal identifiers
- Reserved words
- Special characters
- SQL injection attempts

#### Step 1.4: Create Arrow Conversion Module

**File**: `crates/lattice/src/sql/arrow.rs` (new)

Implement:
- `lattice_list_to_recordbatch()` - Main conversion with validation
- `infer_schema_from_list()` - Schema inference from first N rows
- `build_arrow_array()` - Per-column array building
- Type promotion logic

**Test cases**:
- Empty list → empty RecordBatch
- Homogeneous types (all Int, all String)
- Type promotion (Int + Float → Float, Null + X → X)
- **Error**: Mixed incompatible types (String + Int)
- **Error**: Non-Map elements in list
- **Error**: Late-appearing columns
- Null handling

#### Step 1.5: Update SQL Module Exports

**File**: `crates/lattice/src/sql/mod.rs`

```rust
pub mod convert;  // existing
pub mod refs;     // new - table reference extraction
pub mod ident;    // new - identifier utilities

#[cfg(feature = "sql-arrow")]
pub mod arrow;    // new - Arrow conversion
```

### Phase 2: Provider Extension

#### Step 2.1: Add table_exists to SqlProvider

**File**: `crates/lattice/src/runtime/providers/sql.rs`

```rust
pub trait SqlProvider: Send + Sync {
    // Existing
    fn query(&self, sql: &str) -> Result<SqlResult, SqlError>;
    fn execute(&self, sql: &str) -> Result<usize, SqlError>;
    
    // NEW: Check if table exists in provider
    fn table_exists(&self, name: &str) -> Result<bool, SqlError>;
}
```

#### Step 2.2: Create SqlArrowProvider Trait

**File**: `crates/lattice/src/runtime/providers/sql.rs`

```rust
#[cfg(feature = "sql-arrow")]
pub trait SqlArrowProvider: SqlProvider {
    fn register_arrow_table(&self, name: &str, data: Arc<RecordBatch>) -> Result<(), SqlError>;
    fn unregister_table(&self, name: &str) -> Result<(), SqlError>;
}
```

#### Step 2.3: Implement for DuckDbProvider

**File**: `crates/lattice/src/runtime/providers/sql.rs`

- Add `registered_tables: HashMap<String, Arc<RecordBatch>>` field
- Implement `table_exists` using DuckDB catalog query
- Implement `register_arrow_table` with proper quoting
- Implement `unregister_table` with cleanup

#### Step 2.4: Implement for NoSqlProvider

Return appropriate error: "SQL on Lattice data requires sql-arrow feature"

### Phase 3: VM Integration

#### Step 3.1: Modify op_sql_query

**File**: `crates/lattice/src/vm/machine.rs`

Implement the full flow with:
- Pre-validation of all tables
- Clear error messages
- Guaranteed cleanup

#### Step 3.2: Modify op_sql_query_typed

**File**: `crates/lattice/src/vm/machine.rs`

Same changes, reusing the same registration logic.

#### Step 3.3: Define Error Types

**File**: `crates/lattice/src/error.rs`

```rust
pub enum LatticeError {
    // ... existing variants
    
    SqlTableNotFound { name: String, hint: &'static str },
    SqlWrongType { name: String, expected: &'static str, found: String },
    SqlUnsupportedConstruct { construct: &'static str, hint: &'static str },
    SqlSchemaInference { column: String, message: String },
}
```

### Phase 4: Testing

#### Step 4.1: Unit Tests

**Files**:
- `crates/lattice/src/sql/refs.rs` - Table extraction tests
- `crates/lattice/src/sql/ident.rs` - Identifier tests
- `crates/lattice/src/sql/arrow.rs` - Conversion tests

#### Step 4.2: Integration Tests

**File**: `crates/lattice/tests/sql_on_data.rs` (new)

```rust
#[test]
fn test_sql_on_lattice_list() {
    let rt = RuntimeBuilder::new()
        .with_default_sql_provider()
        .build()
        .unwrap()
        .into_runtime();

    rt.eval(r#"
        let users = [
            {id: 1, name: "Alice", age: 30},
            {id: 2, name: "Bob", age: 18}
        ]
    "#).unwrap();

    let result = rt.eval(r#"SQL("SELECT * FROM users WHERE age > 21")"#).unwrap();
    // Assert: single row with Alice
}

#[test]
fn test_sql_rejects_non_list_map() {
    // Variable is List<Int> - should error
}

#[test]
fn test_sql_rejects_cte() {
    // WITH clause - should error with clear message
}

#[test]
fn test_sql_mixed_lattice_and_file_tables() {
    // Query joining Lattice variable with CSV file
}

#[test]
fn test_sql_shadowing_warning() {
    // DuckDB table takes precedence, Lattice var ignored
}
```

#### Step 4.3: Binding Tests

Verify the feature works through each binding:
- `crates/lattice-py/tests/`
- `crates/lattice-node/tests/`
- `crates/lattice-nif/` (Elixir tests)

### Phase 5: Documentation

#### Step 5.1: Update SQL Documentation

Document:
- New capability with examples
- Supported data shapes (List<Map>)
- Type mapping and promotion rules
- Identifier naming rules
- Limitations (no subqueries, no CTEs, no mutations)
- Error messages and troubleshooting

#### Step 5.2: Update Binding READMEs

Add examples for each language showing the feature.

---

## 5. Testing Strategy

### 5.1 Test Matrix

| Scenario | Unit | Integration | Binding |
|----------|------|-------------|---------|
| Empty list | ✓ | ✓ | |
| Single column | ✓ | ✓ | |
| Multiple columns | ✓ | ✓ | |
| All Lattice types | ✓ | ✓ | |
| Null values | ✓ | ✓ | |
| Type promotion (Int+Float) | ✓ | ✓ | |
| **Error**: Mixed types (String+Int) | ✓ | ✓ | |
| **Error**: Non-Map elements | ✓ | ✓ | |
| **Error**: CTE query | ✓ | ✓ | |
| **Error**: Subquery | ✓ | ✓ | |
| Large dataset (1M rows) | | ✓ | |
| Multiple tables in query | | ✓ | |
| Mixed Lattice + file tables | | ✓ | |
| Shadowed variable warning | | ✓ | |
| SQL injection attempts | ✓ | ✓ | |
| Python integration | | | ✓ |
| Node.js integration | | | ✓ |
| Elixir integration | | | ✓ |

### 5.2 Performance Benchmarks

**File**: `crates/lattice/benches/sql_on_data.rs`

Measure:
- Registration overhead (time to convert + register)
- Query execution time vs. native DuckDB table
- Memory usage during query
- Narrow vs wide tables (few columns vs many columns)

Target: <10% overhead vs. pre-loaded DuckDB table for datasets <100K rows.

---

## 6. Rollout Plan

### 6.1 Feature Flag

Initially behind `sql-arrow` feature flag:

```toml
# Users opt-in
lattice = { version = "...", features = ["sql-arrow"] }
```

### 6.2 Binding Build Configuration

Each binding's build must enable the feature:
- `crates/lattice-py/Cargo.toml`: Add `sql-arrow` to default features
- `crates/lattice-node/Cargo.toml`: Add `sql-arrow` to default features  
- `crates/lattice-nif/Cargo.toml`: Add `sql-arrow` to default features

### 6.3 CI Matrix

Test builds:
- `sql` only (no Arrow integration)
- `sql + sql-arrow` (full feature)
- `no-sql` (SQL disabled entirely)

### 6.4 Graduation Criteria

Move to default `sql` feature when:
1. All tests pass
2. Performance benchmarks acceptable
3. No breaking changes to existing SQL behavior
4. Documentation complete

---

## 7. Future Enhancements (Out of Scope)

1. **Subquery support**: `FROM (SELECT ...) AS t`
2. **CTE support**: `WITH t AS (...)`
3. **Mutation support**: `UPDATE`, `DELETE` reflecting back
4. **Streaming large datasets**: For lists that don't fit in memory
5. **Index hints**: Optimize queries on known data shapes
6. **Query caching**: Cache Arrow conversions for repeated queries on same data
7. **Explicit table passing**: `SQL_FROM(users, "SELECT * FROM users")` for explicit scoping

---

## 8. Resolved Questions

### 8.1 Schema Inference Strategy
**Decision**: First N rows (default 100), configurable.
- Union of keys across sampled rows
- Missing entries become null
- Error if columns appear after inference window

### 8.2 Name Collision Handling
**Decision**: DuckDB table takes precedence (Option C).
- Preserves existing behavior
- Emit debug warning when Lattice variable is shadowed
- Document in user-facing docs

### 8.3 Memory Management
**Decision**: Arrow RecordBatch freed after query completes.
- `DuckDbProvider` holds `Arc<RecordBatch>` while registered
- Cleanup always runs, even on query error
- No cross-query caching in v1

### 8.4 Variable Scoping
**Decision**: SQL only sees global variables.
- Local variables in functions are not visible to SQL
- Documented as a language rule
- Future: Consider explicit `SQL_FROM()` for explicit data passing

---

## 9. Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| sqlparser | 0.43.x | SQL AST parsing |
| arrow | 56.x | Columnar data format |
| duckdb | 1.x | With `vtab-arrow` feature |

---

## 10. Success Metrics

1. **Functionality**: All test scenarios pass
2. **Performance**: <10% overhead vs native tables for <100K rows
3. **Adoption**: Feature used in real user workflows (track via docs page views, GitHub issues)
4. **Stability**: No regressions in existing SQL functionality
5. **Error Quality**: Users can self-diagnose issues from error messages alone
