use tree_sitter::ffi::TSLanguage;

extern "C" {
    fn tree_sitter_dsl() -> *const TSLanguage;
}

/// Get the tree-sitter Language for this grammar.
pub fn language() -> tree_sitter::Language {
    unsafe { tree_sitter::Language::from_raw(tree_sitter_dsl()) }
}

pub const HIGHLIGHTS_QUERY: &str = include_str!("../../queries/highlights.scm");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_load_grammar() {
        let lang = language();
        assert!(lang.node_kind_count() > 0);
    }
}
