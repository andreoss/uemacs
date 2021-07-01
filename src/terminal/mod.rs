use crate::core::{CONTROL, KeyCode, META, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Control(char),
    Meta(char),
    MetaControl(char),
    Up,
    Down,
    Left,
    Right,
    PageUp,
    PageDown,
    Home,
    End,
    Delete,
    Insert,
    Backspace,
    Tab,
    Enter,
    #[cfg_attr(not(test), allow(dead_code))]
    Escape,
    Function(u8),
    Unknown(u32),
}

fn special_key_code(key: &Key) -> Option<KeyCode> {
    match key {
        Key::Up => Some(KeyCode(0x101)),
        Key::Down => Some(KeyCode(0x102)),
        Key::Left => Some(KeyCode(0x103)),
        Key::Right => Some(KeyCode(0x104)),
        Key::PageUp => Some(KeyCode(0x105)),
        Key::PageDown => Some(KeyCode(0x106)),
        Key::Home => Some(KeyCode(0x107)),
        Key::End => Some(KeyCode(0x108)),
        Key::Delete => Some(KeyCode(0x109)),
        Key::Insert => Some(KeyCode(0x10a)),
        Key::Function(n) => Some(KeyCode(0x10b + (u32::from(*n) - 1))),
        _ => None,
    }
}

fn ascii_key_code(key: &Key) -> Option<KeyCode> {
    match key {
        Key::Backspace => Some(KeyCode(0x7f)),
        Key::Tab => Some(KeyCode(u32::from(b'\t'))),
        Key::Enter => Some(KeyCode(u32::from(b'\r'))),
        Key::Escape => Some(KeyCode(0x1b)),
        _ => None,
    }
}

impl From<Key> for KeyCode {
    fn from(key: Key) -> Self {
        if let Some(c) = special_key_code(&key) {
            return c;
        }
        if let Some(c) = ascii_key_code(&key) {
            return c;
        }
        match key {
            Key::Char(c) => Self(c as u32),
            Key::Control(c) => Self(CONTROL | (c as u32 & 0x1f)),
            Key::Meta(c) => Self(META | (c as u32)),
            Key::MetaControl(c) => Self(META | CONTROL | (c as u32 & 0x1f)),
            Key::Unknown(code) => Self(code),
            _ => unreachable!(),
        }
    }
}

pub trait TerminalBackend {
    fn open(&mut self) -> Result<()>;
    fn close(&mut self) -> Result<()>;
    fn dimensions(&self) -> (usize, usize);
    fn move_to(&mut self, row: usize, col: usize);
    fn clear_eol(&mut self);
    fn beep(&mut self);
    fn set_reverse(&mut self, on: bool);
    fn put_char(&mut self, c: char);
    fn get_key(&mut self) -> Option<Key>;
    fn flush(&mut self) -> Result<()>;
}

pub mod mock;
pub mod posix;

#[cfg(test)]
mod tests;
