use super::{Command, Editor, Error, LineOffset, Result, WindowFlags, is_utf8_start_byte};

pub struct GotoBol;

impl Command for GotoBol {
    fn execute(&self, editor: &mut Editor, _f: bool, _n: usize) -> Result<()> {
        let win = editor.cur_window_mut()?;
        win.dot_offset = LineOffset(0);
        Ok(())
    }
}

pub struct GotoEol;

impl Command for GotoEol {
    fn execute(&self, editor: &mut Editor, _f: bool, _n: usize) -> Result<()> {
        let (buffer_id, dot_line) = {
            let win = editor.cur_window()?;
            (win.buffer_id, win.dot_line)
        };
        let len = editor
            .buffers
            .get(buffer_id)
            .and_then(|b| b.line_len(dot_line))
            .ok_or(Error::Abort)?;
        let win = editor.cur_window_mut()?;
        win.dot_offset = LineOffset(len);
        Ok(())
    }
}

pub struct ForwardChar;

impl Command for ForwardChar {
    fn execute(&self, editor: &mut Editor, _f: bool, mut n: usize) -> Result<()> {
        while n > 0 {
            let (buffer_id, dot_line, dot_offset) = {
                let win = editor.cur_window()?;
                (win.buffer_id, win.dot_line, win.dot_offset.0)
            };
            let (len, line_text) = {
                let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
                let len = buf.line_len(dot_line).ok_or(Error::Abort)?;
                let text: Vec<u8> = buf.line(dot_line).unwrap().text.clone();
                (len, text)
            };
            if dot_offset >= len {
                let next = {
                    let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
                    buf.next_line(dot_line).ok_or(Error::Abort)?
                };
                let win = editor.cur_window_mut()?;
                win.dot_line = next;
                win.dot_offset = LineOffset(0);
                win.set_flag(WindowFlags::MOVED);
            } else {
                let mut new_offset = dot_offset + 1;
                while new_offset < len {
                    let byte = line_text[new_offset];
                    if is_utf8_start_byte(byte) {
                        break;
                    }
                    new_offset += 1;
                }
                let win = editor.cur_window_mut()?;
                win.dot_offset = LineOffset(new_offset);
            }
            n -= 1;
        }
        Ok(())
    }
}

pub struct BackwardChar;

impl Command for BackwardChar {
    fn execute(&self, editor: &mut Editor, _f: bool, mut n: usize) -> Result<()> {
        while n > 0 {
            let (buffer_id, dot_line, dot_offset) = {
                let win = editor.cur_window()?;
                (win.buffer_id, win.dot_line, win.dot_offset.0)
            };
            if dot_offset == 0 {
                let prev = {
                    let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
                    buf.prev_line(dot_line).ok_or(Error::Abort)?
                };
                let prev_len = {
                    let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
                    buf.line_len(prev).ok_or(Error::Abort)?
                };
                let win = editor.cur_window_mut()?;
                win.dot_line = prev;
                win.dot_offset = LineOffset(prev_len);
                win.set_flag(WindowFlags::MOVED);
            } else {
                let line_text = {
                    let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
                    buf.line(dot_line).unwrap().text.clone()
                };
                let mut new_offset = dot_offset - 1;
                while new_offset > 0 {
                    let byte = line_text[new_offset];
                    if is_utf8_start_byte(byte) {
                        break;
                    }
                    new_offset -= 1;
                }
                let win = editor.cur_window_mut()?;
                win.dot_offset = LineOffset(new_offset);
            }
            n -= 1;
        }
        Ok(())
    }
}
