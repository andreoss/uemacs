use super::{
    BufferFlags, CmdFlags, Command, Editor, Error, LineOffset, Result, UndoAction, WindowFlags,
    utf8_char_width,
};

pub struct ForwardDelete;

impl Command for ForwardDelete {
    fn execute(&self, editor: &mut Editor, f: bool, mut n: usize) -> Result<()> {
        if f {
            editor.this_flag |= CmdFlags::KILL;
        }
        let mut crossed_line = false;
        while n > 0 {
            let (buffer_id, dot_line, dot_offset) = {
                let win = editor.cur_window()?;
                (win.buffer_id, win.dot_line, win.dot_offset.0)
            };
            let len = {
                let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
                buf.line_len(dot_line).ok_or(Error::Abort)?
            };
            if dot_offset >= len {
                crossed_line = true;
                let (next_id, next_text, after_next, orig_len) = {
                    let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
                    let next_id = buf.next_line(dot_line).ok_or(Error::Abort)?;
                    let next_text = buf.line(next_id).unwrap().text.clone();
                    let after_next = buf.line(next_id).unwrap().next();
                    let orig_len = buf.line_len(dot_line).ok_or(Error::Abort)?;
                    (next_id, next_text, after_next, orig_len)
                };
                editor.push_undo(vec![UndoAction::Merge {
                    line: dot_line,
                    offset: orig_len,
                    next_line: next_id,
                    next_data: next_text.clone(),
                    after_next,
                }]);
                let buf = editor.buffers.get_mut(buffer_id).ok_or(Error::Abort)?;
                buf.line_mut(dot_line).unwrap().text.extend(next_text);
                buf.remove(next_id);
            } else {
                let (cw, data) = {
                    let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
                    let line = buf.line(dot_line).ok_or(Error::Abort)?;
                    let cw = utf8_char_width(line.text[dot_offset]);
                    let mut data = Vec::with_capacity(cw);
                    for j in 0..cw {
                        data.push(line.text[dot_offset + j]);
                    }
                    (cw, data)
                };
                editor.push_undo(vec![UndoAction::Delete {
                    line: dot_line,
                    offset: dot_offset,
                    data: data.clone(),
                }]);
                let buf = editor.buffers.get_mut(buffer_id).ok_or(Error::Abort)?;
                let line = buf.line_mut(dot_line).ok_or(Error::Abort)?;
                line.delete_bytes(dot_offset, cw);
            }
            n -= 1;
        }
        {
            let buffer_id = editor.cur_window()?.buffer_id;
            let buf = editor.buffers.get_mut(buffer_id).ok_or(Error::Abort)?;
            buf.flags |= BufferFlags::CHANGED;
        }
        let flag = if crossed_line {
            WindowFlags::HARD
        } else {
            WindowFlags::EDITED
        };
        editor
            .current_window_mut()
            .ok_or(Error::Abort)?
            .set_flag(flag);
        Ok(())
    }
}

pub struct BackwardDelete;

impl Command for BackwardDelete {
    fn execute(&self, editor: &mut Editor, f: bool, n: usize) -> Result<()> {
        if f {
            editor.this_flag |= CmdFlags::KILL;
        }
        {
            let mut remaining = n;
            while remaining > 0 {
                let (buffer_id, dot_line, dot_offset) = {
                    let win = editor.cur_window()?;
                    (win.buffer_id, win.dot_line, win.dot_offset.0)
                };
                if dot_offset == 0 {
                    let prev_id = {
                        let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
                        match buf.prev_line(dot_line) {
                            Some(prev) => prev,
                            None => return Ok(()),
                        }
                    };
                    let prev_len = {
                        let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
                        buf.line_len(prev_id).ok_or(Error::Abort)?
                    };
                    let win = editor.cur_window_mut()?;
                    win.dot_line = prev_id;
                    win.dot_offset = LineOffset(prev_len);
                } else {
                    let line_text = {
                        let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
                        buf.line(dot_line).unwrap().text.clone()
                    };
                    let mut new_offset = dot_offset - 1;
                    while new_offset > 0 {
                        let byte = line_text[new_offset];
                        if !(0x80..0xC0).contains(&byte) {
                            break;
                        }
                        new_offset -= 1;
                    }
                    let win = editor.cur_window_mut()?;
                    win.dot_offset = LineOffset(new_offset);
                }
                remaining -= 1;
            }
        }
        ForwardDelete.execute(editor, f, n)?;
        Ok(())
    }
}
