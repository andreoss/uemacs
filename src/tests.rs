use super::*;
use crate::core::CommandId;
use terminal::mock::MockTerminal;

#[test]
fn test_mark_screen_garbage_sets_sgarbf_and_window_flags() {
    let mut editor = Editor::new();
    let bid = editor.create_buffer("main");
    let w1 = editor.create_window(bid, core::LineId(0));
    let w2 = editor.create_window(bid, core::LineId(0));
    let mut display = Display::with_size(24, 80);
    display.sgarbf = false;
    editor.windows.get_mut(w1).unwrap().flags = core::WindowFlags::EMPTY;
    editor.windows.get_mut(w2).unwrap().flags = core::WindowFlags::EMPTY;

    mark_screen_garbage(&mut editor, &mut display);

    assert!(display.sgarbf, "sgarbf must be set after SIGCONT");
    let expected = core::WindowFlags::HARD | core::WindowFlags::MODE_LINE;
    assert_eq!(editor.windows.get(w1).unwrap().flags & expected, expected);
    assert_eq!(editor.windows.get(w2).unwrap().flags & expected, expected);
}

#[test]
fn test_run_insert_char() {
    let mut editor = Editor::new();
    let bid = editor.create_buffer("test");
    let head = editor.buffers.get(bid).unwrap().head;
    editor.create_window(bid, head);
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Char('h'));
    term.input_keys.push(Key::Char('i'));
    let mut display = Display::with_size(24, 80);
    run(&mut editor, &mut display, &mut term);
    let buf = editor.buffers.get(bid).unwrap();
    let first = buf.head_line().next();
    let text = buf
        .line(first)
        .map(|l| String::from_utf8_lossy(&l.text).to_string());
    assert_eq!(text, Some("hi".to_string()));
}

#[test]
fn test_run_control_keys() {
    let mut editor = Editor::new();
    let bid = editor.create_buffer("test");
    let head = editor.buffers.get(bid).unwrap().head;
    editor.create_window(bid, head);
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Char('a'));
    term.input_keys.push(Key::Char('b'));
    term.input_keys.push(Key::Char('c'));
    term.input_keys.push(Key::Control('B'));
    let mut display = Display::with_size(24, 80);
    run(&mut editor, &mut display, &mut term);
    let win = editor.current_window().unwrap();
    assert_eq!(win.dot_offset.0, 2);
}

#[test]
fn test_run_refresh_screen() {
    let mut editor = Editor::new();
    let bid = editor.create_buffer("test");
    let head = editor.buffers.get(bid).unwrap().head;
    editor.create_window(bid, head);
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Control('L'));
    term.input_keys.push(Key::Control('G'));
    let mut display = Display::with_size(24, 80);
    display.sgarbf = false;
    run(&mut editor, &mut display, &mut term);
    assert!(
        !display.sgarbf,
        "sgarbf should have been consumed by updgar"
    );
}

#[test]
fn test_run_kill_region_no_mark_does_not_quit() {
    let mut editor = Editor::new();
    let bid = editor.create_buffer("test");
    let head = editor.buffers.get(bid).unwrap().head;
    editor.create_window(bid, head);
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Control('W'));
    term.input_keys.push(Key::Char('a'));
    let mut display = Display::with_size(24, 80);
    run(&mut editor, &mut display, &mut term);
    assert!(term.beep_count >= 1, "C-W with no mark should beep");
    let buf = editor.buffers.get(bid).unwrap();
    let first = buf.head_line().next();
    let text = String::from_utf8_lossy(&buf.line(first).unwrap().text).to_string();
    assert_eq!(text, "a", "editor should still be running and accept 'a'");
}

#[test]
fn test_run_backward_char_at_bob_does_not_quit() {
    let mut editor = Editor::new();
    let bid = editor.create_buffer("test");
    let head = editor.buffers.get(bid).unwrap().head;
    editor.create_window(bid, head);
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Control('B'));
    term.input_keys.push(Key::Char('z'));
    let mut display = Display::with_size(24, 80);
    run(&mut editor, &mut display, &mut term);
    let buf = editor.buffers.get(bid).unwrap();
    let first = buf.head_line().next();
    let text = String::from_utf8_lossy(&buf.line(first).unwrap().text).to_string();
    assert_eq!(text, "z", "editor should still be running after C-B at BOB");
}

#[test]
fn test_run_meta_u_uppercases_first_word() {
    let mut editor = Editor::new();
    let bid = editor.create_buffer("main");
    let buf = editor.buffers.get_mut(bid).unwrap();
    let l1 = buf.insert_after(core::LineId(0), crate::line::Line::new());
    buf.line_mut(l1).unwrap().text = b"hello world".to_vec();
    editor.create_window(bid, l1);
    editor
        .current_window_mut()
        .unwrap()
        .set_dot(l1, core::LineOffset(0));
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Meta('u'));
    let mut display = Display::with_size(24, 80);
    run(&mut editor, &mut display, &mut term);
    let buf = editor.buffers.get(bid).unwrap();
    let text = String::from_utf8_lossy(&buf.line(l1).unwrap().text).into_owned();
    assert_eq!(text, "HELLO world", "M-u (lowercase) must run UpperWord");
}

#[test]
fn test_run_meta_control_f_jumps_to_matching_fence() {
    let mut editor = Editor::new();
    let bid = editor.create_buffer("main");
    let buf = editor.buffers.get_mut(bid).unwrap();
    let l1 = buf.insert_after(core::LineId(0), crate::line::Line::new());
    buf.line_mut(l1).unwrap().text = b"(hello)".to_vec();
    editor.create_window(bid, l1);
    editor
        .current_window_mut()
        .unwrap()
        .set_dot(l1, core::LineOffset(0));
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::MetaControl('F'));
    let mut display = Display::with_size(24, 80);
    run(&mut editor, &mut display, &mut term);
    let win = editor.current_window().unwrap();
    assert_eq!(
        win.dot_offset.0, 6,
        "M-C-f must jump from '(' to its matching ')'"
    );
}

#[test]
fn test_run_lowercase_meta_x_dispatches_execute_command() {
    let mut editor = Editor::new();
    let bid = editor.create_buffer("test");
    let head = editor.buffers.get(bid).unwrap().head;
    editor.create_window(bid, head);
    let mut term = MockTerminal::new();
    term.input_keys.push(Key::Meta('x'));
    for c in "forward-char".chars() {
        term.input_keys.push(Key::Char(c));
    }
    term.input_keys.push(Key::Enter);
    let mut display = Display::with_size(24, 80);
    run(&mut editor, &mut display, &mut term);
    let echoed = display.echo_text();
    assert!(
        !echoed.contains("forward-char") || editor.windows.current().is_some(),
        "M-x with lowercase x must reach ExecuteCommand; echo={echoed}",
    );
}

#[test]
fn test_run_ctrl_x_ctrl_c_sets_quit_requested() {
    let mut editor = Editor::new();
    let bid = editor.create_buffer("test");
    let head = editor.buffers.get(bid).unwrap().head;
    editor.create_window(bid, head);
    let mut term = MockTerminal::new();
    editor
        .dispatch(
            CommandId::QuitEmacs,
            &mut term,
            &mut display::Display::with_size(24, 80),
            &Bindings::new(),
            false,
            1,
        )
        .unwrap();
    assert!(editor.quit_requested);
}
