//! Draws the timer: a header with the rounds, the phase panel, the keys, and the
//! Ready popup over everything when a phase waits.
//!
//! The panel has three layouts by window size. Large shows the braille dial beside
//! 8-cell block digits; medium shows 4-cell digits; compact shows the time as text.
//! Every layout has the gradient bar, and the larger two the cycle ribbon.
//!
//! Everything is drawn in the palette's 24-bit colours; the frame is fitted to the
//! terminal's colour tier afterwards.

mod bar;
mod clock;
mod dial;
mod popup;
mod ribbon;

use crate::glyphs::{Glyph, GlyphTier};
use crate::options::Appearance;
use crate::theme;
use crate::timer::{Phase, State, Timer};
use clock::Size;
use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Padding, Paragraph};
use std::time::{Duration, Instant};

/// More rounds than this are counted in words only.
const MOST_ROUND_GLYPHS: u8 = 8;

/// The panel's layout, chosen by window size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tier {
    /// At least 80 × 24: dial and large digits.
    Large,
    /// At least 48 × 16: medium digits.
    Medium,
    /// Anything smaller: the time as text.
    Compact,
}

impl Tier {
    fn of(area: Rect) -> Self {
        if area.width >= 80 && area.height >= 24 {
            Self::Large
        } else if area.width >= 48 && area.height >= 16 {
            Self::Medium
        } else {
            Self::Compact
        }
    }
}

/// Draws `timer` as it stands at `now`, with `keys` on the last line.
pub(crate) fn render(
    frame: &mut Frame<'_>,
    timer: &Timer,
    now: Instant,
    appearance: Appearance,
    keys: &[(&str, &str)],
) {
    let area = frame.area();
    let painted = Style::new().bg(theme::BACKGROUND).fg(theme::TEXT);
    frame.render_widget(Block::new().style(painted), area);
    let [header, panel, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(area);
    let tier = Tier::of(area);
    frame.render_widget(header_line(timer, appearance.glyphs), header);
    render_panel(frame, panel, timer, now, appearance.glyphs, tier);
    frame.render_widget(key_caps(keys), footer);
    if timer.state() == State::Ready {
        theme::veil(frame.buffer_mut());
        popup::render(frame, area, timer, appearance.glyphs, tier != Tier::Compact);
    }
}

fn header_line(timer: &Timer, glyphs: GlyphTier) -> Paragraph<'static> {
    let (round, rounds) = (timer.round(), timer.rounds());
    let mut spans = vec![
        Span::styled(
            format!(" {} pomodoro ", glyphs.glyph(Glyph::App)),
            Style::new().fg(theme::TEXT).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
    ];
    if rounds <= MOST_ROUND_GLYPHS {
        let done = if timer.phase() == Phase::Work {
            round.saturating_sub(1)
        } else {
            round
        };
        for position in 1..=rounds {
            let glyph = if position <= done {
                Glyph::RoundDone
            } else {
                Glyph::RoundToDo
            };
            spans.push(Span::raw(glyphs.glyph(glyph)));
        }
        spans.push(Span::raw(" "));
    }
    spans.push(Span::styled(
        format!("Round {round} of {rounds}"),
        Style::new().fg(theme::DIM),
    ));
    Paragraph::new(Line::from(spans))
}

fn render_panel(
    frame: &mut Frame<'_>,
    area: Rect,
    timer: &Timer,
    now: Instant,
    glyphs: GlyphTier,
    tier: Tier,
) {
    let phase = timer.phase();
    let accent = theme::accent(phase);
    let state = timer.state();
    let title = Span::styled(
        format!(
            " {} {} ",
            glyphs.glyph(phase_glyph(phase)),
            phase_label(phase)
        ),
        Style::new().fg(accent).add_modifier(Modifier::BOLD),
    );
    let status = Span::styled(
        format!(
            " {} {} ",
            glyphs.glyph(state_glyph(state)),
            state_label(state)
        ),
        Style::new()
            .fg(state_color(state))
            .add_modifier(Modifier::BOLD),
    );
    let block = Block::bordered()
        .border_type(border_type(phase))
        .border_style(Style::new().fg(accent))
        .style(Style::new().bg(theme::SURFACE).fg(theme::TEXT))
        .title_top(Line::from(title).left_aligned())
        .title_top(Line::from(status).right_aligned())
        .padding(Padding::horizontal(2));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let clock_color = if state == State::Paused {
        theme::faded(accent)
    } else {
        accent
    };
    let time = remaining_label(timer.remaining(now));
    let hero_height = match tier {
        Tier::Large => 10,
        Tier::Medium => Size::Medium.cells().1,
        Tier::Compact => 1,
    };
    let [_, badge, _, hero, _, bar, label, _, ribbon, _] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(u16::from(tier != Tier::Compact)),
        Constraint::Length(u16::from(tier != Tier::Compact)),
        Constraint::Length(hero_height),
        Constraint::Length(u16::from(tier != Tier::Compact)),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(u16::from(tier != Tier::Compact)),
        Constraint::Length(u16::from(tier != Tier::Compact)),
        Constraint::Fill(1),
    ])
    .areas(inner);
    if state == State::Paused {
        let paused = Span::styled(
            format!(" {} PAUSED ", glyphs.glyph(Glyph::Paused)),
            Style::new()
                .bg(theme::DIM)
                .fg(theme::BACKGROUND)
                .add_modifier(Modifier::BOLD),
        );
        frame.render_widget(Paragraph::new(Line::from(paused)).centered(), badge);
    }
    match tier {
        Tier::Large => {
            let [dial_area, _, digits] = Layout::horizontal([
                Constraint::Length(24),
                Constraint::Length(4),
                Constraint::Length(Size::Large.cells().0 * 5),
            ])
            .flex(Flex::Center)
            .areas(hero);
            let glyph = glyphs.glyph(phase_glyph(phase));
            dial::render(frame, dial_area, timer.progress(now), accent, glyph);
            let [_, digits, _] = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(Size::Large.cells().1),
                Constraint::Fill(1),
            ])
            .areas(digits);
            clock::render(frame, digits, &time, clock_color, Size::Large);
        }
        Tier::Medium => clock::render(frame, hero, &time, clock_color, Size::Medium),
        Tier::Compact => {
            let clock = Paragraph::new(time)
                .style(Style::new().fg(clock_color).add_modifier(Modifier::BOLD))
                .centered();
            frame.render_widget(clock, hero);
        }
    }
    let progress = timer.progress(now);
    bar::render(frame.buffer_mut(), bar, progress, accent);
    frame.render_widget(bar_label(timer, now), label);
    let segments = timer.cycle();
    if !ribbon::render(
        frame.buffer_mut(),
        ribbon,
        &segments,
        timer.position(),
        progress,
    ) {
        let words = format!("Round {} of {}", timer.round(), timer.rounds());
        frame.render_widget(
            Paragraph::new(words)
                .style(Style::new().fg(theme::DIM))
                .centered(),
            ribbon,
        );
    }
}

/// The line under the bar: elapsed time out of the phase length.
fn bar_label(timer: &Timer, now: Instant) -> Paragraph<'static> {
    let elapsed = remaining_label(timer.progress(now).elapsed());
    let total = remaining_label(timer.length());
    Paragraph::new(Line::from(vec![Span::styled(
        format!("{elapsed} of {total}"),
        Style::new().fg(theme::DIM),
    )]))
    .centered()
}

fn key_caps(keys: &[(&str, &str)]) -> Paragraph<'static> {
    let mut spans = Vec::new();
    for (key, action) in keys {
        spans.push(Span::styled(
            format!(" {key} "),
            Style::new()
                .bg(theme::TRACK)
                .fg(theme::TEXT)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!(" {action}  "),
            Style::new().fg(theme::DIM),
        ));
    }
    Paragraph::new(Line::from(spans)).centered()
}

fn phase_label(phase: Phase) -> &'static str {
    match phase {
        Phase::Work => "Work",
        Phase::ShortBreak => "Short break",
        Phase::LongBreak => "Long break",
    }
}

fn phase_glyph(phase: Phase) -> Glyph {
    match phase {
        Phase::Work => Glyph::Work,
        Phase::ShortBreak => Glyph::ShortBreak,
        Phase::LongBreak => Glyph::LongBreak,
    }
}

/// Each phase has its own border as well as its own colour, so colour is never the
/// only difference (P08).
fn border_type(phase: Phase) -> BorderType {
    match phase {
        Phase::Work => BorderType::Thick,
        Phase::ShortBreak => BorderType::Rounded,
        Phase::LongBreak => BorderType::Double,
    }
}

fn state_label(state: State) -> &'static str {
    match state {
        State::Running => "Running",
        State::Paused => "Paused",
        State::Ready => "Ready",
    }
}

fn state_glyph(state: State) -> Glyph {
    match state {
        State::Running => Glyph::Running,
        State::Paused => Glyph::Paused,
        State::Ready => Glyph::Ready,
    }
}

fn state_color(state: State) -> Color {
    match state {
        State::Running => theme::SUCCESS,
        State::Paused => theme::DIM,
        State::Ready => theme::ALERT,
    }
}

/// `remaining` as `MM:SS`, rounded up to the next whole second.
fn remaining_label(remaining: Duration) -> String {
    let whole = remaining.as_secs();
    let seconds = if remaining.subsec_nanos() > 0 {
        whole.saturating_add(1)
    } else {
        whole
    };
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}

#[cfg(test)]
mod tests;
