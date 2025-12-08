use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use std::fs;
use std::path::Path;

use lattice::compiler::{CompileResult, Compiler};
use lattice::output::{value_to_output, value_to_json, CellOutput, format_table_as_text};
use lattice::runtime::{LatticeRuntime, LatticeValue, RuntimeBuilder};
use lattice::syntax::{parser, parse_markdown_llm};
use lattice::types::Value;

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
    /// Convert a markdown LLM file to Lattice source (for preview/debugging)
    Convert {
        /// Path to the markdown file
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
        Some(Commands::Convert { file }) => convert_file(&file),
        None => {
            // Default to REPL when no command is given
            let mut repl = repl::Repl::new()?;
            repl.run()
        }
    }
}

/// Create a LatticeRuntime with default providers
fn create_runtime() -> Result<LatticeRuntime> {
    let built = RuntimeBuilder::new()
        .with_default_providers()
        .map_err(|e| anyhow::anyhow!("Failed to initialize providers: {}", e))?
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build runtime: {}", e))?;
    Ok(LatticeRuntime::from_built(built))
}

/// Execute a Lattice source file
fn run_file(file_path: &str, verbose: bool, format: OutputFormat) -> Result<()> {
    let path = Path::new(file_path);

    if verbose {
        // Read source file for display
        let source = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", file_path))?;
        eprintln!("=== Source ===");
        eprintln!("{}", source);
        eprintln!();

        let program = parser::parse(&source)
            .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;
        eprintln!("=== Parsed {} items ===", program.items.len());
        eprintln!();

        let compile_result = Compiler::compile(&program)
            .map_err(|e| anyhow::anyhow!("Compile error: {}", e))?;
        print_compile_info(&compile_result);
    }

    // Create runtime with default providers and evaluate file (with import resolution)
    let mut runtime = create_runtime()?;
    let result = runtime.eval_file(path)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    // Print result (unless null)
    if !matches!(result, LatticeValue::Null) {
        print_formatted_lattice(&result, format);
    }

    Ok(())
}

/// Evaluate a code string directly
fn eval_code(code: &str, format: OutputFormat) -> Result<()> {
    // Create runtime with default providers and evaluate
    let mut runtime = create_runtime()?;
    let result = runtime.eval(code)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    // Print result (unless null)
    if !matches!(result, LatticeValue::Null) {
        print_formatted_lattice(&result, format);
    }

    Ok(())
}

/// Print a LatticeValue formatted according to the specified output format
fn print_formatted_lattice(value: &LatticeValue, format: OutputFormat) {
    // Convert to internal Value for formatting (reuse existing formatting logic)
    let internal_value = value.to_internal();
    print_formatted(&internal_value, format);
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

/// Convert a markdown LLM file to Lattice source code
fn convert_file(file_path: &str) -> Result<()> {
    let path = Path::new(file_path);

    // Read the markdown file
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", file_path))?;

    // Parse and transpile
    let md_def = parse_markdown_llm(&content)
        .map_err(|e| anyhow::anyhow!("Error parsing markdown file: {}", e))?;

    // Output the transpiled Lattice source
    println!("{}", md_def.to_lattice_source());

    Ok(())
}
