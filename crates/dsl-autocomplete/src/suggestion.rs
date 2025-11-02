//! Suggestion data structures

use std::cmp::Ordering;

/// A completion suggestion
#[derive(Debug, Clone, PartialEq)]
pub struct Suggestion {
    /// The text to display to the user
    pub label: String,

    /// The text to insert when the suggestion is accepted
    /// If None, uses the label
    pub insert_text: Option<String>,

    /// The kind of suggestion (for styling/filtering)
    pub kind: SuggestionKind,

    /// Additional detail about the suggestion (e.g., type signature)
    pub detail: Option<String>,

    /// Documentation or help text
    pub documentation: Option<String>,

    /// Fuzzy match score (higher is better)
    pub score: f32,

    /// Sort priority (higher is better, used before score)
    pub priority: i32,
}

impl Suggestion {
    /// Create a new suggestion
    pub fn new(label: impl Into<String>, kind: SuggestionKind) -> Self {
        Self {
            label: label.into(),
            insert_text: None,
            kind,
            detail: None,
            documentation: None,
            score: 0.0,
            priority: 0,
        }
    }

    /// Set the insert text (builder pattern)
    pub fn insert_text(mut self, text: impl Into<String>) -> Self {
        self.insert_text = Some(text.into());
        self
    }

    /// Set the detail text (builder pattern)
    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Set the documentation (builder pattern)
    pub fn documentation(mut self, doc: impl Into<String>) -> Self {
        self.documentation = Some(doc.into());
        self
    }

    /// Set the priority (builder pattern)
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Get the text that should be inserted
    pub fn get_insert_text(&self) -> &str {
        self.insert_text.as_deref().unwrap_or(&self.label)
    }
}

impl PartialOrd for Suggestion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Suggestion {}

impl Ord for Suggestion {
    fn cmp(&self, other: &Self) -> Ordering {
        // First compare by priority (higher is better)
        match other.priority.cmp(&self.priority) {
            Ordering::Equal => {
                // Then by score (higher is better)
                match other.score.partial_cmp(&self.score) {
                    Some(Ordering::Equal) | None => {
                        // Finally by label (alphabetically)
                        self.label.cmp(&other.label)
                    }
                    Some(ord) => ord,
                }
            }
            ord => ord,
        }
    }
}

/// The kind of suggestion
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SuggestionKind {
    /// A language keyword
    Keyword,

    /// A function (built-in or user-defined)
    Function,

    /// A variable
    Variable,

    /// A type name
    Type,

    /// A field of a struct/class
    Field,

    /// A REPL command
    Command,

    /// An operator
    Operator,

    /// A snippet/template
    Snippet,

    /// Other/custom
    Other,
}

impl SuggestionKind {
    /// Get a single-character prefix for display
    pub fn prefix(&self) -> &'static str {
        match self {
            Self::Keyword => "K",
            Self::Function => "F",
            Self::Variable => "V",
            Self::Type => "T",
            Self::Field => ".",
            Self::Command => ":",
            Self::Operator => "O",
            Self::Snippet => "S",
            Self::Other => "?",
        }
    }

    /// Get a default priority for this kind
    pub fn default_priority(&self) -> i32 {
        match self {
            Self::Variable => 100,  // Variables highest (most commonly used)
            Self::Function => 90,   // Functions second
            Self::Field => 80,      // Fields third
            Self::Keyword => 70,    // Keywords fourth
            Self::Type => 60,       // Types fifth
            Self::Command => 50,    // Commands sixth
            Self::Snippet => 40,    // Snippets seventh
            Self::Operator => 30,   // Operators eighth
            Self::Other => 0,       // Other lowest
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggestion_ordering() {
        let mut suggestions = vec![
            Suggestion::new("foo", SuggestionKind::Variable),
            Suggestion::new("bar", SuggestionKind::Function).priority(10),
            Suggestion::new("baz", SuggestionKind::Keyword),
        ];

        suggestions.sort();

        // Higher priority comes first
        assert_eq!(suggestions[0].label, "bar");
    }

    #[test]
    fn test_suggestion_builder() {
        let suggestion = Suggestion::new("test", SuggestionKind::Function)
            .detail("() -> String")
            .documentation("A test function")
            .priority(5);

        assert_eq!(suggestion.label, "test");
        assert_eq!(suggestion.detail, Some("() -> String".to_string()));
        assert_eq!(suggestion.priority, 5);
    }
}
