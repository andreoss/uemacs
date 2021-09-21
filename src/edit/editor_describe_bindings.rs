use super::{
    Bindings, BufferFlags, BufferId, Buffers, Display, Editor, Error, LineId, LineOffset, Result,
    TerminalBackend, WindowFlags, key_code_display, region_bounds,
};
use std::borrow::Cow;

impl Editor {
    pub(super) fn describe_bindings(&mut self, bindings: &Bindings) -> Result<()> {
        let help_id = self.buffers.find_or_create("*Help*").id;
        clear_help_buffer_lines(&mut self.buffers, help_id)?;
        let mut entries: Vec<(String, Cow<'static, str>)> = bindings
            .entries()
            .into_iter()
            .map(|(kc, cmd)| (key_code_display(kc), cmd.name()))
            .collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        let mut lines = vec![
            b"Key             Binding".to_vec(),
            b"---             -------".to_vec(),
        ];
        for (key_str, cmd_name) in &entries {
            lines.push(format!("{key_str:<16} {cmd_name}").into_bytes());
        }
        insert_lines_into_buffer(&mut self.buffers, help_id, &lines)?;
        switch_window_to_buffer_first_line(self, help_id)
    }

    pub(super) fn count_words<T: TerminalBackend>(
        &self,
        term: &mut T,
        display: &mut Display,
    ) -> Result<()> {
        match count_region_stats(self) {
            Ok((c, w, l)) => {
                display.write_echo(term, &format!("Region has {w} words, {c} chars, {l} lines"))?;
            }
            Err(_) => term.beep(),
        }
        Ok(())
    }
}

pub fn clear_help_buffer_lines(buffers: &mut Buffers, help_id: BufferId) -> Result<()> {
    let help_buf = buffers.get_mut(help_id).ok_or(Error::Abort)?;
    help_buf.flags |= BufferFlags::INVISIBLE;
    help_buf.flags &= !BufferFlags::CHANGED;
    help_buf.filename.clear();
    let head = help_buf.head;
    let mut curr = help_buf.lines[head.0].next();
    let remove_ids: Vec<LineId> = std::iter::from_fn(|| {
        if curr == head {
            None
        } else {
            let id = curr;
            curr = help_buf.lines[curr.0].next();
            Some(id)
        }
    })
    .collect();
    for id in remove_ids {
        help_buf.remove(id);
    }
    Ok(())
}

pub fn insert_lines_into_buffer(
    buffers: &mut Buffers,
    buf_id: BufferId,
    lines: &[Vec<u8>],
) -> Result<()> {
    let buf = buffers.get_mut(buf_id).ok_or(Error::Abort)?;
    let mut after = buf.head;
    for text in lines {
        let mut line = crate::line::Line::new();
        line.text.clone_from(text);
        after = buf.insert_after(after, line);
    }
    Ok(())
}

pub fn switch_window_to_buffer_first_line(editor: &mut Editor, buf_id: BufferId) -> Result<()> {
    let first_line = {
        let buf = editor.buffers.get(buf_id).ok_or(Error::Abort)?;
        buf.nth_line(0).unwrap_or(buf.head)
    };
    let win = editor.cur_window_mut()?;
    win.buffer_id = buf_id;
    win.dot_line = first_line;
    win.dot_offset = LineOffset(0);
    win.set_flag(WindowFlags::HARD);
    Ok(())
}

pub fn count_region_stats(editor: &Editor) -> Result<(usize, usize, usize)> {
    let (start_line, start_off, end_line, end_off) = region_bounds(editor)?;
    let buf_id = editor.cur_window()?.buffer_id;
    let buf = editor.buffers.get(buf_id).ok_or(Error::Abort)?;

    let mut nchars = 0usize;
    let mut nwords = 0usize;
    let mut nlines = 0usize;
    let mut in_word = false;
    let mut line = start_line;
    loop {
        let line_text = &buf.line(line).ok_or(Error::Abort)?.text;
        let line_len = line_text.len();

        let range = if line == start_line && line == end_line {
            start_off..end_off
        } else if line == start_line {
            start_off..line_len
        } else if line == end_line {
            0..end_off
        } else {
            0..line_len
        };

        for &byte in &line_text[range] {
            nchars += 1;
            let word_byte = byte.is_ascii_alphanumeric() || byte >= 0x80;
            if word_byte && !in_word {
                nwords += 1;
                in_word = true;
            } else if !word_byte {
                in_word = false;
            }
        }

        if line == end_line {
            break;
        }
        nchars += 1;
        nlines += 1;
        in_word = false;

        match buf.next_line(line) {
            Some(next) => line = next,
            None => break,
        }
    }

    Ok((nchars, nwords, nlines))
}
