use eframe::egui;
use crate::output_item::{ErrorDetail, ErrorDetails};

/// Render an error with rich formatting and expandable sections
pub fn render_error(ui: &mut egui::Ui, error: &mut ErrorDetail) {
    egui::Frame::default()
        .fill(egui::Color32::from_rgb(40, 30, 30))        // Dark red background
        .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(200, 60, 60)))
        .inner_margin(12.0)
        .corner_radius(4.0)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                // Header: Error Type and Location
                render_error_header(ui, error);

                ui.add_space(8.0);

                // Main message (use monospace for proper alignment of error pointers)
                ui.monospace(egui::RichText::new(&error.message)
                    .color(egui::Color32::WHITE)
                    .size(14.0));

                ui.add_space(4.0);

                // Error-specific details
                match &error.details.clone() {
                    ErrorDetails::Type { expected, got } => {
                        render_type_error_details(ui, &expected, &got);
                    }
                    ErrorDetails::LLM { prompt, response } => {
                        render_llm_error_details(ui, error, &prompt, &response);
                    }
                    ErrorDetails::HTTP { method, url } => {
                        render_http_error_details(ui, &method, &url);
                    }
                    ErrorDetails::SQL { query } => {
                        render_sql_error_details(ui, error, &query);
                    }
                    ErrorDetails::UnknownVariable { name } => {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Variable:")
                                .color(egui::Color32::from_rgb(180, 180, 180)));
                            ui.monospace(egui::RichText::new(name)
                                .color(egui::Color32::from_rgb(255, 200, 100)));
                        });
                    }
                    ErrorDetails::UnknownFunction { name } => {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Function:")
                                .color(egui::Color32::from_rgb(180, 180, 180)));
                            ui.monospace(egui::RichText::new(name)
                                .color(egui::Color32::from_rgb(255, 200, 100)));
                        });
                    }
                    _ => {}
                }

                // Suggestions
                if !error.suggestions.is_empty() {
                    ui.add_space(4.0);
                    render_suggestions_section(ui, error);
                }

                // Source context
                if error.source_span.is_some() {
                    ui.add_space(4.0);
                    render_source_context_section(ui, error);
                }
            });
        });
}

fn render_error_header(ui: &mut egui::Ui, error: &ErrorDetail) {
    ui.horizontal(|ui| {
        // Error icon and type
        ui.colored_label(
            egui::Color32::from_rgb(255, 100, 100),
            egui::RichText::new(format!("🔴 {}", error.error_type.to_uppercase()))
                .strong()
                .size(15.0),
        );

        // Location
        if let Some(span) = &error.source_span {
            ui.label(
                egui::RichText::new(format!("at {}:{}:{}", span.file, span.line, span.column))
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .size(12.0)
            );
        }

        // Function context
        if let Some(func) = &error.function_context {
            ui.label(
                egui::RichText::new(format!("in {}", func))
                    .color(egui::Color32::from_rgb(180, 180, 200))
                    .size(12.0)
            );
        }
    });
}

fn render_type_error_details(ui: &mut egui::Ui, expected: &str, got: &str) {
    ui.add_space(4.0);

    egui::Frame::default()
        .fill(egui::Color32::from_rgb(30, 25, 25))
        .inner_margin(8.0)
        .corner_radius(2.0)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("Type Mismatch:")
                    .color(egui::Color32::from_rgb(220, 220, 220))
                    .strong());

                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Expected:")
                        .color(egui::Color32::from_rgb(180, 180, 180)));
                    ui.monospace(
                        egui::RichText::new(expected)
                            .color(egui::Color32::from_rgb(100, 200, 100))
                    );
                });

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Got:     ")
                        .color(egui::Color32::from_rgb(180, 180, 180)));
                    ui.monospace(
                        egui::RichText::new(got)
                            .color(egui::Color32::from_rgb(255, 100, 100))
                    );
                });
            });
        });
}

fn render_llm_error_details(
    ui: &mut egui::Ui,
    error: &mut ErrorDetail,
    prompt: &Option<String>,
    response: &Option<String>,
) {
    if let Some(p) = prompt {
        render_expandable_section(ui, error, "Prompt", p, "📝");
    }
    if let Some(r) = response {
        render_expandable_section(ui, error, "Response", r, "💬");
    }
}

fn render_http_error_details(
    ui: &mut egui::Ui,
    method: &Option<String>,
    url: &Option<String>,
) {
    ui.add_space(4.0);

    egui::Frame::default()
        .fill(egui::Color32::from_rgb(30, 25, 25))
        .inner_margin(8.0)
        .corner_radius(2.0)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                if let Some(m) = method {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Method:")
                            .color(egui::Color32::from_rgb(180, 180, 180)));
                        ui.monospace(egui::RichText::new(m)
                            .color(egui::Color32::from_rgb(150, 200, 255)));
                    });
                }
                if let Some(u) = url {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("URL:")
                            .color(egui::Color32::from_rgb(180, 180, 180)));
                        ui.monospace(egui::RichText::new(u)
                            .color(egui::Color32::from_rgb(150, 200, 255)));
                    });
                }
            });
        });
}

fn render_sql_error_details(ui: &mut egui::Ui, error: &mut ErrorDetail, query: &Option<String>) {
    if let Some(q) = query {
        render_expandable_section(ui, error, "Query", q, "🗄️");
    }
}

fn render_expandable_section(
    ui: &mut egui::Ui,
    error: &mut ErrorDetail,
    title: &str,
    content: &str,
    icon: &str,
) {
    let is_expanded = error.expanded_sections.contains(title);

    ui.add_space(4.0);

    ui.horizontal(|ui| {
        let arrow = if is_expanded { "▼" } else { "▶" };
        if ui.button(egui::RichText::new(arrow).size(12.0))
            .on_hover_text(if is_expanded { "Collapse" } else { "Expand" })
            .clicked()
        {
            if is_expanded {
                error.expanded_sections.remove(title);
            } else {
                error.expanded_sections.insert(title.to_string());
            }
        }

        ui.label(egui::RichText::new(format!("{} {}", icon, title))
            .color(egui::Color32::from_rgb(200, 200, 100))
            .strong());
    });

    if is_expanded {
        ui.add_space(4.0);

        // Limit displayed length
        let display_text = if content.len() > 500 {
            format!("{}...", &content[..500])
        } else {
            content.to_string()
        };

        egui::Frame::default()
            .fill(egui::Color32::from_rgb(20, 20, 20))
            .inner_margin(8.0)
            .corner_radius(2.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(150.0)
                    .show(ui, |ui| {
                        ui.monospace(egui::RichText::new(display_text)
                            .color(egui::Color32::from_rgb(220, 220, 220))
                            .size(12.0));
                    });
            });
    }
}

fn render_suggestions_section(ui: &mut egui::Ui, error: &mut ErrorDetail) {
    let is_expanded = error.expanded_sections.contains("suggestions");

    ui.horizontal(|ui| {
        let arrow = if is_expanded { "▼" } else { "▶" };
        if ui.button(egui::RichText::new(arrow).size(12.0))
            .on_hover_text(if is_expanded { "Collapse" } else { "Expand" })
            .clicked()
        {
            if is_expanded {
                error.expanded_sections.remove("suggestions");
            } else {
                error.expanded_sections.insert("suggestions".to_string());
            }
        }

        ui.label(egui::RichText::new("💡 Suggestions")
            .color(egui::Color32::from_rgb(255, 220, 100))
            .strong());
    });

    if is_expanded {
        ui.add_space(4.0);

        egui::Frame::default()
            .fill(egui::Color32::from_rgb(30, 28, 20))
            .inner_margin(8.0)
            .corner_radius(2.0)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    for suggestion in &error.suggestions {
                        ui.horizontal(|ui| {
                            ui.label("•");
                            ui.label(egui::RichText::new(suggestion)
                                .color(egui::Color32::from_rgb(220, 220, 180)));
                        });
                    }
                });
            });
    }
}

fn render_source_context_section(ui: &mut egui::Ui, error: &mut ErrorDetail) {
    let is_expanded = error.expanded_sections.contains("source");

    ui.horizontal(|ui| {
        let arrow = if is_expanded { "▼" } else { "▶" };
        if ui.button(egui::RichText::new(arrow).size(12.0))
            .on_hover_text(if is_expanded { "Collapse" } else { "Expand" })
            .clicked()
        {
            if is_expanded {
                error.expanded_sections.remove("source");
            } else {
                error.expanded_sections.insert("source".to_string());
            }
        }

        ui.label(egui::RichText::new("📍 Source Location")
            .color(egui::Color32::from_rgb(200, 200, 200))
            .strong());
    });

    if is_expanded {
        if let Some(span) = &error.source_span {
            ui.add_space(4.0);

            egui::Frame::default()
                .fill(egui::Color32::from_rgb(30, 25, 25))
                .inner_margin(8.0)
                .corner_radius(2.0)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("File:")
                                .color(egui::Color32::from_rgb(180, 180, 180)));
                            ui.monospace(egui::RichText::new(&span.file)
                                .color(egui::Color32::from_rgb(150, 200, 255)));
                        });
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Line:")
                                .color(egui::Color32::from_rgb(180, 180, 180)));
                            ui.monospace(egui::RichText::new(span.line.to_string())
                                .color(egui::Color32::from_rgb(150, 200, 255)));
                            ui.label(egui::RichText::new("Column:")
                                .color(egui::Color32::from_rgb(180, 180, 180)));
                            ui.monospace(egui::RichText::new(span.column.to_string())
                                .color(egui::Color32::from_rgb(150, 200, 255)));
                        });
                    });
                });
        }
    }
}
