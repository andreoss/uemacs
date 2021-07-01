use crate::buffer::Buffer;
use crate::core::{BufferFlags, Error, Result};
use crate::line::Line;
use std::path::Path;

fn try_read_file(filename: &str) -> Result<(Vec<u8>, bool)> {
    if !Path::new(filename).exists() {
        return Ok((Vec::new(), true));
    }
    match std::fs::read(filename) {
        Ok(c) => Ok((c, false)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok((Vec::new(), true)),
        Err(_) => Err(Error::IoError),
    }
}

fn populate_buffer(buf: &mut Buffer, content: &[u8]) -> usize {
    let parts: Vec<&[u8]> = content.split(|&b| b == b'\n').collect();
    let limit = if content.last() == Some(&b'\n') {
        parts.len() - 1
    } else {
        parts.len()
    };
    let mut last_line = buf.head;
    let mut line_count = 0;
    for &line_bytes in &parts[..limit] {
        let id = buf.insert_after(last_line, Line::new());
        if !line_bytes.is_empty() {
            buf.line_mut(id).unwrap().text = line_bytes.to_vec();
        }
        last_line = id;
        line_count += 1;
    }
    line_count
}

pub fn read_into_buffer(buf: &mut Buffer, filename: &str) -> Result<(usize, bool)> {
    buf.clear();
    let (content, is_new) = try_read_file(filename)?;
    if is_new {
        return Ok((0, true));
    }
    buf.filename = filename.to_string();
    buf.flags &= !(BufferFlags::INVISIBLE | BufferFlags::CHANGED | BufferFlags::TRUNCATED);
    if content.is_empty() {
        return Ok((0, false));
    }
    let line_count = populate_buffer(buf, &content);
    Ok((line_count, false))
}

pub fn write_from_buffer(buf: &mut Buffer, filename: &str) -> Result<usize> {
    let mut output = Vec::new();
    let mut line_count = 0;
    for line in buf.line_iter() {
        output.extend_from_slice(&line.text);
        output.push(b'\n');
        line_count += 1;
    }
    std::fs::write(filename, &output)?;
    buf.flags &= !BufferFlags::CHANGED;
    Ok(line_count)
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn file_exists(filename: &str) -> bool {
    Path::new(filename).exists()
}

#[cfg(test)]
mod tests;
