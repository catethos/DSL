use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "dsl-compiler")]
#[command(about = "Compiler for DSL programs", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build DSL to native binary
    Build {
        /// Input .dsl file
        input: PathBuf,

        /// Output binary path
        #[arg(short, long)]
        output: PathBuf,

        /// Save IR to file
        #[arg(long)]
        emit_ir: Option<PathBuf>,

        /// Save generated Rust to file
        #[arg(long)]
        emit_rust: Option<PathBuf>,

        /// Generate library instead of executable
        #[arg(long)]
        lib: bool,

        /// Build with --release
        #[arg(long)]
        release: bool,
    },

    /// Check DSL for errors
    Check {
        /// Input .dsl file
        input: PathBuf,
    },

    /// Compile DSL to IR
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
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build {
            input,
            output,
            emit_ir,
            emit_rust,
            lib,
            release,
        } => build(&input, &output, emit_ir.as_deref(), emit_rust.as_deref(), lib, release).await,

        Commands::Check { input } => check(&input).await,

        Commands::Ir {
            input,
            output,
            json,
        } => ir(&input, &output, json).await,
    }
}

async fn build(
    input: &PathBuf,
    output: &PathBuf,
    emit_ir: Option<&std::path::Path>,
    emit_rust: Option<&std::path::Path>,
    lib: bool,
    release: bool,
) -> Result<()> {
    println!("Parsing {}...", input.display());
    let source = std::fs::read_to_string(input)
        .context(format!("Failed to read input file: {}", input.display()))?;

    println!("Compiling to IR...");
    let ir = dsl_core::compile_to_ir(&source)
        .context("Failed to compile to IR")?;

    if let Some(ir_file) = emit_ir {
        println!("Saving IR to {}...", ir_file.display());
        let bytes = ir.to_msgpack()
            .context("Failed to serialize IR")?;
        std::fs::write(ir_file, bytes)
            .context(format!("Failed to write IR file: {}", ir_file.display()))?;
    }

    println!("Generating Rust code...");
    let rust_code = if lib {
        dsl_codegen::generate_library(&ir)?
    } else {
        dsl_codegen::generate_executable(&ir)?
    };

    // Create temp file in the same directory as output for better error messages
    let temp_dir = output.parent().unwrap_or_else(|| std::path::Path::new("."));
    let temp_file = temp_dir.join("dsl_generated_temp.rs");

    std::fs::write(&temp_file, rust_code)
        .context("Failed to write temporary Rust file")?;

    if let Some(rust_file) = emit_rust {
        println!("Saving Rust code to {}...", rust_file.display());
        std::fs::copy(&temp_file, rust_file)
            .context(format!("Failed to write Rust file: {}", rust_file.display()))?;
    }

    println!("Compiling with rustc...");
    let mut cmd = std::process::Command::new("rustc");
    cmd.arg(&temp_file);
    cmd.arg("-o").arg(output);

    if release {
        cmd.arg("-O");
    }

    // Add common flags for better compilation
    cmd.arg("--edition").arg("2021");

    let status = cmd.status()
        .context("Failed to run rustc")?;

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_file);

    if !status.success() {
        return Err(anyhow!("rustc compilation failed"));
    }

    println!("✓ Built successfully: {}", output.display());
    Ok(())
}

async fn check(input: &PathBuf) -> Result<()> {
    println!("Checking {}...", input.display());
    let source = std::fs::read_to_string(input)
        .context(format!("Failed to read input file: {}", input.display()))?;

    // Try to compile to IR
    match dsl_core::compile_to_ir(&source) {
        Ok(_ir) => {
            println!("✓ No errors found");
            Ok(())
        }
        Err(e) => {
            eprintln!("✗ Compilation failed:");
            eprintln!("{:?}", e);
            Err(e)
        }
    }
}

async fn ir(input: &PathBuf, output: &PathBuf, json: bool) -> Result<()> {
    println!("Parsing {}...", input.display());
    let source = std::fs::read_to_string(input)
        .context(format!("Failed to read input file: {}", input.display()))?;

    println!("Compiling to IR...");
    let ir = dsl_core::compile_to_ir(&source)
        .context("Failed to compile to IR")?;

    println!("Saving IR to {}...", output.display());

    if json {
        let json_str = ir.to_json_pretty()
            .context("Failed to serialize IR to JSON")?;
        std::fs::write(output, json_str)
            .context(format!("Failed to write IR file: {}", output.display()))?;
    } else {
        let bytes = ir.to_msgpack()
            .context("Failed to serialize IR to MessagePack")?;
        std::fs::write(output, bytes)
            .context(format!("Failed to write IR file: {}", output.display()))?;
    }

    println!("✓ IR saved successfully");
    Ok(())
}
