pub use crate::core::{BufferFlags, BufferId, LineId, LineOffset, Mode};
pub use crate::line::Line;

pub struct Buffer {
    pub id: BufferId,
    pub name: String,
    pub filename: String,
    pub lines: Vec<Line>,
    pub head: LineId,
    pub dot_line: LineId,
    pub dot_offset: LineOffset,
    pub mark_line: Option<LineId>,
    pub mark_offset: LineOffset,
    pub mode: Mode,
    pub flags: BufferFlags,
}

#[derive(Default)]
pub struct Buffers {
    buffers: Vec<Buffer>,
    next_id: usize,
}

mod buffers_ops;
mod ops;

#[cfg(test)]
mod tests;
