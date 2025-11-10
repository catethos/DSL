use dsl_core::{compile_function_group, parse_program, resolve_program, SymbolTable};

#[test]
fn test_compile_trivial_function_group() {
    let code = r#"
def add(a, b) => a + b
"#;

    let program = parse_program(code).unwrap();
    let mut symbol_table = SymbolTable::new();
    let groups = resolve_program(program, &mut symbol_table).unwrap();

    assert_eq!(groups.len(), 1);
    let group = &groups[0];
    assert!(group.is_trivial());

    // Compile the group
    let (ir_func, ir_group) = compile_function_group(group).unwrap();

    // Trivial functions should compile to IRFunction, not IRFunctionGroup
    assert!(ir_func.is_some());
    assert!(ir_group.is_none());

    let ir_func = ir_func.unwrap();
    assert_eq!(ir_func.name, "add");
    assert_eq!(ir_func.params, vec!["a", "b"]);
}

#[test]
fn test_compile_pattern_function_group() {
    let code = r#"
def factorial(0) => 1
def factorial(n) => n * factorial(n - 1)
"#;

    let program = parse_program(code).unwrap();
    let mut symbol_table = SymbolTable::new();
    let groups = resolve_program(program, &mut symbol_table).unwrap();

    assert_eq!(groups.len(), 1);
    let group = &groups[0];
    assert!(!group.is_trivial()); // Has pattern matching

    // Compile the group
    let (ir_func, ir_group) = compile_function_group(group).unwrap();

    // Pattern functions should compile to IRFunctionGroup
    assert!(ir_func.is_none());
    assert!(ir_group.is_some());

    let ir_group = ir_group.unwrap();
    assert_eq!(ir_group.name, "factorial");
    assert_eq!(ir_group.clauses.len(), 2);
}

#[test]
fn test_compile_multiple_arities() {
    let code = r#"
def add(a, b) => a + b
def add(a, b, c) => a + b + c
"#;

    let program = parse_program(code).unwrap();
    let mut symbol_table = SymbolTable::new();
    let groups = resolve_program(program, &mut symbol_table).unwrap();

    assert_eq!(groups.len(), 2);

    // Both should be trivial
    for group in &groups {
        assert!(group.is_trivial());

        let (ir_func, ir_group) = compile_function_group(group).unwrap();
        assert!(ir_func.is_some());
        assert!(ir_group.is_none());
    }
}

#[test]
fn test_compile_mixed_functions() {
    let code = r#"
def identity(x) => x
def factorial(0) => 1
def factorial(n) => n * factorial(n - 1)
"#;

    let program = parse_program(code).unwrap();
    let mut symbol_table = SymbolTable::new();
    let groups = resolve_program(program, &mut symbol_table).unwrap();

    assert_eq!(groups.len(), 2);

    let mut trivial_count = 0;
    let mut pattern_count = 0;

    for group in &groups {
        let (ir_func, ir_group) = compile_function_group(group).unwrap();

        if group.is_trivial() {
            assert!(ir_func.is_some());
            assert!(ir_group.is_none());
            trivial_count += 1;
        } else {
            assert!(ir_func.is_none());
            assert!(ir_group.is_some());
            pattern_count += 1;
        }
    }

    assert_eq!(trivial_count, 1); // identity
    assert_eq!(pattern_count, 1); // factorial
}

#[test]
fn test_end_to_end_workflow() {
    // This demonstrates the complete workflow:
    // Parse → Resolve → Compile

    let code = r#"
def add(a, b) => a + b
def factorial(0) => 1
def factorial(1) => 1
def factorial(n) => n * factorial(n - 1)
"#;

    // Step 1: Parse
    let program = parse_program(code).unwrap();

    // Step 2: Resolve into function groups
    let mut symbol_table = SymbolTable::new();
    let groups = resolve_program(program, &mut symbol_table).unwrap();

    assert_eq!(groups.len(), 2);

    // Step 3: Compile each group
    let mut ir_functions = Vec::new();
    let mut ir_function_groups = Vec::new();

    for group in &groups {
        let (ir_func, ir_group) = compile_function_group(group).unwrap();

        if let Some(f) = ir_func {
            ir_functions.push(f);
        }
        if let Some(g) = ir_group {
            ir_function_groups.push(g);
        }
    }

    // Verify results
    assert_eq!(ir_functions.len(), 1); // add
    assert_eq!(ir_function_groups.len(), 1); // factorial

    assert_eq!(ir_functions[0].name, "add");
    assert_eq!(ir_function_groups[0].name, "factorial");
    assert_eq!(ir_function_groups[0].clauses.len(), 3);
}
