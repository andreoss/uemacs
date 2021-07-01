use super::{
    BufferFlags, Command, Editor, Error, Result, UndoAction, WindowFlags, advance_dot_one_line,
};

pub struct DetabLine;

impl Command for DetabLine {
    fn execute(&self, editor: &mut Editor, _f: bool, n: usize) -> Result<()> {
        let count = n.max(1);
        let tw = editor.tab_width;
        let mut undo_actions: Vec<UndoAction> = Vec::new();
        for i in 0..count {
            let (buf_id, line) = {
                let win = editor.cur_window()?;
                (win.buffer_id, win.dot_line)
            };
            let buf = editor.buffers.get_mut(buf_id).ok_or(Error::Abort)?;
            let line_mut = buf.line_mut(line).ok_or(Error::Abort)?;
            let mut new_text = Vec::with_capacity(line_mut.text.len());
            let mut col = 0usize;
            for &b in &line_mut.text {
                if b == b'\t' {
                    let spaces = tw - (col % tw);
                    new_text.extend(std::iter::repeat_n(b' ', spaces));
                    col += spaces;
                } else {
                    new_text.push(b);
                    col += 1;
                }
            }
            if new_text != line_mut.text {
                let old_text = std::mem::replace(&mut line_mut.text, new_text.clone());
                undo_actions.push(UndoAction::Delete {
                    line,
                    offset: 0,
                    data: old_text,
                });
                undo_actions.push(UndoAction::Insert {
                    line,
                    offset: 0,
                    data: new_text,
                });
                buf.flags |= BufferFlags::CHANGED;
            }
            if i + 1 < count && !advance_dot_one_line(editor) {
                break;
            }
        }
        if !undo_actions.is_empty() {
            editor.push_undo(undo_actions);
        }
        editor
            .current_window_mut()
            .ok_or(Error::Abort)?
            .set_flag(WindowFlags::EDITED);
        Ok(())
    }
}

pub struct EntabLine;

impl Command for EntabLine {
    fn execute(&self, editor: &mut Editor, _f: bool, n: usize) -> Result<()> {
        let count = n.max(1);
        let tw = editor.tab_width;
        let mut undo_actions: Vec<UndoAction> = Vec::new();
        for iter in 0..count {
            let (buf_id, line) = {
                let win = editor.cur_window()?;
                (win.buffer_id, win.dot_line)
            };
            let buf = editor.buffers.get_mut(buf_id).ok_or(Error::Abort)?;
            let text = buf.line_mut(line).ok_or(Error::Abort)?;
            let mut new_text = Vec::with_capacity(text.text.len());
            let mut col = 0usize;
            let mut i = 0usize;
            while i < text.text.len() {
                if text.text[i] == b' ' {
                    let start_col = col;
                    let mut j = i;
                    while j < text.text.len() && text.text[j] == b' ' {
                        j += 1;
                        col += 1;
                    }
                    let run_len = j - i;
                    let mut remaining = run_len;
                    let mut tab_col = start_col;
                    while remaining > 0 {
                        let to_stop = tw - (tab_col % tw);
                        if to_stop == 0 || to_stop == tw {
                            if remaining >= tw && tw > 1 {
                                new_text.push(b'\t');
                                remaining -= tw;
                                tab_col += tw;
                            } else {
                                break;
                            }
                        } else if to_stop < remaining && to_stop > 1 {
                            new_text.push(b'\t');
                            remaining -= to_stop;
                            tab_col += to_stop;
                        } else {
                            break;
                        }
                    }
                    new_text.extend(std::iter::repeat_n(b' ', remaining));
                    i = j;
                } else if text.text[i] == b'\t' {
                    new_text.push(b'\t');
                    col += tw - (col % tw);
                    i += 1;
                } else {
                    new_text.push(text.text[i]);
                    col += 1;
                    i += 1;
                }
            }
            if new_text != text.text {
                let old_text = std::mem::replace(&mut text.text, new_text.clone());
                undo_actions.push(UndoAction::Delete {
                    line,
                    offset: 0,
                    data: old_text,
                });
                undo_actions.push(UndoAction::Insert {
                    line,
                    offset: 0,
                    data: new_text,
                });
                buf.flags |= BufferFlags::CHANGED;
            }
            if iter + 1 < count && !advance_dot_one_line(editor) {
                break;
            }
        }
        if !undo_actions.is_empty() {
            editor.push_undo(undo_actions);
        }
        editor
            .current_window_mut()
            .ok_or(Error::Abort)?
            .set_flag(WindowFlags::EDITED);
        Ok(())
    }
}
