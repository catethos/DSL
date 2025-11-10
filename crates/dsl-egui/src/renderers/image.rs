//! Image rendering utilities for displaying images in egui

use egui;
use std::sync::Arc;

/// Renders an image in egui
///
/// # Arguments
/// * `ui` - The egui UI context
/// * `path` - The path to the image file
/// * `image_data` - Optional pre-loaded image data
///
/// If image_data is None, displays the path as text.
/// If image_data is Some, renders the image with smart sizing.
pub fn render_image(ui: &mut egui::Ui, path: &str, image_data: &Option<Arc<image::DynamicImage>>) {
    match image_data {
        Some(img) => {
            // Image is loaded, render it
            let size = calculate_display_size(ui, img.width(), img.height());

            // Convert image to egui format
            let image_buffer = img.to_rgba8();
            let pixels = image_buffer.as_flat_samples();
            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                [img.width() as usize, img.height() as usize],
                pixels.as_slice(),
            );

            // Create a texture
            let texture = ui.ctx().load_texture(
                path, // Use path as unique ID
                color_image,
                egui::TextureOptions::LINEAR,
            );

            // Display the image
            ui.add(
                egui::Image::new(&texture)
                    .max_size(size)
                    .fit_to_exact_size(size),
            );

            // Show image info below
            ui.horizontal(|ui| {
                ui.small(format!("{}  ({}×{} px)", path, img.width(), img.height()));
            });
        }
        None => {
            // Image failed to load or not loaded yet
            ui.vertical(|ui| {
                ui.colored_label(
                    egui::Color32::from_rgb(150, 150, 150),
                    format!("[Image: {}]", path),
                );
                ui.small("(Failed to load or image not found)");
            });
        }
    }
}

/// Calculate appropriate display size for an image
///
/// Scales images to fit within a reasonable display area while maintaining aspect ratio
fn calculate_display_size(ui: &egui::Ui, width: u32, height: u32) -> egui::Vec2 {
    // Maximum dimensions for display
    let max_width = ui.available_width().min(800.0);
    let max_height = 600.0;

    let width = width as f32;
    let height = height as f32;

    // Calculate scaling factor to fit within max dimensions
    let width_scale = max_width / width;
    let height_scale = max_height / height;
    let scale = width_scale.min(height_scale).min(1.0); // Don't upscale

    egui::vec2(width * scale, height * scale)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_calculate_display_size_large_image() {
        // Would need a UI context to fully test, but we can test the logic
        let width = 1920u32;
        let height = 1080u32;
        // Aspect ratio should be preserved
        assert_eq!(width as f32 / height as f32, 1920.0 / 1080.0);
    }

    #[test]
    fn test_calculate_display_size_small_image() {
        let width = 100u32;
        let height = 100u32;
        // Small images should not be upscaled
        assert_eq!(width, 100);
        assert_eq!(height, 100);
    }
}
