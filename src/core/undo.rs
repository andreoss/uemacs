use super::{BufferId, LineId, LineOffset};

#[derive(Debug, Clone)]
pub enum UndoAction {
    Insert {
        line: LineId,
        offset: usize,
        data: Vec<u8>,
    },
    Delete {
        line: LineId,
        offset: usize,
        data: Vec<u8>,
    },
    Split {
        line: LineId,
        offset: usize,
        new_line: LineId,
    },
    Merge {
        line: LineId,
        offset: usize,
        next_line: LineId,
        next_data: Vec<u8>,
        after_next: LineId,
    },
}

#[derive(Debug, Clone)]
pub struct UndoEntry {
    pub buffer_id: BufferId,
    pub dot_line: LineId,
    pub dot_offset: LineOffset,
    pub actions: Vec<UndoAction>,
}
