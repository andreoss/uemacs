use super::{
    Buffer, BufferFlags, CommandId, Editor, Error, LineId, LineOffset, Result, StringInsertMode,
    UndoAction, WindowFlags,
};
#[cfg(test)]
use super::{Key, ctlx_command};

pub fn splice_string(editor: &mut Editor, s: &str, mode: StringInsertMode) -> Result<()> {
    let (buf_id, dot_line, dot_offset) = {
        let win = editor.cur_window()?;
        (win.buffer_id, win.dot_line, win.dot_offset.0)
    };
    let buf = editor.buffers.get_mut(buf_id).ok_or(Error::Abort)?;
    buf.flags |= BufferFlags::CHANGED;
    let (cl, co, ua) = splice_string_loop(buf, dot_line, dot_offset, s, mode)?;
    if !ua.is_empty() {
        editor.push_undo(ua);
    }
    let win = editor.cur_window_mut()?;
    win.dot_line = cl;
    win.dot_offset = LineOffset(co);
    win.set_flag(WindowFlags::HARD);
    Ok(())
}

fn splice_string_loop(
    buf: &mut Buffer,
    mut cl: LineId,
    mut co: usize,
    s: &str,
    mode: StringInsertMode,
) -> Result<(LineId, usize, Vec<UndoAction>)> {
    let mut ua = Vec::new();
    for ch in s.bytes() {
        if ch == b'\n' {
            (cl, co) = splice_newline(buf, cl, co, mode, &mut ua)?;
        } else {
            co = splice_byte(buf, cl, co, ch, mode, &mut ua)?;
        }
    }
    Ok((cl, co, ua))
}

fn splice_newline(
    buf: &mut Buffer,
    cur_line: LineId,
    cur_off: usize,
    mode: StringInsertMode,
    ua: &mut Vec<UndoAction>,
) -> Result<(LineId, usize)> {
    let line_len = buf.line(cur_line).map_or(0, |l| l.text.len());
    let split_off = cur_off.min(line_len);
    let new_id = buf.insert_after(cur_line, crate::line::Line::new());
    ua.push(UndoAction::Split {
        line: cur_line,
        offset: split_off,
        new_line: new_id,
    });
    if cur_off < line_len {
        splice_newline_split(buf, cur_line, cur_off, new_id, mode, ua)?;
    }
    Ok((new_id, 0))
}

fn splice_newline_split(
    buf: &mut Buffer,
    cur_line: LineId,
    cur_off: usize,
    new_id: LineId,
    mode: StringInsertMode,
    ua: &mut Vec<UndoAction>,
) -> Result<()> {
    let tail = buf.line_mut(cur_line).ok_or(Error::Abort)?;
    let rest = tail.text.split_off(cur_off);
    match mode {
        StringInsertMode::Insert => {
            buf.line_mut(new_id).ok_or(Error::Abort)?.text = rest;
        }
        StringInsertMode::Overwrite => {
            ua.push(UndoAction::Delete {
                line: cur_line,
                offset: cur_off,
                data: rest,
            });
        }
    }
    Ok(())
}

fn splice_byte(
    buf: &mut Buffer,
    cur_line: LineId,
    cur_off: usize,
    ch: u8,
    mode: StringInsertMode,
    ua: &mut Vec<UndoAction>,
) -> Result<usize> {
    match mode {
        StringInsertMode::Insert => splice_byte_insert(buf, cur_line, cur_off, ch, ua),
        StringInsertMode::Overwrite => splice_byte_overwrite(buf, cur_line, cur_off, ch, ua),
    }
}

fn splice_byte_insert(
    buf: &mut Buffer,
    cur_line: LineId,
    cur_off: usize,
    ch: u8,
    ua: &mut Vec<UndoAction>,
) -> Result<usize> {
    let line = buf.line_mut(cur_line).ok_or(Error::Abort)?;
    line.text.insert(cur_off, ch);
    ua.push(UndoAction::Insert {
        line: cur_line,
        offset: cur_off,
        data: vec![ch],
    });
    Ok(cur_off + 1)
}

fn splice_byte_overwrite(
    buf: &mut Buffer,
    cur_line: LineId,
    cur_off: usize,
    ch: u8,
    ua: &mut Vec<UndoAction>,
) -> Result<usize> {
    let line = buf.line_mut(cur_line).ok_or(Error::Abort)?;
    if cur_off < line.text.len() {
        let old = line.text[cur_off];
        line.text[cur_off] = ch;
        ua.push(UndoAction::Delete {
            line: cur_line,
            offset: cur_off,
            data: vec![old],
        });
        ua.push(UndoAction::Insert {
            line: cur_line,
            offset: cur_off,
            data: vec![ch],
        });
    } else {
        let push_off = line.text.len();
        line.text.push(ch);
        ua.push(UndoAction::Insert {
            line: cur_line,
            offset: push_off,
            data: vec![ch],
        });
    }
    Ok(cur_off + 1)
}

pub fn read_status_message(nlines: usize, is_new: bool) -> String {
    if is_new {
        "(New file)".to_string()
    } else if nlines == 1 {
        "(Read 1 line)".to_string()
    } else {
        format!("(Read {nlines} lines)")
    }
}

pub const fn creates_text(cmd: CommandId) -> bool {
    matches!(
        cmd,
        CommandId::InsertChar(_)
            | CommandId::InsertNewline
            | CommandId::InsertTab
            | CommandId::InsertSpace
            | CommandId::InsertString
            | CommandId::InsertFile
            | CommandId::OpenLine
            | CommandId::NewlineAndIndent
            | CommandId::OverwriteString
            | CommandId::QuoteChar
            | CommandId::WrapWord
            | CommandId::Yank
            | CommandId::ReplaceString
            | CommandId::QueryReplace
    )
}

pub const fn mutates_buffer(cmd: CommandId) -> bool {
    matches!(
        cmd,
        CommandId::InsertChar(_)
            | CommandId::InsertNewline
            | CommandId::InsertTab
            | CommandId::InsertSpace
            | CommandId::InsertString
            | CommandId::InsertFile
            | CommandId::Yank
            | CommandId::ForwardDelete
            | CommandId::BackwardDelete
            | CommandId::DeleteForwardWord
            | CommandId::DeleteBackwardWord
            | CommandId::DeleteBlankLines
            | CommandId::KillLine
            | CommandId::KillText
            | CommandId::KillRegion
            | CommandId::KillParagraph
            | CommandId::OpenLine
            | CommandId::NewlineAndIndent
            | CommandId::OverwriteString
            | CommandId::TrimLine
            | CommandId::DetabLine
            | CommandId::EntabLine
            | CommandId::Undo
            | CommandId::ReadFile
            | CommandId::LowerRegion
            | CommandId::UpperRegion
            | CommandId::LowerWord
            | CommandId::UpperWord
            | CommandId::CapWord
            | CommandId::WrapWord
            | CommandId::TransposeChars
            | CommandId::QuoteChar
            | CommandId::FillPara
            | CommandId::JustifyPara
            | CommandId::ReplaceString
            | CommandId::QueryReplace
    )
}

#[cfg(test)]
pub const fn ctlx_command_for_test(key: &Key) -> Option<CommandId> {
    ctlx_command(key)
}
