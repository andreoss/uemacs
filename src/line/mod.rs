use crate::core::LineId;

#[derive(Default)]
pub struct Line {
    pub next: LineId,
    pub prev: LineId,
    pub text: Vec<u8>,
}

impl Line {
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(test)]
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            text: Vec::with_capacity(cap),
            ..Self::default()
        }
    }

    pub const fn len(&self) -> usize {
        self.text.len()
    }

    #[cfg(test)]
    pub const fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    #[cfg(test)]
    pub const fn capacity(&self) -> usize {
        self.text.capacity()
    }

    #[cfg(test)]
    pub fn get_byte(&self, offset: usize) -> Option<u8> {
        self.text.get(offset).copied()
    }

    pub fn put_byte(&mut self, offset: usize, byte: u8) {
        if let Some(b) = self.text.get_mut(offset) {
            *b = byte;
        }
    }

    pub fn insert_bytes(&mut self, offset: usize, bytes: &[u8]) -> Result<(), crate::core::Error> {
        if offset > self.text.len() {
            return Err(crate::core::Error::Abort);
        }
        self.text.splice(offset..offset, bytes.iter().copied());
        Ok(())
    }

    pub fn delete_bytes(&mut self, offset: usize, len: usize) {
        let start = offset.min(self.text.len());
        let end = start.saturating_add(len).min(self.text.len());
        self.text.drain(start..end);
    }

    pub fn split_off(&mut self, offset: usize) -> Self {
        Self {
            text: self.text.split_off(offset),
            ..Self::default()
        }
    }

    #[cfg(test)]
    pub fn append_line(&mut self, other: &mut Self) {
        self.text.append(&mut other.text);
    }

    pub const fn next(&self) -> LineId {
        self.next
    }

    pub const fn prev(&self) -> LineId {
        self.prev
    }

    pub const fn set_next(&mut self, id: LineId) {
        self.next = id;
    }

    pub const fn set_prev(&mut self, id: LineId) {
        self.prev = id;
    }

    #[cfg(test)]
    pub const fn is_linked(&self) -> bool {
        self.next.0 != 0 || self.prev.0 != 0
    }

    pub const fn unlink(&mut self) {
        self.next = LineId(0);
        self.prev = LineId(0);
    }
}

#[cfg(test)]
mod tests;
