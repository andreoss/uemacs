use super::{
    BackwardChar, BufferFlags, Command, Editor, Error, ForwardChar, ForwardDelete, GotoEol,
    LineOffset, Mode, Result, UndoAction, WindowFlags,
};

pub struct OpenLine;

impl Command for OpenLine {
    fn execute(&self, editor: &mut Editor, _f: bool, n: usize) -> Result<()> {
        let (buffer_id, dot_line, dot_offset) = {
            let win = editor.cur_window()?;
            (win.buffer_id, win.dot_line, win.dot_offset.0)
        };
        let mut split_line = dot_line;
        let mut split_offset = dot_offset;
        for _ in 0..n {
            let new_line = {
                let buf = editor.buffers.get_mut(buffer_id).ok_or(Error::Abort)?;
                let new_line = buf.split_line(split_line, split_offset);
                buf.flags |= BufferFlags::CHANGED;
                new_line
            };
            editor.push_undo(vec![UndoAction::Split {
                line: split_line,
                offset: split_offset,
                new_line,
            }]);
            split_line = new_line;
            split_offset = 0;
        }
        let win = editor.cur_window_mut()?;
        win.dot_line = dot_line;
        win.dot_offset = LineOffset(dot_offset);
        win.set_flag(WindowFlags::HARD);
        Ok(())
    }
}

pub struct InsertNewline;

impl Command for InsertNewline {
    fn execute(&self, editor: &mut Editor, f: bool, n: usize) -> Result<()> {
        let (buffer_id, dot_line) = {
            let win = editor.cur_window()?;
            (win.buffer_id, win.dot_line)
        };
        {
            let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
            if buf.mode.intersects(Mode::C_MODE) && dot_line != buf.head {
                return NewlineAndIndent.execute(editor, f, n);
            }
        }
        let dot_offset = {
            let win = editor.cur_window()?;
            win.dot_offset.0
        };
        let mut current_line = dot_line;
        let mut current_offset = dot_offset;
        for _ in 0..n.max(1) {
            let (orig_line, split_at) = (current_line, current_offset);
            current_line = {
                let buf = editor.buffers.get_mut(buffer_id).ok_or(Error::Abort)?;
                let new_line = buf.split_line(orig_line, split_at);
                buf.flags |= BufferFlags::CHANGED;
                new_line
            };
            editor.push_undo(vec![UndoAction::Split {
                line: orig_line,
                offset: split_at,
                new_line: current_line,
            }]);
            current_offset = 0;
        }
        let win = editor.cur_window_mut()?;
        win.dot_line = current_line;
        win.dot_offset = LineOffset(0);
        win.set_flag(WindowFlags::HARD);
        Ok(())
    }
}

pub struct NewlineAndIndent;

impl Command for NewlineAndIndent {
    fn execute(&self, editor: &mut Editor, _f: bool, n: usize) -> Result<()> {
        let (buffer_id, dot_line, dot_offset) = {
            let win = editor.cur_window()?;
            (win.buffer_id, win.dot_line, win.dot_offset)
        };
        let nicol = {
            let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
            let line = buf.line(dot_line).ok_or(Error::Abort)?;
            let text = &line.text;
            let mut col = 0;
            for &b in text.iter().take(dot_offset.0) {
                if b == b' ' {
                    col += 1;
                } else if b == b'\t' {
                    col = (col + 8) & !7;
                } else {
                    break;
                }
            }
            col
        };
        let mut current_line = dot_line;
        let mut dot_byte = 0;
        for _ in 0..n {
            let split_from = current_line;
            let split_at = dot_offset.0;
            let new_line = {
                let buf = editor.buffers.get_mut(buffer_id).ok_or(Error::Abort)?;
                let new_line = buf.split_line(split_from, split_at);
                buf.flags |= BufferFlags::CHANGED;
                new_line
            };
            let tabs = nicol / 8;
            let spaces = nicol % 8;
            let indent_data: Vec<u8> = std::iter::repeat_n(b'\t', tabs)
                .chain(std::iter::repeat_n(b' ', spaces))
                .collect();
            let indent_len = indent_data.len();
            let indent_off = {
                let buf = editor.buffers.get_mut(buffer_id).ok_or(Error::Abort)?;
                let line = buf.line_mut(new_line).ok_or(Error::Abort)?;
                let off = line.text.len();
                line.text.extend(&indent_data);
                off
            };
            let mut actions = vec![UndoAction::Split {
                line: split_from,
                offset: split_at,
                new_line,
            }];
            if !indent_data.is_empty() {
                actions.push(UndoAction::Insert {
                    line: new_line,
                    offset: indent_off,
                    data: indent_data,
                });
            }
            editor.push_undo(actions);
            current_line = new_line;
            dot_byte = indent_off + indent_len;
        }
        let win = editor.cur_window_mut()?;
        win.dot_line = current_line;
        win.dot_offset = LineOffset(dot_byte);
        win.set_flag(WindowFlags::HARD);
        Ok(())
    }
}

pub struct WrapWord;

impl Command for WrapWord {
    fn execute(&self, editor: &mut Editor, _f: bool, _n: usize) -> Result<()> {
        if BackwardChar.execute(editor, false, 1).is_err() {
            return Err(Error::Abort);
        }
        let mut cnt = 0usize;
        loop {
            let byte = {
                let win = editor.cur_window()?;
                let buf = editor.buffers.get(win.buffer_id).ok_or(Error::Abort)?;
                let line = buf.line(win.dot_line).ok_or(Error::Abort)?;
                line.text.get(win.dot_offset.0).copied()
            };
            if matches!(byte, Some(b' ' | b'\t')) {
                break;
            }
            cnt += 1;
            if BackwardChar.execute(editor, false, 1).is_err() {
                return Err(Error::Abort);
            }
            if editor.cur_window()?.dot_offset.0 == 0 {
                GotoEol.execute(editor, false, 0)?;
                return InsertNewline.execute(editor, false, 1);
            }
        }
        ForwardDelete.execute(editor, false, 1)?;
        InsertNewline.execute(editor, false, 1)?;
        for _ in 0..cnt {
            ForwardChar.execute(editor, false, 1)?;
        }
        editor.cur_window_mut()?.set_flag(WindowFlags::HARD);
        Ok(())
    }
}
