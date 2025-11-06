//! Keyword completion provider

use crate::{CompletionContext, CompletionProvider, ContextKind, Suggestion, SuggestionKind};

/// A provider for DSL keywords
pub struct KeywordProvider {
    keywords: Vec<Suggestion>,
}

impl KeywordProvider {
    /// Create a new keyword provider with default DSL keywords
    pub fn new() -> Self {
        let keywords = vec![
            // Declaration keywords
            Suggestion::new("def", SuggestionKind::Keyword)
                .detail("Function definition")
                .priority(SuggestionKind::Keyword.default_priority()),
            Suggestion::new("type", SuggestionKind::Keyword)
                .detail("Type definition")
                .priority(SuggestionKind::Keyword.default_priority()),
            Suggestion::new("enum", SuggestionKind::Keyword)
                .detail("Enum definition")
                .priority(SuggestionKind::Keyword.default_priority()),
            Suggestion::new("workflow", SuggestionKind::Keyword)
                .detail("Workflow definition")
                .priority(SuggestionKind::Keyword.default_priority()),
            // Binding and control flow
            Suggestion::new("as", SuggestionKind::Keyword)
                .detail("Variable binding")
                .priority(SuggestionKind::Keyword.default_priority()),
            Suggestion::new("let", SuggestionKind::Keyword)
                .detail("Variable binding (let x = expr)")
                .priority(SuggestionKind::Keyword.default_priority()),
            // Function execution blocks
            Suggestion::new("prompt:", SuggestionKind::Keyword)
                .detail("LLM prompt block")
                .priority(SuggestionKind::Keyword.default_priority()),
            Suggestion::new("sql:", SuggestionKind::Keyword)
                .detail("SQL query block")
                .priority(SuggestionKind::Keyword.default_priority()),
            Suggestion::new("http:", SuggestionKind::Keyword)
                .detail("HTTP request block")
                .priority(SuggestionKind::Keyword.default_priority()),
            // Primitive types
            Suggestion::new("String", SuggestionKind::Type)
                .detail("String type")
                .priority(SuggestionKind::Type.default_priority()),
            Suggestion::new("Int", SuggestionKind::Type)
                .detail("Integer type")
                .priority(SuggestionKind::Type.default_priority()),
            Suggestion::new("Float", SuggestionKind::Type)
                .detail("Float type")
                .priority(SuggestionKind::Type.default_priority()),
            Suggestion::new("Bool", SuggestionKind::Type)
                .detail("Boolean type")
                .priority(SuggestionKind::Type.default_priority()),
            Suggestion::new("Table", SuggestionKind::Type)
                .detail("Table type")
                .priority(SuggestionKind::Type.default_priority()),
            Suggestion::new("Any", SuggestionKind::Type)
                .detail("Any type")
                .priority(SuggestionKind::Type.default_priority()),
            // Boolean literals
            Suggestion::new("true", SuggestionKind::Keyword)
                .detail("Boolean true")
                .priority(SuggestionKind::Keyword.default_priority()),
            Suggestion::new("false", SuggestionKind::Keyword)
                .detail("Boolean false")
                .priority(SuggestionKind::Keyword.default_priority()),
        ];

        Self { keywords }
    }

    /// Create a custom keyword provider with specific keywords
    pub fn with_keywords(keywords: Vec<(&str, &str)>) -> Self {
        let keywords = keywords
            .into_iter()
            .map(|(kw, detail)| {
                Suggestion::new(kw, SuggestionKind::Keyword)
                    .detail(detail)
                    .priority(SuggestionKind::Keyword.default_priority())
            })
            .collect();

        Self { keywords }
    }
}

impl Default for KeywordProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl CompletionProvider for KeywordProvider {
    fn name(&self) -> &str {
        "keywords"
    }

    fn can_provide(&self, context: &CompletionContext) -> bool {
        // Don't provide keywords when we're after a dot (field access),
        // inside a function call, or when we're specifically typing a definition name
        !matches!(
            context.kind,
            ContextKind::FieldAccess { .. }
                | ContextKind::FunctionCall { .. }
                | ContextKind::AfterDef
                | ContextKind::AfterType
                | ContextKind::Command
        )
    }

    fn provide(&self, _context: &CompletionContext) -> Vec<Suggestion> {
        self.keywords.clone()
    }

    fn priority(&self) -> i32 {
        50 // Medium priority - after variables and functions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_provider() {
        let provider = KeywordProvider::new();
        let context = CompletionContext::new("d", 1);

        assert!(provider.can_provide(&context));
        let suggestions = provider.provide(&context);

        assert!(suggestions.iter().any(|s| s.label == "def"));
        assert!(suggestions.iter().any(|s| s.label == "String"));
        assert!(suggestions.iter().any(|s| s.label == "let"));
        assert!(suggestions.iter().any(|s| s.label == "as"));
    }

    #[test]
    fn test_keyword_provider_filtering() {
        let provider = KeywordProvider::new();

        // Should not provide for field access
        let context = CompletionContext::new("obj.", 4);
        assert!(!provider.can_provide(&context));

        // Should not provide after def
        let context = CompletionContext::new("def ", 4);
        assert!(!provider.can_provide(&context));

        // Should not provide inside function calls
        let context = CompletionContext::new("Ask(", 4);
        assert!(!provider.can_provide(&context));
    }
}
