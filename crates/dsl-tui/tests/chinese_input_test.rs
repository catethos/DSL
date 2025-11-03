/// Integration tests for Chinese character handling in the TUI
///
/// These tests verify that multi-byte UTF-8 characters (like Chinese)
/// can be properly inserted, deleted, and navigated in the input field.
use dsl_tui::App;

#[test]
fn test_insert_single_chinese_char() {
    let mut app = App::new();
    app.insert_char('你');

    assert_eq!(app.input, "你");
    assert_eq!(app.cursor_position, 3); // '你' is 3 bytes
}

#[test]
fn test_insert_multiple_chinese_chars() {
    let mut app = App::new();
    app.insert_char('你');
    app.insert_char('好');
    app.insert_char('世');
    app.insert_char('界');

    assert_eq!(app.input, "你好世界");
    assert_eq!(app.cursor_position, 12); // 4 chars × 3 bytes each
}

#[test]
fn test_insert_mixed_ascii_chinese() {
    let mut app = App::new();
    app.insert_char('h');
    app.insert_char('e');
    app.insert_char('l');
    app.insert_char('l');
    app.insert_char('o');
    app.insert_char('你');
    app.insert_char('好');

    assert_eq!(app.input, "hello你好");
    assert_eq!(app.cursor_position, 11); // 5 ASCII + 6 UTF-8 bytes
}

#[test]
fn test_delete_chinese_char() {
    let mut app = App::new();
    app.insert_char('你');
    app.insert_char('好');

    assert_eq!(app.input, "你好");
    assert_eq!(app.cursor_position, 6);

    app.delete_char(); // Delete '好'

    assert_eq!(app.input, "你");
    assert_eq!(app.cursor_position, 3);

    app.delete_char(); // Delete '你'

    assert_eq!(app.input, "");
    assert_eq!(app.cursor_position, 0);
}

#[test]
fn test_cursor_movement_chinese() {
    let mut app = App::new();
    app.insert_char('你');
    app.insert_char('好');
    app.insert_char('世');

    assert_eq!(app.cursor_position, 9); // End of input

    app.move_cursor_left();
    assert_eq!(app.cursor_position, 6); // Start of '世'

    app.move_cursor_left();
    assert_eq!(app.cursor_position, 3); // Start of '好'

    app.move_cursor_left();
    assert_eq!(app.cursor_position, 0); // Start of '你'

    app.move_cursor_right();
    assert_eq!(app.cursor_position, 3); // After '你'

    app.move_cursor_right();
    assert_eq!(app.cursor_position, 6); // After '好'

    app.move_cursor_right();
    assert_eq!(app.cursor_position, 9); // After '世'
}

#[test]
fn test_cursor_home_end_chinese() {
    let mut app = App::new();
    app.insert_char('你');
    app.insert_char('好');
    app.insert_char('世');
    app.insert_char('界');

    app.move_cursor_home();
    assert_eq!(app.cursor_position, 0);

    app.move_cursor_end();
    assert_eq!(app.cursor_position, 12);
}

#[test]
fn test_insert_in_middle_chinese() {
    let mut app = App::new();
    app.insert_char('你');
    app.insert_char('世');

    // Move cursor to middle (after '你')
    app.move_cursor_home();
    app.move_cursor_right();
    assert_eq!(app.cursor_position, 3);

    // Insert '好' in the middle
    app.insert_char('好');

    assert_eq!(app.input, "你好世");
    assert_eq!(app.cursor_position, 6); // After the inserted '好'
}

#[test]
fn test_delete_forward_chinese() {
    let mut app = App::new();
    app.insert_char('你');
    app.insert_char('好');
    app.insert_char('世');

    // Move cursor to start
    app.move_cursor_home();
    assert_eq!(app.cursor_position, 0);

    // Delete forward (should delete '你')
    app.delete_char_forward();

    assert_eq!(app.input, "好世");
    assert_eq!(app.cursor_position, 0); // Cursor stays at start

    // Delete forward again (should delete '好')
    app.delete_char_forward();

    assert_eq!(app.input, "世");
    assert_eq!(app.cursor_position, 0);
}

#[test]
fn test_emoji_handling() {
    let mut app = App::new();
    app.insert_char('😀'); // Emoji is 4 bytes
    app.insert_char('你');

    assert_eq!(app.input, "😀你");
    assert_eq!(app.cursor_position, 7); // 4 + 3 bytes

    app.move_cursor_left();
    assert_eq!(app.cursor_position, 4); // Start of '你'

    app.move_cursor_left();
    assert_eq!(app.cursor_position, 0); // Start of '😀'
}

#[test]
fn test_function_call_with_chinese() {
    let mut app = App::new();

    // Type: emotional_value(你喜欢现在的我)
    for c in "emotional_value(".chars() {
        app.insert_char(c);
    }
    for c in "你喜欢现在的我".chars() {
        app.insert_char(c);
    }
    app.insert_char(')');

    assert_eq!(app.input, "emotional_value(你喜欢现在的我)");

    // The cursor should be at the end
    assert_eq!(app.cursor_position, app.input.len());
}
