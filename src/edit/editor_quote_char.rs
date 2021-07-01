use super::{
    Bindings, BufferFlags, BufferId, Buffers, Command, Display, Editor, Error, InsertNewline, Key,
    LineId, LineOffset, Mode, Result, TerminalBackend, UndoAction, WindowFlags,
    clear_help_buffer_lines, command_name, key_to_bytes, insert_lines_into_buffer,
    switch_window_to_buffer_first_line,
};
use std::borrow::Cow;

impl Editor {
    pub(super) fn quote_char<T: TerminalBackend>(&mut self, term: &mut T, n: usize) -> Result<()> {
        let Some(key) = term.get_key() else {
            return Ok(());
        };
        let count = n.max(1);
        if matches!(key, Key::Enter | Key::Char('\n' | '\r')) {
            for _ in 0..count {
                InsertNewline.execute(self, false, 1)?;
            }
            return Ok(());
        }
        match key_to_bytes(&key) {
            Some(data) => {
                for _ in 0..count {
                    insert_char_data(self, &data)?;
                }
            }
            None => term.beep(),
        }
        Ok(())
    }

    pub(super) fn help_command(&mut self) -> Result<()> {
        let help_buf_id = ensure_help_buffer_populated(&mut self.buffers);
        let (buffer_id, top_line, n_rows) = {
            let win = self.cur_window()?;
            (win.buffer_id, win.top_line, win.n_rows)
        };
        if n_rows < 2 {
            return Err(Error::Abort);
        }
        split_window_for_help(self, help_buf_id, buffer_id, top_line, n_rows)?;
        if let Some(buf) = self.buffers.get_mut(help_buf_id) {
            buf.mode |= Mode::VIEW;
        }
        for win in self.windows.iter_mut() {
            win.flags |= WindowFlags::HARD | WindowFlags::MODE_LINE;
        }
        Ok(())
    }

    pub(super) fn apropos<T: TerminalBackend>(
        &mut self,
        term: &mut T,
        display: &mut Display,
        bindings: &Bindings,
    ) -> Result<()> {
        let pattern = self.minibuffer_readline(term, display, "Apropos string: ")?;
        if pattern.is_empty() {
            return Ok(());
        }
        let help_id = self.buffers.find_or_create("*Help*").id;
        clear_help_buffer_lines(&mut self.buffers, help_id)?;
        let matches = find_apropos_matches(bindings, &pattern);
        let mut lines = vec![
            format!("Commands matching '{pattern}':").into_bytes(),
            b"".to_vec(),
        ];
        for (name, desc) in &matches {
            lines.push(format!("{name:<32} {desc}").into_bytes());
        }
        insert_lines_into_buffer(&mut self.buffers, help_id, &lines)?;
        switch_window_to_buffer_first_line(self, help_id)
    }
}

fn insert_char_data(editor: &mut Editor, data: &[u8]) -> Result<()> {
    let (buf_id, line, offset) = {
        let win = editor.cur_window()?;
        (win.buffer_id, win.dot_line, win.dot_offset.0)
    };
    editor.push_undo(vec![UndoAction::Insert {
        line,
        offset,
        data: data.to_vec(),
    }]);
    let buf = editor.buffers.get_mut(buf_id).ok_or(Error::Abort)?;
    let line_mut = buf.line_mut(line).ok_or(Error::Abort)?;
    line_mut.insert_bytes(offset, data)?;
    buf.flags |= BufferFlags::CHANGED;
    let win = editor.cur_window_mut()?;
    win.dot_offset = LineOffset(offset + data.len());
    win.set_flag(WindowFlags::EDITED);
    Ok(())
}

fn ensure_help_buffer_populated(buffers: &mut Buffers) -> BufferId {
    let buf = buffers.find_or_create("*help*");
    buf.flags |= BufferFlags::INVISIBLE;
    if !buf.is_empty() {
        return buf.id;
    }
    let text = b"Uemacs Help\n\n\
        C-F  forward-character    C-B  backward-character\n\
        C-N  forward-line         C-P  backward-line\n\
        C-A  beginning-of-line    C-E  end-of-line\n\
        C-V  next-page            M-V  previous-page\n\
        C-D  delete-next-character  C-H  delete-previous-character\n\
        C-K  kill-to-end-of-line  C-Y  yank\n\
        C-S  isearch-forward      C-R  isearch-backward\n\
        M-S  search-forward       M-R  search-reverse\n\
        C-X C-F  find-file        C-X C-S  save-file\n\
        C-X C-W  write-file       C-X C-C  exit-emacs\n\
        C-X C-B  list-buffers     M-Z  quick-exit\n\
        M-X      execute-command  C-G  abort-command";
    let head = buf.head;
    let mut prev = head;
    for line_text in text.split(|&b| b == b'\n') {
        let line_id = buf.insert_after(prev, crate::line::Line::new());
        let line = buf.line_mut(line_id).unwrap();
        line.text = line_text.to_vec();
        prev = line_id;
    }
    buf.id
}

fn split_window_for_help(
    editor: &mut Editor,
    help_buf_id: BufferId,
    buffer_id: BufferId,
    top_line: LineId,
    n_rows: usize,
) -> Result<()> {
    let half = n_rows / 2;
    let new_top_row = half + 1;
    {
        let win = editor.cur_window_mut()?;
        win.n_rows = half;
        win.set_flag(WindowFlags::HARD);
    }
    let new_id = editor.create_window(buffer_id, top_line);
    {
        let new_win = editor.windows.get_mut(new_id).ok_or(Error::Abort)?;
        new_win.top_row = new_top_row;
        new_win.n_rows = n_rows.saturating_sub(half + 1);
        new_win.buffer_id = help_buf_id;
        new_win.set_flag(WindowFlags::HARD);
    }
    editor.windows.set_current(new_id);
    Ok(())
}

pub fn find_apropos_matches(
    bindings: &Bindings,
    pattern: &str,
) -> Vec<(Cow<'static, str>, &'static str)> {
    let mut seen = std::collections::HashSet::new();
    let mut matches: Vec<(Cow<'static, str>, &'static str)> = Vec::new();
    for (_, cmd) in bindings.entries() {
        let name = command_name(cmd);
        if (name.contains(pattern)
            || crate::bind::command_description(cmd)
                .to_ascii_lowercase()
                .contains(&pattern.to_ascii_lowercase()))
            && seen.insert(name.clone())
        {
            matches.push((name, crate::bind::command_description(cmd)));
        }
    }
    matches.sort_by_key(|(n, _)| n.clone());
    matches
}
