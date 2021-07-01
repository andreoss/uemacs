use super::{
    BufferFlags, BufferId, Command, Editor, Error, LineId, LineOffset, Mode, Result, UndoAction,
    WindowFlags, collect_justify_para, is_blank_line,
};

pub struct JustifyPara;

impl Command for JustifyPara {
    fn execute(&self, editor: &mut Editor, _f: bool, _n: usize) -> Result<()> {
        let fillcol = editor.fillcol;
        if fillcol == 0 {
            return Ok(());
        }

        let (buf_id, leftmarg) = {
            let win = editor.cur_window()?;
            let buf = editor.buffers.get(win.buffer_id).ok_or(Error::Abort)?;
            if buf.mode.intersects(Mode::VIEW) {
                return Err(Error::Abort);
            }
            (win.buffer_id, win.dot_offset.0)
        };

        if leftmarg + 10 > fillcol {
            return Ok(());
        }

        let (head, start, end, words) = collect_justify_para(editor, buf_id, leftmarg)?;

        if words.is_empty() {
            return Ok(());
        }

        let mut undo_actions = justify_merge_para(editor, buf_id, head, start, end, leftmarg)?;
        justify_reflow_words(
            editor,
            buf_id,
            start,
            &words,
            fillcol,
            leftmarg,
            &mut undo_actions,
        )?;

        if !undo_actions.is_empty() {
            editor.push_undo(undo_actions);
        }

        {
            let buf = editor.buffers.get(buf_id).ok_or(Error::Abort)?;
            let mut last = start;
            loop {
                let next = buf.next_line(last).unwrap_or(head);
                if next == end || next == head || is_blank_line(buf, next) {
                    break;
                }
                last = next;
            }
            let last_len = buf.line_len(last).unwrap_or(0);
            let win = editor.cur_window_mut()?;
            win.dot_line = last;
            win.dot_offset = LineOffset(last_len.min(leftmarg));
            win.set_flag(WindowFlags::MOVED);
        }

        Ok(())
    }
}

fn justify_merge_para(
    editor: &mut Editor,
    buf_id: BufferId,
    head: LineId,
    start: LineId,
    end: LineId,
    leftmarg: usize,
) -> Result<Vec<UndoAction>> {
    let mut undo_actions: Vec<UndoAction> = Vec::new();
    let buf = editor.buffers.get_mut(buf_id).ok_or(Error::Abort)?;

    if let Some(l) = buf.line_mut(start) {
        if l.text.len() > leftmarg {
            let trimmed = l.text.split_off(leftmarg);
            undo_actions.push(UndoAction::Delete {
                line: start,
                offset: leftmarg,
                data: trimmed,
            });
        }
    }

    let line = start;
    loop {
        let next = buf.next_line(line).unwrap_or(head);
        if next == end || next == head {
            break;
        }
        let (next_data, after_next, anchor_off) = {
            let next_text = buf.line(next).ok_or(Error::Abort)?.text.clone();
            let after_next = buf.line(next).ok_or(Error::Abort)?.next();
            let anchor_off = buf.line(line).ok_or(Error::Abort)?.text.len();
            (next_text, after_next, anchor_off)
        };
        undo_actions.push(UndoAction::Merge {
            line,
            offset: anchor_off,
            next_line: next,
            next_data,
            after_next,
        });
        buf.remove(next);
    }

    Ok(undo_actions)
}

fn justify_reflow_words(
    editor: &mut Editor,
    buf_id: BufferId,
    start: LineId,
    words: &[Vec<u8>],
    fillcol: usize,
    leftmarg: usize,
    undo_actions: &mut Vec<UndoAction>,
) -> Result<()> {
    let buf = editor.buffers.get_mut(buf_id).ok_or(Error::Abort)?;
    let mut current = start;
    let mut clength = leftmarg;
    let mut first = true;

    for word in words {
        let space_needed = usize::from(!first);
        let newlength = clength + space_needed + word.len();

        if newlength > fillcol && clength > leftmarg {
            let split_from = current;
            let split_at = buf.line(split_from).map_or(0, |l| l.text.len());
            let new_id = buf.insert_after(current, crate::line::Line::new());
            undo_actions.push(UndoAction::Split {
                line: split_from,
                offset: split_at,
                new_line: new_id,
            });
            current = new_id;
            let pad: Vec<u8> = std::iter::repeat_n(b' ', leftmarg).collect();
            if !pad.is_empty() {
                if let Some(l) = buf.line_mut(current) {
                    l.text.extend(&pad);
                }
                undo_actions.push(UndoAction::Insert {
                    line: current,
                    offset: 0,
                    data: pad,
                });
            }
            clength = leftmarg;
            first = true;
        }

        if !first {
            let off = buf.line(current).map_or(0, |l| l.text.len());
            if let Some(l) = buf.line_mut(current) {
                l.text.push(b' ');
            }
            undo_actions.push(UndoAction::Insert {
                line: current,
                offset: off,
                data: vec![b' '],
            });
            clength += 1;
        }
        first = false;

        let off = buf.line(current).map_or(0, |l| l.text.len());
        if let Some(l) = buf.line_mut(current) {
            l.text.extend_from_slice(word);
        }
        undo_actions.push(UndoAction::Insert {
            line: current,
            offset: off,
            data: word.clone(),
        });
        clength += word.len();
        if word.last() == Some(&b'.') {
            let off = buf.line(current).map_or(0, |l| l.text.len());
            if let Some(l) = buf.line_mut(current) {
                l.text.push(b' ');
            }
            undo_actions.push(UndoAction::Insert {
                line: current,
                offset: off,
                data: vec![b' '],
            });
            clength += 1;
        }
    }

    buf.flags |= BufferFlags::CHANGED;
    Ok(())
}
