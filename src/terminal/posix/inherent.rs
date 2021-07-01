use super::{Key, PosixTerminal};

fn retriable_read_error() -> bool {
    let err = std::io::Error::last_os_error();
    let code = err.raw_os_error();
    code == Some(libc::EINTR) || code == Some(libc::EAGAIN) || code == Some(libc::EWOULDBLOCK)
}

fn read_stdin(buf: &mut [u8; 32]) -> isize {
    unsafe {
        libc::read(
            libc::STDIN_FILENO,
            buf.as_mut_ptr().cast::<std::ffi::c_void>(),
            32,
        )
    }
}

const fn utf8_byte_count(first: u8) -> usize {
    if first & 0xe0 == 0xc0 {
        2
    } else if first & 0xf0 == 0xe0 {
        3
    } else if first & 0xf8 == 0xf0 {
        4
    } else {
        1
    }
}

fn csi_param_to_key(n: usize) -> Key {
    match n {
        1 | 7 => Key::Home,
        2 => Key::Insert,
        3 => Key::Delete,
        4 | 8 => Key::End,
        5 => Key::PageUp,
        6 => Key::PageDown,
        11..=15 => Key::Function(u8::try_from(n - 10).unwrap()),
        17..=21 => Key::Function(u8::try_from(n - 11).unwrap()),
        23..=24 => Key::Function(u8::try_from(n - 12).unwrap()),
        _ => Key::Unknown(0x100 + u32::try_from(n).unwrap()),
    }
}

impl PosixTerminal {
    fn fill_input_buffer(&mut self) -> Option<()> {
        let mut buf = [0u8; 32];
        let n = loop {
            let n = read_stdin(&mut buf);
            if n > 0 {
                break n;
            }
            if n == 0 {
                break 0;
            }
            if !retriable_read_error() {
                break n;
            }
        };
        if n <= 0 {
            return None;
        }
        self.in_buf = buf[..usize::try_from(n).unwrap()].to_vec();
        self.in_pos = 0;
        Some(())
    }

    pub(super) fn read_byte(&mut self) -> Option<u8> {
        if self.in_pos >= self.in_buf.len() {
            self.fill_input_buffer()?;
        }
        let b = self.in_buf[self.in_pos];
        self.in_pos += 1;
        Some(b)
    }

    fn peek_byte(&self) -> Option<u8> {
        if self.in_pos < self.in_buf.len() {
            Some(self.in_buf[self.in_pos])
        } else {
            None
        }
    }

    pub(super) fn read_utf8(&mut self, first: u8) -> Option<Key> {
        let expected = utf8_byte_count(first);
        let mut buf = [first, 0, 0, 0];
        for b in buf.iter_mut().take(expected).skip(1) {
            *b = self.read_byte()?;
        }
        if let Ok(s) = std::str::from_utf8(&buf[..expected]) {
            if let Some(c) = s.chars().next() {
                return Some(Key::Char(c));
            }
        }
        Some(Key::Unknown(u32::from(first)))
    }

    fn parse_csi_digits(&mut self, first: u8) -> usize {
        let mut n = (first - b'0') as usize;
        while let Some(b) = self.peek_byte() {
            if !b.is_ascii_digit() {
                break;
            }
            self.read_byte();
            n = n * 10 + (b - b'0') as usize;
        }
        if self.peek_byte() == Some(b'~') {
            self.read_byte();
        }
        n
    }

    fn read_csi_sequence(&mut self) -> Option<Key> {
        let cmd = self.read_byte()?;
        match cmd {
            b'A' => Some(Key::Up),
            b'B' => Some(Key::Down),
            b'C' => Some(Key::Right),
            b'D' => Some(Key::Left),
            b'H' => Some(Key::Home),
            b'F' => Some(Key::End),
            b'Z' => Some(Key::Tab),
            b'0'..=b'9' => Some(csi_param_to_key(self.parse_csi_digits(cmd))),
            _ => Some(Key::Unknown(0x100 + u32::from(cmd))),
        }
    }

    pub(super) fn read_escape(&mut self) -> Option<Key> {
        let Some(next) = self.read_byte() else {
            return Some(Key::Escape);
        };
        if next == b'[' || next == b'O' {
            self.read_csi_sequence()
        } else if next == 0x1b {
            Some(Key::Escape)
        } else if next == 0x7f {
            Some(Key::Meta('\x7f'))
        } else if next < 0x20 {
            Some(Key::MetaControl((next + b'@') as char))
        } else {
            Some(Key::Meta(next as char))
        }
    }
}
