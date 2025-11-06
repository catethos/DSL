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

    /// Clean the build cache
    Clean,
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

        Commands::Clean => clean().await,
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

    if let Some(rust_file) = emit_rust {
        println!("Saving Rust code to {}...", rust_file.display());
        std::fs::write(rust_file, &rust_code)
            .context(format!("Failed to write Rust file: {}", rust_file.display()))?;
    }

    // Create a persistent cache directory for faster rebuilds
    println!("Creating temporary Cargo project...");
    let cache_dir = std::env::temp_dir().join("dsl_compiler_cache");
    std::fs::create_dir_all(&cache_dir)
        .context("Failed to create cache directory")?;

    let temp_dir = cache_dir.join("build");
    std::fs::create_dir_all(&temp_dir)
        .context("Failed to create build directory")?;

    // Write Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "dsl_generated"
version = "0.1.0"
edition = "2021"

[dependencies]
dsl-runtime = {{ path = "{}/crates/dsl-runtime" }}
dsl-interpreter = {{ path = "{}/crates/dsl-interpreter" }}
dsl-ir = {{ path = "{}/crates/dsl-ir" }}
tokio = {{ version = "1", features = ["full"] }}
anyhow = "1"
serde_json = "1"
"#,
        std::env::current_dir()
            .context("Failed to get current directory")?
            .display(),
        std::env::current_dir()
            .context("Failed to get current directory")?
            .display(),
        std::env::current_dir()
            .context("Failed to get current directory")?
            .display()
    );

    std::fs::write(temp_dir.join("Cargo.toml"), cargo_toml)
        .context("Failed to write Cargo.toml")?;

    // Create src directory
    let src_dir = temp_dir.join("src");
    std::fs::create_dir_all(&src_dir)
        .context("Failed to create src directory")?;

    // Write main.rs
    std::fs::write(src_dir.join("main.rs"), rust_code)
        .context("Failed to write main.rs")?;

    // Build with cargo
    println!("Compiling with cargo...");
    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("build");
    cmd.arg("--manifest-path");
    cmd.arg(temp_dir.join("Cargo.toml"));

    if release {
        cmd.arg("--release");
    }

    let status = cmd.status()
        .context("Failed to run cargo build")?;

    if !status.success() {
        // Keep temp dir for debugging
        eprintln!("Build failed. Temp project at: {}", temp_dir.display());
        return Err(anyhow!("cargo build failed"));
    }

    // Copy the binary to output location
    let binary_name = "dsl_generated";
    let binary_path = if release {
        temp_dir.join("target/release").join(binary_name)
    } else {
        temp_dir.join("target/debug").join(binary_name)
    };

    std::fs::copy(&binary_path, output)
        .context("Failed to copy binary to output location")?;

    // Keep temp directory for faster incremental builds
    // The cache is reused across compilations, significantly speeding up subsequent builds

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

async fn clean() -> Result<()> {
    let cache_dir = std::env::temp_dir().join("dsl_compiler_cache");

    if !cache_dir.exists() {
        println!("No cache to clean");
        return Ok(());
    }

    println!("Cleaning build cache at {}...", cache_dir.display());

    match std::fs::remove_dir_all(&cache_dir) {
        Ok(_) => {
            println!("✓ Cache cleaned successfully");
            Ok(())
        }
        Err(e) => {
            eprintln!("✗ Failed to clean cache: {}", e);
            Err(e.into())
        }
    }
}
