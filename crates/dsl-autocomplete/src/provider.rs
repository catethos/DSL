//! Completion provider trait

use crate::{CompletionContext, Suggestion};

/// A trait for providing completion suggestions
///
/// Providers are pluggable components that generate suggestions based on
/// the completion context. Multiple providers can be registered with the
/// autocomplete engine.
pub trait CompletionProvider: Send + Sync {
    /// Get the name of this provider (for debugging/logging)
    fn name(&self) -> &str;

    /// Check if this provider can provide suggestions for the given context
    ///
    /// This is called before `provide()` to determine if the provider should
    /// be consulted. Providers can use this to quickly filter out irrelevant
    /// contexts without doing expensive work.
    fn can_provide(&self, context: &CompletionContext) -> bool;

    /// Provide completion suggestions for the given context
    ///
    /// This method should return all relevant suggestions. The autocomplete
    /// engine will handle filtering and ranking based on the user's input.
    fn provide(&self, context: &CompletionContext) -> Vec<Suggestion>;

    /// Get the priority of this provider
    ///
    /// When multiple providers can provide suggestions, they are consulted
    /// in order of priority (higher first). This allows more specific providers
    /// to take precedence over general ones.
    ///
    /// Default priority is 0.
    fn priority(&self) -> i32 {
        0
    }
}

/// A simple provider that returns a static list of suggestions
pub struct StaticProvider {
    name: String,
    suggestions: Vec<Suggestion>,
    priority: i32,
}

impl StaticProvider {
    /// Create a new static provider
    pub fn new(name: impl Into<String>, suggestions: Vec<Suggestion>) -> Self {
        Self {
            name: name.into(),
            suggestions,
            priority: 0,
        }
    }

    /// Set the priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

impl CompletionProvider for StaticProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn can_provide(&self, _context: &CompletionContext) -> bool {
        true
    }

    fn provide(&self, _context: &CompletionContext) -> Vec<Suggestion> {
        self.suggestions.clone()
    }

    fn priority(&self) -> i32 {
        self.priority
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SuggestionKind;

    #[test]
    fn test_static_provider() {
        let suggestions = vec![
            Suggestion::new("foo", SuggestionKind::Keyword),
            Suggestion::new("bar", SuggestionKind::Function),
        ];

        let provider = StaticProvider::new("test", suggestions.clone());
        let context = CompletionContext::new("", 0);

        assert_eq!(provider.name(), "test");
        assert!(provider.can_provide(&context));
        assert_eq!(provider.provide(&context), suggestions);
    }
}
