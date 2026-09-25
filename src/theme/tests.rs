use super::{
    ALERT, BACKGROUND, BORDER, ColorDepth, DIM, LONG_BREAK, SHORT_BREAK, SUCCESS, SURFACE, TEXT,
    TRACK, WORK, indexed_rgb, nearest_256, to_256,
};
use crate::environment::tests::terminal;
use ratatui::style::Color;

#[test]
fn color_depth_follows_environment() {
    let cases = [
        (
            terminal(
                &[("NO_COLOR", "1"), ("COLORTERM", "truecolor")],
                false,
                false,
            ),
            ColorDepth::None,
        ),
        (
            terminal(
                &[("TERM", "dumb"), ("COLORTERM", "truecolor")],
                false,
                false,
            ),
            ColorDepth::None,
        ),
        (
            terminal(&[("COLORTERM", "truecolor")], false, false),
            ColorDepth::TrueColor,
        ),
        (
            terminal(&[("COLORTERM", "24bit")], false, false),
            ColorDepth::TrueColor,
        ),
        (
            terminal(&[("WT_SESSION", "x"), ("TERM", "xterm")], false, false),
            ColorDepth::TrueColor,
        ),
        (terminal(&[], true, true), ColorDepth::TrueColor),
        (
            terminal(&[("TERM", "xterm-256color")], true, false),
            ColorDepth::Ansi256,
        ),
        (
            terminal(&[("TERM", "xterm-256color")], false, true),
            ColorDepth::Ansi256,
        ),
        (
            terminal(&[("TERM", "xterm")], false, false),
            ColorDepth::Ansi16,
        ),
        (
            terminal(&[("NO_COLOR", ""), ("TERM", "xterm")], false, false),
            ColorDepth::Ansi16,
        ),
    ];
    for (environment, expected) in cases {
        assert_eq!(
            ColorDepth::detect(&environment),
            expected,
            "{environment:?}"
        );
    }
}

/// WCAG 2.2 relative luminance of an sRGB colour.
fn luminance([red, green, blue]: [u8; 3]) -> f64 {
    let linear = |channel: u8| {
        let value = f64::from(channel) / 255.0;
        if value <= 0.040_45 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(red) + 0.7152 * linear(green) + 0.0722 * linear(blue)
}

/// WCAG 2.2 contrast ratio.
fn contrast(first: [u8; 3], second: [u8; 3]) -> f64 {
    let (one, two) = (luminance(first), luminance(second));
    (one.max(two) + 0.05) / (one.min(two) + 0.05)
}

/// The RGB a colour reaches the terminal as, in truecolor or the 256 tier.
fn shown(color: Color, depth: ColorDepth) -> [u8; 3] {
    let color = if depth == ColorDepth::Ansi256 {
        to_256(color)
    } else {
        color
    };
    if let Color::Indexed(index) = color {
        return indexed_rgb(index);
    }
    let Color::Rgb(red, green, blue) = color else {
        panic!("the palette is RGB, got {color:?}");
    };
    [red, green, blue]
}

#[test]
fn palette_meets_contrast_targets() {
    let text = 4.5;
    let border = 3.0;
    let pairs = [
        (TEXT, BACKGROUND, text),
        (TEXT, SURFACE, text),
        (TEXT, TRACK, text),
        (BACKGROUND, DIM, text),
        (DIM, BACKGROUND, text),
        (DIM, SURFACE, text),
        (WORK, SURFACE, text),
        (SHORT_BREAK, SURFACE, text),
        (LONG_BREAK, SURFACE, text),
        (ALERT, SURFACE, text),
        (SUCCESS, SURFACE, text),
        (WORK, BACKGROUND, text),
        (SHORT_BREAK, BACKGROUND, text),
        (LONG_BREAK, BACKGROUND, text),
        (ALERT, BACKGROUND, text),
        (TRACK, WORK, text),
        (TRACK, SHORT_BREAK, text),
        (TRACK, LONG_BREAK, text),
        (BACKGROUND, ALERT, text),
        (BORDER, BACKGROUND, border),
        (BORDER, SURFACE, border),
    ];
    for depth in [ColorDepth::TrueColor, ColorDepth::Ansi256] {
        for (foreground, background, target) in pairs {
            let ratio = contrast(shown(foreground, depth), shown(background, depth));
            assert!(
                ratio >= target,
                "{foreground:?} on {background:?} in {depth:?}: {ratio:.2} < {target}"
            );
        }
    }
}

#[test]
fn nearest_256_stays_out_of_the_theme_colours() {
    for red in (0_u8..=255).step_by(17) {
        for green in (0_u8..=255).step_by(51) {
            for blue in (0_u8..=255).step_by(85) {
                let index = nearest_256([red, green, blue]);
                assert!(index >= 16, "{red} {green} {blue} gave {index}");
            }
        }
    }
    assert_eq!(nearest_256([0, 0, 0]), 16);
    assert_eq!(nearest_256([255, 255, 255]), 231);
    assert_eq!(indexed_rgb(196), [255, 0, 0]);
    assert_eq!(indexed_rgb(244), [128, 128, 128]);
}
