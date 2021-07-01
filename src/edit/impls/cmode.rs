use super::{
    BufferFlags, BufferId, Command, Editor, Error, ForwardDelete, LineId, LineOffset, Mode, Result,
    UndoAction,
};

pub fn cmode_reindent(
    editor: &mut Editor,
    buf_id: BufferId,
    line: LineId,
    offset: usize,
    ch: char,
) -> Result<()> {
    let cmode_special = {
        let buf = editor.buffers.get(buf_id).ok_or(Error::Abort)?;
        buf.mode.intersects(Mode::C_MODE) && (ch == '}' || ch == '#')
    };
    if cmode_special {
        if ch == '}' {
            cmode_align_close_brace(editor, buf_id, line, offset)?;
        } else if ch == '#' {
            cmode_rewind_hash(editor, buf_id, line, offset)?;
        }
    }
    Ok(())
}

fn cmode_align_close_brace(
    editor: &mut Editor,
    buf_id: BufferId,
    line: LineId,
    offset: usize,
) -> Result<()> {
    let should_align = {
        let buf = editor.buffers.get(buf_id).ok_or(Error::Abort)?;
        let text = &buf.line(line).ok_or(Error::Abort)?.text;
        if offset == 0 {
            true
        } else {
            text[..offset].iter().all(|&b| b == b' ' || b == b'\t')
        }
    };
    if should_align {
        let target_col = cmode_find_open_brace_col(editor, buf_id, line, offset)?;
        if let Some(col) = target_col {
            cmode_set_indent(editor, col)?;
        }
    }
    Ok(())
}

fn cmode_find_open_brace_col(
    editor: &Editor,
    buf_id: BufferId,
    line: LineId,
    offset: usize,
) -> Result<Option<usize>> {
    let buf = editor.buffers.get(buf_id).ok_or(Error::Abort)?;
    let mut scan_line = line;
    let mut scan_off = offset;
    let mut depth = 1usize;
    let mut found_col = None;
    'outer: loop {
        let line_text = &buf.line(scan_line).ok_or(Error::Abort)?.text;
        loop {
            if scan_off == 0 {
                match buf.prev_line(scan_line) {
                    Some(prev) if prev != buf.head => {
                        scan_line = prev;
                        scan_off = buf.line_len(scan_line).ok_or(Error::Abort)?;
                    }
                    _ => break 'outer,
                }
                break;
            }
            scan_off -= 1;
            let c = line_text[scan_off];
            if c == b'{' {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    let l = buf.line(scan_line).ok_or(Error::Abort)?;
                    let mut col = 0;
                    for &b in &l.text {
                        if b == b' ' {
                            col += 1;
                        } else if b == b'\t' {
                            col = (col + 8) & !7;
                        } else {
                            break;
                        }
                    }
                    found_col = Some(col);
                    break 'outer;
                }
            } else if c == b'}' {
                depth += 1;
            }
        }
    }
    Ok(found_col)
}

fn cmode_set_indent(editor: &mut Editor, col: usize) -> Result<()> {
    {
        let win = editor.cur_window_mut()?;
        win.dot_offset = LineOffset(0);
    }
    let mut current_col = 0usize;
    loop {
        let (buf_id2, line2, offset2) = {
            let win = editor.cur_window()?;
            (win.buffer_id, win.dot_line, win.dot_offset.0)
        };
        let buf2 = editor.buffers.get(buf_id2).ok_or(Error::Abort)?;
        let line2_len = buf2.line_len(line2).ok_or(Error::Abort)?;
        if offset2 >= line2_len {
            break;
        }
        let b = buf2.line(line2).ok_or(Error::Abort)?.text[offset2];
        if b != b' ' && b != b'\t' {
            break;
        }
        let w = if b == b'\t' { 8 - (current_col % 8) } else { 1 };
        current_col += w;
        let _ = buf2;
        ForwardDelete.execute(editor, false, 1)?;
    }
    let tabs = col / 8;
    let spaces = col % 8;
    for _ in 0..tabs {
        let (bid, ln, off) = {
            let w = editor.cur_window()?;
            (w.buffer_id, w.dot_line, w.dot_offset.0)
        };
        editor.push_undo(vec![UndoAction::Insert {
            line: ln,
            offset: off,
            data: vec![b'\t'],
        }]);
        let b = editor.buffers.get_mut(bid).ok_or(Error::Abort)?;
        b.line_mut(ln).ok_or(Error::Abort)?.text.insert(off, b'\t');
        b.flags |= BufferFlags::CHANGED;
        let w = editor.cur_window_mut()?;
        w.dot_offset = LineOffset(off + 1);
    }
    for _ in 0..spaces {
        let (bid, ln, off) = {
            let w = editor.cur_window()?;
            (w.buffer_id, w.dot_line, w.dot_offset.0)
        };
        editor.push_undo(vec![UndoAction::Insert {
            line: ln,
            offset: off,
            data: vec![b' '],
        }]);
        let b = editor.buffers.get_mut(bid).ok_or(Error::Abort)?;
        b.line_mut(ln).ok_or(Error::Abort)?.text.insert(off, b' ');
        b.flags |= BufferFlags::CHANGED;
        let w = editor.cur_window_mut()?;
        w.dot_offset = LineOffset(off + 1);
    }
    Ok(())
}

fn cmode_rewind_hash(
    editor: &mut Editor,
    buf_id: BufferId,
    line: LineId,
    offset: usize,
) -> Result<()> {
    let should_rewind = {
        let buf = editor.buffers.get(buf_id).ok_or(Error::Abort)?;
        let text = &buf.line(line).ok_or(Error::Abort)?.text;
        offset > 0 && text[..offset].iter().all(|&b| b == b' ' || b == b'\t')
    };
    if should_rewind {
        {
            let win = editor.cur_window_mut()?;
            win.dot_offset = LineOffset(0);
        }
        loop {
            let (bid, ln, off) = {
                let w = editor.cur_window()?;
                (w.buffer_id, w.dot_line, w.dot_offset.0)
            };
            let buf2 = editor.buffers.get(bid).ok_or(Error::Abort)?;
            let line_len = buf2.line_len(ln).ok_or(Error::Abort)?;
            if off >= line_len {
                break;
            }
            let b = buf2.line(ln).ok_or(Error::Abort)?.text[off];
            if b != b' ' && b != b'\t' {
                break;
            }
            let _ = buf2;
            ForwardDelete.execute(editor, false, 1)?;
        }
    }
    Ok(())
}
