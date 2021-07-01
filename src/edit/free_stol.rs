use super::{Bindings, Display, Key, Result, TerminalBackend, longest_common_prefix};

pub fn stol(val: &str) -> bool {
    match val.as_bytes().first() {
        Some(b'F') => false,
        Some(b'T') => true,
        _ => parse_leading_int(val) != 0,
    }
}

pub fn parse_leading_int(s: &str) -> i64 {
    let s = s.trim_start();
    let bytes = s.as_bytes();
    let (neg, i) = parse_leading_sign(bytes);
    let start = i;
    let mut i = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == start {
        return 0;
    }
    let n: i64 = s[start..i].parse().unwrap_or(0);
    if neg { -n } else { n }
}

fn parse_leading_sign(bytes: &[u8]) -> (bool, usize) {
    if bytes.first() == Some(&b'-') {
        (true, 1)
    } else if bytes.first() == Some(&b'+') {
        (false, 1)
    } else {
        (false, 0)
    }
}

pub fn next_token(src: &str) -> (String, &str) {
    let bytes = src.as_bytes();
    let mut i = 0;
    while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
        i += 1;
    }
    next_token_parse(&src[i..])
}

const fn next_token_escape(esc: u8) -> char {
    match esc {
        b'r' => '\r',
        b'n' => '\n',
        b't' => '\t',
        b'b' => '\x08',
        b'f' => '\x0c',
        x => x as char,
    }
}

fn next_token_parse(src: &str) -> (String, &str) {
    let bytes = src.as_bytes();
    let mut i = 0;
    let mut tok = String::new();
    let mut quoted = false;
    while i < bytes.len() && next_token_byte(&mut tok, bytes, &mut i, &mut quoted) {}
    (tok, &src[i..])
}

fn next_token_byte(tok: &mut String, bytes: &[u8], i: &mut usize, quoted: &mut bool) -> bool {
    let c = bytes[*i];
    if c == b'~' && *i + 1 < bytes.len() {
        tok.push(next_token_escape(bytes[*i + 1]));
        *i += 2;
        return true;
    }
    if *quoted {
        if c == b'"' {
            *i += 1;
            return false;
        }
        tok.push(c as char);
        *i += 1;
        return true;
    }
    if c == b' ' || c == b'\t' {
        return false;
    }
    if c == b'"' {
        *quoted = true;
        tok.push('"');
        *i += 1;
        return true;
    }
    tok.push(c as char);
    *i += 1;
    true
}

pub fn read_command_name<T: TerminalBackend>(
    term: &mut T,
    display: &mut Display,
    bindings: &Bindings,
    prompt: &str,
) -> Result<String> {
    let mut buf = String::new();
    let mut cycle: Option<(Vec<String>, usize)> = None;
    display.write_echo(term, prompt)?;
    loop {
        let Some(key) = term.get_key() else {
            display.write_echo(term, "")?;
            return Ok(String::new());
        };
        if let ReadCmdResult::Done(s) =
            read_command_name_key(&mut buf, &key, term, display, bindings, prompt, &mut cycle)?
        {
            return Ok(s);
        }
    }
}

enum ReadCmdResult {
    Done(String),
    Continue,
}

fn read_command_name_key<T: TerminalBackend>(
    buf: &mut String,
    key: &Key,
    term: &mut T,
    display: &mut Display,
    bindings: &Bindings,
    prompt: &str,
    cycle: &mut Option<(Vec<String>, usize)>,
) -> Result<ReadCmdResult> {
    match key {
        Key::Enter => {
            display.write_echo(term, "")?;
            Ok(ReadCmdResult::Done(buf.clone()))
        }
        Key::Unknown(0) | Key::Control('G') | Key::Escape => {
            display.write_echo(term, "")?;
            Ok(ReadCmdResult::Done(String::new()))
        }
        Key::Backspace | Key::Delete => {
            *cycle = None;
            buf.pop();
            read_cmd_refresh(buf, term, display, prompt)?;
            Ok(ReadCmdResult::Continue)
        }
        Key::Tab | Key::Char(' ') => read_cmd_complete(buf, term, display, bindings, prompt, cycle),
        Key::Char(c) => {
            *cycle = None;
            buf.push(*c);
            read_cmd_refresh(buf, term, display, prompt)?;
            Ok(ReadCmdResult::Continue)
        }
        _ => {
            term.beep();
            Ok(ReadCmdResult::Continue)
        }
    }
}

fn read_cmd_refresh<T: TerminalBackend>(
    buf: &str,
    term: &mut T,
    display: &mut Display,
    prompt: &str,
) -> Result<()> {
    display.write_echo(term, &format!("{prompt}{buf}"))
}

fn read_cmd_complete<T: TerminalBackend>(
    buf: &mut String,
    term: &mut T,
    display: &mut Display,
    bindings: &Bindings,
    prompt: &str,
    cycle: &mut Option<(Vec<String>, usize)>,
) -> Result<ReadCmdResult> {
    let continuing = matches!(cycle.as_ref(), Some((cands, idx)) if *buf == cands[*idx]);
    if let Some((cands, idx)) = cycle.take().filter(|_| continuing) {
        let next = (idx + 1) % cands.len();
        buf.clone_from(&cands[next]);
        read_cmd_refresh(buf, term, display, prompt)?;
        *cycle = Some((cands, next));
        return Ok(ReadCmdResult::Continue);
    }
    *cycle = None;
    let candidates = bindings.command_names_with_prefix(buf);
    if candidates.is_empty() {
        term.beep();
    } else if candidates.len() == 1 {
        *buf = candidates[0].to_string();
        read_cmd_refresh(buf, term, display, prompt)?;
    } else {
        let refs: Vec<&str> = candidates.iter().map(AsRef::as_ref).collect();
        let common = longest_common_prefix(&refs);
        if common.len() > buf.len() {
            *buf = common;
            read_cmd_refresh(buf, term, display, prompt)?;
        } else {
            let cands: Vec<String> = candidates.iter().map(ToString::to_string).collect();
            buf.clone_from(&cands[0]);
            read_cmd_refresh(buf, term, display, prompt)?;
            *cycle = Some((cands, 0));
        }
    }
    Ok(ReadCmdResult::Continue)
}

#[derive(Clone, Copy)]
pub enum StringInsertMode {
    Insert,
    Overwrite,
}
