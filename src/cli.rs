//! The command line: the flags and their choices, declared for clap, which writes
//! the usage text and the argument errors.
//!
//! This is the only module that names the flags and the only one that uses clap.
//! A length keeps its own check: clap reads it through [`Minutes`]'s parser, so its
//! range and defaults are stated once, in `settings`.

use crate::glyphs::GlyphTier;
use crate::motion::Motion;
use crate::options::{Choice, Options};
use crate::settings::{Every, Minutes, RANGE, Settings};
use crate::theme::ColorDepth;
use clap::{CommandFactory, FromArgMatches, Parser, ValueEnum};
use std::ffi::OsString;

/// The flags, as clap reads them. The numbers accept a leading `-`, so `-1` is a
/// value that fails its range check rather than an unknown flag.
#[derive(Debug, Parser)]
#[command(bin_name = "pomodoro", about = "A pomodoro timer for the terminal.")]
struct Arguments {
    #[arg(
        long,
        value_name = "MIN",
        allow_negative_numbers = true,
        help = within("Work phase length in minutes"),
        default_value_t = Settings::default().work
    )]
    work: Minutes,
    #[arg(
        long = "short",
        value_name = "MIN",
        allow_negative_numbers = true,
        help = within("Short break length in minutes"),
        default_value_t = Settings::default().short_break
    )]
    short_break: Minutes,
    #[arg(
        long = "long",
        value_name = "MIN",
        allow_negative_numbers = true,
        help = within("Long break length in minutes"),
        default_value_t = Settings::default().long_break
    )]
    long_break: Minutes,
    #[arg(
        long,
        value_name = "N",
        allow_negative_numbers = true,
        help = within("Work phases before a long break"),
        default_value_t = Settings::default().every
    )]
    every: Every,
    /// How many colours to draw with; auto asks the terminal
    #[arg(long, value_name = "WHEN", value_enum, default_value_t = Colors::Auto)]
    color: Colors,
    /// Pictures for the phases and states; auto asks the terminal
    #[arg(long, value_name = "SET", value_enum, default_value_t = Pictures::Auto)]
    glyphs: Pictures,
    /// Animated effects
    #[arg(long, value_name = "ON", value_enum, default_value_t = Animation::On)]
    motion: Animation,
}

/// A setting's help, with the range every setting accepts.
fn within(what: &str) -> String {
    format!("{what}, {} to {}", RANGE.start(), RANGE.end())
}

/// The words `--color` accepts.
#[derive(Clone, Copy, Debug, ValueEnum)]
enum Colors {
    Auto,
    Truecolor,
    #[value(name = "256")]
    Ansi256,
    #[value(name = "16")]
    Ansi16,
    None,
}

/// The words `--glyphs` accepts.
#[derive(Clone, Copy, Debug, ValueEnum)]
enum Pictures {
    Auto,
    Emoji,
    Symbols,
    Ascii,
}

/// The words `--motion` accepts.
#[derive(Clone, Copy, Debug, ValueEnum)]
enum Animation {
    On,
    Off,
}

/// Reads the command line, program name first, with `key_help` after the flags in
/// the usage text.
///
/// # Errors
///
/// Returns clap's error for the first wrong argument, in order. A request for help
/// is returned the same way: clap prints it to stdout, with a success status.
pub(crate) fn parse(
    arguments: impl IntoIterator<Item = OsString>,
    key_help: &str,
) -> Result<Options, clap::Error> {
    let command = Arguments::command().after_help(format!("Keys: {key_help}"));
    let arguments = Arguments::from_arg_matches(&command.try_get_matches_from(arguments)?)?;
    Ok(Options {
        settings: Settings {
            work: arguments.work,
            short_break: arguments.short_break,
            long_break: arguments.long_break,
            every: arguments.every,
        },
        color: match arguments.color {
            Colors::Auto => Choice::Auto,
            Colors::Truecolor => Choice::Fixed(ColorDepth::TrueColor),
            Colors::Ansi256 => Choice::Fixed(ColorDepth::Ansi256),
            Colors::Ansi16 => Choice::Fixed(ColorDepth::Ansi16),
            Colors::None => Choice::Fixed(ColorDepth::None),
        },
        glyphs: match arguments.glyphs {
            Pictures::Auto => Choice::Auto,
            Pictures::Emoji => Choice::Fixed(GlyphTier::Emoji),
            Pictures::Symbols => Choice::Fixed(GlyphTier::Symbols),
            Pictures::Ascii => Choice::Fixed(GlyphTier::Ascii),
        },
        motion: match arguments.motion {
            Animation::On => Motion::On,
            Animation::Off => Motion::Off,
        },
    })
}

#[cfg(test)]
mod tests;
