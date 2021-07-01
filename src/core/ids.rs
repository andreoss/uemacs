use super::CommandId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LineId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct BufferId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct WindowId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LineOffset(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacroKey {
    pub cmd: CommandId,
    pub f: bool,
    pub n: usize,
}

impl MacroKey {
    pub const fn new(cmd: CommandId, f: bool, n: usize) -> Self {
        Self { cmd, f, n }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyCode(pub u32);
