use super::{Choice, Options};
use crate::environment::tests::terminal;
use crate::glyphs::GlyphTier;
use crate::theme::ColorDepth;

#[test]
fn color_flag_overrides_environment() {
    let no_color = terminal(&[("NO_COLOR", "1")], false, false);
    let truecolor = terminal(&[("COLORTERM", "truecolor")], false, false);
    for depth in [
        ColorDepth::TrueColor,
        ColorDepth::Ansi256,
        ColorDepth::Ansi16,
        ColorDepth::None,
    ] {
        let options = Options {
            color: Choice::Fixed(depth),
            ..Options::default()
        };
        assert_eq!(options.appearance(&no_color).depth, depth);
        assert_eq!(options.appearance(&truecolor).depth, depth);
    }
    assert_eq!(
        Options::default().appearance(&no_color).depth,
        ColorDepth::None
    );
}

#[test]
fn glyph_flag_overrides_environment() {
    let linux_console = terminal(&[("TERM", "linux")], false, false);
    let options = Options {
        glyphs: Choice::Fixed(GlyphTier::Emoji),
        ..Options::default()
    };
    assert_eq!(options.appearance(&linux_console).glyphs, GlyphTier::Emoji);
    assert_eq!(
        Options::default().appearance(&linux_console).glyphs,
        GlyphTier::Ascii
    );
}
