use super::{Command, Editor, Error, Result, WindowFlags};

pub struct SetMark;

impl Command for SetMark {
    fn execute(&self, editor: &mut Editor, _f: bool, _n: usize) -> Result<()> {
        let (dot_line, dot_offset) = {
            let win = editor.cur_window()?;
            (win.dot_line, win.dot_offset)
        };
        let win = editor.cur_window_mut()?;
        win.set_mark(dot_line, dot_offset);
        Ok(())
    }
}

pub struct SwapMark;

impl Command for SwapMark {
    fn execute(&self, editor: &mut Editor, _f: bool, _n: usize) -> Result<()> {
        let (dot_line, dot_offset) = {
            let win = editor.cur_window()?;
            (win.dot_line, win.dot_offset)
        };
        let (mark_line, mark_offset) = {
            let win = editor.cur_window()?;
            win.mark().ok_or(Error::Abort)?
        };
        let win = editor.cur_window_mut()?;
        win.set_dot(mark_line, mark_offset);
        win.set_mark(dot_line, dot_offset);
        win.set_flag(WindowFlags::MOVED);
        Ok(())
    }
}
