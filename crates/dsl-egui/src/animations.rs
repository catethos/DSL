use eframe::egui;

/// Available animation types for the REPL header
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationType {
    None,
    AsciiArt,
    FlowingWave,
    AuroraWave,
    Lissajous,
    SpiralGalaxy,
    ParticleFlow,
    DnaHelix,
    RippleEffect,
    BreathingCircle,
    InfinitySymbol,
    ProbabilityMorph,
}

impl AnimationType {
    pub fn all() -> &'static [AnimationType] {
        &[
            AnimationType::None,
            AnimationType::AsciiArt,
            AnimationType::FlowingWave,
            AnimationType::AuroraWave,
            AnimationType::Lissajous,
            AnimationType::SpiralGalaxy,
            AnimationType::ParticleFlow,
            AnimationType::DnaHelix,
            AnimationType::RippleEffect,
            AnimationType::BreathingCircle,
            AnimationType::InfinitySymbol,
            AnimationType::ProbabilityMorph,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            AnimationType::None => "None (Best Performance)",
            AnimationType::AsciiArt => "ASCII Art",
            AnimationType::FlowingWave => "Flowing Wave",
            AnimationType::AuroraWave => "Aurora Wave",
            AnimationType::Lissajous => "Lissajous Curves",
            AnimationType::SpiralGalaxy => "Spiral Galaxy",
            AnimationType::ParticleFlow => "Particle Flow",
            AnimationType::DnaHelix => "DNA Helix",
            AnimationType::RippleEffect => "Ripple Effect",
            AnimationType::BreathingCircle => "Breathing Circle",
            AnimationType::InfinitySymbol => "Infinity Symbol",
            AnimationType::ProbabilityMorph => "Probability Morph",
        }
    }

    /// Returns true if this animation requires continuous repainting
    pub fn needs_animation(&self) -> bool {
        !matches!(self, AnimationType::None | AnimationType::AsciiArt)
    }
}

/// Render the animation plot
pub fn render_animation(ui: &mut egui::Ui, animation_type: AnimationType, time: f64) {
    const PI: f64 = std::f64::consts::PI;

    // Handle None animation - just skip rendering
    if animation_type == AnimationType::None {
        return;
    }

    // Handle ASCII art separately (no plot needed)
    if animation_type == AnimationType::AsciiArt {
        let ascii_art = r#" .o88b.  .d8b.  d8888b. db    db   d88888b db       .d88b.  db   d8b   db
d8P  Y8 d8' `8b 88  `8D `8b  d8'   88'     88      .8P  Y8. 88   I8I   88
8P      88ooo88 88oodD'  `8bd8'    88ooo   88      88    88 88   I8I   88
8b      88~~~88 88~~~      88      88~~~   88      88    88 Y8   I8I   88
Y8b  d8 88   88 88         88      88      88booo. `8b  d8' `8b d8'8b d8'
 `Y88P' YP   YP 88         YP      YP      Y88888P  `Y88P'   `8b8' `8d8'"#;

        ui.add_space(20.0);
        ui.label(
            egui::RichText::new(ascii_art)
                .monospace()
                .color(egui::Color32::from_rgb(100, 150, 255)),
        );
        ui.add_space(20.0);
        return;
    }

    let mut plot = egui_plot::Plot::new("capy_flow_header")
        .height(120.0)
        .width(ui.available_width())
        .allow_zoom(false)
        .allow_drag(false)
        .allow_scroll(false)
        .allow_boxed_zoom(false)
        .show_axes(false)
        .show_grid(false)
        .show_background(false);

    // For probability morph, use fixed aspect ratio, others use data aspect
    if animation_type != AnimationType::ProbabilityMorph {
        plot = plot.data_aspect(1.0);
    }

    let _plot_response = plot.show(ui, |plot_ui| {
        match animation_type {
            AnimationType::None => {
                // Unreachable - handled above
            }
            AnimationType::AsciiArt => {
                // Unreachable - handled above
            }
            AnimationType::FlowingWave => {
                let wave_points: Vec<[f64; 2]> = (0..200)
                    .map(|i| {
                        let x = i as f64 / 20.0;
                        let y =
                            (x * 0.8 + time * 2.0).sin() * 0.5 + (x * 1.5 - time * 3.0).sin() * 0.3;
                        [x, y]
                    })
                    .collect();
                plot_ui.line(
                    egui_plot::Line::new(wave_points)
                        .color(egui::Color32::from_rgb(100, 150, 255))
                        .width(2.0),
                );
            }
            AnimationType::AuroraWave => {
                let colors = [
                    egui::Color32::from_rgb(100, 150, 255),
                    egui::Color32::from_rgb(150, 100, 255),
                    egui::Color32::from_rgb(255, 100, 150),
                ];
                for (i, color) in colors.iter().enumerate() {
                    let offset = i as f64 * 0.5;
                    let wave: Vec<[f64; 2]> = (0..200)
                        .map(|j| {
                            let x = j as f64 / 20.0;
                            let y = (x * 0.8 + time * 2.0 + offset).sin() * 0.5
                                + (x * 1.5 - time * 1.5 + offset).sin() * 0.3
                                + offset;
                            [x, y]
                        })
                        .collect();
                    plot_ui.line(egui_plot::Line::new(wave).color(*color).width(2.0));
                }
            }
            AnimationType::Lissajous => {
                let points: Vec<[f64; 2]> = (0..1000)
                    .map(|i| {
                        let t = i as f64 / 100.0;
                        let a = 3.0;
                        let b = 2.0;
                        let x = (a * t + time * 2.0).sin();
                        let y = (b * t + time * 3.0).sin();
                        [x, y]
                    })
                    .collect();
                plot_ui.line(
                    egui_plot::Line::new(points)
                        .color(egui::Color32::from_rgb(100, 150, 255))
                        .width(2.0),
                );
            }
            AnimationType::SpiralGalaxy => {
                let colors = [
                    egui::Color32::from_rgb(100, 150, 255),
                    egui::Color32::from_rgb(150, 255, 100),
                    egui::Color32::from_rgb(255, 150, 100),
                ];
                for (arm, color) in colors.iter().enumerate() {
                    let offset = arm as f64 * 2.0 * PI / 3.0;
                    let points: Vec<[f64; 2]> = (0..100)
                        .map(|i| {
                            let t = i as f64 / 10.0;
                            let r = t * 0.3;
                            let angle = t + time + offset;
                            [r * angle.cos(), r * angle.sin()]
                        })
                        .collect();
                    plot_ui.line(egui_plot::Line::new(points).color(*color).width(2.0));
                }
            }
            AnimationType::ParticleFlow => {
                let particles: Vec<[f64; 2]> = (0..50)
                    .map(|i| {
                        let x = ((i as f64 * 0.5 + time * 2.0) % 10.0) - 5.0;
                        let y = (i as f64 * 0.3).sin() * (x * 0.5).cos();
                        [x, y]
                    })
                    .collect();
                plot_ui.points(
                    egui_plot::Points::new(particles)
                        .radius(3.0)
                        .color(egui::Color32::from_rgb(100, 150, 255)),
                );
            }
            AnimationType::DnaHelix => {
                let helix1: Vec<[f64; 2]> = (0..100)
                    .map(|i| {
                        let x = i as f64 / 10.0;
                        let y = (x + time * 2.0).sin();
                        [x, y]
                    })
                    .collect();
                let helix2: Vec<[f64; 2]> = (0..100)
                    .map(|i| {
                        let x = i as f64 / 10.0;
                        let y = (x + time * 2.0 + PI).sin();
                        [x, y]
                    })
                    .collect();
                plot_ui.line(
                    egui_plot::Line::new(helix1)
                        .color(egui::Color32::from_rgb(100, 150, 255))
                        .width(2.0),
                );
                plot_ui.line(
                    egui_plot::Line::new(helix2)
                        .color(egui::Color32::from_rgb(255, 100, 150))
                        .width(2.0),
                );
            }
            AnimationType::RippleEffect => {
                for ring in 0..5 {
                    let phase = ring as f64 * 0.5;
                    let r = ((time * 2.0 + phase) % 3.0) + 0.5;
                    let points: Vec<[f64; 2]> = (0..100)
                        .map(|i| {
                            let angle = i as f64 / 100.0 * 2.0 * PI;
                            [r * angle.cos(), r * angle.sin()]
                        })
                        .collect();
                    plot_ui.line(
                        egui_plot::Line::new(points)
                            .color(egui::Color32::from_rgb(100, 150, 255))
                            .width(1.5),
                    );
                }
            }
            AnimationType::BreathingCircle => {
                let radius = 1.0 + (time * 2.0).sin() * 0.5;
                let points: Vec<[f64; 2]> = (0..100)
                    .map(|i| {
                        let angle = i as f64 / 100.0 * 2.0 * PI;
                        [radius * angle.cos(), radius * angle.sin()]
                    })
                    .collect();
                plot_ui.line(
                    egui_plot::Line::new(points)
                        .color(egui::Color32::from_rgb(100, 150, 255))
                        .width(2.0),
                );
            }
            AnimationType::InfinitySymbol => {
                let points: Vec<[f64; 2]> = (0..200)
                    .map(|i| {
                        let t = i as f64 / 100.0 * 2.0 * PI + time;
                        let scale = 1.5;
                        let x = scale * t.cos() / (1.0 + t.sin().powi(2));
                        let y = scale * t.sin() * t.cos() / (1.0 + t.sin().powi(2));
                        [x, y]
                    })
                    .collect();
                plot_ui.line(
                    egui_plot::Line::new(points)
                        .color(egui::Color32::from_rgb(100, 150, 255))
                        .width(2.0),
                );
            }
            AnimationType::ProbabilityMorph => {
                // Beta distribution PDF - right-skewed
                let alpha = 5.0;
                let beta_param = 2.0;

                let beta_pdf = |x: f64| -> f64 {
                    if x <= 0.0 || x >= 1.0 {
                        return 0.0;
                    }

                    // Simplified beta function using gamma approximation
                    let numerator = x.powf(alpha - 1.0) * (1.0 - x).powf(beta_param - 1.0);

                    // Normalization constant (approximate)
                    let ln_beta =
                        (alpha - 1.0) * (0.5_f64.ln()) + (beta_param - 1.0) * (0.5_f64.ln());
                    let normalizer = ln_beta.exp();

                    numerator / normalizer
                };

                // Generate the distribution curve - wider range
                let points: Vec<[f64; 2]> = (0..200)
                    .map(|i| {
                        let x = i as f64 / 200.0;
                        let y = beta_pdf(x);

                        // Map to wider display range: x from -5 to 5
                        let display_x = x * 10.0 - 5.0;
                        [display_x, y * 3.5] // Scale up for visibility
                    })
                    .collect();

                // Add invisible corner points to fix the plot bounds
                // This prevents the plot from shifting when hovering outside the range
                plot_ui.points(
                    egui_plot::Points::new(vec![[-5.0, 0.0], [5.0, 3.5]])
                        .radius(0.0)
                        .color(egui::Color32::TRANSPARENT),
                );

                // Get mouse pointer position in plot coordinates
                let hover_pos = plot_ui.pointer_coordinate();
                let plot_hovered = plot_ui.response().hovered();

                // Store last cursor position in egui's persistent data
                let last_x_id = egui::Id::new("probability_morph_last_x");

                // Get the last stored position, or use a default
                let mut last_x = plot_ui.ctx().data(|d| d.get_temp::<f64>(last_x_id));

                // Update position if hovering
                if plot_hovered {
                    if let Some(mouse_pos) = hover_pos {
                        let mouse_x = mouse_pos.x.clamp(-5.0, 5.0);
                        last_x = Some(mouse_x);
                        // Store the new position
                        plot_ui
                            .ctx()
                            .data_mut(|d| d.insert_temp(last_x_id, mouse_x));
                    }
                }

                // Draw shading if we have a position (either from current hover or previous)
                if let Some(mouse_x) = last_x {
                    // Draw shaded area using many thin vertical rectangles (bars)
                    // This avoids polygon convexity issues
                    for point in points.iter() {
                        let [x, y] = *point;
                        if x <= mouse_x {
                            // Draw a thin vertical line/rectangle from 0 to y
                            let bar_points = vec![[x, 0.0], [x, y]];
                            plot_ui.line(
                                egui_plot::Line::new(bar_points)
                                    .color(egui::Color32::from_rgba_unmultiplied(
                                        100, 150, 255, 100,
                                    ))
                                    .width(2.0),
                            );
                        }
                    }

                    // Only show vertical line and text when actively hovering
                    if plot_hovered {
                        // Calculate approximate cumulative probability
                        let normalized_x = ((mouse_x + 5.0) / 10.0).clamp(0.0, 1.0);
                        let cdf_approx = normalized_x.powf(alpha)
                            / (normalized_x.powf(alpha) + (1.0 - normalized_x).powf(beta_param));

                        // Show vertical line at mouse position
                        plot_ui.vline(
                            egui_plot::VLine::new(mouse_x)
                                .color(egui::Color32::from_rgba_unmultiplied(255, 100, 100, 150))
                                .width(1.5),
                        );

                        // Display cumulative probability value
                        plot_ui.text(
                            egui_plot::Text::new(
                                egui_plot::PlotPoint::new(mouse_x, 3.0),
                                format!("P ≤ {:.2}\n{:.1}%", normalized_x, cdf_approx * 100.0),
                            )
                            .color(egui::Color32::WHITE),
                        );
                    }
                }

                // Draw the distribution curve
                plot_ui.line(
                    egui_plot::Line::new(points)
                        .color(egui::Color32::from_rgb(100, 150, 255))
                        .width(2.5),
                );
            }
        }
    });
}
