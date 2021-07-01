use super::{Command, Editor, Error, LineOffset, Result, WindowFlags};

pub struct ForwardPage;

impl Command for ForwardPage {
    fn execute(&self, editor: &mut Editor, f: bool, mut n: usize) -> Result<()> {
        if !f {
            let win = editor.cur_window()?;
            n = (win.n_rows / 3 * 2).max(1);
        }
        let buffer_id = editor.cur_window()?.buffer_id;
        let mut top_line = editor.cur_window()?.top_line;
        while n > 0 {
            let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
            match buf.next_line(top_line) {
                Some(next) => top_line = next,
                None => break,
            }
            n -= 1;
        }
        let win = editor.cur_window_mut()?;
        win.top_line = top_line;
        win.dot_line = top_line;
        win.dot_offset = LineOffset(0);
        win.set_flag(WindowFlags::HARD);
        Ok(())
    }
}

pub struct BackwardPage;

impl Command for BackwardPage {
    fn execute(&self, editor: &mut Editor, f: bool, mut n: usize) -> Result<()> {
        if !f {
            let win = editor.cur_window()?;
            n = (win.n_rows / 3 * 2).max(1);
        }
        let buffer_id = editor.cur_window()?.buffer_id;
        let mut top_line = editor.cur_window()?.top_line;
        while n > 0 {
            let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
            match buf.prev_line(top_line) {
                Some(prev) => top_line = prev,
                None => break,
            }
            n -= 1;
        }
        let win = editor.cur_window_mut()?;
        win.top_line = top_line;
        win.dot_line = top_line;
        win.dot_offset = LineOffset(0);
        win.set_flag(WindowFlags::HARD);
        Ok(())
    }
}
