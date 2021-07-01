use super::{Bindings, CONTROL, CommandId, HashMap, KeyCode, META, RefCell, ctrl, meta};

const fn raw(b: u8) -> KeyCode {
    KeyCode(b as u32)
}

const fn mctrl(b: u8) -> KeyCode {
    KeyCode(META | CONTROL | (b as u32 & 0x1f))
}

const ASCII_KEYS: &[(KeyCode, CommandId)] = &[
    (raw(b'\r'), CommandId::InsertNewline),
    (raw(b'\t'), CommandId::InsertTab),
    (raw(0x7f), CommandId::BackwardDelete),
];

const CONTROL_KEYS: &[(KeyCode, CommandId)] = &[
    (ctrl(b'@'), CommandId::SetMark),
    (ctrl(b'A'), CommandId::GotoBol),
    (ctrl(b'B'), CommandId::BackwardChar),
    (ctrl(b'C'), CommandId::InsertSpace),
    (ctrl(b'D'), CommandId::ForwardDelete),
    (ctrl(b'E'), CommandId::GotoEol),
    (ctrl(b'F'), CommandId::ForwardChar),
    (ctrl(b'G'), CommandId::KeyboardQuit),
    (ctrl(b'H'), CommandId::BackwardDelete),
    (ctrl(b'I'), CommandId::InsertTab),
    (ctrl(b'J'), CommandId::NewlineAndIndent),
    (ctrl(b'K'), CommandId::KillText),
    (ctrl(b'L'), CommandId::RefreshScreen),
    (ctrl(b'M'), CommandId::InsertNewline),
    (ctrl(b'N'), CommandId::ForwardLine),
    (ctrl(b'O'), CommandId::OpenLine),
    (ctrl(b'P'), CommandId::BackwardLine),
    (ctrl(b'Q'), CommandId::QuoteChar),
    (ctrl(b'R'), CommandId::IsearchBackward),
    (ctrl(b'S'), CommandId::IsearchForward),
    (ctrl(b'T'), CommandId::TransposeChars),
    (ctrl(b'V'), CommandId::ForwardPage),
    (ctrl(b'W'), CommandId::KillRegion),
    (ctrl(b'X'), CommandId::CtrlXPrefix),
    (ctrl(b'Y'), CommandId::Yank),
    (ctrl(b'Z'), CommandId::BackwardPage),
    (ctrl(b'['), CommandId::MetaPrefix),
    (ctrl(b']'), CommandId::MetaPrefix),
    (ctrl(b'\\'), CommandId::KeyboardQuit),
    (ctrl(b'_'), CommandId::Undo),
];

const CURSOR_KEYS: &[(KeyCode, CommandId)] = &[
    (KeyCode(0x101), CommandId::BackwardLine),
    (KeyCode(0x102), CommandId::ForwardLine),
    (KeyCode(0x103), CommandId::BackwardChar),
    (KeyCode(0x104), CommandId::ForwardChar),
    (KeyCode(0x105), CommandId::BackwardPage),
    (KeyCode(0x106), CommandId::ForwardPage),
    (KeyCode(0x107), CommandId::GotoBol),
    (KeyCode(0x108), CommandId::GotoEol),
    (KeyCode(0x109), CommandId::ForwardDelete),
];

const META_KEYS: &[(KeyCode, CommandId)] = &[
    (meta(b'%'), CommandId::QueryReplace),
    (meta(b'!'), CommandId::RedrawDisplay),
    (meta(b' '), CommandId::SetMark),
    (meta(b'.'), CommandId::SetMark),
    (meta(b'<'), CommandId::GotoBob),
    (meta(b'>'), CommandId::GotoEob),
    (meta(b'?'), CommandId::Help),
    (meta(b'~'), CommandId::UnmarkBuffer),
    (meta(b'A'), CommandId::Apropos),
    (meta(b'B'), CommandId::BackwardWord),
    (meta(b'C'), CommandId::CapWord),
    (meta(b'D'), CommandId::DeleteForwardWord),

    (meta(b'F'), CommandId::ForwardWord),
    (meta(b'G'), CommandId::GotoLine),
    (meta(b'J'), CommandId::JustifyPara),
    (meta(b'K'), CommandId::BindToKey),
    (meta(b'L'), CommandId::LowerWord),
    (meta(b'M'), CommandId::AddGlobalMode),
    (meta(b'N'), CommandId::GotoEop),
    (meta(b'P'), CommandId::GotoBop),
    (meta(b'Q'), CommandId::FillPara),
    (meta(b'R'), CommandId::ReplaceString),
    (meta(b'S'), CommandId::ForwardSearch),
    (meta(b'U'), CommandId::UpperWord),
    (meta(b'V'), CommandId::BackwardPage),
    (meta(b'W'), CommandId::CopyRegion),
    (meta(b'X'), CommandId::ExecuteCommand),
    (meta(b'Z'), CommandId::QuickExit),
    (KeyCode(META | 0x7f), CommandId::DeleteBackwardWord),
];

const META_CONTROL_KEYS: &[(KeyCode, CommandId)] = &[
    (mctrl(b'C'), CommandId::CountWords),
    (mctrl(b'D'), CommandId::ChangeScreenSize),
    (mctrl(b'E'), CommandId::ExecuteProcedure),
    (mctrl(b'F'), CommandId::GotoMatchingFence),
    (mctrl(b'H'), CommandId::DeleteBackwardWord),
    (mctrl(b'K'), CommandId::UnbindKey),
    (mctrl(b'L'), CommandId::RedrawDisplay),
    (mctrl(b'M'), CommandId::DeleteGlobalMode),
    (mctrl(b'N'), CommandId::NameBuffer),
    (mctrl(b'R'), CommandId::QueryReplace),
    (mctrl(b'S'), CommandId::ChangeScreenSize),
    (mctrl(b'T'), CommandId::ChangeScreenWidth),
    (mctrl(b'V'), CommandId::ScrollNextDown),
    (mctrl(b'W'), CommandId::KillParagraph),
    (mctrl(b'Z'), CommandId::ScrollNextUp),
];

const BINDING_TABLES: &[&[(KeyCode, CommandId)]] = &[
    ASCII_KEYS,
    CONTROL_KEYS,
    CURSOR_KEYS,
    META_KEYS,
    META_CONTROL_KEYS,
];

impl Bindings {
    pub fn new() -> Self {
        let mut map = HashMap::new();
        for table in BINDING_TABLES {
            for &(code, cmd) in *table {
                map.insert(code, cmd);
            }
        }
        Self {
            map: RefCell::new(map),
        }
    }
}
