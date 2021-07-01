pub use crate::bind::{Bindings, command_name};
pub use crate::buffer::{Buffer, Buffers};
pub use crate::core::{
    BufferFlags, BufferId, CONTROL, CmdFlags, CommandId, Error, KeyCode, LineId, LineOffset, META,
    MacroKey, Mode, Result, UndoAction, UndoEntry, WindowFlags, WindowId,
};

pub use crate::display::Display;
pub use crate::lock::LockManager;
pub use crate::terminal::{Key, TerminalBackend};
pub use crate::window::Windows;
pub use std::collections::HashMap;

#[allow(clippy::struct_excessive_bools)]
pub struct Editor {
    pub buffers: Buffers,
    pub windows: Windows,
    pub this_flag: CmdFlags,
    pub last_flag: CmdFlags,
    pub cur_goal: usize,
    pub kill_buffer: Vec<u8>,
    pub search_pattern: Vec<u8>,
    pub replace_pattern: Vec<u8>,
    pub last_match: Vec<u8>,
    pub sgarbf_requested: bool,
    pub suspend_requested: bool,
    pub quit_requested: bool,
    pub undo_stack: Vec<UndoEntry>,

    pub recording_macro: bool,
    pub macro_keys: Vec<MacroKey>,
    pub tab_width: usize,
    pub tabsize: usize,
    pub scroll_amount: usize,
    pub fillcol: usize,
    pub gmode: Mode,
    pub saved_window: Option<WindowId>,
    pub stored_macros: [Option<Vec<MacroKey>>; 9],

    pub gacount: usize,
    pub gasave: usize,
    pub lock_manager: LockManager,
    pub macro_store_buffer: Option<BufferId>,
    pub user_vars: HashMap<String, String>,

    pub screen_rows: usize,
    pub screen_cols: usize,
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

pub trait Command {
    fn execute(&self, editor: &mut Editor, f: bool, n: usize) -> Result<()>;
}

pub mod impls;
pub use impls::*;

#[cfg(test)]
mod tests;

mod editor_adjustmode;
mod editor_describe_bindings;
mod editor_evaluate_env_var;
mod editor_evaluate_expression;
mod editor_execute_lines_inner;
mod editor_insert_file;
mod editor_invoke_function;
mod editor_isearch;
mod editor_new;
mod editor_query_replace;
mod editor_query_replace_apply;
mod editor_quote_char;
mod editor_replace_string;
mod editor_run_command;
mod editor_switch_window_to_buffer;
mod free_ctlx_command;
mod free_macro_number;
mod free_splice_string;
mod free_stol;
pub use crate::kbd::{key_code_display, key_name, key_to_bytes};
pub use editor_describe_bindings::{
    clear_help_buffer_lines, insert_lines_into_buffer, switch_window_to_buffer_first_line,
};
pub use free_ctlx_command::*;
pub use free_macro_number::{qr_match_start, qr_preview, region_bounds};
pub use free_splice_string::*;
pub use free_stol::*;
