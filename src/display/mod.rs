pub use crate::buffer::{Buffer, Buffers};
pub use crate::core::{BufferFlags, LineId, Mode, Result, WindowFlags, WindowId};
pub use crate::terminal::TerminalBackend;
pub use crate::window::{Window, Windows};

pub const VFCHG: u8 = 0x01;

pub const MODE_FLAGS: &[(Mode, &str)] = &[
    (Mode::WRAP, "Wrap"),
    (Mode::C_MODE, "Cmode"),
    (Mode::SPELL, "Spell"),
    (Mode::EXACT, "Exact"),
    (Mode::VIEW, "View"),
    (Mode::OVERWRITE, "Over"),
    (Mode::MAGIC, "Magic"),
    (Mode::AUTO_SAVE, "Asave"),
];

pub fn mode_names(buf: &Buffer) -> String {
    let mut s = String::new();
    if buf.flags.intersects(BufferFlags::TRUNCATED) {
        s.push_str("Truncated");
    }
    for &(flag, name) in MODE_FLAGS {
        if buf.mode.intersects(flag) {
            if !s.is_empty() {
                s.push(' ');
            }
            s.push_str(name);
        }
    }
    s
}

fn is_wide(cp: u32) -> bool {
    let ranges: &[(u32, u32)] = &[
        (0x1100, 0x115F),
        (0x2E80, 0x9FFF),
        (0xAC00, 0xD7AF),
        (0xF900, 0xFAFF),
        (0xFE10, 0xFE19),
        (0xFE30, 0xFE6F),
        (0xFF01, 0xFF60),
        (0xFFE0, 0xFFE6),
        (0x20000, 0x2FFFD),
        (0x30000, 0x3FFFD),
    ];
    ranges.iter().any(|&(lo, hi)| cp >= lo && cp <= hi)
}

fn is_single_width(cp: u32) -> bool {
    if cp < 0x80
        || (0xA0..=0xD7FF).contains(&cp)
        || (0xE000..=0xFFFD).contains(&cp)
        || (0x1_0000..0x11_0000).contains(&cp)
    {
        return !is_wide(cp);
    }
    true
}

pub fn char_display_width(ch: char) -> usize {
    if is_single_width(ch as u32) { 1 } else { 2 }
}

pub struct Cell {
    pub ch: char,
    pub reverse: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            reverse: false,
        }
    }
}

pub struct VideoRow {
    pub cells: Vec<Cell>,
    pub flags: u8,
}

pub struct Display {
    nrows: usize,
    ncols: usize,
    vscreen: Vec<VideoRow>,
    pscreen: Vec<VideoRow>,
    pub sgarbf: bool,
    pub tab_width: usize,
    pub isearch_highlight: Option<(LineId, usize, usize)>,
}

fn advance_column_for_multibyte(text: &[u8], i: usize, col: usize) -> (usize, usize) {
    std::str::from_utf8(&text[i..]).map_or_else(
        |_| (i + 1, col + 1),
        |s| s.chars().next().map_or_else(
            || (i + 1, col + 1),
            |ch| (i + ch.len_utf8(), col + char_display_width(ch)),
        ),
    )
}

fn advance_column_for_single_byte(
    text: &[u8],
    i: usize,
    col: usize,
    tab_width: usize,
) -> (usize, usize) {
    let c = text[i];
    if c == b'\t' {
        (i + 1, ((col / tab_width) + 1) * tab_width)
    } else if c.is_ascii() && (c as char).is_ascii_control() && c != b'\n' {
        (i + 1, col + 2)
    } else if c == b'\n' {
        (i + 1, col)
    } else if c < 0x80 {
        (i + 1, col + 1)
    } else {
        advance_column_for_multibyte(text, i, col)
    }
}

pub fn byte_offset_to_column(text: &[u8], offset: usize, tab_width: usize) -> usize {
    let mut col = 0;
    let mut i = 0;
    let limit = offset.min(text.len());
    while i < limit {
        (i, col) = advance_column_for_single_byte(text, i, col, tab_width);
    }
    col
}

impl Display {
    pub const fn nrows(&self) -> usize {
        self.nrows
    }

    pub fn resize(&mut self, nrows: usize, ncols: usize) {
        if nrows == self.nrows && ncols == self.ncols {
            return;
        }
        let ncols = ncols.max(1);
        let nrows = nrows.max(1);
        self.nrows = nrows;
        self.ncols = ncols;
        let make_row = || VideoRow {
            cells: (0..ncols).map(|_| Cell::default()).collect(),
            flags: 0,
        };
        self.vscreen = (0..nrows).map(|_| make_row()).collect();
        self.pscreen = (0..nrows).map(|_| make_row()).collect();
        self.sgarbf = true;
    }
}

mod display_modeline;
mod display_render;
mod display_update;

#[cfg(test)]
mod tests;
