use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use super::text::wrap_text;

/// Helper function to flush accumulated text with wrapping
fn flush_text_spans(
    text: &mut String,
    spans: &mut Vec<Span<'static>>,
    lines: &mut Vec<Line<'static>>,
    style: Style,
    width: usize,
    prefix: &str,
) {
    if text.is_empty() && spans.is_empty() {
        return;
    }

    if !text.is_empty() {
        let available_width = width.saturating_sub(prefix.len());
        let wrapped = wrap_text(text, available_width);

        for (i, wrapped_line) in wrapped.iter().enumerate() {
            let mut line_spans = Vec::new();

            // Add prefix only to first line if we have accumulated spans
            if i == 0 && !spans.is_empty() {
                line_spans.extend(spans.drain(..));
            } else if !prefix.is_empty() {
                line_spans.push(Span::raw(prefix.to_string()));
            }

            line_spans.push(Span::styled(wrapped_line.clone(), style));
            lines.push(Line::from(line_spans));
        }

        text.clear();
    } else if !spans.is_empty() {
        // No text but we have spans - add them as a line
        lines.push(Line::from(spans.drain(..).collect::<Vec<_>>()));
    }
}

/// Convert markdown text to styled lines with rich formatting
pub fn markdown_to_lines(markdown: &str, width: usize) -> Vec<Line<'static>> {
    if markdown.is_empty() {
        return vec![Line::from("")];
    }

    let mut lines = Vec::new();
    let mut current_spans: Vec<Span<'static>> = Vec::new();
    let mut current_text = String::new();
    let mut current_style = Style::default();
    let mut in_code_block = false;
    let mut in_list = false;
    let mut list_indent: usize = 0;

    let parser = Parser::new_ext(markdown, Options::all());

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => {
                    // Flush any pending text
                    flush_text_spans(
                        &mut current_text,
                        &mut current_spans,
                        &mut lines,
                        current_style,
                        width,
                        "",
                    );

                    // Style headers with different colors and bold
                    current_style = match level {
                        pulldown_cmark::HeadingLevel::H1 => Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                        pulldown_cmark::HeadingLevel::H2 => Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD),
                        pulldown_cmark::HeadingLevel::H3 => Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::BOLD),
                        _ => Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    };

                    // Add visual prefix for headers
                    let prefix = match level {
                        pulldown_cmark::HeadingLevel::H1 => "█ ",
                        pulldown_cmark::HeadingLevel::H2 => "▓ ",
                        pulldown_cmark::HeadingLevel::H3 => "▒ ",
                        _ => "░ ",
                    };
                    current_spans.push(Span::styled(prefix.to_string(), current_style));
                }
                Tag::Emphasis => {
                    current_style = current_style.add_modifier(Modifier::ITALIC);
                }
                Tag::Strong => {
                    current_style = current_style.add_modifier(Modifier::BOLD).fg(Color::Yellow);
                }
                Tag::CodeBlock(_) => {
                    flush_text_spans(
                        &mut current_text,
                        &mut current_spans,
                        &mut lines,
                        current_style,
                        width,
                        "",
                    );

                    in_code_block = true;
                    // Add top border for code block
                    lines.push(Line::from(vec![Span::styled(
                        "┌".to_string() + &"─".repeat(width.saturating_sub(2)) + "┐",
                        Style::default().fg(Color::DarkGray),
                    )]));
                    current_style = Style::default().fg(Color::Green);
                }
                Tag::List(_) => {
                    in_list = true;
                    list_indent += 1;
                }
                Tag::Item => {
                    flush_text_spans(
                        &mut current_text,
                        &mut current_spans,
                        &mut lines,
                        current_style,
                        width,
                        "",
                    );

                    // Add bullet point with indent
                    let indent = "  ".repeat(list_indent.saturating_sub(1));
                    current_spans.push(Span::styled(
                        format!("{}• ", indent),
                        Style::default().fg(Color::Cyan),
                    ));
                }
                Tag::Paragraph => {
                    flush_text_spans(
                        &mut current_text,
                        &mut current_spans,
                        &mut lines,
                        current_style,
                        width,
                        "",
                    );
                }
                _ => {}
            },
            Event::End(tag_end) => match tag_end {
                TagEnd::Heading(_) => {
                    flush_text_spans(
                        &mut current_text,
                        &mut current_spans,
                        &mut lines,
                        current_style,
                        width,
                        "",
                    );

                    // Add blank line after heading
                    lines.push(Line::from(""));
                    current_style = Style::default();
                }
                TagEnd::Emphasis => {
                    current_style = current_style.remove_modifier(Modifier::ITALIC);
                }
                TagEnd::Strong => {
                    current_style = current_style
                        .remove_modifier(Modifier::BOLD)
                        .fg(Color::Reset);
                }
                TagEnd::CodeBlock => {
                    flush_text_spans(
                        &mut current_text,
                        &mut current_spans,
                        &mut lines,
                        current_style,
                        width,
                        "",
                    );

                    // Add bottom border for code block
                    lines.push(Line::from(vec![Span::styled(
                        "└".to_string() + &"─".repeat(width.saturating_sub(2)) + "┘",
                        Style::default().fg(Color::DarkGray),
                    )]));
                    lines.push(Line::from(""));
                    in_code_block = false;
                    current_style = Style::default();
                }
                TagEnd::List(_) => {
                    flush_text_spans(
                        &mut current_text,
                        &mut current_spans,
                        &mut lines,
                        current_style,
                        width,
                        "",
                    );

                    list_indent = list_indent.saturating_sub(1);
                    if list_indent == 0 {
                        in_list = false;
                        lines.push(Line::from(""));
                    }
                }
                TagEnd::Paragraph => {
                    flush_text_spans(
                        &mut current_text,
                        &mut current_spans,
                        &mut lines,
                        current_style,
                        width,
                        "",
                    );

                    if !in_list {
                        lines.push(Line::from(""));
                    }
                }
                _ => {}
            },
            Event::Text(text) => {
                if in_code_block {
                    // In code blocks, preserve formatting and add border
                    let code_style = Style::default().fg(Color::Green);
                    for line in text.lines() {
                        let line_spans = vec![
                            Span::styled("│ ".to_string(), Style::default().fg(Color::DarkGray)),
                            Span::styled(line.to_string(), code_style),
                        ];
                        lines.push(Line::from(line_spans));
                    }
                } else {
                    // Accumulate text for wrapping
                    current_text.push_str(&text);
                }
            }
            Event::Code(code) => {
                // Flush accumulated text first if switching styles
                if !current_text.is_empty() {
                    flush_text_spans(
                        &mut current_text,
                        &mut current_spans,
                        &mut lines,
                        current_style,
                        width,
                        "",
                    );
                }

                // Inline code - add to current spans
                current_spans.push(Span::styled(
                    format!("`{}`", code),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ));
            }
            Event::SoftBreak => {
                // Soft break becomes a space in wrapped text
                if !in_code_block {
                    current_text.push(' ');
                }
            }
            Event::HardBreak => {
                // Hard break forces a new line
                flush_text_spans(
                    &mut current_text,
                    &mut current_spans,
                    &mut lines,
                    current_style,
                    width,
                    "",
                );
            }
            Event::Rule => {
                flush_text_spans(
                    &mut current_text,
                    &mut current_spans,
                    &mut lines,
                    current_style,
                    width,
                    "",
                );

                lines.push(Line::from(vec![Span::styled(
                    "─".repeat(width),
                    Style::default().fg(Color::DarkGray),
                )]));
                lines.push(Line::from(""));
            }
            _ => {}
        }
    }

    // Flush any remaining text
    flush_text_spans(
        &mut current_text,
        &mut current_spans,
        &mut lines,
        current_style,
        width,
        "",
    );

    if lines.is_empty() {
        vec![Line::from("")]
    } else {
        lines
    }
}
