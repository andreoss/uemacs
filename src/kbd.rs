use crate::core::{CONTROL, KeyCode, META};
use crate::terminal::Key;

pub fn key_name(key: &Key) -> String {
    match key {
        Key::Char(c) => c.to_string(),
        Key::Control(c) => format!("C-{}", c.to_ascii_uppercase()),
        Key::Meta(c) => format!("M-{c}"),
        Key::MetaControl(c) => format!("M-C-{}", c.to_ascii_uppercase()),
        _ => key_name_special(key),
    }
}

fn key_name_special(key: &Key) -> String {
    match key {
        Key::Up => "Up".to_string(),
        Key::Down => "Down".to_string(),
        Key::Left => "Left".to_string(),
        Key::Right => "Right".to_string(),
        Key::PageUp => "PageUp".to_string(),
        Key::PageDown => "PageDown".to_string(),
        Key::Home => "Home".to_string(),
        Key::End => "End".to_string(),
        _ => key_name_special_ext(key),
    }
}

fn key_name_special_ext(key: &Key) -> String {
    match key {
        Key::Delete => "Delete".to_string(),
        Key::Insert => "Insert".to_string(),
        Key::Backspace => "Backspace".to_string(),
        Key::Tab => "Tab".to_string(),
        Key::Enter => "Enter".to_string(),
        Key::Escape => "Escape".to_string(),
        Key::Function(n) => format!("F{n}"),
        Key::Unknown(code) => format!("0x{code:X}"),
        _ => unreachable!(),
    }
}

pub fn key_code_display(kc: KeyCode) -> String {
    const SPECIAL_KEYS: [(u32, &str); 10] = [
        (0x101, "Up"),
        (0x102, "Down"),
        (0x103, "Left"),
        (0x104, "Right"),
        (0x105, "PageUp"),
        (0x106, "PageDown"),
        (0x107, "Home"),
        (0x108, "End"),
        (0x109, "Delete"),
        (0x10a, "Insert"),
    ];
    match kc.0 {
        k if k & META != 0 && k & CONTROL != 0 => format!(
            "M-C-{}",
            char::from_u32(((k & 0x1f) + u32::from(b'@')).min(0x7f)).unwrap_or('?')
        ),
        k if k & META != 0 => format!("M-{}", char::from(u8::try_from(k & 0xff).unwrap_or(b'?'))),
        k if k & CONTROL != 0 => format!(
            "C-{}",
            char::from_u32((k & 0x1f) + u32::from(b'@')).unwrap_or('?')
        ),
        k if k == u32::from(b'\t') => "Tab".to_string(),
        k if k == u32::from(b'\r') => "Enter".to_string(),
        0x7f => "Backspace".to_string(),
        0x1b => "Escape".to_string(),
        k if k == u32::from(b' ') => "SPC".to_string(),
        k if k >= 0x10b => format!("F{}", k - 0x10a),
        _ => {
            for &(code, name) in &SPECIAL_KEYS {
                if kc.0 == code {
                    return name.to_string();
                }
            }
            char::from(u8::try_from(kc.0 & 0xff).unwrap_or(b'?')).to_string()
        }
    }
}

pub fn key_to_bytes(key: &Key) -> Option<Vec<u8>> {
    match key {
        Key::Char(c) => {
            let mut buf = [0u8; 4];
            let s = c.encode_utf8(&mut buf);
            Some(s.as_bytes().to_vec())
        }
        Key::Control(c) => Some(vec![*c as u8 & 0x1f]),
        Key::Tab => Some(vec![b'\t']),
        Key::Backspace => Some(vec![0x7f]),
        Key::Escape => Some(vec![0x1b]),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_name_char() {
        assert_eq!(key_name(&Key::Char('a')), "a");
        assert_eq!(key_name(&Key::Char('A')), "A");
        assert_eq!(key_name(&Key::Char(' ')), " ");
    }

    #[test]
    fn test_key_name_control() {
        assert_eq!(key_name(&Key::Control('a')), "C-A");
        assert_eq!(key_name(&Key::Control('A')), "C-A");
        assert_eq!(key_name(&Key::Control('?')), "C-?");
    }

    #[test]
    fn test_key_name_meta() {
        assert_eq!(key_name(&Key::Meta('x')), "M-x");
        assert_eq!(key_name(&Key::Meta('X')), "M-X");
    }

    #[test]
    fn test_key_name_meta_control() {
        assert_eq!(key_name(&Key::MetaControl('f')), "M-C-F");
        assert_eq!(key_name(&Key::MetaControl('F')), "M-C-F");
    }

    #[test]
    fn test_key_name_special() {
        assert_eq!(key_name(&Key::Up), "Up");
        assert_eq!(key_name(&Key::Down), "Down");
        assert_eq!(key_name(&Key::Left), "Left");
        assert_eq!(key_name(&Key::Right), "Right");
        assert_eq!(key_name(&Key::PageUp), "PageUp");
        assert_eq!(key_name(&Key::PageDown), "PageDown");
        assert_eq!(key_name(&Key::Home), "Home");
        assert_eq!(key_name(&Key::End), "End");
        assert_eq!(key_name(&Key::Delete), "Delete");
        assert_eq!(key_name(&Key::Insert), "Insert");
        assert_eq!(key_name(&Key::Backspace), "Backspace");
        assert_eq!(key_name(&Key::Tab), "Tab");
        assert_eq!(key_name(&Key::Enter), "Enter");
        assert_eq!(key_name(&Key::Escape), "Escape");
    }

    #[test]
    fn test_key_name_function() {
        assert_eq!(key_name(&Key::Function(1)), "F1");
        assert_eq!(key_name(&Key::Function(12)), "F12");
    }

    #[test]
    fn test_key_name_unknown() {
        assert_eq!(key_name(&Key::Unknown(0x1b)), "0x1B");
        assert_eq!(key_name(&Key::Unknown(0xff)), "0xFF");
    }

    #[test]
    fn test_key_code_display_control() {
        assert_eq!(key_code_display(KeyCode(crate::core::CONTROL | 1)), "C-A");
        assert_eq!(key_code_display(KeyCode(crate::core::CONTROL | 2)), "C-B");
        assert_eq!(key_code_display(KeyCode(crate::core::CONTROL | 0x1f)), "C-_");
    }

    #[test]
    fn test_key_code_display_meta() {
        assert_eq!(key_code_display(KeyCode(crate::core::META | u32::from(b'x'))), "M-x");
        assert_eq!(key_code_display(KeyCode(crate::core::META | u32::from(b'X'))), "M-X");
    }

    #[test]
    fn test_key_code_display_meta_control() {
        let kc = KeyCode(crate::core::META | crate::core::CONTROL | 1);
        assert_eq!(key_code_display(kc), "M-C-A");
    }

    #[test]
    fn test_key_code_display_special() {
        assert_eq!(key_code_display(KeyCode(0x101)), "Up");
        assert_eq!(key_code_display(KeyCode(0x102)), "Down");
        assert_eq!(key_code_display(KeyCode(0x103)), "Left");
        assert_eq!(key_code_display(KeyCode(0x104)), "Right");
        assert_eq!(key_code_display(KeyCode(0x105)), "PageUp");
        assert_eq!(key_code_display(KeyCode(0x106)), "PageDown");
        assert_eq!(key_code_display(KeyCode(0x107)), "Home");
        assert_eq!(key_code_display(KeyCode(0x108)), "End");
        assert_eq!(key_code_display(KeyCode(0x109)), "Delete");
        assert_eq!(key_code_display(KeyCode(0x10a)), "Insert");
    }

    #[test]
    fn test_key_code_display_function() {
        assert_eq!(key_code_display(KeyCode(0x10b)), "F1");
        assert_eq!(key_code_display(KeyCode(0x10d)), "F3");
    }

    #[test]
    fn test_key_code_display_punctuation() {
        assert_eq!(key_code_display(KeyCode(u32::from(b'\t'))), "Tab");
        assert_eq!(key_code_display(KeyCode(u32::from(b'\r'))), "Enter");
        assert_eq!(key_code_display(KeyCode(0x7f)), "Backspace");
        assert_eq!(key_code_display(KeyCode(0x1b)), "Escape");
        assert_eq!(key_code_display(KeyCode(u32::from(b' '))), "SPC");
    }

    #[test]
    fn test_key_code_display_ascii() {
        assert_eq!(key_code_display(KeyCode(u32::from(b'a'))), "a");
        assert_eq!(key_code_display(KeyCode(u32::from(b'Z'))), "Z");
    }

    #[test]
    fn test_key_to_bytes_char() {
        assert_eq!(key_to_bytes(&Key::Char('a')), Some(vec![b'a']));
        assert_eq!(key_to_bytes(&Key::Char(' ')), Some(vec![b' ']));
        assert_eq!(key_to_bytes(&Key::Char('\n')), Some(vec![b'\n']));
    }

    #[test]
    fn test_key_to_bytes_control() {
        assert_eq!(key_to_bytes(&Key::Control('a')), Some(vec![0x01]));
        assert_eq!(key_to_bytes(&Key::Control('A')), Some(vec![0x01]));
    }

    #[test]
    fn test_key_to_bytes_tab() {
        assert_eq!(key_to_bytes(&Key::Tab), Some(vec![b'\t']));
    }

    #[test]
    fn test_key_to_bytes_backspace() {
        assert_eq!(key_to_bytes(&Key::Backspace), Some(vec![0x7f]));
    }

    #[test]
    fn test_key_to_bytes_escape() {
        assert_eq!(key_to_bytes(&Key::Escape), Some(vec![0x1b]));
    }

    #[test]
    fn test_key_to_bytes_unsupported() {
        assert_eq!(key_to_bytes(&Key::Up), None);
        assert_eq!(key_to_bytes(&Key::Function(1)), None);
        assert_eq!(key_to_bytes(&Key::Enter), None);
        assert_eq!(key_to_bytes(&Key::Meta('x')), None);
        assert_eq!(key_to_bytes(&Key::MetaControl('f')), None);
    }
}
