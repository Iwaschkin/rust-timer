//! The command line: flags, usage text and argument errors.
//!
//! This is the only module that names the flags and their choices.

use crate::glyphs::GlyphTier;
use crate::motion::Motion;
use crate::options::{Choice, Options};
use crate::settings::{InvalidNumber, RANGE, Settings};
use crate::theme::ColorDepth;
use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::fmt;

/// What the command line asks for.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Invocation {
    /// Run the timer with these options.
    Run(Options),
    /// Print the usage text.
    Help,
}

/// A flag that takes a value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Flag {
    /// `--work`.
    Work,
    /// `--short`.
    Short,
    /// `--long`.
    Long,
    /// `--every`.
    Every,
    /// `--color`.
    Color,
    /// `--glyphs`.
    Glyphs,
    /// `--motion`.
    Motion,
}

impl Flag {
    const ALL: [Self; 7] = [
        Self::Work,
        Self::Short,
        Self::Long,
        Self::Every,
        Self::Color,
        Self::Glyphs,
        Self::Motion,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::Work => "--work",
            Self::Short => "--short",
            Self::Long => "--long",
            Self::Every => "--every",
            Self::Color => "--color",
            Self::Glyphs => "--glyphs",
            Self::Motion => "--motion",
        }
    }

    fn named(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|flag| flag.name() == text)
    }
}

impl fmt::Display for Flag {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

/// The words `--color` accepts.
const COLORS: [(&str, Choice<ColorDepth>); 5] = [
    ("auto", Choice::Auto),
    ("truecolor", Choice::Fixed(ColorDepth::TrueColor)),
    ("256", Choice::Fixed(ColorDepth::Ansi256)),
    ("16", Choice::Fixed(ColorDepth::Ansi16)),
    ("none", Choice::Fixed(ColorDepth::None)),
];

/// The words `--glyphs` accepts.
const GLYPHS: [(&str, Choice<GlyphTier>); 4] = [
    ("auto", Choice::Auto),
    ("emoji", Choice::Fixed(GlyphTier::Emoji)),
    ("symbols", Choice::Fixed(GlyphTier::Symbols)),
    ("ascii", Choice::Fixed(GlyphTier::Ascii)),
];

/// The words `--motion` accepts.
const MOTIONS: [(&str, Motion); 2] = [("on", Motion::On), ("off", Motion::Off)];

/// The words of a choice flag, joined for the usage text and error messages.
fn words<T>(choices: &[(&str, T)]) -> String {
    let names: Vec<&str> = choices.iter().map(|(name, _)| *name).collect();
    match names.split_last() {
        Some((last, [])) => (*last).to_owned(),
        Some((last, rest)) => format!("{} or {last}", rest.join(", ")),
        None => String::new(),
    }
}

/// Why the arguments were rejected.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum UsageError {
    /// An argument is not valid Unicode; it is shown with replacement characters.
    NotUnicode(String),
    /// An argument is not a known flag.
    Unknown(String),
    /// A flag was given twice.
    Repeated(Flag),
    /// A flag was the last argument, with no value after it.
    MissingValue(Flag),
    /// A flag's value is not a whole number in range.
    InvalidValue {
        /// The flag.
        flag: Flag,
        /// Why its value was rejected.
        source: InvalidNumber,
    },
    /// A flag's value is not one of its words.
    InvalidChoice {
        /// The flag.
        flag: Flag,
        /// The rejected value.
        value: String,
        /// The words it accepts.
        accepted: String,
    },
}

impl fmt::Display for UsageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotUnicode(shown) => write!(formatter, "argument {shown:?} is not valid Unicode"),
            Self::Unknown(argument) => write!(formatter, "unrecognised argument {argument:?}"),
            Self::Repeated(flag) => write!(formatter, "{flag} given more than once"),
            Self::MissingValue(flag) => write!(formatter, "{flag} needs a value"),
            Self::InvalidValue { flag, .. } => write!(formatter, "invalid value for {flag}"),
            Self::InvalidChoice {
                flag,
                value,
                accepted,
            } => write!(
                formatter,
                "invalid value for {flag}: {value:?} is not one of {accepted}"
            ),
        }
    }
}

impl Error for UsageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidValue { source, .. } => Some(source),
            Self::NotUnicode(_)
            | Self::Unknown(_)
            | Self::Repeated(_)
            | Self::MissingValue(_)
            | Self::InvalidChoice { .. } => None,
        }
    }
}

fn is_help(argument: &OsStr) -> bool {
    argument == "-h" || argument == "--help"
}

fn text(argument: &OsStr) -> Result<&str, UsageError> {
    argument
        .to_str()
        .ok_or_else(|| UsageError::NotUnicode(argument.to_string_lossy().into_owned()))
}

/// The choice `value` names among `choices`.
fn choose<T: Copy>(flag: Flag, value: &str, choices: &[(&str, T)]) -> Result<T, UsageError> {
    choices
        .iter()
        .find(|(name, _)| *name == value)
        .map(|(_, choice)| *choice)
        .ok_or_else(|| UsageError::InvalidChoice {
            flag,
            value: value.to_owned(),
            accepted: words(choices),
        })
}

/// Reads the arguments after the program name.
///
/// `-h` or `--help` anywhere asks for help, whatever else is there. Otherwise the
/// first wrong argument, in order, is the error.
///
/// # Errors
///
/// Fails on the first argument that is not Unicode, not a known flag, a repeated
/// flag, or a flag without a valid value.
pub(crate) fn parse(
    arguments: impl IntoIterator<Item = OsString>,
) -> Result<Invocation, UsageError> {
    let arguments: Vec<OsString> = arguments.into_iter().collect();
    if arguments.iter().any(|argument| is_help(argument)) {
        return Ok(Invocation::Help);
    }
    let mut options = Options::default();
    let mut seen = Vec::new();
    let mut remaining = arguments.iter();
    while let Some(argument) = remaining.next() {
        let argument = text(argument)?;
        let flag = Flag::named(argument).ok_or_else(|| UsageError::Unknown(argument.to_owned()))?;
        if seen.contains(&flag) {
            return Err(UsageError::Repeated(flag));
        }
        seen.push(flag);
        let value = text(remaining.next().ok_or(UsageError::MissingValue(flag))?)?;
        let invalid = |source| UsageError::InvalidValue { flag, source };
        let settings = &mut options.settings;
        match flag {
            Flag::Work => settings.work = value.parse().map_err(invalid)?,
            Flag::Short => settings.short_break = value.parse().map_err(invalid)?,
            Flag::Long => settings.long_break = value.parse().map_err(invalid)?,
            Flag::Every => settings.every = value.parse().map_err(invalid)?,
            Flag::Color => options.color = choose(flag, value, &COLORS)?,
            Flag::Glyphs => options.glyphs = choose(flag, value, &GLYPHS)?,
            Flag::Motion => options.motion = choose(flag, value, &MOTIONS)?,
        }
    }
    Ok(Invocation::Run(options))
}

/// The usage text, ending with `key_help`.
pub(crate) fn usage(key_help: &str) -> String {
    let defaults = Settings::default();
    let (low, high) = (RANGE.start(), RANGE.end());
    let (colors, glyphs, motions) = (words(&COLORS), words(&GLYPHS), words(&MOTIONS));
    format!(
        "usage: pomodoro [--work MIN] [--short MIN] [--long MIN] [--every N]\n\
         \x20               [--color WHEN] [--glyphs SET] [--motion ON]\n\
         \n\
         \x20 --work MIN     work phase length in minutes, {low} to {high} (default {work})\n\
         \x20 --short MIN    short break length in minutes, {low} to {high} (default {short})\n\
         \x20 --long MIN     long break length in minutes, {low} to {high} (default {long})\n\
         \x20 --every N      work phases before a long break, {low} to {high} (default {every})\n\
         \x20 --color WHEN   colours: {colors} (default auto)\n\
         \x20 --glyphs SET   pictures: {glyphs} (default auto)\n\
         \x20 --motion ON    animation: {motions} (default on)\n\
         \x20 -h, --help     print this help\n\
         \n\
         keys: {key_help}\n",
        work = defaults.work.get(),
        short = defaults.short_break.get(),
        long = defaults.long_break.get(),
        every = defaults.every.get(),
    )
}

#[cfg(test)]
mod tests;
