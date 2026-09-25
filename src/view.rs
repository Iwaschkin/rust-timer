//! Draws the timer: a header with the rounds, the phase panel with its time and
//! bar, and the keys.
//!
//! Everything is drawn in the palette's 24-bit colours; the frame is fitted to the
//! terminal's colour tier afterwards.

use crate::glyphs::{Glyph, GlyphTier};
use crate::options::Appearance;
use crate::theme;
use crate::timer::{Phase, State, Timer};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Gauge, Padding, Paragraph};
use std::time::{Duration, Instant};

/// More rounds than this are counted in words only.
const MOST_ROUND_GLYPHS: u8 = 8;

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
    frame.render_widget(header_line(timer, appearance.glyphs), header);
    render_panel(frame, panel, timer, now, appearance.glyphs);
    frame.render_widget(key_caps(keys), footer);
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

fn render_panel(frame: &mut Frame<'_>, area: Rect, timer: &Timer, now: Instant, glyphs: GlyphTier) {
    let phase = timer.phase();
    let accent = theme::accent(phase);
    let title = Span::styled(
        format!(
            " {} {} ",
            glyphs.glyph(phase_glyph(phase)),
            phase_label(phase)
        ),
        Style::new().fg(accent).add_modifier(Modifier::BOLD),
    );
    let state = timer.state();
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
    let [_, time, _, bar, _] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(inner);
    let clock = Paragraph::new(remaining_label(timer.remaining(now)))
        .style(Style::new().fg(accent).add_modifier(Modifier::BOLD))
        .centered();
    frame.render_widget(clock, time);
    let gauge = Gauge::default()
        .gauge_style(Style::new().fg(accent).bg(theme::TRACK))
        .use_unicode(true)
        .ratio(timer.progress(now).ratio())
        .label(format!("{:.0}%", timer.progress(now).ratio() * 100.0));
    frame.render_widget(gauge, bar);
}

fn key_caps(keys: &[(&str, &str)]) -> Paragraph<'static> {
    let mut spans = Vec::new();
    for (key, action) in keys {
        spans.push(Span::styled(
            format!(" {key} "),
            Style::new()
                .bg(theme::SURFACE)
                .fg(theme::TEXT)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!(" {action}   "),
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
