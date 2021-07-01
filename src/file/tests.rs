use super::*;
use crate::core::BufferId;

fn setup_buffer() -> Buffer {
    Buffer::new(BufferId(0), "test")
}

fn add_line(buf: &mut Buffer, text: &[u8]) {
    let last = buf.head_line().prev();
    let id = buf.insert_after(last, Line::new());
    buf.line_mut(id).unwrap().text = text.to_vec();
}

#[test]
fn test_read_new_file() {
    let filename = "/tmp/uemacs_test_read_new.txt";
    let _ = std::fs::remove_file(filename);
    let mut buf = setup_buffer();
    let (lc, is_new) = read_into_buffer(&mut buf, filename).unwrap();
    assert!(is_new);
    assert_eq!(lc, 0);
    assert!(buf.is_empty());
}

#[test]
fn test_write_and_read_back() {
    let filename = "/tmp/uemacs_test_write_read.txt";
    let _ = std::fs::remove_file(filename);

    let mut buf = setup_buffer();
    add_line(&mut buf, b"hello");
    add_line(&mut buf, b"world");

    let count = write_from_buffer(&mut buf, filename).unwrap();
    assert_eq!(count, 2);

    let mut read_buf = setup_buffer();
    let (_lc, is_new) = read_into_buffer(&mut read_buf, filename).unwrap();
    assert!(!is_new);

    let mut lines = Vec::new();
    let mut cur = read_buf.head_line().next();
    while cur != read_buf.head {
        lines.push(read_buf.line(cur).unwrap().text.clone());
        cur = read_buf.line(cur).unwrap().next();
    }
    assert_eq!(lines, vec![b"hello".to_vec(), b"world".to_vec()]);

    let _ = std::fs::remove_file(filename);
}

#[test]
fn test_write_empty_buffer() {
    let filename = "/tmp/uemacs_test_write_empty.txt";
    let _ = std::fs::remove_file(filename);

    let mut buf = setup_buffer();
    let count = write_from_buffer(&mut buf, filename).unwrap();
    assert_eq!(count, 0);

    let content = std::fs::read(filename).unwrap();
    assert!(content.is_empty());

    let _ = std::fs::remove_file(filename);
}

#[test]
fn test_file_exists() {
    let filename = "/tmp/uemacs_test_exists.txt";
    let _ = std::fs::remove_file(filename);
    assert!(!file_exists(filename));
    std::fs::write(filename, b"data").unwrap();
    assert!(file_exists(filename));
    let _ = std::fs::remove_file(filename);
}

#[test]
fn test_read_file_with_trailing_newline() {
    let filename = "/tmp/uemacs_test_trailing_nl.txt";
    std::fs::write(filename, b"hello\nworld\n").unwrap();

    let mut buf = setup_buffer();
    read_into_buffer(&mut buf, filename).unwrap();

    let mut lines = Vec::new();
    let mut cur = buf.head_line().next();
    while cur != buf.head {
        lines.push(buf.line(cur).unwrap().text.clone());
        cur = buf.line(cur).unwrap().next();
    }
    assert_eq!(lines, vec![b"hello".to_vec(), b"world".to_vec()]);

    let _ = std::fs::remove_file(filename);
}

#[test]
fn test_read_file_without_trailing_newline() {
    let filename = "/tmp/uemacs_test_no_trailing_nl.txt";
    std::fs::write(filename, b"hello\nworld").unwrap();

    let mut buf = setup_buffer();
    read_into_buffer(&mut buf, filename).unwrap();

    let mut lines = Vec::new();
    let mut cur = buf.head_line().next();
    while cur != buf.head {
        lines.push(buf.line(cur).unwrap().text.clone());
        cur = buf.line(cur).unwrap().next();
    }
    assert_eq!(lines, vec![b"hello".to_vec(), b"world".to_vec()]);

    let _ = std::fs::remove_file(filename);
}

#[test]
fn test_write_and_verify_content() {
    let filename = "/tmp/uemacs_test_verify.txt";
    let _ = std::fs::remove_file(filename);

    let mut buf = setup_buffer();
    add_line(&mut buf, b"line1");
    add_line(&mut buf, b"line2");

    write_from_buffer(&mut buf, filename).unwrap();
    let content = std::fs::read(filename).unwrap();
    assert_eq!(content, b"line1\nline2\n");

    let _ = std::fs::remove_file(filename);
}

#[test]
fn test_read_empty_file() {
    let filename = "/tmp/uemacs_test_empty_file.txt";
    std::fs::write(filename, b"").unwrap();

    let mut buf = setup_buffer();
    let (lc, is_new) = read_into_buffer(&mut buf, filename).unwrap();
    assert!(!is_new);
    assert_eq!(lc, 0);

    let _ = std::fs::remove_file(filename);
}

#[test]
fn test_read_file_with_blank_lines() {
    let filename = "/tmp/uemacs_test_blank_lines.txt";
    std::fs::write(filename, b"hello\n\nworld\n").unwrap();

    let mut buf = setup_buffer();
    read_into_buffer(&mut buf, filename).unwrap();

    let mut lines = Vec::new();
    let mut cur = buf.head_line().next();
    while cur != buf.head {
        lines.push(buf.line(cur).unwrap().text.clone());
        cur = buf.line(cur).unwrap().next();
    }
    assert_eq!(
        lines,
        vec![b"hello".to_vec(), b"".to_vec(), b"world".to_vec()]
    );

    let _ = std::fs::remove_file(filename);
}

#[test]
fn test_file_not_found_error() {
    let filename = "/tmp/uemacs_test_not_found_99.txt";
    let _ = std::fs::remove_file(filename);
    let mut buf = setup_buffer();
    let (_lc, is_new) = read_into_buffer(&mut buf, filename).unwrap();
    assert!(is_new);
}

#[test]
fn test_read_file_clears_flags() {
    let filename = "/tmp/uemacs_test_flags.txt";
    std::fs::write(filename, b"test").unwrap();

    let mut buf = setup_buffer();
    buf.flags = BufferFlags::INVISIBLE | BufferFlags::CHANGED | BufferFlags::TRUNCATED;
    read_into_buffer(&mut buf, filename).unwrap();

    assert_eq!(
        buf.flags & (BufferFlags::INVISIBLE | BufferFlags::CHANGED | BufferFlags::TRUNCATED),
        BufferFlags::EMPTY
    );

    let _ = std::fs::remove_file(filename);
}

#[test]
fn test_write_clears_bfchg() {
    let filename = "/tmp/uemacs_test_write_chg.txt";
    let _ = std::fs::remove_file(filename);

    let mut buf = setup_buffer();
    add_line(&mut buf, b"test");
    buf.flags |= BufferFlags::CHANGED;
    write_from_buffer(&mut buf, filename).unwrap();

    assert_eq!(buf.flags & BufferFlags::CHANGED, BufferFlags::EMPTY);

    let _ = std::fs::remove_file(filename);
}

#[test]
fn test_read_single_line_file() {
    let filename = "/tmp/uemacs_test_single_line.txt";
    std::fs::write(filename, b"single line").unwrap();

    let mut buf = setup_buffer();
    let (lc, is_new) = read_into_buffer(&mut buf, filename).unwrap();
    assert!(!is_new);
    assert_eq!(lc, 1);
    assert_eq!(buf.line_count(), 1);

    let _ = std::fs::remove_file(filename);
}

#[test]
fn test_write_from_buffer_io_error() {
    let mut buf = setup_buffer();
    add_line(&mut buf, b"test");
    let result = write_from_buffer(&mut buf, "/nonexistent/path/file.txt");
    assert!(result.is_err());
}

#[test]
fn test_read_into_buffer_io_error() {
    let mut buf = setup_buffer();
    let result = read_into_buffer(&mut buf, "/nonexistent/path/file.txt");
    assert!(result.is_ok());
    let (_, is_new) = result.unwrap();
    assert!(is_new);
}

#[test]
fn test_file_exists_function() {
    assert!(file_exists("/etc/hostname"));
    assert!(!file_exists("/nonexistent_file_12345"));
}

#[test]
fn test_read_into_buffer_permission_error() {
    let mut buf = setup_buffer();
    let result = read_into_buffer(&mut buf, "/root/.bashrc");
    if let Err(e) = result {
        assert!(matches!(e, Error::IoError));
    }
}


