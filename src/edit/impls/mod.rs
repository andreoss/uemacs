pub(super) use super::Command;
pub(super) use super::Editor;
pub(super) use super::region_bounds;
pub(super) use crate::core::{
    BufferFlags, BufferId, CmdFlags, Error, LineId, LineOffset, Mode, Result, UndoAction,
    UndoEntry, WindowFlags, WindowId,
};

pub(super) use crate::display::Display;
pub(super) use crate::terminal::{Key, TerminalBackend};

mod editor_io;
pub use editor_io::*;
mod cmode;
mod helpers_para;
pub use cmode::*;
pub use helpers_para::*;
mod motion_char;
pub use motion_char::*;
mod motion_line;
pub use motion_line::*;
mod motion_para;
pub use motion_para::*;
mod mark;
pub use mark::*;
mod open_insert;
pub use open_insert::*;
mod char_delete;
pub use char_delete::*;
mod kill_line;
pub use kill_line::*;
mod region;
pub use region::*;
mod indent;
pub use indent::*;
mod detab;
pub use detab::*;
mod blank;
pub use blank::*;
mod page;
pub use page::*;
mod insert_char;
pub use insert_char::*;
mod screen;
pub use screen::*;
mod fillpara;
pub use fillpara::*;
mod justify;
pub use justify::*;
mod para_kill;
pub use para_kill::*;
mod yank;
pub use yank::*;
mod buffers;
pub use buffers::*;
mod window;
pub use window::*;
mod window_scroll;
pub use window_scroll::*;
mod search;
pub use search::*;
mod word;
pub use word::*;
mod helpers_char;
pub use helpers_char::*;
mod helpers_word;
pub use helpers_word::*;
mod helpers_region;
pub use helpers_region::*;
mod helpers_misc;
pub use helpers_misc::*;

#[cfg(test)]
mod inline_tests;
