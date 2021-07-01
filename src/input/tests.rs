use super::*;
use crate::terminal::mock::MockTerminal;

#[test]
fn test_getkey_char() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Char('x'));
    let ev = getkey(&mut term);
    assert!(!ev.f);
    assert_eq!(ev.n, 1);
    assert_eq!(ev.key, Key::Char('x'));
}

#[test]
fn test_getkey_control() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Control('G'));
    let ev = getkey(&mut term);
    assert!(!ev.f);
    assert_eq!(ev.key, Key::Control('G'));
}

#[test]
fn test_getkey_universal_arg_default() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Control('U'));
    term.input_keys.push(Key::Char('x'));
    let ev = getkey(&mut term);
    assert!(ev.f);
    assert_eq!(ev.n, 4);
    assert_eq!(ev.key, Key::Char('x'));
}

#[test]
fn test_getkey_universal_arg_digit() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Control('U'));
    term.input_keys.push(Key::Char('1'));
    term.input_keys.push(Key::Char('2'));
    term.input_keys.push(Key::Char('x'));
    let ev = getkey(&mut term);
    assert!(ev.f);
    assert_eq!(ev.n, 12);
    assert_eq!(ev.key, Key::Char('x'));
}

#[test]
fn test_getkey_universal_arg_negative() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Control('U'));
    term.input_keys.push(Key::Char('-'));
    term.input_keys.push(Key::Char('x'));
    let ev = getkey(&mut term);
    assert!(ev.f);
    assert_eq!(ev.n, 1);
    assert_eq!(ev.key, Key::Char('x'));
}

#[test]
fn test_getkey_meta_digit() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Meta('5'));
    term.input_keys.push(Key::Char('x'));
    let ev = getkey(&mut term);
    assert!(ev.f);
    assert_eq!(ev.n, 5);
    assert_eq!(ev.key, Key::Char('x'));
}

#[test]
fn test_getkey_meta_digits() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Meta('1'));
    term.input_keys.push(Key::Char('2'));
    term.input_keys.push(Key::Char('0'));
    term.input_keys.push(Key::Char('x'));
    let ev = getkey(&mut term);
    assert!(ev.f);
    assert_eq!(ev.n, 120);
    assert_eq!(ev.key, Key::Char('x'));
}

#[test]
fn test_getkey_empty_input() {
    let mut term = MockTerminal::new();
    let ev = getkey(&mut term);
    assert!(!ev.f);
    assert_eq!(ev.n, 1);
}

#[test]
fn test_getkey_universal_arg_twice() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Control('U'));
    term.input_keys.push(Key::Control('U'));
    term.input_keys.push(Key::Char('x'));
    let ev = getkey(&mut term);
    assert!(ev.f);
    assert_eq!(ev.n, 16);
    assert_eq!(ev.key, Key::Char('x'));
}

#[test]
fn test_getkey_escape_returns_control_bracket() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Escape);
    let ev = getkey(&mut term);
    assert!(!ev.f);
    assert_eq!(ev.n, 1);
    assert_eq!(ev.key, Key::Control('['));
}

#[test]
fn test_getkey_meta_non_digit() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Meta('x'));
    let ev = getkey(&mut term);
    assert!(!ev.f);
    assert_eq!(ev.n, 1);
    assert_eq!(ev.key, Key::Meta('x'));
}

#[test]
fn test_getkey_meta_negative() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Meta('-'));
    term.input_keys.push(Key::Char('x'));
    let ev = getkey(&mut term);
    assert!(ev.f);
    assert_eq!(ev.n, 1);
    assert_eq!(ev.key, Key::Char('x'));
}

#[test]
fn test_getkey_universal_arg_meta_followup() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Control('U'));
    term.input_keys.push(Key::Meta('x'));
    let ev = getkey(&mut term);
    assert!(ev.f);
    assert_eq!(ev.n, 4);
    assert_eq!(ev.key, Key::Meta('x'));
}

#[test]
fn test_getkey_universal_arg_negative_digit() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Control('U'));
    term.input_keys.push(Key::Char('-'));
    term.input_keys.push(Key::Char('5'));
    term.input_keys.push(Key::Char('x'));
    let ev = getkey(&mut term);
    assert!(ev.f);
    assert_eq!(ev.n, 5);
    assert_eq!(ev.key, Key::Char('x'));
}

#[test]
fn test_getkey_empty_input_returns_unknown() {
    let mut term = MockTerminal::new();
    let ev = getkey(&mut term);
    assert_eq!(ev.key, Key::Unknown(0));
}

#[test]
fn test_getkey_universal_arg_no_followup() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Control('U'));
    let ev = getkey(&mut term);
    assert!(ev.f);
    assert_eq!(ev.n, 4);
}

#[test]
fn test_getkey_ctrl_u_with_digits() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Control('U'));
    term.input_keys.push(Key::Char('2'));
    term.input_keys.push(Key::Char('3'));
    let ev = getkey(&mut term);
    assert!(ev.f);
    assert_eq!(ev.n, 23);
}

#[test]
fn test_getkey_ctrl_u_cancels_on_ctrl_g() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Control('U'));
    term.input_keys.push(Key::Control('G'));
    let ev = getkey(&mut term);
    assert!(ev.f);
    assert_eq!(ev.n, 4);
    assert_eq!(ev.key, Key::Control('G'));
}

#[test]
fn test_getkey_meta_negative_with_digits() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Meta('-'));
    term.input_keys.push(Key::Char('3'));
    term.input_keys.push(Key::Char('x'));
    let ev = getkey(&mut term);
    assert!(ev.f);
    assert_eq!(ev.n, 3);
    assert_eq!(ev.key, Key::Char('x'));
}

#[test]
fn test_getkey_meta_digit_stream_ends() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Meta('5'));
    let ev = getkey(&mut term);
    assert!(ev.f);
    assert_eq!(ev.n, 5);
    assert_eq!(ev.key, Key::Meta('5'));
}

#[test]
fn test_getkey_ctrl_u_negative() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Control('U'));
    term.input_keys.push(Key::Char('-'));
    term.input_keys.push(Key::Char('3'));
    term.input_keys.push(Key::Char('x'));
    let ev = getkey(&mut term);
    assert!(ev.f);
    assert_eq!(ev.n, 3);
    assert_eq!(ev.key, Key::Char('x'));
}

#[test]
fn test_getkey_ctrl_u_negative_no_digit() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Control('U'));
    term.input_keys.push(Key::Char('-'));
    term.input_keys.push(Key::Char('x'));
    let ev = getkey(&mut term);
    assert!(ev.f);
    assert_eq!(ev.n, 1);
    assert_eq!(ev.key, Key::Char('x'));
}

#[test]
fn test_getkey_ctrl_u_followed_by_meta() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Control('U'));
    term.input_keys.push(Key::Meta('x'));
    let ev = getkey(&mut term);
    assert!(ev.f);
    assert_eq!(ev.n, 4);
    assert_eq!(ev.key, Key::Meta('x'));
}

#[test]
fn test_getkey_ctrl_u_multiple() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Control('U'));
    term.input_keys.push(Key::Control('U'));
    term.input_keys.push(Key::Control('U'));
    term.input_keys.push(Key::Char('x'));
    let ev = getkey(&mut term);
    assert!(ev.f);
    assert_eq!(ev.n, 64);
    assert_eq!(ev.key, Key::Char('x'));
}

#[test]
fn test_getkey_escape_returns_control_bracket_extended() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Escape);
    let ev = getkey(&mut term);
    assert_eq!(ev.key, Key::Control('['));
}

#[test]
fn test_getkey_ctrl_u_with_no_followup_returns_default() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Control('U'));
    let ev = getkey(&mut term);
    assert!(ev.f);
    assert_eq!(ev.n, 4);
}

#[test]
fn test_getkey_meta_digit_dash_n_zero() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Meta('5'));
    term.input_keys.push(Key::Char('-'));
    let ev = getkey(&mut term);
    assert!(ev.f);
    assert_eq!(ev.n, 5);
    assert_eq!(ev.key, Key::Char('-'));
}

#[test]
fn test_getkey_meta_digit_stream_ends_no_followup() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Meta('3'));
    let ev = getkey(&mut term);
    assert!(ev.f);
    assert_eq!(ev.n, 3);
    assert_eq!(ev.key, Key::Meta('3'));
}

#[test]
fn test_getkey_meta_negative_stream_ends() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Meta('-'));
    let ev = getkey(&mut term);
    assert!(ev.f);
    assert_eq!(ev.n, 1);
    assert_eq!(ev.key, Key::Meta('-'));
}

#[test]
fn test_getkey_ctrl_u_no_followup_returns_unknown() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Control('U'));
    let ev = getkey(&mut term);
    assert!(ev.f);
    assert_eq!(ev.n, 4);
    assert_eq!(ev.key, Key::Unknown(0));
}

#[test]
fn test_getkey_no_input_returns_unknown() {
    let mut term = MockTerminal::new();
    let ev = getkey(&mut term);
    assert!(!ev.f);
    assert_eq!(ev.n, 1);
    assert_eq!(ev.key, Key::Unknown(0));
}

#[test]
fn test_getkey_meta_negative_followed_by_dash() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Meta('-'));
    term.input_keys.push(Key::Char('-'));
    term.input_keys.push(Key::Char('x'));
    let ev = getkey(&mut term);
    assert!(ev.f);
    assert_eq!(ev.n, 1);
    assert_eq!(ev.key, Key::Char('x'));
}

#[test]
fn test_getkey_meta_digit_then_non_digit() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Meta('3'));
    term.input_keys.push(Key::Char('x'));
    let ev = getkey(&mut term);
    assert!(ev.f);
    assert_eq!(ev.n, 3);
    assert_eq!(ev.key, Key::Char('x'));
}

#[test]
fn test_getkey_meta_digit_stream_ends_with_non_digit() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Meta('5'));
    term.input_keys.push(Key::Char('y'));
    let ev = getkey(&mut term);
    assert!(ev.f);
    assert_eq!(ev.n, 5);
    assert_eq!(ev.key, Key::Char('y'));
}

#[test]
fn test_getkey_ctrl_u_with_non_digit_non_meta() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Control('U'));
    term.input_keys.push(Key::Char('x'));
    let ev = getkey(&mut term);
    assert!(ev.f);
    assert_eq!(ev.n, 4);
    assert_eq!(ev.key, Key::Char('x'));
}

#[test]
fn test_getkey_ctrl_u_digit_then_non_digit() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Control('U'));
    term.input_keys.push(Key::Char('2'));
    term.input_keys.push(Key::Char('x'));
    let ev = getkey(&mut term);
    assert!(ev.f);
    assert_eq!(ev.n, 2);
    assert_eq!(ev.key, Key::Char('x'));
}

#[test]
fn test_getkey_meta_digit_then_non_digit_non_meta() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Meta('3'));
    term.input_keys.push(Key::Char('x'));
    let ev = getkey(&mut term);
    assert!(ev.f);
    assert_eq!(ev.n, 3);
    assert_eq!(ev.key, Key::Char('x'));
}

#[test]
fn test_getkey_meta_digit_stream_ends_no_followup_v2() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Meta('5'));
    let ev = getkey(&mut term);
    assert!(ev.f);
    assert_eq!(ev.n, 5);
    assert_eq!(ev.key, Key::Meta('5'));
}

#[test]
fn test_getkey_ctrl_u_no_followup_returns_unknown_v2() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Control('U'));
    let ev = getkey(&mut term);
    assert!(ev.f);
    assert_eq!(ev.n, 4);
    assert_eq!(ev.key, Key::Unknown(0));
}

#[test]
fn test_getkey_ctrl_u_meta_followup() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Control('U'));
    term.input_keys.push(Key::Meta('x'));
    let ev = getkey(&mut term);
    assert!(ev.f);
    assert_eq!(ev.n, 4);
    assert_eq!(ev.key, Key::Meta('x'));
}

#[test]
fn test_getkey_no_input_v2() {
    let mut term = MockTerminal::new();
    let ev = getkey(&mut term);
    assert!(!ev.f);
    assert_eq!(ev.n, 1);
    assert_eq!(ev.key, Key::Unknown(0));
}

#[test]
fn test_getkey_escape_v2() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Escape);
    let ev = getkey(&mut term);
    assert!(!ev.f);
    assert_eq!(ev.n, 1);
    assert_eq!(ev.key, Key::Control('['));
}

#[test]
fn test_getkey_meta_non_digit_v2() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Meta('x'));
    let ev = getkey(&mut term);
    assert!(!ev.f);
    assert_eq!(ev.n, 1);
    assert_eq!(ev.key, Key::Meta('x'));
}

#[test]
fn test_getkey_meta_digit_then_non_digit_v2() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Meta('3'));
    term.input_keys.push(Key::Char('x'));
    let ev = getkey(&mut term);
    assert!(ev.f);
    assert_eq!(ev.n, 3);
    assert_eq!(ev.key, Key::Char('x'));
}

#[test]
fn test_getkey_meta_digit_stream_ends_v2() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Meta('5'));
    let ev = getkey(&mut term);
    assert!(ev.f);
    assert_eq!(ev.n, 5);
    assert_eq!(ev.key, Key::Meta('5'));
}

#[test]
fn test_getkey_ctrl_u_no_followup_v3() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Control('U'));
    let ev = getkey(&mut term);
    assert!(ev.f);
    assert_eq!(ev.n, 4);
    assert_eq!(ev.key, Key::Unknown(0));
}

#[test]
fn test_getkey_ctrl_u_digit_then_non_digit_v2() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Control('U'));
    term.input_keys.push(Key::Char('2'));
    term.input_keys.push(Key::Char('x'));
    let ev = getkey(&mut term);
    assert!(ev.f);
    assert_eq!(ev.n, 2);
    assert_eq!(ev.key, Key::Char('x'));
}

#[test]
fn test_getkey_ctrl_u_meta_followup_v2() {
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Control('U'));
    term.input_keys.push(Key::Meta('x'));
    let ev = getkey(&mut term);
    assert!(ev.f);
    assert_eq!(ev.n, 4);
    assert_eq!(ev.key, Key::Meta('x'));
}
