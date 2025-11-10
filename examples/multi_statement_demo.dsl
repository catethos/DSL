# Multi-Statement Functions Demo
# Demonstrates the new syntax for readable, step-by-step computations

def analyze_number(n) {
    let squared = n * n;
    let cubed = squared * n;
    let sum = squared + cubed;

    {
        original: n,
        squared: squared,
        cubed: cubed,
        sum: sum
    }
}

def classify_value(value) {
    let abs_val = value < 0 ? -value : value;
    let category = abs_val < 10 ? "small" : (abs_val < 100 ? "medium" : "large");

    {
        value: value,
        absolute: abs_val,
        category: category
    }
}

# Test the functions
analyze_number(5)
