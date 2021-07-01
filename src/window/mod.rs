use crate::core::{BufferId, LineId, LineOffset, WindowFlags, WindowId};

pub struct Window {
    pub id: WindowId,
    pub buffer_id: BufferId,
    pub top_line: LineId,
    pub dot_line: LineId,
    pub dot_offset: LineOffset,
    pub mark_line: Option<LineId>,
    pub mark_offset: LineOffset,
    pub top_row: usize,
    pub n_rows: usize,
    pub force: i8,
    pub flags: WindowFlags,
}

impl Window {
    pub const fn new(id: WindowId, buffer_id: BufferId, top_line: LineId) -> Self {
        Self {
            id,
            buffer_id,
            top_line,
            dot_line: top_line,
            dot_offset: LineOffset(0),
            mark_line: None,
            mark_offset: LineOffset(0),
            top_row: 0,
            n_rows: 0,
            force: 0,
            flags: WindowFlags::EMPTY,
        }
    }

    pub const fn dot(&self) -> (LineId, LineOffset) {
        (self.dot_line, self.dot_offset)
    }

    pub const fn set_dot(&mut self, line: LineId, offset: LineOffset) {
        self.dot_line = line;
        self.dot_offset = offset;
    }

    pub fn mark(&self) -> Option<(LineId, LineOffset)> {
        self.mark_line.map(|l| (l, self.mark_offset))
    }

    pub const fn set_mark(&mut self, line: LineId, offset: LineOffset) {
        self.mark_line = Some(line);
        self.mark_offset = offset;
    }

    pub const fn clear_mark(&mut self) {
        self.mark_line = None;
        self.mark_offset = LineOffset(0);
    }

    pub fn set_flag(&mut self, flag: WindowFlags) {
        self.flags |= flag;
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn clear_flag(&mut self, flag: WindowFlags) {
        self.flags &= !flag;
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub const fn has_flag(&self, flag: WindowFlags) -> bool {
        self.flags.intersects(flag)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub const fn clear_flags(&mut self) {
        self.flags = WindowFlags::EMPTY;
    }
}

#[derive(Default)]
pub struct Windows {
    items: Vec<Window>,
    next_id: usize,
    current: Option<WindowId>,
}

impl Windows {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create(&mut self, buffer_id: BufferId, top_line: LineId) -> &mut Window {
        let id = WindowId(self.next_id);
        self.next_id += 1;
        self.items.push(Window::new(id, buffer_id, top_line));
        self.current.get_or_insert(id);
        self.items.last_mut().expect("just pushed a window")
    }

    pub fn current(&self) -> Option<&Window> {
        self.current.and_then(|id| self.get(id))
    }

    pub fn current_mut(&mut self) -> Option<&mut Window> {
        self.current.and_then(|id| self.get_mut(id))
    }

    pub fn set_current(&mut self, id: WindowId) -> bool {
        if self.get(id).is_some() {
            self.current = Some(id);
            true
        } else {
            false
        }
    }

    pub fn get(&self, id: WindowId) -> Option<&Window> {
        self.items.iter().find(|w| w.id == id)
    }

    pub fn get_mut(&mut self, id: WindowId) -> Option<&mut Window> {
        self.items.iter_mut().find(|w| w.id == id)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn find_by_buffer(&self, buffer_id: BufferId) -> Option<&Window> {
        self.items.iter().find(|w| w.buffer_id == buffer_id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Window> {
        self.items.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Window> {
        self.items.iter_mut()
    }

    pub fn remove(&mut self, id: WindowId) -> Option<Window> {
        let idx = self.items.iter().position(|w| w.id == id)?;
        let removed = self.items.remove(idx);
        if self.current == Some(id) {
            self.current = self.items.first().map(|w| w.id);
        }
        Some(removed)
    }

    pub const fn len(&self) -> usize {
        self.items.len()
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub const fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(test)]
mod tests;
