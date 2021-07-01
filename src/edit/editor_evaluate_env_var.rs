use super::{Bindings, Display, Editor, Result, TerminalBackend};

impl Editor {
    pub(crate) fn evaluate_env_var(&self, name: &str) -> String {
        match name {
            "fillcol" => self.fillcol.to_string(),
            "tab" => self.tab_width.to_string(),
            "scroll" => self.scroll_amount.to_string(),
            "gmode" => self.gmode.bits().to_string(),
            "acount" => self.gacount.to_string(),
            "asave" => self.gasave.to_string(),
            "curcol" => self
                .current_window()
                .map(|w| w.dot_offset.0.to_string())
                .unwrap_or_default(),
            "curline" | "linenumber" => self
                .current_window()
                .and_then(|w| {
                    let buf = self.buffers.get(w.buffer_id)?;
                    let mut count = 1;
                    let mut cur = buf.head_line().next();
                    while cur != buf.head && cur != w.dot_line {
                        count += 1;
                        cur = buf
                            .line(cur)
                            .map_or(buf.head, super::super::line::Line::next);
                    }
                    Some(count.to_string())
                })
                .unwrap_or_default(),
            "bufname" | "cbufname" => self
                .current_window()
                .and_then(|w| self.buffers.get(w.buffer_id))
                .map(|b| b.name.clone())
                .unwrap_or_default(),
            "fname" | "cfname" => self
                .current_window()
                .and_then(|w| self.buffers.get(w.buffer_id))
                .map(|b| b.filename.clone())
                .unwrap_or_default(),
            "version" => "1.0".to_string(),
            "progname" => "uemacs".to_string(),
            "search" => String::from_utf8_lossy(&self.search_pattern).to_string(),
            "replace" => String::from_utf8_lossy(&self.replace_pattern).to_string(),
            "match" => String::from_utf8_lossy(&self.last_match).to_string(),
            "kill" => String::from_utf8_lossy(&self.kill_buffer).to_string(),
            "cmode" => self
                .current_window()
                .and_then(|w| self.buffers.get(w.buffer_id))
                .map(|b| b.mode.bits().to_string())
                .unwrap_or_default(),
            "lwidth" => self
                .current_window()
                .and_then(|w| {
                    let buf = self.buffers.get(w.buffer_id)?;
                    let line = buf.line(w.dot_line)?;
                    Some(line.text.len().to_string())
                })
                .unwrap_or_default(),
            "line" => self
                .current_window()
                .and_then(|w| {
                    let buf = self.buffers.get(w.buffer_id)?;
                    let line = buf.line(w.dot_line)?;
                    Some(String::from_utf8_lossy(&line.text).to_string())
                })
                .unwrap_or_default(),
            "wline" => self
                .current_window()
                .map(|w| w.n_rows.to_string())
                .unwrap_or_default(),
            "cwline" => self
                .current_window()
                .map(|w| (w.top_row + 1).to_string())
                .unwrap_or_default(),
            "curchar" => self
                .current_window()
                .and_then(|w| {
                    let buf = self.buffers.get(w.buffer_id)?;
                    let line = buf.line(w.dot_line)?;
                    let off = w.dot_offset.0;
                    if off >= line.text.len() {
                        Some(u32::from(b'\n'))
                    } else {
                        Some(u32::from(line.text[off]))
                    }
                })
                .map(|c| c.to_string())
                .unwrap_or_default(),
            "lastkey" | "rval" | "ram" | "seed" | "gflags" | "overlap" | "jump" => "0".to_string(),
            "pagelen" => self.screen_rows.to_string(),
            "curwidth" => self.screen_cols.to_string(),
            "status" | "discmd" | "disinp" => "T".to_string(),
            "target" => self.cur_goal.to_string(),
            "flicker" | "debug" | "pending" => "F".to_string(),
            "sres" => "NORMAL".to_string(),
            "tpause" => "100".to_string(),
            _ => String::new(),
        }
    }

    pub(crate) fn execute_lines_with_directives<T: TerminalBackend>(
        &mut self,
        lines: &[String],
        term: &mut T,
        display: &mut Display,
        bindings: &Bindings,
    ) -> Result<()> {
        let was_recording = self.recording_macro;
        self.recording_macro = false;
        let result = self.run_directive_lines(lines, term, display, bindings);
        self.recording_macro = was_recording;
        result
    }
}
