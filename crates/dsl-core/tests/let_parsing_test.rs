use dsl_core::parser::parse_program;

#[test]
fn test_parse_function_with_let() {
    let source = r#"
def f(x) {
    let y=2
    x + y
}
"#;

    let result = parse_program(source);
    match result {
        Ok(program) => {
            // Check that we have one function
            assert_eq!(program.functions.len(), 1, "Expected 1 function");

            let func = &program.functions[0];
            assert_eq!(func.name, "f");
            assert_eq!(func.params.len(), 1);
            assert_eq!(func.params[0], "x");

            // Print the AST for debugging
            println!("Function execution: {:#?}", func.execution);
        }
        Err(e) => {
            panic!("Parsing failed: {:?}", e);
        }
    }
}
