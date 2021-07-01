use super::{CmdFlags, Command, Editor, Error, LineOffset, Result, WindowFlags, kill_n_chars};

pub struct KillText;

impl Command for KillText {
    fn execute(&self, editor: &mut Editor, f: bool, n: usize) -> Result<()> {
        if !editor.last_flag.intersects(CmdFlags::KILL) {
            editor.kdelete();
        }
        editor.this_flag |= CmdFlags::KILL;

        let (buffer_id, dot_line, dot_offset) = {
            let win = editor.cur_window()?;
            (win.buffer_id, win.dot_line, win.dot_offset.0)
        };

        let chunk = if !f {
            let len = editor
                .buffers
                .get(buffer_id)
                .ok_or(Error::Abort)?
                .line_len(dot_line)
                .ok_or(Error::Abort)?;
            let c = len.saturating_sub(dot_offset);
            if c == 0 { 1 } else { c }
        } else if n == 0 {
            {
                let win = editor.cur_window_mut()?;
                win.dot_offset = LineOffset(0);
            }
            dot_offset
        } else {
            let mut total = 0;
            let mut current = dot_line;
            for i in 0..n {
                let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
                let line_len = buf.line_len(current).ok_or(Error::Abort)?;
                if i == 0 {
                    total += line_len.saturating_sub(dot_offset) + 1;
                } else {
                    total += line_len + 1;
                }
                if i + 1 < n {
                    match buf.next_line(current) {
                        Some(next) => current = next,
                        None => return Err(Error::Abort),
                    }
                }
            }
            total
        };

        kill_n_chars(editor, chunk)?;

        editor
            .current_window_mut()
            .ok_or(Error::Abort)?
            .set_flag(WindowFlags::HARD);
        Ok(())
    }
}

pub struct KillLine;

impl Command for KillLine {
    fn execute(&self, editor: &mut Editor, f: bool, n: usize) -> Result<()> {
        KillText.execute(editor, f, n)
    }
}
