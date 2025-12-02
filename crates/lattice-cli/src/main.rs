use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use std::fs;
use std::path::Path;

use lattice::compiler::{CompileResult, Compiler};
use lattice::output::{value_to_output, value_to_json, CellOutput, format_table_as_text};
use lattice::syntax::parser;
use lattice::types::Value;
use lattice::vm::VM;

/// Output format for CLI results
#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable pretty output (default)
    #[default]
    Pretty,
    /// JSON structured output
    Json,
    /// Table format (for SQL results and lists of maps)
    Table,
}

mod repl;

#[derive(Parser)]
#[command(name = "lat")]
#[command(about = "Lattice DSL interpreter and REPL")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Execute a Lattice source file
    Run {
        /// Path to the source file
        file: String,
        /// Enable verbose output for debugging
        #[arg(short, long)]
        verbose: bool,
        /// Output format
        #[arg(short, long, value_enum, default_value_t = OutputFormat::Pretty)]
        format: OutputFormat,
    },
    /// Start an interactive REPL session
    Repl,
    /// Evaluate a Lattice expression
    Eval {
        /// Code to evaluate
        code: String,
        /// Output format
        #[arg(short, long, value_enum, default_value_t = OutputFormat::Pretty)]
        format: OutputFormat,
    },
    /// Check a file for syntax errors without executing
    Check {
        /// Path to the source file
        file: String,
    },
    /// Dump the AST of a source file
    DumpAst {
        /// Path to the source file
        file: String,
    },
    /// Dump the bytecode of a source file
    DumpBytecode {
        /// Path to the source file
        file: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Run { file, verbose, format }) => run_file(&file, verbose, format),
        Some(Commands::Repl) => {
            let mut repl = repl::Repl::new()?;
            repl.run()
        }
        Some(Commands::Eval { code, format }) => eval_code(&code, format),
        Some(Commands::Check { file }) => {
            todo!("Implement check: {}", file)
        }
        Some(Commands::DumpAst { file }) => {
            todo!("Implement dump-ast: {}", file)
        }
        Some(Commands::DumpBytecode { file }) => {
            todo!("Implement dump-bytecode: {}", file)
        }
        None => {
            // Default to REPL when no command is given
            let mut repl = repl::Repl::new()?;
            repl.run()
        }
    }
}

/// Execute a Lattice source file
fn run_file(file_path: &str, verbose: bool, format: OutputFormat) -> Result<()> {
    let path = Path::new(file_path);

    // Read source file
    let source = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", file_path))?;

    if verbose {
        eprintln!("=== Source ===");
        eprintln!("{}", source);
        eprintln!();
    }

    // Parse source to AST
    let program = parser::parse(&source)
        .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

    if verbose {
        eprintln!("=== Parsed {} items ===", program.items.len());
        eprintln!();
    }

    // Compile AST to bytecode
    let compile_result = Compiler::compile(&program)
        .map_err(|e| anyhow::anyhow!("Compile error: {}", e))?;

    if verbose {
        print_compile_info(&compile_result);
    }

    // Create VM and register compiled artifacts
    let mut vm = VM::new();

    // Register types
    for class in compile_result.classes {
        vm.ir_mut().classes.push(class);
    }
    for enum_def in compile_result.enums {
        vm.ir_mut().enums.push(enum_def);
    }

    // Register functions
    for func in compile_result.functions {
        vm.register_function(func);
    }
    for llm_func in compile_result.llm_functions {
        vm.register_llm_function(llm_func);
    }

    // Execute
    let result = vm.run(&compile_result.chunk)
        .map_err(|e| anyhow::anyhow!("Runtime error: {}", e))?;

    // Print result (unless null)
    if !matches!(result, Value::Null) {
        print_formatted(&result, format);
    }

    Ok(())
}

/// Evaluate a code string directly
fn eval_code(code: &str, format: OutputFormat) -> Result<()> {
    // Parse source to AST
    let program = parser::parse(code)
        .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

    // Compile AST to bytecode
    let compile_result = Compiler::compile(&program)
        .map_err(|e| anyhow::anyhow!("Compile error: {}", e))?;

    // Create VM and register compiled artifacts
    let mut vm = VM::new();

    // Register types
    for class in compile_result.classes {
        vm.ir_mut().classes.push(class);
    }
    for enum_def in compile_result.enums {
        vm.ir_mut().enums.push(enum_def);
    }

    // Register functions
    for func in compile_result.functions {
        vm.register_function(func);
    }
    for llm_func in compile_result.llm_functions {
        vm.register_llm_function(llm_func);
    }

    // Execute
    let result = vm.run(&compile_result.chunk)
        .map_err(|e| anyhow::anyhow!("Runtime error: {}", e))?;

    // Print result (unless null)
    if !matches!(result, Value::Null) {
        print_formatted(&result, format);
    }

    Ok(())
}

/// Print a value formatted according to the specified output format
fn print_formatted(value: &Value, format: OutputFormat) {
    match format {
        OutputFormat::Pretty => {
            let output = value_to_output(value);
            match output {
                CellOutput::Text { content } => println!("{}", content),
                CellOutput::Json { content } => {
                    // Pretty print the JSON for human readability
                    match serde_json::to_string_pretty(&content) {
                        Ok(json) => println!("{}", json),
                        Err(e) => eprintln!("JSON formatting error: {}", e),
                    }
                }
                CellOutput::Table { .. } => print!("{}", format_table_as_text(&output)),
                CellOutput::Struct { type_name, fields } => {
                    println!("{} {{", type_name);
                    for (key, val) in &fields {
                        println!("  {}: {}", key, val);
                    }
                    println!("}}");
                }
                CellOutput::Error { message, .. } => eprintln!("Error: {}", message),
                CellOutput::None => {}
            }
        }
        OutputFormat::Json => {
            // For JSON format, convert Value directly to JSON without CellOutput wrapper
            let json_value = value_to_json(value);
            match serde_json::to_string_pretty(&json_value) {
                Ok(json) => println!("{}", json),
                Err(e) => eprintln!("JSON serialization error: {}", e),
            }
        }
        OutputFormat::Table => {
            let output = value_to_output(value);
            match &output {
                CellOutput::Table { .. } => print!("{}", format_table_as_text(&output)),
                _ => {
                    // For non-table values, wrap in single-row table
                    println!("{}", value);
                }
            }
        }
    }
}

/// Print compilation information in verbose mode
fn print_compile_info(result: &CompileResult) {
    eprintln!("=== Compilation Result ===");
    eprintln!("Classes: {}", result.classes.len());
    for class in &result.classes {
        eprintln!("  - {} ({} fields)", class.name, class.fields.len());
    }
    eprintln!("Enums: {}", result.enums.len());
    for enum_def in &result.enums {
        eprintln!("  - {} ({} variants)", enum_def.name, enum_def.values.len());
    }
    eprintln!("Functions: {}", result.functions.len());
    for func in &result.functions {
        eprintln!("  - {}(arity={})", func.name, func.arity);
    }
    eprintln!("LLM Functions: {}", result.llm_functions.len());
    for func in &result.llm_functions {
        eprintln!("  - {}", func.name);
    }
    eprintln!("Bytecode: {} instructions", result.chunk.code.len());
    eprintln!();
}
