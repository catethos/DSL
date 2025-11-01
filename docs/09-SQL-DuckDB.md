# SQL and DuckDB Integration

## Overview

The DSL integrates **DuckDB** - a fast, in-memory analytical database - for powerful data processing with SQL.

## Key Features

- **In-memory database** - Fast performance
- **CSV file reading** - Direct SQL queries on CSV files
- **Full SQL support** - JOINs, aggregations, window functions
- **Table management** - Automatic registration
- **Template interpolation** - Use variables in queries

## Quick Start

### Direct CSV Querying

```javascript
SQL("SELECT * FROM 'data.csv'")
// Returns Table with all rows

SQL("SELECT * FROM 'data.csv' WHERE age > 25")
// Returns filtered Table
```

### With Variable

```javascript
SQL("SELECT * FROM 'users.csv'") as users
SQL("SELECT COUNT(*) FROM users") as count
```

## SQL() Function

Built-in function for executing SQL queries.

**Syntax:**
```javascript
SQL(query: String) -> Table
```

**Examples:**
```javascript
// Simple query
SQL("SELECT * FROM 'data.csv'")

// With WHERE clause
SQL("SELECT * FROM 'sales.csv' WHERE amount > 100")

// With aggregation
SQL("SELECT category, SUM(sales) FROM 'data.csv' GROUP BY category")
```

## Table Operations

### Creating Tables from Variables

Tables are automatically registered:

```javascript
SQL("SELECT * FROM 'data.csv'") as mydata
// 'mydata' is now a table in DuckDB

SQL("SELECT * FROM mydata WHERE score > 0.5")
// Query the registered table
```

### Joining Tables

```javascript
SQL("SELECT * FROM 'users.csv'") as users
SQL("SELECT * FROM 'orders.csv'") as orders

SQL("""
  SELECT u.name, o.amount
  FROM users u
  JOIN orders o ON u.id = o.user_id
""")
```

### Multiple Operations

```javascript
SQL("SELECT * FROM 'raw.csv'") as raw
  >> SQL("SELECT * FROM raw WHERE value IS NOT NULL") as clean
  >> SQL("SELECT AVG(value) FROM clean") as avg
```

## User-Defined SQL Functions

### Basic SQL Function

```javascript
def FilterHighScores(data: Table, threshold: Float) -> Table {
  sql: """
    SELECT *
    FROM data
    WHERE score >= ${threshold}
    ORDER BY score DESC
  """
}

SQL("SELECT * FROM 'scores.csv'") as scores
  >> FilterHighScores(scores, 0.8)
```

### Aggregation Function

```javascript
def SummarizeByCategory(data: Table) -> Table {
  sql: """
    SELECT
      category,
      COUNT(*) as count,
      AVG(value) as avg_value,
      SUM(value) as total_value
    FROM data
    GROUP BY category
    ORDER BY total_value DESC
  """
}
```

### Window Function

```javascript
def TopNPerGroup(data: Table, n: Int) -> Table {
  sql: """
    WITH ranked AS (
      SELECT
        *,
        ROW_NUMBER() OVER (
          PARTITION BY category
          ORDER BY score DESC
        ) as rank
      FROM data
    )
    SELECT * FROM ranked WHERE rank <= ${n}
  """
}
```

## Template Interpolation

Use `${variable}` to inject values:

```javascript
def FilterByThreshold(data: Table, threshold: Float) -> Table {
  sql: """
    SELECT * FROM data WHERE score > ${threshold}
  """
}

42 as minAge
SQL("SELECT * FROM 'users.csv' WHERE age > ${minAge}")
```

## Common SQL Patterns

### Pattern 1: Filter → Aggregate

```javascript
SQL("SELECT * FROM 'sales.csv'") as sales
  >> SQL("SELECT * FROM sales WHERE amount > 100") as filtered
  >> SQL("SELECT SUM(amount) FROM filtered") as total
```

### Pattern 2: Group By Analysis

```javascript
SQL("""
  SELECT
    region,
    COUNT(*) as count,
    AVG(revenue) as avg_revenue,
    SUM(revenue) as total_revenue
  FROM 'sales.csv'
  GROUP BY region
  ORDER BY total_revenue DESC
""")
```

### Pattern 3: Join and Filter

```javascript
SQL("SELECT * FROM 'users.csv'") as users
SQL("SELECT * FROM 'purchases.csv'") as purchases

SQL("""
  SELECT
    u.name,
    u.email,
    p.product,
    p.amount
  FROM users u
  INNER JOIN purchases p ON u.id = p.user_id
  WHERE p.amount > 100
""")
```

### Pattern 4: Ranking

```javascript
SQL("""
  SELECT
    *,
    DENSE_RANK() OVER (ORDER BY score DESC) as rank
  FROM 'scores.csv'
""") as ranked
  >> SQL("SELECT * FROM ranked WHERE rank <= 10")
```

## Advanced SQL Features

### Supported SQL Operations

- **SELECT, FROM, WHERE** - Basic queries
- **JOIN** (INNER, LEFT, RIGHT, FULL) - Table joins
- **GROUP BY, HAVING** - Aggregation
- **ORDER BY, LIMIT** - Sorting and limiting
- **DISTINCT** - Unique values
- **UNION, INTERSECT, EXCEPT** - Set operations
- **WITH (CTEs)** - Common table expressions
- **Window functions** - RANK, ROW_NUMBER, LAG, LEAD, etc.
- **Subqueries** - Nested queries

### Window Functions

```javascript
SQL("""
  SELECT
    date,
    value,
    AVG(value) OVER (
      ORDER BY date
      ROWS BETWEEN 2 PRECEDING AND CURRENT ROW
    ) as moving_avg
  FROM 'timeseries.csv'
""")
```

### CTEs (Common Table Expressions)

```javascript
SQL("""
  WITH monthly_sales AS (
    SELECT
      EXTRACT(MONTH FROM date) as month,
      SUM(amount) as total
    FROM 'sales.csv'
    GROUP BY month
  )
  SELECT
    month,
    total,
    total - LAG(total) OVER (ORDER BY month) as growth
  FROM monthly_sales
""")
```

## Integration with Other Features

### SQL + LLM

```javascript
SQL("SELECT * FROM 'articles.csv' WHERE score > 0.8") as top
  >> AnalyzeArticles(top) as insights
```

### SQL + HTTP

```javascript
def FetchAndAnalyze(category: String) -> Report {
  http: "GET"
  url: "https://api.example.com/data?category=${category}"

  // Result automatically converted to table
  sql: """
    SELECT
      category,
      COUNT(*) as count,
      AVG(score) as avg_score
    FROM _
    GROUP BY category
  """
}
```

### Parallel SQL Queries

```javascript
(
  SQL("SELECT COUNT(*) FROM 'data.csv'") ||
  SQL("SELECT AVG(value) FROM 'data.csv'") ||
  SQL("SELECT MAX(value) FROM 'data.csv'")
) as [count, avg, max]
```

## Performance Tips

### 1. Filter Early

```javascript
// Good: Filter first
SQL("SELECT * FROM 'huge.csv' WHERE category = 'A'") as filtered
  >> SQL("SELECT AVG(value) FROM filtered")

// Bad: Load all then filter
SQL("SELECT * FROM 'huge.csv'") as all
  >> SQL("SELECT AVG(value) FROM all WHERE category = 'A'")
```

### 2. Use Appropriate Aggregations

```javascript
// Good: Single pass
SQL("SELECT category, AVG(value), COUNT(*) FROM 'data.csv' GROUP BY category")

// Bad: Multiple passes
SQL("SELECT category, AVG(value) FROM 'data.csv' GROUP BY category")
SQL("SELECT category, COUNT(*) FROM 'data.csv' GROUP BY category")
```

### 3. Index-Friendly Queries

DuckDB automatically optimizes, but writing efficient SQL helps:

```javascript
// Good: Specific columns
SQL("SELECT name, age FROM 'users.csv'")

// Bad: Select all when not needed
SQL("SELECT * FROM 'users.csv'")
```

## Limitations

### Current Limitations

1. **In-Memory Only** - Large files may exhaust memory
2. **No Persistent Database** - Tables cleared after session
3. **CSV/JSON Only** - Limited file format support
4. **No Database Modifications** - Read-only operations

### Workarounds

**Large Files:**
```javascript
// Filter in the SQL itself
SQL("SELECT * FROM 'huge.csv' WHERE date > '2024-01-01'")
```

**Persistence:**
```javascript
// Export results
:copy result.csv
```

## Best Practices

### 1. Use Clear Aliases

```javascript
SQL("""
  SELECT
    u.name,
    u.email,
    o.total
  FROM users u
  JOIN orders o ON u.id = o.user_id
""")
```

### 2. Format Long Queries

```javascript
def ComplexQuery() {
  sql: """
    SELECT
      category,
      COUNT(*) as count,
      AVG(value) as avg,
      SUM(value) as total
    FROM data
    WHERE
      date >= '2024-01-01'
      AND status = 'active'
    GROUP BY category
    HAVING COUNT(*) > 10
    ORDER BY total DESC
    LIMIT 100
  """
}
```

### 3. Validate Data First

```javascript
SQL("SELECT * FROM 'data.csv'") as raw
SQL("SELECT COUNT(*) FROM raw WHERE value IS NULL") as nulls
// Check nulls before processing
```

## Next Steps

- **[06-Functions.md](06-Functions.md)** - Define SQL functions
- **[07-Workflow-Constructs.md](07-Workflow-Constructs.md)** - SQL workflows
- **[examples/](../examples/)** - SQL examples

## Related Documents

- **[CSV_USAGE.md](../CSV_USAGE.md)** - CSV file handling
- **[PHASE6_EXAMPLES.md](../PHASE6_EXAMPLES.md)** - SQL examples
- **[PHASE6_SUMMARY.md](../PHASE6_SUMMARY.md)** - SQL implementation details
