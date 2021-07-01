use super::{
    BufferFlags, Command, Editor, Error, InsertNewline, LineOffset, Result, UndoAction, WindowFlags,
};

pub struct Yank;

impl Command for Yank {
    fn execute(&self, editor: &mut Editor, _f: bool, n: usize) -> Result<()> {
        if editor.kill_buffer.is_empty() {
            return Ok(());
        }
        let kb = editor.kill_buffer.clone();
        for _ in 0..n.max(1) {
            let mut pending: Vec<u8> = Vec::new();
            let flush = |editor: &mut Editor, pending: &mut Vec<u8>| -> Result<()> {
                if pending.is_empty() {
                    return Ok(());
                }
                let (buf_id, line, offset) = {
                    let win = editor.cur_window()?;
                    (win.buffer_id, win.dot_line, win.dot_offset.0)
                };
                editor.push_undo(vec![UndoAction::Insert {
                    line,
                    offset,
                    data: pending.clone(),
                }]);
                let buf = editor.buffers.get_mut(buf_id).ok_or(Error::Abort)?;
                buf.line_mut(line)
                    .ok_or(Error::Abort)?
                    .text
                    .splice(offset..offset, pending.iter().copied());
                let added = pending.len();
                pending.clear();
                let win = editor.cur_window_mut()?;
                win.dot_offset = LineOffset(offset + added);
                win.set_flag(WindowFlags::EDITED);
                Ok(())
            };
            for &b in &kb {
                if b == b'\n' {
                    flush(editor, &mut pending)?;
                    InsertNewline.execute(editor, false, 1)?;
                } else {
                    pending.push(b);
                }
            }
            flush(editor, &mut pending)?;
        }
        let buffer_id = editor.cur_window()?.buffer_id;
        editor.buffers.get_mut(buffer_id).ok_or(Error::Abort)?.flags |= BufferFlags::CHANGED;
        Ok(())
    }
}
