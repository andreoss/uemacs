use super::{Buffer, BufferId, Buffers};

impl Buffers {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create(&mut self, name: &str) -> &mut Buffer {
        let id = BufferId(self.next_id);
        self.next_id += 1;
        self.buffers.push(Buffer::new(id, name));
        self.buffers.last_mut().expect("just pushed a buffer")
    }

    pub fn find(&self, name: &str) -> Option<&Buffer> {
        self.buffers.iter().find(|b| b.name == name)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn find_mut(&mut self, name: &str) -> Option<&mut Buffer> {
        self.buffers.iter_mut().find(|b| b.name == name)
    }

    pub fn find_or_create(&mut self, name: &str) -> &mut Buffer {
        if let Some(idx) = self.buffers.iter().position(|b| b.name == name) {
            &mut self.buffers[idx]
        } else {
            self.create(name)
        }
    }

    pub fn get(&self, id: BufferId) -> Option<&Buffer> {
        self.buffers.iter().find(|b| b.id == id)
    }

    pub fn get_mut(&mut self, id: BufferId) -> Option<&mut Buffer> {
        self.buffers.iter_mut().find(|b| b.id == id)
    }

    pub fn remove(&mut self, id: BufferId) -> Option<Buffer> {
        let idx = self.buffers.iter().position(|b| b.id == id)?;
        Some(self.buffers.remove(idx))
    }

    pub fn iter(&self) -> impl Iterator<Item = &Buffer> {
        self.buffers.iter()
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Buffer> {
        self.buffers.iter_mut()
    }

    pub const fn len(&self) -> usize {
        self.buffers.len()
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub const fn is_empty(&self) -> bool {
        self.buffers.is_empty()
    }
}
