use super::{Bindings, Display, Editor, Result, TerminalBackend, stol};

enum DirectiveFlow {
    Return,
    Continue,
    Fallthrough,
}

struct DirectiveState {
    i: usize,
    exec_level: i32,
    while_loops: Vec<(usize, usize)>,
}

impl Editor {
    pub(super) fn run_directive_lines<T: TerminalBackend>(
        &mut self,
        lines: &[String],
        term: &mut T,
        display: &mut Display,
        bindings: &Bindings,
    ) -> Result<()> {
        let mut state = DirectiveState {
            i: 0,
            exec_level: 0,
            while_loops: Vec::new(),
        };
        let mut force = false;
        while state.i < lines.len() {
            let mut line = lines[state.i].trim().to_string();
            state.i += 1;
            if line.is_empty() || line.starts_with(';') {
                continue;
            }
            if line == "!endm" {
                self.macro_store_buffer = None;
                continue;
            }
            if line == "!force" || line.starts_with("!force ") || line.starts_with("!force\t") {
                force = true;
                line = line["!force".len()..].trim_start().to_string();
                if line.is_empty() {
                    continue;
                }
            }
            if line == "!return" {
                if state.exec_level == 0 {
                    return Ok(());
                }
                continue;
            }
            if let Some(rest) = line.strip_prefix('!').map(str::trim) {
                match self.execute_directive(rest, lines, &mut state, term, display)? {
                    DirectiveFlow::Return => return Ok(()),
                    DirectiveFlow::Continue => continue,
                    DirectiveFlow::Fallthrough => {}
                }
            }
            if state.exec_level > 0 {
                continue;
            }
            if line.starts_with('*') {
                continue;
            }
            if let Some(buf_id) = self.macro_store_buffer {
                if let Some(buf) = self.buffers.get_mut(buf_id) {
                    let head = buf.head;
                    let line_id = buf.nth_line(0).unwrap_or(buf.head);
                    let after = if line_id == head { head } else { line_id };
                    let new_line = buf.insert_after(after, crate::line::Line::new());
                    if let Some(l) = buf.line_mut(new_line) {
                        l.text = line.as_bytes().to_vec();
                    }
                }
            }
            let result = self.execute_cmd_str(&line, term, display, bindings);
            if force {
                force = false;
                let _ = result;
            } else {
                result?;
            }
        }
        Ok(())
    }

    fn execute_directive<T: TerminalBackend>(
        &self,
        rest: &str,
        lines: &[String],
        state: &mut DirectiveState,
        term: &mut T,
        display: &mut Display,
    ) -> Result<DirectiveFlow> {
        if rest.starts_with("if ") || rest.starts_with("if\t") {
            let cond = rest[3..].trim();
            if state.exec_level == 0 {
                let val = self.evaluate_expression(cond);
                if !stol(&val) {
                    state.exec_level += 1;
                }
            } else {
                state.exec_level += 1;
            }
            return Ok(DirectiveFlow::Continue);
        }
        if rest == "else" {
            if state.exec_level == 1 {
                state.exec_level = 0;
            } else if state.exec_level == 0 {
                state.exec_level = 1;
            }
            return Ok(DirectiveFlow::Continue);
        }
        if rest == "endif" {
            if state.exec_level > 0 {
                state.exec_level -= 1;
            }
            return Ok(DirectiveFlow::Continue);
        }
        if rest.starts_with("while ") || rest.starts_with("while\t") {
            let cond = rest[6..].trim();
            if state.exec_level == 0 {
                let val = self.evaluate_expression(cond);
                if stol(&val) {
                    state.while_loops.push((state.i - 1, 0));
                } else {
                    state.exec_level += 1;
                }
            } else {
                state.exec_level += 1;
            }
            return Ok(DirectiveFlow::Continue);
        }
        if rest == "endwhile" {
            if state.exec_level > 0 {
                state.exec_level -= 1;
            } else if let Some((start, _)) = state.while_loops.last() {
                let pos = *start;
                state.while_loops.pop();
                state.i = pos;
            }
            return Ok(DirectiveFlow::Continue);
        }
        if rest == "break" {
            if state.exec_level > 0 {
                return Ok(DirectiveFlow::Continue);
            }
            state.while_loops.pop();
            let mut depth = 1;
            while state.i < lines.len() {
                let l = lines[state.i].trim();
                state.i += 1;
                if l.starts_with("!while ") || l.starts_with("!while\t") {
                    depth += 1;
                } else if l == "!endwhile" {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
            }
            return Ok(DirectiveFlow::Continue);
        }
        if rest.starts_with("goto ") || rest.starts_with("goto\t") {
            let label = rest[5..].trim();
            if state.exec_level == 0 {
                let mut found = false;
                for (j, lbl_line) in lines.iter().enumerate() {
                    let lbl = lbl_line.trim();
                    if lbl.len() > 1 && lbl.starts_with('*') && lbl[1..].starts_with(label) {
                        state.i = j + 1;
                        found = true;
                        break;
                    }
                }
                if !found {
                    display.write_echo(term, "(No such label)")?;
                    return Ok(DirectiveFlow::Return);
                }
            }
            return Ok(DirectiveFlow::Continue);
        }
        Ok(DirectiveFlow::Fallthrough)
    }
}
