use super::{
    BufferFlags, Command, Editor, Error, ForwardDelete, LineOffset, Mode, Result, UndoAction,
    WindowFlags, WrapWord, cmode_reindent, cursor_column,
};

pub struct InsertChar(pub char);

impl Command for InsertChar {
    fn execute(&self, editor: &mut Editor, _f: bool, n: usize) -> Result<()> {
        let ch = self.0;

        if ch == ' ' && n.max(1) == 1 {
            let do_wrap = {
                let Some(win) = editor.current_window() else {
                    return Err(Error::Abort);
                };
                let Some(buf) = editor.buffers.get(win.buffer_id) else {
                    return Err(Error::Abort);
                };
                buf.mode.intersects(Mode::WRAP)
                    && editor.fillcol > 0
                    && !buf.mode.intersects(Mode::VIEW)
                    && cursor_column(editor).is_some_and(|c| c > editor.fillcol)
            };
            if do_wrap {
                WrapWord.execute(editor, false, 1)?;
            }
        }

        for _ in 0..n.max(1) {
            let (buf_id, line, offset) = {
                let win = editor.cur_window()?;
                (win.buffer_id, win.dot_line, win.dot_offset.0)
            };

            {
                let buf = editor.buffers.get(buf_id).ok_or(Error::Abort)?;
                let line_len = buf.line_len(line).ok_or(Error::Abort)?;
                if buf.mode.intersects(Mode::OVERWRITE) && offset < line_len {
                    let is_tab = buf
                        .line(line)
                        .is_some_and(|l| l.text.get(offset) == Some(&b'\t'));
                    if !is_tab || (offset % 8 == 7) {
                        let _ = buf;
                        ForwardDelete.execute(editor, false, 1)?;
                    }
                }
            }

            cmode_reindent(editor, buf_id, line, offset, ch)?;

            let mut bytes = [0u8; 4];
            let s = ch.encode_utf8(&mut bytes);
            let insert_off = {
                let win = editor.cur_window()?;
                win.dot_offset.0
            };
            let data = s.as_bytes().to_vec();
            editor.push_undo(vec![UndoAction::Insert {
                line,
                offset: insert_off,
                data,
            }]);
            let buf = editor.buffers.get_mut(buf_id).ok_or(Error::Abort)?;
            let line_mut = buf.line_mut(line).ok_or(Error::Abort)?;
            line_mut.insert_bytes(insert_off, s.as_bytes())?;
            buf.flags |= BufferFlags::CHANGED;
            let win = editor.cur_window_mut()?;
            win.dot_offset = LineOffset(insert_off + s.len());
            win.set_flag(WindowFlags::EDITED);
        }

        let buf_id = {
            let win = editor.cur_window()?;
            win.buffer_id
        };
        let buf_id2 = buf_id;
        let fname = {
            let buf = editor.buffers.get(buf_id2).ok_or(Error::Abort)?;
            if !buf.mode.intersects(crate::core::Mode::AUTO_SAVE) || buf.filename.is_empty() {
                return Ok(());
            }
            editor.gacount = editor.gacount.saturating_sub(1);
            if editor.gacount != 0 {
                return Ok(());
            }
            buf.filename.clone()
        };
        let buf = editor.buffers.get_mut(buf_id2).ok_or(Error::Abort)?;
        let _ = crate::file::write_from_buffer(buf, &fname);
        editor.gacount = editor.gasave;

        Ok(())
    }
}
