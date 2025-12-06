//! VM Performance Benchmarks
//!
//! Benchmarks for the Lattice VM to measure baseline performance
//! and track improvements from optimizations.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use lattice::compiler::Compiler;
use lattice::syntax::parser;
use lattice::vm::VM;

/// Helper to compile and run Lattice code
fn run_code(source: &str) -> lattice::types::Value {
    let program = parser::parse(source).expect("parse failed");
    let compile_result = Compiler::compile(&program).expect("compile failed");

    let mut vm = VM::new();

    for class in compile_result.classes {
        vm.ir_mut().classes.push(class);
    }
    for enum_def in compile_result.enums {
        vm.ir_mut().enums.push(enum_def);
    }
    for func in compile_result.functions {
        vm.register_function(func);
    }
    for llm_func in compile_result.llm_functions {
        vm.register_llm_function(llm_func);
    }

    vm.run(&compile_result.chunk).expect("run failed")
}

/// Benchmark tight arithmetic loop
fn bench_arithmetic(c: &mut Criterion) {
    let mut group = c.benchmark_group("arithmetic");

    // Simple addition in a while loop
    let code_add = r#"
        let sum = 0
        let i = 0
        while i < 1000 {
            sum = sum + i
            i = i + 1
        }
        sum
    "#;

    group.bench_function("add_loop_1000", |b| {
        b.iter(|| run_code(black_box(code_add)))
    });

    // Mixed arithmetic
    let code_mixed = r#"
        let result = 0
        let i = 1
        while i < 100 {
            result = (result + i * 2) - i / 2
            i = i + 1
        }
        result
    "#;

    group.bench_function("mixed_arithmetic_100", |b| {
        b.iter(|| run_code(black_box(code_mixed)))
    });

    group.finish();
}

/// Benchmark fibonacci (tests function calls and recursion)
fn bench_fibonacci(c: &mut Criterion) {
    let mut group = c.benchmark_group("fibonacci");

    // Iterative fibonacci
    let code_iter = r#"
        def fib_iter(n: Int) -> Int {
            if n <= 1 {
                return n
            }
            let a = 0
            let b = 1
            let i = 2
            while i <= n {
                let temp = a + b
                a = b
                b = temp
                i = i + 1
            }
            b
        }
        fib_iter(30)
    "#;

    group.bench_function("iterative_fib_30", |b| {
        b.iter(|| run_code(black_box(code_iter)))
    });

    // Recursive fibonacci (smaller N due to stack depth)
    let code_rec = r#"
        def fib(n: Int) -> Int {
            if n <= 1 {
                return n
            }
            fib(n - 1) + fib(n - 2)
        }
        fib(20)
    "#;

    group.bench_function("recursive_fib_20", |b| {
        b.iter(|| run_code(black_box(code_rec)))
    });

    group.finish();
}

/// Benchmark function calls
fn bench_function_calls(c: &mut Criterion) {
    let mut group = c.benchmark_group("function_calls");

    // Many small function calls
    let code = r#"
        def increment(x: Int) -> Int {
            x + 1
        }

        let result = 0
        let i = 0
        while i < 500 {
            result = increment(result)
            i = i + 1
        }
        result
    "#;

    group.bench_function("simple_calls_500", |b| {
        b.iter(|| run_code(black_box(code)))
    });

    // Nested function calls
    let code_nested = r#"
        def add(a: Int, b: Int) -> Int { a + b }
        def mul(a: Int, b: Int) -> Int { a * b }
        def compute(x: Int) -> Int {
            add(mul(x, 2), mul(x, 3))
        }

        let sum = 0
        let i = 1
        while i < 100 {
            sum = sum + compute(i)
            i = i + 1
        }
        sum
    "#;

    group.bench_function("nested_calls_100", |b| {
        b.iter(|| run_code(black_box(code_nested)))
    });

    group.finish();
}

/// Benchmark collection operations
fn bench_collections(c: &mut Criterion) {
    let mut group = c.benchmark_group("collections");

    // List creation and indexing using push()
    let code_list = r#"
        let list = []
        let i = 0
        while i < 100 {
            list = push(list, i)
            i = i + 1
        }
        let sum = 0
        let j = 0
        while j < 100 {
            sum = sum + list[j]
            j = j + 1
        }
        sum
    "#;

    group.bench_function("list_build_and_index_100", |b| {
        b.iter(|| run_code(black_box(code_list)))
    });

    // Map operations
    let code_map = r#"
        let map = {}
        let i = 0
        while i < 50 {
            map[f"key_{i}"] = i * 2
            i = i + 1
        }
        let sum = 0
        let j = 0
        while j < 50 {
            sum = sum + map[f"key_{j}"]
            j = j + 1
        }
        sum
    "#;

    group.bench_function("map_build_and_access_50", |b| {
        b.iter(|| run_code(black_box(code_map)))
    });

    group.finish();
}

/// Benchmark control flow
fn bench_control_flow(c: &mut Criterion) {
    let mut group = c.benchmark_group("control_flow");

    // Branches
    let code_branches = r#"
        let count = 0
        let i = 0
        while i < 500 {
            if i % 2 == 0 {
                count = count + 1
            } else if i % 3 == 0 {
                count = count + 2
            } else {
                count = count + 3
            }
            i = i + 1
        }
        count
    "#;

    group.bench_function("branches_500", |b| {
        b.iter(|| run_code(black_box(code_branches)))
    });

    // While loop
    let code_while = r#"
        let i = 0
        let sum = 0
        while i < 1000 {
            sum = sum + i
            i = i + 1
        }
        sum
    "#;

    group.bench_function("while_loop_1000", |b| {
        b.iter(|| run_code(black_box(code_while)))
    });

    group.finish();
}

/// Benchmark global variable access (important for LLM use case)
fn bench_globals(c: &mut Criterion) {
    let mut group = c.benchmark_group("globals");

    // Many global reads/writes
    let code = r#"
        let a = 0
        let b = 0
        let c = 0
        let i = 0
        while i < 200 {
            a = a + 1
            b = b + a
            c = c + b
            i = i + 1
        }
        a + b + c
    "#;

    group.bench_function("global_read_write_200", |b| {
        b.iter(|| run_code(black_box(code)))
    });

    group.finish();
}

/// Benchmark struct operations
fn bench_structs(c: &mut Criterion) {
    let mut group = c.benchmark_group("structs");

    // Struct creation and field access
    let code = r#"
        type Point {
            x: Int,
            y: Int
        }

        def make_point(px: Int, py: Int) -> Point {
            Point { x: px, y: py }
        }

        def distance_sq(p: Point) -> Int {
            p.x * p.x + p.y * p.y
        }

        let sum = 0
        let i = 0
        while i < 100 {
            let p = make_point(i, i * 2)
            sum = sum + distance_sq(p)
            i = i + 1
        }
        sum
    "#;

    group.bench_function("struct_create_access_100", |b| {
        b.iter(|| run_code(black_box(code)))
    });

    group.finish();
}

/// Benchmark string operations (important for LLM prompts)
fn bench_strings(c: &mut Criterion) {
    let mut group = c.benchmark_group("strings");

    // F-string interpolation
    let code = r#"
        let result = ""
        let i = 0
        while i < 50 {
            result = f"iteration {i}: {result}"
            i = i + 1
        }
        len(result)
    "#;

    group.bench_function("fstring_concat_50", |b| {
        b.iter(|| run_code(black_box(code)))
    });

    group.finish();
}

/// Benchmark scaling behavior
fn bench_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("scaling");

    for size in [10, 100, 500, 1000].iter() {
        let code = format!(r#"
            let sum = 0
            let i = 0
            while i < {} {{
                sum = sum + i
                i = i + 1
            }}
            sum
        "#, size);

        group.bench_with_input(
            BenchmarkId::new("loop_iterations", size),
            size,
            |b, _| b.iter(|| run_code(black_box(&code)))
        );
    }

    group.finish();
}

/// Benchmark parallel_map operations
///
/// This tests the VM-level parallel_map infrastructure.
/// For actual LLM parallel benchmarks, use a mock API server.
fn bench_parallel_map(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_map");

    // Simple parallel_map with identity function
    let code_identity = r#"
        let items = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
        parallel_map(items, |x| x)
    "#;

    group.bench_function("identity_10", |b| {
        b.iter(|| run_code(black_box(code_identity)))
    });

    // parallel_map with arithmetic
    let code_arithmetic = r#"
        let items = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
        parallel_map(items, |x| x * 2 + 1)
    "#;

    group.bench_function("arithmetic_10", |b| {
        b.iter(|| run_code(black_box(code_arithmetic)))
    });

    // Compare parallel_map vs for loop with equivalent operation
    for size in [10, 50, 100].iter() {
        // Build list code
        let build_list = (1..=*size).map(|i| i.to_string()).collect::<Vec<_>>().join(", ");

        // parallel_map version
        let code_parallel = format!(r#"
            let items = [{}]
            parallel_map(items, |x| x * 2)
        "#, build_list);

        group.bench_with_input(
            BenchmarkId::new("parallel_map_mul2", size),
            size,
            |b, _| b.iter(|| run_code(black_box(&code_parallel)))
        );

        // For loop equivalent
        let code_for = format!(r#"
            let items = [{}]
            let results = []
            for item in items {{
                results = push(results, item * 2)
            }}
            results
        "#, build_list);

        group.bench_with_input(
            BenchmarkId::new("for_loop_mul2", size),
            size,
            |b, _| b.iter(|| run_code(black_box(&code_for)))
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_arithmetic,
    bench_fibonacci,
    bench_function_calls,
    bench_collections,
    bench_control_flow,
    bench_globals,
    bench_structs,
    bench_strings,
    bench_scaling,
    bench_parallel_map,
);

criterion_main!(benches);
