use super::{
    BufferFlags, BufferId, Editor, Error, LineId, LineOffset, Result, UndoAction, WindowFlags,
};

impl Editor {
    pub(super) fn query_replace_apply(
        &mut self,
        buf_id: BufferId,
        start_line: LineId,
        start_off: usize,
        end_line: LineId,
        end_off: usize,
        replacement: &[u8],
    ) -> Result<()> {
        let buf_id2 = buf_id;
        let repl = replacement.to_vec();
        let mut undo_actions: Vec<UndoAction> = Vec::new();

        let end_append_off = if start_line == end_line {
            None
        } else {
            self.query_replace_merge_lines(buf_id2, start_line, end_line, &mut undo_actions)?
        };

        let del_len = end_append_off.map_or_else(
            || end_off.saturating_sub(start_off),
            |anchor| (anchor + end_off).saturating_sub(start_off),
        );
        let deleted = {
            let buf = self.buffers.get(buf_id2).ok_or(Error::Abort)?;
            let line = buf.line(start_line).ok_or(Error::Abort)?;
            let start = start_off.min(line.text.len());
            let end = start.saturating_add(del_len).min(line.text.len());
            line.text[start..end].to_vec()
        };
        {
            let buf = self.buffers.get_mut(buf_id2).ok_or(Error::Abort)?;
            buf.line_mut(start_line)
                .ok_or(Error::Abort)?
                .delete_bytes(start_off, del_len);
        }
        if !deleted.is_empty() {
            undo_actions.push(UndoAction::Delete {
                line: start_line,
                offset: start_off,
                data: deleted,
            });
        }

        {
            let buf = self.buffers.get_mut(buf_id2).ok_or(Error::Abort)?;
            buf.line_mut(start_line)
                .ok_or(Error::Abort)?
                .insert_bytes(start_off, &repl)?;
        }
        if !repl.is_empty() {
            undo_actions.push(UndoAction::Insert {
                line: start_line,
                offset: start_off,
                data: repl.clone(),
            });
        }

        if !undo_actions.is_empty() {
            self.push_undo(undo_actions);
        }

        let new_off = start_off + repl.len();
        let win = self.cur_window_mut()?;
        win.dot_line = start_line;
        win.dot_offset = LineOffset(new_off);
        win.set_flag(WindowFlags::EDITED);

        if let Some(buf) = self.buffers.get_mut(buf_id2) {
            buf.flags |= BufferFlags::CHANGED;
        }
        Ok(())
    }

    fn query_replace_merge_lines(
        &mut self,
        buf_id2: BufferId,
        start_line: LineId,
        end_line: LineId,
        undo_actions: &mut Vec<UndoAction>,
    ) -> Result<Option<usize>> {
        loop {
            let nxt = {
                let buf = self.buffers.get(buf_id2).ok_or(Error::Abort)?;
                buf.next_line(start_line)
            };
            match nxt {
                Some(nl) => {
                    if nl == end_line {
                        break;
                    }
                    let (next_text, after_next, append_off) = {
                        let buf = self.buffers.get(buf_id2).ok_or(Error::Abort)?;
                        let next_text = buf.line(nl).ok_or(Error::Abort)?.text.clone();
                        let after_next = buf.line(nl).ok_or(Error::Abort)?.next();
                        let append_off = buf.line(start_line).ok_or(Error::Abort)?.text.len();
                        (next_text, after_next, append_off)
                    };
                    undo_actions.push(UndoAction::Merge {
                        line: start_line,
                        offset: append_off,
                        next_line: nl,
                        next_data: next_text.clone(),
                        after_next,
                    });
                    let buf = self.buffers.get_mut(buf_id2).ok_or(Error::Abort)?;
                    buf.line_mut(start_line)
                        .ok_or(Error::Abort)?
                        .text
                        .extend(next_text);
                    buf.remove(nl);
                }
                None => break,
            }
        }
        let (end_text, end_after_next, end_off_anchor) = {
            let buf = self.buffers.get(buf_id2).ok_or(Error::Abort)?;
            let end_text = buf.line(end_line).ok_or(Error::Abort)?.text.clone();
            let end_after_next = buf.line(end_line).ok_or(Error::Abort)?.next();
            let anchor = buf.line(start_line).ok_or(Error::Abort)?.text.len();
            (end_text, end_after_next, anchor)
        };
        undo_actions.push(UndoAction::Merge {
            line: start_line,
            offset: end_off_anchor,
            next_line: end_line,
            next_data: end_text.clone(),
            after_next: end_after_next,
        });
        let buf = self.buffers.get_mut(buf_id2).ok_or(Error::Abort)?;
        buf.line_mut(start_line)
            .ok_or(Error::Abort)?
            .text
            .extend(end_text);
        buf.remove(end_line);
        Ok(Some(end_off_anchor))
    }
}
