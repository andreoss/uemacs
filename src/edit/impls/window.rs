use super::{Command, Editor, Error, Result, WindowFlags, WindowId, move_window_impl};

pub struct DeleteWindow;

impl Command for DeleteWindow {
    fn execute(&self, editor: &mut Editor, _f: bool, _n: usize) -> Result<()> {
        if editor.windows.len() <= 1 {
            return Err(Error::Abort);
        }
        let (cur_id, cur_top, cur_rows) = {
            let w = editor.cur_window()?;
            (w.id, w.top_row, w.n_rows)
        };
        let space = cur_rows + 1;
        let neighbor = if cur_top == 0 {
            editor
                .windows
                .iter()
                .find(|w| w.id != cur_id && w.top_row == space)
                .map(|w| w.id)
        } else {
            editor
                .windows
                .iter()
                .find(|w| w.id != cur_id && w.top_row + w.n_rows == cur_top - 1)
                .map(|w| w.id)
        };
        let recv_id = neighbor.or_else(|| {
            let ids: Vec<WindowId> = editor.windows.iter().map(|w| w.id).collect();
            let pos = ids.iter().position(|&id| id == cur_id)?;
            Some(if pos > 0 { ids[pos - 1] } else { ids[pos + 1] })
        });
        let recv_id = recv_id.ok_or(Error::Abort)?;
        {
            let recv = editor.windows.get_mut(recv_id).ok_or(Error::Abort)?;
            if neighbor == Some(recv_id) {
                if cur_top == 0 {
                    recv.top_row = 0;
                }
                recv.n_rows += space;
            }
            recv.set_flag(WindowFlags::HARD);
        }
        editor.windows.remove(cur_id);
        editor.windows.set_current(recv_id);
        for w in editor.windows.iter_mut() {
            w.set_flag(WindowFlags::MODE_LINE);
        }
        Ok(())
    }
}

pub struct OneWindow;

impl Command for OneWindow {
    fn execute(&self, editor: &mut Editor, _f: bool, _n: usize) -> Result<()> {
        let current_id = editor.cur_window()?.id;
        let other_ids: Vec<WindowId> = editor
            .windows
            .iter()
            .filter(|w| w.id != current_id)
            .map(|w| w.id)
            .collect();
        let total_rows: usize = editor.windows.iter().map(|w| w.n_rows + 1).sum();
        for id in other_ids {
            editor.windows.remove(id);
        }
        if total_rows > 0 {
            let win = editor.cur_window_mut()?;
            win.n_rows = total_rows.saturating_sub(1);
            win.top_row = 0;
            win.set_flag(WindowFlags::HARD);
        }
        Ok(())
    }
}

pub struct SplitWindowDown;

impl Command for SplitWindowDown {
    fn execute(&self, editor: &mut Editor, _f: bool, _n: usize) -> Result<()> {
        let (buffer_id, top_line, dot_line, dot_offset, top_row, n_rows, bid) = {
            let win = editor.cur_window()?;
            (
                win.buffer_id,
                win.top_line,
                win.dot_line,
                win.dot_offset,
                win.top_row,
                win.n_rows,
                win.id,
            )
        };
        if n_rows < 3 {
            return Err(Error::Abort);
        }
        let upper = (n_rows - 1) / 2;
        let lower = (n_rows - 1) - upper;
        let new_top_row = top_row + upper + 1;
        let half = upper;
        let new_n_rows = lower;
        {
            let win = editor.cur_window_mut()?;
            win.n_rows = half;
            win.flags |= WindowFlags::HARD | WindowFlags::MODE_LINE;
        }
        let new_id = editor.create_window(buffer_id, top_line);
        {
            let new_win = editor.windows.get_mut(new_id).ok_or(Error::Abort)?;
            new_win.top_row = new_top_row;
            new_win.n_rows = new_n_rows;
            new_win.dot_line = dot_line;
            new_win.dot_offset = dot_offset;
            new_win.top_line = top_line;
            new_win.flags |= WindowFlags::HARD | WindowFlags::MODE_LINE;
        }
        editor.windows.set_current(bid);
        Ok(())
    }
}

pub struct OtherWindow;

impl Command for OtherWindow {
    fn execute(&self, editor: &mut Editor, _f: bool, _n: usize) -> Result<()> {
        let current_id = editor.cur_window()?.id;
        let ids: Vec<WindowId> = editor.windows.iter().map(|w| w.id).collect();
        if ids.len() <= 1 {
            return Ok(());
        }
        let pos = ids
            .iter()
            .position(|&id| id == current_id)
            .ok_or(Error::Abort)?;
        let next_id = ids[(pos + 1) % ids.len()];
        editor.windows.set_current(next_id);
        if let Some(win) = editor.current_window_mut() {
            win.set_flag(WindowFlags::MOVED);
        }
        Ok(())
    }
}

pub struct NextWindow;

impl Command for NextWindow {
    fn execute(&self, editor: &mut Editor, f: bool, n: usize) -> Result<()> {
        OtherWindow.execute(editor, f, n)
    }
}

pub struct PreviousWindow;

impl Command for PreviousWindow {
    fn execute(&self, editor: &mut Editor, _f: bool, _n: usize) -> Result<()> {
        let current_id = editor.cur_window()?.id;
        let ids: Vec<WindowId> = editor.windows.iter().map(|w| w.id).collect();
        if ids.len() <= 1 {
            return Ok(());
        }
        let pos = ids
            .iter()
            .position(|&id| id == current_id)
            .ok_or(Error::Abort)?;
        let prev_id = ids[(pos + ids.len() - 1) % ids.len()];
        editor.windows.set_current(prev_id);
        if let Some(win) = editor.current_window_mut() {
            win.set_flag(WindowFlags::MOVED);
        }
        Ok(())
    }
}

pub struct MoveWindowDown;

impl Command for MoveWindowDown {
    fn execute(&self, editor: &mut Editor, _f: bool, n: usize) -> Result<()> {
        move_window_impl(editor, n, false)
    }
}

pub struct MoveWindowUp;

impl Command for MoveWindowUp {
    fn execute(&self, editor: &mut Editor, _f: bool, n: usize) -> Result<()> {
        move_window_impl(editor, n, true)
    }
}
