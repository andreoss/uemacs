use super::{Key, TerminalBackend};
use crate::core::Result;

#[cfg_attr(not(test), allow(dead_code))]
pub struct MockTerminal {
    pub output: Vec<String>,
    pub input_keys: Vec<Key>,
    pub dimensions: (usize, usize),
    pub reverse: bool,
    pub cursor: (usize, usize),
    pub input_index: usize,
    pub beep_count: usize,
}

impl MockTerminal {
    #[cfg_attr(not(test), allow(dead_code))]
    pub const fn new() -> Self {
        Self {
            output: Vec::new(),
            input_keys: Vec::new(),
            dimensions: (24, 80),
            reverse: false,
            cursor: (0, 0),
            input_index: 0,
            beep_count: 0,
        }
    }
}

impl TerminalBackend for MockTerminal {
    fn open(&mut self) -> Result<()> {
        self.output.push("open".to_string());
        Ok(())
    }

    fn close(&mut self) -> Result<()> {
        self.output.push("close".to_string());
        Ok(())
    }

    fn dimensions(&self) -> (usize, usize) {
        self.dimensions
    }

    fn move_to(&mut self, row: usize, col: usize) {
        self.cursor = (row, col);
        self.output.push(format!("move_to({row},{col})"));
    }

    fn clear_eol(&mut self) {
        self.output.push("clear_eol".to_string());
    }

    fn beep(&mut self) {
        self.beep_count += 1;
        self.output.push("beep".to_string());
    }

    fn set_reverse(&mut self, on: bool) {
        self.reverse = on;
        self.output.push(format!("set_reverse({on})"));
    }

    fn put_char(&mut self, c: char) {
        self.output.push(format!("put_char({c})"));
    }

    fn get_key(&mut self) -> Option<Key> {
        if self.input_index < self.input_keys.len() {
            let key = self.input_keys[self.input_index].clone();
            self.input_index += 1;
            Some(key)
        } else {
            None
        }
    }

    fn flush(&mut self) -> Result<()> {
        self.output.push("flush".to_string());
        Ok(())
    }
}
