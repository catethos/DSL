# Rich Display System Test
# This file tests all the different output renderers

# Test 1: Table Rendering (automatic detection)
# A list of maps with consistent keys should display as a table
let users = [
  {name: "Alice", age: 30, city: "NYC"},
  {name: "Bob", age: 25, city: "LA"},
  {name: "Charlie", age: 35, city: "SF"}
]

# Test 2: Tree Rendering
# Nested structures should display as an interactive tree
let config = {
  app: {
    name: "MyApp",
    version: "1.0.0",
    settings: {
      debug: true,
      port: 8080,
      features: ["auth", "api", "admin"]
    }
  },
  database: {
    host: "localhost",
    port: 5432
  }
}

# Test 3: Simple List (should be a tree)
let numbers = [1, 2, 3, 4, 5]

# Test 4: Markdown Output (if supported by interpreter)
# Note: This requires the interpreter to return Value::Markdown
# For now, regular text will be displayed

# Test 5: Chart Data (Line Chart)
# Format: list of [x, y] pairs
let chart_data = [
  [1, 10],
  [2, 20],
  [3, 15],
  [4, 25],
  [5, 30]
]

# Test 6: Nested Table (List of lists)
let matrix = [
  [1, 2, 3],
  [4, 5, 6],
  [7, 8, 9]
]

# Test 7: Mixed Data Types
let mixed = {
  string: "Hello",
  number: 42,
  float: 3.14,
  bool: true,
  null_val: null,
  list: [1, 2, 3],
  nested: {a: 1, b: 2}
}
