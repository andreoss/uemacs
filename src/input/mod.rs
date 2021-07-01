use crate::core::{CONTROL, KeyCode, META};
use crate::terminal::{Key, TerminalBackend};

pub struct KeyEvent {
    pub f: bool,
    pub n: usize,
    pub key: Key,
}

impl KeyEvent {
    const fn plain(key: Key) -> Self {
        Self {
            f: false,
            n: 1,
            key,
        }
    }

    fn prefixed(mflag: i8, n: usize, key: Key) -> Self {
        Self {
            f: true,
            n: if mflag < 0 { n.max(1) } else { n },
            key,
        }
    }
}

fn ascii_digit(kc: KeyCode) -> Option<usize> {
    let c = (kc.0 & 0xff) as u8;
    c.is_ascii_digit().then(|| (c - b'0') as usize)
}

fn try_meta_argument<T: TerminalBackend>(term: &mut T, kc: KeyCode, raw: &Key) -> Option<KeyEvent> {
    let base = (kc.0 & 0xff) as u8 as char;
    if !base.is_ascii_digit() && base != '-' {
        return None;
    }
    let mut mflag = 1i8;
    let mut n = if base == '-' {
        mflag = -1;
        0
    } else {
        (base as u8 - b'0') as usize
    };
    loop {
        let Some(next) = term.get_key() else {
            return Some(KeyEvent::prefixed(mflag, n, raw.clone()));
        };
        let nkc: KeyCode = next.clone().into();
        if let Some(d) = ascii_digit(nkc) {
            n = n * 10 + d;
            continue;
        }
        if (nkc.0 & 0xff) as u8 == b'-' && n == 0 {
            mflag = -1;
            continue;
        }
        return Some(KeyEvent::prefixed(mflag, n, next));
    }
}

fn read_universal_argument<T: TerminalBackend>(term: &mut T) -> KeyEvent {
    let mut n: usize = 4;
    let mut mflag = 0i8;
    loop {
        let Some(next) = term.get_key() else {
            return KeyEvent {
                f: true,
                n,
                key: Key::Unknown(0),
            };
        };
        let nkc: KeyCode = next.clone().into();
        if nkc == KeyCode(CONTROL | 0x15) {
            n = n.saturating_mul(4);
            continue;
        }
        if nkc.0 & META != 0 {
            return KeyEvent::prefixed(mflag, n, next);
        }
        if let Some(d) = ascii_digit(nkc) {
            if mflag == 0 {
                n = 0;
                mflag = 1;
            }
            n = n * 10 + d;
            continue;
        }
        if (nkc.0 & 0xff) as u8 == b'-' && mflag == 0 {
            n = 0;
            mflag = -1;
            continue;
        }
        return KeyEvent::prefixed(mflag, n, next);
    }
}

pub fn getkey<T: TerminalBackend>(term: &mut T) -> KeyEvent {
    let Some(raw) = term.get_key() else {
        return KeyEvent::plain(Key::Unknown(0));
    };
    let kc: KeyCode = raw.clone().into();
    if kc == KeyCode(0x1b) {
        return KeyEvent::plain(Key::Control('['));
    }
    if kc.0 & META != 0 {
        if let Some(ev) = try_meta_argument(term, kc, &raw) {
            return ev;
        }
        return KeyEvent::plain(raw);
    }
    if kc == KeyCode(CONTROL | 0x15) {
        return read_universal_argument(term);
    }
    KeyEvent::plain(raw)
}

#[cfg(test)]
mod tests;
