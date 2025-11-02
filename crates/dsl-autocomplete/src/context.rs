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

    /// Detect what kind of completion is appropriate
    fn detect_kind(input: &str, cursor: usize) -> ContextKind {
        let before_cursor = &input[..cursor.min(input.len())];

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
                // Extract function name before the paren
                let before_paren = before_cursor[..paren_pos].trim();
                if let Some(func_start) = before_paren.rfind(|c: char| !c.is_alphanumeric() && c != '_') {
                    let func_name = before_paren[func_start + 1..].to_string();
                    return ContextKind::FunctionCall { function: func_name };
                } else {
                    let func_name = before_paren.to_string();
                    return ContextKind::FunctionCall { function: func_name };
                }
            }
        }

        // Check if we're after a def keyword
        if before_cursor.trim_end().ends_with("def") {
            return ContextKind::AfterDef;
        }

        // Check if we're after a type keyword
        if before_cursor.trim_end().ends_with("type") || before_cursor.trim_end().ends_with("enum") {
            return ContextKind::AfterType;
        }

        // Default: general completion
        ContextKind::General
    }

    /// Extract the partial word being typed
    fn extract_partial(input: &str, cursor: usize) -> String {
        let before_cursor = &input[..cursor.min(input.len())];

        // Find the start of the current word
        let word_start = before_cursor
            .rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != ':')
            .map(|pos| pos + 1)
            .unwrap_or(0);

        before_cursor[word_start..].to_string()
    }

    /// Extract additional context-specific data
    fn extract_data(input: &str, cursor: usize, kind: &ContextKind) -> ContextData {
        match kind {
            ContextKind::FunctionCall { function: _ } => {
                // Count how many arguments we've typed so far
                let before_cursor = &input[..cursor.min(input.len())];
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
}
