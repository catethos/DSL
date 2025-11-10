use dsl_core::{compile_program_to_ir, parse_program};
use dsl_interpreter::{TraceConfig, TracingInterpreter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== DSL Tracing Interpreter Demo ===\n");

    // Example 1: Simple expression with default tracing
    println!("Example 1: Simple Expression");
    println!("------------------------------");

    let source = "2 + 3 * 4";
    let ast = parse_program(source)?;
    let ir = compile_program_to_ir(&ast)?;

    let mut tracer = TracingInterpreter::from_ir(&ir)?;
    let result = tracer
        .eval(&ir.entry_expr)
        .await
        .map_err(|e| format!("Execution error: {}", e))?;

    println!("Expression: {}", source);
    println!("Result: {:?}", result);
    println!("\n{}", tracer.trace.summary());

    // Example 2: Nested computations
    println!("\n\nExample 2: Nested Computations");
    println!("------------------------------");

    let nested_source = "(2 + 3) * (4 + 5)";

    let ast = parse_program(nested_source)?;
    let ir = compile_program_to_ir(&ast)?;

    let mut tracer = TracingInterpreter::from_ir(&ir)?;
    let result = tracer
        .eval(&ir.entry_expr)
        .await
        .map_err(|e| format!("Execution error: {}", e))?;

    println!("Expression: {}", nested_source);
    println!("Result: {:?}", result);
    println!("\n{}", tracer.trace.summary());

    // Show all events
    println!("\nDetailed trace:");
    for event in &tracer.trace.events {
        println!(
            "  Step {}: {} - {}μs - {}",
            event.step, event.node_type, event.duration_micros, event.description
        );
    }

    // Example 3: Filtered tracing (only BinaryOp)
    println!("\n\nExample 3: Filtered Tracing (BinaryOp Only)");
    println!("---------------------------------------------");

    let complex_expr = "1 + 2 + 3 + 4 + 5";
    let ast = parse_program(complex_expr)?;
    let ir = compile_program_to_ir(&ast)?;

    let config = TraceConfig {
        max_events: 1000,
        capture_variables: false,
        node_filter: vec!["BinaryOp".to_string()],
        min_duration_micros: 0,
        recursive: true,
    };

    let mut tracer = TracingInterpreter::from_ir_with_config(&ir, config)?;
    let result = tracer
        .eval(&ir.entry_expr)
        .await
        .map_err(|e| format!("Execution error: {}", e))?;

    println!("Expression: {}", complex_expr);
    println!("Result: {:?}", result);
    println!("Total events captured: {}", tracer.trace.events.len());
    println!("All events are BinaryOp nodes:");
    for event in &tracer.trace.events {
        println!("  Step {}: {}", event.step, event.description);
    }

    // Example 4: Export to JSON
    println!("\n\nExample 4: Export Trace to JSON");
    println!("--------------------------------");

    tracer.export_trace("/tmp/dsl_trace.json")?;
    println!("Trace exported to: /tmp/dsl_trace.json");

    // Show sample of JSON output
    let json = tracer.trace.to_json()?;
    let lines: Vec<&str> = json.lines().take(20).collect();
    println!("\nFirst 20 lines of JSON trace:");
    for line in lines {
        println!("{}", line);
    }

    // Example 5: Complex expression with sequential operations
    println!("\n\nExample 5: Sequential Operations");
    println!("----------------------------------");

    let sequential_source = "[1, 2, 3, 4, 5]";

    let ast = parse_program(sequential_source)?;
    let ir = compile_program_to_ir(&ast)?;

    let mut tracer = TracingInterpreter::from_ir(&ir)?;
    let result = tracer
        .eval(&ir.entry_expr)
        .await
        .map_err(|e| format!("Execution error: {}", e))?;

    println!("Expression: {}", sequential_source);
    println!("Result: {:?}", result);
    println!("\nExecution trace:");
    for (i, event) in tracer.trace.events.iter().enumerate() {
        println!(
            "  {:2}. [{}μs] {:15} - {} (depth: {})",
            i + 1,
            event.duration_micros,
            event.node_type,
            event.description,
            event.depth
        );
    }

    // Example 6: Performance analysis
    println!("\n\nExample 6: Performance Analysis");
    println!("--------------------------------");

    let stats = tracer.trace.stats_by_node_type();
    println!("\nPerformance breakdown by node type:");
    for (node_type, count, duration) in stats {
        println!("  {:20} {:4} calls  {:?}", node_type, count, duration);
    }

    println!("\n=== Demo Complete ===");

    Ok(())
}
