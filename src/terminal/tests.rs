use super::*;

#[test]
fn test_mock_open_close() {
    let mut term = mock::MockTerminal::new();
    term.open().unwrap();
    term.close().unwrap();
    assert_eq!(term.output, vec!["open", "close"]);
}

#[test]
fn test_mock_dimensions() {
    let term = mock::MockTerminal::new();
    assert_eq!(term.dimensions(), (24, 80));
}

#[test]
fn test_mock_move_to() {
    let mut term = mock::MockTerminal::new();
    term.move_to(5, 10);
    assert_eq!(term.cursor, (5, 10));
}

#[test]
fn test_mock_put_char() {
    let mut term = mock::MockTerminal::new();
    term.put_char('a');
    assert_eq!(term.output, vec!["put_char(a)"]);
}

#[test]
fn test_mock_get_key() {
    let mut term = mock::MockTerminal::new();
    term.input_keys.push(Key::Char('x'));
    assert_eq!(term.get_key(), Some(Key::Char('x')));
    assert_eq!(term.get_key(), None);
}

#[test]
fn test_key_to_keycode() {
    let kc: KeyCode = Key::Char('a').into();
    assert_eq!(kc, KeyCode(u32::from(b'a')));
}

#[test]
fn test_key_backspace_to_keycode() {
    let kc: KeyCode = Key::Backspace.into();
    assert_eq!(kc, KeyCode(0x7f));
}

#[test]
fn test_key_tab_to_keycode() {
    let kc: KeyCode = Key::Tab.into();
    assert_eq!(kc, KeyCode(u32::from(b'\t')));
}

#[test]
fn test_key_enter_to_keycode() {
    let kc: KeyCode = Key::Enter.into();
    assert_eq!(kc, KeyCode(u32::from(b'\r')));
}

#[test]
fn test_key_escape_to_keycode() {
    let kc: KeyCode = Key::Escape.into();
    assert_eq!(kc, KeyCode(0x1b));
}

#[test]
fn test_key_function_to_keycode() {
    let kc: KeyCode = Key::Function(1).into();
    assert_eq!(kc, KeyCode(0x10b));
}

#[test]
fn test_key_unknown_to_keycode() {
    let kc: KeyCode = Key::Unknown(999).into();
    assert_eq!(kc, KeyCode(999));
}

#[test]
fn test_key_arrow_to_keycode() {
    assert_eq!(KeyCode::from(Key::Up), KeyCode(0x101));
    assert_eq!(KeyCode::from(Key::Down), KeyCode(0x102));
    assert_eq!(KeyCode::from(Key::Left), KeyCode(0x103));
    assert_eq!(KeyCode::from(Key::Right), KeyCode(0x104));
}

#[test]
fn test_key_navigation_to_keycode() {
    assert_eq!(KeyCode::from(Key::PageUp), KeyCode(0x105));
    assert_eq!(KeyCode::from(Key::PageDown), KeyCode(0x106));
    assert_eq!(KeyCode::from(Key::Home), KeyCode(0x107));
    assert_eq!(KeyCode::from(Key::End), KeyCode(0x108));
    assert_eq!(KeyCode::from(Key::Delete), KeyCode(0x109));
    assert_eq!(KeyCode::from(Key::Insert), KeyCode(0x10a));
}

#[test]
fn test_key_control_meta_to_keycode() {
    assert_eq!(KeyCode::from(Key::Control('A')), KeyCode(CONTROL | 0x01));
    assert_eq!(
        KeyCode::from(Key::Meta('x')),
        KeyCode(META | u32::from(b'x'))
    );
}

#[test]
fn test_mock_beep_count() {
    let mut term = mock::MockTerminal::new();
    assert_eq!(term.beep_count, 0);
    term.beep();
    assert_eq!(term.beep_count, 1);
    term.beep();
    assert_eq!(term.beep_count, 2);
}

#[test]
fn test_mock_set_reverse() {
    let mut term = mock::MockTerminal::new();
    assert!(!term.reverse);
    term.set_reverse(true);
    assert!(term.reverse);
    assert_eq!(term.output, vec!["set_reverse(true)"]);
    term.set_reverse(false);
    assert!(!term.reverse);
    assert_eq!(term.output, vec!["set_reverse(true)", "set_reverse(false)"]);
}

#[test]
fn test_mock_clear_eol() {
    let mut term = mock::MockTerminal::new();
    term.clear_eol();
    assert_eq!(term.output, vec!["clear_eol"]);
}

#[test]
fn test_mock_flush() {
    let mut term = mock::MockTerminal::new();
    term.flush().unwrap();
    assert_eq!(term.output, vec!["flush"]);
}

#[test]
fn test_mock_multiple_keys() {
    let mut term = mock::MockTerminal::new();
    term.input_keys.push(Key::Char('a'));
    term.input_keys.push(Key::Char('b'));
    term.input_keys.push(Key::Char('c'));
    assert_eq!(term.get_key(), Some(Key::Char('a')));
    assert_eq!(term.get_key(), Some(Key::Char('b')));
    assert_eq!(term.get_key(), Some(Key::Char('c')));
    assert_eq!(term.get_key(), None);
}

#[test]
fn test_mock_input_index_reset() {
    let mut term = mock::MockTerminal::new();
    term.input_keys.push(Key::Char('x'));
    assert_eq!(term.get_key(), Some(Key::Char('x')));
    term.input_index = 0;
    assert_eq!(term.get_key(), Some(Key::Char('x')));
}

#[cfg(unix)]
mod posix_tests {
    use super::super::posix::PosixTerminal;
    use super::super::{Key, TerminalBackend};

    #[test]
    fn test_csi_single_digit_home() {
        let mut term = PosixTerminal::with_input(b"\x1b[1~");
        assert_eq!(term.get_key(), Some(Key::Home));
    }

    #[test]
    fn test_csi_single_digit_insert() {
        let mut term = PosixTerminal::with_input(b"\x1b[2~");
        assert_eq!(term.get_key(), Some(Key::Insert));
    }

    #[test]
    fn test_csi_single_digit_delete() {
        let mut term = PosixTerminal::with_input(b"\x1b[3~");
        assert_eq!(term.get_key(), Some(Key::Delete));
    }

    #[test]
    fn test_csi_single_digit_end() {
        let mut term = PosixTerminal::with_input(b"\x1b[4~");
        assert_eq!(term.get_key(), Some(Key::End));
    }

    #[test]
    fn test_csi_single_digit_pageup() {
        let mut term = PosixTerminal::with_input(b"\x1b[5~");
        assert_eq!(term.get_key(), Some(Key::PageUp));
    }

    #[test]
    fn test_csi_single_digit_pagedown() {
        let mut term = PosixTerminal::with_input(b"\x1b[6~");
        assert_eq!(term.get_key(), Some(Key::PageDown));
    }

    #[test]
    fn test_csi_multi_digit_f1() {
        let mut term = PosixTerminal::with_input(b"\x1b[11~");
        assert_eq!(term.get_key(), Some(Key::Function(1)));
    }

    #[test]
    fn test_csi_multi_digit_f2() {
        let mut term = PosixTerminal::with_input(b"\x1b[12~");
        assert_eq!(term.get_key(), Some(Key::Function(2)));
    }

    #[test]
    fn test_csi_multi_digit_f5() {
        let mut term = PosixTerminal::with_input(b"\x1b[15~");
        assert_eq!(term.get_key(), Some(Key::Function(5)));
    }

    #[test]
    fn test_csi_multi_digit_f6() {
        let mut term = PosixTerminal::with_input(b"\x1b[17~");
        assert_eq!(term.get_key(), Some(Key::Function(6)));
    }

    #[test]
    fn test_csi_multi_digit_f9() {
        let mut term = PosixTerminal::with_input(b"\x1b[20~");
        assert_eq!(term.get_key(), Some(Key::Function(9)));
    }

    #[test]
    fn test_csi_multi_digit_f12() {
        let mut term = PosixTerminal::with_input(b"\x1b[24~");
        assert_eq!(term.get_key(), Some(Key::Function(12)));
    }

    #[test]
    fn test_csi_arrow_keys() {
        let mut term = PosixTerminal::with_input(b"\x1b[A");
        assert_eq!(term.get_key(), Some(Key::Up));
        let mut term = PosixTerminal::with_input(b"\x1b[B");
        assert_eq!(term.get_key(), Some(Key::Down));
        let mut term = PosixTerminal::with_input(b"\x1b[C");
        assert_eq!(term.get_key(), Some(Key::Right));
        let mut term = PosixTerminal::with_input(b"\x1b[D");
        assert_eq!(term.get_key(), Some(Key::Left));
    }

    #[test]
    fn test_escape_standalone_returns_escape() {
        let mut term = PosixTerminal::with_input(b"\x1b");
        assert_eq!(term.get_key(), Some(Key::Escape));
    }
}
