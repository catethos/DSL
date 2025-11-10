//! The core autocomplete engine

use crate::{matcher::PrefixMatcher, CompletionContext, CompletionProvider, Matcher, Suggestion};

/// The main autocomplete engine
///
/// This is the core component that coordinates providers and matchers to
/// generate completion suggestions.
pub struct AutocompleteEngine {
    providers: Vec<Box<dyn CompletionProvider>>,
    matcher: Box<dyn Matcher>,
    max_suggestions: usize,
}

impl AutocompleteEngine {
    /// Create a new autocomplete engine with default settings
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
            matcher: Box::new(PrefixMatcher::new()),
            max_suggestions: 50,
        }
    }

    /// Register a completion provider
    pub fn register_provider(&mut self, provider: Box<dyn CompletionProvider>) {
        self.providers.push(provider);
        // Sort providers by priority (higher first)
        self.providers
            .sort_by_key(|b| std::cmp::Reverse(b.priority()));
    }

    /// Set the matcher to use for filtering suggestions
    pub fn set_matcher(&mut self, matcher: Box<dyn Matcher>) {
        self.matcher = matcher;
    }

    /// Set the maximum number of suggestions to return
    pub fn set_max_suggestions(&mut self, max: usize) {
        self.max_suggestions = max;
    }

    /// Get completion suggestions for the given input and cursor position
    pub fn complete(&self, input: impl Into<String>, cursor: usize) -> CompletionResult {
        let context = CompletionContext::new(input, cursor);
        self.complete_with_context(&context)
    }

    /// Get completion suggestions for a pre-built context
    pub fn complete_with_context(&self, context: &CompletionContext) -> CompletionResult {
        // Collect suggestions from all providers
        let mut suggestions = Vec::new();

        for provider in &self.providers {
            if provider.can_provide(context) {
                let provider_suggestions = provider.provide(context);
                suggestions.extend(provider_suggestions);
            }
        }

        // Filter and rank using the matcher
        suggestions = self.matcher.filter(&context.partial, suggestions);

        // Limit to max suggestions
        suggestions.truncate(self.max_suggestions);

        CompletionResult {
            suggestions,
            context: context.clone(),
        }
    }
}

impl Default for AutocompleteEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// The result of a completion request
#[derive(Debug, Clone)]
pub struct CompletionResult {
    /// The list of suggestions
    pub suggestions: Vec<Suggestion>,

    /// The context that generated these suggestions
    pub context: CompletionContext,
}

impl CompletionResult {
    /// Check if there are any suggestions
    pub fn is_empty(&self) -> bool {
        self.suggestions.is_empty()
    }

    /// Get the number of suggestions
    pub fn len(&self) -> usize {
        self.suggestions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{provider::StaticProvider, SuggestionKind};

    #[test]
    fn test_engine_with_static_provider() {
        let mut engine = AutocompleteEngine::new();

        let suggestions = vec![
            Suggestion::new("foobar", SuggestionKind::Function),
            Suggestion::new("foobaz", SuggestionKind::Function),
            Suggestion::new("barbaz", SuggestionKind::Function),
        ];

        let provider = StaticProvider::new("test", suggestions);
        engine.register_provider(Box::new(provider));

        let result = engine.complete("foo", 3);

        assert_eq!(result.len(), 2);
        assert!(result
            .suggestions
            .iter()
            .all(|s| s.label.starts_with("foo")));
    }

    #[test]
    fn test_engine_max_suggestions() {
        let mut engine = AutocompleteEngine::new();
        engine.set_max_suggestions(2);

        let suggestions = vec![
            Suggestion::new("foo1", SuggestionKind::Function),
            Suggestion::new("foo2", SuggestionKind::Function),
            Suggestion::new("foo3", SuggestionKind::Function),
        ];

        let provider = StaticProvider::new("test", suggestions);
        engine.register_provider(Box::new(provider));

        let result = engine.complete("foo", 3);

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_provider_priority() {
        let mut engine = AutocompleteEngine::new();

        let low_priority = StaticProvider::new(
            "low",
            vec![Suggestion::new("low", SuggestionKind::Function).priority(1)],
        )
        .with_priority(1);

        let high_priority = StaticProvider::new(
            "high",
            vec![Suggestion::new("high", SuggestionKind::Function).priority(10)],
        )
        .with_priority(10);

        // Register in reverse order
        engine.register_provider(Box::new(low_priority));
        engine.register_provider(Box::new(high_priority));

        // Should still be sorted by priority
        assert_eq!(engine.providers[0].priority(), 10);
        assert_eq!(engine.providers[1].priority(), 1);
    }
}
