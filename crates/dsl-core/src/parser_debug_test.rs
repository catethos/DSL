#[cfg(test)]
mod tests {
    use crate::parse_program;

    #[test]
    fn test_parse_simple_function() {
        let input = "def add(a, b) {a + b}";
        let result = parse_program(input);
        match &result {
            Ok(program) => {
                println!("Functions: {}", program.functions.len());
                println!("Pattern functions: {}", program.pattern_functions.len());
                for func in &program.functions {
                    println!("  Function: {}", func.name);
                }
            }
            Err(e) => {
                println!("Parse error: {}", e);
            }
        }
        assert!(result.is_ok(), "Should parse function definition");
        let program = result.unwrap();
        assert_eq!(program.functions.len(), 1, "Should have 1 function");
        assert_eq!(program.functions[0].name, "add");
    }

    #[test]
    fn test_parse_function_with_entry() {
        let input = "def add(a, b) {a + b}\nadd(1, 2)";
        let result = parse_program(input);
        match &result {
            Ok(program) => {
                println!("Functions: {}", program.functions.len());
                println!("Entry expr: {:?}", program.entry_expr.is_some());
            }
            Err(e) => {
                println!("Parse error: {}", e);
            }
        }
        assert!(result.is_ok(), "Should parse function + entry");
        let program = result.unwrap();
        assert_eq!(program.functions.len(), 1);
        assert!(program.entry_expr.is_some());
    }
}
