macro_rules! commands {
    ( $( $variant:ident = $name:literal $(| $alias:literal)* : $desc:literal ; )* ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum CommandId {
            $( $variant, )*
            InsertChar(char),
            ExecuteMacro(u8),
        }

        impl CommandId {
            #[must_use]
            #[allow(clippy::too_many_lines)]
            pub fn name(self) -> std::borrow::Cow<'static, str> {
                match self {
                    $( Self::$variant => std::borrow::Cow::Borrowed($name), )*
                    Self::InsertChar(_) => std::borrow::Cow::Borrowed("self-insert-command"),
                    Self::ExecuteMacro(n) => std::borrow::Cow::Owned(format!("execute-macro-{n}")),
                }
            }

            #[must_use]
            #[allow(clippy::too_many_lines)]
            pub const fn description(self) -> &'static str {
                match self {
                    $( Self::$variant => $desc, )*
                    Self::InsertChar(_) => "Insert a literal character at point",
                    Self::ExecuteMacro(_) => "Execute a stored macro",
                }
            }

            #[must_use]
            #[allow(clippy::too_many_lines)]
            pub fn from_name(s: &str) -> Option<Self> {
                if let Some(n) = s.strip_prefix("execute-macro-") {
                    return n.parse().ok().map(Self::ExecuteMacro);
                }
                match s {
                    $( $name $(| $alias)* => Some(Self::$variant), )*
                    _ => None,
                }
            }
        }
    };
}

commands! {
    ForwardChar = "forward-char" | "forward-character" : "Move point forward one character";
    BackwardChar = "backward-char" | "backward-character" : "Move point backward one character";
    GotoBop = "goto-bop" | "previous-paragraph" : "Move point to the beginning of the paragraph";
    GotoEop = "goto-eop" | "next-paragraph" : "Move point to the end of the paragraph";
    SwapMark = "swap-mark" | "exchange-point-and-mark" : "Exchange point and mark";
    QuitEmacs = "quit-emacs" | "exit-emacs" : "Exit the editor";
    ForwardLine = "forward-line" | "next-line" : "Move point to the next line";
    BackwardLine = "backward-line" | "previous-line" : "Move point to the previous line";
    SetMark = "set-mark" : "Set the mark at point";
    KillLine = "kill-line" : "Kill from point to end of line";
    InsertNewline = "insert-newline" | "newline" : "Insert a newline at point";
    InsertTab = "insert-tab" | "handle-tab" : "Insert a tab character at point";
    ForwardDelete = "forward-delete" | "delete-next-character" : "Delete the character after point";
    BackwardDelete = "backward-delete" | "delete-previous-character" : "Delete the character before point";
    GotoBol = "goto-bol" | "beginning-of-line" : "Move point to the beginning of the line";
    GotoEol = "goto-eol" | "end-of-line" : "Move point to the end of the line";
    GotoBob = "goto-bob" | "beginning-of-file" : "Move point to the beginning of the buffer";
    GotoEob = "goto-eob" | "end-of-file" : "Move point to the end of the buffer";
    GotoLine = "goto-line" : "Move point to a specified line number";
    ForwardPage = "forward-page" | "next-page" : "Scroll forward one page";
    BackwardPage = "backward-page" | "previous-page" : "Scroll backward one page";
    KillText = "kill-text" | "kill-to-end-of-line" : "Kill text from point to end of line";
    KillRegion = "kill-region" : "Kill the region between point and mark";
    Yank = "yank" : "Insert the most recently killed text";
    CopyRegion = "copy-region" : "Copy the region to the kill buffer";
    LowerRegion = "lower-region" | "case-region-lower" : "Lowercase the region";
    UpperRegion = "upper-region" | "case-region-upper" : "Uppercase the region";
    ForwardWord = "forward-word" | "next-word" : "Move point forward one word";
    BackwardWord = "backward-word" | "previous-word" : "Move point backward one word";
    UpperWord = "upper-word" | "case-word-upper" : "Uppercase the word at point";
    LowerWord = "lower-word" | "case-word-lower" : "Lowercase the word at point";
    CapWord = "cap-word" | "case-word-capitalize" : "Capitalize the word at point";
    DeleteForwardWord = "delete-forward-word" | "delete-next-word" : "Delete forward one word";
    DeleteBackwardWord = "delete-backward-word" | "delete-previous-word" : "Delete backward one word";
    OpenLine = "open-line" : "Insert a newline after point, leaving point in place";
    ForwardSearch = "forward-search" | "search-forward" : "Search forward for a string";
    BackwardSearch = "backward-search" | "search-reverse" : "Search backward for a string";
    KeyboardQuit = "keyboard-quit" | "abort-command" : "Abort the current command";
    RefreshScreen = "refresh-screen" | "clear-and-redraw" : "Redraw the screen and recenter point";
    UniversalArgument = "universal-argument" : "Begin numeric prefix argument";
    MetaPrefix = "meta-prefix" : "Meta prefix key (ESC)";
    SuspendEmacs = "suspend-emacs" : "Suspend the editor process";
    Undo = "undo" : "Undo the last edit";
    QueryReplace = "query-replace" | "query-replace-string" : "Replace occurrences of a string with confirmation";
    ExecuteCommand = "execute-extended-command" | "execute-named-command" : "Read a command name from the minibuffer and execute it";
    IsearchForward = "isearch-forward" | "incremental-search" : "Incremental search forward";
    IsearchBackward = "isearch-backward" | "reverse-incremental-search" : "Incremental search backward";
    StartKbdMacro = "start-kbd-macro" | "begin-macro" : "Begin recording a keyboard macro";
    EndKbdMacro = "end-kbd-macro" | "end-macro" : "Finish recording a keyboard macro";
    CallKbdMacro = "call-last-kbd-macro" | "execute-macro" : "Execute the last recorded keyboard macro";
    CtrlXPrefix = "ctrl-x-prefix" | "ctlx-prefix" : "Control-X prefix key";
    SetVar = "set-variable" | "set" : "Set the value of an editor variable";
    SaveFile = "save-file" : "Save the current buffer to its file";
    FindFile = "find-file" : "Visit a file in a new buffer";
    WriteFile = "write-file" : "Write the current buffer to a named file";
    DeleteWindow = "delete-window" : "Delete the current window";
    OneWindow = "one-window" | "delete-other-windows" : "Delete all windows except the current one";
    SplitWindowDown = "split-window" | "split-current-window" : "Split the current window into two";
    OtherWindow = "other-window" : "Switch focus to another window";
    SwitchBuffer = "switch-buffer" | "select-buffer" : "Switch the current window to a different buffer";
    KillBuffer = "kill-buffer" | "delete-buffer" : "Delete a buffer";
    ToggleMagic = "toggle-magic" : "Toggle MAGIC search mode (regex)";
    FillPara = "fill-paragraph" : "Re-fill the current paragraph";
    SetFillColumn = "set-fill-column" : "Set the fill column for paragraph filling";
    DeleteBlankLines = "delete-blank-lines" : "Delete blank lines around point";
    QuoteChar = "quote-char" | "quote-character" : "Insert the next keystroke literally";
    TransposeChars = "transpose-characters" : "Transpose the characters around point";
    CountWords = "count-words" : "Count words in the region";
    ClearMessageLine = "clear-message-line" : "Clear the echo area message";
    ListBuffers = "list-buffers" : "Display a list of all buffers";
    TrimLine = "trim-line" : "Remove trailing whitespace from the current line";
    DetabLine = "detab-line" : "Convert tabs to spaces on the current line";
    EntabLine = "entab-line" : "Convert runs of spaces to tabs on the current line";
    Nop = "nop" : "No operation";
    BufferPosition = "buffer-position" : "Show point position in the buffer";
    DescribeKey = "describe-key" : "Describe the command bound to a key";
    DescribeBindings = "describe-bindings" : "Display all key bindings";
    Apropos = "apropos" : "List commands matching a pattern";
    NextWindow = "next-window" : "Switch to the next window";
    PreviousWindow = "previous-window" : "Switch to the previous window";
    GrowWindow = "grow-window" : "Enlarge the current window";
    ShrinkWindow = "shrink-window" : "Shrink the current window";
    ResizeWindow = "resize-window" : "Resize the current window to a specified height";
    ReadFile = "read-file" : "Read a file into the current buffer";
    InsertFile = "insert-file" : "Insert the contents of a file at point";
    ViewFile = "view-file" : "Visit a file in read-only mode";
    OverwriteString = "overwrite-string" : "Read a string and overwrite text at point";
    InsertString = "insert-string" : "Read a string and insert it at point";
    GotoMatchingFence = "goto-matching-fence" : "Move to the matching brace/bracket/paren";
    ReplaceString = "replace-string" : "Replace all occurrences of a string";
    AddMode = "add-mode" : "Add a mode to the current buffer";
    DeleteMode = "delete-mode" : "Remove a mode from the current buffer";
    AddGlobalMode = "add-global-mode" : "Add a global mode";
    DeleteGlobalMode = "delete-global-mode" : "Remove a global mode";
    WriteMessage = "write-message" : "Write a message to the echo area";
    NameBuffer = "name-buffer" : "Rename the current buffer";
    ChangeFileName = "change-file-name" : "Change the file name associated with the current buffer";
    UnmarkBuffer = "unmark-buffer" : "Mark the current buffer as unchanged";
    UnbindKey = "unbind-key" : "Remove a key binding";
    BindToKey = "bind-to-key" : "Bind a key to a command";
    QuickExit = "quick-exit" : "Save all modified buffers and exit";
    MoveWindowDown = "move-window-down" : "Scroll the current window down";
    MoveWindowUp = "move-window-up" : "Scroll the current window up";
    ScrollNextUp = "scroll-next-up" : "Scroll the next window up";
    ScrollNextDown = "scroll-next-down" : "Scroll the next window down";
    HuntForward = "hunt-forward" : "Repeat the last forward search";
    HuntBackward = "hunt-backward" : "Repeat the last backward search";
    NextBuffer = "next-buffer" : "Switch to the next buffer";
    InsertSpace = "insert-space" : "Insert a space character at point";
    NewlineAndIndent = "newline-and-indent" : "Insert a newline and indent to match the previous line";
    WrapWord = "wrap-word" : "Wrap the current line at the fill column";
    RedrawDisplay = "redraw-display" : "Recenter the current window around point";
    UpdateScreen = "update-screen" : "Force a screen update";
    ChangeScreenSize = "change-screen-size" : "Change the screen height";
    ChangeScreenWidth = "change-screen-width" : "Change the screen width";
    SaveWindow = "save-window" : "Save the current window configuration";
    RestoreWindow = "restore-window" : "Restore a saved window configuration";
    Help = "help" : "Display help";
    StoreMacro = "store-macro" : "Save the current keyboard macro to a named slot";
    ExecuteBuffer = "execute-buffer" : "Execute a buffer as a macro";
    ExecuteCommandLine = "execute-command-line" : "Execute a single command line";
    ExecuteFile = "execute-file" : "Execute a file as a macro";
    ExecuteProgram = "execute-program" : "Run an external program";
    ShellCommand = "shell-command" : "Run a shell command";
    FilterBuffer = "filter-buffer" : "Filter the buffer through a shell command";
    PipeCommand = "pipe-command" : "Pipe shell command output into a buffer";
    IShell = "i-shell" : "Start an interactive shell";
    KillParagraph = "kill-paragraph" : "Kill the current paragraph";
    JustifyPara = "justify-paragraph" : "Justify the current paragraph";
    StoreProcedure = "store-procedure" : "Store a named procedure";
    ExecuteProcedure = "execute-procedure" | "run" : "Execute a named procedure";
}
