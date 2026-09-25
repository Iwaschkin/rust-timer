use super::{PROGRESS_REFRESH, Signals, taskbar};
use crate::glyphs::GlyphTier;
use crate::settings::Settings;
use crate::terminal::{Taskbar, progress_sequence};
use crate::timer::Timer;
use crate::view::window_title;
use std::time::{Duration, Instant};

const WORK: Duration = Duration::from_secs(25 * 60);

#[test]
fn title_follows_timer() {
    let base = Instant::now();
    let mut timer = Timer::start(Settings::default(), base);
    let later = base + Duration::from_secs(7 * 60 + 18);
    assert_eq!(
        window_title(&timer, later, GlyphTier::Emoji),
        "🍅 17:42 Work — pomodoro"
    );
    assert!(!timer.toggle(later));
    assert_eq!(
        window_title(&timer, later, GlyphTier::Emoji),
        "💤 17:42 Work paused — pomodoro"
    );
    assert!(!timer.skip(later));
    assert_eq!(
        window_title(&timer, later, GlyphTier::Emoji),
        "🔔 Short break ready — pomodoro"
    );
    assert_eq!(
        window_title(&timer, later, GlyphTier::Ascii),
        "! Short break ready — pomodoro"
    );
    let mut signals = Signals::default();
    let first = signals.title("a".to_owned());
    assert_eq!(first.as_deref(), Some("a"), "the first title is written");
    assert_eq!(
        signals.title("a".to_owned()),
        None,
        "an unchanged title is not"
    );
    assert_eq!(signals.title("b".to_owned()).as_deref(), Some("b"));
}

#[test]
fn progress_follows_state() {
    let base = Instant::now();
    let mut timer = Timer::start(Settings::default(), base);
    let quarter = base + WORK / 4;
    assert_eq!(taskbar(&timer, quarter), Taskbar::Running(25));
    assert!(!timer.toggle(quarter));
    assert_eq!(taskbar(&timer, quarter), Taskbar::Paused(25));
    assert!(!timer.skip(quarter));
    assert_eq!(taskbar(&timer, quarter), Taskbar::Waiting);
    let sequences = [
        (Taskbar::Running(25), "\x1b]9;4;1;25\x07"),
        (Taskbar::Paused(25), "\x1b]9;4;4;25\x07"),
        (Taskbar::Waiting, "\x1b]9;4;3;0\x07"),
        (Taskbar::Clear, "\x1b]9;4;0;0\x07"),
    ];
    for (state, expected) in sequences {
        assert_eq!(progress_sequence(state), expected, "{state:?}");
    }
    let mut signals = Signals::default();
    assert_eq!(
        signals.taskbar(Taskbar::Running(25), base),
        Some(Taskbar::Running(25))
    );
    assert_eq!(
        signals.taskbar(Taskbar::Running(25), base + Duration::from_secs(9)),
        None
    );
    assert_eq!(
        signals.taskbar(Taskbar::Running(25), base + PROGRESS_REFRESH),
        Some(Taskbar::Running(25)),
        "an unchanged state is written again after the refresh interval"
    );
    assert_eq!(
        signals.taskbar(Taskbar::Running(26), base + PROGRESS_REFRESH),
        Some(Taskbar::Running(26)),
        "a change is written at once"
    );
}
