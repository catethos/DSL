/// Validation layer for syntax errors that Pest's PEG parser might miss
/// due to backtracking. This catches common errors early with better messages.

#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    pub message: String,
    pub position: usize,
    pub line: usize,
    pub column: usize,
}

impl ValidationError {
    pub fn new(message: String, position: usize, input: &str) -> Self {
        let (line, column) = position_to_line_col(input, position);
        Self {
            message,
            position,
            line,
            column,
        }
    }

    pub fn format(&self, input: &str) -> String {
        format!(
            "Syntax error at line {}, column {}: {}\n{}",
            self.line,
            self.column,
            self.message,
            self.format_context(input)
        )
    }

    fn format_context(&self, input: &str) -> String {
        let lines: Vec<&str> = input.lines().collect();
        if self.line == 0 || self.line > lines.len() {
            return String::new();
        }

        let line_content = lines[self.line - 1];
        let pointer = format!("{}^", " ".repeat(self.column.saturating_sub(1)));

        format!("{}\n{}", line_content, pointer)
    }
}

/// Convert byte position to (line, column) - both 1-indexed
fn position_to_line_col(input: &str, position: usize) -> (usize, usize) {
    let mut line = 1;
    let mut column = 1;

    for (i, ch) in input.chars().enumerate() {
        if i >= position {
            break;
        }
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }

    (line, column)
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Delimiter {
    Paren,      // ()
    Bracket,    // []
    Brace,      // {}
    DoubleQuote, // ""
    SingleQuote, // ''
    TripleQuote, // """
}

impl Delimiter {
    fn opening(&self) -> &'static str {
        match self {
            Delimiter::Paren => "(",
            Delimiter::Bracket => "[",
            Delimiter::Brace => "{",
            Delimiter::DoubleQuote => "\"",
            Delimiter::SingleQuote => "'",
            Delimiter::TripleQuote => "\"\"\"",
        }
    }

    fn closing(&self) -> &'static str {
        match self {
            Delimiter::Paren => ")",
            Delimiter::Bracket => "]",
            Delimiter::Brace => "}",
            Delimiter::DoubleQuote => "\"",
            Delimiter::SingleQuote => "'",
            Delimiter::TripleQuote => "\"\"\"",
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Delimiter::Paren => "parenthesis",
            Delimiter::Bracket => "bracket",
            Delimiter::Brace => "brace",
            Delimiter::DoubleQuote => "double quote",
            Delimiter::SingleQuote => "single quote",
            Delimiter::TripleQuote => "triple quote",
        }
    }
}

struct DelimiterFrame {
    delimiter: Delimiter,
    position: usize,
}

/// Validate that all delimiters are properly balanced
pub fn validate_balanced_delimiters(input: &str) -> Result<(), ValidationError> {
    let mut stack: Vec<DelimiterFrame> = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    let mut in_comment = false;

    while i < chars.len() {
        // Skip content inside strings first (before processing comments)
        if let Some(top) = stack.last() {
            match top.delimiter {
                Delimiter::TripleQuote => {
                    // Check for triple quotes to close
                    if i + 2 < chars.len()
                        && chars[i] == '"'
                        && chars[i + 1] == '"'
                        && chars[i + 2] == '"'
                    {
                        stack.pop();
                        i += 3;
                        continue;
                    }
                    i += 1;
                    continue;
                }
                Delimiter::DoubleQuote => {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        // Skip escaped character
                        i += 2;
                        continue;
                    } else if chars[i] == '"' {
                        stack.pop();
                        i += 1;
                        continue;
                    } else {
                        i += 1;
                        continue;
                    }
                }
                Delimiter::SingleQuote => {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        // Skip escaped character
                        i += 2;
                        continue;
                    } else if chars[i] == '\'' {
                        stack.pop();
                        i += 1;
                        continue;
                    } else {
                        i += 1;
                        continue;
                    }
                }
                _ => {}
            }
        }

        // Handle line comments
        if !in_comment && i + 1 < chars.len() && chars[i] == '#' {
            // Skip to end of line
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }

        // Handle block comments
        if !in_comment && i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '*' {
            in_comment = true;
            i += 2;
            continue;
        }

        if in_comment && i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '/' {
            in_comment = false;
            i += 2;
            continue;
        }

        if in_comment {
            i += 1;
            continue;
        }

        // Check for triple quotes first (only when not already inside a string)
        if i + 2 < chars.len()
            && chars[i] == '"'
            && chars[i + 1] == '"'
            && chars[i + 2] == '"'
        {
            stack.push(DelimiterFrame {
                delimiter: Delimiter::TripleQuote,
                position: i,
            });
            i += 3;
            continue;
        }

        // Handle delimiters
        match chars[i] {
            '(' => {
                stack.push(DelimiterFrame {
                    delimiter: Delimiter::Paren,
                    position: i,
                });
            }
            ')' => {
                if let Some(top) = stack.last() {
                    if top.delimiter == Delimiter::Paren {
                        stack.pop();
                    } else {
                        return Err(ValidationError::new(
                            format!(
                                "Unexpected closing ')'. Expected closing {} for {} opened at position {}",
                                top.delimiter.closing(),
                                top.delimiter.name(),
                                top.position
                            ),
                            i,
                            input,
                        ));
                    }
                } else {
                    return Err(ValidationError::new(
                        "Unexpected closing ')' with no matching opening '('".to_string(),
                        i,
                        input,
                    ));
                }
            }
            '[' => {
                stack.push(DelimiterFrame {
                    delimiter: Delimiter::Bracket,
                    position: i,
                });
            }
            ']' => {
                if let Some(top) = stack.last() {
                    if top.delimiter == Delimiter::Bracket {
                        stack.pop();
                    } else {
                        return Err(ValidationError::new(
                            format!(
                                "Unexpected closing ']'. Expected closing {} for {} opened at position {}",
                                top.delimiter.closing(),
                                top.delimiter.name(),
                                top.position
                            ),
                            i,
                            input,
                        ));
                    }
                } else {
                    return Err(ValidationError::new(
                        "Unexpected closing ']' with no matching opening '['".to_string(),
                        i,
                        input,
                    ));
                }
            }
            '{' => {
                stack.push(DelimiterFrame {
                    delimiter: Delimiter::Brace,
                    position: i,
                });
            }
            '}' => {
                if let Some(top) = stack.last() {
                    if top.delimiter == Delimiter::Brace {
                        stack.pop();
                    } else {
                        return Err(ValidationError::new(
                            format!(
                                "Unexpected closing '}}'. Expected closing {} for {} opened at position {}",
                                top.delimiter.closing(),
                                top.delimiter.name(),
                                top.position
                            ),
                            i,
                            input,
                        ));
                    }
                } else {
                    return Err(ValidationError::new(
                        "Unexpected closing '}' with no matching opening '{'".to_string(),
                        i,
                        input,
                    ));
                }
            }
            '"' => {
                stack.push(DelimiterFrame {
                    delimiter: Delimiter::DoubleQuote,
                    position: i,
                });
            }
            '\'' => {
                stack.push(DelimiterFrame {
                    delimiter: Delimiter::SingleQuote,
                    position: i,
                });
            }
            _ => {}
        }

        i += 1;
    }

    // Check for unclosed delimiters - report the innermost (most recent) one
    if let Some(top) = stack.last() {
        return Err(ValidationError::new(
            format!(
                "Unclosed {}. Expected closing {} for {} opened here",
                top.delimiter.name(),
                top.delimiter.closing(),
                top.delimiter.opening()
            ),
            top.position,
            input,
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balanced_delimiters() {
        assert!(validate_balanced_delimiters("foo()").is_ok());
        assert!(validate_balanced_delimiters("foo(bar())").is_ok());
        assert!(validate_balanced_delimiters("arr[0]").is_ok());
        assert!(validate_balanced_delimiters("{x: 1}").is_ok());
        assert!(validate_balanced_delimiters(r#""hello""#).is_ok());
        assert!(validate_balanced_delimiters("'hello'").is_ok());
    }

    #[test]
    fn test_unclosed_paren() {
        let result = validate_balanced_delimiters("foo(");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Unclosed"));
        assert!(err.message.contains("parenthesis"));
    }

    #[test]
    fn test_unclosed_bracket() {
        let result = validate_balanced_delimiters("arr[5");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Unclosed"));
        assert!(err.message.contains("bracket"));
    }

    #[test]
    fn test_unclosed_brace() {
        let result = validate_balanced_delimiters("Person { name: \"Alice\"");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Unclosed"));
        assert!(err.message.contains("brace"));
    }

    #[test]
    fn test_unclosed_string() {
        let result = validate_balanced_delimiters(r#"AnalyzeWithClaude("test"#);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Unclosed"));
        assert!(err.message.contains("quote"));
    }

    #[test]
    fn test_mismatched_delimiter() {
        let result = validate_balanced_delimiters("foo(]");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Unexpected"));
    }

    #[test]
    fn test_escaped_quotes_in_string() {
        assert!(validate_balanced_delimiters(r#""hello \"world\"""#).is_ok());
    }

    #[test]
    fn test_nested_delimiters() {
        assert!(validate_balanced_delimiters("foo(bar[baz{x: 1}])").is_ok());
    }

    #[test]
    fn test_triple_quoted_string() {
        assert!(validate_balanced_delimiters(r#""""hello""""#).is_ok());
        let result = validate_balanced_delimiters(r#""""hello"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_comments_ignored() {
        assert!(validate_balanced_delimiters("foo() # unclosed (").is_ok());
        assert!(validate_balanced_delimiters("foo() /* unclosed ( */").is_ok());
    }

    #[test]
    fn test_hash_inside_string() {
        // Regression test: '#' inside a string should not be treated as a comment
        assert!(validate_balanced_delimiters(r#"RenderMarkdown(" # Hello ")"#).is_ok());
        assert!(validate_balanced_delimiters("\"# not a comment\"").is_ok());
        assert!(validate_balanced_delimiters("'# also not a comment'").is_ok());
        assert!(validate_balanced_delimiters("\"\"\"# triple quote not a comment\"\"\"").is_ok());
    }
}
