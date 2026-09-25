use super::{remaining_label, render};
use crate::settings::Settings;
use crate::timer::Timer;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use std::error::Error;
use std::time::{Duration, Instant};

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
        terminal.draw(|frame| render(frame, &timer, base, "q quit"))?;
    }
    Ok(())
}
