use clap::Parser;
use dsl_tui::{run_stdin, run_tui};

#[derive(Parser)]
#[command(name = "dsl")]
#[command(about = "DSL REPL - Interactive shell for the DSL language", long_about = None)]
struct Cli {
    /// Run in non-interactive mode, reading from stdin
    #[arg(long, short = 'c')]
    stdin: bool,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    if cli.stdin {
        // Run in non-interactive mode (reads from stdin)
        run_stdin().await
    } else {
        // Run in interactive TUI mode
        run_tui().await
    }
}
