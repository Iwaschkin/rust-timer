//! The progress bar: it fills in eighths of a cell, and its colour ramps from a
//! faded shade of the phase colour at the left to the full colour at the leading
//! edge, so the edge always shows the phase in full.

use crate::theme;
use crate::timer::Progress;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::Color;

/// Draws the bar across every row of `area`.
pub(super) fn render(buffer: &mut Buffer, area: Rect, progress: Progress, accent: Color) {
    let width = u32::from(area.width);
    let filled = progress.scaled(width * 8);
    let lit_columns = filled.div_ceil(8);
    let start = theme::faded(accent);
    for (column, x) in (0..width).zip(area.left()..area.right()) {
        let color = theme::mix(start, accent, ramp(column, lit_columns));
        let eighths = filled.saturating_sub(column * 8).min(8);
        let symbol = eighth_block(eighths);
        for y in area.top()..area.bottom() {
            if let Some(cell) = buffer.cell_mut(Position::new(x, y)) {
                cell.set_symbol(symbol).set_fg(color).set_bg(theme::TRACK);
            }
        }
    }
}

/// The left-aligned block that fills `eighths` eighths of a cell; 8 or more is full.
pub(super) fn eighth_block(eighths: u32) -> &'static str {
    match eighths {
        0 => " ",
        1 => "▏",
        2 => "▎",
        3 => "▍",
        4 => "▌",
        5 => "▋",
        6 => "▊",
        7 => "▉",
        _ => "█",
    }
}

/// How far along the ramp `column` sits, out of 255, when `span` columns are lit.
fn ramp(column: u32, span: u32) -> u8 {
    let last = span.saturating_sub(1).max(1);
    u8::try_from(column.min(last) * 255 / last).unwrap_or(u8::MAX)
}
