use clap::{Parser, Subcommand};
use dsl_tui::{run_stdin, run_tui};
use std::path::PathBuf;

mod config;
mod update;

const VERSION: &str = env!("DSL_VERSION");

#[derive(Parser)]
#[command(name = "dsl")]
#[command(about = "DSL REPL - Interactive shell for the DSL language", long_about = None)]
#[command(version = VERSION)]
struct Cli {
    /// Run in non-interactive mode, reading from stdin
    #[arg(long, short = 'c')]
    stdin: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Update to the latest version
    Update {
        /// Check for updates without installing
        #[arg(long)]
        check: bool,
    },

    /// Check DSL file for errors
    Check {
        /// Input .dsl file
        input: PathBuf,
    },

    /// Compile DSL to IR (Intermediate Representation)
    Ir {
        /// Input .dsl file
        input: PathBuf,

        /// Output IR file
        #[arg(short, long)]
        output: PathBuf,

        /// Output JSON instead of MessagePack
        #[arg(long)]
        json: bool,
    },

    /// Run a DSL file directly
    Run {
        /// Input .dsl file
        input: PathBuf,

        /// Enable execution tracing
        #[arg(long)]
        trace: bool,

        /// Export trace to JSON file
        #[arg(long, requires = "trace")]
        trace_output: Option<PathBuf>,

        /// Show detailed trace events
        #[arg(long, requires = "trace")]
        trace_verbose: bool,

        /// Filter trace by node types (comma-separated, e.g. "FunctionCall,BinaryOp")
        #[arg(long, requires = "trace", value_delimiter = ',')]
        trace_filter: Vec<String>,

        /// Only show events slower than N microseconds
        #[arg(long, requires = "trace")]
        trace_min_duration: Option<u128>,
    },
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    // Handle subcommands first
    if let Some(command) = cli.command {
        match command {
            Commands::Update { check } => {
                if check {
                    match update::check_for_update(VERSION).await {
                        Ok(Some(new_version)) => {
                            println!("New version available: {} -> {}", VERSION, new_version);
                            println!("Run 'dsl update' to install the update.");
                            std::process::exit(0);
                        }
                        Ok(None) => {
                            println!("You are running the latest version: {}", VERSION);
                            std::process::exit(0);
                        }
                        Err(e) => {
                            eprintln!("Error checking for updates: {}", e);
                            std::process::exit(1);
                        }
                    }
                } else {
                    match update::self_update(VERSION).await {
                        Ok(_) => std::process::exit(0),
                        Err(e) => {
                            eprintln!("Update failed: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
            }

            Commands::Check { input } => match cmd_check(&input).await {
                Ok(_) => std::process::exit(0),
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            },

            Commands::Ir {
                input,
                output,
                json,
            } => match cmd_ir(&input, &output, json).await {
                Ok(_) => std::process::exit(0),
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            },

            Commands::Run {
                input,
                trace,
                trace_output,
                trace_verbose,
                trace_filter,
                trace_min_duration,
            } => {
                match cmd_run(
                    &input,
                    trace,
                    trace_output,
                    trace_verbose,
                    trace_filter,
                    trace_min_duration,
                )
                .await
                {
                    Ok(_) => std::process::exit(0),
                    Err(e) => {
                        eprintln!("{}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
    }

    // Check for updates periodically (non-blocking, only in interactive mode)
    if !cli.stdin && config::should_check_for_updates() {
        tokio::spawn(async {
            if let Ok(Some(new_version)) = update::check_for_update(VERSION).await {
                eprintln!("\n🔔 New version available: {} -> {}", VERSION, new_version);
                eprintln!("   Run 'dsl update' to install the update.\n");
            }
            config::record_update_check();
        });
    }

    if cli.stdin {
        // Run in non-interactive mode (reads from stdin)
        run_stdin().await
    } else {
        // Run in interactive TUI mode
        run_tui().await
    }
}

// ===== Command Implementations =====

async fn cmd_check(input: &PathBuf) -> Result<(), String> {
    println!("Checking {}...", input.display());
    let source =
        std::fs::read_to_string(input).map_err(|e| format!("Failed to read input file: {}", e))?;

    // Try to compile to IR (handles both programs and expressions)
    match dsl_core::compile_to_ir(&source) {
        Ok(_ir) => {
            println!("✓ No errors found");
            Ok(())
        }
        Err(e) => {
            eprintln!("✗ Compilation failed:");
            Err(format!("{:?}", e))
        }
    }
}

async fn cmd_ir(input: &PathBuf, output: &PathBuf, json: bool) -> Result<(), String> {
    println!("Parsing {}...", input.display());
    let source =
        std::fs::read_to_string(input).map_err(|e| format!("Failed to read input file: {}", e))?;

    println!("Compiling to IR...");
    let ir = dsl_core::compile_to_ir(&source)
        .map_err(|e| format!("Failed to compile to IR: {:?}", e))?;

    println!("Saving IR to {}...", output.display());

    if json {
        let json_str = ir
            .to_json_pretty()
            .map_err(|e| format!("Failed to serialize IR to JSON: {:?}", e))?;
        std::fs::write(output, json_str).map_err(|e| format!("Failed to write IR file: {}", e))?;
    } else {
        let bytes = ir
            .to_msgpack()
            .map_err(|e| format!("Failed to serialize IR to MessagePack: {:?}", e))?;
        std::fs::write(output, bytes).map_err(|e| format!("Failed to write IR file: {}", e))?;
    }

    println!("✓ IR saved successfully");
    Ok(())
}

async fn cmd_run(
    input: &PathBuf,
    trace: bool,
    trace_output: Option<PathBuf>,
    trace_verbose: bool,
    trace_filter: Vec<String>,
    trace_min_duration: Option<u128>,
) -> Result<(), String> {
    println!("Running {}...", input.display());
    let source =
        std::fs::read_to_string(input).map_err(|e| format!("Failed to read input file: {}", e))?;

    println!("Compiling to IR...");
    let ir = dsl_core::compile_to_ir(&source)
        .map_err(|e| format!("Failed to compile to IR: {:?}", e))?;

    println!("Executing...");

    if trace {
        // Use tracing interpreter
        let config = dsl_interpreter::TraceConfig {
            max_events: 100000,
            capture_variables: false,
            node_filter: trace_filter,
            min_duration_micros: trace_min_duration.unwrap_or(0),
            recursive: true, // Enable recursive tracing
        };

        let mut tracer = dsl_interpreter::TracingInterpreter::from_ir_with_config(&ir, config)
            .map_err(|e| format!("Failed to create tracing interpreter: {:?}", e))?;

        let result = tracer
            .eval(&ir.entry_expr)
            .await
            .map_err(|e| format!("Runtime error: {}", e))?;

        // Print the result
        println!("\nResult:");
        println!("{}", result.display());

        // Print trace summary
        println!("\n{}", tracer.trace.summary());

        // Print detailed trace if requested
        if trace_verbose {
            println!("\nDetailed Trace:");
            println!(
                "{:<6} {:<15} {:<12} {:<8} {:<30} Description",
                "Step", "Node Type", "Duration", "Depth", "Inputs"
            );
            println!("{}", "-".repeat(120));
            for event in &tracer.trace.events {
                // Format inputs
                let inputs_str = if event.inputs.is_empty() {
                    "".to_string()
                } else {
                    event
                        .inputs
                        .iter()
                        .map(|v| v.display().to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                };
                let inputs_display = if inputs_str.len() > 28 {
                    format!("{}...", &inputs_str[..25])
                } else {
                    inputs_str
                };

                println!(
                    "{:<6} {:<15} {:>10}μs {:<8} {:<30} {}",
                    event.step,
                    event.node_type,
                    event.duration_micros,
                    event.depth,
                    inputs_display,
                    event.description
                );
            }
        }

        // Export trace if requested
        if let Some(output_path) = trace_output {
            tracer
                .export_trace(output_path.to_str().unwrap())
                .map_err(|e| format!("Failed to export trace: {}", e))?;
            println!("\n✓ Trace exported to {}", output_path.display());
        }
    } else {
        // Use normal interpreter
        let mut interpreter = dsl_interpreter::Interpreter::from_ir(&ir)
            .map_err(|e| format!("Failed to create interpreter: {:?}", e))?;

        let result = interpreter
            .eval(&ir.entry_expr)
            .await
            .map_err(|e| format!("Runtime error: {}", e))?;

        // Print the result
        println!("\n{}", result.display());
    }

    Ok(())
}
