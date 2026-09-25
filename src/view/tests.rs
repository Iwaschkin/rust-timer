use super::{remaining_label, render};
use crate::settings::Settings;
use crate::terminal::KEY_HELP;
use crate::timer::Timer;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use std::error::Error;
use std::time::{Duration, Instant};

/// The frame `render` draws for `timer` at `now`, one line per terminal row.
fn screen(width: u16, height: u16, timer: &Timer, now: Instant) -> Result<String, Box<dyn Error>> {
    let mut terminal = Terminal::new(TestBackend::new(width, height))?;
    terminal.draw(|frame| render(frame, timer, now, KEY_HELP))?;
    let rows = terminal
        .backend()
        .buffer()
        .content
        .chunks(usize::from(width))
        .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
        .collect::<Vec<_>>();
    Ok(rows.join("\n"))
}

/// Skips `count` phases ahead of a fresh timer.
fn skipped(count: usize, now: Instant) -> Timer {
    let mut timer = Timer::start(Settings::default(), now);
    for _ in 0..count {
        assert!(!timer.skip(now));
    }
    timer
}

#[test]
fn remaining_label_rounds_up_to_whole_seconds() {
    let cases = [
        (Duration::from_secs(25 * 60), "25:00"),
        (Duration::from_millis(24 * 60_000 + 59_001), "25:00"),
        (Duration::from_secs(24 * 60 + 59), "24:59"),
        (Duration::from_millis(1), "00:01"),
        (Duration::ZERO, "00:00"),
        (Duration::from_secs(99 * 60), "99:00"),
    ];
    for (remaining, expected) in cases {
        assert_eq!(remaining_label(remaining), expected, "{remaining:?}");
    }
}

#[test]
fn renders_in_tiny_terminals() -> Result<(), Box<dyn Error>> {
    let base = Instant::now();
    let timer = Timer::start(Settings::default(), base);
    for (width, height) in [(0, 0), (1, 1), (10, 3)] {
        let mut terminal = Terminal::new(TestBackend::new(width, height))?;
        terminal.draw(|frame| render(frame, &timer, base, KEY_HELP))?;
    }
    Ok(())
}

#[test]
fn frame_shows_phase_state_round_and_keys() -> Result<(), Box<dyn Error>> {
    let base = Instant::now();
    let mut timer = Timer::start(Settings::default(), base);
    let running = screen(60, 10, &timer, base)?;
    for expected in ["Work · Running", "Round 1 of 4", "25:00", KEY_HELP] {
        assert!(running.contains(expected), "{expected:?} in\n{running}");
    }
    assert!(!timer.toggle(base + Duration::from_secs(61)));
    let paused = screen(60, 10, &timer, base + Duration::from_secs(61))?;
    for expected in ["Work · Paused", "23:59"] {
        assert!(paused.contains(expected), "{expected:?} in\n{paused}");
    }
    Ok(())
}

#[test]
fn break_shows_round_it_follows() -> Result<(), Box<dyn Error>> {
    let base = Instant::now();
    let short = screen(60, 10, &skipped(3, base), base)?;
    for expected in ["Short break · Ready", "Round 2 of 4", "05:00"] {
        assert!(short.contains(expected), "{expected:?} in\n{short}");
    }
    let long = screen(60, 10, &skipped(7, base), base)?;
    for expected in ["Long break · Ready", "Round 4 of 4", "15:00"] {
        assert!(long.contains(expected), "{expected:?} in\n{long}");
    }
    Ok(())
}
