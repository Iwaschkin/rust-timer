//! The Ready popup: when a phase ends, the next one is announced over the dimmed
//! screen, in a thick amber frame with a drop shadow, until space starts it.

use super::clock::{self, Size};
use super::{key_caps, phase_glyph, phase_label};
use crate::glyphs::{Glyph, GlyphTier};
use crate::theme;
use crate::timer::{Phase, Timer};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Offset, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, Paragraph, Shadow};

/// The keys the popup offers.
const HINT: [(&str, &str); 2] = [("space", "start"), ("s", "skip")];

/// Draws the popup for the phase `timer` has loaded, centred in `area`; with
/// `letters`, the phase's name is in block letters too.
pub(super) fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    timer: &Timer,
    glyphs: GlyphTier,
    letters: bool,
) {
    let phase = timer.phase();
    let accent = theme::accent(phase);
    let height = if letters { 12 } else { 6 };
    let popup = centred(area, 46, height);
    if popup.is_empty() {
        return;
    }
    let title = Span::styled(
        format!(" {} Time's up ", glyphs.glyph(Glyph::Ready)),
        Style::new().fg(theme::ALERT).add_modifier(Modifier::BOLD),
    );
    let block = Block::bordered()
        .border_type(BorderType::Thick)
        .border_style(Style::new().fg(theme::ALERT))
        .style(Style::new().bg(theme::SURFACE).fg(theme::TEXT))
        .title_top(Line::from(title).centered())
        .shadow(
            Shadow::overlay()
                .style(Style::new().bg(theme::SHADOW).fg(theme::SHADOW))
                .offset(Offset::new(2, 1)),
        );
    let inner = block.inner(popup);
    frame.render_widget(Clear, popup);
    frame.render_widget(block, popup);
    let [word, _, announcement, detail, _, keys] = Layout::vertical([
        Constraint::Length(if letters { 4 } else { 0 }),
        Constraint::Length(u16::from(letters)),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(inner);
    if letters {
        clock::render(frame, word, name(phase), accent, Size::Medium);
    }
    let announced = Line::from(vec![
        Span::raw(format!("{} ", glyphs.glyph(phase_glyph(phase)))),
        Span::styled(
            format!("{} is ready", phase_label(phase)),
            Style::new().fg(accent).add_modifier(Modifier::BOLD),
        ),
    ]);
    frame.render_widget(Paragraph::new(announced).centered(), announcement);
    let minutes = timer.length().as_secs() / 60;
    let about = format!(
        "{minutes} min · round {} of {}",
        timer.round(),
        timer.rounds()
    );
    frame.render_widget(
        Paragraph::new(about)
            .style(Style::new().fg(theme::DIM))
            .centered(),
        detail,
    );
    frame.render_widget(key_caps(&HINT), keys);
}

/// The phase's one-word name, for block letters.
fn name(phase: Phase) -> &'static str {
    match phase {
        Phase::Work => "WORK",
        Phase::ShortBreak => "BREAK",
        Phase::LongBreak => "REST",
    }
}

/// A `width` × `height` area centred in `area`, shrunk to fit.
fn centred(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width.saturating_sub(2));
    let height = height.min(area.height.saturating_sub(1));
    Rect::new(
        area.x.saturating_add((area.width - width) / 2),
        area.y.saturating_add((area.height - height) / 2),
        width,
        height,
    )
}
