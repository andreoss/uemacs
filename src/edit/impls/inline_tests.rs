use super::*;
use crate::display::Display;
use crate::terminal::mock::MockTerminal;

#[test]
fn test_minibuffer_readline_inline_enter() {
    let ed = Editor::new();
    let mut term = MockTerminal::new();
    term.input_keys = vec![Key::Char('a'), Key::Enter];
    let mut display = Display::with_size(24, 80);
    let result = ed
        .minibuffer_readline(&mut term, &mut display, "Test: ")
        .unwrap();
    assert_eq!(result, "a");
}

#[test]
fn test_minibuffer_readline_inline_ctrl_g() {
    let ed = Editor::new();
    let mut term = MockTerminal::new();
    term.input_keys = vec![Key::Char('a'), Key::Control('G')];
    let mut display = Display::with_size(24, 80);
    let result = ed
        .minibuffer_readline(&mut term, &mut display, "Test: ")
        .unwrap();
    assert_eq!(result, "");
}

#[test]
fn test_minibuffer_readline_inline_backspace() {
    let ed = Editor::new();
    let mut term = MockTerminal::new();
    term.input_keys = vec![Key::Char('a'), Key::Char('b'), Key::Backspace, Key::Enter];
    let mut display = Display::with_size(24, 80);
    let result = ed
        .minibuffer_readline(&mut term, &mut display, "Test: ")
        .unwrap();
    assert_eq!(result, "a");
}

#[test]
fn test_minibuffer_readline_tab_cycles_candidates() {
    let ed = Editor::new();
    let mut term = MockTerminal::new();
    term.input_keys = vec![
        Key::Char('C'),
        Key::Char('a'),
        Key::Char('r'),
        Key::Char('g'),
        Key::Char('o'),
        Key::Tab,
        Key::Tab,
        Key::Tab,
        Key::Tab,
        Key::Enter,
    ];
    let mut display = Display::with_size(24, 80);
    let result = ed
        .minibuffer_readline(&mut term, &mut display, "Find file: ")
        .unwrap();
    assert_eq!(result, "Cargo.lock");
    assert_eq!(term.beep_count, 0);
}

#[test]
fn test_minibuffer_readline_tab_second_candidate() {
    let ed = Editor::new();
    let mut term = MockTerminal::new();
    term.input_keys = vec![
        Key::Char('C'),
        Key::Char('a'),
        Key::Char('r'),
        Key::Char('g'),
        Key::Char('o'),
        Key::Char('.'),
        Key::Tab,
        Key::Tab,
        Key::Enter,
    ];
    let mut display = Display::with_size(24, 80);
    let result = ed
        .minibuffer_readline(&mut term, &mut display, "Find file: ")
        .unwrap();
    assert_eq!(result, "Cargo.toml");
}

#[test]
fn test_minibuffer_readline_tab_typing_resets_cycle() {
    let ed = Editor::new();
    let mut term = MockTerminal::new();
    term.input_keys = vec![
        Key::Char('C'),
        Key::Char('a'),
        Key::Char('r'),
        Key::Char('g'),
        Key::Char('o'),
        Key::Tab,
        Key::Char('t'),
        Key::Tab,
        Key::Enter,
    ];
    let mut display = Display::with_size(24, 80);
    let result = ed
        .minibuffer_readline(&mut term, &mut display, "Find file: ")
        .unwrap();
    assert_eq!(result, "Cargo.toml");
}

#[test]
fn test_minibuffer_readline_inline_no_input() {
    let ed = Editor::new();
    let mut term = MockTerminal::new();
    term.input_keys = vec![];
    let mut display = Display::with_size(24, 80);
    let result = ed
        .minibuffer_readline(&mut term, &mut display, "Test: ")
        .unwrap();
    assert_eq!(result, "");
}

#[test]
fn test_forward_line_goal_exceeds_line_len() {
    let mut ed = Editor::new();
    let buf_id = ed.create_buffer("main");
    let buf = ed.buffers.get_mut(buf_id).unwrap();
    let l1 = buf.insert_after(LineId(0), crate::line::Line::new());
    let l2 = buf.insert_after(l1, crate::line::Line::new());
    buf.line_mut(l1).unwrap().text = b"hello world".to_vec();
    buf.line_mut(l2).unwrap().text = b"hi".to_vec();
    ed.create_window(buf_id, l1);
    ed.current_window_mut().unwrap().set_dot(l1, LineOffset(10));
    ed.cur_goal = 10;
    ForwardLine.execute(&mut ed, false, 1).unwrap();
    assert_eq!(ed.current_window().unwrap().dot_offset.0, 2);
}

#[test]
fn test_kill_line_beyond_buffer_error() {
    let mut ed = Editor::new();
    let buf_id = ed.create_buffer("main");
    let buf = ed.buffers.get_mut(buf_id).unwrap();
    let l1 = buf.insert_after(LineId(0), crate::line::Line::new());
    buf.line_mut(l1).unwrap().text = b"hello".to_vec();
    ed.create_window(buf_id, l1);
    ed.current_window_mut().unwrap().set_dot(l1, LineOffset(0));
    let result = KillLine.execute(&mut ed, true, 2);
    assert!(result.is_err());
}

#[test]
fn test_entab_line_non_tab_stop_run() {
    let mut ed = Editor::new();
    let buf_id = ed.create_buffer("main");
    let buf = ed.buffers.get_mut(buf_id).unwrap();
    let l1 = buf.insert_after(LineId(0), crate::line::Line::new());
    buf.line_mut(l1).unwrap().text = b"ab          cd".to_vec();
    ed.create_window(buf_id, l1);
    ed.current_window_mut().unwrap().set_dot(l1, LineOffset(0));
    EntabLine.execute(&mut ed, false, 1).unwrap();
    let result = &ed.buffers.get(buf_id).unwrap().line(l1).unwrap().text;
    assert!(result.contains(&b'\t'));
}

#[test]
fn test_goto_bop_from_middle() {
    let mut ed = Editor::new();
    let buf_id = ed.create_buffer("main");
    let buf = ed.buffers.get_mut(buf_id).unwrap();
    let l1 = buf.insert_after(LineId(0), crate::line::Line::new());
    let l2 = buf.insert_after(l1, crate::line::Line::new());
    buf.line_mut(l1).unwrap().text = b"line1".to_vec();
    buf.line_mut(l2).unwrap().text = b"line2".to_vec();
    ed.create_window(buf_id, l1);
    ed.current_window_mut().unwrap().set_dot(l2, LineOffset(3));
    GotoBop.execute(&mut ed, false, 1).unwrap();
    assert_eq!(ed.current_window().unwrap().dot_line, l1);
}

#[test]
fn test_forward_line_with_large_goal() {
    let mut ed = Editor::new();
    let buf_id = ed.create_buffer("main");
    let buf = ed.buffers.get_mut(buf_id).unwrap();
    let l1 = buf.insert_after(LineId(0), crate::line::Line::new());
    let l2 = buf.insert_after(l1, crate::line::Line::new());
    buf.line_mut(l1).unwrap().text = b"short".to_vec();
    buf.line_mut(l2).unwrap().text = b"hi".to_vec();
    ed.create_window(buf_id, l1);
    ed.current_window_mut().unwrap().set_dot(l1, LineOffset(0));
    ed.cur_goal = 100;
    ed.last_flag = CmdFlags::LINE_MOVE;
    ForwardLine.execute(&mut ed, false, 1).unwrap();
    assert_eq!(ed.current_window().unwrap().dot_offset.0, 2);
}

#[test]
fn test_set_var_fillcol() {
    let mut ed = Editor::new();
    ed.set_var("fillcol", 72);
    assert_eq!(ed.fillcol, 72);
}

#[test]
fn test_forward_line_goal_exceeds_length() {
    let mut ed = Editor::new();
    let buf_id = ed.create_buffer("main");
    let buf = ed.buffers.get_mut(buf_id).unwrap();
    let l1 = buf.insert_after(LineId(0), crate::line::Line::new());
    let l2 = buf.insert_after(l1, crate::line::Line::new());
    buf.line_mut(l1).unwrap().text = b"hello world".to_vec();
    buf.line_mut(l2).unwrap().text = b"hi".to_vec();
    ed.create_window(buf_id, l1);
    ed.current_window_mut().unwrap().set_dot(l1, LineOffset(0));
    ed.cur_goal = 100;
    ed.last_flag = CmdFlags::LINE_MOVE;
    ForwardLine.execute(&mut ed, false, 1).unwrap();
    let win = ed.current_window().unwrap();
    assert_eq!(win.dot_offset.0, 2);
}
