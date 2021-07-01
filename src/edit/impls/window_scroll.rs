use super::{BackwardPage, Command, Editor, Error, ForwardPage, Result, WindowFlags, WindowId};

pub struct ScrollNextUp;

impl Command for ScrollNextUp {
    fn execute(&self, editor: &mut Editor, f: bool, n: usize) -> Result<()> {
        let ids: Vec<WindowId> = editor.windows.iter().map(|w| w.id).collect();
        if ids.len() <= 1 {
            return Ok(());
        }
        let current_id = editor.cur_window()?.id;
        let pos = ids
            .iter()
            .position(|&id| id == current_id)
            .ok_or(Error::Abort)?;
        let next_id = ids[(pos + 1) % ids.len()];
        editor.windows.set_current(next_id);
        BackwardPage.execute(editor, f, n)?;
        editor.windows.set_current(current_id);
        Ok(())
    }
}

pub struct ScrollNextDown;

impl Command for ScrollNextDown {
    fn execute(&self, editor: &mut Editor, f: bool, n: usize) -> Result<()> {
        let ids: Vec<WindowId> = editor.windows.iter().map(|w| w.id).collect();
        if ids.len() <= 1 {
            return Ok(());
        }
        let current_id = editor.cur_window()?.id;
        let pos = ids
            .iter()
            .position(|&id| id == current_id)
            .ok_or(Error::Abort)?;
        let next_id = ids[(pos + 1) % ids.len()];
        editor.windows.set_current(next_id);
        ForwardPage.execute(editor, f, n)?;
        editor.windows.set_current(current_id);
        Ok(())
    }
}

pub struct GrowWindow;

impl Command for GrowWindow {
    fn execute(&self, editor: &mut Editor, _f: bool, n: usize) -> Result<()> {
        let (_current_id, pos, ids) = {
            let current = editor.cur_window()?;
            let ids: Vec<WindowId> = editor.windows.iter().map(|w| w.id).collect();
            let pos = ids
                .iter()
                .position(|&id| id == current.id)
                .ok_or(Error::Abort)?;
            (current.id, pos, ids)
        };
        if ids.len() <= 1 {
            return Err(Error::Abort);
        }
        let adj_pos = if pos + 1 < ids.len() {
            pos + 1
        } else {
            pos - 1
        };
        let adj_id = ids[adj_pos];
        let n = n.max(1);
        let adj_n_rows = editor
            .windows
            .get(adj_id)
            .map(|w| w.n_rows)
            .ok_or(Error::Abort)?;
        if adj_n_rows <= n {
            return Err(Error::Abort);
        }
        {
            let adj = editor.windows.get_mut(adj_id).ok_or(Error::Abort)?;
            adj.n_rows -= n;
            if adj_pos > pos {
                adj.top_row += n;
            }
        }
        {
            let cur = editor.cur_window_mut()?;
            cur.n_rows += n;
            if adj_pos < pos {
                cur.top_row = cur.top_row.saturating_sub(n);
            }
            cur.set_flag(WindowFlags::HARD);
        }
        Ok(())
    }
}

pub struct ShrinkWindow;

impl Command for ShrinkWindow {
    fn execute(&self, editor: &mut Editor, _f: bool, n: usize) -> Result<()> {
        let (_current_id, pos, ids) = {
            let current = editor.cur_window()?;
            let ids: Vec<WindowId> = editor.windows.iter().map(|w| w.id).collect();
            let pos = ids
                .iter()
                .position(|&id| id == current.id)
                .ok_or(Error::Abort)?;
            (current.id, pos, ids)
        };
        if ids.len() <= 1 {
            return Err(Error::Abort);
        }
        let adj_pos = if pos + 1 < ids.len() {
            pos + 1
        } else {
            pos - 1
        };
        let adj_id = ids[adj_pos];
        let n = n.max(1);
        let cur_n_rows = editor
            .current_window()
            .map(|w| w.n_rows)
            .ok_or(Error::Abort)?;
        if cur_n_rows <= n {
            return Err(Error::Abort);
        }
        {
            let adj = editor.windows.get_mut(adj_id).ok_or(Error::Abort)?;
            adj.n_rows += n;
            if adj_pos > pos {
                adj.top_row = adj.top_row.saturating_sub(n);
            }
        }
        {
            let cur = editor.cur_window_mut()?;
            cur.n_rows -= n;
            if adj_pos < pos {
                cur.top_row += n;
            }
            cur.set_flag(WindowFlags::HARD);
        }
        Ok(())
    }
}

pub struct ResizeWindow;

impl Command for ResizeWindow {
    fn execute(&self, editor: &mut Editor, f: bool, n: usize) -> Result<()> {
        if !f {
            return Ok(());
        }
        let current_n_rows = editor
            .current_window()
            .map(|w| w.n_rows)
            .ok_or(Error::Abort)?;
        if current_n_rows == n {
            return Ok(());
        }
        let delta = n.abs_diff(current_n_rows);
        if n > current_n_rows {
            GrowWindow.execute(editor, true, delta)
        } else {
            ShrinkWindow.execute(editor, true, delta)
        }
    }
}
