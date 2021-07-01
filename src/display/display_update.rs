use super::{
    Buffer, Buffers, Cell, Display, LineId, Result, TerminalBackend, VideoRow, Window, WindowFlags,
    WindowId, Windows,
};

impl Display {
    pub fn new(term: &dyn TerminalBackend) -> Self {
        let (nrows, ncols) = term.dimensions();
        Self::with_size(nrows, ncols)
    }

    pub fn with_size(nrows: usize, ncols: usize) -> Self {
        let ncols = ncols.max(1);
        let nrows = nrows.max(1);
        Self {
            nrows,
            ncols,
            vscreen: make_screen(nrows, ncols),
            pscreen: make_screen(nrows, ncols),
            sgarbf: true,
            tab_width: 8,
            isearch_highlight: None,
        }
    }

    pub fn update<T: TerminalBackend>(
        &mut self,
        windows: &mut Windows,
        buffers: &Buffers,
        term: &mut T,
    ) -> Result<()> {
        let ids: Vec<WindowId> = windows
            .iter()
            .filter(|w| !w.flags.is_empty())
            .map(|w| w.id)
            .collect();
        for id in ids {
            self.update_window(windows, buffers, id);
        }
        self.finish_update(term, windows, buffers)
    }

    fn update_window(&mut self, windows: &mut Windows, buffers: &Buffers, id: WindowId) {
        let wp = windows.get_mut(id).unwrap();
        self.reframe(wp, buffers);
        if wp
            .flags
            .intersects(WindowFlags::HARD | WindowFlags::FORCE | WindowFlags::MOVED)
        {
            self.updall(wp, buffers);
        } else if wp.flags.intersects(WindowFlags::EDITED) {
            self.updone(wp, buffers);
        }
        if wp
            .flags
            .intersects(WindowFlags::MODE_LINE | WindowFlags::MOVED | WindowFlags::EDITED)
        {
            self.modeline(wp, buffers);
        }
        wp.flags = WindowFlags::EMPTY;
    }

    fn finish_update<T: TerminalBackend>(
        &mut self,
        term: &mut T,
        windows: &Windows,
        buffers: &Buffers,
    ) -> Result<()> {
        if self.sgarbf {
            self.handle_garbage_or_resize_lines(term);
        }
        self.flush_virtual_to_physical(term)?;
        self.sgarbf = false;
        if let Some(win) = windows.current() {
            term.move_to(
                self.find_screen_line(win, buffers),
                self.dot_column(win, buffers),
            );
            term.flush()?;
        }
        Ok(())
    }

    pub(super) fn reframe(&self, wp: &mut Window, buffers: &Buffers) {
        if !wp.flags.intersects(WindowFlags::FORCE) && self.dot_is_visible(wp, buffers) {
            return;
        }
        wp.flags |= WindowFlags::MODE_LINE;
        self.recenter_dot(wp, buffers);
        wp.flags |= WindowFlags::HARD;
        wp.flags &= !WindowFlags::FORCE;
    }

    #[allow(clippy::unused_self)]
    fn dot_is_visible(&self, wp: &Window, buffers: &Buffers) -> bool {
        if let Some(buf) = buffers.get(wp.buffer_id) {
            let mut lp = wp.top_line;
            for _ in 0..wp.n_rows {
                if lp == wp.dot_line {
                    return true;
                }
                if lp == buf.head {
                    break;
                }
                if let Some(l) = buf.line(lp) {
                    lp = l.next();
                } else {
                    break;
                }
            }
        }
        false
    }

    #[allow(clippy::unused_self)]
    fn recenter_dot(&self, wp: &mut Window, buffers: &Buffers) {
        let half = wp.n_rows / 2;
        let mut lp = wp.dot_line;
        for _ in 0..half {
            if let Some(buf) = buffers.get(wp.buffer_id) {
                if let Some(l) = buf.line(lp) {
                    let prev = l.prev();
                    if prev == buf.head {
                        break;
                    }
                    lp = prev;
                } else {
                    break;
                }
            }
        }
        wp.top_line = lp;
    }

    pub(super) fn updone(&mut self, wp: &Window, buffers: &Buffers) {
        let sline = self.find_screen_line(wp, buffers);
        if let Some(buf) = buffers.get(wp.buffer_id) {
            self.render_line(sline, wp.dot_line, buf);
        }
    }

    pub(super) fn updall(&mut self, wp: &Window, buffers: &Buffers) {
        let Some(buf) = buffers.get(wp.buffer_id) else {
            return;
        };
        let mut lp = skip_head(wp.top_line, buf);
        let last = wp.top_row + wp.n_rows;
        let mut sline = wp.top_row;
        while sline < last && sline < self.nrows {
            if lp == buf.head {
                self.clear_row(sline);
                sline += 1;
                continue;
            }
            self.render_line(sline, lp, buf);
            if let Some(l) = buf.line(lp) {
                lp = l.next();
            }
            sline += 1;
        }
    }

    pub(super) fn resize_screens(&mut self) {
        let ncols = self.ncols;
        self.vscreen.resize_with(self.nrows, || VideoRow {
            cells: (0..ncols).map(|_| Cell::default()).collect(),
            flags: 0,
        });
        self.pscreen.resize_with(self.nrows, || VideoRow {
            cells: (0..ncols).map(|_| Cell::default()).collect(),
            flags: 0,
        });
    }
}

fn make_screen(rows: usize, cols: usize) -> Vec<VideoRow> {
    (0..rows)
        .map(|_| VideoRow {
            cells: (0..cols).map(|_| Cell::default()).collect(),
            flags: 0,
        })
        .collect()
}

fn skip_head(top_line: LineId, buf: &Buffer) -> LineId {
    if top_line == buf.head {
        if let Some(l) = buf.line(top_line) {
            return l.next();
        }
    }
    top_line
}
