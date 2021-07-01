use super::{
    BufferFlags, Editor, Error, LineId, Result, UndoAction, WindowFlags, region_bounds,
    utf8_char_width,
};

pub fn kill_n_chars(editor: &mut Editor, n: usize) -> Result<()> {
    let mut deleted = Vec::new();
    let mut undo_actions: Vec<UndoAction> = Vec::new();
    let mut remaining = n;
    while remaining > 0 {
        let (buffer_id, dot_line, dot_offset) = {
            let win = editor.cur_window()?;
            (win.buffer_id, win.dot_line, win.dot_offset.0)
        };
        let local = {
            let buf = editor.buffers.get_mut(buffer_id).ok_or(Error::Abort)?;
            let len = buf.line_len(dot_line).ok_or(Error::Abort)?;
            if dot_offset >= len {
                buf.next_line(dot_line).map_or_else(Vec::new, |next_id| {
                    let next_text = buf.line(next_id).unwrap().text.clone();
                    let after_next = buf.line(next_id).unwrap().next();
                    undo_actions.push(UndoAction::Merge {
                        line: dot_line,
                        offset: len,
                        next_line: next_id,
                        next_data: next_text.clone(),
                        after_next,
                    });
                    let collected = vec![b'\n'];
                    buf.line_mut(dot_line).unwrap().text.extend(next_text);
                    buf.remove(next_id);
                    collected
                })
            } else {
                let cw = utf8_char_width(buf.line(dot_line).unwrap().text[dot_offset]);
                let mut collected = Vec::with_capacity(cw);
                for j in 0..cw {
                    collected.push(buf.line(dot_line).unwrap().text[dot_offset + j]);
                }
                undo_actions.push(UndoAction::Delete {
                    line: dot_line,
                    offset: dot_offset,
                    data: collected.clone(),
                });
                buf.line_mut(dot_line).unwrap().delete_bytes(dot_offset, cw);
                collected
            }
        };
        if local.is_empty() {
            break;
        }
        deleted.extend(local);
        remaining -= 1;
    }
    if !undo_actions.is_empty() {
        editor.push_undo(undo_actions);
    }
    editor.kill_buffer.extend(deleted);
    {
        let buffer_id = editor.cur_window()?.buffer_id;
        let buf = editor.buffers.get_mut(buffer_id).ok_or(Error::Abort)?;
        buf.flags |= BufferFlags::CHANGED;
    }
    Ok(())
}

pub fn transform_region<F>(editor: &mut Editor, mut transform: F) -> Result<()>
where
    F: FnMut(u8) -> u8,
{
    let (start_line, start_off, end_line, end_off) = region_bounds(editor)?;
    let multi_line = start_line != end_line;
    let buffer_id = editor.cur_window()?.buffer_id;
    let mut undo_actions: Vec<UndoAction> = Vec::new();

    let apply_span = |buf: &mut crate::buffer::Buffer,
                      line: LineId,
                      span: std::ops::Range<usize>,
                      undo: &mut Vec<UndoAction>,
                      transform: &mut F| {
        if let Some(l) = buf.line_mut(line) {
            let original = l.text[span.clone()].to_vec();
            for c in &mut l.text[span.clone()] {
                *c = transform(*c);
            }
            let new = l.text[span.clone()].to_vec();
            if new != original {
                undo.push(UndoAction::Delete {
                    line,
                    offset: span.start,
                    data: original,
                });
                undo.push(UndoAction::Insert {
                    line,
                    offset: span.start,
                    data: new,
                });
            }
        }
    };

    if start_line == end_line {
        let buf = editor.buffers.get_mut(buffer_id).ok_or(Error::Abort)?;
        apply_span(
            buf,
            start_line,
            start_off..end_off,
            &mut undo_actions,
            &mut transform,
        );
    } else {
        {
            let buf = editor.buffers.get_mut(buffer_id).ok_or(Error::Abort)?;
            let len = buf.line(start_line).map_or(0, |l| l.text.len());
            apply_span(
                buf,
                start_line,
                start_off..len,
                &mut undo_actions,
                &mut transform,
            );
        }
        let mut current = start_line;
        loop {
            let next = editor
                .buffers
                .get(buffer_id)
                .ok_or(Error::Abort)?
                .next_line(current);
            match next {
                Some(next_line) => {
                    let buf = editor.buffers.get_mut(buffer_id).ok_or(Error::Abort)?;
                    if next_line == end_line {
                        apply_span(
                            buf,
                            next_line,
                            0..end_off,
                            &mut undo_actions,
                            &mut transform,
                        );
                        break;
                    }
                    let len = buf.line(next_line).map_or(0, |l| l.text.len());
                    apply_span(buf, next_line, 0..len, &mut undo_actions, &mut transform);
                    current = next_line;
                }
                None => return Err(Error::Abort),
            }
        }
    }

    if !undo_actions.is_empty() {
        editor.push_undo(undo_actions);
        editor.buffers.get_mut(buffer_id).ok_or(Error::Abort)?.flags |= BufferFlags::CHANGED;
    }
    let win = editor.cur_window_mut()?;
    win.clear_mark();
    win.set_flag(if multi_line {
        WindowFlags::HARD
    } else {
        WindowFlags::EDITED
    });
    Ok(())
}
