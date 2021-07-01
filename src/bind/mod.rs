pub use crate::core::{CONTROL, CommandId, KeyCode, META};

pub const fn ctrl(b: u8) -> KeyCode {
    KeyCode(CONTROL | (b as u32 & 0x1f))
}

pub const fn meta(b: u8) -> KeyCode {
    KeyCode(META | (b as u32))
}

use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;

pub struct Bindings {
    map: RefCell<HashMap<KeyCode, CommandId>>,
}

impl Bindings {
    pub fn lookup(&self, code: KeyCode) -> Option<CommandId> {
        self.map.borrow().get(&code).copied()
    }

    pub fn bind(&self, code: KeyCode, cmd: CommandId) {
        self.map.borrow_mut().insert(code, cmd);
    }

    pub fn unbind(&self, code: KeyCode) {
        self.map.borrow_mut().remove(&code);
    }

    pub fn entries(&self) -> Vec<(KeyCode, CommandId)> {
        self.map.borrow().iter().map(|(&k, &v)| (k, v)).collect()
    }

    pub fn command_names_with_prefix(&self, prefix: &str) -> Vec<Cow<'static, str>> {
        let mut names: Vec<Cow<'static, str>> = self
            .entries()
            .into_iter()
            .map(|(_, cmd)| command_name(cmd))
            .filter(|n| *n != "unknown" && n.starts_with(prefix))
            .collect();
        names.sort_unstable();
        names.dedup();
        names
    }
}

mod descriptions;
mod lookup_name;
mod names;
mod table;
pub use descriptions::*;
pub use names::*;

#[cfg(test)]
mod tests;
