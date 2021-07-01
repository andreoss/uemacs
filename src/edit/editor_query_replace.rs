use super::{
    Display, Editor, Error, LineOffset, Result, TerminalBackend, WindowFlags, qr_match_start,
    qr_preview,
};

impl Editor {
    pub(super) fn query_replace<T: TerminalBackend>(
        &mut self,
        term: &mut T,
        display: &mut Display,
    ) -> Result<()> {
        let pattern_str = self.minibuffer_readline(term, display, "Query replace: ")?;
        if pattern_str.is_empty() {
            return Ok(());
        }
        let pattern = pattern_str.as_bytes().to_vec();
        self.search_pattern.clone_from(&pattern);

        let replacement_str = self.minibuffer_readline(term, display, "with: ")?;
        let replacement = replacement_str.as_bytes().to_vec();
        self.replace_pattern.clone_from(&replacement);

        loop {
            let (buf_id, line, offset) = {
                let win = self.cur_window()?;
                (win.buffer_id, win.dot_line, win.dot_offset.0)
            };
            let buf = self.buffers.get(buf_id).ok_or(Error::Abort)?;
            let Some((end_line, end_off)) =
                crate::search::find_forward(buf, &pattern, line, offset)
            else {
                break;
            };

            let (start_line, start_off) = qr_match_start(
                self.buffers.get(buf_id).ok_or(Error::Abort)?,
                pattern.len(),
                line,
                end_line,
                end_off,
            )?;

            let preview = qr_preview(
                self.buffers.get(buf_id).ok_or(Error::Abort)?,
                start_line,
                start_off,
                end_line,
                end_off,
            )?;

            let prompt = format!("replace '{preview}' ? (y/n)");
            let answer = self.minibuffer_readline(term, display, &prompt)?;

            match answer.as_str() {
                "y" => {
                    self.query_replace_apply(
                        buf_id,
                        start_line,
                        start_off,
                        end_line,
                        end_off,
                        &replacement,
                    )?;
                }
                "n" => {
                    let win = self.cur_window_mut()?;
                    win.dot_line = end_line;
                    win.dot_offset = LineOffset(end_off);
                    win.set_flag(WindowFlags::MOVED);
                }
                _ => {
                    return Ok(());
                }
            }
        }

        Ok(())
    }
}
