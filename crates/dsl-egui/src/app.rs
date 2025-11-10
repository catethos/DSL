use eframe::egui;

use crate::animations::AnimationType;
use crate::editor::EditorPane;
use crate::repl::ReplPane;

/// The main application state
pub struct DslApp {
    /// REPL pane
    repl: ReplPane,

    /// Editor pane
    editor: EditorPane,

    /// Which pane is currently focused
    active_pane: ActivePane,

    /// Horizontal split ratio (0.0 = all left, 1.0 = all right)
    split_ratio: f32,

    /// Recent files (up to 10)
    recent_files: Vec<String>,

    /// Status message (for error/success feedback)
    status_message: Option<(String, f64, StatusKind)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StatusKind {
    Success,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivePane {
    Repl,
    Editor,
}

impl DslApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Configure fonts to be larger
        let mut style = (*cc.egui_ctx.style()).clone();

        // Increase all text sizes by ~40%
        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(16.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(16.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::new(22.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Monospace,
            egui::FontId::new(16.0, egui::FontFamily::Monospace),
        );
        style.text_styles.insert(
            egui::TextStyle::Small,
            egui::FontId::new(13.0, egui::FontFamily::Proportional),
        );

        cc.egui_ctx.set_style(style);

        Self {
            repl: ReplPane::new(),
            editor: EditorPane::new(),
            active_pane: ActivePane::Repl,
            split_ratio: 0.5, // 50/50 split by default
            recent_files: Vec::new(),
            status_message: None,
        }
    }
}

impl eframe::App for DslApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle global keyboard shortcuts first (before any UI elements consume the input)
        self.handle_global_shortcuts(ctx);

        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open...").clicked() {
                        self.open_file_dialog(ctx.clone());
                        ui.close_menu();
                    }
                    if ui.button("Save").clicked() {
                        self.save_file(ctx.clone());
                        ui.close_menu();
                    }
                    if ui.button("Save As...").clicked() {
                        self.save_file_as_dialog(ctx.clone());
                        ui.close_menu();
                    }
                    ui.separator();

                    // Recent files submenu
                    ui.menu_button("Recent Files", |ui| {
                        if self.recent_files.is_empty() {
                            ui.label("(no recent files)");
                        } else {
                            for path in self.recent_files.clone() {
                                // Extract filename for display
                                let display_name = std::path::Path::new(&path)
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or(&path);

                                if ui.button(display_name).clicked() {
                                    self.load_file(&path, ctx.clone());
                                    ui.close_menu();
                                }
                            }
                        }
                    });

                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.button("Copy").clicked() {
                        // TODO: Copy
                        ui.close_menu();
                    }
                    if ui.button("Paste").clicked() {
                        // TODO: Paste
                        ui.close_menu();
                    }
                });

                ui.menu_button("View", |ui| {
                    if ui.button("Reset Layout").clicked() {
                        self.split_ratio = 0.5;
                        ui.close_menu();
                    }
                });

                ui.menu_button("Animation", |ui| {
                    let current_animation = self.repl.animation_type();
                    for animation_type in AnimationType::all() {
                        let is_selected = current_animation == *animation_type;
                        if ui
                            .selectable_label(is_selected, animation_type.name())
                            .clicked()
                        {
                            self.repl.set_animation_type(*animation_type);
                            ui.close_menu();
                        }
                    }
                });
            });
        });

        // Bottom status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            // Check if status message should be cleared (after 3 seconds)
            let now = ui.input(|i| i.time);
            if let Some((_, timestamp, _)) = self.status_message {
                if now - timestamp > 3.0 {
                    self.status_message = None;
                }
            }

            ui.horizontal(|ui| {
                // Show status message if present
                if let Some((msg, _, kind)) = &self.status_message {
                    let color = match kind {
                        StatusKind::Success => egui::Color32::from_rgb(100, 200, 100),
                        StatusKind::Error => egui::Color32::from_rgb(255, 100, 100),
                    };
                    ui.colored_label(color, msg);
                    ui.separator();
                }

                ui.label(egui::RichText::new(format!("Active: {:?}", self.active_pane))
                    .family(egui::FontFamily::Monospace));
                ui.separator();
                ui.label(egui::RichText::new("Shift+Tab: Switch pane")
                    .family(egui::FontFamily::Monospace));
                ui.separator();
                ui.label(egui::RichText::new("Ctrl+E: Send to REPL")
                    .family(egui::FontFamily::Monospace));
                ui.separator();
                ui.label(egui::RichText::new("Ctrl+R: Run all")
                    .family(egui::FontFamily::Monospace));
                ui.separator();
                ui.label(egui::RichText::new("Ctrl+S: Save")
                    .family(egui::FontFamily::Monospace));
            });
        });

        // Main content area with split panes
        egui::CentralPanel::default().show(ctx, |ui| {
            let available_width = ui.available_width();
            let available_height = ui.available_height();
            let left_width = available_width * self.split_ratio;

            ui.horizontal(|ui| {
                // Left pane - REPL
                let repl_response = ui.allocate_ui_with_layout(
                    egui::vec2(left_width - 2.0, available_height),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        egui::Frame::default()
                            .fill(if self.active_pane == ActivePane::Repl {
                                ui.visuals().extreme_bg_color
                            } else {
                                ui.visuals().window_fill
                            })
                            .stroke(egui::Stroke::new(
                                1.0,
                                if self.active_pane == ActivePane::Repl {
                                    ui.visuals().selection.bg_fill
                                } else {
                                    ui.visuals().window_stroke.color
                                },
                            ))
                            .inner_margin(8.0)
                            .show(ui, |ui| {
                                ui.set_min_height(available_height - 16.0); // Account for margins
                                self.repl.ui(ui);
                            });
                    },
                );

                // Check if REPL pane area was clicked (detect click on the background)
                let repl_rect = repl_response.response.rect;
                if ui.ctx().input(|i| {
                    i.pointer.primary_clicked()
                        && i.pointer
                            .interact_pos()
                            .is_some_and(|pos| repl_rect.contains(pos))
                }) {
                    self.active_pane = ActivePane::Repl;
                }

                // Draggable separator
                let separator_response = ui.allocate_rect(
                    egui::Rect::from_min_size(
                        repl_response.response.rect.right_top(),
                        egui::vec2(6.0, available_height),
                    ),
                    egui::Sense::drag(),
                );

                if separator_response.dragged() {
                    // Update split ratio based on drag delta
                    let delta_ratio = separator_response.drag_delta().x / available_width;
                    self.split_ratio = (self.split_ratio + delta_ratio).clamp(0.2, 0.8);
                }

                // Draw separator with hover effect
                let separator_color = if separator_response.hovered() {
                    ui.visuals().selection.bg_fill
                } else {
                    ui.visuals().window_stroke.color.gamma_multiply(0.5)
                };

                ui.painter()
                    .rect_filled(separator_response.rect, 1.0, separator_color);

                // Show resize cursor on hover
                if separator_response.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                }

                // Right pane - Editor
                let editor_response = ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), available_height),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        egui::Frame::default()
                            .fill(if self.active_pane == ActivePane::Editor {
                                ui.visuals().extreme_bg_color
                            } else {
                                ui.visuals().window_fill
                            })
                            .stroke(egui::Stroke::new(
                                1.0,
                                if self.active_pane == ActivePane::Editor {
                                    ui.visuals().selection.bg_fill
                                } else {
                                    ui.visuals().window_stroke.color
                                },
                            ))
                            .inner_margin(8.0)
                            .show(ui, |ui| {
                                ui.set_min_height(available_height - 16.0); // Account for margins
                                self.editor.ui(ui);
                            });
                    },
                );

                // Check if Editor pane area was clicked (detect click on the background)
                let editor_rect = editor_response.response.rect;
                if ui.ctx().input(|i| {
                    i.pointer.primary_clicked()
                        && i.pointer
                            .interact_pos()
                            .is_some_and(|pos| editor_rect.contains(pos))
                }) {
                    self.active_pane = ActivePane::Editor;
                }
            });
        });
    }
}

impl DslApp {
    fn handle_global_shortcuts(&mut self, ctx: &egui::Context) {
        // Check for keyboard shortcuts
        // Note: We use input_mut to consume events so widgets don't see them

        // Tab: Autocomplete in REPL (must be first to consume before Shift+Tab check)
        // Tab handling is now done inside REPL's render() method
        // (removed from here to fix autocomplete cursor positioning)

        // Shift+Tab: Switch panes
        if ctx.input_mut(|i| {
            if i.key_pressed(egui::Key::Tab) && i.modifiers.shift {
                i.consume_key(egui::Modifiers::SHIFT, egui::Key::Tab);
                true
            } else {
                false
            }
        }) {
            self.active_pane = match self.active_pane {
                ActivePane::Repl => ActivePane::Editor,
                ActivePane::Editor => ActivePane::Repl,
            };
            ctx.request_repaint();
        }

        // Cmd/Ctrl+S: Save file (works from any pane)
        if ctx.input_mut(|i| {
            // Check for Ctrl (which is Cmd on macOS) or Command modifier
            if i.key_pressed(egui::Key::S) && (i.modifiers.ctrl || i.modifiers.command) {
                i.consume_key(egui::Modifiers::CTRL, egui::Key::S);
                true
            } else {
                false
            }
        }) {
            self.save_file(ctx.clone());
            ctx.request_repaint();
        }

        // Cmd/Ctrl+E: Send entire content to REPL (works from any pane)
        if ctx.input_mut(|i| {
            if i.key_pressed(egui::Key::E) && (i.modifiers.ctrl || i.modifiers.command) {
                i.consume_key(egui::Modifiers::CTRL, egui::Key::E);
                true
            } else {
                false
            }
        }) {
            let content = self.editor.get_content().to_string();
            if !content.trim().is_empty() {
                self.repl.eval_code(content);
                self.active_pane = ActivePane::Repl;
                ctx.request_repaint();
            }
        }

        // Cmd/Ctrl+R: Run all lines (works from any pane)
        if ctx.input_mut(|i| {
            if i.key_pressed(egui::Key::R) && (i.modifiers.ctrl || i.modifiers.command) {
                i.consume_key(egui::Modifiers::CTRL, egui::Key::R);
                true
            } else {
                false
            }
        }) {
            let content = self.editor.get_content().to_string();
            if !content.trim().is_empty() {
                // Send entire content as one block to preserve multi-line structures
                self.repl.eval_code(content);
                self.active_pane = ActivePane::Repl;
                ctx.request_repaint();
            }
        }
    }

    // File operations

    fn open_file_dialog(&mut self, ctx: egui::Context) {
        // Use synchronous file picker dialog
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("DSL Files", &["dsl", "flow"])
            .add_filter("Text Files", &["txt"])
            .add_filter("All Files", &["*"])
            .pick_file()
        {
            let path = path.to_string_lossy().to_string();
            self.load_file(&path, ctx);
        }
    }

    fn save_file(&mut self, ctx: egui::Context) {
        let time = ctx.input(|i| i.time);

        // If no file path, prompt for Save As
        if self.editor.get_file_path().is_none() {
            self.save_file_as_dialog(ctx);
            return;
        }

        match self.editor.save() {
            Ok(()) => {
                self.set_status_message(
                    "Saved successfully!".to_string(),
                    time,
                    StatusKind::Success,
                );
            }
            Err(e) => {
                self.set_status_message(format!("Save error: {}", e), time, StatusKind::Error);
            }
        }
    }

    fn save_file_as_dialog(&mut self, ctx: egui::Context) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("DSL Files", &["dsl", "flow"])
            .add_filter("Text Files", &["txt"])
            .add_filter("All Files", &["*"])
            .save_file()
        {
            let path_str = path.to_string_lossy().to_string();
            let time = ctx.input(|i| i.time);

            // Save content to the chosen path
            match std::fs::write(&path_str, self.editor.get_content()) {
                Ok(()) => {
                    self.editor.set_file_path(Some(path_str.clone()));
                    self.editor.clear_modified();
                    self.add_to_recent_files(path_str);
                    self.set_status_message(
                        "Saved successfully!".to_string(),
                        time,
                        StatusKind::Success,
                    );
                }
                Err(e) => {
                    self.set_status_message(format!("Save error: {}", e), time, StatusKind::Error);
                }
            }
        }
    }

    fn load_file(&mut self, path: &str, ctx: egui::Context) {
        let time = ctx.input(|i| i.time);

        match self.editor.load(path) {
            Ok(()) => {
                self.add_to_recent_files(path.to_string());
                self.set_status_message(format!("Opened {}", path), time, StatusKind::Success);
            }
            Err(e) => {
                self.set_status_message(
                    format!("Error opening file: {}", e),
                    time,
                    StatusKind::Error,
                );
            }
        }
    }

    fn add_to_recent_files(&mut self, path: String) {
        // Remove if already exists (we'll add it to the front)
        self.recent_files.retain(|p| p != &path);

        // Add to front
        self.recent_files.insert(0, path);

        // Keep only the 10 most recent
        if self.recent_files.len() > 10 {
            self.recent_files.truncate(10);
        }
    }

    fn set_status_message(&mut self, message: String, time: f64, kind: StatusKind) {
        self.status_message = Some((message, time, kind));
    }
}
