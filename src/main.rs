#![recursion_limit = "512"]

mod bind;
mod buffer;
mod core;

mod display;
mod edit;
mod file;
mod input;
mod kbd;
mod line;
mod lock;
mod search;
mod terminal;
mod util;
mod window;

use bind::Bindings;
use core::CommandId;
use display::Display;
use edit::Editor;
use input::getkey;
use terminal::{Key, TerminalBackend};

#[cfg(unix)]
const unsafe extern "C" fn sigcont_handler(_sig: libc::c_int) {}

pub(crate) fn mark_screen_garbage(editor: &mut Editor, display: &mut Display) {
    display.sgarbf = true;
    for win in editor.windows.iter_mut() {
        win.flags |= core::WindowFlags::HARD | core::WindowFlags::MODE_LINE;
    }
}

pub(crate) fn run<T: TerminalBackend>(editor: &mut Editor, display: &mut Display, term: &mut T) {
    let bindings = Bindings::new();
    if let Some(win) = editor.current_window_mut() {
        win.n_rows = display.nrows().saturating_sub(2);
        win.flags = core::WindowFlags::HARD | core::WindowFlags::MODE_LINE;
    }
    display
        .update(&mut editor.windows, &editor.buffers, term)
        .ok();
    loop {
        editor.swap_flags();
        let ev = getkey(term);
        if ev.key == Key::Unknown(0) {
            break;
        }
        let kc = core::KeyCode::from(ev.key.clone());
        let folded_kc = if let Key::Meta(c) = ev.key {
            if c.is_ascii_lowercase() {
                core::KeyCode(core::META | u32::from(c.to_ascii_uppercase() as u8))
            } else {
                kc
            }
        } else {
            kc
        };
        let cmd = bindings
            .lookup(kc)
            .or_else(|| bindings.lookup(folded_kc))
            .or(match ev.key {
                Key::Char(c) => Some(CommandId::InsertChar(c)),
                _ => None,
            });
        if let Some(cmd) = cmd {
            match editor.dispatch(cmd, term, display, &bindings, ev.f, ev.n) {
                Ok(()) => {}
                Err(core::Error::Abort) => term.beep(),
                Err(core::Error::IoError) => break,
            }
        }
        if editor.quit_requested {
            break;
        }
        if editor.suspend_requested {
            editor.suspend_requested = false;
            #[cfg(unix)]
            {
                let _ = term.close();
                unsafe {
                    libc::signal(libc::SIGTSTP, libc::SIG_DFL);
                    let mut action: libc::sigaction = std::mem::zeroed();
                    action.sa_sigaction = sigcont_handler as *const () as usize;
                    libc::sigaction(libc::SIGCONT, &raw const action, std::ptr::null_mut());
                    libc::raise(libc::SIGTSTP);
                }
                let _ = term.open();
                if let Some(win) = editor.current_window_mut() {
                    win.n_rows = display.nrows().saturating_sub(2);
                }
                mark_screen_garbage(editor, display);
            }
        }
        if editor.sgarbf_requested {
            display.sgarbf = true;
            editor.sgarbf_requested = false;
        }
        let (rows, cols) = term.dimensions();
        editor.screen_rows = rows;
        editor.screen_cols = cols;
        let prev_nrows = display.nrows();
        display.resize(rows, cols);
        display.tab_width = editor.tab_width;
        if display.nrows() != prev_nrows {
            let total_text = display.nrows().saturating_sub(1);
            let n = editor.windows.iter().count().max(1);
            let per = total_text.checked_div(n).unwrap_or(1).max(1);
            let mut row = 0usize;
            for (i, win) in editor.windows.iter_mut().enumerate() {
                win.top_row = row;
                win.n_rows = if i + 1 == n {
                    total_text.saturating_sub(row).saturating_sub(1)
                } else {
                    per.saturating_sub(1)
                };
                row += per;
            }
        }
        if display
            .update(&mut editor.windows, &editor.buffers, term)
            .is_err()
        {
            break;
        }
    }
}

fn main() {
    let mut editor = Editor::new();
    let bid = editor.create_buffer("main");
    let head = editor.buffers.get(bid).unwrap().head;
    editor.create_window(bid, head);

    if let Some(filename) = std::env::args().nth(1) {
        let buf = editor.buffers.get_mut(bid).unwrap();
        let _ = crate::file::read_into_buffer(buf, &filename);
        let first = if buf.is_empty() {
            buf.head
        } else {
            buf.nth_line(0).unwrap_or(buf.head)
        };
        if let Some(win) = editor.current_window_mut() {
            win.top_line = first;
            win.dot_line = first;
            win.dot_offset = core::LineOffset(0);
        }
    }

    #[cfg(unix)]
    {
        let mut term = terminal::posix::PosixTerminal::new();
        if term.open().is_err() {
            return;
        }
        let mut display = Display::new(&term);
        run(&mut editor, &mut display, &mut term);
        let _ = term.close();
    }

    #[cfg(not(unix))]
    {
        let mut term = terminal::mock::MockTerminal::new();
        let mut display = Display::new(&term);
        run(&mut editor, &mut display, &mut term);
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
