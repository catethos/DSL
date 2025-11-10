# Scope Stack Performance Benchmark
# Tests recursive functions with multi-statement bodies

def factorial(n) {
    let is_zero = n == 0;
    if is_zero {
        1
    } else {
        let n_minus_1 = n - 1;
        let factorial_rec = factorial(n_minus_1);
        n * factorial_rec
    }
}

def fib(n) {
    let base_case = n <= 1;
    if base_case {
        n
    } else {
        let n_minus_1 = n - 1;
        let n_minus_2 = n - 2;
        let fib_1 = fib(n_minus_1);
        let fib_2 = fib(n_minus_2);
        fib_1 + fib_2
    }
}

def run_benchmarks() {
    let test1 = factorial(15);
    let test2 = fib(15);

    {
        factorial_15: test1,
        fibonacci_15: test2
    }
}

# Run the benchmarks
run_benchmarks()
