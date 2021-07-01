use super::{
    BufferFlags, Command, Editor, Error, LineOffset, Result, UndoAction, WindowFlags, is_blank_line,
};

pub struct KillParagraph;

impl Command for KillParagraph {
    fn execute(&self, editor: &mut Editor, f: bool, n: usize) -> Result<()> {
        let buf_id = editor.cur_window()?.buffer_id;
        let count = if f { n } else { 1 };
        let mut total_killed = Vec::new();
        let mut final_end = None;
        let mut undo_actions: Vec<UndoAction> = Vec::new();
        for _ in 0..count {
            let dot_line = editor.cur_window()?.dot_line;
            let (para_start, end) = {
                let buf = editor.buffers.get(buf_id).ok_or(Error::Abort)?;
                let head = buf.head;
                let mut start = dot_line;
                loop {
                    let prev = buf.prev_line(start);
                    match prev {
                        Some(p) if p != head && !is_blank_line(buf, p) => start = p,
                        _ => break,
                    }
                }
                let mut end_line = start;
                let mut any_blank = false;
                loop {
                    let next = buf.next_line(end_line);
                    match next {
                        Some(n) if n != head && !is_blank_line(buf, n) => {
                            end_line = n;
                            any_blank = false;
                        }
                        Some(n) if n != head && is_blank_line(buf, n) && !any_blank => {
                            any_blank = true;
                        }
                        _ => break,
                    }
                    if any_blank {
                        break;
                    }
                }
                let after = buf.next_line(end_line).unwrap_or(head);
                (start, after)
            };
            if para_start == end {
                break;
            }
            let anchor = {
                let buf = editor.buffers.get(buf_id).ok_or(Error::Abort)?;
                buf.prev_line(para_start).unwrap_or(buf.head)
            };
            {
                let buf = editor.buffers.get_mut(buf_id).ok_or(Error::Abort)?;
                let anchor_off = buf.line(anchor).map_or(0, |l| l.text.len());
                let mut cur = para_start;
                loop {
                    if cur == end {
                        break;
                    }
                    let (text_clone, next) = {
                        let l = buf.line(cur).ok_or(Error::Abort)?;
                        (l.text.clone(), l.next())
                    };
                    let after_next = if next == buf.head { buf.head } else { next };
                    total_killed.extend_from_slice(&text_clone);
                    total_killed.push(b'\n');
                    undo_actions.push(UndoAction::Merge {
                        line: anchor,
                        offset: anchor_off,
                        next_line: cur,
                        next_data: text_clone,
                        after_next,
                    });
                    buf.remove(cur);
                    cur = next;
                }
                buf.flags |= BufferFlags::CHANGED;
            }
            final_end = Some(end);
            {
                let win = editor.cur_window_mut()?;
                win.dot_line = end;
                win.dot_offset = LineOffset(0);
                win.set_flag(WindowFlags::HARD);
            }
        }

        if total_killed.is_empty() {
            return Ok(());
        }
        if !undo_actions.is_empty() {
            editor.push_undo(undo_actions);
        }
        editor.kill_buffer = total_killed;

        if let Some(end) = final_end {
            let win = editor.cur_window_mut()?;
            win.dot_line = end;
            win.dot_offset = LineOffset(0);
            win.set_flag(WindowFlags::MOVED);
        }

        Ok(())
    }
}

pub struct Undo;

impl Command for Undo {
    fn execute(&self, editor: &mut Editor, _f: bool, _n: usize) -> Result<()> {
        let Some(entry) = editor.undo_stack.pop() else {
            return Ok(());
        };
        let Some(buf) = editor.buffers.get_mut(entry.buffer_id) else {
            return Err(Error::Abort);
        };
        for action in entry.actions.into_iter().rev() {
            match action {
                UndoAction::Insert { line, offset, data } => {
                    buf.line_mut(line)
                        .ok_or(Error::Abort)?
                        .delete_bytes(offset, data.len());
                }
                UndoAction::Delete { line, offset, data } => {
                    buf.line_mut(line)
                        .ok_or(Error::Abort)?
                        .insert_bytes(offset, &data)?;
                }
                UndoAction::Split {
                    line,
                    offset,
                    new_line,
                } => {
                    let new_text = buf.line(new_line).ok_or(Error::Abort)?.text.clone();
                    buf.line_mut(line)
                        .ok_or(Error::Abort)?
                        .text
                        .splice(offset..offset, new_text.iter().copied());
                    buf.remove(new_line);
                }
                UndoAction::Merge {
                    line,
                    offset,
                    next_line,
                    next_data,
                    after_next,
                } => {
                    buf.line_mut(line).ok_or(Error::Abort)?.text.drain(offset..);
                    {
                        let next = buf.line_mut(next_line).ok_or(Error::Abort)?;
                        next.text = next_data;
                        next.set_prev(line);
                        next.set_next(after_next);
                    }
                    buf.line_mut(line).ok_or(Error::Abort)?.set_next(next_line);
                    buf.line_mut(after_next)
                        .ok_or(Error::Abort)?
                        .set_prev(next_line);
                }
            }
        }
        buf.flags |= BufferFlags::CHANGED;
        let win = editor.cur_window_mut()?;
        win.dot_line = entry.dot_line;
        win.dot_offset = entry.dot_offset;
        win.set_flag(WindowFlags::HARD);
        Ok(())
    }
}
