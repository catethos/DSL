use dsl_tui::run_tui;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    run_tui().await
}
