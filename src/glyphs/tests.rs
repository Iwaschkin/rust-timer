use super::{Glyph, GlyphTier};
use crate::environment::tests::terminal;
use ratatui::text::Span;

const ALL: [Glyph; 9] = [
    Glyph::App,
    Glyph::Work,
    Glyph::ShortBreak,
    Glyph::LongBreak,
    Glyph::Running,
    Glyph::Paused,
    Glyph::Ready,
    Glyph::RoundDone,
    Glyph::RoundToDo,
];

#[test]
fn glyph_tier_follows_environment() {
    let cases = [
        (
            terminal(&[("TERM", "linux")], false, false),
            GlyphTier::Ascii,
        ),
        (terminal(&[("TERM", "dumb")], true, true), GlyphTier::Ascii),
        (terminal(&[], true, true), GlyphTier::Symbols),
        (
            terminal(&[("WT_SESSION", "x")], true, true),
            GlyphTier::Emoji,
        ),
        (
            terminal(&[("TERM_PROGRAM", "vscode")], true, true),
            GlyphTier::Emoji,
        ),
        (
            terminal(&[("TERM", "xterm-256color")], false, false),
            GlyphTier::Emoji,
        ),
        (terminal(&[], false, false), GlyphTier::Emoji),
    ];
    for (environment, expected) in cases {
        assert_eq!(GlyphTier::detect(&environment), expected, "{environment:?}");
    }
}

#[test]
fn emoji_glyphs_are_wide_and_single() {
    for role in ALL {
        let glyph = GlyphTier::Emoji.glyph(role);
        assert_eq!(
            glyph.chars().count(),
            1,
            "{role:?} {glyph:?} is one code point"
        );
        assert!(
            !glyph.contains(['\u{FE0F}', '\u{200D}']),
            "{role:?} {glyph:?}"
        );
        assert_eq!(
            Span::raw(glyph).width(),
            2,
            "{role:?} {glyph:?} is two cells"
        );
    }
}

#[test]
fn fallback_glyphs_are_narrow() {
    for tier in [GlyphTier::Symbols, GlyphTier::Ascii] {
        for role in ALL {
            let glyph = tier.glyph(role);
            assert_eq!(Span::raw(glyph).width(), 1, "{tier:?} {role:?} {glyph:?}");
        }
    }
    for role in ALL {
        assert!(GlyphTier::Ascii.glyph(role).is_ascii(), "{role:?}");
    }
}
