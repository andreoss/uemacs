use super::*;

#[test]
fn test_bindings_iter_contains_known_keys() {
    let bindings = Bindings::new();
    let pairs = bindings.entries();
    assert!(pairs.contains(&(ctrl(b'A'), CommandId::GotoBol)));
    assert!(pairs.contains(&(ctrl(b'F'), CommandId::ForwardChar)));
}

#[test]
fn test_ctrl_z_is_backward_page() {
    let bindings = Bindings::new();
    assert_eq!(
        bindings.lookup(ctrl(b'Z')),
        Some(CommandId::BackwardPage),
        "C ebind.h binds C-Z to backpage; suspend is on C-X D / M-S in C"
    );
}

#[test]
fn test_root_bindings() {
    let b = Bindings::new();
    let mc = |c: u8| KeyCode(META | CONTROL | (u32::from(c) & 0x1f));

    let pairs: &[(KeyCode, CommandId, &str)] = &[
        (ctrl(b'A'), CommandId::GotoBol, "C-A gotobol"),
        (ctrl(b'B'), CommandId::BackwardChar, "C-B backchar"),
        (ctrl(b'C'), CommandId::InsertSpace, "C-C insspace"),
        (ctrl(b'D'), CommandId::ForwardDelete, "C-D forwdel"),
        (ctrl(b'E'), CommandId::GotoEol, "C-E gotoeol"),
        (ctrl(b'F'), CommandId::ForwardChar, "C-F forwchar"),
        (ctrl(b'G'), CommandId::KeyboardQuit, "C-G ctrlg"),
        (ctrl(b'H'), CommandId::BackwardDelete, "C-H backdel"),
        (ctrl(b'I'), CommandId::InsertTab, "C-I insert_tab"),
        (ctrl(b'J'), CommandId::NewlineAndIndent, "C-J indent"),
        (ctrl(b'K'), CommandId::KillText, "C-K killtext"),
        (ctrl(b'L'), CommandId::RefreshScreen, "C-L redraw"),
        (ctrl(b'M'), CommandId::InsertNewline, "C-M insert_newline"),
        (ctrl(b'N'), CommandId::ForwardLine, "C-N forwline"),
        (ctrl(b'O'), CommandId::OpenLine, "C-O openline"),
        (ctrl(b'P'), CommandId::BackwardLine, "C-P backline"),
        (ctrl(b'Q'), CommandId::QuoteChar, "C-Q quote"),
        (ctrl(b'T'), CommandId::TransposeChars, "C-T twiddle"),
        (ctrl(b'V'), CommandId::ForwardPage, "C-V forwpage"),
        (ctrl(b'W'), CommandId::KillRegion, "C-W killregion"),
        (ctrl(b'X'), CommandId::CtrlXPrefix, "C-X cex"),
        (ctrl(b'Y'), CommandId::Yank, "C-Y yank"),
        (ctrl(b'Z'), CommandId::BackwardPage, "C-Z backpage"),
        (KeyCode(0x7f), CommandId::BackwardDelete, "DEL backdel"),
        (meta(b' '), CommandId::SetMark, "M-SPC setmark"),
        (meta(b'.'), CommandId::SetMark, "M-. setmark"),
        (meta(b'<'), CommandId::GotoBob, "M-< gotobob"),
        (meta(b'>'), CommandId::GotoEob, "M-> gotoeob"),
        (meta(b'?'), CommandId::Help, "M-? help"),
        (meta(b'~'), CommandId::UnmarkBuffer, "M-~ unmark"),
        (meta(b'A'), CommandId::Apropos, "M-A apro"),
        (meta(b'B'), CommandId::BackwardWord, "M-B backword"),
        (meta(b'C'), CommandId::CapWord, "M-C capword"),
        (meta(b'D'), CommandId::DeleteForwardWord, "M-D delfword"),
        (meta(b'F'), CommandId::ForwardWord, "M-F forwword"),
        (meta(b'G'), CommandId::GotoLine, "M-G gotoline"),
        (meta(b'J'), CommandId::JustifyPara, "M-J justpara"),
        (meta(b'K'), CommandId::BindToKey, "M-K bindtokey"),
        (meta(b'L'), CommandId::LowerWord, "M-L lowerword"),
        (meta(b'M'), CommandId::AddGlobalMode, "M-M setgmode"),
        (meta(b'N'), CommandId::GotoEop, "M-N gotoeop"),
        (meta(b'P'), CommandId::GotoBop, "M-P gotobop"),
        (meta(b'Q'), CommandId::FillPara, "M-Q fillpara"),
        (meta(b'R'), CommandId::ReplaceString, "M-R sreplace"),
        (
            meta(b'S'),
            CommandId::ForwardSearch,
            "M-S forwsearch (PKCODE)",
        ),
        (meta(b'U'), CommandId::UpperWord, "M-U upperword"),
        (meta(b'V'), CommandId::BackwardPage, "M-V backpage"),
        (meta(b'W'), CommandId::CopyRegion, "M-W copyregion"),
        (meta(b'X'), CommandId::ExecuteCommand, "M-X namedcmd"),
        (meta(b'Z'), CommandId::QuickExit, "M-Z quickexit"),
        (
            KeyCode(META | 0x7f),
            CommandId::DeleteBackwardWord,
            "M-DEL delbword",
        ),
        (mc(b'C'), CommandId::CountWords, "M-C-C wordcount"),
        (mc(b'D'), CommandId::ChangeScreenSize, "M-C-D newsize"),
        (
            mc(b'F'),
            CommandId::GotoMatchingFence,
            "M-C-F getfence (CFENCE)",
        ),
        (mc(b'H'), CommandId::DeleteBackwardWord, "M-C-H delbword"),
        (mc(b'K'), CommandId::UnbindKey, "M-C-K unbindkey"),
        (mc(b'L'), CommandId::RedrawDisplay, "M-C-L reposition"),
        (mc(b'M'), CommandId::DeleteGlobalMode, "M-C-M delgmode"),
        (mc(b'N'), CommandId::NameBuffer, "M-C-N namebuffer"),
        (mc(b'R'), CommandId::QueryReplace, "M-C-R qreplace"),
        (mc(b'S'), CommandId::ChangeScreenSize, "M-C-S newsize"),
        (mc(b'T'), CommandId::ChangeScreenWidth, "M-C-T newwidth"),
        (mc(b'V'), CommandId::ScrollNextDown, "M-C-V scrnextdw"),
        (
            mc(b'W'),
            CommandId::KillParagraph,
            "M-C-W killpara (WORDPRO)",
        ),
        (mc(b'Z'), CommandId::ScrollNextUp, "M-C-Z scrnextup"),
    ];

    for (kc, expected_cmd, desc) in pairs {
        assert_eq!(
            b.lookup(*kc),
            Some(*expected_cmd),
            "{desc}: keycode {:#x} should bind to {expected_cmd:?}",
            kc.0
        );
    }
}

#[test]
fn test_meta_h_is_unbound() {
    let b = Bindings::new();
    assert_eq!(b.lookup(meta(b'H')), None);
}

#[test]
fn test_ctlx_bindings() {
    use crate::terminal::Key;
    let ctlx = crate::edit::ctlx_command_for_test;

    let pairs: &[(Key, CommandId, &str)] = &[
        (Key::Control('A'), CommandId::DetabLine, "C-X C-A detab"),
        (
            Key::Control('B'),
            CommandId::ListBuffers,
            "C-X C-B listbuffers",
        ),
        (Key::Control('C'), CommandId::QuitEmacs, "C-X C-C quit"),
        (
            Key::Control('D'),
            CommandId::SaveFile,
            "C-X C-D filesave (PKCODE)",
        ),
        (Key::Control('E'), CommandId::EntabLine, "C-X C-E entab"),
        (Key::Control('F'), CommandId::FindFile, "C-X C-F filefind"),
        (Key::Control('I'), CommandId::InsertFile, "C-X C-I insfile"),
        (
            Key::Control('L'),
            CommandId::LowerRegion,
            "C-X C-L lowerregion",
        ),
        (Key::Control('M'), CommandId::DeleteMode, "C-X C-M delmode"),
        (
            Key::Control('N'),
            CommandId::MoveWindowDown,
            "C-X C-N mvdnwind",
        ),
        (
            Key::Control('O'),
            CommandId::DeleteBlankLines,
            "C-X C-O deblank",
        ),
        (
            Key::Control('P'),
            CommandId::MoveWindowUp,
            "C-X C-P mvupwind",
        ),
        (Key::Control('R'), CommandId::ReadFile, "C-X C-R fileread"),
        (Key::Control('S'), CommandId::SaveFile, "C-X C-S filesave"),
        (Key::Control('T'), CommandId::TrimLine, "C-X C-T trim"),
        (
            Key::Control('U'),
            CommandId::UpperRegion,
            "C-X C-U upperregion",
        ),
        (Key::Control('V'), CommandId::ViewFile, "C-X C-V viewfile"),
        (Key::Control('W'), CommandId::WriteFile, "C-X C-W filewrite"),
        (Key::Control('X'), CommandId::SwapMark, "C-X C-X swapmark"),
        (
            Key::Control('Z'),
            CommandId::ShrinkWindow,
            "C-X C-Z shrinkwind",
        ),
    ];

    for (k, expected_cmd, desc) in pairs {
        assert_eq!(
            ctlx(k),
            Some(*expected_cmd),
            "{desc}: should bind to {expected_cmd:?}"
        );
    }
}

#[test]
fn test_ctlx_bindings_chars() {
    use crate::terminal::Key;
    let ctlx = crate::edit::ctlx_command_for_test;

    let pairs: &[(Key, CommandId, &str)] = &[
        (Key::Char('?'), CommandId::DescribeKey, "C-X ? deskey"),
        (
            Key::Char('!'),
            CommandId::ShellCommand,
            "C-X ! spawn (scope-boundary)",
        ),
        (
            Key::Char('@'),
            CommandId::PipeCommand,
            "C-X @ pipecmd (scope-boundary)",
        ),
        (
            Key::Char('#'),
            CommandId::FilterBuffer,
            "C-X # filter_buffer (scope-boundary)",
        ),
        (
            Key::Char('$'),
            CommandId::ExecuteProgram,
            "C-X $ execprg (scope-boundary)",
        ),
        (Key::Char('='), CommandId::BufferPosition, "C-X = showcpos"),
        (Key::Char('('), CommandId::StartKbdMacro, "C-X ( ctlxlp"),
        (Key::Char(')'), CommandId::EndKbdMacro, "C-X ) ctlxrp"),
        (Key::Char('^'), CommandId::GrowWindow, "C-X ^ enlargewind"),
        (Key::Char('0'), CommandId::DeleteWindow, "C-X 0 delwind"),
        (Key::Char('1'), CommandId::OneWindow, "C-X 1 onlywind"),
        (
            Key::Char('2'),
            CommandId::SplitWindowDown,
            "C-X 2 splitwind",
        ),
        (Key::Char('A'), CommandId::SetVar, "C-X A setvar"),
        (Key::Char('B'), CommandId::SwitchBuffer, "C-X B usebuffer"),
        (
            Key::Char('C'),
            CommandId::IShell,
            "C-X C spawncli (scope-boundary)",
        ),
        (Key::Char('E'), CommandId::CallKbdMacro, "C-X E ctlxe"),
        (Key::Char('F'), CommandId::SetFillColumn, "C-X F setfillcol"),
        (Key::Char('K'), CommandId::KillBuffer, "C-X K killbuffer"),
        (Key::Char('M'), CommandId::AddMode, "C-X M setemode"),
        (Key::Char('N'), CommandId::ChangeFileName, "C-X N filename"),
        (Key::Char('O'), CommandId::OtherWindow, "C-X O nextwind"),
        (Key::Char('P'), CommandId::PreviousWindow, "C-X P prevwind"),
        (
            Key::Char('Q'),
            CommandId::QuoteChar,
            "C-X Q quote (PKCODE alt)",
        ),
        (
            Key::Char('R'),
            CommandId::IsearchBackward,
            "C-X R risearch (ISRCH)",
        ),
        (
            Key::Char('S'),
            CommandId::IsearchForward,
            "C-X S fisearch (ISRCH)",
        ),
        (Key::Char('W'), CommandId::ResizeWindow, "C-X W resize"),
        (Key::Char('X'), CommandId::NextBuffer, "C-X X nextbuffer"),
        (Key::Char('Z'), CommandId::GrowWindow, "C-X Z enlargewind"),
    ];

    for (k, expected_cmd, desc) in pairs {
        assert_eq!(
            ctlx(k),
            Some(*expected_cmd),
            "{desc}: should bind to {expected_cmd:?}"
        );
    }
}

#[test]
fn test_unbind_removes_key() {
    let bindings = Bindings::new();
    bindings.bind(KeyCode(0xBBB), CommandId::ForwardChar);
    assert_eq!(
        bindings.lookup(KeyCode(0xBBB)),
        Some(CommandId::ForwardChar)
    );
    bindings.unbind(KeyCode(0xBBB));
    assert_eq!(bindings.lookup(KeyCode(0xBBB)), None);
}
