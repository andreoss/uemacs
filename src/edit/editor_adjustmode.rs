use super::{Display, Editor, Mode, Result, TerminalBackend, WindowFlags};

static MODE_NAMES: &[&str] = &[
    "WRAP", "CMODE", "SPELL", "EXACT", "VIEW", "OVER", "MAGIC", "ASAVE",
];

const MODES: [Mode; 8] = [
    Mode::WRAP,
    Mode::C_MODE,
    Mode::SPELL,
    Mode::EXACT,
    Mode::VIEW,
    Mode::OVERWRITE,
    Mode::MAGIC,
    Mode::AUTO_SAVE,
];

impl Editor {
    pub(super) fn adjustmode<T: TerminalBackend>(
        &mut self,
        kind: bool,
        global: bool,
        term: &mut T,
        display: &mut Display,
    ) -> Result<()> {
        let prompt = adjustmode_prompt(kind, global);
        let input = self.minibuffer_readline(term, display, prompt)?;
        if input.is_empty() {
            return Ok(());
        }
        let upper = input.to_uppercase();
        if let Some(i) = MODE_NAMES.iter().position(|&m| m == upper.as_str()) {
            apply_mode_flag(self, i, kind, global);
        } else {
            term.beep();
        }
        Ok(())
    }
}

const fn adjustmode_prompt(kind: bool, global: bool) -> &'static str {
    match (kind, global) {
        (true, true) => "Global mode to add: ",
        (true, false) => "Mode to add: ",
        (false, true) => "Global mode to delete: ",
        (false, false) => "Mode to delete: ",
    }
}

fn apply_mode_flag(editor: &mut Editor, i: usize, kind: bool, global: bool) {
    if global {
        if kind {
            editor.gmode |= MODES[i];
        } else {
            editor.gmode &= !MODES[i];
        }
    } else {
        let buf_id = match editor.cur_window() {
            Ok(w) => w.buffer_id,
            Err(_) => return,
        };
        let Some(buf) = editor.buffers.get_mut(buf_id) else {
            return;
        };
        if kind {
            buf.mode |= MODES[i];
        } else {
            buf.mode &= !MODES[i];
        }
        for win in editor.windows.iter_mut() {
            if win.buffer_id == buf_id {
                win.set_flag(WindowFlags::MODE_LINE);
            }
        }
    }
}
