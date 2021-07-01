use super::{
    BackwardChar, BufferFlags, Command, Editor, Error, ForwardChar, LineId, Result, UndoAction,
    WindowFlags,
};

pub const fn is_letter(b: u8) -> bool {
    b.is_ascii_alphabetic() || b >= 0x80
}

pub fn is_para_boundary(buf: &crate::buffer::Buffer, line: LineId) -> bool {
    match buf.line(line).and_then(|l| l.text.first()) {
        Some(&b) => !is_letter(b),
        None => true,
    }
}

pub const fn is_digit(b: u8) -> bool {
    b.is_ascii_digit()
}

pub const fn is_word_char(b: u8) -> bool {
    is_letter(b) || is_digit(b)
}

#[cfg(test)]
pub const fn is_word_char_for_test(b: u8) -> bool {
    is_word_char(b)
}

pub fn in_word(buf: &crate::buffer::Buffer, line: LineId, offset: usize) -> bool {
    buf.line(line)
        .and_then(|l| l.text.get(offset))
        .is_some_and(|&b| is_letter(b))
}

pub fn char_at_dot_is_word_char(editor: &Editor) -> Result<bool> {
    let win = editor.cur_window()?;
    let buf = editor.buffers.get(win.buffer_id).ok_or(Error::Abort)?;
    let offset = win.dot_offset.0;
    let line = buf.line(win.dot_line).ok_or(Error::Abort)?;
    if offset >= line.len() {
        Ok(false)
    } else {
        Ok(is_word_char(line.text[offset]))
    }
}

pub fn at_eol(editor: &Editor) -> Result<bool> {
    let win = editor.cur_window()?;
    let buf = editor.buffers.get(win.buffer_id).ok_or(Error::Abort)?;
    let offset = win.dot_offset.0;
    let len = buf.line_len(win.dot_line).ok_or(Error::Abort)?;
    Ok(offset >= len)
}

pub fn skip_non_word_forward(editor: &mut Editor) -> Result<usize> {
    let mut count = 0;
    while !char_at_dot_is_word_char(editor)? {
        ForwardChar.execute(editor, false, 1)?;
        count += 1;
    }
    Ok(count)
}

pub fn skip_word_forward(editor: &mut Editor) -> Result<usize> {
    let mut count = 0;
    while char_at_dot_is_word_char(editor)? {
        ForwardChar.execute(editor, false, 1)?;
        count += 1;
    }
    Ok(count)
}

pub fn skip_non_word_backward(editor: &mut Editor) -> Result<usize> {
    let mut count = 0;
    while !char_at_dot_is_word_char(editor)? {
        BackwardChar.execute(editor, false, 1)?;
        count += 1;
    }
    Ok(count)
}

pub fn skip_word_backward(editor: &mut Editor) -> Result<usize> {
    let mut count = 0;
    while char_at_dot_is_word_char(editor)? {
        BackwardChar.execute(editor, false, 1)?;
        count += 1;
    }
    Ok(count)
}

pub fn word_byte_at_dot(editor: &Editor) -> Result<Option<u8>> {
    let win = editor.cur_window()?;
    let buf = editor.buffers.get(win.buffer_id).ok_or(Error::Abort)?;
    let line = buf.line(win.dot_line).ok_or(Error::Abort)?;
    let off = win.dot_offset.0;
    Ok(if off >= line.len() {
        None
    } else {
        let b = line.text[off];
        is_word_char(b).then_some(b)
    })
}

pub fn put_byte_at_dot(editor: &mut Editor, byte: u8) -> Result<()> {
    let (buf_id, line, off) = {
        let win = editor.cur_window()?;
        (win.buffer_id, win.dot_line, win.dot_offset.0)
    };
    editor
        .buffers
        .get_mut(buf_id)
        .ok_or(Error::Abort)?
        .line_mut(line)
        .ok_or(Error::Abort)?
        .text[off] = byte;
    Ok(())
}

pub fn case_words<F>(editor: &mut Editor, mut n: usize, mut transform: F) -> Result<()>
where
    F: FnMut(usize, u8) -> Option<u8>,
{
    let mut mutated = false;
    let mut undo_actions: Vec<UndoAction> = Vec::new();
    while n > 0 {
        let _ = skip_non_word_forward(editor)?;
        let mut idx = 0usize;
        while let Some(b) = word_byte_at_dot(editor)? {
            if let Some(new_b) = transform(idx, b) {
                let (line, off) = {
                    let win = editor.cur_window()?;
                    (win.dot_line, win.dot_offset.0)
                };
                undo_actions.push(UndoAction::Delete {
                    line,
                    offset: off,
                    data: vec![b],
                });
                undo_actions.push(UndoAction::Insert {
                    line,
                    offset: off,
                    data: vec![new_b],
                });
                put_byte_at_dot(editor, new_b)?;
                mutated = true;
            }
            ForwardChar.execute(editor, false, 1)?;
            idx += 1;
        }
        n -= 1;
    }
    if !undo_actions.is_empty() {
        editor.push_undo(undo_actions);
    }
    let buffer_id = editor.cur_window()?.buffer_id;
    editor.buffers.get_mut(buffer_id).ok_or(Error::Abort)?.flags |= BufferFlags::CHANGED;
    if mutated {
        editor.cur_window_mut()?.flags |= WindowFlags::EDITED | WindowFlags::MODE_LINE;
    }
    Ok(())
}
