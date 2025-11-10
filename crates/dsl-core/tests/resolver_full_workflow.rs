use dsl_core::compile_program_with_resolver;

#[test]
fn test_full_workflow_simple_functions() {
    let code = r#"
def add(a, b) => a + b
def multiply(x, y) => x * y

add(2, 3)
"#;

    let ir = compile_program_with_resolver(code).unwrap();

    // Should have 2 simple functions (compiled as IRFunction)
    assert_eq!(ir.functions.len(), 2);
    assert_eq!(ir.function_groups.len(), 0);

    // Verify function names
    let names: Vec<&str> = ir.functions.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"add"));
    assert!(names.contains(&"multiply"));
}

#[test]
fn test_full_workflow_pattern_functions() {
    let code = r#"
def factorial(0) => 1
def factorial(1) => 1
def factorial(n) => n * factorial(n - 1)

factorial(5)
"#;

    let ir = compile_program_with_resolver(code).unwrap();

    // Should have 1 pattern function (compiled as IRFunctionGroup)
    assert_eq!(ir.functions.len(), 0);
    assert_eq!(ir.function_groups.len(), 1);

    let group = &ir.function_groups[0];
    assert_eq!(group.name, "factorial");
    assert_eq!(group.clauses.len(), 3);
}

#[test]
fn test_full_workflow_mixed() {
    let code = r#"
def add(a, b) => a + b
def factorial(0) => 1
def factorial(n) => n * factorial(n - 1)
def identity(x) => x

add(factorial(5), identity(10))
"#;

    let ir = compile_program_with_resolver(code).unwrap();

    // Should have 2 simple functions and 1 pattern function
    assert_eq!(ir.functions.len(), 2); // add, identity
    assert_eq!(ir.function_groups.len(), 1); // factorial

    // Verify function names
    let func_names: Vec<&str> = ir.functions.iter().map(|f| f.name.as_str()).collect();
    assert!(func_names.contains(&"add"));
    assert!(func_names.contains(&"identity"));

    assert_eq!(ir.function_groups[0].name, "factorial");
}

#[test]
fn test_full_workflow_different_arities() {
    let code = r#"
def add(a, b) => a + b
def add(a, b, c) => a + b + c

add(1, 2)
"#;

    let ir = compile_program_with_resolver(code).unwrap();

    // Should have 2 simple functions (different arities)
    assert_eq!(ir.functions.len(), 2);
    assert_eq!(ir.function_groups.len(), 0);

    // Both should be named "add" but have different param counts
    for func in &ir.functions {
        assert_eq!(func.name, "add");
    }

    let arities: Vec<usize> = ir.functions.iter().map(|f| f.params.len()).collect();
    assert!(arities.contains(&2));
    assert!(arities.contains(&3));
}

#[test]
fn test_full_workflow_converts_duplicate_to_pattern() {
    let code = r#"
def add(a, b) => a + b
def add(x, y) => x + y

add(1, 2)
"#;

    let ir = compile_program_with_resolver(code).unwrap();

    // Two simple functions with same name/arity should become a pattern function
    assert_eq!(ir.functions.len(), 0);
    assert_eq!(ir.function_groups.len(), 1);

    let group = &ir.function_groups[0];
    assert_eq!(group.name, "add");
    assert_eq!(group.clauses.len(), 2);
}

#[test]
fn test_full_workflow_complex_example() {
    let code = r#"
def square(x) => x * x
def double(x) => x + x

def fib(0) => 0
def fib(1) => 1
def fib(n) => fib(n - 1) + fib(n - 2)

def sum(a, b) => a + b
def sum(a, b, c) => a + b + c

double(square(fib(5)))
"#;

    let ir = compile_program_with_resolver(code).unwrap();

    // Should have 4 simple functions (square, double, sum/2, sum/3)
    assert_eq!(ir.functions.len(), 4);

    // Should have 1 pattern function (fib)
    assert_eq!(ir.function_groups.len(), 1);

    let fib_group = &ir.function_groups[0];
    assert_eq!(fib_group.name, "fib");
    assert_eq!(fib_group.clauses.len(), 3);
}

#[test]
fn test_backward_compatibility() {
    // The old compile_to_ir should still work
    use dsl_core::compile_to_ir;

    let code = r#"
def add(a, b) => a + b
add(2, 3)
"#;

    // Old workflow
    let ir_old = compile_to_ir(code).unwrap();

    // New workflow
    let ir_new = compile_program_with_resolver(code).unwrap();

    // Both should produce valid IR
    assert!(!ir_old.functions.is_empty() || !ir_old.function_groups.is_empty());
    assert!(!ir_new.functions.is_empty() || !ir_new.function_groups.is_empty());
}
