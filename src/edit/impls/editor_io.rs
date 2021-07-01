use super::{
    BufferFlags, BufferId, Display, Editor, Key, Result, TerminalBackend, UndoAction, UndoEntry,
    find_common_prefix,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Completion {
    Filename,
    Buffer,
}

impl Editor {
    pub fn push_undo(&mut self, actions: Vec<UndoAction>) {
        let (dot_line, dot_offset) = self
            .current_window()
            .map(|w| (w.dot_line, w.dot_offset))
            .unwrap_or_default();
        let buffer_id = self.current_window().map_or(BufferId(0), |w| w.buffer_id);
        self.undo_stack.push(UndoEntry {
            buffer_id,
            dot_line,
            dot_offset,
            actions,
        });
    }

    pub fn minibuffer_readline<T: TerminalBackend>(
        &self,
        term: &mut T,
        display: &mut Display,
        prompt: &str,
    ) -> Result<String> {
        self.minibuffer_readline_opts(term, display, prompt, true, Completion::Filename)
    }

    pub fn complete_buffer_name(&self, prefix: &str) -> Vec<String> {
        let mut names: Vec<String> = self
            .buffers
            .iter()
            .filter(|b| !b.flags.intersects(BufferFlags::INVISIBLE) && b.name.starts_with(prefix))
            .map(|b| b.name.clone())
            .collect();
        names.sort();
        names
    }

    pub fn minibuffer_readline_opts<T: TerminalBackend>(
        &self,
        term: &mut T,
        display: &mut Display,
        prompt: &str,
        echo: bool,
        completion: Completion,
    ) -> Result<String> {
        let mut buf = String::new();
        let mut cycle: Option<(Vec<String>, usize)> = None;
        let render = |buf: &str| -> String {
            if echo {
                format!("{prompt}{buf}")
            } else {
                prompt.to_string()
            }
        };
        display.write_echo(term, &render(&buf))?;
        loop {
            let Some(key) = term.get_key() else {
                display.write_echo(term, "")?;
                return Ok(String::new());
            };
            let key = if matches!(key, Key::Unknown(0)) {
                display.write_echo(term, "")?;
                return Ok(String::new());
            } else {
                key
            };
            match key {
                Key::Enter => {
                    display.write_echo(term, "")?;
                    return Ok(buf);
                }
                Key::Control('G') | Key::Escape => {
                    display.write_echo(term, "")?;
                    return Ok(String::new());
                }
                Key::Backspace | Key::Delete => {
                    buf.pop();
                    display.write_echo(term, &render(&buf))?;
                }
                Key::Tab if echo => {
                    let continuing = matches!(&cycle, Some((cands, idx)) if buf == cands[*idx]);
                    if let Some((cands, idx)) = cycle.take().filter(|_| continuing) {
                        let next = (idx + 1) % cands.len();
                        buf.clone_from(&cands[next]);
                        display.write_echo(term, &render(&buf))?;
                        cycle = Some((cands, next));
                    } else {
                        cycle = None;
                        let completions = match completion {
                            Completion::Filename => crate::util::complete_filename(&buf),
                            Completion::Buffer => self.complete_buffer_name(&buf),
                        };
                        if completions.is_empty() {
                            term.beep();
                        } else if completions.len() == 1 {
                            buf.clone_from(&completions[0]);
                            display.write_echo(term, &render(&buf))?;
                        } else {
                            let common = find_common_prefix(&completions);
                            if common.len() > buf.len() {
                                buf = common;
                                display.write_echo(term, &render(&buf))?;
                            } else {
                                buf.clone_from(&completions[0]);
                                display.write_echo(term, &render(&buf))?;
                                cycle = Some((completions, 0));
                            }
                        }
                    }
                }
                Key::Char(c) => {
                    buf.push(c);
                    display.write_echo(term, &render(&buf))?;
                }
                _ => {
                    term.beep();
                }
            }
        }
    }
}
