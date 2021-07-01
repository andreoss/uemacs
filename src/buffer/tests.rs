use super::*;

#[test]
fn test_new_empty() {
    let buf = Buffer::new(BufferId(0), "test");
    assert_eq!(buf.id, BufferId(0));
    assert_eq!(buf.name, "test");
    assert!(buf.is_empty());
    assert_eq!(buf.lines.len(), 1);
    let head = buf.head_line();
    assert_eq!(head.next(), LineId(0));
    assert_eq!(head.prev(), LineId(0));
}

#[test]
fn test_insert_after() {
    let mut buf = Buffer::new(BufferId(0), "test");
    let line = Line::new();
    let id = buf.insert_after(LineId(0), line);
    assert!(!buf.is_empty());
    assert_eq!(id, LineId(1));
    let head = buf.head_line();
    assert_eq!(head.next(), id);
    assert_eq!(head.prev(), id);
    let inserted = buf.line(id).unwrap();
    assert_eq!(inserted.next(), LineId(0));
    assert_eq!(inserted.prev(), LineId(0));
}

#[test]
fn test_insert_multiple() {
    let mut buf = Buffer::new(BufferId(0), "test");
    let id1 = buf.insert_after(LineId(0), Line::new());
    let id2 = buf.insert_after(id1, Line::new());
    let head = buf.head_line();
    assert_eq!(head.next(), id1);
    assert_eq!(head.prev(), id2);
    let line1 = buf.line(id1).unwrap();
    assert_eq!(line1.next(), id2);
    assert_eq!(line1.prev(), LineId(0));
    let line2 = buf.line(id2).unwrap();
    assert_eq!(line2.next(), LineId(0));
    assert_eq!(line2.prev(), id1);
}

#[test]
fn test_dot_and_mark() {
    let mut buf = Buffer::new(BufferId(0), "test");
    assert_eq!(buf.dot(), (LineId(0), LineOffset(0)));
    buf.set_dot(LineId(1), LineOffset(5));
    assert_eq!(buf.dot(), (LineId(1), LineOffset(5)));
    assert_eq!(buf.mark(), None);
    buf.set_mark(LineId(2), LineOffset(3));
    assert_eq!(buf.mark(), Some((LineId(2), LineOffset(3))));
    buf.clear_mark();
    assert_eq!(buf.mark(), None);
}

#[test]
fn test_remove() {
    let mut buf = Buffer::new(BufferId(0), "test");
    let id1 = buf.insert_after(LineId(0), Line::new());
    let id2 = buf.insert_after(id1, Line::new());
    let removed = buf.remove(id1);
    assert!(removed.is_some());
    let head = buf.head_line();
    assert_eq!(head.next(), id2);
    assert_eq!(head.prev(), id2);
    let line2 = buf.line(id2).unwrap();
    assert_eq!(line2.prev(), LineId(0));
    assert_eq!(line2.next(), LineId(0));
    assert!(buf.remove(LineId(0)).is_none());
}

#[test]
fn test_buffers_new() {
    let bs = Buffers::new();
    assert!(bs.is_empty());
    assert_eq!(bs.len(), 0);
}

#[test]
fn test_buffers_create() {
    let mut bs = Buffers::new();
    let b = bs.create("foo");
    assert_eq!(b.id, BufferId(0));
    assert_eq!(b.name, "foo");
    assert_eq!(bs.len(), 1);
    let b2 = bs.create("bar");
    assert_eq!(b2.id, BufferId(1));
    assert_eq!(bs.len(), 2);
}

#[test]
fn test_buffers_find() {
    let mut bs = Buffers::new();
    bs.create("foo");
    bs.create("bar");
    assert!(bs.find("foo").is_some());
    assert!(bs.find("baz").is_none());
}

#[test]
fn test_buffers_find_or_create() {
    let mut bs = Buffers::new();
    let b1 = bs.find_or_create("foo");
    assert_eq!(b1.id, BufferId(0));
    let b2 = bs.find_or_create("foo");
    assert_eq!(b2.id, BufferId(0));
    let b3 = bs.find_or_create("bar");
    assert_eq!(b3.id, BufferId(1));
    assert_eq!(bs.len(), 2);
}

#[test]
fn test_buffers_get() {
    let mut bs = Buffers::new();
    bs.create("foo");
    assert!(bs.get(BufferId(0)).is_some());
    assert!(bs.get(BufferId(99)).is_none());
}

#[test]
fn test_buffers_remove() {
    let mut bs = Buffers::new();
    bs.create("foo");
    bs.create("bar");
    let removed = bs.remove(BufferId(0));
    assert!(removed.is_some());
    assert_eq!(bs.len(), 1);
    assert!(bs.find("foo").is_none());
    assert!(bs.remove(BufferId(99)).is_none());
}

#[test]
fn test_clear_empty() {
    let mut buf = Buffer::new(BufferId(0), "test");
    buf.filename = "foo.txt".to_string();
    buf.set_dot(LineId(1), LineOffset(5));
    buf.set_mark(LineId(2), LineOffset(3));
    buf.flags = BufferFlags::CHANGED;
    buf.clear();
    assert!(buf.is_empty());
    assert_eq!(buf.dot(), (LineId(0), LineOffset(0)));
    assert_eq!(buf.mark(), None);
    assert!(!buf.flags.intersects(BufferFlags::CHANGED));
    assert!(buf.filename.is_empty());
}

#[test]
fn test_clear_with_lines() {
    let mut buf = Buffer::new(BufferId(0), "test");
    buf.insert_after(LineId(0), Line::new());
    buf.insert_after(LineId(1), Line::new());
    assert!(!buf.is_empty());
    buf.clear();
    assert!(buf.is_empty());
    assert_eq!(buf.lines.len(), 1);
    let head = buf.head_line();
    assert_eq!(head.next(), LineId(0));
    assert_eq!(head.prev(), LineId(0));
}

#[test]
fn test_line_count() {
    let mut buf = Buffer::new(BufferId(0), "test");
    assert_eq!(buf.line_count(), 0);
    buf.insert_after(LineId(0), Line::new());
    assert_eq!(buf.line_count(), 1);
    buf.insert_after(LineId(1), Line::new());
    buf.insert_after(LineId(2), Line::new());
    assert_eq!(buf.line_count(), 3);
}

#[test]
fn test_nth_line() {
    let mut buf = Buffer::new(BufferId(0), "test");
    assert_eq!(buf.nth_line(0), None);
    let id1 = buf.insert_after(LineId(0), Line::new());
    let id2 = buf.insert_after(id1, Line::new());
    let id3 = buf.insert_after(id2, Line::new());
    assert_eq!(buf.nth_line(0), Some(id1));
    assert_eq!(buf.nth_line(1), Some(id2));
    assert_eq!(buf.nth_line(2), Some(id3));
    assert_eq!(buf.nth_line(3), None);
}

#[test]
fn test_line_iter() {
    let mut buf = Buffer::new(BufferId(0), "test");
    let id1 = buf.insert_after(LineId(0), Line::new());
    let id2 = buf.insert_after(id1, Line::new());
    let ids: Vec<LineId> = buf
        .line_iter()
        .map(|l| {
            let idx = buf.lines.iter().position(|x| std::ptr::eq(x, l)).unwrap();
            LineId(idx)
        })
        .collect();
    assert_eq!(ids, vec![id1, id2]);
}

#[test]
fn test_for_each_line_mut() {
    let mut buf = Buffer::new(BufferId(0), "test");
    let mut l1 = Line::new();
    l1.text = vec![1];
    let mut l2 = Line::new();
    l2.text = vec![2];
    let id1 = buf.insert_after(LineId(0), l1);
    let id2 = buf.insert_after(id1, l2);
    buf.for_each_line_mut(|line| {
        if let Some(b) = line.text.get_mut(0) {
            *b += 10;
        }
    });
    assert_eq!(buf.line(id1).unwrap().text, vec![11]);
    assert_eq!(buf.line(id2).unwrap().text, vec![12]);
}

#[test]
fn test_line_mut() {
    let mut buf = Buffer::new(BufferId(0), "test");
    let id = buf.insert_after(LineId(0), Line::new());
    buf.line_mut(id).unwrap().text = vec![1, 2, 3];
    assert_eq!(buf.line(id).unwrap().text, vec![1, 2, 3]);
}

#[test]
fn test_head_line_mut() {
    let mut buf = Buffer::new(BufferId(0), "test");
    buf.head_line_mut().text = vec![9];
    assert_eq!(buf.head_line().text, vec![9]);
}

#[test]
fn test_buffers_find_mut() {
    let mut bs = Buffers::new();
    bs.create("foo");
    bs.find_mut("foo").unwrap().name = "baz".to_string();
    assert!(bs.find("baz").is_some());
}

#[test]
fn test_buffers_get_mut() {
    let mut bs = Buffers::new();
    bs.create("foo");
    bs.get_mut(BufferId(0)).unwrap().name = "qux".to_string();
    assert!(bs.find("qux").is_some());
}

#[test]
fn test_buffers_iter() {
    let mut bs = Buffers::new();
    bs.create("foo");
    bs.create("bar");
    let names: Vec<&str> = bs.iter().map(|b| b.name.as_str()).collect();
    assert_eq!(names, vec!["foo", "bar"]);
}

#[test]
fn test_buffers_iter_mut() {
    let mut bs = Buffers::new();
    bs.create("foo");
    bs.create("bar");
    for b in bs.iter_mut() {
        b.name.push_str("_x");
    }
    assert!(bs.find("foo_x").is_some());
    assert!(bs.find("bar_x").is_some());
}

#[test]
fn test_split_line() {
    let mut buf = Buffer::new(BufferId(0), "test");
    let id = buf.insert_after(LineId(0), Line::new());
    buf.line_mut(id).unwrap().text = vec![b'a', b'b', b'c', b'd'];
    let new_id = buf.split_line(id, 2);
    assert_eq!(buf.line(id).unwrap().text, vec![b'a', b'b']);
    assert_eq!(buf.line(new_id).unwrap().text, vec![b'c', b'd']);
    let head = buf.head_line();
    assert_eq!(head.next(), id);
    let line = buf.line(id).unwrap();
    assert_eq!(line.next(), new_id);
}
