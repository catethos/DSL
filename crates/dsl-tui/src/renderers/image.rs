use ratatui::text::{Line, Span};
use ratatui::style::{Color as RatatuiColor, Style};
use image::DynamicImage;

/// Render image using ASCII halfblocks for inline display
pub fn image_to_lines(path: &str, image_data: Option<&DynamicImage>, width: usize) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::from(vec![
            Span::styled("📊 ", Style::default()),
            Span::styled("Chart Generated", Style::default().fg(RatatuiColor::Green)),
        ]),
        Line::from(""),
    ];

    // If we have image data, render it as ASCII art with halfblocks
    if let Some(img) = image_data {
        lines.extend(render_image_as_halfblocks(img, width));
        lines.push(Line::from(""));
    }

    // Add path and instructions
    lines.extend(vec![
        Line::from(vec![
            Span::styled("Path: ", Style::default().fg(RatatuiColor::Cyan)),
            Span::raw(path.to_string()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("💡 To view full size: ", Style::default().fg(RatatuiColor::Yellow)),
            Span::raw("open ".to_string()),
            Span::styled(path.to_string(), Style::default().fg(RatatuiColor::Blue)),
        ]),
    ]);

    lines
}

/// Render image as ASCII halfblocks (works in any terminal with 24-bit color)
fn render_image_as_halfblocks(img: &DynamicImage, max_width: usize) -> Vec<Line<'static>> {
    // Resize image to fit terminal width (each character is 2 pixels high)
    // Use much higher resolution for better quality
    let max_width = (max_width.saturating_sub(4)).max(120).min(250); // Increased to 250 pixels wide
    let max_height = 60; // Max lines for image display (60 lines = 120 pixels height)

    let img = img.resize(
        max_width as u32,
        (max_height * 2) as u32, // *2 because we use halfblocks (2 pixels per char)
        image::imageops::FilterType::Lanczos3,
    );

    let img = img.to_rgb8();
    let (width, height) = img.dimensions();

    let mut lines = Vec::new();

    // Process image in pairs of rows (top and bottom half of each character)
    for y in (0..height).step_by(2) {
        let mut spans = Vec::new();

        for x in 0..width {
            // Get top pixel
            let top_pixel = img.get_pixel(x, y);
            let top_color = RatatuiColor::Rgb(top_pixel[0], top_pixel[1], top_pixel[2]);

            // Get bottom pixel (if exists)
            let bottom_color = if y + 1 < height {
                let bottom_pixel = img.get_pixel(x, y + 1);
                RatatuiColor::Rgb(bottom_pixel[0], bottom_pixel[1], bottom_pixel[2])
            } else {
                RatatuiColor::Black
            };

            // Use upper half block character with fg=top, bg=bottom
            spans.push(Span::styled(
                "▀",
                Style::default().fg(top_color).bg(bottom_color),
            ));
        }

        lines.push(Line::from(spans));
    }

    lines
}
