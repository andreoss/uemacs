use super::{
    BufferFlags, BufferId, Command, Editor, Error, LineId, LineOffset, Result, UndoAction,
    WindowFlags, collect_paragraph, is_blank_line,
};

pub struct FillPara;

pub struct SetFillColumn;

impl Command for SetFillColumn {
    fn execute(&self, editor: &mut Editor, _f: bool, n: usize) -> Result<()> {
        editor.fillcol = n;
        Ok(())
    }
}

impl Command for FillPara {
    fn execute(&self, editor: &mut Editor, _f: bool, _n: usize) -> Result<()> {
        let fillcol = editor.fillcol;
        if fillcol == 0 {
            return Ok(());
        }

        let buf_id = editor.cur_window()?.buffer_id;
        let dot_line = editor.cur_window()?.dot_line;

        let (head, start, end, words) = collect_paragraph(editor, buf_id, dot_line)?;

        if words.is_empty() {
            return Ok(());
        }

        let undo_actions = fill_paragraph_lines(editor, buf_id, head, start, end, &words, fillcol)?;

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
            win.dot_offset = LineOffset(last_len);
            win.set_flag(WindowFlags::MOVED);
        }

        Ok(())
    }
}

fn fill_paragraph_lines(
    editor: &mut Editor,
    buf_id: BufferId,
    head: LineId,
    start: LineId,
    end: LineId,
    words: &[Vec<u8>],
    fillcol: usize,
) -> Result<Vec<UndoAction>> {
    let mut undo_actions: Vec<UndoAction> = Vec::new();
    let buf = editor.buffers.get_mut(buf_id).ok_or(Error::Abort)?;

    if let Some(l) = buf.line_mut(start) {
        if !l.text.is_empty() {
            let old_text = std::mem::take(&mut l.text);
            undo_actions.push(UndoAction::Delete {
                line: start,
                offset: 0,
                data: old_text,
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

    let mut current = start;
    let mut line_len = 0usize;

    for (i, word) in words.iter().enumerate() {
        let space_needed = usize::from(i != 0);
        let new_len = line_len + space_needed + word.len();

        if new_len > fillcol && line_len > 0 {
            let split_from = current;
            let split_at = buf.line(split_from).map_or(0, |l| l.text.len());
            let new_id = buf.insert_after(current, crate::line::Line::new());
            undo_actions.push(UndoAction::Split {
                line: split_from,
                offset: split_at,
                new_line: new_id,
            });
            current = new_id;
            line_len = 0;
        }

        if line_len > 0 {
            let insert_off = buf.line(current).map_or(0, |l| l.text.len());
            if let Some(l) = buf.line_mut(current) {
                l.text.push(b' ');
            }
            undo_actions.push(UndoAction::Insert {
                line: current,
                offset: insert_off,
                data: vec![b' '],
            });
            line_len += 1;
        }

        let word_insert_off = buf.line(current).map_or(0, |l| l.text.len());
        if let Some(l) = buf.line_mut(current) {
            l.text.extend_from_slice(word);
        }
        undo_actions.push(UndoAction::Insert {
            line: current,
            offset: word_insert_off,
            data: word.clone(),
        });
        line_len += word.len();
    }

    buf.flags |= BufferFlags::CHANGED;
    Ok(undo_actions)
}
