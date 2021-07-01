use super::{
    BackwardChar, CmdFlags, Command, Editor, Error, ForwardChar, Result, WindowFlags, at_eol,
    case_words, char_at_dot_is_word_char, kill_n_chars, skip_non_word_backward,
    skip_non_word_forward, skip_word_backward, skip_word_forward,
};

pub struct ForwardWord;

impl Command for ForwardWord {
    fn execute(&self, editor: &mut Editor, _f: bool, mut n: usize) -> Result<()> {
        while n > 0 {
            let _ = skip_word_forward(editor)?;
            let _ = skip_non_word_forward(editor)?;
            n -= 1;
        }
        Ok(())
    }
}

pub struct BackwardWord;

impl Command for BackwardWord {
    fn execute(&self, editor: &mut Editor, _f: bool, mut n: usize) -> Result<()> {
        BackwardChar.execute(editor, false, 1)?;
        while n > 0 {
            let _ = skip_non_word_backward(editor)?;
            let _ = skip_word_backward(editor)?;
            n -= 1;
        }
        ForwardChar.execute(editor, false, 1)
    }
}

pub struct UpperWord;

impl Command for UpperWord {
    fn execute(&self, editor: &mut Editor, _f: bool, n: usize) -> Result<()> {
        case_words(editor, n, |_, b| {
            b.is_ascii_lowercase().then(|| b.to_ascii_uppercase())
        })
    }
}

pub struct LowerWord;

impl Command for LowerWord {
    fn execute(&self, editor: &mut Editor, _f: bool, n: usize) -> Result<()> {
        case_words(editor, n, |_, b| {
            b.is_ascii_uppercase().then(|| b.to_ascii_lowercase())
        })
    }
}

pub struct CapWord;

impl Command for CapWord {
    fn execute(&self, editor: &mut Editor, _f: bool, n: usize) -> Result<()> {
        case_words(editor, n, |idx, b| match idx {
            0 => b.is_ascii_lowercase().then(|| b.to_ascii_uppercase()),
            _ => b.is_ascii_uppercase().then(|| b.to_ascii_lowercase()),
        })
    }
}

pub struct DeleteForwardWord;

impl Command for DeleteForwardWord {
    fn execute(&self, editor: &mut Editor, _f: bool, n: usize) -> Result<()> {
        if !editor.last_flag.intersects(CmdFlags::KILL) {
            editor.kdelete();
        }
        editor.this_flag |= CmdFlags::KILL;

        let (saved_line, saved_offset) = {
            let win = editor.cur_window()?;
            (win.dot_line, win.dot_offset)
        };

        let mut count = 0usize;
        let wn = if n == 0 { 1 } else { n };

        count += skip_non_word_forward(editor)?;

        for wi in 0..wn {
            while at_eol(editor)? {
                ForwardChar.execute(editor, false, 1)?;
                count += 1;
            }
            count += skip_word_forward(editor)?;
            if wi + 1 < wn {
                count += skip_non_word_forward(editor)?;
            }
        }

        if n != 0 {
            loop {
                let is_ws = {
                    let win = editor.cur_window()?;
                    let buf = editor.buffers.get(win.buffer_id).ok_or(Error::Abort)?;
                    let offset = win.dot_offset.0;
                    let line = buf.line(win.dot_line).ok_or(Error::Abort)?;
                    if offset >= line.len() {
                        true
                    } else {
                        let b = line.text[offset];
                        b == b' ' || b == b'\t'
                    }
                };
                if !is_ws {
                    break;
                }
                if ForwardChar.execute(editor, false, 1).is_err() {
                    break;
                }
                count += 1;
            }
        }

        {
            let win = editor.cur_window_mut()?;
            win.dot_line = saved_line;
            win.dot_offset = saved_offset;
        }

        kill_n_chars(editor, count)?;

        editor
            .current_window_mut()
            .ok_or(Error::Abort)?
            .set_flag(WindowFlags::EDITED);
        Ok(())
    }
}

pub struct DeleteBackwardWord;

impl Command for DeleteBackwardWord {
    fn execute(&self, editor: &mut Editor, _f: bool, n: usize) -> Result<()> {
        if n == 0 {
            return Err(Error::Abort);
        }
        if !editor.last_flag.intersects(CmdFlags::KILL) {
            editor.kdelete();
        }
        editor.this_flag |= CmdFlags::KILL;

        if BackwardChar.execute(editor, false, 1).is_err() {
            return Err(Error::Abort);
        }
        let mut count = 0usize;
        let mut hit_bob = false;

        for _ in 0..n {
            loop {
                if char_at_dot_is_word_char(editor)? {
                    break;
                }
                if BackwardChar.execute(editor, false, 1).is_err() {
                    break;
                }
                count += 1;
            }
            loop {
                if !char_at_dot_is_word_char(editor)? {
                    break;
                }
                count += 1;
                if BackwardChar.execute(editor, false, 1).is_err() {
                    hit_bob = true;
                    break;
                }
            }
            if hit_bob {
                break;
            }
        }

        if !hit_bob {
            let _ = ForwardChar.execute(editor, false, 1);
        }

        kill_n_chars(editor, count)?;

        editor
            .current_window_mut()
            .ok_or(Error::Abort)?
            .set_flag(WindowFlags::EDITED);
        Ok(())
    }
}
