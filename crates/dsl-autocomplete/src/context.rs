//! Completion context analysis

/// Context information for completion
#[derive(Debug, Clone, PartialEq)]
pub struct CompletionContext {
    /// The full input text
    pub input: String,

    /// The cursor position (byte offset)
    pub cursor: usize,

    /// The kind of context detected
    pub kind: ContextKind,

    /// The partial word being typed (before cursor)
    pub partial: String,

    /// Additional context-specific data
    pub data: ContextData,
}

impl CompletionContext {
    /// Create a new completion context from input and cursor position
    pub fn new(input: impl Into<String>, cursor: usize) -> Self {
        let input = input.into();
        let kind = Self::detect_kind(&input, cursor);
        let partial = Self::extract_partial(&input, cursor);
        let data = Self::extract_data(&input, cursor, &kind);

        Self {
            input,
            cursor,
            kind,
            partial,
            data,
        }
    }

    /// Safely slice a string at UTF-8 character boundaries
    /// Returns a substring from `start` to `end` (byte offsets), adjusting boundaries if needed
    fn safe_slice(input: &str, start: usize, end: usize) -> &str {
        let len = input.len();
        let start = start.min(len);
        let end = end.min(len);

        // Adjust start to the nearest valid character boundary
        let start = if start > 0 && !input.is_char_boundary(start) {
            // Move backward to find the start of the character
            (0..=start)
                .rev()
                .find(|&i| input.is_char_boundary(i))
                .unwrap_or(0)
        } else {
            start
        };

        // Adjust end to the nearest valid character boundary
        let end = if end > 0 && !input.is_char_boundary(end) {
            // Move backward to find the start of the character
            (0..=end)
                .rev()
                .find(|&i| input.is_char_boundary(i))
                .unwrap_or(0)
        } else {
            end
        };

        &input[start..end]
    }

    /// Detect what kind of completion is appropriate
    fn detect_kind(input: &str, cursor: usize) -> ContextKind {
        let before_cursor = Self::safe_slice(input, 0, cursor);

        // Check for REPL command (starts with :)
        if before_cursor.trim_start().starts_with(':') {
            return ContextKind::Command;
        }

        // Check for field access (ends with .)
        if let Some(dot_pos) = before_cursor.rfind('.') {
            // Make sure the dot is not inside a string
            if !Self::is_inside_string(before_cursor, dot_pos) {
                let object = before_cursor[..dot_pos].trim().to_string();
                return ContextKind::FieldAccess { object };
            }
        }

        // Check if we're inside a function call (after opening paren)
        if let Some(paren_pos) = before_cursor.rfind('(') {
            if !Self::is_inside_string(before_cursor, paren_pos) {
                // Check if there's a matching closing paren after the opening paren
                // If so, we're not inside the function call anymore
                let after_paren = &before_cursor[paren_pos..];
                let has_closing_paren = after_paren.contains(')');

                if !has_closing_paren {
                    // We're inside the function call
                    // Extract function name before the paren
                    let before_paren = before_cursor[..paren_pos].trim();
                    if let Some(func_start) =
                        before_paren.rfind(|c: char| !c.is_alphanumeric() && c != '_')
                    {
                        let func_name = before_paren[func_start + 1..].to_string();
                        return ContextKind::FunctionCall {
                            function: func_name,
                        };
                    } else {
                        let func_name = before_paren.to_string();
                        return ContextKind::FunctionCall {
                            function: func_name,
                        };
                    }
                }
            }
        }

        // Check if we're after a def keyword
        if before_cursor.trim_end().ends_with("def") {
            return ContextKind::AfterDef;
        }

        // Check if we're after a type keyword
        if before_cursor.trim_end().ends_with("type") || before_cursor.trim_end().ends_with("enum")
        {
            return ContextKind::AfterType;
        }

        // Default: general completion
        ContextKind::General
    }

    /// Extract the partial word being typed
    fn extract_partial(input: &str, cursor: usize) -> String {
        let before_cursor = Self::safe_slice(input, 0, cursor);

        // Find the start of the current word
        let word_start = before_cursor
            .rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != ':')
            .map(|pos| pos + 1)
            .unwrap_or(0);

        // Safe slice from word_start to end of before_cursor
        Self::safe_slice(before_cursor, word_start, before_cursor.len()).to_string()
    }

    /// Extract additional context-specific data
    fn extract_data(input: &str, cursor: usize, kind: &ContextKind) -> ContextData {
        match kind {
            ContextKind::FunctionCall { function: _ } => {
                // Count how many arguments we've typed so far
                let before_cursor = Self::safe_slice(input, 0, cursor);
                if let Some(paren_pos) = before_cursor.rfind('(') {
                    let args_text = &before_cursor[paren_pos + 1..];
                    let arg_count = args_text.matches(',').count();
                    ContextData::ArgumentIndex(arg_count)
                } else {
                    ContextData::None
                }
            }
            _ => ContextData::None,
        }
    }

    /// Check if a position is inside a string literal
    fn is_inside_string(text: &str, pos: usize) -> bool {
        let mut in_string = false;
        let mut escape_next = false;

        for (i, ch) in text.char_indices() {
            if i >= pos {
                break;
            }

            if escape_next {
                escape_next = false;
                continue;
            }

            match ch {
                '"' => in_string = !in_string,
                '\\' if in_string => escape_next = true,
                _ => {}
            }
        }

        in_string
    }
}

/// The kind of completion context
#[derive(Debug, Clone, PartialEq)]
pub enum ContextKind {
    /// General completion (keywords, functions, variables)
    General,

    /// After a dot, suggesting fields
    FieldAccess {
        /// The object whose fields we want
        object: String,
    },

    /// Inside a function call, suggesting parameters
    FunctionCall {
        /// The function being called
        function: String,
    },

    /// After a colon, suggesting REPL commands
    Command,

    /// After "def", user is defining a new function
    AfterDef,

    /// After "type" or "enum", user is defining a new type
    AfterType,
}

/// Additional context-specific data
#[derive(Debug, Clone, PartialEq)]
pub enum ContextData {
    /// No additional data
    None,

    /// The index of the current argument (for function calls)
    ArgumentIndex(usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_general_context() {
        let ctx = CompletionContext::new("let x = ", 8);
        assert_eq!(ctx.kind, ContextKind::General);
        assert_eq!(ctx.partial, "");
    }

    #[test]
    fn test_field_access_context() {
        let ctx = CompletionContext::new("obj.fie", 7);
        assert_eq!(
            ctx.kind,
            ContextKind::FieldAccess {
                object: "obj".to_string()
            }
        );
        assert_eq!(ctx.partial, "fie");
    }

    #[test]
    fn test_command_context() {
        let ctx = CompletionContext::new(":hel", 4);
        assert_eq!(ctx.kind, ContextKind::Command);
        assert_eq!(ctx.partial, ":hel");
    }

    #[test]
    fn test_function_call_context() {
        let ctx = CompletionContext::new("Upper(str", 9);
        assert_eq!(
            ctx.kind,
            ContextKind::FunctionCall {
                function: "Upper".to_string()
            }
        );
    }

    #[test]
    fn test_partial_extraction() {
        let ctx = CompletionContext::new("let foo = Up", 12);
        assert_eq!(ctx.partial, "Up");
    }

    #[test]
    fn test_chinese_characters() {
        // Test with Chinese characters in function call
        let ctx = CompletionContext::new("emotional_value(你喜欢现在的我", 41);
        assert_eq!(
            ctx.kind,
            ContextKind::FunctionCall {
                function: "emotional_value".to_string()
            }
        );

        // Test partial extraction with Chinese
        let ctx2 = CompletionContext::new("我跟游戏进比较重要", 27);
        assert_eq!(ctx2.kind, ContextKind::General);
    }

    #[test]
    fn test_safe_slice() {
        // Test that safe_slice handles multibyte characters correctly
        let text = "你喜欢现在的我";
        let slice = CompletionContext::safe_slice(text, 0, 100);
        assert_eq!(slice, text);

        // Test slicing in the middle - should round down to valid boundary
        let slice2 = CompletionContext::safe_slice(text, 0, 17);
        assert!(slice2.len() <= 17);
        assert!(text.is_char_boundary(slice2.len()));
    }

    #[test]
    fn test_pipe_operator_context() {
        // Test after first |>
        let ctx1 = CompletionContext::new("\"Hello\" |> Upp", 14);
        println!(
            "Test 1 - After first |>: kind={:?}, partial='{}'",
            ctx1.kind, ctx1.partial
        );
        assert_eq!(ctx1.kind, ContextKind::General);
        assert_eq!(ctx1.partial, "Upp");

        // Test after second |>
        let ctx2 = CompletionContext::new("\"Hello\" |> Lower(_) |> ", 24);
        println!(
            "Test 2 - After second |>: kind={:?}, partial='{}'",
            ctx2.kind, ctx2.partial
        );
        assert_eq!(ctx2.kind, ContextKind::General);
        assert_eq!(ctx2.partial, "");

        // Test after second |> with partial
        let ctx3 = CompletionContext::new("\"Hello\" |> Lower(_) |> Upp", 27);
        println!(
            "Test 3 - After second |> with partial: kind={:?}, partial='{}'",
            ctx3.kind, ctx3.partial
        );
        assert_eq!(ctx3.kind, ContextKind::General);
        assert_eq!(ctx3.partial, "Upp");

        // Test that we still detect function call context correctly (inside parens)
        let ctx4 = CompletionContext::new("\"Hello\" |> Lower(", 17);
        println!(
            "Test 4 - Inside function call: kind={:?}, partial='{}'",
            ctx4.kind, ctx4.partial
        );
        assert_eq!(
            ctx4.kind,
            ContextKind::FunctionCall {
                function: "Lower".to_string()
            }
        );
        assert_eq!(ctx4.partial, "");
    }
}
