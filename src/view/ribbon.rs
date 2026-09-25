//! The cycle ribbon: one segment per phase of the cycle, as wide as the phase is
//! long. Passed phases are filled, the current one fills as it runs, and the rest
//! show the track.

use super::bar;
use crate::theme;
use crate::timer::{Progress, Segment};
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};

/// Each segment's width in cells, with one cell between segments, or `None` when
/// `width` cannot give every segment at least one cell.
pub(super) fn widths(
    segments: impl Iterator<Item = Segment> + Clone,
    width: u16,
) -> Option<impl Iterator<Item = u16> + Clone> {
    let count = u16::try_from(segments.clone().count()).ok()?;
    let available = u64::from(width.checked_sub(count.checked_sub(1)?)?);
    let total: u64 = segments
        .clone()
        .map(|segment| segment.length.as_secs())
        .sum();
    // Where each segment ends, scaled to the cells available, then the gaps between.
    let edges = segments.scan(0_u64, move |passed, segment| {
        *passed += segment.length.as_secs();
        Some(*passed * available / total.max(1))
    });
    let widths = edges.scan(0_u64, |previous, edge| {
        let cells = u16::try_from(edge - *previous).ok()?;
        *previous = edge;
        Some(cells)
    });
    let whole = widths.clone().filter(|cells| *cells > 0).count();
    (whole == usize::from(count)).then_some(widths)
}

/// Draws the ribbon on the first row of `area`, with the phase at `position`
/// current and `progress` through it.
pub(super) fn render(
    buffer: &mut Buffer,
    area: Rect,
    segments: impl Iterator<Item = Segment> + Clone,
    position: usize,
    progress: Progress,
) -> bool {
    let Some(widths) = widths(segments.clone(), area.width) else {
        return false;
    };
    let mut x = area.left();
    for (index, (segment, cells)) in segments.zip(widths).enumerate() {
        let accent = theme::accent(segment.phase);
        let lit = progress.scaled(u32::from(cells) * 8);
        for column in 0..cells {
            let (symbol, color) = if index < position {
                ("█", theme::faded(accent))
            } else if index > position {
                (" ", theme::TRACK)
            } else {
                let eighths = lit.saturating_sub(u32::from(column) * 8).min(8);
                (bar::eighth_block(eighths), accent)
            };
            if let Some(cell) = buffer.cell_mut(Position::new(x.saturating_add(column), area.top()))
            {
                cell.set_symbol(symbol).set_fg(color).set_bg(theme::TRACK);
            }
        }
        x = x.saturating_add(cells).saturating_add(1);
    }
    true
}
