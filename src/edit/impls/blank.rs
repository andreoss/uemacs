use super::{BufferFlags, Command, Editor, Error, LineOffset, Result, UndoAction, WindowFlags};

pub struct DeleteBlankLines;

impl Command for DeleteBlankLines {
    fn execute(&self, editor: &mut Editor, _f: bool, _n: usize) -> Result<()> {
        let buf_id = editor.cur_window()?.buffer_id;
        let dot_line = editor.cur_window()?.dot_line;

        let (to_remove, lp1) = {
            let buf = editor.buffers.get(buf_id).ok_or(Error::Abort)?;
            let head = buf.head;

            let mut lp1 = dot_line;
            loop {
                let is_blank = buf.line(lp1).is_some_and(|l| {
                    l.text.is_empty() || l.text.iter().all(|&b| b == b' ' || b == b'\t')
                });
                if !is_blank {
                    break;
                }
                match buf.prev_line(lp1) {
                    Some(p) if p != head => lp1 = p,
                    _ => break,
                }
            }

            let mut to_remove = Vec::new();
            let mut lp2 = lp1;
            loop {
                match buf.next_line(lp2) {
                    Some(n) if n != head => {
                        let is_blank = buf.line(n).is_some_and(|l| {
                            l.text.is_empty() || l.text.iter().all(|&b| b == b' ' || b == b'\t')
                        });
                        if is_blank {
                            lp2 = n;
                            to_remove.push(n);
                        } else {
                            break;
                        }
                    }
                    _ => break,
                }
            }
            (to_remove, lp1)
        };

        if to_remove.is_empty() {
            return Ok(());
        }

        let mut undo_actions: Vec<UndoAction> = Vec::new();
        {
            let buf = editor.buffers.get_mut(buf_id).ok_or(Error::Abort)?;
            let anchor_off = buf.line(lp1).ok_or(Error::Abort)?.text.len();
            for &line_id in &to_remove {
                let next_data = buf.line(line_id).ok_or(Error::Abort)?.text.clone();
                let after_next = buf.line(line_id).ok_or(Error::Abort)?.next();
                undo_actions.push(UndoAction::Merge {
                    line: lp1,
                    offset: anchor_off,
                    next_line: line_id,
                    next_data,
                    after_next,
                });
                buf.remove(line_id);
            }
            buf.flags |= BufferFlags::CHANGED;
        }
        if !undo_actions.is_empty() {
            editor.push_undo(undo_actions);
        }

        let head;
        let (next, end_len) = {
            let buf = editor.buffers.get(buf_id).ok_or(Error::Abort)?;
            head = buf.head;
            let n = buf.next_line(lp1).unwrap_or(head);
            let len = if n == head {
                buf.line_len(lp1).unwrap_or(0)
            } else {
                0
            };
            (n, len)
        };
        {
            let win = editor.cur_window_mut()?;
            if next == head {
                win.dot_line = lp1;
                win.dot_offset = LineOffset(end_len);
            } else {
                win.dot_line = next;
                win.dot_offset = LineOffset(0);
            }
            win.set_flag(WindowFlags::HARD);
        }

        Ok(())
    }
}
