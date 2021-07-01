use super::{
    BufferId, Completion, Display, Editor, Error, LineId, LineOffset, Result, TerminalBackend,
    WindowFlags,
};

impl Editor {
    pub fn switch_window_to_buffer(&mut self, new_buf_id: BufferId) -> Result<()> {
        let old_buf_id = self.cur_window()?.buffer_id;
        if new_buf_id == old_buf_id {
            self.cur_window_mut()?.flags |= WindowFlags::HARD | WindowFlags::MODE_LINE;
            return Ok(());
        }
        if self
            .windows
            .iter()
            .filter(|w| w.buffer_id == old_buf_id)
            .count()
            == 1
        {
            save_dot_to_buffer(self, old_buf_id)?;
        }
        let (dot_line, dot_offset, saved_mark, head) = resolve_new_buffer_state(self, new_buf_id)?;
        apply_buffer_switch(self, new_buf_id, dot_line, dot_offset, saved_mark, head)
    }

    pub fn ensure_dot_on_content_line(&mut self) -> Result<()> {
        let (buf_id, dot_line) = {
            let win = self.cur_window()?;
            (win.buffer_id, win.dot_line)
        };
        let head = self.buffers.get(buf_id).ok_or(Error::Abort)?.head;
        if dot_line != head {
            return Ok(());
        }
        let new_line = {
            let buf = self.buffers.get_mut(buf_id).ok_or(Error::Abort)?;
            buf.insert_after(head, crate::line::Line::new())
        };
        set_dot_on_content_line_and_mark(self, buf_id, new_line)
    }

    pub(super) fn kill_buffer<T: TerminalBackend>(
        &mut self,
        term: &mut T,
        display: &mut Display,
    ) -> Result<()> {
        let Some(buf_id) = resolve_kill_target(self, term, display)? else {
            return Ok(());
        };
        let other_buf = self.buffers.iter().find(|b| b.id != buf_id).map(|b| b.id);
        reassign_windows_from_buffer(self, buf_id, other_buf);
        self.buffers.remove(buf_id);
        Ok(())
    }
}

fn save_dot_to_buffer(editor: &mut Editor, old_buf_id: BufferId) -> Result<()> {
    let (dot_line, dot_offset, mark) = {
        let win = editor.cur_window()?;
        (win.dot_line, win.dot_offset, win.mark())
    };
    if let Some(buf) = editor.buffers.get_mut(old_buf_id) {
        buf.set_dot(dot_line, dot_offset);
        match mark {
            Some((ml, mo)) => buf.set_mark(ml, mo),
            None => buf.clear_mark(),
        }
    }
    Ok(())
}

#[allow(clippy::type_complexity)]
fn resolve_new_buffer_state(
    editor: &Editor,
    new_buf_id: BufferId,
) -> Result<(LineId, LineOffset, Option<(LineId, LineOffset)>, LineId)> {
    let (saved_dot, saved_off, saved_mark, head, is_empty) = {
        let buf = editor.buffers.get(new_buf_id).ok_or(Error::Abort)?;
        (
            buf.dot_line,
            buf.dot_offset,
            buf.mark(),
            buf.head,
            buf.is_empty(),
        )
    };
    let dot_line = if saved_dot == head {
        if is_empty {
            head
        } else {
            editor
                .buffers
                .get(new_buf_id)
                .unwrap()
                .nth_line(0)
                .unwrap_or(head)
        }
    } else {
        saved_dot
    };
    let dot_offset = if saved_dot == head {
        LineOffset(0)
    } else {
        saved_off
    };
    Ok((dot_line, dot_offset, saved_mark, head))
}

fn apply_buffer_switch(
    editor: &mut Editor,
    new_buf_id: BufferId,
    dot_line: LineId,
    dot_offset: LineOffset,
    saved_mark: Option<(LineId, LineOffset)>,
    head: LineId,
) -> Result<()> {
    let win = editor.cur_window_mut()?;
    win.buffer_id = new_buf_id;
    win.top_line = dot_line;
    win.dot_line = dot_line;
    win.dot_offset = dot_offset;
    match saved_mark {
        Some((ml, mo)) if ml != head => win.set_mark(ml, mo),
        _ => win.clear_mark(),
    }
    win.flags |= WindowFlags::HARD | WindowFlags::MODE_LINE;
    Ok(())
}

fn set_dot_on_content_line_and_mark(
    editor: &mut Editor,
    buf_id: BufferId,
    new_line: LineId,
) -> Result<()> {
    let win = editor.cur_window_mut()?;
    win.top_line = new_line;
    win.dot_line = new_line;
    win.dot_offset = LineOffset(0);
    for w in editor.windows.iter_mut() {
        if w.buffer_id == buf_id {
            w.flags |= WindowFlags::MODE_LINE;
        }
    }
    Ok(())
}

fn resolve_kill_target<T: TerminalBackend>(
    editor: &Editor,
    term: &mut T,
    display: &mut Display,
) -> Result<Option<BufferId>> {
    let name = editor.minibuffer_readline_opts(
        term,
        display,
        "Kill buffer: ",
        true,
        Completion::Buffer,
    )?;
    let target = if name.is_empty() {
        let buf = editor
            .buffers
            .get(editor.cur_window()?.buffer_id)
            .ok_or(Error::Abort)?;
        buf.name.clone()
    } else {
        name
    };
    if editor.buffers.len() <= 1 {
        term.beep();
        return Ok(None);
    }
    editor.buffers.find(&target).map_or_else(
        || {
            term.beep();
            Ok(None)
        },
        |b| Ok(Some(b.id)),
    )
}

fn reassign_windows_from_buffer(
    editor: &mut Editor,
    buf_id: BufferId,
    other_buf: Option<BufferId>,
) {
    let new_top = other_buf
        .and_then(|other| {
            let buf = editor.buffers.get(other)?;
            Some(if buf.is_empty() {
                buf.head
            } else {
                buf.nth_line(0).unwrap_or(buf.head)
            })
        })
        .unwrap_or(LineId(0));
    for win in editor.windows.iter_mut() {
        if win.buffer_id == buf_id {
            if let Some(other) = other_buf {
                win.buffer_id = other;
                win.top_line = new_top;
                win.dot_line = new_top;
                win.dot_offset = LineOffset(0);
                win.clear_mark();
                win.flags |= WindowFlags::HARD | WindowFlags::MODE_LINE;
            }
        }
    }
}
