use dsl_core::compiler::compile_program;

#[test]
fn test_compile_function_with_let() {
    let source = r#"
def f(x) {
    let y=2
    x + y
}
"#;

    let result = compile_program(source);
    match result {
        Ok(ir) => {
            // Check that we have one function
            assert_eq!(ir.functions.len(), 1, "Expected 1 function");

            let func = &ir.functions[0];
            assert_eq!(func.name, "f");
            assert_eq!(func.params.len(), 1);
            assert_eq!(func.params[0], "x");

            // Print the IR for debugging
            println!("Function execution: {:#?}", func.execution);
        }
        Err(e) => {
            panic!("Compilation failed: {:?}", e);
        }
    }
}
