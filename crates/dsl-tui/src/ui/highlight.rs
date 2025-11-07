use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};

/// Highlight multi-line DSL text using Tree-sitter
pub fn highlight_text(text: &str) -> Vec<Line<'static>> {
    // Get the tree-sitter language and highlights query
    let language = tree_sitter_dsl::language();
    let highlights_query = tree_sitter_dsl::HIGHLIGHTS_QUERY;

    // Configure the highlighter
    let mut config = match HighlightConfiguration::new(
        language,
        "dsl",
        highlights_query,
        "", // injections query
        "", // locals query
    ) {
        Ok(config) => config,
        Err(_) => {
            // If tree-sitter grammar is out of sync, fall back to plain text
            return text
                .lines()
                .map(|line| Line::from(line.to_string()))
                .collect();
        }
    };

    // Define highlight names that match our queries/highlights.scm
    let highlight_names = vec![
        "keyword",
        "keyword.special",
        "type",
        "type.builtin",
        "type.definition",
        "function",
        "function.call",
        "constant",
        "constant.builtin",
        "variable",
        "variable.parameter",
        "property",
        "operator",
        "punctuation.bracket",
        "punctuation.delimiter",
        "string",
        "embedded",
        "number",
        "comment",
    ];

    config.configure(&highlight_names);

    // Create highlighter
    let mut highlighter = Highlighter::new();

    // Highlight the text
    let highlights = highlighter
        .highlight(&config, text.as_bytes(), None, |_| None)
        .ok();

    if let Some(highlights) = highlights {
        let mut lines: Vec<Line<'static>> = vec![Line::from(vec![])];
        let mut current_line_spans: Vec<Span<'static>> = Vec::new();
        let mut current_highlight: Option<usize> = None;

        for event in highlights {
            match event {
                Ok(HighlightEvent::Source { start, end }) => {
                    let source = &text[start..end];

                    // Handle newlines within source
                    for (i, line_text) in source.split('\n').enumerate() {
                        if i > 0 {
                            // Push current line and start a new one
                            lines.push(Line::from(current_line_spans.clone()));
                            current_line_spans.clear();
                        }

                        if !line_text.is_empty() {
                            let style = current_highlight
                                .map(|idx| get_style(highlight_names[idx]))
                                .unwrap_or_default();
                            current_line_spans.push(Span::styled(line_text.to_string(), style));
                        }
                    }
                }
                Ok(HighlightEvent::HighlightStart(idx)) => {
                    current_highlight = Some(idx.0);
                }
                Ok(HighlightEvent::HighlightEnd) => {
                    current_highlight = None;
                }
                Err(_) => break,
            }
        }

        // Push the last line
        if !current_line_spans.is_empty() {
            lines.push(Line::from(current_line_spans));
        }

        // Remove the initial empty line if we have content
        if lines.len() > 1 && lines[0].spans.is_empty() {
            lines.remove(0);
        }

        if lines.is_empty() {
            vec![Line::from("")]
        } else {
            lines
        }
    } else {
        // Fallback: no highlighting, just split into lines
        text.lines()
            .map(|line| Line::from(line.to_string()))
            .collect()
    }
}

/// Highlight a single line of DSL code using Tree-sitter
pub fn highlight_line(line: &str) -> Vec<Span<'static>> {
    let lines = highlight_text(line);
    if let Some(first_line) = lines.into_iter().next() {
        first_line.spans
    } else {
        vec![Span::raw(line.to_string())]
    }
}

/// Get style for a highlight name
fn get_style(highlight_name: &str) -> Style {
    match highlight_name {
        "keyword" => Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
        "keyword.special" => Style::default().fg(Color::LightMagenta).add_modifier(Modifier::BOLD),
        "type" | "type.builtin" | "type.definition" => {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        }
        "function" | "function.call" => Style::default().fg(Color::Green),
        "constant" | "constant.builtin" => Style::default().fg(Color::Yellow),
        "variable.parameter" => Style::default().fg(Color::LightBlue),
        "property" => Style::default().fg(Color::LightGreen),
        "operator" => Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        "punctuation.bracket" => Style::default().fg(Color::White),
        "punctuation.delimiter" => Style::default().fg(Color::Gray),
        "string" | "embedded" => Style::default().fg(Color::Green),
        "number" => Style::default().fg(Color::Yellow),
        "comment" => Style::default().fg(Color::DarkGray),
        _ => Style::default(),
    }
}
