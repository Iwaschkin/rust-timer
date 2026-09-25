//! The environment variables that decide colour, glyphs and progress support.
//!
//! They are read once, through a lookup function, so tests describe a terminal
//! without touching the real environment. This is the only module that names them.

use std::ffi::OsString;

/// What the environment says about the terminal the program runs in.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Environment {
    no_color: bool,
    term: String,
    colorterm: String,
    windows_terminal: bool,
    term_program: String,
    windows: bool,
    console_truecolor: bool,
}

impl Environment {
    /// Reads the variables through `lookup`. `windows` says whether the program
    /// runs on Windows, and `console_truecolor` whether the Windows console
    /// accepted virtual-terminal output.
    pub(crate) fn from_lookup(
        lookup: impl Fn(&str) -> Option<OsString>,
        windows: bool,
        console_truecolor: bool,
    ) -> Self {
        let text = |name: &str| {
            lookup(name)
                .map(|value| value.to_string_lossy().into_owned())
                .unwrap_or_default()
        };
        Self {
            no_color: !text("NO_COLOR").is_empty(),
            term: text("TERM"),
            colorterm: text("COLORTERM"),
            windows_terminal: lookup("WT_SESSION").is_some(),
            term_program: text("TERM_PROGRAM"),
            windows,
            console_truecolor,
        }
    }

    /// `NO_COLOR` is set and not empty.
    pub(crate) fn no_color(&self) -> bool {
        self.no_color
    }

    /// `TERM` is `dumb`, a terminal that takes no escape sequences.
    pub(crate) fn dumb(&self) -> bool {
        self.term == "dumb"
    }

    /// `TERM` is the Linux virtual console, which has no emoji font.
    pub(crate) fn linux_console(&self) -> bool {
        self.term == "linux"
    }

    /// `COLORTERM` announces 24-bit colour.
    pub(crate) fn announces_truecolor(&self) -> bool {
        self.colorterm.contains("truecolor") || self.colorterm.contains("24bit")
    }

    /// Windows Terminal, which draws 24-bit colour but does not set `COLORTERM`.
    pub(crate) fn windows_terminal(&self) -> bool {
        self.windows_terminal
    }

    /// A Windows console that accepted virtual-terminal output.
    pub(crate) fn console_truecolor(&self) -> bool {
        self.windows && self.console_truecolor
    }

    /// `TERM` names a 256-colour terminal.
    pub(crate) fn announces_256(&self) -> bool {
        self.term.contains("256")
    }

    /// Windows with neither Windows Terminal nor another terminal program: the
    /// legacy console host, which cannot draw emoji.
    pub(crate) fn legacy_windows_console(&self) -> bool {
        self.windows && !self.windows_terminal && self.term_program.is_empty()
    }
}

#[cfg(test)]
pub(crate) mod tests;
