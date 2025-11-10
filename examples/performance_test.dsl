// Performance test: Generate many output items to test virtual scrolling
// and lazy loading

// Test 1: Generate many text items
def generate_text_items(count) = {
  range(1, count) | map(i -> "Item " + str(i))
}

// Test 2: Generate table data
def generate_table(rows) = {
  range(1, rows) | map(i -> {
    id: i,
    name: "Person " + str(i),
    age: 20 + (i % 50),
    score: (i * 7) % 100
  })
}

// Test 3: Simulate image outputs
// Note: Replace paths with actual image paths on your system
def test_images() = {
  "Test complete - check output for performance"
}

// Usage examples:
// generate_text_items(100)   // Test with 100 text items
// generate_table(200)        // Test with 200 row table
// test_images()              // Test with multiple images
