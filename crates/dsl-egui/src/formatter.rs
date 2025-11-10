//! Code formatting utilities for DSL editor

/// Formatting configuration
#[derive(Debug, Clone)]
pub struct FormatterConfig {
    /// Number of spaces per indent level
    pub indent_size: usize,
    /// Use spaces (true) or tabs (false)
    pub use_spaces: bool,
    /// Auto-close brackets/braces
    pub auto_close_brackets: bool,
}

impl Default for FormatterConfig {
    fn default() -> Self {
        Self {
            indent_size: 2,
            use_spaces: true,
            auto_close_brackets: true,
        }
    }
}

impl FormatterConfig {
    /// Get the indentation string for one level
    pub fn indent_string(&self) -> String {
        if self.use_spaces {
            " ".repeat(self.indent_size)
        } else {
            "\t".to_string()
        }
    }
}

/// Calculate the indentation level at a given position in the text
pub fn calculate_indent_level(text: &str, cursor_pos: usize, _config: &FormatterConfig) -> usize {
    let text_before_cursor = &text[..cursor_pos.min(text.len())];

    let mut level: usize = 0;
    let mut in_string = false;
    let mut escape_next = false;
    let mut chars = text_before_cursor.chars().peekable();

    while let Some(ch) = chars.next() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match ch {
            '\\' if in_string => {
                escape_next = true;
            }
            '"' | '\'' => {
                in_string = !in_string;
            }
            '{' | '[' | '(' if !in_string => {
                level += 1;
            }
            '}' | ']' | ')' if !in_string => {
                level = level.saturating_sub(1);
            }
            _ => {}
        }
    }

    level
}

/// Get the indentation of the current line
pub fn get_current_line_indent(text: &str, cursor_pos: usize) -> usize {
    let line_start = text[..cursor_pos.min(text.len())]
        .rfind('\n')
        .map(|pos| pos + 1)
        .unwrap_or(0);

    let line = &text[line_start..];
    line.chars().take_while(|ch| ch.is_whitespace() && *ch != '\n').count()
}

/// Handle Enter key press with smart indentation
pub fn handle_enter(
    text: &str,
    cursor_pos: usize,
    config: &FormatterConfig,
) -> (String, usize) {
    let mut new_text = String::with_capacity(text.len() + 20);
    new_text.push_str(&text[..cursor_pos]);

    // Check if we just opened a bracket
    let char_before = text[..cursor_pos].chars().last();
    let char_after = text[cursor_pos..].chars().next();

    let should_auto_close = matches!(
        (char_before, char_after),
        (Some('{'), Some('}')) | (Some('['), Some(']')) | (Some('('), Some(')'))
    );

    // Calculate indent level
    let base_indent = calculate_indent_level(text, cursor_pos, config);
    let indent_str = config.indent_string();

    new_text.push('\n');

    if should_auto_close {
        // Add indented line and closing bracket line
        new_text.push_str(&indent_str.repeat(base_indent));
        let new_cursor = new_text.len();
        new_text.push('\n');
        new_text.push_str(&indent_str.repeat(base_indent - 1));
        new_text.push_str(&text[cursor_pos..]);
        (new_text, new_cursor)
    } else {
        // Just add indentation
        new_text.push_str(&indent_str.repeat(base_indent));
        let new_cursor = new_text.len();
        new_text.push_str(&text[cursor_pos..]);
        (new_text, new_cursor)
    }
}

/// Handle opening bracket/brace with auto-close
pub fn handle_open_bracket(
    text: &str,
    cursor_pos: usize,
    bracket: char,
    config: &FormatterConfig,
) -> Option<(String, usize)> {
    if !config.auto_close_brackets {
        return None;
    }

    let closing = match bracket {
        '{' => '}',
        '[' => ']',
        '(' => ')',
        _ => return None,
    };

    let mut new_text = String::with_capacity(text.len() + 2);
    new_text.push_str(&text[..cursor_pos]);
    new_text.push(bracket);
    let new_cursor = new_text.len();
    new_text.push(closing);
    new_text.push_str(&text[cursor_pos..]);

    Some((new_text, new_cursor))
}

/// Format entire document
pub fn format_document(text: &str, config: &FormatterConfig) -> String {
    let mut result = String::with_capacity(text.len());
    let mut indent_level: usize = 0;
    let mut at_line_start = true;
    let mut in_string = false;
    let mut string_char = '\0';
    let mut escape_next = false;

    let indent_str = config.indent_string();

    for ch in text.chars() {
        if escape_next {
            result.push(ch);
            escape_next = false;
            continue;
        }

        // Handle strings
        if in_string {
            result.push(ch);
            if ch == '\\' {
                escape_next = true;
            } else if ch == string_char {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                if at_line_start {
                    result.push_str(&indent_str.repeat(indent_level));
                    at_line_start = false;
                }
                result.push(ch);
                in_string = true;
                string_char = ch;
            }
            '{' | '[' | '(' => {
                if at_line_start {
                    result.push_str(&indent_str.repeat(indent_level));
                    at_line_start = false;
                }
                result.push(ch);
                indent_level += 1;
            }
            '}' | ']' | ')' => {
                indent_level = indent_level.saturating_sub(1);
                if at_line_start {
                    result.push_str(&indent_str.repeat(indent_level));
                    at_line_start = false;
                }
                result.push(ch);
            }
            '\n' => {
                result.push(ch);
                at_line_start = true;
            }
            c if c.is_whitespace() => {
                if !at_line_start {
                    result.push(c);
                }
            }
            _ => {
                if at_line_start {
                    result.push_str(&indent_str.repeat(indent_level));
                    at_line_start = false;
                }
                result.push(ch);
            }
        }
    }

    result
}

/// Find matching bracket position
pub fn find_matching_bracket(text: &str, cursor_pos: usize) -> Option<usize> {
    if cursor_pos >= text.len() {
        return None;
    }

    let chars: Vec<char> = text.chars().collect();
    let ch = chars.get(cursor_pos)?;

    let (opening, closing, direction): (char, char, i32) = match ch {
        '{' => ('{', '}', 1),
        '}' => ('{', '}', -1),
        '[' => ('[', ']', 1),
        ']' => ('[', ']', -1),
        '(' => ('(', ')', 1),
        ')' => ('(', ')', -1),
        _ => return None,
    };

    let mut depth = 0;
    let mut pos = cursor_pos as i32;

    loop {
        if pos < 0 || pos >= chars.len() as i32 {
            return None;
        }

        let current = chars[pos as usize];

        if current == opening {
            if direction == 1 {
                depth += 1;
            } else {
                depth -= 1;
            }
        } else if current == closing {
            if direction == 1 {
                depth -= 1;
            } else {
                depth += 1;
            }
        }

        if depth == 0 && pos != cursor_pos as i32 {
            return Some(pos as usize);
        }

        pos += direction;
    }
}

/// Handle Tab key - insert indentation
pub fn handle_tab(
    text: &str,
    cursor_pos: usize,
    config: &FormatterConfig,
) -> (String, usize) {
    let indent = config.indent_string();
    let mut new_text = String::with_capacity(text.len() + indent.len());
    new_text.push_str(&text[..cursor_pos]);
    new_text.push_str(&indent);
    let new_cursor = new_text.len();
    new_text.push_str(&text[cursor_pos..]);
    (new_text, new_cursor)
}

/// Handle Shift+Tab - remove indentation
pub fn handle_untab(
    text: &str,
    cursor_pos: usize,
    config: &FormatterConfig,
) -> (String, usize) {
    let line_start = text[..cursor_pos]
        .rfind('\n')
        .map(|pos| pos + 1)
        .unwrap_or(0);

    let line_indent = &text[line_start..cursor_pos];
    let indent_size = config.indent_size;

    // Remove up to one indent level
    let chars_to_remove = line_indent
        .chars()
        .rev()
        .take(indent_size)
        .take_while(|ch| ch.is_whitespace())
        .count();

    if chars_to_remove == 0 {
        return (text.to_string(), cursor_pos);
    }

    let remove_pos = cursor_pos - chars_to_remove;
    let mut new_text = String::with_capacity(text.len());
    new_text.push_str(&text[..remove_pos]);
    new_text.push_str(&text[cursor_pos..]);

    (new_text, remove_pos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indent_level_simple() {
        let config = FormatterConfig::default();
        let text = "def foo() {";
        let level = calculate_indent_level(text, text.len(), &config);
        assert_eq!(level, 1);
    }

    #[test]
    fn test_indent_level_nested() {
        let config = FormatterConfig::default();
        let text = "def foo() {\n  let x = {";
        let level = calculate_indent_level(text, text.len(), &config);
        assert_eq!(level, 2);
    }

    #[test]
    fn test_matching_bracket() {
        let text = "def foo() { return 42 }";
        assert_eq!(find_matching_bracket(text, 10), Some(22));
        assert_eq!(find_matching_bracket(text, 22), Some(10));
    }

    #[test]
    fn test_format_simple() {
        let config = FormatterConfig::default();
        let input = "def foo(){\nlet x=1\nx\n}";
        let output = format_document(input, &config);
        println!("Input: {:?}", input);
        println!("Output: {:?}", output);
        // Check that content inside braces is indented
        assert!(output.contains("  let") || output.contains("  x"));
    }
}
