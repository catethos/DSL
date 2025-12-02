//! Simplified partial JSON parser for streaming support
//!
//! Handles incomplete JSON objects and arrays from streaming LLM responses.

use anyhow::Result;
use serde_json::Value as JsonValue;

/// Attempt to parse potentially incomplete JSON from a streaming response
///
/// This function tries multiple strategies to handle partial JSON:
/// 1. Parse as-is (might already be complete)
/// 2. Auto-close open structures (objects and arrays)
/// 3. Handle incomplete strings
/// 4. Extract from markdown code blocks first
///
/// # Arguments
/// * `partial_json` - The potentially incomplete JSON string
///
/// # Returns
/// * `Ok(Some(JsonValue))` - Successfully parsed (complete or auto-closed)
/// * `Ok(None)` - Cannot parse yet, need more data
/// * `Err(...)` - Unrecoverable parsing error
pub fn try_parse_partial_json(partial_json: &str) -> Result<Option<JsonValue>> {
    let trimmed = partial_json.trim();

    if trimmed.is_empty() {
        return Ok(None);
    }

    // First, try to extract from markdown
    let extracted = extract_from_markdown(trimmed);

    // Try parsing as-is first (might already be complete)
    if let Ok(value) = serde_json::from_str::<JsonValue>(&extracted) {
        return Ok(Some(value));
    }

    // Try auto-closing structures
    let attempts = generate_completion_attempts(&extracted);

    for attempt in attempts {
        if let Ok(value) = serde_json::from_str::<JsonValue>(&attempt) {
            return Ok(Some(value));
        }
    }

    // If we can't parse it yet, return None (need more data)
    Ok(None)
}

/// Extract JSON from markdown code blocks
fn extract_from_markdown(text: &str) -> String {
    // Check for ```json blocks
    if let Some(start) = text.find("```json") {
        let json_start = start + 7;
        if let Some(end_offset) = text[json_start..].find("```") {
            let json_end = json_start + end_offset;
            return text[json_start..json_end].trim().to_string();
        }
        // If no closing ```, return everything after ```json
        return text[json_start..].trim().to_string();
    }

    // Check for ``` blocks without language
    if let Some(start) = text.find("```") {
        let content_start = start + 3;
        if let Some(end) = text[content_start..].find("```") {
            let content_end = content_start + end;
            let content = text[content_start..content_end].trim();
            if content.starts_with('{') || content.starts_with('[') {
                return content.to_string();
            }
        } else {
            // No closing ```, return everything after ```
            let content = text[content_start..].trim();
            if content.starts_with('{') || content.starts_with('[') {
                return content.to_string();
            }
        }
    }

    // Try to find JSON boundaries
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            if end > start {
                return text[start..=end].to_string();
            }
        }
        // No closing brace, return from { to end
        return text[start..].to_string();
    }

    if let Some(start) = text.find('[') {
        if let Some(end) = text.rfind(']') {
            if end > start {
                return text[start..=end].to_string();
            }
        }
        // No closing bracket, return from [ to end
        return text[start..].to_string();
    }

    text.to_string()
}

/// Generate multiple attempts to complete partial JSON
fn generate_completion_attempts(json: &str) -> Vec<String> {
    let json = json.trim();
    let mut attempts = Vec::new();

    // Strategy 1: Count braces/brackets and auto-close
    let open_braces = json.matches('{').count();
    let close_braces = json.matches('}').count();
    let open_brackets = json.matches('[').count();
    let close_brackets = json.matches(']').count();

    let mut completion = json.to_string();

    // Check if we have an incomplete string at the end
    if has_incomplete_string(&completion) {
        completion.push('"');
    }

    // Close arrays first (inner to outer)
    for _ in 0..(open_brackets.saturating_sub(close_brackets)) {
        completion.push(']');
    }

    // Close objects
    for _ in 0..(open_braces.saturating_sub(close_braces)) {
        completion.push('}');
    }

    attempts.push(completion);

    // Strategy 2: More aggressive - assume we're in the middle of writing a value
    let mut aggressive = json.to_string();

    // If ends with a colon, comma, or opening bracket/brace, might be waiting for a value
    if json.trim_end().ends_with(':') {
        aggressive.push_str("null");
    } else if json.trim_end().ends_with(',') {
        // Remove trailing comma and try closing
        aggressive = aggressive.trim_end().trim_end_matches(',').to_string();
    }

    // Close incomplete string
    if has_incomplete_string(&aggressive) {
        aggressive.push('"');
    }

    // Close structures
    for _ in 0..(open_brackets.saturating_sub(close_brackets)) {
        aggressive.push(']');
    }
    for _ in 0..(open_braces.saturating_sub(close_braces)) {
        aggressive.push('}');
    }

    attempts.push(aggressive);

    // Strategy 3: Remove incomplete last field/element
    if let Some(last_comma) = json.rfind(',') {
        let mut truncated = json[..=last_comma].to_string();
        truncated = truncated.trim_end().trim_end_matches(',').to_string();

        for _ in 0..(open_brackets.saturating_sub(close_brackets)) {
            truncated.push(']');
        }
        for _ in 0..(open_braces.saturating_sub(close_braces)) {
            truncated.push('}');
        }

        attempts.push(truncated);
    }

    attempts
}

/// Check if the JSON string has an incomplete string value at the end
fn has_incomplete_string(json: &str) -> bool {
    let mut in_string = false;
    let mut escape_next = false;
    let mut last_quote_pos = None;

    for (i, c) in json.chars().enumerate() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match c {
            '\\' if in_string => escape_next = true,
            '"' => {
                in_string = !in_string;
                if in_string {
                    last_quote_pos = Some(i);
                } else {
                    last_quote_pos = None;
                }
            }
            _ => {}
        }
    }

    // If we're in a string at the end, it's incomplete
    in_string && last_quote_pos.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_json() {
        let json = r#"{"name": "John", "age": 30}"#;
        let result = try_parse_partial_json(json).unwrap();
        assert!(result.is_some());

        let value = result.unwrap();
        assert_eq!(value["name"], "John");
        assert_eq!(value["age"], 30);
    }

    #[test]
    fn test_incomplete_object() {
        let partial = r#"{"name": "John", "age": 30"#;
        let result = try_parse_partial_json(partial).unwrap();
        assert!(result.is_some());

        let value = result.unwrap();
        assert_eq!(value["name"], "John");
        assert_eq!(value["age"], 30);
    }

    #[test]
    fn test_incomplete_string() {
        let partial = r#"{"name": "Joh"#;
        let result = try_parse_partial_json(partial).unwrap();
        assert!(result.is_some());

        let value = result.unwrap();
        assert_eq!(value["name"], "Joh");
    }

    #[test]
    fn test_incomplete_array() {
        let partial = r#"{"items": [1, 2, 3"#;
        let result = try_parse_partial_json(partial).unwrap();
        assert!(result.is_some());

        let value = result.unwrap();
        assert_eq!(value["items"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_nested_incomplete() {
        let partial = r#"{"person": {"name": "John", "age": 30"#;
        let result = try_parse_partial_json(partial).unwrap();
        assert!(result.is_some());

        let value = result.unwrap();
        assert_eq!(value["person"]["name"], "John");
        assert_eq!(value["person"]["age"], 30);
    }

    #[test]
    fn test_markdown_extraction() {
        let partial = r#"Here's the data:
```json
{"name": "John", "age": 30
```"#;
        let result = try_parse_partial_json(partial).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_markdown_incomplete() {
        let partial = r#"```json
{"name": "John", "age": 30"#;
        let result = try_parse_partial_json(partial).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_trailing_comma() {
        let partial = r#"{"name": "John", "age": 30,"#;
        let result = try_parse_partial_json(partial).unwrap();
        assert!(result.is_some());

        let value = result.unwrap();
        assert_eq!(value["name"], "John");
    }

    #[test]
    fn test_empty_input() {
        let result = try_parse_partial_json("").unwrap();
        assert!(result.is_none());
    }
}
