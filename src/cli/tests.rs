use super::parse;
use crate::glyphs::GlyphTier;
use crate::motion::Motion;
use crate::options::{Choice, Options};
use crate::settings::Settings;
use crate::theme::ColorDepth;
use std::error::Error;
use std::ffi::OsString;

fn options(arguments: &[&str]) -> Result<Options, Box<dyn Error>> {
    let command_line = ["pomodoro"].iter().chain(arguments).map(OsString::from);
    Ok(parse(command_line, "space start/pause")?)
}

/// The four numbers as the usage text shows them.
fn settings(arguments: &[&str]) -> Result<[String; 4], Box<dyn Error>> {
    let settings = options(arguments)?.settings;
    Ok([
        settings.work.to_string(),
        settings.short_break.to_string(),
        settings.long_break.to_string(),
        settings.every.to_string(),
    ])
}

#[test]
fn accepts_bounds_and_leading_zeros() -> Result<(), Box<dyn Error>> {
    let cases = [
        (&["--work", "1"][..], ["1", "5", "15", "4"]),
        (&["--work", "99"], ["99", "5", "15", "4"]),
        (&["--short", "1"], ["25", "1", "15", "4"]),
        (&["--long", "99"], ["25", "5", "99", "4"]),
        (&["--every", "1"], ["25", "5", "15", "1"]),
        (&["--every", "99"], ["25", "5", "15", "99"]),
        (&["--work", "025"], ["25", "5", "15", "4"]),
        (&["--work=50"], ["50", "5", "15", "4"]),
        (
            &[
                "--every", "2", "--long", "20", "--short", "3", "--work", "50",
            ],
            ["50", "3", "20", "2"],
        ),
    ];
    for (arguments, expected) in cases {
        assert_eq!(settings(arguments)?, expected, "{arguments:?}");
    }
    assert_eq!(options(&[])?.settings, Settings::default());
    Ok(())
}

#[test]
fn accepts_appearance_choices() -> Result<(), Box<dyn Error>> {
    let colors = [
        ("auto", Choice::Auto),
        ("truecolor", Choice::Fixed(ColorDepth::TrueColor)),
        ("256", Choice::Fixed(ColorDepth::Ansi256)),
        ("16", Choice::Fixed(ColorDepth::Ansi16)),
        ("none", Choice::Fixed(ColorDepth::None)),
    ];
    for (word, expected) in colors {
        assert_eq!(options(&["--color", word])?.color, expected, "{word}");
    }
    let glyphs = [
        ("auto", Choice::Auto),
        ("emoji", Choice::Fixed(GlyphTier::Emoji)),
        ("symbols", Choice::Fixed(GlyphTier::Symbols)),
        ("ascii", Choice::Fixed(GlyphTier::Ascii)),
    ];
    for (word, expected) in glyphs {
        assert_eq!(options(&["--glyphs", word])?.glyphs, expected, "{word}");
    }
    for (word, expected) in [("on", Motion::On), ("off", Motion::Off)] {
        assert_eq!(options(&["--motion", word])?.motion, expected, "{word}");
    }
    let defaults = options(&[])?;
    assert_eq!(
        (defaults.color, defaults.glyphs, defaults.motion),
        (Choice::Auto, Choice::Auto, Motion::On)
    );
    Ok(())
}
