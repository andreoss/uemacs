use super::{
    Buffer, Cell, Display, LineId, Result, TerminalBackend, VFCHG, byte_offset_to_column,
    char_display_width,
};

impl Display {
    pub(super) fn render_line(&mut self, row: usize, line_id: LineId, buf: &Buffer) {
        if row >= self.nrows {
            return;
        }
        let ncols = self.ncols;
        let tab_width = self.tab_width;
        let isearch = self.isearch_highlight;
        let row_data = &mut self.vscreen[row];
        row_data.flags |= VFCHG;
        for cell in &mut row_data.cells {
            *cell = Cell::default();
        }
        if let Some(line) = buf.line(line_id) {
            render_line_text(
                &mut row_data.cells,
                ncols,
                tab_width,
                isearch,
                line_id,
                &line.text,
            );
        }
    }

    pub(super) fn clear_row(&mut self, row: usize) {
        if row < self.nrows {
            let row_data = &mut self.vscreen[row];
            row_data.flags |= VFCHG;
            for cell in &mut row_data.cells {
                *cell = Cell::default();
            }
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn echo_text(&self) -> String {
        if self.nrows == 0 {
            return String::new();
        }
        let row = self.nrows - 1;
        self.vscreen[row].cells.iter().map(|c| c.ch).collect()
    }

    pub fn write_echo<T: TerminalBackend>(&mut self, term: &mut T, text: &str) -> Result<()> {
        if self.nrows == 0 {
            return Ok(());
        }
        let row = self.nrows - 1;
        let row_data = &mut self.vscreen[row];
        row_data.flags |= VFCHG;
        for cell in &mut row_data.cells {
            *cell = Cell::default();
        }
        put_echo_text(&mut row_data.cells, text);
        self.flush_virtual_to_physical(term)
    }

    pub(super) fn flush_virtual_to_physical<T: TerminalBackend>(
        &mut self,
        term: &mut T,
    ) -> Result<()> {
        let sgarbf = self.sgarbf;
        let ncols = self.ncols;
        for row in 0..self.nrows {
            self.update_pscreen_row(term, row, ncols, sgarbf);
        }
        term.flush()?;
        Ok(())
    }

    fn update_pscreen_row<T: TerminalBackend>(
        &mut self,
        term: &mut T,
        row: usize,
        ncols: usize,
        sgarbf: bool,
    ) {
        let vflags = self.vscreen[row].flags;
        if vflags & VFCHG == 0 && !sgarbf {
            self.vscreen[row].flags = 0;
            return;
        }
        let dirty =
            sgarbf || cells_differ(&self.vscreen[row].cells, &self.pscreen[row].cells, ncols);
        if dirty {
            emit_row(term, &self.vscreen[row].cells, ncols, row);
        }
        copy_cells_from(
            &mut self.pscreen[row].cells,
            &self.vscreen[row].cells,
            ncols,
        );
        self.pscreen[row].flags = 0;
        self.vscreen[row].flags = 0;
    }

    pub(super) fn handle_garbage_or_resize_lines<T: TerminalBackend>(&mut self, term: &T) {
        let (rows, _) = term.dimensions();
        if rows != self.nrows {
            self.nrows = rows;
            self.resize_screens();
        }
        for row in 0..self.nrows {
            self.vscreen[row].flags |= VFCHG;
            self.pscreen[row].flags |= VFCHG;
        }
    }
}

fn put_echo_text(cells: &mut [Cell], text: &str) {
    let mut col = 0;
    let ncols = cells.len();
    for ch in text.chars() {
        if col < ncols {
            cells[col] = Cell { ch, reverse: false };
            col += 1;
        }
    }
}

fn render_line_text(
    cells: &mut [Cell],
    ncols: usize,
    tab_width: usize,
    isearch: Option<(LineId, usize, usize)>,
    line_id: LineId,
    text: &[u8],
) {
    let (col, _) = render_chars(cells, text, ncols, tab_width);
    if col >= ncols && ncols > 0 {
        cells[ncols - 1] = Cell {
            ch: '$',
            reverse: false,
        };
    }
    if let Some((hl_line, hl_start, hl_end)) = isearch {
        if hl_line == line_id {
            let start = byte_offset_to_column(text, hl_start, tab_width);
            let end = byte_offset_to_column(text, hl_end, tab_width);
            for c in cells.iter_mut().take(end.min(ncols)).skip(start) {
                c.reverse = true;
            }
        }
    }
}

fn render_chars(cells: &mut [Cell], text: &[u8], ncols: usize, tab_width: usize) -> (usize, usize) {
    let mut col = 0;
    let mut i = 0;
    while i < text.len() && col < ncols {
        let (ni, ncol) = step_render(cells, text, i, col, ncols, tab_width);
        i = ni;
        col = ncol;
    }
    (col, i)
}

fn step_render(
    cells: &mut [Cell],
    text: &[u8],
    i: usize,
    col: usize,
    ncols: usize,
    tab_width: usize,
) -> (usize, usize) {
    let c = text[i];
    if c == b'\t' {
        (i + 1, render_tab_cells(cells, col, ncols, tab_width))
    } else if c.is_ascii() && (c as char).is_ascii_control() && c != b'\n' {
        (i + 1, render_control_cells(cells, col, c, ncols))
    } else if c == b'\n' {
        (i + 1, col)
    } else if c < 0x80 {
        (i + 1, render_ascii_cells(cells, col, c))
    } else {
        render_utf8_cells(cells, text, i, col, ncols)
    }
}

fn render_tab_cells(cells: &mut [Cell], col: usize, ncols: usize, tab_width: usize) -> usize {
    let next = ((col / tab_width) + 1) * tab_width;
    let mut c = col;
    while c < next && c < ncols {
        cells[c] = Cell {
            ch: ' ',
            reverse: false,
        };
        c += 1;
    }
    c
}

fn render_control_cells(cells: &mut [Cell], col: usize, c: u8, ncols: usize) -> usize {
    if col + 1 < ncols {
        cells[col] = Cell {
            ch: '^',
            reverse: false,
        };
        cells[col + 1] = Cell {
            ch: (c ^ 0x40) as char,
            reverse: false,
        };
        return col + 2;
    }
    col
}

fn render_ascii_cells(cells: &mut [Cell], col: usize, c: u8) -> usize {
    if col < cells.len() {
        cells[col] = Cell {
            ch: c as char,
            reverse: false,
        };
        return col + 1;
    }
    col
}

fn render_utf8_cells(
    cells: &mut [Cell],
    text: &[u8],
    i: usize,
    col: usize,
    ncols: usize,
) -> (usize, usize) {
    if let Ok(s) = std::str::from_utf8(&text[i..]) {
        if let Some(ch) = s.chars().next() {
            let w = ch.len_utf8();
            let dw = char_display_width(ch);
            if col + dw <= ncols {
                cells[col] = Cell { ch, reverse: false };
            }
            (i + w, col + dw)
        } else {
            cells[col] = Cell { ch: '?', reverse: false };
            (i + 1, col + 1)
        }
    } else {
        cells[col] = Cell { ch: '?', reverse: false };
        (i + 1, col + 1)
    }
}

fn cells_differ(a: &[Cell], b: &[Cell], ncols: usize) -> bool {
    (0..ncols).any(|c| a[c].ch != b[c].ch || a[c].reverse != b[c].reverse)
}

fn copy_cells_from(dst: &mut [Cell], src: &[Cell], ncols: usize) {
    for c in 0..ncols {
        dst[c].ch = src[c].ch;
        dst[c].reverse = src[c].reverse;
    }
}

fn emit_row<T: TerminalBackend>(term: &mut T, cells: &[Cell], ncols: usize, row: usize) {
    term.move_to(row, 0);
    let mut rev = false;
    for cell in cells.iter().take(ncols) {
        if cell.reverse != rev {
            term.set_reverse(cell.reverse);
            rev = cell.reverse;
        }
        if cell.ch == '\0' {
            term.put_char(' ');
        } else {
            term.put_char(cell.ch);
        }
    }
    if rev {
        term.set_reverse(false);
    }
    term.clear_eol();
}
