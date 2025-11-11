// Test map and filter with lambdas and string references

def double(x) {
    x * 2
}

def isEven(x) {
    x % 2 == 0
}

def test_map_lambda() {
    map([1, 2, 3, 4], fn x => x * 2 end)
}

def test_map_string() {
    map([1, 2, 3, 4], "double")
}

def test_filter_lambda() {
    filter([1, 2, 3, 4, 5], fn x => x > 2 end)
}

def test_filter_string() {
    filter([1, 2, 3, 4, 5, 6], "isEven")
}

def test_chaining() {
    [1, 2, 3, 4, 5, 6]
    |> filter(_, fn x => x > 2 end)
    |> map(_, fn x => x * 2 end)
}

// Call all tests
test_map_lambda()
