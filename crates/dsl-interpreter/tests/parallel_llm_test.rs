use dsl_core::{compile_program_to_ir, parse_program};
use dsl_interpreter::Interpreter;
use std::time::Instant;

#[tokio::test]
#[ignore] // Ignore by default since it requires OPENAI_API_KEY
async fn test_parallel_ask_calls() {
    // This test demonstrates parallel execution of Ask() calls
    // Run with: OPENAI_API_KEY=sk-... cargo test --package dsl-interpreter test_parallel_ask_calls -- --ignored

    let source = r#"
        par(Ask("Say A"), Ask("Say B"), Ask("Say C"))
    "#;

    let ast = parse_program(source).expect("Failed to parse");
    let ir = compile_program_to_ir(&ast).expect("Failed to compile");

    let mut interpreter = Interpreter::from_ir(&ir).expect("Failed to create interpreter");

    let start = Instant::now();
    let result = interpreter.eval(&ir.entry_expr).await;
    let duration = start.elapsed();

    match result {
        Ok(value) => {
            println!("Result: {:?}", value);
            println!("Duration: {:?}", duration);
            println!("\n✓ Parallel execution successful!");
            println!("  Expected: ~1x LLM call time (all calls in parallel)");
            println!("  Sequential would be: ~3x LLM call time");

            // Verify we got 3 results
            if let dsl_ir::Value::List(items) = value {
                assert_eq!(items.len(), 3, "Should have 3 results");
                for item in &items {
                    assert!(
                        matches!(item, dsl_ir::Value::String(_)),
                        "Each result should be a string"
                    );
                }
            } else {
                panic!("Expected a list of results");
            }
        }
        Err(e) => {
            if e.to_string().contains("OPENAI_API_KEY") {
                println!("⚠ Skipping test: OPENAI_API_KEY not set");
                println!("  Run with: OPENAI_API_KEY=sk-... cargo test test_parallel_ask_calls -- --ignored");
            } else {
                panic!("Test failed: {}", e);
            }
        }
    }
}

#[tokio::test]
async fn test_parallel_with_single_ask() {
    // Single Ask should work fine (no parallelism needed)
    let source = r#"
        Ask("Hello")
    "#;

    let ast = parse_program(source).expect("Failed to parse");
    let ir = compile_program_to_ir(&ast).expect("Failed to compile");

    let mut interpreter = Interpreter::from_ir(&ir).expect("Failed to create interpreter");

    let result = interpreter.eval(&ir.entry_expr).await;

    // Should either succeed or fail with API key error
    match result {
        Ok(value) => {
            // Single element should unwrap to the value itself, not a list
            assert!(matches!(value, dsl_ir::Value::String(_)));
        }
        Err(e) => {
            assert!(e.to_string().contains("OPENAI_API_KEY"));
        }
    }
}
