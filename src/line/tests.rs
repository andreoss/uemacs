use super::*;

#[test]
fn test_new() {
    let line = Line::new();
    assert_eq!(line.len(), 0);
    assert!(line.is_empty());
    assert_eq!(line.capacity(), 0);
}

#[test]
fn test_with_capacity() {
    let line = Line::with_capacity(20);
    assert_eq!(line.len(), 0);
    assert_eq!(line.capacity(), 20);
}

#[test]
fn test_get_byte() {
    let mut line = Line::new();
    line.text = vec![1, 2, 3];
    assert_eq!(line.get_byte(0), Some(1));
    assert_eq!(line.get_byte(2), Some(3));
    assert_eq!(line.get_byte(3), None);
}

#[test]
fn test_put_byte() {
    let mut line = Line::new();
    line.text = vec![1, 2, 3];
    line.put_byte(1, 9);
    assert_eq!(line.text, vec![1, 9, 3]);
}

#[test]
fn test_insert_bytes() {
    let mut line = Line::new();
    line.text = vec![1, 3];
    line.insert_bytes(1, &[2]).unwrap();
    assert_eq!(line.text, vec![1, 2, 3]);
}

#[test]
fn test_insert_bytes_oob_returns_err() {
    let mut line = Line::new();
    line.text = vec![1, 2, 3];
    assert!(line.insert_bytes(5, &[4]).is_err());
    assert!(line.insert_bytes(usize::MAX, &[4]).is_err());
}

#[test]
fn test_delete_bytes() {
    let mut line = Line::new();
    line.text = vec![1, 2, 3, 4];
    line.delete_bytes(1, 2);
    assert_eq!(line.text, vec![1, 4]);
    line.delete_bytes(0, 10);
    assert!(line.is_empty());
}

#[test]
fn test_delete_bytes_offset_past_end_is_noop() {
    let mut line = Line::new();
    line.text = vec![1, 2, 3];
    line.delete_bytes(10, 5);
    assert_eq!(line.text, vec![1, 2, 3]);
    line.delete_bytes(usize::MAX, 1);
    assert_eq!(line.text, vec![1, 2, 3]);
}

#[test]
fn test_split_off() {
    let mut line = Line::new();
    line.text = vec![1, 2, 3, 4];
    let other = line.split_off(2);
    assert_eq!(line.text, vec![1, 2]);
    assert_eq!(other.text, vec![3, 4]);
    assert_eq!(other.next, LineId(0));
    assert_eq!(other.prev, LineId(0));
}

#[test]
fn test_append_line() {
    let mut a = Line::new();
    a.text = vec![1, 2];
    let mut b = Line::new();
    b.text = vec![3, 4];
    a.append_line(&mut b);
    assert_eq!(a.text, vec![1, 2, 3, 4]);
    assert!(b.is_empty());
}

#[test]
fn test_navigation() {
    let mut line = Line::new();
    assert!(!line.is_linked());
    line.set_next(LineId(42));
    line.set_prev(LineId(7));
    assert!(line.is_linked());
    assert_eq!(line.next(), LineId(42));
    assert_eq!(line.prev(), LineId(7));
    line.unlink();
    assert!(!line.is_linked());
    assert_eq!(line.next(), LineId(0));
    assert_eq!(line.prev(), LineId(0));
}
