use super::{Command, Editor, Error, LineOffset, Result, WindowFlags};

pub struct ForwardSearch;

impl Command for ForwardSearch {
    fn execute(&self, editor: &mut Editor, _f: bool, n: usize) -> Result<()> {
        if editor.search_pattern.is_empty() {
            return Err(Error::Abort);
        }
        let pattern = editor.search_pattern.clone();
        for _ in 0..n.max(1) {
            let (buf_id, line, offset) = {
                let win = editor.cur_window()?;
                (win.buffer_id, win.dot_line, win.dot_offset.0)
            };
            let buf = editor.buffers.get(buf_id).ok_or(Error::Abort)?;
            let use_regex = crate::search::is_regex(buf);
            let result = if use_regex {
                crate::search::find_forward_regex(buf, &pattern, line, offset)
            } else {
                crate::search::find_forward(buf, &pattern, line, offset)
            };
            match result {
                Some((new_line, new_off)) => {
                    let match_text = buf.line(new_line).map_or_else(
                        || pattern.clone(),
                        |l| {
                            let start = new_off.saturating_sub(pattern.len());
                            l.text[start..new_off.min(l.text.len())].to_vec()
                        },
                    );
                    editor.last_match = match_text;
                    let win = editor.cur_window_mut()?;
                    win.dot_line = new_line;
                    win.dot_offset = LineOffset(new_off);
                    win.set_flag(WindowFlags::MOVED);
                }
                None => return Err(Error::Abort),
            }
        }
        Ok(())
    }
}

pub struct BackwardSearch;

impl Command for BackwardSearch {
    fn execute(&self, editor: &mut Editor, _f: bool, n: usize) -> Result<()> {
        if editor.search_pattern.is_empty() {
            return Err(Error::Abort);
        }
        let pattern = editor.search_pattern.clone();
        for _ in 0..n.max(1) {
            let (buf_id, line, offset) = {
                let win = editor.cur_window()?;
                (win.buffer_id, win.dot_line, win.dot_offset.0)
            };
            let buf = editor.buffers.get(buf_id).ok_or(Error::Abort)?;
            let use_regex = crate::search::is_regex(buf);
            let result = if use_regex {
                crate::search::find_backward_regex(buf, &pattern, line, offset)
            } else {
                crate::search::find_backward(buf, &pattern, line, offset)
            };
            match result {
                Some((new_line, new_off)) => {
                    let plen = pattern.len();
                    let match_text = buf.line(new_line).map_or_else(Vec::new, |l| {
                        let end = (new_off + plen).min(l.text.len());
                        l.text[new_off..end].to_vec()
                    });
                    editor.last_match = match_text;
                    let win = editor.cur_window_mut()?;
                    win.dot_line = new_line;
                    win.dot_offset = LineOffset(new_off);
                    win.set_flag(WindowFlags::MOVED);
                }
                None => return Err(Error::Abort),
            }
        }
        Ok(())
    }
}
