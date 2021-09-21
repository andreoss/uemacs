use super::{
    BackwardChar, BackwardDelete, BackwardLine, BackwardPage, BackwardSearch, BackwardWord,
    Bindings, BufferFlags, BufferId, CapWord, Command, CommandId, Completion, CopyRegion,
    DeleteBackwardWord, DeleteBlankLines, DeleteForwardWord, DeleteWindow, DetabLine, Display,
    Editor, EntabLine, Error, FillPara, ForwardChar, ForwardDelete, ForwardLine, ForwardPage,
    ForwardSearch, ForwardWord, GotoBob, GotoBol, GotoBop, GotoEob, GotoEol, GotoEop, GotoLine,
    GrowWindow, InsertChar, InsertNewline, InsertTab, JustifyPara, Key, KeyCode, KeyboardQuit,
    KillLine, KillParagraph, KillRegion, KillText, LineOffset, ListBuffers, LowerRegion, LowerWord,
    META, MacroKey, Mode, MoveWindowDown, MoveWindowUp, NewlineAndIndent, NextWindow, Nop,
    OneWindow, OpenLine, OtherWindow, PreviousWindow, RefreshScreen, ResizeWindow, Result,
    ScrollNextDown, ScrollNextUp, SetFillColumn, SetMark, ShrinkWindow, SplitWindowDown,
    StringInsertMode, SuspendEmacs, SwapMark, TerminalBackend, ToggleMagic, TransposeChars,
    TrimLine, Undo, UpperRegion, UpperWord, WindowFlags, WrapWord, Yank, creates_text,
    ctlx_command, key_name, mutates_buffer, read_command_name, read_status_message,
    splice_string,
};

#[allow(clippy::too_many_lines)]
impl Editor {
    pub fn dispatch<T: TerminalBackend>(
        &mut self,
        cmd: CommandId,
        term: &mut T,
        display: &mut Display,
        bindings: &Bindings,
        f: bool,
        n: usize,
    ) -> Result<()> {
        try_record_macro(self, cmd, f, n);
        check_read_only_buffer(self, cmd)?;
        if creates_text(cmd) {
            self.ensure_dot_on_content_line()?;
        }
        let pre = track_dirty_state(self, cmd);
        let result = self.run_command(cmd, term, display, bindings, f, n);
        sync_dirty_state(self, pre);
        result
    }

    pub(super) fn run_command<T: TerminalBackend>(
        &mut self,
        cmd: CommandId,
        term: &mut T,
        display: &mut Display,
        bindings: &Bindings,
        f: bool,
        n: usize,
    ) -> Result<()> {
        match cmd {
            CommandId::ForwardChar => ForwardChar.execute(self, f, n),
            CommandId::BackwardChar => BackwardChar.execute(self, f, n),
            CommandId::ForwardLine => ForwardLine.execute(self, f, n),
            CommandId::BackwardLine => BackwardLine.execute(self, f, n),
            CommandId::SetMark => {
                SetMark.execute(self, f, n)?;
                display.write_echo(term, "(Mark set)")?;
                Ok(())
            }
            CommandId::KillLine => KillLine.execute(self, f, n),
            CommandId::InsertChar(c) => InsertChar(c).execute(self, f, n),
            CommandId::InsertNewline => InsertNewline.execute(self, f, n),
            CommandId::InsertTab => InsertTab.execute(self, f, n),
            CommandId::ForwardDelete => ForwardDelete.execute(self, f, n),
            CommandId::BackwardDelete => BackwardDelete.execute(self, f, n),
            CommandId::GotoBol => GotoBol.execute(self, f, n),
            CommandId::GotoEol => GotoEol.execute(self, f, n),
            CommandId::GotoBob => GotoBob.execute(self, f, n),
            CommandId::GotoEob => GotoEob.execute(self, f, n),
            CommandId::GotoBop => GotoBop.execute(self, f, n),
            CommandId::GotoEop => GotoEop.execute(self, f, n),
            CommandId::SwapMark => match SwapMark.execute(self, f, n) {
                Ok(()) => Ok(()),
                Err(e) => {
                    display.write_echo(term, "No mark in this window")?;
                    Err(e)
                }
            },
            CommandId::GotoLine => {
                let (eff_f, eff_n) = if f {
                    (true, n)
                } else {
                    let s = self.minibuffer_readline(term, display, "Line to GOTO: ")?;
                    let s = s.trim();
                    if s.is_empty() {
                        return Err(Error::Abort);
                    }
                    match s.parse::<usize>() {
                        Ok(parsed) => (true, parsed),
                        Err(_) => return Err(Error::Abort),
                    }
                };
                GotoLine.execute(self, eff_f, eff_n)
            }
            CommandId::ForwardPage => ForwardPage.execute(self, f, n),
            CommandId::BackwardPage => BackwardPage.execute(self, f, n),
            CommandId::KillText => KillText.execute(self, f, n),
            CommandId::KillRegion => {
                if self.current_window().is_some_and(|w| w.mark().is_none()) {
                    display.write_echo(term, "No mark set in this window")?;
                    return Err(Error::Abort);
                }
                KillRegion.execute(self, f, n)
            }
            CommandId::LowerRegion => LowerRegion.execute(self, f, n),
            CommandId::UpperRegion => UpperRegion.execute(self, f, n),
            CommandId::ForwardWord => ForwardWord.execute(self, f, n),
            CommandId::BackwardWord => BackwardWord.execute(self, f, n),
            CommandId::UpperWord => UpperWord.execute(self, f, n),
            CommandId::LowerWord => LowerWord.execute(self, f, n),
            CommandId::CapWord => CapWord.execute(self, f, n),
            CommandId::DeleteForwardWord => DeleteForwardWord.execute(self, f, n),
            CommandId::DeleteBackwardWord => DeleteBackwardWord.execute(self, f, n),
            CommandId::OpenLine => OpenLine.execute(self, f, n),
            CommandId::QuitEmacs => {
                let anycb = self.buffers.iter().any(|b| {
                    !b.flags.intersects(BufferFlags::INVISIBLE)
                        && b.flags.intersects(BufferFlags::CHANGED)
                });
                if !f && anycb {
                    display.write_echo(term, "Modified buffers exist. Leave anyway (y/n)? ")?;
                    let leave = loop {
                        match term.get_key() {
                            Some(Key::Char('y' | 'Y')) => break true,
                            Some(Key::Char('n' | 'N') | Key::Control('G') | Key::Escape) | None => {
                                break false;
                            }
                            _ => term.beep(),
                        }
                    };
                    display.write_echo(term, "")?;
                    if !leave {
                        return Ok(());
                    }
                }
                self.lock_manager.release_locks();
                self.quit_requested = true;
                Ok(())
            }
            CommandId::CopyRegion => {
                if self.current_window().is_some_and(|w| w.mark().is_none()) {
                    display.write_echo(term, "No mark set in this window")?;
                    return Err(Error::Abort);
                }
                CopyRegion.execute(self, f, n)?;
                display.write_echo(term, "(region copied)")?;
                Ok(())
            }
            CommandId::QuickExit => {
                let dirty: Vec<(BufferId, String)> = self
                    .buffers
                    .iter()
                    .filter(|b| b.flags.intersects(BufferFlags::CHANGED) && !b.filename.is_empty())
                    .map(|b| (b.id, b.filename.clone()))
                    .collect();
                for (id, fname) in dirty {
                    let b = self.buffers.get_mut(id).ok_or(Error::Abort)?;
                    crate::file::write_from_buffer(b, &fname)?;
                }
                self.lock_manager.release_locks();
                self.quit_requested = true;
                Ok(())
            }
            CommandId::KeyboardQuit => KeyboardQuit.execute(self, f, n),
            CommandId::RefreshScreen => RefreshScreen.execute(self, f, n),
            CommandId::Yank => Yank.execute(self, f, n),
            CommandId::ForwardSearch => {
                let input = self.minibuffer_readline(term, display, "Search: ")?;
                if !input.is_empty() {
                    self.search_pattern = input.into_bytes();
                }
                if self.search_pattern.is_empty() {
                    display.write_echo(term, "No pattern set")?;
                    return Err(Error::Abort);
                }
                match ForwardSearch.execute(self, f, n) {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        display.write_echo(term, "Not found")?;
                        Err(e)
                    }
                }
            }
            CommandId::BackwardSearch => {
                let input = self.minibuffer_readline(term, display, "Reverse search: ")?;
                if !input.is_empty() {
                    self.search_pattern = input.into_bytes();
                }
                if self.search_pattern.is_empty() {
                    display.write_echo(term, "No pattern set")?;
                    return Err(Error::Abort);
                }
                match BackwardSearch.execute(self, f, n) {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        display.write_echo(term, "Not found")?;
                        Err(e)
                    }
                }
            }
            CommandId::IsearchForward | CommandId::IsearchBackward => {
                self.isearch(term, display, matches!(cmd, CommandId::IsearchForward))
            }
            CommandId::QueryReplace => self.query_replace(term, display),
            CommandId::SuspendEmacs => SuspendEmacs.execute(self, f, n),
            CommandId::Undo => Undo.execute(self, f, n),
            CommandId::ExecuteCommand => {
                let name = read_command_name(term, display, bindings, "M-x ")?;
                if name.is_empty() {
                    return Ok(());
                }
                if let Some(cmd) = CommandId::from_name(&name) {
                    self.dispatch(cmd, term, display, bindings, false, 1)
                } else {
                    term.beep();
                    Ok(())
                }
            }
            CommandId::SetVar => {
                let raw_name = self.minibuffer_readline(term, display, "Set variable: ")?;
                if raw_name.is_empty() {
                    return Ok(());
                }
                let name = raw_name.trim_start_matches('$');
                let raw_value =
                    self.minibuffer_readline(term, display, &format!("Set {name} to: "))?;
                if raw_value.is_empty() {
                    return Ok(());
                }
                if let Ok(value) = raw_value.parse::<usize>() {
                    self.set_var(name, value);
                    if name == "tab" {
                        display.tab_width = value;
                    }
                } else {
                    term.beep();
                }
                Ok(())
            }
            CommandId::UniversalArgument => Ok(()),
            CommandId::CtrlXPrefix => {
                let Some(next_key) = term.get_key() else {
                    return Ok(());
                };
                let folded = match next_key {
                    Key::Char(c) if c.is_ascii_lowercase() => Key::Char(c.to_ascii_uppercase()),
                    other => other,
                };
                if let Some(cmd) = ctlx_command(&folded) {
                    self.dispatch(cmd, term, display, bindings, f, n)
                } else {
                    term.beep();
                    Ok(())
                }
            }
            CommandId::DeleteWindow => DeleteWindow.execute(self, f, n),
            CommandId::OneWindow => OneWindow.execute(self, f, n),
            CommandId::SplitWindowDown => SplitWindowDown.execute(self, f, n),
            CommandId::OtherWindow => OtherWindow.execute(self, f, n),
            CommandId::NextWindow => NextWindow.execute(self, f, n),
            CommandId::PreviousWindow => PreviousWindow.execute(self, f, n),
            CommandId::GrowWindow => GrowWindow.execute(self, f, n),
            CommandId::ShrinkWindow => ShrinkWindow.execute(self, f, n),
            CommandId::ResizeWindow => ResizeWindow.execute(self, f, n),
            CommandId::OverwriteString => {
                let s = self.minibuffer_readline(term, display, "Overwrite string: ")?;
                if s.is_empty() {
                    return Ok(());
                }
                let n = if f { n } else { 1 };
                for _ in 0..n {
                    splice_string(self, &s, StringInsertMode::Overwrite)?;
                }
                Ok(())
            }
            CommandId::ReadFile => {
                let fname = self.minibuffer_readline(term, display, "Read file: ")?;
                if fname.is_empty() {
                    return Ok(());
                }
                let buf_id = self.cur_window()?.buffer_id;
                self.lock_manager.try_lock(&fname);
                let buf = self.buffers.get_mut(buf_id).ok_or(Error::Abort)?;
                let (nlines, is_new) = crate::file::read_into_buffer(buf, &fname)?;
                let first_line = if buf.is_empty() {
                    buf.head
                } else {
                    buf.nth_line(0).unwrap_or(buf.head)
                };
                let win = self.cur_window_mut()?;
                win.top_line = first_line;
                win.dot_line = first_line;
                win.dot_offset = LineOffset(0);
                win.clear_mark();
                win.set_flag(WindowFlags::HARD);
                let msg = read_status_message(nlines, is_new);
                display.write_echo(term, &msg)?;
                Ok(())
            }
            CommandId::InsertFile => self.insert_file(term, display),
            CommandId::ViewFile => {
                let fname = self.minibuffer_readline(term, display, "View file: ")?;
                if fname.is_empty() {
                    return Ok(());
                }
                self.lock_manager.try_lock(&fname);
                let basename = std::path::Path::new(&fname)
                    .file_name()
                    .map_or_else(|| fname.clone(), |s| s.to_string_lossy().to_string());
                let new_buf_id = self.find_or_create_buffer(&basename);
                let (nlines, is_new) = {
                    let buf = self.buffers.get_mut(new_buf_id).ok_or(Error::Abort)?;
                    buf.mode |= Mode::VIEW;
                    crate::file::read_into_buffer(buf, &fname)?
                };
                self.switch_window_to_buffer(new_buf_id)?;
                let msg = read_status_message(nlines, is_new);
                display.write_echo(term, &msg)?;
                Ok(())
            }
            CommandId::SaveFile => {
                let buf_id = self.cur_window()?.buffer_id;
                let (fname, modified) = {
                    let buf = self.buffers.get(buf_id).ok_or(Error::Abort)?;
                    (buf.filename.clone(), buf.flags.intersects(BufferFlags::CHANGED))
                };
                if !modified {
                    return Ok(());
                }
                if fname.is_empty() {
                    return Err(Error::Abort);
                }
                let nlines = {
                    let buf = self.buffers.get_mut(buf_id).ok_or(Error::Abort)?;
                    crate::file::write_from_buffer(buf, &fname)?
                };
                for win in self.windows.iter_mut() {
                    if win.buffer_id == buf_id {
                        win.set_flag(WindowFlags::MODE_LINE);
                    }
                }
                let msg = if nlines == 1 {
                    "(Wrote 1 line)".to_string()
                } else {
                    format!("(Wrote {nlines} lines)")
                };
                display.write_echo(term, &msg)?;
                Ok(())
            }
            CommandId::FindFile => {
                let fname = self.minibuffer_readline(term, display, "Find file: ")?;
                if fname.is_empty() {
                    return Ok(());
                }
                self.lock_manager.try_lock(&fname);
                let basename = std::path::Path::new(&fname)
                    .file_name()
                    .map_or_else(|| fname.clone(), |s| s.to_string_lossy().to_string());
                let new_buf_id = self.find_or_create_buffer(&basename);
                let (nlines, is_new) = {
                    let buf = self.buffers.get_mut(new_buf_id).ok_or(Error::Abort)?;
                    crate::file::read_into_buffer(buf, &fname)?
                };
                self.switch_window_to_buffer(new_buf_id)?;
                let msg = read_status_message(nlines, is_new);
                display.write_echo(term, &msg)?;
                Ok(())
            }
            _ => self.run_command_2(cmd, term, display, bindings, f, n),
        }
    }

    fn run_command_2<T: TerminalBackend>(
        &mut self,
        cmd: CommandId,
        term: &mut T,
        display: &mut Display,
        bindings: &Bindings,
        f: bool,
        n: usize,
    ) -> Result<()> {
        match cmd {
            CommandId::WriteFile => {
                let fname = self.minibuffer_readline(term, display, "Write file: ")?;
                if fname.is_empty() {
                    return Ok(());
                }
                let buf_id = self.cur_window()?.buffer_id;
                let nlines = {
                    let buf = self.buffers.get_mut(buf_id).ok_or(Error::Abort)?;
                    crate::file::write_from_buffer(buf, &fname)?
                };
                let buf = self.buffers.get_mut(buf_id).ok_or(Error::Abort)?;
                buf.filename = fname;
                for win in self.windows.iter_mut() {
                    if win.buffer_id == buf_id {
                        win.set_flag(WindowFlags::MODE_LINE);
                    }
                }
                let msg = if nlines == 1 {
                    "(Wrote 1 line)".to_string()
                } else {
                    format!("(Wrote {nlines} lines)")
                };
                display.write_echo(term, &msg)?;
                Ok(())
            }
            CommandId::InsertString => {
                let s = self.minibuffer_readline(term, display, "String to insert: ")?;
                if s.is_empty() {
                    return Ok(());
                }
                let n = if f { n } else { 1 };
                for _ in 0..n {
                    splice_string(self, &s, StringInsertMode::Insert)?;
                }
                Ok(())
            }
            CommandId::GotoMatchingFence => self.goto_matching_fence(term),
            CommandId::ReplaceString => self.replace_string(term, display),
            CommandId::SwitchBuffer | CommandId::NextBuffer => {
                let name = self.minibuffer_readline_opts(
                    term,
                    display,
                    "Switch to buffer: ",
                    true,
                    Completion::Buffer,
                )?;
                if name.is_empty() {
                    return Ok(());
                }
                let new_buf_id = self.find_or_create_buffer(&name);
                self.switch_window_to_buffer(new_buf_id)?;
                Ok(())
            }
            CommandId::KillBuffer => self.kill_buffer(term, display),
            CommandId::ToggleMagic => ToggleMagic.execute(self, f, n),
            CommandId::FillPara => {
                if self.fillcol == 0 {
                    display.write_echo(term, "No fill column set")?;
                    return Err(Error::Abort);
                }
                FillPara.execute(self, f, n)
            }
            CommandId::SetFillColumn => {
                SetFillColumn.execute(self, f, n)?;
                display.write_echo(term, &format!("(Fill column is {})", self.fillcol))?;
                Ok(())
            }
            CommandId::DeleteBlankLines => DeleteBlankLines.execute(self, f, n),
            CommandId::TransposeChars => TransposeChars.execute(self, f, n),
            CommandId::CountWords => self.count_words(term, display),
            CommandId::ListBuffers => ListBuffers.execute(self, f, n),
            CommandId::TrimLine => TrimLine.execute(self, f, n),
            CommandId::DetabLine => DetabLine.execute(self, f, n),
            CommandId::EntabLine => EntabLine.execute(self, f, n),
            CommandId::Nop => Nop.execute(self, f, n),
            CommandId::BufferPosition => {
                let msg = {
                    let win = self.cur_window()?;
                    let buf = self.buffers.get(win.buffer_id).ok_or(Error::Abort)?;
                    let dot_line = win.dot_line;
                    let dot_offset = win.dot_offset.0;
                    let head = buf.head;
                    let mut num_lines = 0usize;
                    let mut num_chars = 0usize;
                    let mut pred_lines = 0usize;
                    let mut pred_chars = 0usize;
                    let mut curr = buf.lines[head.0].next();
                    while curr != head {
                        if curr == dot_line {
                            pred_lines = num_lines;
                            pred_chars = num_chars + dot_offset;
                        }
                        num_lines += 1;
                        num_chars += buf.lines[curr.0].len() + 1;
                        curr = buf.lines[curr.0].next();
                    }
                    let ratio = (pred_chars * 100).checked_div(num_chars).unwrap_or(0);
                    format!(
                        "Line {}/{} Col {} ({}%)",
                        pred_lines + 1,
                        num_lines + 1,
                        dot_offset,
                        ratio
                    )
                };
                display.write_echo(term, &msg)?;
                Ok(())
            }
            CommandId::DescribeKey => {
                let Some(key) = term.get_key() else {
                    return Ok(());
                };
                let key_name = key_name(&key);
                let kc = KeyCode::from(key);
                let msg = bindings.lookup(kc).map_or_else(
                    || format!("{key_name} is undefined"),
                    |cmd| {
                        format!(
                            "{} is bound to {} — {}",
                            key_name,
                            cmd.name(),
                            cmd.description()
                        )
                    },
                );
                display.write_echo(term, &msg)?;
                Ok(())
            }
            CommandId::DescribeBindings => self.describe_bindings(bindings),
            CommandId::Apropos => self.apropos(term, display, bindings),
            CommandId::ClearMessageLine => {
                display.write_echo(term, "")?;
                Ok(())
            }
            CommandId::QuoteChar => self.quote_char(term, n),
            CommandId::StartKbdMacro => {
                self.recording_macro = true;
                self.macro_keys.clear();
                Ok(())
            }
            CommandId::EndKbdMacro => {
                self.recording_macro = false;
                Ok(())
            }
            CommandId::CallKbdMacro => {
                let keys = self.macro_keys.clone();
                let was_recording = self.recording_macro;
                self.recording_macro = false;
                let result = (|| -> Result<()> {
                    for _ in 0..n.max(1) {
                        for &MacroKey { cmd, f, n } in &keys {
                            self.dispatch(cmd, term, display, bindings, f, n)?;
                        }
                    }
                    Ok(())
                })();
                self.recording_macro = was_recording;
                result
            }
            CommandId::MetaPrefix => {
                let Some(next_key) = term.get_key() else {
                    return Ok(());
                };
                let next_kc = KeyCode::from(next_key.clone());
                let raw = next_kc.0 & 0xff;
                let folded = if (u32::from(b'a')..=u32::from(b'z')).contains(&raw) {
                    raw ^ 0x20
                } else {
                    raw
                };
                let meta_kc = KeyCode(META | folded);
                if let Some(cmd) = bindings.lookup(meta_kc) {
                    self.dispatch(cmd, term, display, bindings, f, n)
                } else if let Key::Char(c) = next_key {
                    let folded_c = if c.is_ascii_lowercase() {
                        c.to_ascii_uppercase()
                    } else {
                        c
                    };
                    let meta_char_kc = KeyCode(META | (folded_c as u32));
                    if let Some(cmd) = bindings.lookup(meta_char_kc) {
                        self.dispatch(cmd, term, display, bindings, f, n)
                    } else {
                        term.beep();
                        Ok(())
                    }
                } else {
                    term.beep();
                    Ok(())
                }
            }
            CommandId::AddMode => self.adjustmode(true, false, term, display),
            CommandId::DeleteMode => self.adjustmode(false, false, term, display),
            CommandId::AddGlobalMode => self.adjustmode(true, true, term, display),
            CommandId::DeleteGlobalMode => self.adjustmode(false, true, term, display),
            CommandId::WriteMessage => {
                let msg = self.minibuffer_readline(term, display, "Message to write: ")?;
                if !msg.is_empty() {
                    display.write_echo(term, &msg)?;
                }
                Ok(())
            }
            CommandId::NameBuffer => {
                let name = self.minibuffer_readline(term, display, "Change buffer name to: ")?;
                if name.is_empty() {
                    return Ok(());
                }
                if self.buffers.find(&name).is_some() {
                    term.beep();
                    return Ok(());
                }
                let buf_id = self.cur_window()?.buffer_id;
                let buf = self.buffers.get_mut(buf_id).ok_or(Error::Abort)?;
                buf.name = name;
                for win in self.windows.iter_mut() {
                    if win.buffer_id == buf_id {
                        win.set_flag(WindowFlags::MODE_LINE);
                    }
                }
                Ok(())
            }
            CommandId::ChangeFileName => {
                let fname = self.minibuffer_readline(term, display, "Name: ")?;
                let buf_id = self.cur_window()?.buffer_id;
                let buf = self.buffers.get_mut(buf_id).ok_or(Error::Abort)?;
                buf.filename = fname;
                buf.mode &= !Mode::VIEW;
                for win in self.windows.iter_mut() {
                    if win.buffer_id == buf_id {
                        win.set_flag(WindowFlags::MODE_LINE);
                    }
                }
                Ok(())
            }
            CommandId::UnmarkBuffer => {
                let buf_id = self.cur_window()?.buffer_id;
                let buf = self.buffers.get_mut(buf_id).ok_or(Error::Abort)?;
                buf.flags &= !BufferFlags::CHANGED;
                for win in self.windows.iter_mut() {
                    if win.buffer_id == buf_id {
                        win.set_flag(WindowFlags::MODE_LINE);
                    }
                }
                Ok(())
            }
            CommandId::UnbindKey => {
                display.write_echo(term, ": unbind-key ")?;
                let Some(key) = term.get_key() else {
                    return Ok(());
                };
                let kc = KeyCode::from(key.clone());
                let kn = key_name(&key);
                bindings.unbind(kc);
                display.write_echo(term, &format!("[{kn} unbound]"))?;
                Ok(())
            }
            CommandId::BindToKey => {
                let name = self.minibuffer_readline(term, display, "Bind to key: ")?;
                let Some(cmd) = CommandId::from_name(&name) else {
                    term.beep();
                    return Ok(());
                };
                display.write_echo(term, "Press key: ")?;
                let Some(key) = term.get_key() else {
                    return Ok(());
                };
                let kc = KeyCode::from(key.clone());
                let kn = key_name(&key);
                bindings.bind(kc, cmd);
                display.write_echo(term, &format!("[{} bound to {}]", kn, cmd.name()))?;
                Ok(())
            }
            CommandId::MoveWindowDown => MoveWindowDown.execute(self, f, n),
            CommandId::MoveWindowUp => MoveWindowUp.execute(self, f, n),
            CommandId::ScrollNextUp => ScrollNextUp.execute(self, f, n),
            CommandId::ScrollNextDown => ScrollNextDown.execute(self, f, n),
            CommandId::HuntForward => ForwardSearch.execute(self, f, n),
            CommandId::HuntBackward => BackwardSearch.execute(self, f, n),
            CommandId::InsertSpace => {
                InsertChar(' ').execute(self, f, n)?;
                let win = self.cur_window_mut()?;
                let count = n.max(1).min(win.dot_offset.0);
                win.dot_offset = LineOffset(win.dot_offset.0 - count);
                Ok(())
            }
            CommandId::NewlineAndIndent => NewlineAndIndent.execute(self, f, n),
            CommandId::WrapWord => WrapWord.execute(self, f, n),
            CommandId::RedrawDisplay => {
                let n = if f { n } else { 0 };
                if let Some(win) = self.current_window_mut() {
                    win.force = i8::try_from(n).unwrap_or(i8::MAX);
                    win.flags |= WindowFlags::FORCE;
                }
                Ok(())
            }
            CommandId::UpdateScreen => {
                self.sgarbf_requested = true;
                Ok(())
            }
            CommandId::ChangeScreenSize => {
                let rows = term.dimensions().0;
                let n = if f { n } else { rows };
                if n >= 3 && n <= rows {
                    self.sgarbf_requested = true;
                }
                Ok(())
            }
            CommandId::ChangeScreenWidth => {
                let cols = term.dimensions().1;
                let n = if f { n } else { cols };
                if n >= 10 && n <= cols {
                    for win in self.windows.iter_mut() {
                        win.flags |=
                            WindowFlags::HARD | WindowFlags::MOVED | WindowFlags::MODE_LINE;
                    }
                    self.sgarbf_requested = true;
                }
                Ok(())
            }
            _ => self.run_command_3(cmd, term, display, bindings, f, n),
        }
    }

    fn run_command_3<T: TerminalBackend>(
        &mut self,
        cmd: CommandId,
        term: &mut T,
        display: &mut Display,
        bindings: &Bindings,
        f: bool,
        n: usize,
    ) -> Result<()> {
        match cmd {
            CommandId::SaveWindow => {
                if let Some(win) = self.current_window() {
                    self.saved_window = Some(win.id);
                }
                Ok(())
            }
            CommandId::RestoreWindow => {
                if let Some(id) = self.saved_window {
                    if !self.windows.set_current(id) {
                        display.write_echo(term, "(No such window exists)")?;
                    }
                }
                Ok(())
            }
            CommandId::Help => self.help_command(),
            CommandId::StoreMacro => {
                if f && (1..=9).contains(&n) && !self.macro_keys.is_empty() {
                    self.stored_macros[n - 1] = Some(self.macro_keys.clone());
                }
                Ok(())
            }
            CommandId::ExecuteMacro(slot @ 1..=9) => {
                self.execute_stored_macro((slot - 1) as usize, term, display, bindings, f, n)
            }
            CommandId::ExecuteMacro(slot) => {
                let bname = format!("*Macro {slot:02}*");
                if let Some(buf) = self.buffers.find(&bname) {
                    let lines: Vec<String> = buf
                        .line_iter()
                        .map(|l| String::from_utf8_lossy(&l.text).to_string())
                        .collect();
                    self.execute_lines_with_directives(&lines, term, display, bindings)?;
                }
                Ok(())
            }
            CommandId::StoreProcedure => {
                let name = self.minibuffer_readline(term, display, "Procedure name: ")?;
                if name.is_empty() {
                    return Ok(());
                }
                let bname = format!("*{name}*");
                let buf = self.buffers.find_or_create(&bname);
                buf.flags |= BufferFlags::INVISIBLE;
                buf.clear();
                self.macro_store_buffer = Some(buf.id);
                Ok(())
            }
            CommandId::ExecuteProcedure => {
                let name = self.minibuffer_readline(term, display, "Execute procedure: ")?;
                if name.is_empty() {
                    return Ok(());
                }
                let bname = format!("*{name}*");
                if let Some(buf) = self.buffers.find(&bname) {
                    let lines: Vec<String> = buf
                        .line_iter()
                        .map(|l| String::from_utf8_lossy(&l.text).to_string())
                        .collect();
                    for _ in 0..n.max(1) {
                        self.execute_lines_with_directives(&lines, term, display, bindings)?;
                    }
                } else {
                    display.write_echo(term, "(No such procedure)")?;
                }
                Ok(())
            }
            CommandId::ExecuteBuffer => {
                let name = self.minibuffer_readline(term, display, "Execute buffer: ")?;
                if name.is_empty() {
                    return Ok(());
                }
                if let Some(buf) = self.buffers.find(&name) {
                    let lines: Vec<String> = buf
                        .line_iter()
                        .map(|l| String::from_utf8_lossy(&l.text).to_string())
                        .collect();
                    self.execute_lines_with_directives(&lines, term, display, bindings)?;
                } else {
                    display.write_echo(term, &format!("(No such buffer: {name})"))?;
                }
                Ok(())
            }
            CommandId::ExecuteCommandLine => {
                let input = self.minibuffer_readline(term, display, ": ")?;
                if input.is_empty() {
                    return Ok(());
                }
                self.execute_cmd_str(&input, term, display, bindings)?;
                Ok(())
            }
            CommandId::ExecuteFile => {
                let fname = self.minibuffer_readline(term, display, "Execute file: ")?;
                if fname.is_empty() {
                    return Ok(());
                }
                let Ok(content) = std::fs::read_to_string(&fname) else {
                    display.write_echo(term, &format!("(Cannot read file: {fname})"))?;
                    return Ok(());
                };
                let lines: Vec<String> = content
                    .lines()
                    .map(std::string::ToString::to_string)
                    .collect();
                self.execute_lines_with_directives(&lines, term, display, bindings)?;
                Ok(())
            }
            CommandId::ExecuteProgram => {
                let cmd = self.minibuffer_readline(term, display, "$")?;
                if cmd.is_empty() {
                    return Ok(());
                }
                term.close()?;
                let _ = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(&cmd)
                    .status();
                term.open()?;
                display.write_echo(term, "(End)")?;
                let _ = term.get_key();
                self.sgarbf_requested = true;
                Ok(())
            }
            CommandId::ShellCommand => {
                let cmd = self.minibuffer_readline(term, display, "!")?;
                if cmd.is_empty() {
                    return Ok(());
                }
                term.close()?;
                let _ = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(&cmd)
                    .status();
                term.open()?;
                display.write_echo(term, "(End)")?;
                let _ = term.get_key();
                self.sgarbf_requested = true;
                Ok(())
            }
            CommandId::FilterBuffer => {
                let filter = self.minibuffer_readline(term, display, "#")?;
                if filter.is_empty() {
                    return Ok(());
                }
                let buf_id = self.cur_window()?.buffer_id;
                let inpath = "/tmp/uemacs_fltinp";
                let outpath = "/tmp/uemacs_fltout";
                {
                    let buf = self.buffers.get_mut(buf_id).ok_or(Error::Abort)?;
                    crate::file::write_from_buffer(buf, inpath)?;
                }
                let cmd = format!("{filter} <{inpath} >{outpath}");
                term.close()?;
                let _ = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(&cmd)
                    .status();
                term.open()?;
                if std::fs::metadata(outpath).is_ok() {
                    let buf = self.buffers.get_mut(buf_id).ok_or(Error::Abort)?;
                    crate::file::read_into_buffer(buf, outpath)?;
                    buf.flags |= BufferFlags::CHANGED;
                }
                let _ = std::fs::remove_file(inpath);
                let _ = std::fs::remove_file(outpath);
                let win = self.cur_window_mut()?;
                win.set_flag(WindowFlags::HARD);
                self.sgarbf_requested = true;
                Ok(())
            }
            CommandId::PipeCommand => {
                let cmd = self.minibuffer_readline(term, display, "@")?;
                if cmd.is_empty() {
                    return Ok(());
                }
                let outpath = "/tmp/uemacs_pipeout";
                let full_cmd = format!("{cmd} >{outpath}");
                term.close()?;
                let _ = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(&full_cmd)
                    .status();
                term.open()?;
                if std::fs::metadata(outpath).is_ok() {
                    let bname = "command";
                    let buf = self.buffers.find_or_create(bname);
                    buf.mode |= Mode::VIEW;
                    crate::file::read_into_buffer(buf, outpath)?;
                    let buf_id = buf.id;
                    let top = buf.nth_line(0).unwrap_or(buf.head);
                    if !self.windows.is_empty() {
                        let new_win = self.windows.create(buf_id, top);
                        new_win.set_flag(WindowFlags::HARD);
                    }
                }
                let _ = std::fs::remove_file(outpath);
                self.sgarbf_requested = true;
                Ok(())
            }
            CommandId::IShell => {
                term.close()?;
                let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
                let _ = std::process::Command::new(&shell).status();
                term.open()?;
                self.sgarbf_requested = true;
                Ok(())
            }
            CommandId::KillParagraph => KillParagraph.execute(self, f, n),
            CommandId::JustifyPara => {
                if self.fillcol == 0 {
                    display.write_echo(term, "No fill column set")?;
                    return Err(Error::Abort);
                }
                JustifyPara.execute(self, f, n)
            }
            _ => unreachable!("run_command_3 received unhandled command"),
        }
    }
}

fn try_record_macro(editor: &mut Editor, cmd: CommandId, f: bool, n: usize) {
    if !editor.recording_macro {
        return;
    }
    if matches!(
        cmd,
        CommandId::StartKbdMacro
            | CommandId::EndKbdMacro
            | CommandId::CallKbdMacro
            | CommandId::CtrlXPrefix
    ) {
        return;
    }
    editor.macro_keys.push(MacroKey::new(cmd, f, n));
}

fn check_read_only_buffer(editor: &Editor, cmd: CommandId) -> Result<()> {
    if !mutates_buffer(cmd) {
        return Ok(());
    }
    if let Some(win) = editor.current_window() {
        if let Some(buf) = editor.buffers.get(win.buffer_id) {
            if buf.mode.intersects(Mode::VIEW) {
                return Err(Error::Abort);
            }
        }
    }
    Ok(())
}

fn track_dirty_state(editor: &Editor, cmd: CommandId) -> Option<(BufferId, bool)> {
    if !mutates_buffer(cmd) {
        return None;
    }
    editor.current_window().and_then(|w| {
        let bid = w.buffer_id;
        editor
            .buffers
            .get(bid)
            .map(|b| (bid, b.flags.intersects(BufferFlags::CHANGED)))
    })
}

fn sync_dirty_state(editor: &mut Editor, pre: Option<(BufferId, bool)>) {
    if let Some((bid, was_dirty)) = pre {
        if !was_dirty {
            if let Some(buf) = editor.buffers.get(bid) {
                if buf.flags.intersects(BufferFlags::CHANGED) {
                    for w in editor.windows.iter_mut() {
                        if w.buffer_id == bid {
                            w.flags |= WindowFlags::MODE_LINE;
                        }
                    }
                }
            }
        }
    }
}
