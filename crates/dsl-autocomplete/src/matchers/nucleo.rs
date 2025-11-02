//! Fuzzy matcher using nucleo-matcher

use crate::Matcher;
use nucleo_matcher::{
    pattern::{CaseMatching, Normalization, Pattern},
    Matcher as NucleoMatcherCore,
};

/// A fuzzy matcher using the nucleo algorithm
pub struct NucleoMatcher {
    matcher: NucleoMatcherCore,
    case_matching: CaseMatching,
}

impl NucleoMatcher {
    /// Create a new nucleo matcher with default settings
    pub fn new() -> Self {
        Self {
            matcher: NucleoMatcherCore::new(nucleo_matcher::Config::DEFAULT),
            case_matching: CaseMatching::Smart,
        }
    }

    /// Create a case-sensitive matcher
    pub fn case_sensitive() -> Self {
        Self {
            matcher: NucleoMatcherCore::new(nucleo_matcher::Config::DEFAULT),
            case_matching: CaseMatching::Respect,
        }
    }

    /// Create a case-insensitive matcher
    pub fn case_insensitive() -> Self {
        Self {
            matcher: NucleoMatcherCore::new(nucleo_matcher::Config::DEFAULT),
            case_matching: CaseMatching::Ignore,
        }
    }
}

impl Default for NucleoMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Matcher for NucleoMatcher {
    fn name(&self) -> &str {
        "nucleo"
    }

    fn score(&self, query: &str, candidate: &str) -> Option<f32> {
        // Don't match anything if query is empty
        if query.is_empty() {
            return None;
        }

        let pattern = Pattern::parse(query, self.case_matching, Normalization::Smart);
        let mut matcher = self.matcher.clone();

        let mut buf = Vec::new();
        let candidate_str = nucleo_matcher::Utf32Str::new(candidate, &mut buf);
        pattern
            .score(candidate_str, &mut matcher)
            .map(|score| score as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nucleo_matcher() {
        let matcher = NucleoMatcher::new();

        // Exact match
        assert!(matcher.score("foo", "foo").is_some());

        // Prefix match
        assert!(matcher.score("foo", "foobar").is_some());

        // Fuzzy match
        assert!(matcher.score("fb", "foobar").is_some());

        // No match
        assert!(matcher.score("xyz", "foobar").is_none());
    }

    #[test]
    fn test_nucleo_case_insensitive() {
        let matcher = NucleoMatcher::case_insensitive();

        assert!(matcher.score("FOO", "foobar").is_some());
        assert!(matcher.score("foo", "FOOBAR").is_some());
    }

    #[test]
    fn test_empty_query() {
        let matcher = NucleoMatcher::new();
        // Empty queries should not match anything
        assert_eq!(matcher.score("", "anything"), None);
    }
}
