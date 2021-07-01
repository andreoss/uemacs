use super::{Command, Editor, Error, LineOffset, Result, WindowFlags, in_word, is_para_boundary};

pub struct GotoBop;

impl Command for GotoBop {
    fn execute(&self, editor: &mut Editor, _f: bool, mut n: usize) -> Result<()> {
        while n > 0 {
            let buffer_id = editor.cur_window()?.buffer_id;
            let (mut dot_line, mut dot_offset) = {
                let win = editor.cur_window()?;
                (win.dot_line, win.dot_offset.0)
            };

            loop {
                let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
                if in_word(buf, dot_line, dot_offset) {
                    break;
                }
                if dot_offset == 0 {
                    match buf.prev_line(dot_line) {
                        Some(prev) => {
                            dot_line = prev;
                            dot_offset = buf.line_len(prev).unwrap_or(0);
                        }
                        None => break,
                    }
                } else {
                    dot_offset -= 1;
                }
            }

            dot_offset = 0;

            loop {
                let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
                match buf.prev_line(dot_line) {
                    Some(prev) => {
                        if is_para_boundary(buf, prev) {
                            break;
                        }
                        dot_line = prev;
                    }
                    None => break,
                }
            }

            loop {
                let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
                if in_word(buf, dot_line, dot_offset) {
                    break;
                }
                let len = buf.line_len(dot_line).unwrap_or(0);
                if dot_offset >= len {
                    match buf.next_line(dot_line) {
                        Some(next) => {
                            dot_line = next;
                            dot_offset = 0;
                        }
                        None => break,
                    }
                } else {
                    dot_offset += 1;
                }
            }

            let win = editor.cur_window_mut()?;
            win.dot_line = dot_line;
            win.dot_offset = LineOffset(dot_offset);
            n -= 1;
        }
        editor
            .current_window_mut()
            .ok_or(Error::Abort)?
            .set_flag(WindowFlags::MOVED);
        Ok(())
    }
}

pub struct GotoEop;

impl Command for GotoEop {
    fn execute(&self, editor: &mut Editor, _f: bool, mut n: usize) -> Result<()> {
        while n > 0 {
            let buffer_id = editor.cur_window()?.buffer_id;
            let mut dot_line = editor.cur_window()?.dot_line;

            loop {
                let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
                if is_para_boundary(buf, dot_line) {
                    break;
                }
                match buf.next_line(dot_line) {
                    Some(next) => dot_line = next,
                    None => break,
                }
            }

            let win = editor.cur_window_mut()?;
            win.dot_line = dot_line;
            win.dot_offset = LineOffset(0);
            n -= 1;
        }
        editor
            .current_window_mut()
            .ok_or(Error::Abort)?
            .set_flag(WindowFlags::MOVED);
        Ok(())
    }
}
