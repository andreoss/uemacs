use super::{Buffer, BufferFlags, BufferId, Line, LineId, LineOffset, Mode};

fn make_head() -> Line {
    let mut head = Line::new();
    let head_id = LineId(0);
    head.set_next(head_id);
    head.set_prev(head_id);
    head
}

impl Buffer {
    pub fn new(id: BufferId, name: &str) -> Self {
        let lines = vec![make_head()];
        Self {
            id,
            name: name.to_string(),
            filename: String::new(),
            lines,
            head: LineId(0),
            dot_line: LineId(0),
            dot_offset: LineOffset(0),
            mark_line: None,
            mark_offset: LineOffset(0),
            mode: Mode::EMPTY,
            flags: BufferFlags::EMPTY,
        }
    }

    pub fn line(&self, id: LineId) -> Option<&Line> {
        self.lines.get(id.0)
    }

    pub fn line_mut(&mut self, id: LineId) -> Option<&mut Line> {
        self.lines.get_mut(id.0)
    }

    pub fn head_line(&self) -> &Line {
        &self.lines[self.head.0]
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn head_line_mut(&mut self) -> &mut Line {
        &mut self.lines[self.head.0]
    }

    pub fn is_empty(&self) -> bool {
        let head = self.head_line();
        head.next() == self.head
    }

    #[cfg_attr(not(test), allow(dead_code))]
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

    pub fn insert_after(&mut self, after: LineId, mut line: Line) -> LineId {
        let new_id = LineId(self.lines.len());
        let next_id = self.lines[after.0].next();

        line.set_prev(after);
        line.set_next(next_id);

        self.lines[after.0].set_next(new_id);
        self.lines[next_id.0].set_prev(new_id);

        self.lines.push(line);
        new_id
    }

    pub fn remove(&mut self, id: LineId) -> Option<Line> {
        if id.0 >= self.lines.len() || id == self.head {
            return None;
        }

        let prev = self.lines[id.0].prev();
        let next = self.lines[id.0].next();

        self.lines[prev.0].set_next(next);
        self.lines[next.0].set_prev(prev);

        let mut removed = std::mem::take(&mut self.lines[id.0]);
        removed.unlink();
        Some(removed)
    }

    pub fn clear(&mut self) {
        self.lines = vec![Line::default()];
        self.head = LineId(0);
        self.dot_line = LineId(0);
        self.dot_offset = LineOffset(0);
        self.mark_line = None;
        self.mark_offset = LineOffset(0);
        self.filename.clear();
        self.flags &= !BufferFlags::CHANGED;
    }

    pub fn line_count(&self) -> usize {
        let mut count = 0;
        let mut curr = self.lines[self.head.0].next();
        while curr != self.head {
            count += 1;
            curr = self.lines[curr.0].next();
        }
        count
    }

    pub fn nth_line(&self, n: usize) -> Option<LineId> {
        let mut curr = self.lines[self.head.0].next();
        for _ in 0..n {
            if curr == self.head {
                return None;
            }
            curr = self.lines[curr.0].next();
        }
        if curr == self.head { None } else { Some(curr) }
    }

    pub fn line_iter(&self) -> impl Iterator<Item = &Line> {
        let head = self.head;
        let mut curr = self.lines[self.head.0].next();
        std::iter::from_fn(move || {
            if curr == head {
                None
            } else {
                let line = &self.lines[curr.0];
                curr = line.next();
                Some(line)
            }
        })
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn for_each_line_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Line),
    {
        let head = self.head;
        let mut curr = self.lines[self.head.0].next();
        while curr != head {
            let next = self.lines[curr.0].next();
            f(&mut self.lines[curr.0]);
            curr = next;
        }
    }

    pub fn prev_line(&self, id: LineId) -> Option<LineId> {
        let prev = self.lines[id.0].prev();
        if prev == self.head { None } else { Some(prev) }
    }

    pub fn next_line(&self, id: LineId) -> Option<LineId> {
        let next = self.lines[id.0].next();
        if next == self.head { None } else { Some(next) }
    }

    pub fn line_len(&self, id: LineId) -> Option<usize> {
        self.lines.get(id.0).map(super::super::line::Line::len)
    }

    pub fn split_line(&mut self, id: LineId, offset: usize) -> LineId {
        let new_line = self.lines[id.0].split_off(offset);
        self.insert_after(id, new_line)
    }
}
