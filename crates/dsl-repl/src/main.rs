use clap::{Parser, Subcommand};
use dsl_tui::{run_stdin, run_tui};

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
