use super::{
    Buffer, BufferFlags, Display, Editor, Error, LineId, Result, TerminalBackend, UndoAction,
    WindowFlags,
};

impl Editor {
    pub(super) fn insert_file<T: TerminalBackend>(
        &mut self,
        term: &mut T,
        display: &mut Display,
    ) -> Result<()> {
        let fname = self.minibuffer_readline(term, display, "Insert file: ")?;
        if fname.is_empty() {
            return Ok(());
        }
        let path = std::path::Path::new(&fname);
        if !path.exists() {
            display.write_echo(term, "(No such file)")?;
            return Ok(());
        }
        let Ok(content) = std::fs::read(&fname) else {
            return Err(Error::IoError);
        };
        if content.is_empty() {
            return Ok(());
        }
        let win = self.cur_window()?;
        let buf_id = win.buffer_id;
        let dot_line = win.dot_line;
        let dot_offset = win.dot_offset.0;
        let buf = self.buffers.get_mut(buf_id).ok_or(Error::Abort)?;
        buf.flags |= BufferFlags::CHANGED;
        let parts: Vec<&[u8]> = content.split(|&b| b == b'\n').collect();
        let has_trailing_newline = content.last() == Some(&b'\n');
        let limit = if has_trailing_newline {
            parts.len() - 1
        } else {
            parts.len()
        };
        let nlines = limit;
        let undo_actions = Self::insert_file_lines(buf, &parts, limit, dot_line, dot_offset)?;
        if !undo_actions.is_empty() {
            self.push_undo(undo_actions);
        }
        let win = self.cur_window_mut()?;
        win.set_flag(WindowFlags::HARD);
        let msg = if nlines == 1 {
            "(Inserted 1 line)".to_string()
        } else {
            format!("(Inserted {nlines} lines)")
        };
        display.write_echo(term, &msg)?;
        Ok(())
    }

    fn insert_file_lines(
        buf: &mut Buffer,
        parts: &[&[u8]],
        limit: usize,
        dot_line: LineId,
        dot_offset: usize,
    ) -> Result<Vec<UndoAction>> {
        let mut insert_after = dot_line;
        let mut undo_actions: Vec<UndoAction> = Vec::new();
        if limit > 0 {
            let line = buf.line_mut(insert_after).ok_or(Error::Abort)?;
            let pos = dot_offset.min(line.text.len());
            if limit == 1 {
                let remaining = line.text.split_off(pos);
                line.text.extend_from_slice(parts[0]);
                line.text.extend_from_slice(&remaining);
                if !parts[0].is_empty() {
                    undo_actions.push(UndoAction::Insert {
                        line: insert_after,
                        offset: pos,
                        data: parts[0].to_vec(),
                    });
                }
            } else {
                let tail = line.text.split_off(pos);
                let line_len_before_part0 = line.text.len();
                line.text.extend_from_slice(parts[0]);
                if !tail.is_empty() {
                    undo_actions.push(UndoAction::Delete {
                        line: insert_after,
                        offset: line_len_before_part0,
                        data: tail.clone(),
                    });
                }
                if !parts[0].is_empty() {
                    undo_actions.push(UndoAction::Insert {
                        line: insert_after,
                        offset: line_len_before_part0,
                        data: parts[0].to_vec(),
                    });
                }
                for &part in &parts[1..limit] {
                    let split_from = insert_after;
                    let split_at = buf.line(split_from).map_or(0, |l| l.text.len());
                    let new_id = buf.insert_after(insert_after, crate::line::Line::new());
                    undo_actions.push(UndoAction::Split {
                        line: split_from,
                        offset: split_at,
                        new_line: new_id,
                    });
                    let new_line = buf.line_mut(new_id).ok_or(Error::Abort)?;
                    new_line.text = part.to_vec();
                    if !part.is_empty() {
                        undo_actions.push(UndoAction::Insert {
                            line: new_id,
                            offset: 0,
                            data: part.to_vec(),
                        });
                    }
                    insert_after = new_id;
                }
                let last_line = buf.line_mut(insert_after).ok_or(Error::Abort)?;
                let last_off_before_tail = last_line.text.len();
                last_line.text.extend_from_slice(&tail);
                if !tail.is_empty() {
                    undo_actions.push(UndoAction::Insert {
                        line: insert_after,
                        offset: last_off_before_tail,
                        data: tail,
                    });
                }
            }
        }
        Ok(undo_actions)
    }
}
