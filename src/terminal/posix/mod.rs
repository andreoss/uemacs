pub(super) use super::{Key, TerminalBackend};
pub(super) use crate::core::{Error, Result};
pub(super) use std::io::{self, Write};

pub struct PosixTerminal {
    orig: libc::termios,
    opened: bool,
    out_buf: Vec<u8>,
    in_buf: Vec<u8>,
    in_pos: usize,
}

impl PosixTerminal {
    pub const fn new() -> Self {
        Self {
            orig: unsafe { std::mem::zeroed() },
            opened: false,
            out_buf: Vec::new(),
            in_buf: Vec::new(),
            in_pos: 0,
        }
    }
}

impl Drop for PosixTerminal {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

#[cfg(test)]
impl PosixTerminal {
    pub fn with_input(data: &[u8]) -> Self {
        Self {
            orig: unsafe { std::mem::zeroed() },
            opened: false,
            out_buf: Vec::new(),
            in_buf: data.to_vec(),
            in_pos: 0,
        }
    }
}

mod backend;
mod inherent;
