use super::*;
use crate::core::WindowFlags;

#[test]
fn test_window_new() {
    let w = Window::new(WindowId(0), BufferId(1), LineId(2));
    assert_eq!(w.id, WindowId(0));
    assert_eq!(w.buffer_id, BufferId(1));
    assert_eq!(w.top_line, LineId(2));
    assert_eq!(w.dot(), (LineId(2), LineOffset(0)));
    assert_eq!(w.mark(), None);
    assert!(w.flags.is_empty());
}

#[test]
fn test_window_dot_and_mark() {
    let mut w = Window::new(WindowId(0), BufferId(1), LineId(2));
    w.set_dot(LineId(3), LineOffset(5));
    assert_eq!(w.dot(), (LineId(3), LineOffset(5)));
    w.set_mark(LineId(4), LineOffset(7));
    assert_eq!(w.mark(), Some((LineId(4), LineOffset(7))));
    w.clear_mark();
    assert_eq!(w.mark(), None);
}

#[test]
fn test_window_flags() {
    let mut w = Window::new(WindowId(0), BufferId(1), LineId(2));
    assert!(!w.has_flag(WindowFlags::FORCE));
    w.set_flag(WindowFlags::FORCE);
    assert!(w.has_flag(WindowFlags::FORCE));
    w.set_flag(WindowFlags::HARD);
    assert!(w.has_flag(WindowFlags::HARD));
    w.clear_flag(WindowFlags::FORCE);
    assert!(!w.has_flag(WindowFlags::FORCE));
    assert!(w.has_flag(WindowFlags::HARD));
    w.clear_flags();
    assert!(!w.has_flag(WindowFlags::HARD));
}

#[test]
fn test_windows_new() {
    let ws = Windows::new();
    assert!(ws.is_empty());
    assert_eq!(ws.len(), 0);
    assert!(ws.current().is_none());
}

#[test]
fn test_windows_create() {
    let mut ws = Windows::new();
    let w = ws.create(BufferId(0), LineId(0));
    assert_eq!(w.id, WindowId(0));
    assert_eq!(w.buffer_id, BufferId(0));
    assert_eq!(ws.len(), 1);
    assert_eq!(ws.current().unwrap().id, WindowId(0));

    let w2 = ws.create(BufferId(1), LineId(1));
    assert_eq!(w2.id, WindowId(1));
    assert_eq!(ws.current().unwrap().id, WindowId(0));
}

#[test]
fn test_windows_set_current() {
    let mut ws = Windows::new();
    ws.create(BufferId(0), LineId(0));
    let w2_id = {
        let w2 = ws.create(BufferId(1), LineId(1));
        w2.id
    };
    assert_eq!(ws.current().unwrap().id, WindowId(0));
    assert!(ws.set_current(w2_id));
    assert_eq!(ws.current().unwrap().id, WindowId(1));
    assert!(!ws.set_current(WindowId(99)));
}

#[test]
fn test_windows_get() {
    let mut ws = Windows::new();
    ws.create(BufferId(0), LineId(0));
    assert!(ws.get(WindowId(0)).is_some());
    assert!(ws.get(WindowId(99)).is_none());
}

#[test]
fn test_windows_find_by_buffer() {
    let mut ws = Windows::new();
    ws.create(BufferId(0), LineId(0));
    ws.create(BufferId(1), LineId(1));
    assert!(ws.find_by_buffer(BufferId(0)).is_some());
    assert!(ws.find_by_buffer(BufferId(2)).is_none());
}

#[test]
fn test_windows_remove() {
    let mut ws = Windows::new();
    ws.create(BufferId(0), LineId(0));
    ws.create(BufferId(1), LineId(1));
    let removed = ws.remove(WindowId(0));
    assert!(removed.is_some());
    assert_eq!(ws.len(), 1);
    assert_eq!(ws.current().unwrap().id, WindowId(1));
}

#[test]
fn test_windows_remove_current_fallback() {
    let mut ws = Windows::new();
    ws.create(BufferId(0), LineId(0));
    let w2_id = {
        let w2 = ws.create(BufferId(1), LineId(1));
        w2.id
    };
    ws.set_current(w2_id);
    ws.remove(WindowId(1));
    assert_eq!(ws.current().unwrap().id, WindowId(0));
}
