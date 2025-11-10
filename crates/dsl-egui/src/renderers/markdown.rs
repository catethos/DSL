//! Markdown rendering utilities using pulldown-cmark

use egui;
use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};

/// Renders markdown text in egui
///
/// # Arguments
/// * `ui` - The egui UI context
/// * `markdown` - The markdown text to render
pub fn render_markdown(ui: &mut egui::Ui, markdown: &str) {
    let parser = Parser::new(markdown);

    let mut renderer = MarkdownRenderer {
        ui,
        current_text: String::new(),
        current_style: TextStyle::default(),
        in_code_block: false,
        in_list: false,
        list_item_number: 0,
        heading_level: None,
    };

    for event in parser {
        renderer.process_event(event);
    }

    // Flush any remaining text
    renderer.flush_text();
}

#[derive(Default, Clone)]
struct TextStyle {
    bold: bool,
    italic: bool,
    code: bool,
    heading: Option<HeadingLevel>,
}

struct MarkdownRenderer<'a> {
    ui: &'a mut egui::Ui,
    current_text: String,
    current_style: TextStyle,
    in_code_block: bool,
    in_list: bool,
    list_item_number: usize,
    heading_level: Option<HeadingLevel>,
}

impl<'a> MarkdownRenderer<'a> {
    fn process_event(&mut self, event: Event) {
        match event {
            Event::Start(tag) => self.start_tag(tag),
            Event::End(tag_end) => self.end_tag(tag_end),
            Event::Text(text) => {
                self.current_text.push_str(&text);
            }
            Event::Code(code) => {
                self.flush_text();
                self.ui.monospace(code.as_ref());
            }
            Event::SoftBreak => {
                self.current_text.push(' ');
            }
            Event::HardBreak => {
                self.flush_text();
            }
            Event::Html(_) | Event::InlineHtml(_) => {
                // Ignore HTML for now
            }
            Event::Rule => {
                self.flush_text();
                self.ui.separator();
            }
            _ => {}
        }
    }

    fn start_tag(&mut self, tag: Tag) {
        match tag {
            Tag::Paragraph => {
                // Just continue accumulating text
            }
            Tag::Heading { level, .. } => {
                self.flush_text();
                self.heading_level = Some(level);
                self.current_style.heading = Some(level);
            }
            Tag::BlockQuote(_) => {
                self.flush_text();
            }
            Tag::CodeBlock(_) => {
                self.flush_text();
                self.in_code_block = true;
            }
            Tag::List(start_num) => {
                self.flush_text();
                self.in_list = true;
                self.list_item_number = start_num.unwrap_or(1) as usize;
            }
            Tag::Item => {
                self.flush_text();
            }
            Tag::Emphasis => {
                self.current_style.italic = true;
            }
            Tag::Strong => {
                self.current_style.bold = true;
            }
            Tag::Link { .. } => {
                // Links will be handled in end_tag
            }
            _ => {}
        }
    }

    fn end_tag(&mut self, tag_end: TagEnd) {
        match tag_end {
            TagEnd::Paragraph => {
                self.flush_text();
                self.ui.add_space(4.0);
            }
            TagEnd::Heading(_) => {
                self.flush_text();
                self.heading_level = None;
                self.current_style.heading = None;
                self.ui.add_space(8.0);
            }
            TagEnd::BlockQuote(_) => {
                self.flush_text();
            }
            TagEnd::CodeBlock => {
                if !self.current_text.is_empty() {
                    // Render code block
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(40, 40, 40))
                        .inner_margin(egui::Margin::same(8.0))
                        .show(self.ui, |ui| {
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(&self.current_text)
                                        .monospace()
                                        .color(egui::Color32::from_rgb(200, 200, 200)),
                                )
                                .wrap_mode(egui::TextWrapMode::Extend),
                            );
                        });
                    self.current_text.clear();
                }
                self.in_code_block = false;
                self.ui.add_space(4.0);
            }
            TagEnd::List(_) => {
                self.flush_text();
                self.in_list = false;
                self.list_item_number = 0;
                self.ui.add_space(4.0);
            }
            TagEnd::Item => {
                self.flush_text();
                self.list_item_number += 1;
            }
            TagEnd::Emphasis => {
                self.current_style.italic = false;
            }
            TagEnd::Strong => {
                self.current_style.bold = false;
            }
            TagEnd::Link => {
                // Link text has been accumulated, now render it
                self.flush_text();
            }
            _ => {}
        }
    }

    fn flush_text(&mut self) {
        if self.current_text.is_empty() {
            return;
        }

        let text = self.current_text.trim_end();
        if text.is_empty() {
            self.current_text.clear();
            return;
        }

        // Create styled text
        let mut rich_text = egui::RichText::new(text);

        // Apply heading styles
        if let Some(level) = self.current_style.heading {
            let (size, color) = match level {
                HeadingLevel::H1 => (24.0, egui::Color32::from_rgb(200, 200, 255)),
                HeadingLevel::H2 => (20.0, egui::Color32::from_rgb(180, 180, 240)),
                HeadingLevel::H3 => (18.0, egui::Color32::from_rgb(160, 160, 220)),
                HeadingLevel::H4 => (16.0, egui::Color32::from_rgb(140, 140, 200)),
                HeadingLevel::H5 => (14.0, egui::Color32::from_rgb(120, 120, 180)),
                HeadingLevel::H6 => (12.0, egui::Color32::from_rgb(100, 100, 160)),
            };
            rich_text = rich_text.size(size).color(color).strong();
        }

        // Apply inline styles
        if self.current_style.bold {
            rich_text = rich_text.strong();
        }
        if self.current_style.italic {
            rich_text = rich_text.italics();
        }
        if self.current_style.code {
            rich_text = rich_text.monospace();
        }

        // Handle list items
        if self.in_list {
            self.ui.horizontal(|ui| {
                ui.add_space(16.0); // Indent
                if self.list_item_number > 0 {
                    // Ordered list
                    ui.label(format!("{}.", self.list_item_number));
                } else {
                    // Unordered list
                    ui.label("•");
                }
                ui.label(rich_text);
            });
        } else {
            self.ui.label(rich_text);
        }

        self.current_text.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_parsing() {
        let markdown = "# Hello\n\nThis is **bold** and *italic*.";
        let parser = Parser::new(markdown);
        let events: Vec<_> = parser.collect();
        assert!(!events.is_empty());
    }

    #[test]
    fn test_code_block() {
        let markdown = "```rust\nlet x = 42;\n```";
        let parser = Parser::new(markdown);
        let events: Vec<_> = parser.collect();
        assert!(!events.is_empty());
    }
}
