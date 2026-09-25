//! Large block letters: the time left, and a phase's name in the Ready popup.
//!
//! This is the only module that uses tui-big-text. Its one font is 8 × 8; the
//! sizes here use full or quadrant blocks, which every target terminal draws.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use tui_big_text::{BigText, PixelSize};

/// How large the letters are.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Size {
    /// 8 × 8 cells a letter.
    Large,
    /// 4 × 4 cells a letter.
    Medium,
}

impl Size {
    /// Cells a letter takes, across and down.
    pub(super) fn cells(self) -> (u16, u16) {
        match self {
            Self::Large => (8, 8),
            Self::Medium => (4, 4),
        }
    }
}

/// Draws `text` centred in `area`, in `color`.
pub(super) fn render(frame: &mut Frame<'_>, area: Rect, text: &str, color: Color, size: Size) {
    let pixel_size = match size {
        Size::Large => PixelSize::Full,
        Size::Medium => PixelSize::Quadrant,
    };
    let letters = BigText::builder()
        .pixel_size(pixel_size)
        .style(Style::new().fg(color))
        .lines(vec![Line::from(text.to_owned())])
        .centered()
        .build();
    frame.render_widget(letters, area);
}
