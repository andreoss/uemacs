use super::{
    BufferFlags, Command, Editor, Error, LineOffset, Result, UndoAction, WindowFlags,
    advance_dot_one_line, cursor_column,
};

pub struct InsertTab;

impl Command for InsertTab {
    fn execute(&self, editor: &mut Editor, _f: bool, n: usize) -> Result<()> {
        if n == 0 || n > 1 {
            editor.tabsize = n;
            return Ok(());
        }
        let bytes: Vec<u8> = if editor.tabsize == 0 {
            vec![b'\t']
        } else {
            let col = cursor_column(editor).unwrap_or(0);
            vec![b' '; editor.tabsize - (col % editor.tabsize)]
        };
        let (buf_id, line, offset) = {
            let win = editor.cur_window()?;
            (win.buffer_id, win.dot_line, win.dot_offset.0)
        };
        editor.push_undo(vec![UndoAction::Insert {
            line,
            offset,
            data: bytes.clone(),
        }]);
        let count = bytes.len();
        let buf = editor.buffers.get_mut(buf_id).ok_or(Error::Abort)?;
        let text = &mut buf.line_mut(line).ok_or(Error::Abort)?.text;
        for (i, &b) in bytes.iter().enumerate() {
            text.insert(offset + i, b);
        }
        buf.flags |= BufferFlags::CHANGED;
        let win = editor.cur_window_mut()?;
        win.dot_offset = LineOffset(offset + count);
        win.set_flag(WindowFlags::EDITED);
        Ok(())
    }
}

pub struct TransposeChars;

impl Command for TransposeChars {
    fn execute(&self, editor: &mut Editor, _f: bool, _n: usize) -> Result<()> {
        let (buf_id, line, offset) = {
            let win = editor.cur_window()?;
            (win.buffer_id, win.dot_line, win.dot_offset.0)
        };

        let (c1, c2, pos1, pos2) = {
            let buf = editor.buffers.get(buf_id).ok_or(Error::Abort)?;
            let len = buf.line_len(line).ok_or(Error::Abort)?;
            if len < 2 {
                return Err(Error::Abort);
            }
            let mut doto = offset;
            if doto == len {
                doto = doto.checked_sub(1).ok_or(Error::Abort)?;
            }
            let p1 = doto.checked_sub(1).ok_or(Error::Abort)?;
            let p2 = doto;
            let ch1 = buf.line(line).ok_or(Error::Abort)?.text[p1];
            let ch2 = buf.line(line).ok_or(Error::Abort)?.text[p2];
            (ch1, ch2, p1, p2)
        };

        editor.push_undo(vec![
            UndoAction::Delete {
                line,
                offset: pos1,
                data: vec![c1, c2],
            },
            UndoAction::Insert {
                line,
                offset: pos1,
                data: vec![c2, c1],
            },
        ]);
        {
            let buf = editor.buffers.get_mut(buf_id).ok_or(Error::Abort)?;
            buf.line_mut(line).ok_or(Error::Abort)?.put_byte(pos1, c2);
            buf.line_mut(line).ok_or(Error::Abort)?.put_byte(pos2, c1);
            buf.flags |= BufferFlags::CHANGED;
        }

        editor.cur_window_mut()?.set_flag(WindowFlags::EDITED);

        Ok(())
    }
}

pub struct TrimLine;

impl Command for TrimLine {
    fn execute(&self, editor: &mut Editor, _f: bool, n: usize) -> Result<()> {
        let count = n.max(1);
        let mut undo_actions: Vec<UndoAction> = Vec::new();
        for i in 0..count {
            let (buf_id, line) = {
                let win = editor.cur_window()?;
                (win.buffer_id, win.dot_line)
            };
            let buf = editor.buffers.get_mut(buf_id).ok_or(Error::Abort)?;
            let line_mut = buf.line_mut(line).ok_or(Error::Abort)?;
            let orig_len = line_mut.text.len();
            let trim_pos = line_mut
                .text
                .iter()
                .rposition(|&b| b != b' ' && b != b'\t')
                .map_or(0, |p| p + 1);
            if trim_pos < orig_len {
                let trimmed = line_mut.text[trim_pos..orig_len].to_vec();
                line_mut.text.truncate(trim_pos);
                buf.flags |= BufferFlags::CHANGED;
                undo_actions.push(UndoAction::Delete {
                    line,
                    offset: trim_pos,
                    data: trimmed,
                });
            }
            if i + 1 < count && !advance_dot_one_line(editor) {
                break;
            }
        }
        if !undo_actions.is_empty() {
            editor.push_undo(undo_actions);
        }
        editor
            .current_window_mut()
            .ok_or(Error::Abort)?
            .set_flag(WindowFlags::EDITED);
        Ok(())
    }
}
