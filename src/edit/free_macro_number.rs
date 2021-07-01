use super::{Buffer, Editor, Error, LineId, Result};

pub fn region_bounds(editor: &Editor) -> Result<(LineId, usize, LineId, usize)> {
    let win = editor.current_window().ok_or(Error::Abort)?;
    let mark = win.mark().ok_or(Error::Abort)?;
    let dot = win.dot();
    let bounds = if dot < mark {
        (dot.0, dot.1.0, mark.0, mark.1.0)
    } else {
        (mark.0, mark.1.0, dot.0, dot.1.0)
    };
    Ok(bounds)
}

pub fn qr_match_start(
    buf: &Buffer,
    plen: usize,
    search_line: LineId,
    end_line: LineId,
    end_off: usize,
) -> Result<(LineId, usize)> {
    if end_line == search_line {
        return Ok((end_line, end_off - plen));
    }
    let mut remaining = plen;
    let mut cur = end_line;
    let mut cur_off = end_off;
    loop {
        if remaining <= cur_off {
            return Ok((cur, cur_off - remaining));
        }
        remaining -= cur_off + 1;
        cur = buf.prev_line(cur).unwrap_or(cur);
        cur_off = buf.line(cur).ok_or(Error::Abort)?.text.len();
    }
}

pub fn qr_preview(
    buf: &Buffer,
    start_line: LineId,
    start_off: usize,
    end_line: LineId,
    end_off: usize,
) -> Result<String> {
    if start_line == end_line {
        let l = buf.line(start_line).ok_or(Error::Abort)?;
        return Ok(String::from_utf8_lossy(&l.text[start_off..end_off]).to_string());
    }
    let s = qr_preview_multiline(buf, start_line, start_off, end_line, end_off)?;
    Ok(String::from_utf8_lossy(&s).to_string())
}

fn qr_preview_multiline(
    buf: &Buffer,
    start_line: LineId,
    start_off: usize,
    end_line: LineId,
    end_off: usize,
) -> Result<Vec<u8>> {
    let mut s = Vec::new();
    s.extend_from_slice(&buf.line(start_line).ok_or(Error::Abort)?.text[start_off..]);
    let mut cur = start_line;
    while let Some(nl) = buf.next_line(cur) {
        s.push(b'\n');
        let ll = buf.line(nl).ok_or(Error::Abort)?;
        if nl == end_line {
            s.extend_from_slice(&ll.text[..end_off]);
            break;
        }
        s.extend_from_slice(&ll.text);
        cur = nl;
    }
    Ok(s)
}
