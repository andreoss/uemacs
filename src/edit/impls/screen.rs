use super::{Command, Editor, Error, Mode, Result, WindowFlags};

pub struct KeyboardQuit;

impl Command for KeyboardQuit {
    fn execute(&self, _editor: &mut Editor, _f: bool, _n: usize) -> Result<()> {
        Ok(())
    }
}

pub struct RefreshScreen;

impl Command for RefreshScreen {
    fn execute(&self, editor: &mut Editor, _f: bool, _n: usize) -> Result<()> {
        editor.sgarbf_requested = true;
        Ok(())
    }
}

pub struct SuspendEmacs;

impl Command for SuspendEmacs {
    fn execute(&self, editor: &mut Editor, _f: bool, _n: usize) -> Result<()> {
        editor.suspend_requested = true;
        Ok(())
    }
}

pub struct ToggleMagic;

impl Command for ToggleMagic {
    fn execute(&self, editor: &mut Editor, _f: bool, _n: usize) -> Result<()> {
        let buf_id = editor.cur_window()?.buffer_id;
        let buf = editor.buffers.get_mut(buf_id).ok_or(Error::Abort)?;
        buf.mode ^= Mode::MAGIC;
        let win = editor.cur_window_mut()?;
        win.set_flag(WindowFlags::HARD);
        Ok(())
    }
}

pub struct Nop;

impl Command for Nop {
    fn execute(&self, _editor: &mut Editor, _f: bool, _n: usize) -> Result<()> {
        Ok(())
    }
}
