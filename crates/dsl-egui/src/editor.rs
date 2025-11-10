use crate::formatter::{format_document, FormatterConfig};
use crate::syntax::{highlight_code, ColorScheme};
use eframe::egui;

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

    /// Formatter configuration
    formatter_config: FormatterConfig,

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
            formatter_config: FormatterConfig::default(),
            status_message: None,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        // Check if status message should be cleared (after 3 seconds)
        if let Some((_, timestamp)) = self.status_message {
            let now = ui.input(|i| i.time);
            if now - timestamp > 3.0 {
                self.status_message = None;
            }
        }

        // Check for formatting keyboard shortcuts BEFORE any widgets
        // This ensures we capture the event before TextEdit consumes it
        let format_requested = ui.ctx().input_mut(|i| {
            // Check for Cmd+Shift+F (Mac) or Ctrl+Shift+F (Windows/Linux)
            if i.consume_key(egui::Modifiers::COMMAND | egui::Modifiers::SHIFT, egui::Key::F) {
                return true;
            }
            false
        });

        if format_requested {
            self.format_document();
            let now = ui.input(|i| i.time);
            self.set_status_with_time("Document formatted".to_string(), now);
        }

        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.heading("Editor");
                ui.separator();
                if let Some(path) = &self.file_path {
                    ui.label(egui::RichText::new(path)
                        .family(egui::FontFamily::Monospace));
                    if self.modified {
                        ui.label(egui::RichText::new("(modified)")
                            .family(egui::FontFamily::Monospace));
                    }
                } else {
                    ui.label(egui::RichText::new("(no file)")
                        .family(egui::FontFamily::Monospace));
                }

                ui.separator();

                // Format button with keyboard shortcut hint
                if ui.button("Format (⌘⇧F)").clicked() {
                    self.format_document();
                    let now = ui.input(|i| i.time);
                    self.set_status_with_time("Document formatted".to_string(), now);
                }

                // Show status message if present
                if let Some((msg, _)) = &self.status_message {
                    ui.separator();
                    ui.colored_label(egui::Color32::from_rgb(100, 200, 100), msg);
                }
            });

            ui.separator();

            // Editor with syntax highlighting
            let color_scheme = self.color_scheme.clone();
            let response = ui.add_sized(
                egui::vec2(ui.available_width(), ui.available_height()),
                egui::TextEdit::multiline(&mut self.content)
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

    /// Format the entire document
    pub fn format_document(&mut self) {
        let formatted = format_document(&self.content, &self.formatter_config);
        if formatted != self.content {
            self.content = formatted;
            self.modified = true;
            self.set_status("Document formatted");
        }
    }

    /// Get formatter config (for customization)
    pub fn formatter_config_mut(&mut self) -> &mut FormatterConfig {
        &mut self.formatter_config
    }
}

impl Default for EditorPane {
    fn default() -> Self {
        Self::new()
    }
}
