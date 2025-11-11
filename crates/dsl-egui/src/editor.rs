use crate::syntax::{highlight_code, ColorScheme};
use eframe::egui;

/// Actions that can be triggered from the editor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorAction {
    RunAll,
    LoadFile,
    SaveFile,
}

/// Editor pane for editing DSL code
pub struct EditorPane {
    /// Editor content
    content: String,

    /// Current file path
    file_path: Option<String>,

    /// Whether the content has been modified
    modified: bool,

    /// Color scheme for syntax highlighting
    color_scheme: ColorScheme,

    /// Status message to display
    status_message: Option<(String, f64)>, // (message, timestamp)
}

impl EditorPane {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            file_path: None,
            modified: false,
            color_scheme: ColorScheme::dark(),
            status_message: None,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> Option<EditorAction> {
        let mut action = None;

        // Check if status message should be cleared (after 3 seconds)
        if let Some((_, timestamp)) = self.status_message {
            let now = ui.input(|i| i.time);
            if now - timestamp > 3.0 {
                self.status_message = None;
            }
        }

        ui.vertical(|ui| {
            // Header - split into two rows to prevent overlap
            ui.horizontal(|ui| {
                ui.heading("Editor");
                
                // Add buttons on the right
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(egui::RichText::new("▶ Run All")
                        .color(egui::Color32::from_rgb(80, 250, 123)))
                        .clicked()
                    {
                        action = Some(EditorAction::RunAll);
                    }
                    
                    if ui.button(egui::RichText::new("💾 Save")
                        .color(egui::Color32::from_rgb(255, 184, 108)))
                        .clicked()
                    {
                        action = Some(EditorAction::SaveFile);
                    }
                    
                    if ui.button(egui::RichText::new("📂 Load")
                        .color(egui::Color32::from_rgb(139, 233, 253)))
                        .clicked()
                    {
                        action = Some(EditorAction::LoadFile);
                    }
                });
            });
            
            // File path and status row
            ui.horizontal(|ui| {
                if let Some(path) = &self.file_path {
                    ui.label(egui::RichText::new(path)
                        .family(egui::FontFamily::Monospace)
                        .size(13.0));
                    if self.modified {
                        ui.label(egui::RichText::new("(modified)")
                            .family(egui::FontFamily::Monospace)
                            .size(13.0));
                    }
                } else {
                    ui.label(egui::RichText::new("(no file)")
                        .family(egui::FontFamily::Monospace)
                        .size(13.0));
                }

                // Show status message if present
                if let Some((msg, _)) = &self.status_message {
                    ui.separator();
                    ui.colored_label(egui::Color32::from_rgb(100, 200, 100), msg);
                }
            });

            ui.separator();

            // Editor with syntax highlighting in a scrollable area
            let color_scheme = self.color_scheme.clone();
            let available_height = ui.available_height();

            egui::ScrollArea::vertical()
                .id_salt("editor_scroll_area")
                .max_height(available_height)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let response = ui.add(
                        egui::TextEdit::multiline(&mut self.content)
                            .desired_width(f32::INFINITY)
                            .font(egui::TextStyle::Monospace)
                            .code_editor()
                            .layouter(&mut |ui, text, _wrap_width| {
                                // Create a syntax-highlighted layout job
                                let font_id = egui::FontId::monospace(14.0);
                                let job = highlight_code(text, font_id, &color_scheme);
                                ui.fonts(|f| f.layout_job(job))
                            }),
                    );

                    if response.changed() {
                        self.modified = true;
                    }
                });
        });

        action
    }

    pub fn set_content(&mut self, content: String) {
        self.content = content;
        self.modified = false;
    }

    pub fn get_content(&self) -> &str {
        &self.content
    }

    pub fn save(&mut self) -> Result<(), String> {
        if let Some(path) = &self.file_path {
            std::fs::write(path, &self.content)
                .map_err(|e| format!("Failed to save file: {}", e))?;
            self.modified = false;
            self.set_status("Saved successfully!");
            Ok(())
        } else {
            Err("No file path set".to_string())
        }
    }

    fn set_status(&mut self, message: &str) {
        // Note: We can't get time here, so we'll set it in ui() method
        // For now, use a dummy timestamp
        self.status_message = Some((message.to_string(), 0.0));
    }

    pub fn set_status_with_time(&mut self, message: String, time: f64) {
        self.status_message = Some((message, time));
    }

    pub fn load(&mut self, path: &str) -> Result<(), String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to load file: {}", e))?;
        self.content = content;
        self.file_path = Some(path.to_string());
        self.modified = false;
        Ok(())
    }

    pub fn get_file_path(&self) -> Option<&str> {
        self.file_path.as_deref()
    }

    pub fn set_file_path(&mut self, path: Option<String>) {
        self.file_path = path;
    }

    pub fn clear_modified(&mut self) {
        self.modified = false;
    }
}

impl Default for EditorPane {
    fn default() -> Self {
        Self::new()
    }
}
