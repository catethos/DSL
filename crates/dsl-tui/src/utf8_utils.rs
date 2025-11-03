/// UTF-8 string slicing utilities to safely handle multi-byte characters
///
/// This module provides utilities for safely slicing UTF-8 strings at character
/// boundaries, preventing panics when dealing with multi-byte characters like Chinese,
/// Japanese, emoji, etc.

/// Safely slice a string at UTF-8 character boundaries
/// Returns a substring from `start` to `end` (byte offsets), adjusting boundaries if needed
pub fn safe_slice(input: &str, start: usize, end: usize) -> &str {
    let len = input.len();
    let start = start.min(len);
    let end = end.min(len);

    // Adjust start to the nearest valid character boundary
    let start = if start > 0 && !input.is_char_boundary(start) {
        // Move backward to find the start of the character
        (0..=start)
            .rev()
            .find(|&i| input.is_char_boundary(i))
            .unwrap_or(0)
    } else {
        start
    };

    // Adjust end to the nearest valid character boundary
    let end = if end > 0 && !input.is_char_boundary(end) {
        // Move backward to find the start of the character
        (0..=end)
            .rev()
            .find(|&i| input.is_char_boundary(i))
            .unwrap_or(0)
    } else {
        end
    };

    &input[start..end]
}

/// Safely get a prefix of a string up to a byte position
/// Equivalent to `&input[..end]` but safe for UTF-8
pub fn safe_prefix(input: &str, end: usize) -> &str {
    safe_slice(input, 0, end)
}

/// Safely get a suffix of a string from a byte position
/// Equivalent to `&input[start..]` but safe for UTF-8
pub fn safe_suffix(input: &str, start: usize) -> &str {
    let len = input.len();
    safe_slice(input, start, len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_slice_ascii() {
        let text = "hello world";
        assert_eq!(safe_slice(text, 0, 5), "hello");
        assert_eq!(safe_slice(text, 6, 11), "world");
    }

    #[test]
    fn test_safe_slice_chinese() {
        let text = "你好世界"; // "Hello World" in Chinese
                               // Each Chinese character is 3 bytes
        assert_eq!(safe_slice(text, 0, 100), text);

        // Slicing in the middle of a character should round down
        let slice = safe_slice(text, 0, 4); // Would be in the middle of second char
        assert!(slice.len() <= 4);
        assert!(text.is_char_boundary(slice.len()));
    }

    #[test]
    fn test_safe_slice_mixed() {
        let text = "hello你好world";
        let slice = safe_slice(text, 0, 100);
        assert_eq!(slice, text);
    }

    #[test]
    fn test_safe_prefix() {
        let text = "你喜欢现在的我";
        let prefix = safe_prefix(text, 17);
        assert!(prefix.len() <= 17);
        assert!(text.is_char_boundary(prefix.len()));
    }

    #[test]
    fn test_safe_suffix() {
        let text = "emotional_value(你喜欢";
        let suffix = safe_suffix(text, 16);
        assert!(text.is_char_boundary(suffix.as_ptr() as usize - text.as_ptr() as usize));
    }
}
