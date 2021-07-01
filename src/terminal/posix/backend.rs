use super::{Error, Key, PosixTerminal, Result, TerminalBackend, Write, io};

const fn configure_raw_mode(raw: &mut libc::termios) {
    raw.c_iflag &= !(libc::IGNBRK
        | libc::BRKINT
        | libc::IGNPAR
        | libc::PARMRK
        | libc::INPCK
        | libc::ISTRIP
        | libc::INLCR
        | libc::IGNCR
        | libc::ICRNL
        | libc::IXON);
    raw.c_oflag &= !(libc::OPOST);
    raw.c_lflag &= !(libc::ISIG
        | libc::ICANON
        | libc::ECHO
        | libc::ECHOE
        | libc::ECHOK
        | libc::ECHONL
        | libc::ECHOCTL
        | libc::ECHOPRT
        | libc::ECHOKE
        | libc::IEXTEN);
    raw.c_cc[libc::VMIN] = 1;
    raw.c_cc[libc::VTIME] = 0;
}

impl TerminalBackend for PosixTerminal {
    fn open(&mut self) -> Result<()> {
        let mut raw: libc::termios = unsafe { std::mem::zeroed() };
        if unsafe { libc::tcgetattr(libc::STDIN_FILENO, &raw mut raw) } < 0 {
            return Err(Error::IoError);
        }
        self.orig = raw;
        configure_raw_mode(&mut raw);
        if unsafe { libc::tcsetattr(libc::STDIN_FILENO, libc::TCSADRAIN, &raw const raw) } < 0 {
            return Err(Error::IoError);
        }
        self.opened = true;
        Ok(())
    }

    fn close(&mut self) -> Result<()> {
        if self.opened {
            let ret = unsafe {
                libc::tcsetattr(libc::STDIN_FILENO, libc::TCSADRAIN, &raw const self.orig)
            };
            if ret < 0 {
                return Err(Error::IoError);
            }
            self.opened = false;
        }
        Ok(())
    }

    fn dimensions(&self) -> (usize, usize) {
        let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
        let ret = unsafe { libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut ws) };
        if ret == 0 && ws.ws_row > 0 && ws.ws_col > 0 {
            (ws.ws_row as usize, ws.ws_col as usize)
        } else {
            (24, 80)
        }
    }

    fn move_to(&mut self, row: usize, col: usize) {
        write!(self.out_buf, "\x1b[{};{}H", row + 1, col + 1).unwrap();
    }

    fn clear_eol(&mut self) {
        self.out_buf.extend_from_slice(b"\x1b[K");
    }

    fn beep(&mut self) {
        self.out_buf.push(b'\x07');
    }

    fn set_reverse(&mut self, on: bool) {
        if on {
            self.out_buf.extend_from_slice(b"\x1b[7m");
        } else {
            self.out_buf.extend_from_slice(b"\x1b[27m");
        }
    }

    fn put_char(&mut self, c: char) {
        let mut buf = [0u8; 4];
        let s = c.encode_utf8(&mut buf);
        self.out_buf.extend_from_slice(s.as_bytes());
    }

    fn get_key(&mut self) -> Option<Key> {
        let byte = self.read_byte()?;
        match byte {
            0x1b => self.read_escape(),
            0x7f => Some(Key::Backspace),
            b'\r' => Some(Key::Enter),
            b'\t' => Some(Key::Tab),
            b if b < 0x20 => Some(Key::Control((b + b'@') as char)),
            b if b & 0x80 != 0 => self.read_utf8(b),
            b => Some(Key::Char(b as char)),
        }
    }

    fn flush(&mut self) -> Result<()> {
        if !self.out_buf.is_empty() {
            let mut stdout = io::stdout().lock();
            stdout
                .write_all(&self.out_buf)
                .map_err(|_e| Error::IoError)?;
            stdout.flush().map_err(|_e| Error::IoError)?;
            self.out_buf.clear();
        }
        Ok(())
    }
}
