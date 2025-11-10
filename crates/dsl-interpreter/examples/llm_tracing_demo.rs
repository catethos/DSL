use dsl_core::{compile_program_to_ir, parse_program};
use dsl_interpreter::TracingInterpreter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== LLM Tracing Demo ===\n");
    println!("This demo shows how to trace LLM function calls,");
    println!("capturing both the prompts sent and responses received.\n");

    // Example: Tracing builtin LLM functions
    println!("Example 1: Builtin Ask() Function");
    println!("----------------------------------");

    let source = r#"Ask("What is the capital of France?")"#;
    let ast = parse_program(source)?;
    let ir = compile_program_to_ir(&ast)?;

    let mut tracer = TracingInterpreter::from_ir(&ir)?;

    match tracer.eval(&ir.entry_expr).await {
        Ok(result) => {
            println!("Result: {:?}\n", result);

            // Show trace events with LLM data
            println!("Trace Events:");
            for event in &tracer.trace.events {
                println!("\nStep {}: {}", event.step, event.node_type);
                println!("  Description: {}", event.description);
                println!("  Duration: {}μs", event.duration_micros);

                if let Some(prompt) = &event.llm_prompt {
                    println!("  LLM Prompt:");
                    println!(
                        "    {}",
                        prompt.lines().take(3).collect::<Vec<_>>().join("\n    ")
                    );
                    if prompt.lines().count() > 3 {
                        println!("    ... ({} more lines)", prompt.lines().count() - 3);
                    }
                }

                if let Some(response) = &event.llm_response {
                    println!("  LLM Response:");
                    println!(
                        "    {}",
                        response.lines().take(3).collect::<Vec<_>>().join("\n    ")
                    );
                    if response.lines().count() > 3 {
                        println!("    ... ({} more lines)", response.lines().count() - 3);
                    }
                }
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    // Example 2: User-defined LLM function
    println!("\n\nExample 2: User-Defined LLM Function");
    println!("-------------------------------------");

    let source = r#"
function summarize(text: string) -> string {
    prompt: "Summarize the following text in one sentence:\n\n${text}"
    model: "gpt-3.5-turbo"
}

summarize("Rust is a systems programming language focused on safety, speed, and concurrency.")
"#;

    let ast = parse_program(source)?;
    let ir = compile_program_to_ir(&ast)?;

    let mut tracer = TracingInterpreter::from_ir(&ir)?;

    match tracer.eval(&ir.entry_expr).await {
        Ok(result) => {
            println!("Result: {:?}\n", result);

            // Export trace to JSON for analysis
            tracer.export_trace("/tmp/llm_trace.json")?;
            println!("Full trace exported to: /tmp/llm_trace.json");

            // Show summary
            println!("\n{}", tracer.trace.summary());

            // Count LLM calls
            let llm_calls = tracer
                .trace
                .events
                .iter()
                .filter(|e| e.llm_prompt.is_some())
                .count();
            println!("\nTotal LLM calls traced: {}", llm_calls);
        }
        Err(e) => println!("Error: {}", e),
    }

    // Example 3: Analyzing LLM performance
    println!("\n\nExample 3: LLM Performance Analysis");
    println!("------------------------------------");

    let source = r#"
Ask("Hello") |> Ask("Respond to: ${_}")
"#;

    let ast = parse_program(source)?;
    let ir = compile_program_to_ir(&ast)?;

    let mut tracer = TracingInterpreter::from_ir(&ir)?;

    match tracer.eval(&ir.entry_expr).await {
        Ok(_) => {
            println!("Analyzing LLM call performance:\n");

            for (i, event) in tracer
                .trace
                .events
                .iter()
                .filter(|e| e.llm_prompt.is_some())
                .enumerate()
            {
                println!("LLM Call #{}:", i + 1);
                println!("  Duration: {}μs", event.duration_micros);
                if let Some(prompt) = &event.llm_prompt {
                    println!("  Prompt length: {} chars", prompt.len());
                }
                if let Some(response) = &event.llm_response {
                    println!("  Response length: {} chars", response.len());
                }
                println!();
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\n=== Demo Complete ===");
    println!("\nNote: This demo requires OPENAI_API_KEY environment variable to be set.");
    println!("Set it with: export OPENAI_API_KEY=sk-...");

    Ok(())
}
