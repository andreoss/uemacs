use super::{
    CmdFlags, Command, Editor, Error, LineOffset, Result, WindowFlags, kill_n_chars, region_bounds,
    transform_region,
};

pub struct KillRegion;

impl Command for KillRegion {
    fn execute(&self, editor: &mut Editor, _f: bool, _n: usize) -> Result<()> {
        let (start_line, start_off, end_line, end_off) = region_bounds(editor)?;

        if !editor.last_flag.intersects(CmdFlags::KILL) {
            editor.kdelete();
        }
        editor.this_flag |= CmdFlags::KILL;

        let buffer_id = editor.cur_window()?.buffer_id;

        let region_size = if start_line == end_line {
            end_off.saturating_sub(start_off)
        } else {
            let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
            let mut region_size = buf
                .line_len(start_line)
                .ok_or(Error::Abort)?
                .saturating_sub(start_off);
            let mut current = start_line;
            loop {
                match buf.next_line(current) {
                    Some(next) => {
                        region_size += 1;
                        if next == end_line {
                            region_size += end_off;
                            break;
                        }
                        region_size += buf.line_len(next).ok_or(Error::Abort)?;
                        current = next;
                    }
                    None => return Err(Error::Abort),
                }
            }
            region_size
        };

        {
            let win = editor.cur_window_mut()?;
            win.dot_line = start_line;
            win.dot_offset = LineOffset(start_off);
        }

        kill_n_chars(editor, region_size)?;

        editor
            .current_window_mut()
            .ok_or(Error::Abort)?
            .clear_mark();
        editor
            .current_window_mut()
            .ok_or(Error::Abort)?
            .set_flag(WindowFlags::HARD);
        Ok(())
    }
}

pub struct CopyRegion;

impl Command for CopyRegion {
    fn execute(&self, editor: &mut Editor, _f: bool, _n: usize) -> Result<()> {
        let (start_line, start_off, end_line, end_off) = region_bounds(editor)?;

        if !editor.last_flag.intersects(CmdFlags::KILL) {
            editor.kdelete();
        }
        editor.this_flag |= CmdFlags::KILL;

        let buffer_id = editor.cur_window()?.buffer_id;

        let mut copied = Vec::new();
        if start_line == end_line {
            let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
            let text = &buf.line(start_line).ok_or(Error::Abort)?.text;
            copied.extend_from_slice(&text[start_off..end_off]);
        } else {
            {
                let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
                let text = &buf.line(start_line).ok_or(Error::Abort)?.text;
                copied.extend_from_slice(&text[start_off..]);
            }
            let mut current = start_line;
            loop {
                let next = {
                    let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
                    buf.next_line(current)
                };
                match next {
                    Some(next_line) => {
                        copied.push(b'\n');
                        if next_line == end_line {
                            let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
                            let text = &buf.line(next_line).ok_or(Error::Abort)?.text;
                            copied.extend_from_slice(&text[..end_off]);
                            break;
                        }
                        let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
                        let text = &buf.line(next_line).ok_or(Error::Abort)?.text;
                        copied.extend_from_slice(text);
                        current = next_line;
                    }
                    None => return Err(Error::Abort),
                }
            }
        }

        editor.kill_buffer.extend(copied);
        Ok(())
    }
}

pub struct LowerRegion;

impl Command for LowerRegion {
    fn execute(&self, editor: &mut Editor, _f: bool, _n: usize) -> Result<()> {
        transform_region(editor, |b| b.to_ascii_lowercase())
    }
}

pub struct UpperRegion;

impl Command for UpperRegion {
    fn execute(&self, editor: &mut Editor, _f: bool, _n: usize) -> Result<()> {
        transform_region(editor, |b| b.to_ascii_uppercase())
    }
}
