/// Enhanced error formatting for parser errors

use pest::error::{Error as PestError, LineColLocation};
use pest::RuleType;

/// Format a Pest error with enhanced context and messaging
pub fn format_parse_error<R: RuleType>(error: &PestError<R>, input: &str) -> String {
    let (line, column) = match error.line_col {
        LineColLocation::Pos((line, col)) => (line, col),
        LineColLocation::Span((line, col), _) => (line, col),
    };

    let error_kind = match &error.variant {
        pest::error::ErrorVariant::ParsingError {
            positives,
            negatives,
        } => {
            let mut message = String::from("Unexpected input");

            if !negatives.is_empty() {
                let neg_str: Vec<String> = negatives.iter().map(|r| format!("{:?}", r)).collect();
                message.push_str(&format!(", found: {}", neg_str.join(", ")));
            }

            if !positives.is_empty() {
                let pos_str: Vec<String> = positives
                    .iter()
                    .map(|r| format_expected_rule(r))
                    .collect();
                message.push_str(&format!(". Expected: {}", pos_str.join(" or ")));
            }

            message
        }
        pest::error::ErrorVariant::CustomError { message } => message.clone(),
    };

    let context = format_error_context(input, line, column);

    format!(
        "Parse error at line {}, column {}:\n{}\n\n{}",
        line, column, error_kind, context
    )
}

/// Format the expected rule in a more user-friendly way
fn format_expected_rule<R: RuleType>(rule: &R) -> String {
    let rule_name = format!("{:?}", rule);
    match rule_name.as_str() {
        "expr" => "expression".to_string(),
        "identifier" => "identifier".to_string(),
        "integer" => "integer".to_string(),
        "float" => "floating-point number".to_string(),
        "string_literal" => "string literal".to_string(),
        "EOI" => "end of input".to_string(),
        _ => rule_name,
    }
}

/// Format the error context showing the line and a pointer to the error location
fn format_error_context(input: &str, line: usize, column: usize) -> String {
    let lines: Vec<&str> = input.lines().collect();

    if line == 0 || line > lines.len() {
        return String::new();
    }

    // Show up to 2 lines before and after for context
    // Convert 1-based line number to 0-based index
    let error_idx = line - 1;
    let start_idx = error_idx.saturating_sub(2);
    let end_idx = (error_idx + 3).min(lines.len());

    let mut context = String::new();

    for i in start_idx..end_idx {
        let line_num = i + 1;
        let line_content = lines[i];

        context.push_str(&format!("{:4} | {}\n", line_num, line_content));

        if line_num == line {
            // Use the actual characters from the line for perfect alignment in all fonts
            // Replace each character with a space, except at the error position where we put ^
            let mut pointer_line = String::from("     | ");
            for (idx, ch) in line_content.chars().enumerate() {
                if idx + 1 == column {
                    pointer_line.push('^');
                } else if ch == '\t' {
                    pointer_line.push('\t'); // Preserve tabs for alignment
                } else {
                    pointer_line.push(' ');
                }
            }
            context.push_str(&format!("{}\n", pointer_line));
        }
    }

    context
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_error_context() {
        let input = "line 1\nline 2\nerror line\nline 4\nline 5";
        let context = format_error_context(input, 3, 7);

        assert!(context.contains("line 2"));
        assert!(context.contains("error line"));
        assert!(context.contains("line 4"));
        assert!(context.contains("^"));
    }

    #[test]
    fn test_format_error_context_first_line() {
        let input = "error line\nline 2\nline 3";
        let context = format_error_context(input, 1, 1);

        assert!(context.contains("error line"));
        assert!(context.contains("^"));
    }

    #[test]
    fn test_format_error_context_last_line() {
        let input = "line 1\nline 2\nerror line";
        let context = format_error_context(input, 3, 5);

        assert!(context.contains("line 1"));
        assert!(context.contains("line 2"));
        assert!(context.contains("error line"));
        assert!(context.contains("^"));
    }
}
