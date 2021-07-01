use super::*;
use crate::core::BufferId;
use crate::line::Line;

fn make_buf(text: &[&[u8]]) -> Buffer {
    let mut buf = Buffer::new(BufferId(0), "test");
    let mut last = buf.head;
    for &line_text in text {
        let id = buf.insert_after(last, Line::new());
        if !line_text.is_empty() {
            buf.line_mut(id).unwrap().text = line_text.to_vec();
        }
        last = id;
    }
    buf
}

#[test]
fn test_find_forward_simple() {
    let buf = make_buf(&[b"hello world"]);
    let r = find_forward(&buf, b"world", LineId(1), 0);
    assert_eq!(r, Some((LineId(1), 11)));
}

#[test]
fn test_find_forward_at_start() {
    let buf = make_buf(&[b"hello world"]);
    let r = find_forward(&buf, b"hello", LineId(1), 0);
    assert_eq!(r, Some((LineId(1), 5)));
}

#[test]
fn test_find_forward_not_found() {
    let buf = make_buf(&[b"hello world"]);
    let r = find_forward(&buf, b"xyz", LineId(1), 0);
    assert_eq!(r, None);
}

#[test]
fn test_find_forward_empty_pattern() {
    let buf = make_buf(&[b"hello"]);
    let r = find_forward(&buf, b"", LineId(1), 0);
    assert_eq!(r, None);
}

#[test]
fn test_find_forward_case_insensitive() {
    let buf = make_buf(&[b"Hello World"]);
    let r = find_forward(&buf, b"world", LineId(1), 0);
    assert_eq!(r, Some((LineId(1), 11)));
}

#[test]
fn test_find_forward_multi_line() {
    let buf = make_buf(&[b"hello", b"world"]);
    let r = find_forward(&buf, b"hello\nworld", LineId(1), 0);
    assert_eq!(r, Some((LineId(2), 5)));
}

#[test]
fn test_find_forward_past_start() {
    let buf = make_buf(&[b"abc abc"]);
    let r = find_forward(&buf, b"abc", LineId(1), 4);
    assert_eq!(r, Some((LineId(1), 7)));
}

#[test]
fn test_find_forward_empty_buffer() {
    let buf = Buffer::new(BufferId(0), "empty");
    let r = find_forward(&buf, b"a", LineId(0), 0);
    assert_eq!(r, None);
}

#[test]
fn test_find_forward_single_char() {
    let buf = make_buf(&[b"abc"]);
    let r = find_forward(&buf, b"b", LineId(1), 0);
    assert_eq!(r, Some((LineId(1), 2)));
}

#[test]
fn test_find_backward_simple() {
    let buf = make_buf(&[b"hello world"]);
    let r = find_backward(&buf, b"world", LineId(1), 11);
    assert_eq!(r, Some((LineId(1), 6)));
}

#[test]
fn test_find_backward_at_start() {
    let buf = make_buf(&[b"hello world"]);
    let r = find_backward(&buf, b"hello", LineId(1), 11);
    assert_eq!(r, Some((LineId(1), 0)));
}

#[test]
fn test_find_backward_not_found() {
    let buf = make_buf(&[b"hello world"]);
    let r = find_backward(&buf, b"xyz", LineId(1), 11);
    assert_eq!(r, None);
}

#[test]
fn test_find_backward_empty_pattern() {
    let buf = make_buf(&[b"hello"]);
    let r = find_backward(&buf, b"", LineId(1), 0);
    assert_eq!(r, None);
}

#[test]
fn test_find_backward_case_insensitive() {
    let buf = make_buf(&[b"Hello World"]);
    let r = find_backward(&buf, b"hello", LineId(1), 11);
    assert_eq!(r, Some((LineId(1), 0)));
}

#[test]
fn test_find_backward_multi_line() {
    let buf = make_buf(&[b"hello", b"world"]);
    let r = find_backward(&buf, b"hello\nworld", LineId(2), 6);
    assert_eq!(r, Some((LineId(1), 0)));
}

#[test]
fn test_find_backward_no_match_from_self() {
    let buf = make_buf(&[b"abc abc"]);
    let r = find_backward(&buf, b"abc", LineId(1), 4);
    assert_eq!(r, Some((LineId(1), 0)));
}

#[test]
fn test_find_backward_empty_buffer() {
    let buf = Buffer::new(BufferId(0), "empty");
    let r = find_backward(&buf, b"a", LineId(0), 0);
    assert_eq!(r, None);
}

#[test]
fn test_find_forward_and_backward_match() {
    let buf = make_buf(&[b"find me"]);
    let fwd = find_forward(&buf, b"find", LineId(1), 0);
    let bwd = find_backward(&buf, b"find", LineId(1), 7);
    assert_eq!(fwd, Some((LineId(1), 4)));
    assert_eq!(bwd, Some((LineId(1), 0)));
}

#[test]
fn test_matches_at_exact() {
    let buf = make_buf(&[b"abc"]);
    let r = matches_at(&buf, b"abc", LineId(1), 0);
    assert_eq!(r, Some((LineId(1), 3)));
}

#[test]
fn test_matches_at_partial() {
    let buf = make_buf(&[b"abc"]);
    let r = matches_at(&buf, b"ab", LineId(1), 0);
    assert_eq!(r, Some((LineId(1), 2)));
}

#[test]
fn test_matches_at_no_match_extended() {
    let buf = make_buf(&[b"abc"]);
    let r = matches_at(&buf, b"abd", LineId(1), 0);
    assert_eq!(r, None);
}

#[test]
fn test_matches_at_case_insensitive() {
    let buf = make_buf(&[b"ABC"]);
    let r = matches_at(&buf, b"abc", LineId(1), 0);
    assert_eq!(r, Some((LineId(1), 3)));
}

#[test]
fn test_find_forward_multiple_matches() {
    let buf = make_buf(&[b"a b a b"]);
    let first = find_forward(&buf, b"a", LineId(1), 0);
    assert_eq!(first, Some((LineId(1), 1)));
    let second = find_forward(&buf, b"a", LineId(1), 2);
    assert_eq!(second, Some((LineId(1), 5)));
}

#[test]
fn test_find_forward_regex_match() {
    let buf = make_buf(&[b"hello 42 world"]);
    let r = find_forward_regex(&buf, b"\\d+", LineId(1), 0);
    assert_eq!(r, Some((LineId(1), 6)));
}

#[test]
fn test_find_forward_regex_no_match() {
    let buf = make_buf(&[b"hello world"]);
    let r = find_forward_regex(&buf, b"\\d+", LineId(1), 0);
    assert_eq!(r, None);
}

#[test]
fn test_find_backward_regex_match() {
    let buf = make_buf(&[b"abc 123 def 456"]);
    let r = find_backward_regex(&buf, b"\\d+", LineId(1), 15);
    assert_eq!(r, Some((LineId(1), 12)));
}

#[test]
fn test_find_forward_regex_multi_line() {
    let buf = make_buf(&[b"hello", b"world"]);
    let r = find_forward_regex(&buf, b"hello\\nworld", LineId(1), 0);
    assert_eq!(r, Some((LineId(1), 0)));
}

#[test]
fn test_is_regex_default_false() {
    let buf = Buffer::new(BufferId(0), "test");
    assert!(!is_regex(&buf));
}

#[test]
fn test_is_boundary_forward_at_end() {
    let buf = make_buf(&[b"hello"]);
    let line = buf.head_line().next();
    assert!(is_boundary_forward(&buf, line, 5));
}

#[test]
fn test_is_boundary_forward_not_at_end() {
    let buf = make_buf(&[b"hello"]);
    let line = buf.head_line().next();
    assert!(!is_boundary_forward(&buf, line, 3));
}

#[test]
fn test_is_boundary_backward_at_start() {
    let buf = make_buf(&[b"hello"]);
    let line = buf.head_line().next();
    assert!(is_boundary_backward(&buf, line, 0));
}

#[test]
fn test_is_boundary_backward_not_at_start() {
    let buf = make_buf(&[b"hello"]);
    let line = buf.head_line().next();
    assert!(!is_boundary_backward(&buf, line, 3));
}

#[test]
fn test_eq_ignore_ascii_case() {
    assert!(eq(b'A', b'a', false));
    assert!(eq(b'Z', b'z', false));
    assert!(!eq(b'A', b'b', false));
}

#[test]
fn test_eq_exact() {
    assert!(eq(b'A', b'A', true));
    assert!(!eq(b'A', b'a', true));
}

#[test]
fn test_advance_forward_at_end_of_line() {
    let buf = make_buf(&[b"hello", b"world"]);
    let line = buf.head_line().next();
    let mut l = line;
    let mut o = 5;
    let result = advance_forward(&buf, &mut l, &mut o);
    assert!(result.is_some());
    let expected = buf.next_line(line).unwrap();
    assert_eq!(l, expected);
    assert_eq!(o, 0);
}

#[test]
fn test_advance_forward_past_last_line() {
    let buf = make_buf(&[b"hello"]);
    let line = buf.head_line().next();
    let mut l = line;
    let mut o = 5;
    let result = advance_forward(&buf, &mut l, &mut o);
    assert!(result.is_none());
}

#[test]
fn test_advance_backward_at_start_of_line() {
    let buf = make_buf(&[b"hello", b"world"]);
    let first = buf.head_line().next();
    let second = buf.next_line(first).unwrap();
    let mut l = second;
    let mut o = 0;
    let result = advance_backward(&buf, &mut l, &mut o);
    assert!(result.is_some());
    assert_eq!(l, first);
    assert_eq!(o, 5);
}

#[test]
fn test_advance_backward_past_first_line() {
    let buf = make_buf(&[b"hello"]);
    let line = buf.head_line().next();
    let mut l = line;
    let mut o = 0;
    let result = advance_backward(&buf, &mut l, &mut o);
    assert!(result.is_none());
}

#[test]
fn test_matches_at_cross_line() {
    let buf = make_buf(&[b"hel", b"lo"]);
    let line = buf.head_line().next();
    let result = matches_at(&buf, b"hel", line, 0);
    assert_eq!(result, Some((line, 3)));
}

#[test]
fn test_matches_at_cross_line_with_newline() {
    let buf = make_buf(&[b"hel", b"lo"]);
    let line = buf.head_line().next();
    let result = matches_at(&buf, b"hel\nlo", line, 0);
    let expected = buf.next_line(line).unwrap();
    assert_eq!(result, Some((expected, 2)));
}

#[test]
fn test_matches_at_no_match() {
    let buf = make_buf(&[b"hello"]);
    let line = buf.head_line().next();
    let result = matches_at(&buf, b"world", line, 0);
    assert!(result.is_none());
}

#[test]
fn test_matches_at_past_end() {
    let buf = make_buf(&[b"hi"]);
    let line = buf.head_line().next();
    let result = matches_at(&buf, b"hello", line, 0);
    assert!(result.is_none());
}

#[test]
fn test_find_forward_regex_invalid_utf8() {
    let buf = make_buf(&[b"hello"]);
    let r = find_forward_regex(&buf, &[0xff, 0xfe], LineId(1), 0);
    assert!(r.is_none());
}

#[test]
fn test_find_forward_regex_invalid_pattern() {
    let buf = make_buf(&[b"hello"]);
    let r = find_forward_regex(&buf, b"[invalid", LineId(1), 0);
    assert!(r.is_none());
}

#[test]
fn test_find_backward_regex_invalid_utf8() {
    let buf = make_buf(&[b"hello"]);
    let r = find_backward_regex(&buf, &[0xff, 0xfe], LineId(1), 5);
    assert!(r.is_none());
}

#[test]
fn test_find_backward_regex_invalid_pattern() {
    let buf = make_buf(&[b"hello"]);
    let r = find_backward_regex(&buf, b"[invalid", LineId(1), 5);
    assert!(r.is_none());
}

#[test]
fn test_find_forward_regex_from_later_line() {
    let buf = make_buf(&[b"hello", b"world", b"test123"]);
    let r = find_forward_regex(&buf, b"\\d+", LineId(3), 0);
    assert_eq!(r, Some((LineId(3), 4)));
}

#[test]
fn test_find_backward_regex_from_later_line() {
    let buf = make_buf(&[b"hello", b"world", b"test123"]);
    let r = find_backward_regex(&buf, b"\\d+", LineId(3), 7);
    assert_eq!(r, Some((LineId(3), 4)));
}

#[test]
fn test_line_offset_for_past_end() {
    let buf = make_buf(&[b"hello"]);
    let result = line_offset_for(&buf, 100);
    assert!(result.is_none());
}

#[test]
fn test_buffer_text_empty() {
    let buf = Buffer::new(BufferId(0), "empty");
    let text = buffer_text(&buf);
    assert!(text.is_empty());
}

#[test]
fn test_find_forward_regex_case_insensitive_default() {
    let buf = make_buf(&[b"Hello World"]);
    let r = find_forward_regex(&buf, b"hello", LineId(1), 0);
    assert!(r.is_some(), "expected case-insensitive match by default");
}

#[test]
fn test_find_forward_regex_case_sensitive_when_exact() {
    let mut buf = make_buf(&[b"Hello World"]);
    buf.mode |= Mode::EXACT;
    let r = find_forward_regex(&buf, b"hello", LineId(1), 0);
    assert!(r.is_none(), "expected case-sensitive miss with MDEXACT");
    let r = find_forward_regex(&buf, b"Hello", LineId(1), 0);
    assert!(r.is_some(), "expected case-sensitive hit with MDEXACT");
}

#[test]
fn test_find_backward_regex_case_insensitive_default() {
    let buf = make_buf(&[b"Hello World"]);
    let r = find_backward_regex(&buf, b"WORLD", LineId(1), 11);
    assert!(r.is_some(), "expected case-insensitive backward match");
}

#[test]
fn test_is_regex_with_magic() {
    let mut buf = Buffer::new(BufferId(0), "test");
    buf.mode |= Mode::MAGIC;
    assert!(is_regex(&buf));
}
