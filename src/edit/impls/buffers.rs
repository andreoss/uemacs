use super::{BufferFlags, Command, Editor, Error, LineId, LineOffset, Result, WindowFlags};

pub struct ListBuffers;

impl Command for ListBuffers {
    fn execute(&self, editor: &mut Editor, _f: bool, _n: usize) -> Result<()> {
        let list_name = "*List*";
        let list_id = editor.buffers.find_or_create(list_name).id;

        {
            let list_buf = editor.buffers.get_mut(list_id).ok_or(Error::Abort)?;
            list_buf.flags |= BufferFlags::INVISIBLE;
            list_buf.flags &= !BufferFlags::CHANGED;
            list_buf.filename.clear();

            let head = list_buf.head;
            let mut curr = list_buf.lines[head.0].next();
            let remove_ids: Vec<LineId> = std::iter::from_fn(|| {
                if curr == head {
                    None
                } else {
                    let id = curr;
                    curr = list_buf.lines[curr.0].next();
                    Some(id)
                }
            })
            .collect();
            for id in remove_ids {
                list_buf.remove(id);
            }
        }

        let mut lines: Vec<Vec<u8>> = Vec::new();
        lines.push(b" MR Size  Name          File".to_vec());
        lines.push(b" -- ----  ----          ----".to_vec());

        for buf in editor.buffers.iter() {
            if buf.flags.intersects(BufferFlags::INVISIBLE) {
                continue;
            }
            let modified = if buf.flags.intersects(BufferFlags::CHANGED) {
                '*'
            } else {
                ' '
            };
            let size = buf.line_count();
            let name = &buf.name;
            let fname = &buf.filename;
            let line = format!(
                " {:>2} {:>5}  {:<14} {}",
                modified,
                size,
                name,
                if fname.is_empty() { "" } else { fname.as_str() }
            );
            lines.push(line.into_bytes());
        }

        {
            let list_buf = editor.buffers.get_mut(list_id).ok_or(Error::Abort)?;
            let mut after = list_buf.head;
            for text in &lines {
                let mut line = crate::line::Line::new();
                line.text.clone_from(text);
                after = list_buf.insert_after(after, line);
            }
        }

        {
            let list_buf = editor.buffers.get(list_id).ok_or(Error::Abort)?;
            let first_line = list_buf.lines[list_buf.head.0].next();
            let win = editor.cur_window_mut()?;
            win.buffer_id = list_id;
            win.dot_line = first_line;
            win.dot_offset = LineOffset(0);
            win.set_flag(WindowFlags::HARD | WindowFlags::MODE_LINE);
        }

        Ok(())
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub struct SaveFile;

impl Command for SaveFile {
    fn execute(&self, editor: &mut Editor, _f: bool, _n: usize) -> Result<()> {
        let buf_id = editor.cur_window()?.buffer_id;
        let (fname, modified) = {
            let buf = editor.buffers.get(buf_id).ok_or(Error::Abort)?;
            (buf.filename.clone(), buf.flags.intersects(BufferFlags::CHANGED))
        };
        if !modified {
            return Ok(());
        }
        if fname.is_empty() {
            return Err(Error::Abort);
        }
        let buf = editor.buffers.get_mut(buf_id).ok_or(Error::Abort)?;
        crate::file::write_from_buffer(buf, &fname)?;
        for win in editor.windows.iter_mut() {
            if win.buffer_id == buf_id {
                win.set_flag(WindowFlags::MODE_LINE);
            }
        }
        Ok(())
    }
}
