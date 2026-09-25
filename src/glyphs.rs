//! Glyphs: the small pictures beside the text, in three tiers.
//!
//! Emoji are single code points that are wide by default everywhere; symbols and
//! ASCII are for terminals that cannot draw them. Every glyph sits beside a text
//! label, so none carries meaning alone. This is the only module with emoji.

use crate::environment::Environment;

/// Which set of glyphs the terminal can draw.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GlyphTier {
    /// Colour emoji, two cells wide.
    Emoji,
    /// One-cell symbols from common fonts.
    Symbols,
    /// Plain ASCII.
    Ascii,
}

/// What a glyph stands for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Glyph {
    /// The program itself.
    App,
    /// A work phase.
    Work,
    /// A short break.
    ShortBreak,
    /// A long break.
    LongBreak,
    /// Counting down.
    Running,
    /// Paused.
    Paused,
    /// A phase waiting for space; the alert.
    Ready,
    /// A work phase of this cycle already done.
    RoundDone,
    /// A work phase of this cycle still to come.
    RoundToDo,
}

impl GlyphTier {
    /// The tier the environment describes (G01).
    pub(crate) fn detect(environment: &Environment) -> Self {
        if environment.dumb() || environment.linux_console() {
            Self::Ascii
        } else if environment.legacy_windows_console() {
            Self::Symbols
        } else {
            Self::Emoji
        }
    }

    /// The glyph for `role` in this tier.
    pub(crate) fn glyph(self, role: Glyph) -> &'static str {
        match self {
            Self::Emoji => match role {
                Glyph::App | Glyph::Work | Glyph::RoundDone => "🍅",
                Glyph::ShortBreak => "☕",
                Glyph::LongBreak => "🌙",
                Glyph::Running => "⏳",
                Glyph::Paused => "💤",
                Glyph::Ready => "🔔",
                Glyph::RoundToDo => "⚪",
            },
            Self::Symbols => match role {
                Glyph::App => "◆",
                Glyph::Work => "■",
                Glyph::ShortBreak => "□",
                Glyph::LongBreak => "▣",
                Glyph::Running => "▸",
                Glyph::Paused => "‖",
                Glyph::Ready => "◉",
                Glyph::RoundDone => "●",
                Glyph::RoundToDo => "○",
            },
            Self::Ascii => match role {
                Glyph::App => "*",
                Glyph::Work => "W",
                Glyph::ShortBreak => "s",
                Glyph::LongBreak => "L",
                Glyph::Running => ">",
                Glyph::Paused => "=",
                Glyph::Ready => "!",
                Glyph::RoundDone => "#",
                Glyph::RoundToDo => ".",
            },
        }
    }
}

#[cfg(test)]
mod tests;
