use super::{BufferId, Editor, Error, LineId, LineOffset, Result, WindowFlags};

pub fn advance_dot_one_line(editor: &mut Editor) -> bool {
    let (buf_id, line) = match editor.current_window() {
        Some(w) => (w.buffer_id, w.dot_line),
        None => return false,
    };
    let next = match editor.buffers.get(buf_id) {
        Some(b) => b.next_line(line),
        None => return false,
    };
    next.is_some_and(|next| {
        if let Some(w) = editor.current_window_mut() {
            w.dot_line = next;
            w.dot_offset = LineOffset(0);
        }
        true
    })
}

pub fn is_blank_line(buf: &crate::buffer::Buffer, line: LineId) -> bool {
    buf.line(line)
        .is_none_or(|l| l.text.iter().all(|&b| b == b' ' || b == b'\t'))
}

pub fn cursor_column(editor: &Editor) -> Option<usize> {
    let win = editor.current_window()?;
    let buf = editor.buffers.get(win.buffer_id)?;
    let line = buf.line(win.dot_line)?;
    let text = &line.text;
    let tab_width = editor.tab_width;
    let mut col = 0;
    for &b in text.iter().take(win.dot_offset.0) {
        if b == b'\t' {
            col = col + tab_width - (col % tab_width);
        } else {
            col += 1;
        }
    }
    Some(col)
}

pub fn find_common_prefix(strings: &[String]) -> String {
    if strings.is_empty() {
        return String::new();
    }
    let first = &strings[0];
    let mut max_prefix = first.len();
    for s in &strings[1..] {
        let common = first
            .chars()
            .zip(s.chars())
            .take_while(|(a, b)| a == b)
            .count();
        max_prefix = max_prefix.min(common);
    }
    first[..max_prefix].to_string()
}

pub fn move_window_impl(editor: &mut Editor, n: usize, up: bool) -> Result<()> {
    let n = if n == 0 { 1 } else { n };
    let (buffer_id, top_line) = {
        let win = editor.cur_window()?;
        (win.buffer_id, win.top_line)
    };
    let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
    let mut lp = top_line;
    if up {
        for _ in 0..n {
            match buf.prev_line(lp) {
                Some(prev) if prev != buf.head => lp = prev,
                _ => break,
            }
        }
    } else {
        for _ in 0..n {
            match buf.next_line(lp) {
                Some(next) => lp = next,
                None => break,
            }
        }
    }
    let new_top = lp;
    {
        let win = editor.cur_window_mut()?;
        win.top_line = new_top;
        win.set_flag(WindowFlags::HARD);
    }
    let n_rows = {
        let win = editor.cur_window()?;
        win.n_rows
    };
    let dot_line = editor.cur_window()?.dot_line;
    let mut scan = new_top;
    let mut visible = false;
    for _ in 0..n_rows {
        let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
        if scan == dot_line {
            visible = true;
            break;
        }
        if scan == buf.head {
            break;
        }
        scan = buf.next_line(scan).unwrap_or(buf.head);
    }
    if !visible && n_rows > 0 {
        let buf = editor.buffers.get(buffer_id).ok_or(Error::Abort)?;
        let mut center = new_top;
        let half = n_rows / 2;
        for _ in 0..half {
            match buf.next_line(center) {
                Some(next) if next != buf.head => center = next,
                _ => break,
            }
        }
        let win = editor.cur_window_mut()?;
        win.dot_line = center;
        win.dot_offset = LineOffset(0);
    }
    Ok(())
}

pub fn collect_paragraph(
    editor: &Editor,
    buf_id: BufferId,
    dot_line: LineId,
) -> Result<(LineId, LineId, LineId, Vec<Vec<u8>>)> {
    let buf = editor.buffers.get(buf_id).ok_or(Error::Abort)?;
    let head = buf.head;

    let mut para_start = dot_line;
    let mut any_blank_found = false;
    loop {
        match buf.prev_line(para_start) {
            Some(p) if p != head && !is_blank_line(buf, p) => para_start = p,
            _ => break,
        }
    }

    let mut para_end = para_start;
    loop {
        match buf.next_line(para_end) {
            Some(n) if n != head && !is_blank_line(buf, n) => {
                para_end = n;
                any_blank_found = false;
            }
            Some(n) if n != head && is_blank_line(buf, n) && !any_blank_found => {
                any_blank_found = true;
            }
            _ => break,
        }
        if any_blank_found {
            break;
        }
    }
    let start = para_start;
    let end = buf.next_line(para_end).unwrap_or(head);

    let mut words = Vec::new();
    let mut line = start;
    loop {
        if line == end || line == head {
            break;
        }
        if let Some(l) = buf.line(line) {
            for word in l.text.split(|&b| b == b' ' || b == b'\t') {
                if !word.is_empty() {
                    words.push(word.to_vec());
                }
            }
        }
        match buf.next_line(line) {
            Some(n) => line = n,
            None => break,
        }
    }
    Ok((head, start, end, words))
}
