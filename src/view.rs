//! Draws the timer: the phase and its state, the round, the progress bar with the
//! time left, and the keys.

use crate::timer::{Phase, State, Timer};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::widgets::{Block, Gauge, Paragraph};
use std::time::{Duration, Instant};

/// Draws `timer` as it stands at `now`, with `key_help` on the last line.
pub(crate) fn render(frame: &mut Frame<'_>, timer: &Timer, now: Instant, key_help: &str) {
    let [title, round, bar, help] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(1),
    ])
    .areas(frame.area());
    let heading = format!(
        "{} · {}",
        phase_label(timer.phase()),
        state_label(timer.state())
    );
    frame.render_widget(Paragraph::new(heading), title);
    let position = format!("Round {} of {}", timer.round(), timer.rounds());
    frame.render_widget(Paragraph::new(position), round);
    let gauge = Gauge::default()
        .block(Block::bordered())
        .ratio(timer.progress(now).ratio())
        .label(remaining_label(timer.remaining(now)));
    frame.render_widget(gauge, bar);
    frame.render_widget(Paragraph::new(key_help), help);
}

fn phase_label(phase: Phase) -> &'static str {
    match phase {
        Phase::Work => "Work",
        Phase::ShortBreak => "Short break",
        Phase::LongBreak => "Long break",
    }
}

fn state_label(state: State) -> &'static str {
    match state {
        State::Running => "Running",
        State::Paused => "Paused",
        State::Ready => "Ready",
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
