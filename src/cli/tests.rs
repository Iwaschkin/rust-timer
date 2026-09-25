use super::{Invocation, parse};
use crate::glyphs::GlyphTier;
use crate::options::{Choice, Options};
use crate::settings::Settings;
use crate::theme::ColorDepth;
use std::ffi::OsString;

fn options(arguments: &[&str]) -> Options {
    let invocation = parse(arguments.iter().map(OsString::from));
    let Ok(Invocation::Run(options)) = invocation else {
        panic!("{arguments:?} should run, got {invocation:?}");
    };
    options
}

fn settings(arguments: &[&str]) -> Settings {
    options(arguments).settings
}

#[test]
fn accepts_bounds_and_leading_zeros() {
    assert_eq!(settings(&["--work", "1"]).work.get(), 1);
    assert_eq!(settings(&["--work", "99"]).work.get(), 99);
    assert_eq!(settings(&["--short", "1"]).short_break.get(), 1);
    assert_eq!(settings(&["--long", "99"]).long_break.get(), 99);
    assert_eq!(settings(&["--every", "1"]).every.get(), 1);
    assert_eq!(settings(&["--every", "99"]).every.get(), 99);
    assert_eq!(settings(&["--work", "025"]).work.get(), 25);
    let all = settings(&[
        "--every", "2", "--long", "20", "--short", "3", "--work", "50",
    ]);
    let chosen = (
        all.work.get(),
        all.short_break.get(),
        all.long_break.get(),
        all.every.get(),
    );
    assert_eq!(chosen, (50, 3, 20, 2));
    assert_eq!(settings(&[]), Settings::default());
}

#[test]
fn accepts_appearance_choices() {
    let colors = [
        ("auto", Choice::Auto),
        ("truecolor", Choice::Fixed(ColorDepth::TrueColor)),
        ("256", Choice::Fixed(ColorDepth::Ansi256)),
        ("16", Choice::Fixed(ColorDepth::Ansi16)),
        ("none", Choice::Fixed(ColorDepth::None)),
    ];
    for (word, expected) in colors {
        assert_eq!(options(&["--color", word]).color, expected, "{word}");
    }
    let glyphs = [
        ("auto", Choice::Auto),
        ("emoji", Choice::Fixed(GlyphTier::Emoji)),
        ("symbols", Choice::Fixed(GlyphTier::Symbols)),
        ("ascii", Choice::Fixed(GlyphTier::Ascii)),
    ];
    for (word, expected) in glyphs {
        assert_eq!(options(&["--glyphs", word]).glyphs, expected, "{word}");
    }
    let defaults = options(&[]);
    assert_eq!(
        (defaults.color, defaults.glyphs),
        (Choice::Auto, Choice::Auto)
    );
}
