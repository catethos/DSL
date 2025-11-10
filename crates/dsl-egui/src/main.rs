use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("DSL Interactive Environment"),
        ..Default::default()
    };

    eframe::run_native(
        "DSL",
        options,
        Box::new(|cc| Ok(Box::new(dsl_egui::DslApp::new(cc)))),
    )
}
