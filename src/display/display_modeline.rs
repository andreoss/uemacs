use super::{
    Buffer, BufferFlags, Buffers, Cell, Display, LineId, VFCHG, Window, byte_offset_to_column,
    mode_names,
};

impl Display {
    pub(super) fn modeline(&mut self, wp: &Window, buffers: &Buffers) {
        let n = wp.top_row + wp.n_rows;
        if n >= self.nrows {
            return;
        }
        let indicator = self.indicator_or_fallback(wp, buffers);
        let bfchg = buffers
            .get(wp.buffer_id)
            .is_some_and(|b| b.flags.intersects(BufferFlags::CHANGED));
        self.write_modeline_row(n, &indicator, wp, buffers, bfchg);
    }

    fn indicator_or_fallback(&self, wp: &Window, buffers: &Buffers) -> String {
        buffers.get(wp.buffer_id).map_or_else(
            || "(--)".to_string(),
            |buf| self.position_indicator(wp, buf),
        )
    }

    fn write_modeline_row(
        &mut self,
        row: usize,
        indicator: &str,
        wp: &Window,
        buffers: &Buffers,
        bfchg: bool,
    ) {
        self.vscreen[row].flags |= VFCHG;
        let cells = &mut self.vscreen[row].cells;
        let mut col = 0;
        let rev = true;
        write_left(cells, &mut col, wp, buffers, bfchg, rev);
        while col < cells.len() {
            cells[col] = Cell {
                ch: '-',
                reverse: rev,
            };
            col += 1;
        }
        place_indicator(cells, indicator, rev);
    }

    pub(super) fn position_indicator(&self, wp: &Window, buf: &Buffer) -> String {
        let totallines = buf.line_count();
        if totallines == 0 {
            return " Emp ".to_string();
        }
        let (at_top, at_bottom) = self.window_edge_state(wp, buf);
        if at_top && at_bottom {
            return " All ".to_string();
        }
        let predlines = self.count_preceding(wp, buf);
        if predlines == 0 {
            return " Top ".to_string();
        }
        let lineno = predlines + 1;
        if lineno >= totallines {
            return " Bot ".to_string();
        }
        let ratio = (100 * lineno).checked_div(totallines).unwrap_or(0).min(99);
        format!(" {ratio:2}% ")
    }

    #[allow(clippy::unused_self)]
    fn window_edge_state(&self, wp: &Window, buf: &Buffer) -> (bool, bool) {
        let mut lp = wp.top_line;
        let mut rows = wp.n_rows;
        let mut at_bottom = false;
        while rows > 0 {
            if lp == buf.head {
                at_bottom = true;
                break;
            }
            if let Some(l) = buf.line(lp) {
                lp = l.next();
            } else {
                at_bottom = true;
                break;
            }
            rows -= 1;
        }
        let at_top = buf.line(wp.top_line).is_some_and(|l| l.prev() == buf.head);
        (at_top, at_bottom)
    }

    #[allow(clippy::unused_self)]
    fn count_preceding(&self, wp: &Window, buf: &Buffer) -> usize {
        let mut predlines = 0;
        let mut curr = buf.head_line().next();
        while curr != buf.head {
            if curr == wp.dot_line {
                break;
            }
            predlines += 1;
            if let Some(l) = buf.line(curr) {
                curr = l.next();
            } else {
                break;
            }
        }
        predlines
    }

    pub(super) fn find_screen_line(&self, wp: &Window, buffers: &Buffers) -> usize {
        let Some(buf) = buffers.get(wp.buffer_id) else {
            return wp.top_row;
        };
        let last = (wp.top_row + wp.n_rows)
            .saturating_sub(1)
            .min(self.nrows.saturating_sub(1));
        self.scan_for_dot(buf, wp.top_line, wp.dot_line, wp.top_row, last)
    }

    #[allow(clippy::unused_self)]
    fn scan_for_dot(
        &self,
        buf: &Buffer,
        top: LineId,
        dot: LineId,
        srow: usize,
        last: usize,
    ) -> usize {
        let mut lp = top;
        let mut sline = srow;
        while sline <= last {
            if lp == dot {
                return sline;
            }
            match buf.line(lp) {
                Some(l) => {
                    lp = l.next();
                    sline += 1;
                }
                None => break,
            }
        }
        last
    }

    pub(super) fn dot_column(&self, wp: &Window, buffers: &Buffers) -> usize {
        let Some(buf) = buffers.get(wp.buffer_id) else {
            return 0;
        };
        let Some(line) = buf.line(wp.dot_line) else {
            return 0;
        };
        let offset = wp.dot_offset.0.min(line.text.len());
        byte_offset_to_column(&line.text, offset, self.tab_width).min(self.ncols.saturating_sub(1))
    }
}

fn put_cell(cells: &mut [Cell], col: &mut usize, ch: char, rev: bool) {
    if *col < cells.len() {
        cells[*col] = Cell { ch, reverse: rev };
        *col += 1;
    }
}

fn put_str(cells: &mut [Cell], col: &mut usize, s: &str, rev: bool) {
    for ch in s.chars() {
        put_cell(cells, col, ch, rev);
    }
}

fn write_left(
    cells: &mut [Cell],
    col: &mut usize,
    wp: &Window,
    buffers: &Buffers,
    bfchg: bool,
    rev: bool,
) {
    let lchar = '-';
    put_cell(cells, col, lchar, rev);
    put_cell(cells, col, if bfchg { '*' } else { lchar }, rev);
    put_cell(cells, col, ' ', rev);
    put_str(cells, col, crate::util::version(), rev);
    put_str(cells, col, ": ", rev);
    write_buf_info(cells, col, wp, buffers, rev);
}

fn write_buf_info(cells: &mut [Cell], col: &mut usize, wp: &Window, buffers: &Buffers, rev: bool) {
    if let Some(buf) = buffers.get(wp.buffer_id) {
        put_str(cells, col, &buf.name, rev);
        let modes = mode_names(buf);
        put_str(cells, col, " (", rev);
        put_str(cells, col, &modes, rev);
        put_cell(cells, col, ')', rev);
        if !buf.filename.is_empty() && buf.filename != buf.name {
            put_cell(cells, col, ' ', rev);
            put_str(cells, col, &buf.filename, rev);
            put_cell(cells, col, ' ', rev);
        }
    }
}

fn place_indicator(cells: &mut [Cell], indicator: &str, rev: bool) {
    let chars: Vec<char> = indicator.chars().collect();
    let start = cells.len().saturating_sub(chars.len() + 1);
    for (i, &ch) in chars.iter().enumerate() {
        let pos = start + i;
        if pos < cells.len() {
            cells[pos] = Cell { ch, reverse: rev };
        }
    }
}
