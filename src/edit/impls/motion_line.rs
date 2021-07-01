use super::{CmdFlags, Command, Editor, Error, LineOffset, Result, WindowFlags};

pub struct ForwardLine;

impl Command for ForwardLine {
    fn execute(&self, editor: &mut Editor, _f: bool, mut n: usize) -> Result<()> {
        let (buffer_id, dot_line) = {
            let win = editor.cur_window()?;
            (win.buffer_id, win.dot_line)
        };
        {
            let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
            if buf.next_line(dot_line).is_none() {
                return Err(Error::Abort);
            }
        }

        let dot_offset = editor.cur_window()?.dot_offset.0;
        if !editor.last_flag.intersects(CmdFlags::LINE_MOVE) {
            editor.cur_goal = dot_offset;
        }
        editor.this_flag |= CmdFlags::LINE_MOVE;

        let mut current_line = dot_line;
        while n > 0 {
            let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
            match buf.next_line(current_line) {
                Some(next) => current_line = next,
                None => break,
            }
            n -= 1;
        }

        let goal = editor.cur_goal;
        let new_offset = {
            let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
            let len = buf.line_len(current_line).ok_or(Error::Abort)?;
            if goal > len { len } else { goal }
        };

        let win = editor.cur_window_mut()?;
        win.dot_line = current_line;
        win.dot_offset = LineOffset(new_offset);
        win.set_flag(WindowFlags::MOVED);
        Ok(())
    }
}

pub struct BackwardLine;

impl Command for BackwardLine {
    fn execute(&self, editor: &mut Editor, _f: bool, mut n: usize) -> Result<()> {
        let (buffer_id, dot_line) = {
            let win = editor.cur_window()?;
            (win.buffer_id, win.dot_line)
        };
        {
            let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
            if buf.prev_line(dot_line).is_none() {
                return Err(Error::Abort);
            }
        }

        let dot_offset = editor.cur_window()?.dot_offset.0;
        if !editor.last_flag.intersects(CmdFlags::LINE_MOVE) {
            editor.cur_goal = dot_offset;
        }
        editor.this_flag |= CmdFlags::LINE_MOVE;

        let mut current_line = dot_line;
        while n > 0 {
            let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
            match buf.prev_line(current_line) {
                Some(prev) => current_line = prev,
                None => break,
            }
            n -= 1;
        }

        let goal = editor.cur_goal;
        let new_offset = {
            let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
            let len = buf.line_len(current_line).ok_or(Error::Abort)?;
            if goal > len { len } else { goal }
        };

        let win = editor.cur_window_mut()?;
        win.dot_line = current_line;
        win.dot_offset = LineOffset(new_offset);
        win.set_flag(WindowFlags::MOVED);
        Ok(())
    }
}

pub struct GotoBob;

impl Command for GotoBob {
    fn execute(&self, editor: &mut Editor, _f: bool, _n: usize) -> Result<()> {
        let buffer_id = editor.cur_window()?.buffer_id;
        let first_line = {
            let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
            buf.nth_line(0).ok_or(Error::Abort)?
        };
        let win = editor.cur_window_mut()?;
        win.dot_line = first_line;
        win.dot_offset = LineOffset(0);
        win.set_flag(WindowFlags::MOVED);
        Ok(())
    }
}

pub struct GotoEob;

impl Command for GotoEob {
    fn execute(&self, editor: &mut Editor, _f: bool, _n: usize) -> Result<()> {
        let buffer_id = editor.cur_window()?.buffer_id;
        let last_line = {
            let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
            let count = buf.line_count();
            if count == 0 {
                buf.head
            } else {
                buf.nth_line(count - 1).ok_or(Error::Abort)?
            }
        };
        let len = {
            let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
            buf.line_len(last_line).ok_or(Error::Abort)?
        };
        let win = editor.cur_window_mut()?;
        win.dot_line = last_line;
        win.dot_offset = LineOffset(len);
        win.set_flag(WindowFlags::MOVED);
        Ok(())
    }
}

pub struct GotoLine;

impl Command for GotoLine {
    fn execute(&self, editor: &mut Editor, f: bool, n: usize) -> Result<()> {
        if !f || n == 0 {
            GotoEob.execute(editor, f, n)?;
            return Ok(());
        }
        let buffer_id = editor.cur_window()?.buffer_id;
        let target_line = {
            let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
            buf.nth_line(n - 1).ok_or(Error::Abort)?
        };
        let win = editor.cur_window_mut()?;
        win.dot_line = target_line;
        win.dot_offset = LineOffset(0);
        win.set_flag(WindowFlags::MOVED);
        Ok(())
    }
}
