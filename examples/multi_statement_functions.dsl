# Multi-Statement Functions Example
# This demonstrates the new multi-statement function body syntax

# Simple multi-statement function
def compute(x) {
  let doubled = x * 2
  let added = doubled + 1
  added
}

# More complex example with intermediate calculations
def analyze_number(n) {
  let squared = n * n
  let cubed = squared * n
  let sum = squared + cubed
  {
    original: n,
    squared: squared,
    cubed: cubed,
    sum: sum
  }
}

# Function with conditional logic
def classify(value) {
  let abs_val = value < 0 ? -value : value
  let category = abs_val < 10 ? "small" : (abs_val < 100 ? "medium" : "large")
  {
    value: value,
    absolute: abs_val,
    category: category
  }
}

# Test the functions
compute(5)
