use super::{
    BufferId, Display, Editor, Error, Key, LineId, LineOffset, Result, TerminalBackend, WindowFlags,
};

struct IsearchState {
    pattern: Vec<u8>,
    history: Vec<(usize, LineId, usize)>,
    forward: bool,
    last_match: Option<(LineId, usize)>,
    orig_buf_id: BufferId,
    orig_line: LineId,
    orig_offset: usize,
}

impl Editor {
    pub(super) fn isearch<T: TerminalBackend>(
        &mut self,
        term: &mut T,
        display: &mut Display,
        initial_forward: bool,
    ) -> Result<()> {
        let (orig_buf_id, orig_line, orig_offset) = {
            let win = self.cur_window()?;
            (win.buffer_id, win.dot_line, win.dot_offset.0)
        };
        let mut state = IsearchState {
            pattern: Vec::new(),
            history: Vec::new(),
            forward: initial_forward,
            last_match: Some((orig_line, orig_offset)),
            orig_buf_id,
            orig_line,
            orig_offset,
        };

        display.isearch_highlight = None;

        loop {
            let dir_str = if state.forward { "" } else { "-back" };
            let prompt = format!(
                "I-search{}: {}",
                dir_str,
                String::from_utf8_lossy(&state.pattern)
            );
            display.write_echo(term, &prompt)?;

            let Some(key) = term.get_key() else {
                display.write_echo(term, "")?;
                if let Some((line, offset)) = state.last_match {
                    let win = self.cur_window_mut()?;
                    win.dot_line = line;
                    win.dot_offset = LineOffset(offset);
                    win.set_flag(WindowFlags::MOVED);
                }
                display.isearch_highlight = None;
                return Ok(());
            };

            if self.isearch_handle_key(term, display, &key, &mut state)? {
                return Ok(());
            }
        }
    }

    fn isearch_handle_key<T: TerminalBackend>(
        &mut self,
        term: &mut T,
        display: &mut Display,
        key: &Key,
        state: &mut IsearchState,
    ) -> Result<bool> {
        match key {
            Key::Enter | Key::Escape => {
                display.write_echo(term, "")?;
                if let Some((line, offset)) = state.last_match {
                    self.search_pattern.clone_from(&state.pattern);
                    let win = self.cur_window_mut()?;
                    win.dot_line = line;
                    win.dot_offset = LineOffset(offset);
                    win.set_flag(WindowFlags::MOVED);
                }
                display.isearch_highlight = None;
                return Ok(true);
            }
            Key::Control('G') => {
                display.write_echo(term, "")?;
                let win = self.cur_window_mut()?;
                win.dot_line = state.orig_line;
                win.dot_offset = LineOffset(state.orig_offset);
                win.set_flag(WindowFlags::MOVED);
                display.isearch_highlight = None;
                return Ok(true);
            }
            Key::Control('R') => {
                state.forward = false;
                self.isearch_reverse(term, display, state)?;
            }
            Key::Control('S') => {
                state.forward = true;
                self.isearch_forward(term, display, state)?;
            }
            Key::Backspace | Key::Delete => match state.history.pop() {
                Some((added, prev_line, prev_offset)) => {
                    for _ in 0..added {
                        state.pattern.pop();
                    }
                    state.last_match = Some((prev_line, prev_offset));
                    let win = self.cur_window_mut()?;
                    win.dot_line = prev_line;
                    win.dot_offset = LineOffset(prev_offset);
                    win.set_flag(WindowFlags::MOVED);
                    display.isearch_highlight = if state.pattern.is_empty() {
                        None
                    } else {
                        Some((
                            prev_line,
                            prev_offset.saturating_sub(state.pattern.len()),
                            prev_offset,
                        ))
                    };
                    display.update(&mut self.windows, &self.buffers, term)?;
                }
                None => {
                    term.beep();
                }
            },
            Key::Char(c) => {
                self.isearch_add_char(term, display, *c, state)?;
            }
            _ => {
                term.beep();
            }
        }
        Ok(false)
    }

    fn isearch_reverse<T: TerminalBackend>(
        &mut self,
        term: &mut T,
        display: &mut Display,
        state: &mut IsearchState,
    ) -> Result<()> {
        if state.pattern.is_empty() {
            return Ok(());
        }
        let buf = self.buffers.get(state.orig_buf_id).ok_or(Error::Abort)?;
        let (search_line, search_offset) = state
            .last_match
            .unwrap_or((state.orig_line, state.orig_offset));
        let mut result =
            crate::search::find_backward(buf, &state.pattern, search_line, search_offset);
        let mut wrapped = false;
        if result.is_none() {
            let last_line = buf.head_line().prev();
            if last_line != buf.head {
                let last_len = buf.line_len(last_line).unwrap_or(0);
                result = crate::search::find_backward(buf, &state.pattern, last_line, last_len);
                wrapped = true;
            }
        }
        if let Some((line, offset)) = result {
            state.last_match = Some((line, offset));
            let win = self.cur_window_mut()?;
            win.dot_line = line;
            win.dot_offset = LineOffset(offset);
            win.set_flag(WindowFlags::MOVED);
            display.isearch_highlight =
                Some((line, offset.saturating_sub(state.pattern.len()), offset));
            if wrapped {
                display.write_echo(
                    term,
                    &format!(
                        "I-search-back: {} (wrapped)",
                        String::from_utf8_lossy(&state.pattern)
                    ),
                )?;
            }
            display.update(&mut self.windows, &self.buffers, term)?;
        } else {
            term.beep();
        }
        Ok(())
    }

    fn isearch_forward<T: TerminalBackend>(
        &mut self,
        term: &mut T,
        display: &mut Display,
        state: &mut IsearchState,
    ) -> Result<()> {
        if state.pattern.is_empty() {
            return Ok(());
        }
        let buf = self.buffers.get(state.orig_buf_id).ok_or(Error::Abort)?;
        let (search_line, search_offset) = state
            .last_match
            .unwrap_or((state.orig_line, state.orig_offset));
        let mut result =
            crate::search::find_forward(buf, &state.pattern, search_line, search_offset);
        let mut wrapped = false;
        if result.is_none() {
            let first_line = buf.head_line().next();
            if first_line != buf.head {
                result = crate::search::find_forward(buf, &state.pattern, first_line, 0);
                wrapped = true;
            }
        }
        if let Some((line, offset)) = result {
            state.last_match = Some((line, offset));
            let win = self.cur_window_mut()?;
            win.dot_line = line;
            win.dot_offset = LineOffset(offset);
            win.set_flag(WindowFlags::MOVED);
            display.isearch_highlight =
                Some((line, offset.saturating_sub(state.pattern.len()), offset));
            if wrapped {
                display.write_echo(
                    term,
                    &format!(
                        "I-search: {} (wrapped)",
                        String::from_utf8_lossy(&state.pattern)
                    ),
                )?;
            }
            display.update(&mut self.windows, &self.buffers, term)?;
        } else {
            term.beep();
        }
        Ok(())
    }

    fn isearch_add_char<T: TerminalBackend>(
        &mut self,
        term: &mut T,
        display: &mut Display,
        c: char,
        state: &mut IsearchState,
    ) -> Result<()> {
        let mut bytes = [0u8; 4];
        let s = c.encode_utf8(&mut bytes);
        let prev_match = state
            .last_match
            .unwrap_or((state.orig_line, state.orig_offset));
        state.history.push((s.len(), prev_match.0, prev_match.1));
        state.pattern.extend_from_slice(s.as_bytes());
        let buf = self.buffers.get(state.orig_buf_id).ok_or(Error::Abort)?;
        let search_line = state.last_match.map_or(state.orig_line, |(l, _)| l);
        let search_offset = state.last_match.map_or(state.orig_offset, |(_, o)| o);
        let result = if state.forward {
            crate::search::find_forward(buf, &state.pattern, search_line, search_offset)
        } else {
            crate::search::find_backward(buf, &state.pattern, search_line, search_offset)
        };
        if let Some((line, offset)) = result {
            self.isearch_apply_match(term, display, state, line, offset)?;
        } else {
            let buf = self.buffers.get(state.orig_buf_id).ok_or(Error::Abort)?;
            let result = if state.forward {
                crate::search::find_forward(buf, &state.pattern, state.orig_line, state.orig_offset)
            } else {
                crate::search::find_backward(
                    buf,
                    &state.pattern,
                    state.orig_line,
                    state.orig_offset,
                )
            };
            if let Some((line, offset)) = result {
                self.isearch_apply_match(term, display, state, line, offset)?;
            } else {
                term.beep();
            }
        }
        Ok(())
    }

    fn isearch_apply_match<T: TerminalBackend>(
        &mut self,
        term: &mut T,
        display: &mut Display,
        state: &mut IsearchState,
        line: LineId,
        offset: usize,
    ) -> Result<()> {
        state.last_match = Some((line, offset));
        let win = self.cur_window_mut()?;
        win.dot_line = line;
        win.dot_offset = LineOffset(offset);
        win.set_flag(WindowFlags::MOVED);
        display.isearch_highlight =
            Some((line, offset.saturating_sub(state.pattern.len()), offset));
        display.update(&mut self.windows, &self.buffers, term)?;
        Ok(())
    }
}
