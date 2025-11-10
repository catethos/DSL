use dsl_core::{parse_program, resolve_program, SymbolTable};

#[test]
fn test_resolver_with_simple_functions() {
    let code = r#"
def add(a, b) => a + b
def identity(x) => x
"#;

    let program = parse_program(code).unwrap();
    let mut symbol_table = SymbolTable::new();
    let groups = resolve_program(program, &mut symbol_table).unwrap();

    assert_eq!(groups.len(), 2);

    // Both should be trivial (single clause, simple patterns)
    for group in &groups {
        assert!(group.is_trivial(), "{} should be trivial", group.name);
    }
}

#[test]
fn test_resolver_with_pattern_functions() {
    let code = r#"
def factorial(0) => 1
def factorial(n) => n * factorial(n - 1)
"#;

    let program = parse_program(code).unwrap();
    let mut symbol_table = SymbolTable::new();
    let groups = resolve_program(program, &mut symbol_table).unwrap();

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].name, "factorial");
    assert_eq!(groups[0].arity, 1);
    assert_eq!(groups[0].clauses.len(), 2);
    assert!(!groups[0].is_trivial()); // Not trivial - has pattern matching
}

#[test]
fn test_resolver_with_different_arities() {
    let code = r#"
def add(a, b) => a + b
def add(a, b, c) => a + b + c
"#;

    let program = parse_program(code).unwrap();
    let mut symbol_table = SymbolTable::new();
    let groups = resolve_program(program, &mut symbol_table).unwrap();

    // Should create 2 groups (different arities)
    assert_eq!(groups.len(), 2);

    let add2 = groups.iter().find(|g| g.arity == 2).unwrap();
    let add3 = groups.iter().find(|g| g.arity == 3).unwrap();

    assert_eq!(add2.name, "add");
    assert_eq!(add3.name, "add");
    assert!(add2.is_trivial());
    assert!(add3.is_trivial());
}

#[test]
fn test_resolver_mixed_simple_and_pattern() {
    let code = r#"
def add(a, b) => a + b
def add(x, y) => x + y
def factorial(0) => 1
def factorial(n) => n * factorial(n - 1)
"#;

    let program = parse_program(code).unwrap();
    let mut symbol_table = SymbolTable::new();
    let groups = resolve_program(program, &mut symbol_table).unwrap();

    // Should have 2 groups: add/2 and factorial/1
    assert_eq!(groups.len(), 2);

    let add_group = groups.iter().find(|g| g.name == "add").unwrap();
    let fib_group = groups.iter().find(|g| g.name == "factorial").unwrap();

    // add/2 has 2 clauses (both simple, but duplicate names)
    assert_eq!(add_group.clauses.len(), 2);
    // Not trivial anymore because it has multiple clauses
    assert!(!add_group.is_trivial());

    // factorial/1 has 2 clauses (with pattern matching)
    assert_eq!(fib_group.clauses.len(), 2);
    assert!(!fib_group.is_trivial());
}

#[test]
fn test_symbol_table_incremental() {
    // Simulate REPL-style incremental addition
    let mut symbol_table = SymbolTable::new();

    // First definition
    let code1 = "def add(a, b) => a + b";
    let prog1 = parse_program(code1).unwrap();
    resolve_program(prog1, &mut symbol_table).unwrap();

    assert!(symbol_table.has_function("add", 2));
    assert_eq!(symbol_table.get_group("add", 2).unwrap().clauses.len(), 1);

    // Second definition - different arity
    let code2 = "def add(a, b, c) => a + b + c";
    let prog2 = parse_program(code2).unwrap();
    resolve_program(prog2, &mut symbol_table).unwrap();

    assert!(symbol_table.has_function("add", 2));
    assert!(symbol_table.has_function("add", 3));
    assert_eq!(symbol_table.get_group("add", 2).unwrap().clauses.len(), 1);
    assert_eq!(symbol_table.get_group("add", 3).unwrap().clauses.len(), 1);

    // Third definition - same arity as first (should add clause)
    let code3 = "def add(x, y) => x + y";
    let prog3 = parse_program(code3).unwrap();
    resolve_program(prog3, &mut symbol_table).unwrap();

    assert_eq!(symbol_table.get_group("add", 2).unwrap().clauses.len(), 2);
    assert_eq!(symbol_table.get_group("add", 3).unwrap().clauses.len(), 1);
}
