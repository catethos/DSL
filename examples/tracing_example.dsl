// Tracing Example - Demonstrates the tracing interpreter
// This DSL program will be executed with full tracing enabled

// Define a recursive factorial function
function factorial(n) {
    n == 0 ? 1 : n * factorial(n - 1)
}

// Define a function with multiple steps
function compute(x) {
    let doubled = x * 2;
    let added = doubled + 10;
    added
}

// Main execution
let numbers = [1, 2, 3, 4, 5];

// Use higher-order function (will generate many trace events)
let squares = Map(numbers, function(n) { n * n });

// Compute factorial (recursive, generates deep traces)
let fact5 = factorial(5);

// Multi-step computation
let result = compute(20);

// Return summary
{
    squares: squares,
    factorial: fact5,
    computed: result
}
