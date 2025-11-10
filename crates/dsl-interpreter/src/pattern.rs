use dsl_ir::{IRNode, IRPattern, Value};
use std::collections::HashMap;

pub struct PatternMatcher;

impl PatternMatcher {
    /// Check if pattern matches value
    pub fn matches(pattern: &IRPattern, value: &Value) -> bool {
        match (pattern, value) {
            // Wildcard always matches
            (IRPattern::Any, _) => true,

            // Variable always matches (it binds)
            (IRPattern::Variable(_), _) => true,

            // Literal matching
            (IRPattern::Literal(lit_node), val) => Self::literal_matches(lit_node, val),

            // Binding matches if inner pattern matches
            (IRPattern::Binding(_, inner), val) => Self::matches(inner, val),

            // Type matching
            (IRPattern::Type { type_name, inner }, val) => {
                if !Self::type_name_matches(type_name, val) {
                    return false;
                }
                if let Some(inner_pattern) = inner {
                    Self::matches(inner_pattern, val)
                } else {
                    true
                }
            }

            // List pattern matching
            (IRPattern::List { patterns, rest }, Value::List(values)) => {
                Self::list_matches(patterns, rest.as_ref(), values)
            }

            // Map pattern matching
            (IRPattern::Map { fields, strict }, Value::Map(map)) => {
                Self::map_matches(fields, *strict, map)
            }

            // Tuple pattern (treat as list)
            (IRPattern::Tuple(patterns), Value::List(values)) => {
                if patterns.len() != values.len() {
                    return false;
                }
                patterns
                    .iter()
                    .zip(values.iter())
                    .all(|(p, v)| Self::matches(p, v))
            }

            _ => false,
        }
    }

    /// Extract variable bindings from a successful match
    pub fn extract_bindings(
        pattern: &IRPattern,
        value: &Value,
    ) -> Result<HashMap<String, Value>, String> {
        let mut bindings = HashMap::new();
        Self::extract_recursive(pattern, value, &mut bindings)?;
        Ok(bindings)
    }

    fn extract_recursive(
        pattern: &IRPattern,
        value: &Value,
        bindings: &mut HashMap<String, Value>,
    ) -> Result<(), String> {
        match pattern {
            IRPattern::Variable(name) => {
                bindings.insert(name.clone(), value.clone());
                Ok(())
            }

            IRPattern::Binding(name, inner) => {
                bindings.insert(name.clone(), value.clone());
                Self::extract_recursive(inner, value, bindings)
            }

            IRPattern::Type {
                inner: Some(inner), ..
            } => Self::extract_recursive(inner, value, bindings),

            IRPattern::List { patterns, rest } => {
                if let Value::List(values) = value {
                    // Match fixed patterns
                    for (i, pattern) in patterns.iter().enumerate() {
                        if let Some(val) = values.get(i) {
                            Self::extract_recursive(pattern, val, bindings)?;
                        }
                    }

                    // Bind rest if present
                    if let Some(rest_name) = rest {
                        let rest_values = values.iter().skip(patterns.len()).cloned().collect();
                        bindings.insert(rest_name.clone(), Value::List(rest_values));
                    }
                }
                Ok(())
            }

            IRPattern::Map { fields, .. } => {
                if let Value::Map(map) = value {
                    for (key, pattern) in fields {
                        if let Some(val) = map.get(key) {
                            Self::extract_recursive(pattern, val, bindings)?;
                        }
                    }
                }
                Ok(())
            }

            IRPattern::Tuple(patterns) => {
                if let Value::List(values) = value {
                    for (pattern, val) in patterns.iter().zip(values.iter()) {
                        Self::extract_recursive(pattern, val, bindings)?;
                    }
                }
                Ok(())
            }

            _ => Ok(()),
        }
    }

    fn literal_matches(lit_node: &IRNode, value: &Value) -> bool {
        // Convert IRNode literal to Value and compare
        match (lit_node, value) {
            (IRNode::Int(a), Value::Int(b)) => a == b,
            (IRNode::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
            (IRNode::Bool(a), Value::Bool(b)) => a == b,
            (IRNode::String(a), Value::String(b)) => a == b,
            _ => false,
        }
    }

    fn type_name_matches(type_name: &str, value: &Value) -> bool {
        match (type_name, value) {
            ("String", Value::String(_)) => true,
            ("Int", Value::Int(_)) => true,
            ("Float", Value::Float(_)) => true,
            ("Bool", Value::Bool(_)) => true,
            ("List", Value::List(_)) => true,
            ("Map", Value::Map(_)) => true,
            ("Null", Value::Null) => true,
            ("Markdown", Value::Markdown(_)) => true,
            // TODO: User-defined types
            _ => false,
        }
    }

    fn list_matches(patterns: &[IRPattern], rest: Option<&String>, values: &[Value]) -> bool {
        // Must have at least as many values as patterns
        if values.len() < patterns.len() {
            return false;
        }

        // If no rest pattern, must match exactly
        if rest.is_none() && values.len() != patterns.len() {
            return false;
        }

        // Check each pattern matches corresponding value
        patterns
            .iter()
            .zip(values.iter())
            .all(|(p, v)| Self::matches(p, v))
    }

    fn map_matches(
        fields: &[(String, IRPattern)],
        strict: bool,
        map: &indexmap::IndexMap<String, Value>,
    ) -> bool {
        // All pattern fields must exist and match
        for (key, pattern) in fields {
            match map.get(key) {
                Some(val) => {
                    if !Self::matches(pattern, val) {
                        return false;
                    }
                }
                None => return false,
            }
        }

        // If strict, map can't have extra fields
        if strict {
            map.len() == fields.len()
        } else {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;

    #[test]
    fn test_wildcard() {
        assert!(PatternMatcher::matches(&IRPattern::Any, &Value::Int(42)));
    }

    #[test]
    fn test_literal() {
        let pattern = IRPattern::Literal(Box::new(IRNode::Int(42)));
        assert!(PatternMatcher::matches(&pattern, &Value::Int(42)));
        assert!(!PatternMatcher::matches(&pattern, &Value::Int(43)));
    }

    #[test]
    fn test_variable() {
        let pattern = IRPattern::Variable("x".to_string());
        let value = Value::Int(42);

        assert!(PatternMatcher::matches(&pattern, &value));

        let bindings = PatternMatcher::extract_bindings(&pattern, &value).unwrap();
        assert_eq!(bindings.get("x"), Some(&Value::Int(42)));
    }

    #[test]
    fn test_list_pattern() {
        let pattern = IRPattern::List {
            patterns: vec![
                IRPattern::Variable("head".to_string()),
                IRPattern::Variable("second".to_string()),
            ],
            rest: Some("tail".to_string()),
        };

        let value = Value::List(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
        ]);

        assert!(PatternMatcher::matches(&pattern, &value));

        let bindings = PatternMatcher::extract_bindings(&pattern, &value).unwrap();
        assert_eq!(bindings.get("head"), Some(&Value::Int(1)));
        assert_eq!(bindings.get("second"), Some(&Value::Int(2)));
        assert_eq!(
            bindings.get("tail"),
            Some(&Value::List(vec![Value::Int(3), Value::Int(4)]))
        );
    }

    #[test]
    fn test_map_pattern() {
        let pattern = IRPattern::Map {
            fields: vec![
                ("name".to_string(), IRPattern::Variable("n".to_string())),
                ("age".to_string(), IRPattern::Variable("a".to_string())),
            ],
            strict: false,
        };

        let mut map = IndexMap::new();
        map.insert("name".to_string(), Value::String("Alice".to_string()));
        map.insert("age".to_string(), Value::Int(30));
        map.insert("city".to_string(), Value::String("NYC".to_string()));

        let value = Value::Map(map);

        assert!(PatternMatcher::matches(&pattern, &value));

        let bindings = PatternMatcher::extract_bindings(&pattern, &value).unwrap();
        assert_eq!(bindings.get("n"), Some(&Value::String("Alice".to_string())));
        assert_eq!(bindings.get("a"), Some(&Value::Int(30)));
    }

    #[test]
    fn test_type_pattern() {
        let pattern = IRPattern::Type {
            type_name: "Int".to_string(),
            inner: Some(Box::new(IRPattern::Variable("x".to_string()))),
        };

        let value = Value::Int(42);
        assert!(PatternMatcher::matches(&pattern, &value));

        let bindings = PatternMatcher::extract_bindings(&pattern, &value).unwrap();
        assert_eq!(bindings.get("x"), Some(&Value::Int(42)));
    }
}
