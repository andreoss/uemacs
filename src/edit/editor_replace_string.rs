use super::{
    Buffer, BufferFlags, Display, Editor, Error, LineId, LineOffset, Result, TerminalBackend,
    UndoAction, WindowFlags,
};

impl Editor {
    pub(super) fn replace_string<T: TerminalBackend>(
        &mut self,
        term: &mut T,
        display: &mut Display,
    ) -> Result<()> {
        let pattern = self.minibuffer_readline(term, display, "Replace: ")?;
        if pattern.is_empty() {
            return Ok(());
        }
        let replacement = self.minibuffer_readline(term, display, "With: ")?;
        self.search_pattern = pattern.as_bytes().to_vec();
        self.replace_pattern = replacement.as_bytes().to_vec();
        let (buf_id, dot_line, dot_offset) = {
            let win = self.cur_window()?;
            (win.buffer_id, win.dot_line, win.dot_offset.0)
        };
        let pat = pattern.as_bytes().to_vec();
        let repl = replacement.as_bytes().to_vec();
        let mut numsub = 0usize;
        let mut cur_line = dot_line;
        let mut cur_off = dot_offset;
        let mut undo_actions: Vec<UndoAction> = Vec::new();
        loop {
            let found = {
                let buf = self.buffers.get(buf_id).ok_or(Error::Abort)?;
                let mut scan_line = cur_line;
                let mut scan_off = cur_off;
                let mut found_pos = None;
                loop {
                    if scan_line == buf.head {
                        break;
                    }
                    let scan = buf.line(scan_line).ok_or(Error::Abort)?;
                    let remaining = scan.text.len().saturating_sub(scan_off);
                    if remaining >= pat.len()
                        && scan.text[scan_off..scan_off + pat.len()] == pat[..]
                    {
                        found_pos = Some((scan_line, scan_off));
                        break;
                    }
                    scan_off += 1;
                    if scan_off > scan.text.len() {
                        scan_line = scan.next();
                        scan_off = 0;
                    }
                }
                found_pos
            };
            match found {
                Some((line, off)) => {
                    undo_actions.push(UndoAction::Delete {
                        line,
                        offset: off,
                        data: pat.clone(),
                    });
                    undo_actions.push(UndoAction::Insert {
                        line,
                        offset: off,
                        data: repl.clone(),
                    });
                    let buf = self.buffers.get_mut(buf_id).ok_or(Error::Abort)?;
                    let l = buf.line_mut(line).ok_or(Error::Abort)?;
                    l.text.splice(off..off + pat.len(), repl.iter().copied());
                    cur_line = line;
                    cur_off = off + repl.len();
                    numsub += 1;
                }
                None => break,
            }
        }
        if !undo_actions.is_empty() {
            self.push_undo(undo_actions);
        }
        if numsub > 0 {
            let buf = self.buffers.get_mut(buf_id).ok_or(Error::Abort)?;
            buf.flags |= BufferFlags::CHANGED;
        }
        display.write_echo(term, &format!("{numsub} substitutions"))?;
        let win = self.cur_window_mut()?;
        win.dot_line = cur_line;
        win.dot_offset = LineOffset(cur_off);
        win.set_flag(WindowFlags::HARD);
        Ok(())
    }

    pub(super) fn goto_matching_fence<T: TerminalBackend>(&mut self, term: &mut T) -> Result<()> {
        let (buf_id, dot_line, dot_offset) = {
            let win = self.cur_window()?;
            (win.buffer_id, win.dot_line, win.dot_offset.0)
        };
        let buf = self.buffers.get(buf_id).ok_or(Error::Abort)?;
        let line = buf.line(dot_line).ok_or(Error::Abort)?;
        let start_ch = if dot_offset >= line.text.len() {
            b'\n'
        } else {
            line.text[dot_offset]
        };
        let (match_ch, forward) = match start_ch {
            b'(' => (b')', true),
            b')' => (b'(', false),
            b'{' => (b'}', true),
            b'}' => (b'{', false),
            b'[' => (b']', true),
            b']' => (b'[', false),
            _ => {
                term.beep();
                return Ok(());
            }
        };
        let step = |buf: &Buffer, line: &mut LineId, off: &mut usize| -> bool {
            if forward {
                let len = buf.line(*line).map_or(0, |l| l.text.len());
                if *off >= len {
                    let next = buf
                        .line(*line)
                        .map_or(*line, super::super::line::Line::next);
                    if next == buf.head {
                        return false;
                    }
                    *line = next;
                    *off = 0;
                } else {
                    *off += 1;
                }
            } else if *off == 0 {
                let prev = buf
                    .line(*line)
                    .map_or(*line, super::super::line::Line::prev);
                if prev == buf.head {
                    return false;
                }
                *line = prev;
                *off = buf.line(*line).map_or(0, |l| l.text.len());
            } else {
                *off -= 1;
            }
            true
        };
        let mut scan_line = dot_line;
        let mut scan_off = dot_offset;
        if !step(buf, &mut scan_line, &mut scan_off) {
            term.beep();
            return Ok(());
        }
        let mut count: usize = 1;
        loop {
            let scan = buf.line(scan_line).ok_or(Error::Abort)?;
            let c = if scan_off >= scan.text.len() {
                b'\n'
            } else {
                scan.text[scan_off]
            };
            if c == start_ch {
                count += 1;
            } else if c == match_ch {
                count -= 1;
                if count == 0 {
                    break;
                }
            }
            if !step(buf, &mut scan_line, &mut scan_off) {
                term.beep();
                return Ok(());
            }
        }
        let win = self.cur_window_mut()?;
        win.dot_line = scan_line;
        win.dot_offset = LineOffset(scan_off);
        win.set_flag(WindowFlags::MOVED);
        Ok(())
    }
}
