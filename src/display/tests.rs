use super::*;
use crate::core::{BufferId, LineOffset};
use crate::line::Line;
use crate::terminal::mock::MockTerminal;

fn make_env(text: &[&[u8]], rows: usize) -> (Display, Windows, Buffers, MockTerminal) {
    let term = MockTerminal::new();
    let display = Display::with_size(rows + 1, 80);
    let mut buffers = Buffers::new();
    let buf_id = {
        let buf = buffers.create("test");
        let mut last = buf.head;
        for &t in text {
            let id = buf.insert_after(last, {
                let mut l = Line::new();
                l.text = t.to_vec();
                l
            });
            last = id;
        }
        buf.id
    };
    let mut windows = Windows::new();
    let line1 = buffers.get(buf_id).unwrap().head_line().next();
    let win = windows.create(buf_id, line1);
    win.top_row = 0;
    win.n_rows = rows;
    win.flags = WindowFlags::HARD | WindowFlags::MODE_LINE;
    (display, windows, buffers, term)
}

#[test]
fn test_display_new() {
    let term = MockTerminal::new();
    let display = Display::new(&term);
    assert_eq!(display.nrows, 24);
    assert_eq!(display.ncols, 80);
    assert!(display.sgarbf);
}

#[test]
fn test_display_with_size() {
    let display = Display::with_size(10, 40);
    assert_eq!(display.nrows, 10);
    assert_eq!(display.ncols, 40);
    assert_eq!(display.vscreen.len(), 10);
    assert_eq!(display.vscreen[0].cells.len(), 40);
}

#[test]
fn test_update_clears_sgarbf() {
    let (mut display, mut windows, buffers, mut term) = make_env(&[b"hello"], 20);
    assert!(display.sgarbf);
    display.update(&mut windows, &buffers, &mut term).unwrap();
    assert!(!display.sgarbf);
}

#[test]
fn test_update_renders_text() {
    let (mut display, mut windows, buffers, mut term) = make_env(&[b"hello"], 20);
    display.sgarbf = false;
    display.update(&mut windows, &buffers, &mut term).unwrap();
    let row = &display.vscreen[0];
    let mut text = String::new();
    for cell in &row.cells {
        text.push(cell.ch);
    }
    assert!(text.starts_with("hello"));
}

#[test]
fn test_update_multi_line() {
    let (mut display, mut windows, buffers, mut term) = make_env(&[b"line1", b"line2"], 20);
    display.sgarbf = false;
    display.update(&mut windows, &buffers, &mut term).unwrap();
    let row0 = &display.vscreen[0];
    let mut t0 = String::new();
    for c in &row0.cells {
        t0.push(c.ch);
    }
    assert!(t0.starts_with("line1"), "got: {t0}");
    let row1 = &display.vscreen[1];
    let mut t1 = String::new();
    for c in &row1.cells {
        t1.push(c.ch);
    }
    assert!(t1.starts_with("line2"), "got: {t1}");
}

#[test]
fn test_update_modeline() {
    let (mut display, mut windows, buffers, mut term) = make_env(&[b"hello"], 20);
    display.sgarbf = false;
    display.update(&mut windows, &buffers, &mut term).unwrap();
    let modeline_row = &display.vscreen[20];
    let mut text = String::new();
    for c in &modeline_row.cells {
        text.push(c.ch);
    }
    assert!(
        text.contains("test"),
        "modeline should contain buf name, got: {text}"
    );
    assert!(
        modeline_row.cells.iter().any(|c| c.reverse),
        "modeline should have reverse"
    );
}

#[test]
fn test_modeline_position_indicator() {
    let _term = MockTerminal::new();
    let mut buffers = Buffers::new();
    let buf = buffers.create("test");
    let mut last = buf.head;
    for i in 0..50 {
        let id = buf.insert_after(last, {
            let mut l = Line::new();
            l.text = format!("line{i}").into_bytes();
            l
        });
        last = id;
    }
    let mut windows = Windows::new();
    let first_line = buf.head_line().next();
    let win = windows.create(buf.id, first_line);
    win.top_row = 0;
    win.n_rows = 20;
    win.top_line = first_line;
    win.dot_line = first_line;
    win.flags = WindowFlags::MODE_LINE;

    let mut display = Display::with_size(24, 80);
    display.modeline(win, &buffers);
    let modeline_row = &display.vscreen[20];
    let text: String = modeline_row.cells.iter().map(|c| c.ch).collect();
    assert!(
        text.contains(" Top "),
        "expected Top indicator, got: {text}"
    );
}

#[test]
fn test_render_line_tab() {
    let (mut display, _windows, buffers, _term) = make_env(&[b"\t"], 20);
    let buf_id = buffers.iter().next().unwrap().id;
    let buf = buffers.get(buf_id).unwrap();
    let line = buf.head_line().next();
    display.sgarbf = false;
    display.render_line(0, line, buf);
    let cells = &display.vscreen[0].cells;
    assert_eq!(cells[0].ch, ' ');
    assert_eq!(cells[7].ch, ' ');
    assert_eq!(cells[8].ch, ' ');
}

#[test]
fn test_render_line_control() {
    let (mut display, _windows, buffers, _term) = make_env(&[b"\x01"], 20);
    let buf_id = buffers.iter().next().unwrap().id;
    let buf = buffers.get(buf_id).unwrap();
    let line = buf.head_line().next();
    display.render_line(0, line, buf);
    let cells = &display.vscreen[0].cells;
    assert_eq!(cells[0].ch, '^');
    assert_eq!(cells[1].ch, 'A');
}

#[test]
fn test_render_line_overflow() {
    let long_line: Vec<u8> = (0..90).map(|i| b'a' + (i % 26)).collect();
    let (mut display, _windows, buffers, _term) = make_env(&[&long_line], 20);
    let buf_id = buffers.iter().next().unwrap().id;
    let buf = buffers.get(buf_id).unwrap();
    let line = buf.head_line().next();
    display.render_line(0, line, buf);
    assert_eq!(display.vscreen[0].cells[79].ch, '$');
}

#[test]
fn test_modeline_modified() {
    let (mut display, mut windows, buffers, _term) = make_env(&[b"data"], 20);
    let mut bufs = buffers;
    let buf_id = bufs.iter().next().unwrap().id;
    bufs.get_mut(buf_id).unwrap().flags |= BufferFlags::CHANGED;
    let win = windows.iter_mut().next().unwrap();
    win.flags = WindowFlags::MODE_LINE;
    display.modeline(win, &bufs);
    let modeline_row = &display.vscreen[20];
    let text: String = modeline_row.cells.iter().map(|c| c.ch).collect();
    assert!(
        text.starts_with("-* "),
        "modified modeline should have -*, got: {text}"
    );
}

#[test]
fn test_resize_zero_clamps() {
    let mut display = Display::with_size(0, 0);
    assert_eq!(display.nrows, 1);
    assert_eq!(display.ncols, 1);
    display.resize(0, 0);
    assert_eq!(display.nrows, 1);
    assert_eq!(display.ncols, 1);
    display.resize(5, 10);
    assert_eq!(display.nrows, 5);
    assert_eq!(display.ncols, 10);
}

#[test]
fn test_write_echo() {
    let mut display = Display::with_size(10, 20);
    let mut term = MockTerminal::new();
    display.write_echo(&mut term, "hello").unwrap();
    let row = &display.vscreen[9];
    let text: String = row.cells.iter().map(|c| c.ch).collect();
    assert!(
        text.starts_with("hello"),
        "echo text should appear, got: {text}"
    );
}

#[test]
fn test_modeline_empty_buffer_indicator() {
    let mut buffers = Buffers::new();
    let buf = buffers.create("empty");
    let mut windows = Windows::new();
    let win = windows.create(buf.id, buf.head);
    win.top_row = 0;
    win.n_rows = 20;
    win.top_line = buf.head;
    win.dot_line = buf.head;
    win.flags = WindowFlags::MODE_LINE;

    let mut display = Display::with_size(24, 80);
    display.modeline(win, &buffers);
    let modeline_row = &display.vscreen[20];
    let text: String = modeline_row.cells.iter().map(|c| c.ch).collect();
    assert!(
        text.contains(" Emp "),
        "expected Emp indicator, got: {text}"
    );
}

#[test]
fn test_modeline_position_bot() {
    let _term = MockTerminal::new();
    let mut buffers = Buffers::new();
    let buf = buffers.create("test");
    let mut last = buf.head;
    for i in 0..10 {
        let id = buf.insert_after(last, {
            let mut l = Line::new();
            l.text = format!("line{i}").into_bytes();
            l
        });
        last = id;
    }
    let mut windows = Windows::new();
    let win = windows.create(buf.id, last);
    win.top_row = 0;
    win.n_rows = 20;
    win.top_line = last;
    win.dot_line = last;
    win.flags = WindowFlags::MODE_LINE;

    let mut display = Display::with_size(24, 80);
    display.modeline(win, &buffers);
    let modeline_row = &display.vscreen[20];
    let text: String = modeline_row.cells.iter().map(|c| c.ch).collect();
    assert!(
        text.contains(" Bot ") || text.contains(" All "),
        "expected Bot or All indicator, got: {text}"
    );
}

#[test]
fn test_modeline_position_percentage() {
    let _term = MockTerminal::new();
    let mut buffers = Buffers::new();
    let buf = buffers.create("test");
    let mut last = buf.head;
    for i in 0..100 {
        let id = buf.insert_after(last, {
            let mut l = Line::new();
            l.text = format!("line{i}").into_bytes();
            l
        });
        last = id;
    }
    let mut windows = Windows::new();
    let mut mid = buf.head_line().next();
    for _ in 0..50 {
        if let Some(l) = buf.line(mid) {
            mid = l.next();
        }
    }
    let win = windows.create(buf.id, mid);
    win.top_row = 0;
    win.n_rows = 20;
    win.top_line = mid;
    win.dot_line = mid;
    win.flags = WindowFlags::MODE_LINE;

    let mut display = Display::with_size(24, 80);
    display.modeline(win, &buffers);
    let modeline_row = &display.vscreen[20];
    let text: String = modeline_row.cells.iter().map(|c| c.ch).collect();
    assert!(
        text.contains('%'),
        "expected percentage indicator, got: {text}"
    );
}

#[test]
fn test_position_indicator_all() {
    let _term = MockTerminal::new();
    let mut buffers = Buffers::new();
    let buf = buffers.create("test");
    let l1 = buf.insert_after(buf.head, Line::new());
    buf.line_mut(l1).unwrap().text = b"x".to_vec();
    let mut windows = Windows::new();
    let win = windows.create(buf.id, l1);
    win.top_row = 0;
    win.n_rows = 20;
    win.top_line = l1;
    win.dot_line = l1;
    let display = Display::with_size(24, 80);
    let indicator = display.position_indicator(win, buf);
    assert_eq!(indicator, " All ");
}

#[test]
fn test_find_screen_line_with_buffer() {
    let _term = MockTerminal::new();
    let mut buffers = Buffers::new();
    let buf = buffers.create("test");
    let l1 = buf.insert_after(buf.head, Line::new());
    buf.line_mut(l1).unwrap().text = b"hello".to_vec();
    let mut windows = Windows::new();
    let win = windows.create(buf.id, l1);
    win.top_row = 0;
    win.n_rows = 10;
    win.top_line = l1;
    let display = Display::with_size(24, 80);
    let sline = display.find_screen_line(win, &buffers);
    assert_eq!(sline, 0);
}

#[test]
fn test_find_screen_line_no_buffer() {
    let _term = MockTerminal::new();
    let buffers = Buffers::new();
    let mut windows = Windows::new();
    let win = windows.create(BufferId(99), LineId(0));
    win.top_row = 5;
    let display = Display::with_size(24, 80);
    let sline = display.find_screen_line(win, &buffers);
    assert_eq!(sline, 5);
}

#[test]
fn test_position_indicator_with_prev_line_at_top() {
    let _term = MockTerminal::new();
    let mut buffers = Buffers::new();
    let buf = buffers.create("test");
    let l1 = buf.insert_after(buf.head, Line::new());
    buf.line_mut(l1).unwrap().text = b"hello".to_vec();
    let mut windows = Windows::new();
    let win = windows.create(buf.id, l1);
    win.top_row = 0;
    win.n_rows = 20;
    win.top_line = l1;
    win.dot_line = l1;
    let display = Display::with_size(24, 80);
    let indicator = display.position_indicator(win, buf);
    assert!(
        indicator == " Top " || indicator == " All ",
        "expected Top or All, got: {indicator}"
    );
}

#[test]
fn test_resize_same_size_noop() {
    let mut display = Display::with_size(24, 80);
    display.sgarbf = false;
    display.resize(24, 80);
    assert!(!display.sgarbf);
}

#[test]
fn test_resize_min_dimensions() {
    let mut display = Display::with_size(24, 80);
    display.resize(0, 0);
    assert_eq!(display.nrows, 1);
    assert_eq!(display.ncols, 1);
}

#[test]
fn test_cell_default() {
    let cell = Cell::default();
    assert_eq!(cell.ch, ' ');
    assert!(!cell.reverse);
}

#[test]
fn test_video_row_default() {
    let row = VideoRow {
        cells: vec![Cell::default()],
        flags: 0,
    };
    assert_eq!(row.cells[0].ch, ' ');
    assert_eq!(row.flags, 0);
}

#[test]
fn test_modeline_with_modified_buffer() {
    let mut term = MockTerminal::new();
    let mut buffers = Buffers::new();
    let buf = buffers.create("test");
    let l1 = buf.insert_after(buf.head, Line::new());
    buf.line_mut(l1).unwrap().text = b"hello".to_vec();
    buf.flags |= BufferFlags::CHANGED;
    let mut windows = Windows::new();
    let win = windows.create(buf.id, l1);
    win.top_row = 0;
    win.n_rows = 20;
    win.flags = WindowFlags::MODE_LINE;
    let mut display = Display::with_size(24, 80);
    display.update(&mut windows, &buffers, &mut term).unwrap();
    let modeline_row = &display.vscreen[20];
    let text: String = modeline_row.cells.iter().map(|c| c.ch).collect();
    assert!(
        text.contains('*'),
        "modeline should show modified indicator"
    );
}

#[test]
fn test_reframe_with_dot_at_top() {
    let mut buffers = Buffers::new();
    let buf = buffers.create("test");
    let l1 = buf.insert_after(buf.head, Line::new());
    buf.line_mut(l1).unwrap().text = b"hello".to_vec();
    let mut windows = Windows::new();
    let win = windows.create(buf.id, l1);
    win.top_row = 0;
    win.n_rows = 10;
    win.top_line = l1;
    win.dot_line = l1;
    let display = Display::with_size(24, 80);
    display.reframe(win, &buffers);
    assert_eq!(win.top_line, l1);
}

#[test]
fn test_reframe_with_wfforce() {
    let mut buffers = Buffers::new();
    let buf = buffers.create("test");
    let l1 = buf.insert_after(buf.head, Line::new());
    buf.line_mut(l1).unwrap().text = b"hello".to_vec();
    let mut windows = Windows::new();
    let win = windows.create(buf.id, l1);
    win.top_row = 0;
    win.n_rows = 10;
    win.top_line = l1;
    win.dot_line = l1;
    win.flags = WindowFlags::FORCE;
    let display = Display::with_size(24, 80);
    display.reframe(win, &buffers);
    assert_eq!(win.flags & WindowFlags::FORCE, WindowFlags::EMPTY);
}

#[test]
fn test_position_indicator_with_no_prev_line() {
    let mut buffers = Buffers::new();
    let buf = buffers.create("test");
    let l1 = buf.insert_after(buf.head, Line::new());
    buf.line_mut(l1).unwrap().text = b"hello".to_vec();
    let mut windows = Windows::new();
    let win = windows.create(buf.id, l1);
    win.top_row = 0;
    win.n_rows = 20;
    win.top_line = l1;
    win.dot_line = l1;
    let display = Display::with_size(24, 80);
    let indicator = display.position_indicator(win, buf);
    assert!(!indicator.is_empty());
}

#[test]
fn test_clear_row() {
    let mut display = Display::with_size(10, 40);
    display.clear_row(5);
    let row = &display.vscreen[5];
    assert!(row.flags & VFCHG != 0);
}

#[test]
fn test_modeline_at_bottom() {
    let mut buffers = Buffers::new();
    let buf_id = buffers.create("test").id;
    let buf = buffers.get_mut(buf_id).unwrap();
    let l1 = buf.insert_after(buf.head, Line::new());
    buf.line_mut(l1).unwrap().text = b"hello".to_vec();
    let mut windows = Windows::new();
    let win = windows.create(buf_id, l1);
    win.top_row = 0;
    win.n_rows = 10;
    win.top_line = l1;
    win.dot_line = l1;
    let display = Display::with_size(24, 80);
    let buf = buffers.get(buf_id).unwrap();
    let indicator = display.position_indicator(win, buf);
    assert!(indicator.contains("All") || indicator.contains("Bot"));
}

#[test]
fn test_modeline_invalid_top_line() {
    let mut buffers = Buffers::new();
    let buf_id = buffers.create("test").id;
    let buf = buffers.get_mut(buf_id).unwrap();
    let l1 = buf.insert_after(buf.head, Line::new());
    buf.line_mut(l1).unwrap().text = b"hello".to_vec();
    let mut windows = Windows::new();
    let win = windows.create(buf_id, l1);
    win.top_row = 0;
    win.n_rows = 10;
    win.top_line = LineId(999);
    win.dot_line = l1;
    let display = Display::with_size(24, 80);
    let buf = buffers.get(buf_id).unwrap();
    let indicator = display.position_indicator(win, buf);
    assert!(!indicator.is_empty());
}

#[test]
fn test_modeline_empty_buffer_ratio() {
    let mut buffers = Buffers::new();
    let buf_id = buffers.create("test").id;
    let buf = buffers.get(buf_id).unwrap();
    let head = buf.head;
    let mut windows = Windows::new();
    let win = windows.create(buf_id, head);
    win.top_row = 0;
    win.n_rows = 10;
    win.top_line = head;
    win.dot_line = head;
    let display = Display::with_size(24, 80);
    let buf = buffers.get(buf_id).unwrap();
    let indicator = display.position_indicator(win, buf);
    assert!(!indicator.is_empty());
}

#[test]
fn test_modeline_zero_total_lines() {
    let mut buffers = Buffers::new();
    let buf_id = buffers.create("test").id;
    let buf = buffers.get(buf_id).unwrap();
    let head = buf.head;
    let mut windows = Windows::new();
    let win = windows.create(buf_id, head);
    win.top_row = 0;
    win.n_rows = 10;
    win.top_line = head;
    win.dot_line = head;
    let display = Display::with_size(24, 80);
    let buf = buffers.get(buf_id).unwrap();
    let indicator = display.position_indicator(win, buf);
    assert!(!indicator.is_empty());
}

#[test]
fn test_find_screen_line_dot_not_found() {
    let mut buffers = Buffers::new();
    let buf = buffers.create("test");
    let l1 = buf.insert_after(buf.head, Line::new());
    let l2 = buf.insert_after(l1, Line::new());
    buf.line_mut(l1).unwrap().text = b"line1".to_vec();
    buf.line_mut(l2).unwrap().text = b"line2".to_vec();
    let mut windows = Windows::new();
    let win = windows.create(buf.id, l1);
    win.top_row = 0;
    win.n_rows = 10;
    win.top_line = l1;
    win.dot_line = LineId(999);
    let display = Display::with_size(24, 80);
    let sline = display.find_screen_line(win, &buffers);
    assert!(sline <= 10);
}

#[test]
fn test_position_indicator_zero_total_lines() {
    let mut buffers = Buffers::new();
    let buf_id = buffers.create("test").id;
    let buf = buffers.get(buf_id).unwrap();
    let head = buf.head;
    let mut windows = Windows::new();
    let win = windows.create(buf_id, head);
    win.top_row = 0;
    win.n_rows = 10;
    win.top_line = head;
    win.dot_line = head;
    let display = Display::with_size(24, 80);
    let buf = buffers.get(buf_id).unwrap();
    let indicator = display.position_indicator(win, buf);
    assert!(!indicator.is_empty());
}

#[test]
fn test_echo_text_normal() {
    let mut display = Display::with_size(24, 80);
    display.vscreen[23].cells[0].ch = 'H';
    display.vscreen[23].cells[1].ch = 'i';
    let text = display.echo_text();
    assert!(text.starts_with("Hi"));
}

#[test]
fn test_updgar_marks_all_rows_changed() {
    let mut display = Display::with_size(24, 80);
    let term = MockTerminal::new();
    display.handle_garbage_or_resize_lines(&term);
    for row in 0..display.nrows {
        assert!(display.vscreen[row].flags & VFCHG != 0);
        assert!(display.pscreen[row].flags & VFCHG != 0);
    }
}

#[test]
fn test_ch_width_ascii() {
    assert_eq!(char_display_width('a'), 1);
    assert_eq!(char_display_width('Z'), 1);
    assert_eq!(char_display_width('0'), 1);
}

#[test]
fn test_ch_width_cjk() {
    assert_eq!(char_display_width('\u{4e2d}'), 2);
    assert_eq!(char_display_width('\u{6587}'), 2);
    assert_eq!(char_display_width('\u{5b57}'), 2);
}

#[test]
fn test_ch_width_wide_ranges() {
    assert_eq!(char_display_width('\u{1100}'), 2);
    assert_eq!(char_display_width('\u{AC00}'), 2);
    assert_eq!(char_display_width('\u{F900}'), 2);
    assert_eq!(char_display_width('\u{FE30}'), 2);
    assert_eq!(char_display_width('\u{FF01}'), 2);
    assert_eq!(char_display_width('\u{FFE0}'), 2);
}

#[test]
fn test_render_line_utf8_cjk() {
    let (mut display, _windows, buffers, _term) = make_env(&["\u{4e2d}\u{6587}".as_bytes()], 20);
    let buf_id = buffers.iter().next().unwrap().id;
    let buf = buffers.get(buf_id).unwrap();
    let line = buf.head_line().next();
    display.render_line(0, line, buf);
    let cells = &display.vscreen[0].cells;
    assert_eq!(cells[0].ch, '\u{4e2d}');
    assert_eq!(cells[2].ch, '\u{6587}');
}

#[test]
fn test_render_line_utf8_mixed() {
    let (mut display, _windows, buffers, _term) = make_env(&["Hi\u{4e2d}".as_bytes()], 20);
    let buf_id = buffers.iter().next().unwrap().id;
    let buf = buffers.get(buf_id).unwrap();
    let line = buf.head_line().next();
    display.render_line(0, line, buf);
    let cells = &display.vscreen[0].cells;
    assert_eq!(cells[0].ch, 'H');
    assert_eq!(cells[1].ch, 'i');
    assert_eq!(cells[2].ch, '\u{4e2d}');
}

#[test]
fn test_dot_column_ascii() {
    let mut buffers = Buffers::new();
    let buf = buffers.create("test");
    let l1 = buf.insert_after(buf.head, Line::new());
    buf.line_mut(l1).unwrap().text = b"hello".to_vec();
    let mut windows = Windows::new();
    let win = windows.create(buf.id, l1);
    win.top_row = 0;
    win.n_rows = 10;
    win.top_line = l1;
    win.dot_line = l1;
    win.dot_offset = LineOffset(3);
    let display = Display::with_size(24, 80);
    assert_eq!(display.dot_column(win, &buffers), 3);
}

#[test]
fn test_dot_column_with_tab() {
    let mut buffers = Buffers::new();
    let buf = buffers.create("test");
    let l1 = buf.insert_after(buf.head, Line::new());
    buf.line_mut(l1).unwrap().text = b"\tworld".to_vec();
    let mut windows = Windows::new();
    let win = windows.create(buf.id, l1);
    win.top_row = 0;
    win.n_rows = 10;
    win.top_line = l1;
    win.dot_line = l1;
    win.dot_offset = LineOffset(5);
    let display = Display::with_size(24, 80);
    assert_eq!(display.dot_column(win, &buffers), 12);
}

#[test]
fn test_dot_column_with_utf8() {
    let mut buffers = Buffers::new();
    let buf = buffers.create("test");
    let l1 = buf.insert_after(buf.head, Line::new());
    buf.line_mut(l1).unwrap().text = "\u{4e2d}\u{6587}".to_string().into_bytes();
    let mut windows = Windows::new();
    let win = windows.create(buf.id, l1);
    win.top_row = 0;
    win.n_rows = 10;
    win.top_line = l1;
    win.dot_line = l1;
    win.dot_offset = LineOffset(3);
    let display = Display::with_size(24, 80);
    assert_eq!(display.dot_column(win, &buffers), 2);
}

#[test]
fn test_dot_column_no_buffer() {
    let buffers = Buffers::new();
    let mut windows = Windows::new();
    let win = windows.create(BufferId(99), LineId(0));
    let display = Display::with_size(24, 80);
    assert_eq!(display.dot_column(win, &buffers), 0);
}

#[test]
fn test_render_line_utf8_overflow_indicator() {
    let mut display = Display::with_size(24, 4);
    let mut buffers = Buffers::new();
    let buf = buffers.create("test");
    let l1 = buf.insert_after(buf.head, Line::new());
    buf.line_mut(l1).unwrap().text = "\u{4e2d}\u{6587}\u{5b57}".to_string().into_bytes();
    display.render_line(0, l1, buf);
    assert_eq!(display.vscreen[0].cells[3].ch, '$');
}

#[test]
fn test_dot_column_with_control_char() {
    let mut buffers = Buffers::new();
    let buf = buffers.create("test");
    let l1 = buf.insert_after(buf.head, Line::new());
    buf.line_mut(l1).unwrap().text = b"\x01hello".to_vec();
    let mut windows = Windows::new();
    let win = windows.create(buf.id, l1);
    win.top_row = 0;
    win.n_rows = 10;
    win.top_line = l1;
    win.dot_line = l1;
    win.dot_offset = LineOffset(3);
    let display = Display::with_size(24, 80);
    assert_eq!(display.dot_column(win, &buffers), 4);
}

#[test]
fn test_render_line_control_char() {
    let (mut display, _windows, buffers, _term) = make_env(&[b"\x03"], 20);
    let buf_id = buffers.iter().next().unwrap().id;
    let buf = buffers.get(buf_id).unwrap();
    let line = buf.head_line().next();
    display.render_line(0, line, buf);
    let cells = &display.vscreen[0].cells;
    assert_eq!(cells[0].ch, '^');
    assert_eq!(cells[1].ch, 'C');
}

#[test]
fn test_modeline_shows_modes() {
    let mut buffers = Buffers::new();
    let buf_id = buffers.create("test").id;
    let buf = buffers.get_mut(buf_id).unwrap();
    let l1 = buf.insert_after(buf.head, Line::new());
    buf.line_mut(l1).unwrap().text = b"hi".to_vec();
    buf.mode = Mode::WRAP | Mode::MAGIC;
    let mut windows = Windows::new();
    let win = windows.create(buf_id, l1);
    win.top_row = 0;
    win.n_rows = 20;
    let mut display = Display::with_size(24, 80);
    display.modeline(windows.current().unwrap(), &buffers);
    let text: String = display.vscreen[20].cells.iter().map(|c| c.ch).collect();
    assert!(
        text.contains("Wrap"),
        "modeline should show Wrap mode, got: {text}"
    );
    assert!(
        text.contains("Magic"),
        "modeline should show Magic mode, got: {text}"
    );
}

#[test]
fn test_modeline_shows_filename_when_distinct() {
    let mut buffers = Buffers::new();
    let buf_id = buffers.create("scratch").id;
    let buf = buffers.get_mut(buf_id).unwrap();
    buf.filename = String::from("/tmp/scratch.txt");
    let l1 = buf.insert_after(buf.head, Line::new());
    buf.line_mut(l1).unwrap().text = b"hi".to_vec();
    let mut windows = Windows::new();
    let win = windows.create(buf_id, l1);
    win.top_row = 0;
    win.n_rows = 20;
    let mut display = Display::with_size(24, 80);
    display.modeline(windows.current().unwrap(), &buffers);
    let text: String = display.vscreen[20].cells.iter().map(|c| c.ch).collect();
    assert!(
        text.contains("scratch.txt"),
        "modeline should show filename, got: {text}"
    );
}

#[test]
fn test_position_indicator_cursor_based() {
    let mut buffers = Buffers::new();
    let buf = buffers.create("test");
    let mut last = buf.head;
    let mut lines = vec![];
    for i in 0..100 {
        let mut l = Line::new();
        l.text = format!("line{i}").into_bytes();
        let id = buf.insert_after(last, l);
        lines.push(id);
        last = id;
    }
    let mut windows = Windows::new();
    let win = windows.create(buf.id, lines[0]);
    win.top_row = 0;
    win.n_rows = 10;
    win.top_line = lines[0];
    let display = Display::with_size(24, 80);

    win.dot_line = lines[0];
    assert_eq!(display.position_indicator(win, buf), " Top ");
    win.dot_line = lines[24];
    assert_eq!(display.position_indicator(win, buf), " 25% ");
    win.dot_line = lines[49];
    assert_eq!(display.position_indicator(win, buf), " 50% ");
    win.dot_line = lines[99];
    assert_eq!(display.position_indicator(win, buf), " Bot ");
}

#[test]
fn test_modeline_redraws_pct_on_in_page_move() {
    let term = MockTerminal::new();
    let mut buffers = Buffers::new();
    let buf = buffers.create("test");
    buf.filename = "f.txt".to_string();
    let mut last = buf.head;
    let mut lines = vec![];
    for i in 0..100 {
        let mut l = Line::new();
        l.text = format!("line{i}").into_bytes();
        let id = buf.insert_after(last, l);
        lines.push(id);
        last = id;
    }
    let buf_id = buf.id;
    let mut windows = Windows::new();
    let wid = windows.create(buf_id, lines[0]).id;
    {
        let w = windows.get_mut(wid).unwrap();
        w.top_row = 0;
        w.n_rows = 10;
        w.top_line = lines[0];
        w.dot_line = lines[0];
        w.flags = WindowFlags::HARD | WindowFlags::MODE_LINE;
    }
    let mut display = Display::with_size(12, 80);
    let mut t = term;
    display.update(&mut windows, &buffers, &mut t).unwrap();
    let m0: String = display.vscreen[10].cells.iter().map(|c| c.ch).collect();
    assert!(m0.contains(" Top "), "expected Top initially, got: {m0}");

    windows.get_mut(wid).unwrap().dot_line = lines[4];
    windows.get_mut(wid).unwrap().flags = WindowFlags::MOVED;
    display.update(&mut windows, &buffers, &mut t).unwrap();
    let m1: String = display.vscreen[10].cells.iter().map(|c| c.ch).collect();
    assert!(
        m1.contains(" 5% "),
        "expected 5% after in-page move, got: {m1}"
    );
}
