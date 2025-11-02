//! Matching algorithms for filtering suggestions

use crate::Suggestion;

/// A trait for matching and scoring suggestions
///
/// Matchers are responsible for filtering suggestions based on the user's
/// input and assigning scores to determine the best matches.
pub trait Matcher: Send + Sync {
    /// Get the name of this matcher (for debugging/logging)
    fn name(&self) -> &str;

    /// Calculate a match score for a candidate string
    ///
    /// Returns Some(score) if the query matches the candidate, where higher
    /// scores indicate better matches. Returns None if there's no match.
    fn score(&self, query: &str, candidate: &str) -> Option<f32>;

    /// Filter and rank a list of suggestions
    ///
    /// This method applies the matching algorithm to all suggestions,
    /// filters out non-matches, assigns scores, and sorts by score.
    fn filter(&self, query: &str, mut suggestions: Vec<Suggestion>) -> Vec<Suggestion> {
        // Score all suggestions
        for suggestion in &mut suggestions {
            if let Some(score) = self.score(query, &suggestion.label) {
                suggestion.score = score;
            } else {
                // No match, set score to negative so it's filtered out
                suggestion.score = -1.0;
            }
        }

        // Filter out non-matches and sort
        suggestions.retain(|s| s.score >= 0.0);
        suggestions.sort();

        suggestions
    }
}

/// A simple case-insensitive prefix matcher
pub struct PrefixMatcher {
    case_sensitive: bool,
}

impl PrefixMatcher {
    /// Create a new prefix matcher
    pub fn new() -> Self {
        Self {
            case_sensitive: false,
        }
    }

    /// Create a case-sensitive prefix matcher
    pub fn case_sensitive() -> Self {
        Self {
            case_sensitive: true,
        }
    }
}

impl Default for PrefixMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Matcher for PrefixMatcher {
    fn name(&self) -> &str {
        "prefix"
    }

    fn score(&self, query: &str, candidate: &str) -> Option<f32> {
        // Don't match anything if query is empty
        if query.is_empty() {
            return None;
        }

        let (query, candidate) = if self.case_sensitive {
            (query.to_string(), candidate.to_string())
        } else {
            (query.to_lowercase(), candidate.to_lowercase())
        };

        if candidate.starts_with(&query) {
            // Score based on how much of the candidate is matched
            // Shorter candidates with same prefix score higher
            let score = query.len() as f32 / candidate.len() as f32;
            Some(score)
        } else {
            None
        }
    }
}

/// A matcher that always matches everything (for testing)
pub struct AllMatcher;

impl Matcher for AllMatcher {
    fn name(&self) -> &str {
        "all"
    }

    fn score(&self, _query: &str, _candidate: &str) -> Option<f32> {
        Some(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SuggestionKind;

    #[test]
    fn test_prefix_matcher() {
        let matcher = PrefixMatcher::new();

        assert_eq!(matcher.score("foo", "foobar"), Some(0.5));
        assert_eq!(matcher.score("foo", "foo"), Some(1.0));
        assert_eq!(matcher.score("foo", "bar"), None);
        // Empty queries should not match anything
        assert_eq!(matcher.score("", "anything"), None);
    }

    #[test]
    fn test_prefix_matcher_case_insensitive() {
        let matcher = PrefixMatcher::new();

        assert_eq!(matcher.score("FOO", "foobar"), Some(0.5));
        assert_eq!(matcher.score("foo", "FooBar"), Some(0.5));
    }

    #[test]
    fn test_filter() {
        let matcher = PrefixMatcher::new();
        let suggestions = vec![
            Suggestion::new("foobar", SuggestionKind::Function),
            Suggestion::new("foobaz", SuggestionKind::Function),
            Suggestion::new("barbaz", SuggestionKind::Function),
        ];

        let filtered = matcher.filter("foo", suggestions);

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|s| s.label.starts_with("foo")));
    }
}
