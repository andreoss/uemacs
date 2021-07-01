use super::{BufferId, Editor, Error, LineId, Result, is_blank_line};

pub fn collect_justify_para(
    editor: &Editor,
    buf_id: BufferId,
    leftmarg: usize,
) -> Result<(LineId, LineId, LineId, Vec<Vec<u8>>)> {
    let dot_line = editor.cur_window()?.dot_line;
    let buf = editor.buffers.get(buf_id).ok_or(Error::Abort)?;
    let head = buf.head;

    let mut para_start = dot_line;
    loop {
        match buf.prev_line(para_start) {
            Some(p) if p != head && !is_blank_line(buf, p) => para_start = p,
            _ => break,
        }
    }

    let mut para_end = para_start;
    let mut any_blank_found = false;
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
    let mut first_line = true;
    loop {
        if line == end || line == head {
            break;
        }
        if let Some(l) = buf.line(line) {
            let text: &[u8] = if first_line {
                first_line = false;
                if leftmarg < l.text.len() {
                    &l.text[leftmarg..]
                } else {
                    &[]
                }
            } else {
                &l.text
            };
            for word in text.split(|&b| b == b' ' || b == b'\t') {
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
