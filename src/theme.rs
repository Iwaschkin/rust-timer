//! The palette, "Tomato Night", and how a frame is fitted to the terminal's colours.
//!
//! Everything is designed in 24-bit colour. The last step of every frame fits each
//! cell's colours to the tier the terminal shows. This is the only module with RGB
//! values; the research behind them is `docs/research/visual-overhaul.md`, section 6.

use crate::environment::Environment;
use crate::timer::Phase;
use ratatui::buffer::Buffer;
use ratatui::style::Color;

/// Painted behind every cell.
pub(crate) const BACKGROUND: Color = Color::from_u32(0x0012_141C);
/// Behind panels and popups.
pub(crate) const SURFACE: Color = Color::from_u32(0x001A_1E2A);
/// The empty part of a bar or dial.
pub(crate) const TRACK: Color = Color::from_u32(0x0026_2B3B);
/// Panel borders.
pub(crate) const BORDER: Color = Color::from_u32(0x0064_6D8A);
/// Primary text.
pub(crate) const TEXT: Color = Color::from_u32(0x00E6_E9F2);
/// Secondary text: labels and key help.
pub(crate) const DIM: Color = Color::from_u32(0x009A_A3B8);
/// Work: tomato.
pub(crate) const WORK: Color = Color::from_u32(0x00FF_6B57);
/// Short break: mint.
pub(crate) const SHORT_BREAK: Color = Color::from_u32(0x003D_DBD9);
/// Long break: lavender.
pub(crate) const LONG_BREAK: Color = Color::from_u32(0x00B6_9CFF);
/// The phase-end alert: amber.
pub(crate) const ALERT: Color = Color::from_u32(0x00FF_C94D);
/// Done: green.
pub(crate) const SUCCESS: Color = Color::from_u32(0x007B_D88F);

/// The colour of `phase`.
pub(crate) fn accent(phase: Phase) -> Color {
    match phase {
        Phase::Work => WORK,
        Phase::ShortBreak => SHORT_BREAK,
        Phase::LongBreak => LONG_BREAK,
    }
}

/// How many colours the terminal shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ColorDepth {
    /// 24-bit colour.
    TrueColor,
    /// The xterm 256-colour palette.
    Ansi256,
    /// The 16 colours of the user's theme.
    Ansi16,
    /// No colour: the terminal's own foreground and background.
    None,
}

impl ColorDepth {
    /// The tier the environment describes (P01).
    pub(crate) fn detect(environment: &Environment) -> Self {
        if environment.no_color() || environment.dumb() {
            Self::None
        } else if environment.announces_truecolor()
            || environment.windows_terminal()
            || environment.console_truecolor()
        {
            Self::TrueColor
        } else if environment.announces_256() {
            Self::Ansi256
        } else {
            Self::Ansi16
        }
    }
}

/// Fits every colour in `buffer` to `depth`. The last step of every frame.
///
/// In tier none every colour becomes `Reset`, so the terminal's own colours show and
/// no colour code is written. Left to crossterm's own no-colour handling instead, each
/// colour change would write an attribute reset that drops bold (research 3.1).
pub(crate) fn quantize(buffer: &mut Buffer, depth: ColorDepth) {
    if depth == ColorDepth::TrueColor {
        return;
    }
    for cell in &mut buffer.content {
        cell.fg = fit(cell.fg, depth);
        cell.bg = fit(cell.bg, depth);
        cell.underline_color = fit(cell.underline_color, depth);
    }
}

fn fit(color: Color, depth: ColorDepth) -> Color {
    match depth {
        ColorDepth::TrueColor => color,
        ColorDepth::Ansi256 => to_256(color),
        ColorDepth::Ansi16 => to_16(color),
        ColorDepth::None => Color::Reset,
    }
}

/// The palette's own 256-colour and 16-colour choices, which keep its contrast
/// (research 6.5 and 6.6). Other colours, from gradients and effects, take the
/// nearest match.
const PALETTE: [(Color, u8, Color); 11] = [
    (BACKGROUND, 233, Color::Black),
    (SURFACE, 234, Color::Black),
    (TRACK, 235, Color::Black),
    (BORDER, 243, Color::DarkGray),
    (TEXT, 255, Color::White),
    (DIM, 248, Color::Gray),
    (WORK, 203, Color::LightRed),
    (SHORT_BREAK, 80, Color::LightCyan),
    (LONG_BREAK, 147, Color::LightBlue),
    (ALERT, 221, Color::LightYellow),
    (SUCCESS, 114, Color::LightGreen),
];

fn to_256(color: Color) -> Color {
    let Color::Rgb(red, green, blue) = color else {
        return color;
    };
    if let Some((_, index, _)) = PALETTE.iter().find(|(designed, _, _)| *designed == color) {
        return Color::Indexed(*index);
    }
    Color::Indexed(nearest_256([red, green, blue]))
}

fn to_16(color: Color) -> Color {
    let rgb = match color {
        Color::Rgb(red, green, blue) => [red, green, blue],
        Color::Indexed(index) => indexed_rgb(index),
        Color::Reset
        | Color::Black
        | Color::Red
        | Color::Green
        | Color::Yellow
        | Color::Blue
        | Color::Magenta
        | Color::Cyan
        | Color::Gray
        | Color::DarkGray
        | Color::LightRed
        | Color::LightGreen
        | Color::LightYellow
        | Color::LightBlue
        | Color::LightMagenta
        | Color::LightCyan
        | Color::White => return color,
    };
    if let Some((_, _, named)) = PALETTE.iter().find(|(designed, _, _)| *designed == color) {
        return *named;
    }
    ANSI_16
        .iter()
        .min_by_key(|(_, reference)| distance(rgb, *reference))
        .map_or(Color::Reset, |(named, _)| *named)
}

/// The six channel levels of the xterm colour cube, indices 16 to 231.
const CUBE: [u8; 6] = [0, 95, 135, 175, 215, 255];

/// The nearest xterm colour from the cube or the grey ramp, indices 16 to 255,
/// which the user's theme does not redefine.
fn nearest_256(rgb: [u8; 3]) -> u8 {
    let level = |channel: u8| -> u8 {
        (0_u8..6)
            .zip(CUBE)
            .min_by_key(|(_, value)| value.abs_diff(channel))
            .map_or(0, |(position, _)| position)
    };
    let [red, green, blue] = rgb.map(level);
    let cube = 16 + 36 * red + 6 * green + blue;
    let grey = (0_u8..24)
        .min_by_key(|step| distance(rgb, [8 + 10 * step; 3]))
        .unwrap_or(0);
    if distance(rgb, indexed_rgb(cube)) <= distance(rgb, [8 + 10 * grey; 3]) {
        cube
    } else {
        232 + grey
    }
}

/// The RGB value xterm gives a 256-colour index. The first 16 use the reference
/// colours below; the user's theme may differ.
pub(crate) fn indexed_rgb(index: u8) -> [u8; 3] {
    match index {
        0..=15 => ANSI_16
            .get(usize::from(index))
            .map_or([0; 3], |(_, rgb)| *rgb),
        16..=231 => {
            let offset = index - 16;
            let channel = |step: u8| CUBE.get(usize::from(step)).copied().unwrap_or(0);
            [
                channel(offset / 36),
                channel(offset / 6 % 6),
                channel(offset % 6),
            ]
        }
        232..=255 => [8 + 10 * (index - 232); 3],
    }
}

/// Reference RGB values for the 16 named colours: Windows Terminal's Campbell
/// scheme, used only to choose the nearest one.
const ANSI_16: [(Color, [u8; 3]); 16] = [
    (Color::Black, [12, 12, 12]),
    (Color::Red, [197, 15, 31]),
    (Color::Green, [19, 161, 14]),
    (Color::Yellow, [193, 156, 0]),
    (Color::Blue, [0, 55, 218]),
    (Color::Magenta, [136, 23, 152]),
    (Color::Cyan, [58, 150, 221]),
    (Color::Gray, [204, 204, 204]),
    (Color::DarkGray, [118, 118, 118]),
    (Color::LightRed, [231, 72, 86]),
    (Color::LightGreen, [22, 198, 12]),
    (Color::LightYellow, [249, 241, 165]),
    (Color::LightBlue, [59, 120, 255]),
    (Color::LightMagenta, [180, 0, 158]),
    (Color::LightCyan, [97, 214, 214]),
    (Color::White, [242, 242, 242]),
];

/// Squared distance between two RGB colours.
fn distance(left: [u8; 3], right: [u8; 3]) -> u32 {
    left.iter()
        .zip(right)
        .map(|(left, right)| u32::from(left.abs_diff(right)).pow(2))
        .sum()
}

#[cfg(test)]
mod tests;
