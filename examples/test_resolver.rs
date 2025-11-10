use dsl_core::{parse_program, SymbolTable, resolve_program};

fn main() -> Result<(), String> {
    let code = r#"
// Simple function - should become a trivial FunctionGroup
def add(a, b) => a + b

// Overloaded function - multiple clauses with same name/arity
def factorial(0) => 1
def factorial(n) => n * factorial(n - 1)

// Different arity - should be separate FunctionGroup  
def factorial(n, acc) => factorial(n - 1, n * acc)

// Another simple function
def identity(x) => x

// Duplicate name with different arity
def add(x, y, z) => x + y + z
"#;

    println!("Parsing program...");
    let program = parse_program(code)?;
    
    println!("\nParsed {} functions and {} pattern functions", 
        program.functions.len(), 
        program.pattern_functions.len());

    println!("\nResolving into function groups...");
    let mut symbol_table = SymbolTable::new();
    let groups = resolve_program(program, &mut symbol_table)?;

    println!("\nResolved into {} function groups:", groups.len());
    for group in groups {
        println!("  - {}/{} with {} clauses (trivial: {})",
            group.name,
            group.arity,
            group.clauses.len(),
            group.is_trivial()
        );
    }

    Ok(())
}
