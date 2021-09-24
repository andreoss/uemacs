use super::{Bindings, CONTROL, CommandId, HashMap, KeyCode, META, RefCell, ctrl, meta};

const fn mctrl(b: u8) -> KeyCode {
    KeyCode(META | CONTROL | (b as u32 & 0x1f))
}

const fn ctrl_char(c: char) -> KeyCode {
    KeyCode(CONTROL | (c as u32 & 0x1f))
}

const fn meta_char(c: char) -> KeyCode {
    KeyCode(META | c as u32)
}

const fn first_byte(s: &str) -> u8 {
    s.as_bytes()[0]
}

macro_rules! key {
    (C - M - $k:ident) => { mctrl(first_byte(stringify!($k))) };
    (C - $k:ident) => { ctrl(first_byte(stringify!($k))) };
    (C - $k:literal) => { ctrl_char($k) };
    (M - Backspace) => { KeyCode(META | 0x7f) };
    (M - $k:ident) => { meta(first_byte(stringify!($k))) };
    (M - $k:literal) => { meta_char($k) };
    (Enter) => { KeyCode(0x0d) };
    (Tab) => { KeyCode(0x09) };
    (Backspace) => { KeyCode(0x7f) };
    (Up) => { KeyCode(0x101) };
    (Down) => { KeyCode(0x102) };
    (Left) => { KeyCode(0x103) };
    (Right) => { KeyCode(0x104) };
    (PageUp) => { KeyCode(0x105) };
    (PageDown) => { KeyCode(0x106) };
    (Home) => { KeyCode(0x107) };
    (End) => { KeyCode(0x108) };
    (Delete) => { KeyCode(0x109) };
}

macro_rules! keybindings {
    (@build [ $( ($code:expr, $cmd:ident) )* ]) => {
        const BINDINGS: &[(KeyCode, CommandId)] = &[ $( ($code, CommandId::$cmd), )* ];
        impl Bindings {
            pub fn new() -> Self {
                let mut map = HashMap::new();
                for &(code, cmd) in BINDINGS {
                    map.insert(code, cmd);
                }
                Self {
                    map: RefCell::new(map),
                }
            }
        }
    };
    (@build [$($acc:tt)*] $($rest:tt)*) => {
        keybindings!(@keys [$($acc)*] [] $($rest)*);
    };
    (@keys [$($acc:tt)*] [ $( ($code:expr) )* ] => $cmd:ident ; $($rest:tt)*) => {
        keybindings!(@build [ $($acc)* $( ($code, $cmd) )* ] $($rest)*);
    };
    (@keys [$($acc:tt)*] [$($k:tt)*] | $($rest:tt)*) => {
        keybindings!(@keys [$($acc)*] [$($k)*] $($rest)*);
    };
    (@keys [$($acc:tt)*] [$($k:tt)*] C - M - $key:ident $($rest:tt)*) => {
        keybindings!(@keys [$($acc)*] [$($k)* (key!(C - M - $key))] $($rest)*);
    };
    (@keys [$($acc:tt)*] [$($k:tt)*] C - $key:tt $($rest:tt)*) => {
        keybindings!(@keys [$($acc)*] [$($k)* (key!(C - $key))] $($rest)*);
    };
    (@keys [$($acc:tt)*] [$($k:tt)*] M - $key:tt $($rest:tt)*) => {
        keybindings!(@keys [$($acc)*] [$($k)* (key!(M - $key))] $($rest)*);
    };
    (@keys [$($acc:tt)*] [$($k:tt)*] $key:ident $($rest:tt)*) => {
        keybindings!(@keys [$($acc)*] [$($k)* (key!($key))] $($rest)*);
    };
    ( $($body:tt)* ) => {
        keybindings!(@build [] $($body)*);
    };
}

keybindings! {
    Enter | C-M     => InsertNewline;
    Tab | C-I       => InsertTab;
    Backspace | C-H => BackwardDelete;
    C-D | Delete    => ForwardDelete;
    C-'@'           => SetMark;
    C-A | Home      => GotoBol;
    C-B | Left      => BackwardChar;
    C-C             => InsertSpace;
    C-E | End       => GotoEol;
    C-F | Right     => ForwardChar;
    C-G             => KeyboardQuit;
    C-J             => NewlineAndIndent;
    C-K             => KillText;
    C-L             => RefreshScreen;
    C-N | Down      => ForwardLine;
    C-O             => OpenLine;
    C-P | Up        => BackwardLine;
    C-Q             => QuoteChar;
    C-R             => IsearchBackward;
    C-S             => IsearchForward;
    C-T             => TransposeChars;
    C-V | PageDown  => ForwardPage;
    C-W             => KillRegion;
    C-X             => CtrlXPrefix;
    C-Y             => Yank;
    C-Z | PageUp    => BackwardPage;
    C-'['           => MetaPrefix;
    C-']'           => MetaPrefix;
    C-'\\'          => KeyboardQuit;
    C-'_'           => Undo;
    M-'%'           => QueryReplace;
    M-'!'           => RedrawDisplay;
    M-' '           => SetMark;
    M-'.'           => SetMark;
    M-'<'           => GotoBob;
    M-'>'           => GotoEob;
    M-'?'           => Help;
    M-'~'           => UnmarkBuffer;
    M-A             => Apropos;
    M-B             => BackwardWord;
    M-C             => CapWord;
    M-D             => DeleteForwardWord;
    M-F             => ForwardWord;
    M-G             => GotoLine;
    M-J             => JustifyPara;
    M-K             => BindToKey;
    M-L             => LowerWord;
    M-M             => AddGlobalMode;
    M-N             => GotoEop;
    M-P             => GotoBop;
    M-Q             => FillPara;
    M-R             => ReplaceString;
    M-S             => ForwardSearch;
    M-U             => UpperWord;
    M-V             => BackwardPage;
    M-W             => CopyRegion;
    M-X             => ExecuteCommand;
    M-Z             => QuickExit;
    M-Backspace     => DeleteBackwardWord;
    C-M-C           => CountWords;
    C-M-D           => ChangeScreenSize;
    C-M-E           => ExecuteProcedure;
    C-M-F           => GotoMatchingFence;
    C-M-H           => DeleteBackwardWord;
    C-M-K           => UnbindKey;
    C-M-L           => RedrawDisplay;
    C-M-M           => DeleteGlobalMode;
    C-M-N           => NameBuffer;
    C-M-R           => QueryReplace;
    C-M-S           => ChangeScreenSize;
    C-M-T           => ChangeScreenWidth;
    C-M-V           => ScrollNextDown;
    C-M-W           => KillParagraph;
    C-M-Z           => ScrollNextUp;
}
