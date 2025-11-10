//! Keyword completion provider

use crate::{CompletionContext, CompletionProvider, ContextKind, Suggestion, SuggestionKind};
use dsl_core::{boolean_info, keyword_info, type_info};

/// A provider for DSL keywords
pub struct KeywordProvider {
    keywords: Vec<Suggestion>,
}

impl KeywordProvider {
    /// Create a new keyword provider with default DSL keywords
    /// Keywords are sourced from dsl-core to ensure consistency
    pub fn new() -> Self {
        let mut keywords = Vec::new();

        // Add keywords from dsl-core
        for kw in keyword_info() {
            keywords.push(
                Suggestion::new(kw.keyword, SuggestionKind::Keyword)
                    .detail(kw.description)
                    .priority(SuggestionKind::Keyword.default_priority()),
            );
        }

        // Add types from dsl-core
        for ty in type_info() {
            keywords.push(
                Suggestion::new(ty.type_name, SuggestionKind::Type)
                    .detail(ty.description)
                    .priority(SuggestionKind::Type.default_priority()),
            );
        }

        // Add boolean literals from dsl-core
        for bool_lit in boolean_info() {
            keywords.push(
                Suggestion::new(bool_lit.literal, SuggestionKind::Keyword)
                    .detail(bool_lit.description)
                    .priority(SuggestionKind::Keyword.default_priority()),
            );
        }

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
