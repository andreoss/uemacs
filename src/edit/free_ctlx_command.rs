use super::{CONTROL, CommandId, Key, KeyCode, META};

pub const fn ctlx_command(key: &Key) -> Option<CommandId> {
    match *key {
        Key::Char(c) => ctlx_char(c),
        Key::Control(c) => ctlx_control(c),
        _ => None,
    }
}

const fn ctlx_char(c: char) -> Option<CommandId> {
    match c {
        '(' => Some(CommandId::StartKbdMacro),
        ')' => Some(CommandId::EndKbdMacro),
        'E' => Some(CommandId::CallKbdMacro),
        '0' => Some(CommandId::DeleteWindow),
        '1' => Some(CommandId::OneWindow),
        '2' => Some(CommandId::SplitWindowDown),
        'O' => Some(CommandId::OtherWindow),
        'B' => Some(CommandId::SwitchBuffer),
        'K' => Some(CommandId::KillBuffer),
        'Z' | '^' => Some(CommandId::GrowWindow),
        'W' => Some(CommandId::ResizeWindow),
        '!' => Some(CommandId::ShellCommand),
        '#' => Some(CommandId::FilterBuffer),
        '@' => Some(CommandId::PipeCommand),
        'C' => Some(CommandId::IShell),
        '$' => Some(CommandId::ExecuteProgram),
        '?' => Some(CommandId::DescribeKey),
        '=' => Some(CommandId::BufferPosition),
        'A' => Some(CommandId::SetVar),
        'F' => Some(CommandId::SetFillColumn),
        'M' => Some(CommandId::AddMode),
        'N' => Some(CommandId::ChangeFileName),
        'P' => Some(CommandId::PreviousWindow),
        'R' => Some(CommandId::IsearchBackward),
        'S' => Some(CommandId::IsearchForward),
        'X' => Some(CommandId::NextBuffer),
        'D' => Some(CommandId::SuspendEmacs),
        'Q' => Some(CommandId::QuoteChar),
        _ => None,
    }
}

const fn ctlx_control(c: char) -> Option<CommandId> {
    match c {
        'D' | 'S' => Some(CommandId::SaveFile),
        'C' => Some(CommandId::QuitEmacs),
        'F' => Some(CommandId::FindFile),
        'W' => Some(CommandId::WriteFile),
        'Z' => Some(CommandId::ShrinkWindow),
        'R' => Some(CommandId::ReadFile),
        'I' => Some(CommandId::InsertFile),
        'V' => Some(CommandId::ViewFile),
        'B' => Some(CommandId::ListBuffers),
        'X' => Some(CommandId::SwapMark),
        'A' => Some(CommandId::DetabLine),
        'E' => Some(CommandId::EntabLine),
        'L' => Some(CommandId::LowerRegion),
        'M' => Some(CommandId::DeleteMode),
        'N' => Some(CommandId::MoveWindowDown),
        'O' => Some(CommandId::DeleteBlankLines),
        'P' => Some(CommandId::MoveWindowUp),
        'T' => Some(CommandId::TrimLine),
        'U' => Some(CommandId::UpperRegion),
        _ => None,
    }
}

pub fn longest_common_prefix(strings: &[&str]) -> String {
    if strings.is_empty() {
        return String::new();
    }
    let mut prefix = strings[0].to_string();
    for s in &strings[1..] {
        let cap = prefix
            .chars()
            .zip(s.chars())
            .take_while(|(a, b)| a == b)
            .count();
        prefix.truncate(prefix.chars().take(cap).map(char::len_utf8).sum());
    }
    prefix
}

pub fn parse_key_name(s: &str) -> Option<KeyCode> {
    let bytes = s.as_bytes();
    let (c, i) = parse_key_prefix(bytes);
    let (c, i) = parse_key_control_byte(bytes, c, i)?;
    parse_key_final(bytes, c, i)
}

fn parse_key_prefix(bytes: &[u8]) -> (u32, usize) {
    let mut c: u32 = 0;
    let mut i = 0;
    if i + 1 < bytes.len() && bytes[i] == b'M' && bytes[i + 1] == b'-' {
        c |= META;
        i += 2;
    }
    (c, i)
}

fn parse_key_control_byte(bytes: &[u8], mut c: u32, mut i: usize) -> Option<(u32, usize)> {
    if i + 1 < bytes.len() && bytes[i] == b'^' && bytes[i + 1] == b'X' && i + 2 < bytes.len() {
        return None;
    }
    let mut as_ctrl = false;
    if i + 1 < bytes.len() && bytes[i] == b'^' {
        as_ctrl = true;
        i += 1;
    }
    if i >= bytes.len() {
        return None;
    }
    let mut ch = bytes[i];
    if ch < 32 {
        as_ctrl = true;
        ch = ch.wrapping_add(b'A');
    }
    if ch.is_ascii_lowercase() {
        ch -= 32;
    }
    if as_ctrl {
        c |= CONTROL | (u32::from(ch) & 0x1f);
    } else {
        c |= u32::from(ch);
    }
    Some((c, i))
}

const fn parse_key_final(bytes: &[u8], c: u32, i: usize) -> Option<KeyCode> {
    if i + 1 != bytes.len() {
        return None;
    }
    Some(KeyCode(c))
}

pub fn next_rng() -> u64 {
    use std::cell::Cell;
    thread_local! {
        static SEED: Cell<u64> = Cell::new({
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0x9E37_79B9_7F4A_7C15, |d| u64::from(d.subsec_nanos()) | 1)
        });
    }
    SEED.with(|s| {
        let next = s.get().wrapping_mul(1721).wrapping_add(10007);
        s.set(next);
        next
    })
}

pub fn gtfun_arity(fname: &str) -> usize {
    match fname {
        "neg" | "not" | "ind" | "len" | "upp" | "low" | "tru" | "asc" | "chr" | "rnd" | "abs"
        | "env" | "bin" | "exi" | "fin" | "bno" => 1,
        "add" | "sub" | "tim" | "div" | "mod" | "cat" | "lef" | "rig" | "equ" | "les" | "gre"
        | "seq" | "sle" | "sgr" | "and" | "or" | "sin" | "ban" | "bor" | "bxo" => 2,
        "mid" | "xla" => 3,
        _ => 0,
    }
}
