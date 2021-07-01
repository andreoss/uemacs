use crate::buffer::Buffer;
use crate::core::{LineId, Mode};

pub const fn is_regex(buf: &Buffer) -> bool {
    buf.mode.intersects(Mode::MAGIC)
}

fn buffer_text(buf: &Buffer) -> Vec<u8> {
    let mut text = Vec::new();
    for line in buf.line_iter() {
        text.extend_from_slice(&line.text);
        text.push(b'\n');
    }
    text
}

fn line_offset_for(buf: &Buffer, byte_pos: usize) -> Option<(LineId, usize)> {
    let mut pos = 0;
    let mut cur = buf.head_line().next();
    while cur != buf.head {
        let len = buf.line_len(cur)?;
        let line_end = pos + len;
        if byte_pos <= line_end {
            return Some((cur, byte_pos - pos));
        }
        pos = line_end + 1;
        cur = buf.line(cur)?.next();
    }
    None
}

fn byte_pos_of(buf: &Buffer, line: LineId, offset: usize) -> Option<usize> {
    let mut pos = 0;
    let mut cur = buf.head_line().next();
    while cur != buf.head && cur != line {
        pos += buf.line_len(cur)? + 1;
        cur = buf.line(cur)?.next();
    }
    Some(pos + offset)
}

fn build_regex(buf: &Buffer, pattern: &[u8]) -> Option<regex::Regex> {
    let re_str = std::str::from_utf8(pattern).ok()?;
    let case_sensitive = buf.mode.intersects(Mode::EXACT);
    regex::RegexBuilder::new(re_str)
        .case_insensitive(!case_sensitive)
        .build()
        .ok()
}

pub fn find_forward_regex(
    buf: &Buffer,
    pattern: &[u8],
    line: LineId,
    offset: usize,
) -> Option<(LineId, usize)> {
    let re = build_regex(buf, pattern)?;
    let text = buffer_text(buf);
    let text_str = std::str::from_utf8(&text).ok()?;
    let start_pos = byte_pos_of(buf, line, offset)?;
    let m = re.find_at(text_str, start_pos)?;
    line_offset_for(buf, m.start())
}

pub fn find_backward_regex(
    buf: &Buffer,
    pattern: &[u8],
    line: LineId,
    offset: usize,
) -> Option<(LineId, usize)> {
    let re = build_regex(buf, pattern)?;
    let text = buffer_text(buf);
    let text_str = std::str::from_utf8(&text).ok()?;
    let end_pos = byte_pos_of(buf, line, offset)?;
    let m = re.find_iter(&text_str[..end_pos]).last()?;
    line_offset_for(buf, m.start())
}

const fn eq(a: u8, b: u8, exact: bool) -> bool {
    if exact {
        a == b
    } else {
        a.eq_ignore_ascii_case(&b)
    }
}

fn is_boundary_forward(buf: &Buffer, line: LineId, offset: usize) -> bool {
    let line_len = buf.line_len(line).unwrap_or(0);
    offset >= line_len && buf.next_line(line).is_none()
}

fn is_boundary_backward(buf: &Buffer, line: LineId, offset: usize) -> bool {
    offset == 0 && buf.prev_line(line).is_none()
}

fn advance_forward(buf: &Buffer, line: &mut LineId, offset: &mut usize) -> Option<()> {
    let line_len = buf.line_len(*line)?;
    if *offset >= line_len {
        *line = buf.next_line(*line)?;
        *offset = 0;
    } else {
        *offset += 1;
    }
    Some(())
}

fn advance_backward(buf: &Buffer, line: &mut LineId, offset: &mut usize) -> Option<()> {
    if *offset == 0 {
        *line = buf.prev_line(*line)?;
        *offset = buf.line_len(*line)?;
    } else {
        *offset -= 1;
    }
    Some(())
}

fn advance_byte(buf: &Buffer, line: &mut LineId, offset: &mut usize) -> Option<u8> {
    let line_len = buf.line_len(*line)?;
    if *offset >= line_len {
        *line = buf.next_line(*line)?;
        *offset = 0;
        Some(b'\n')
    } else {
        let b = buf.line(*line)?.text[*offset];
        *offset += 1;
        Some(b)
    }
}

fn matches_at(
    buf: &Buffer,
    pattern: &[u8],
    mut line: LineId,
    mut offset: usize,
) -> Option<(LineId, usize)> {
    let exact = buf.mode.intersects(Mode::EXACT);
    for &p in pattern {
        let byte = advance_byte(buf, &mut line, &mut offset)?;
        if !eq(byte, p, exact) {
            return None;
        }
    }
    Some((line, offset))
}

pub fn find_forward(
    buf: &Buffer,
    pattern: &[u8],
    line: LineId,
    offset: usize,
) -> Option<(LineId, usize)> {
    if pattern.is_empty() {
        return None;
    }
    let mut l = line;
    let mut o = offset;
    loop {
        if is_boundary_forward(buf, l, o) {
            return None;
        }
        if let Some(end) = matches_at(buf, pattern, l, o) {
            return Some(end);
        }
        advance_forward(buf, &mut l, &mut o)?;
    }
}

pub fn find_backward(
    buf: &Buffer,
    pattern: &[u8],
    line: LineId,
    offset: usize,
) -> Option<(LineId, usize)> {
    if pattern.is_empty() {
        return None;
    }
    let mut l = line;
    let mut o = offset;
    loop {
        if is_boundary_backward(buf, l, o) {
            return None;
        }
        advance_backward(buf, &mut l, &mut o)?;
        if matches_at(buf, pattern, l, o).is_some() {
            return Some((l, o));
        }
    }
}

#[cfg(test)]
mod tests;
