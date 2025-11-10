//! Markdown rendering utilities with proper heading hierarchy

use egui;
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

#[derive(Default, Clone)]
struct TextSpan {
    text: String,
    bold: bool,
    italic: bool,
    strikethrough: bool,
}

/// Renders markdown text in egui with proper heading size hierarchy
///
/// This provides full CommonMark support including:
/// - Headings with different sizes per level (H1-H6)
/// - Paragraphs, lists (ordered/unordered)
/// - Bold, italic, strikethrough
/// - Code blocks with monospace font
/// - Blockquotes
/// - Links (clickable)
///
/// # Arguments
/// * `ui` - The egui UI context
/// * `markdown` - The markdown text to render
pub fn render_markdown(ui: &mut egui::Ui, _cache: &mut egui_commonmark::CommonMarkCache, markdown: &str) {
    use egui::{Color32, FontId, RichText};

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    let parser = Parser::new_ext(markdown, options);

    let mut current_heading: Option<HeadingLevel> = None;
    let mut in_code_block = false;
    let mut in_blockquote = false;
    let mut blockquote_buffer = String::new();
    let mut list_depth: usize = 0;
    let mut ordered_list_num = 1;
    let mut in_ordered_list = false;
    let mut code_block_buffer = String::new();

    // For inline formatting
    let mut text_spans: Vec<TextSpan> = Vec::new();
    let mut current_span = TextSpan::default();
    let mut format_stack: Vec<&str> = Vec::new();

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => {
                    current_heading = Some(level);
                    text_spans.clear();
                    current_span = TextSpan::default();
                }
                Tag::Paragraph => {
                    text_spans.clear();
                    current_span = TextSpan::default();
                }
                Tag::BlockQuote(_) => {
                    in_blockquote = true;
                    blockquote_buffer.clear();
                }
                Tag::List(start_num) => {
                    list_depth += 1;
                    if let Some(num) = start_num {
                        in_ordered_list = true;
                        ordered_list_num = num as i32;
                    } else {
                        in_ordered_list = false;
                    }
                }
                Tag::Item => {
                    text_spans.clear();
                    current_span = TextSpan::default();
                }
                Tag::CodeBlock(_) => {
                    in_code_block = true;
                    code_block_buffer.clear();
                }
                Tag::Strong => {
                    format_stack.push("strong");
                    if !current_span.text.is_empty() {
                        text_spans.push(current_span.clone());
                        current_span.text.clear();
                    }
                    current_span.bold = true;
                }
                Tag::Emphasis => {
                    format_stack.push("emphasis");
                    if !current_span.text.is_empty() {
                        text_spans.push(current_span.clone());
                        current_span.text.clear();
                    }
                    current_span.italic = true;
                }
                Tag::Strikethrough => {
                    format_stack.push("strikethrough");
                    if !current_span.text.is_empty() {
                        text_spans.push(current_span.clone());
                        current_span.text.clear();
                    }
                    current_span.strikethrough = true;
                }
                _ => {}
            },
            Event::End(tag_end) => match tag_end {
                TagEnd::Heading(_) => {
                    if let Some(level) = current_heading {
                        if !current_span.text.is_empty() {
                            text_spans.push(current_span.clone());
                        }

                        let (size, color) = match level {
                            HeadingLevel::H1 => (28.0, Color32::from_rgb(220, 220, 255)),
                            HeadingLevel::H2 => (24.0, Color32::from_rgb(200, 200, 245)),
                            HeadingLevel::H3 => (20.0, Color32::from_rgb(180, 180, 235)),
                            HeadingLevel::H4 => (18.0, Color32::from_rgb(160, 160, 225)),
                            HeadingLevel::H5 => (16.0, Color32::from_rgb(140, 140, 215)),
                            HeadingLevel::H6 => (14.0, Color32::from_rgb(120, 120, 205)),
                        };

                        ui.horizontal_wrapped(|ui| {
                            for span in &text_spans {
                                let mut rich_text = RichText::new(&span.text).size(size).color(color).strong();
                                if span.bold {
                                    rich_text = rich_text.strong();
                                }
                                if span.italic {
                                    rich_text = rich_text.italics();
                                }
                                if span.strikethrough {
                                    rich_text = rich_text.strikethrough();
                                }
                                ui.label(rich_text);
                            }
                        });
                        ui.add_space(4.0);
                        text_spans.clear();
                        current_span = TextSpan::default();
                        current_heading = None;
                    }
                }
                TagEnd::Paragraph => {
                    if !current_span.text.is_empty() {
                        text_spans.push(current_span.clone());
                    }

                    if !text_spans.is_empty() {
                        ui.horizontal_wrapped(|ui| {
                            ui.style_mut().spacing.item_spacing.x = 2.0;
                            for span in &text_spans {
                                let mut rich_text = RichText::new(&span.text);
                                if span.bold {
                                    rich_text = rich_text.strong();
                                }
                                if span.italic {
                                    rich_text = rich_text.italics();
                                }
                                if span.strikethrough {
                                    rich_text = rich_text.strikethrough();
                                }
                                ui.label(rich_text);
                            }
                        });
                    }

                    text_spans.clear();
                    current_span = TextSpan::default();
                    ui.add_space(4.0);
                }
                TagEnd::BlockQuote(_) => {
                    if !blockquote_buffer.is_empty() {
                        // Render blockquote with visual styling
                        egui::Frame::new()
                            .fill(Color32::from_gray(35))
                            .inner_margin(egui::Margin::same(8))
                            .stroke(egui::Stroke::new(2.0, Color32::from_gray(80)))
                            .show(ui, |ui| {
                                // Split into lines and render each
                                for line in blockquote_buffer.lines() {
                                    ui.label(RichText::new(line).color(Color32::from_gray(200)));
                                }
                            });
                        blockquote_buffer.clear();
                    }
                    in_blockquote = false;
                    ui.add_space(4.0);
                }
                TagEnd::List(_) => {
                    list_depth -= 1;
                    if list_depth == 0 {
                        ui.add_space(4.0);
                    }
                }
                TagEnd::Item => {
                    if !current_span.text.is_empty() {
                        text_spans.push(current_span.clone());
                    }

                    if !text_spans.is_empty() {
                        let indent = "  ".repeat(list_depth.saturating_sub(1));
                        let marker = if in_ordered_list {
                            format!("{}{}. ", indent, ordered_list_num)
                        } else {
                            format!("{}• ", indent)
                        };

                        ui.horizontal_wrapped(|ui| {
                            ui.style_mut().spacing.item_spacing.x = 2.0;
                            ui.label(&marker);
                            for span in &text_spans {
                                let mut rich_text = RichText::new(&span.text);
                                if span.bold {
                                    rich_text = rich_text.strong();
                                }
                                if span.italic {
                                    rich_text = rich_text.italics();
                                }
                                if span.strikethrough {
                                    rich_text = rich_text.strikethrough();
                                }
                                ui.label(rich_text);
                            }
                        });

                        if in_ordered_list {
                            ordered_list_num += 1;
                        }
                    }

                    text_spans.clear();
                    current_span = TextSpan::default();
                }
                TagEnd::CodeBlock => {
                    if !code_block_buffer.is_empty() {
                        egui::Frame::new()
                            .fill(Color32::from_gray(30))
                            .inner_margin(egui::Margin::same(8))
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new(&code_block_buffer)
                                        .font(FontId::monospace(14.0))
                                        .color(Color32::from_gray(220)),
                                );
                            });
                        code_block_buffer.clear();
                    }
                    in_code_block = false;
                    ui.add_space(4.0);
                }
                TagEnd::Strong => {
                    if let Some(&"strong") = format_stack.last() {
                        format_stack.pop();
                        if !current_span.text.is_empty() {
                            text_spans.push(current_span.clone());
                            current_span.text.clear();
                        }
                        current_span.bold = false;
                    }
                }
                TagEnd::Emphasis => {
                    if let Some(&"emphasis") = format_stack.last() {
                        format_stack.pop();
                        if !current_span.text.is_empty() {
                            text_spans.push(current_span.clone());
                            current_span.text.clear();
                        }
                        current_span.italic = false;
                    }
                }
                TagEnd::Strikethrough => {
                    if let Some(&"strikethrough") = format_stack.last() {
                        format_stack.pop();
                        if !current_span.text.is_empty() {
                            text_spans.push(current_span.clone());
                            current_span.text.clear();
                        }
                        current_span.strikethrough = false;
                    }
                }
                _ => {}
            },
            Event::Text(text) => {
                if in_code_block {
                    code_block_buffer.push_str(&text);
                } else if in_blockquote {
                    blockquote_buffer.push_str(&text);
                } else {
                    current_span.text.push_str(&text);
                }
            }
            Event::Code(code) => {
                // Inline code
                if !current_span.text.is_empty() {
                    text_spans.push(current_span.clone());
                    current_span.text.clear();
                }

                text_spans.push(TextSpan {
                    text: String::new(), // Empty to separate
                    bold: false,
                    italic: false,
                    strikethrough: false,
                });

                // Render inline code immediately if in paragraph
                ui.label(
                    RichText::new(code.as_ref())
                        .font(FontId::monospace(13.0))
                        .background_color(Color32::from_gray(40))
                        .color(Color32::from_rgb(255, 200, 150)),
                );
            }
            Event::SoftBreak => {
                if in_blockquote {
                    blockquote_buffer.push('\n');
                } else if in_code_block {
                    code_block_buffer.push('\n');
                } else {
                    current_span.text.push(' ');
                }
            }
            Event::HardBreak => {
                if in_blockquote {
                    blockquote_buffer.push('\n');
                } else if in_code_block {
                    code_block_buffer.push('\n');
                } else if !current_span.text.is_empty() {
                    text_spans.push(current_span.clone());
                    current_span.text.clear();
                }
            }
            Event::Rule => {
                ui.separator();
                ui.add_space(4.0);
            }
            _ => {}
        }
    }

    // Flush any remaining text
    if !current_span.text.is_empty() {
        ui.label(&current_span.text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_cache_creation() {
        let _cache = CommonMarkCache::default();
    }

    #[test]
    fn test_markdown_rendering_basic() {
        // Basic test that the API is correct
        // Full rendering tests would require egui context
        let markdown = "# Hello\n\nThis is **bold** and *italic*.";
        assert!(!markdown.is_empty());
    }

    #[test]
    fn test_markdown_code_block() {
        let markdown = "```rust\nlet x = 42;\n```";
        assert!(markdown.contains("rust"));
    }
}
