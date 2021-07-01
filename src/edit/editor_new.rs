use super::{
    BufferId, Buffers, CmdFlags, Editor, Error, HashMap, LineId, LockManager, Mode, Result,
    WindowId, Windows,
};

impl Editor {
    pub fn new() -> Self {
        Self {
            buffers: Buffers::new(),
            windows: Windows::new(),
            this_flag: CmdFlags::EMPTY,
            last_flag: CmdFlags::EMPTY,
            cur_goal: 0,
            kill_buffer: Vec::new(),
            search_pattern: Vec::new(),
            replace_pattern: Vec::new(),
            last_match: Vec::new(),
            sgarbf_requested: false,
            suspend_requested: false,
            quit_requested: false,
            undo_stack: Vec::new(),
            recording_macro: false,
            macro_keys: Vec::new(),
            tab_width: 8,
            tabsize: 0,
            scroll_amount: 20,
            fillcol: 78,
            gmode: Mode::EMPTY,
            saved_window: None,
            stored_macros: Default::default(),
            gacount: 256,
            gasave: 256,
            lock_manager: LockManager::new(),
            macro_store_buffer: None,
            user_vars: HashMap::new(),
            screen_rows: 24,
            screen_cols: 80,
        }
    }

    pub const fn swap_flags(&mut self) {
        self.last_flag = self.this_flag;
        self.this_flag = CmdFlags::EMPTY;
    }

    pub fn kdelete(&mut self) {
        self.kill_buffer.clear();
    }

    pub fn create_buffer(&mut self, name: &str) -> BufferId {
        let gmode = self.gmode;
        let buf = self.buffers.create(name);
        buf.mode = gmode;
        buf.id
    }

    pub fn find_or_create_buffer(&mut self, name: &str) -> BufferId {
        if let Some(existing) = self.buffers.find(name) {
            return existing.id;
        }
        self.create_buffer(name)
    }

    pub fn create_window(&mut self, buffer_id: BufferId, top_line: LineId) -> WindowId {
        let win = self.windows.create(buffer_id, top_line);
        win.id
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn current_buffer(&self) -> Option<&crate::buffer::Buffer> {
        let window = self.windows.current()?;
        self.buffers.get(window.buffer_id)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn current_buffer_mut(&mut self) -> Option<&mut crate::buffer::Buffer> {
        let buffer_id = self.windows.current()?.buffer_id;
        self.buffers.get_mut(buffer_id)
    }

    pub fn current_window(&self) -> Option<&crate::window::Window> {
        self.windows.current()
    }

    pub fn current_window_mut(&mut self) -> Option<&mut crate::window::Window> {
        self.windows.current_mut()
    }

    pub fn cur_window(&self) -> Result<&crate::window::Window> {
        self.windows.current().ok_or(Error::Abort)
    }

    pub fn cur_window_mut(&mut self) -> Result<&mut crate::window::Window> {
        self.windows.current_mut().ok_or(Error::Abort)
    }

    #[allow(dead_code)]
    pub fn cur_buffer_mut(&mut self) -> Result<&mut crate::buffer::Buffer> {
        let buf_id = self.cur_window()?.buffer_id;
        self.buffers.get_mut(buf_id).ok_or(Error::Abort)
    }

    pub fn set_var(&mut self, name: &str, value: usize) {
        match name {
            "tab" => self.tab_width = value,
            "scroll" => self.scroll_amount = value,
            "fillcol" => self.fillcol = value,
            _ => {}
        }
    }
}
