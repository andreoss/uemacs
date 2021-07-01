pub fn is_utf8_start_byte(b: u8) -> bool {
    !(0x80..0xC0).contains(&b)
}

pub const fn utf8_char_width(first_byte: u8) -> usize {
    if first_byte < 0x80 {
        1
    } else if first_byte < 0xE0 {
        2
    } else if first_byte < 0xF0 {
        3
    } else {
        4
    }
}
