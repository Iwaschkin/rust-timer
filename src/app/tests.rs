use super::{PROGRESS_REFRESH, Signals, run_loop, taskbar};
use crate::glyphs::GlyphTier;
use crate::motion::Motion;
use crate::options::Appearance;
use crate::settings::Settings;
use crate::terminal::{Command, Host, Taskbar, TerminalError, progress_sequence};
use crate::theme::ColorDepth;
use crate::timer::Timer;
use crate::view::window_title;
use ratatui::backend::TestBackend;
use ratatui::{Frame, Terminal};
use std::collections::VecDeque;
use std::error::Error;
use std::time::{Duration, Instant};

const WORK: Duration = Duration::from_secs(25 * 60);

/// A host whose clock moves only while the loop waits for a key, and whose keys
/// arrive at set times after the start: the live session with the user and time
/// scripted. Once the script is spent the user quits, so every run ends.
struct Scripted {
    start: Instant,
    now: Instant,
    keys: VecDeque<(Duration, Command)>,
    terminal: Terminal<TestBackend>,
    bells: Vec<Duration>,
    titles: Vec<String>,
    taskbars: Vec<Taskbar>,
}

impl Scripted {
    fn new(keys: &[(Duration, Command)]) -> Result<Self, Box<dyn Error>> {
        let start = Instant::now();
        Ok(Self {
            start,
            now: start,
            keys: keys.iter().copied().collect(),
            terminal: Terminal::new(TestBackend::new(20, 6))?,
            bells: Vec::new(),
            titles: Vec::new(),
            taskbars: Vec::new(),
        })
    }

    fn since_start(&self) -> Duration {
        self.now.saturating_duration_since(self.start)
    }
}

impl Host for Scripted {
    fn now(&self) -> Instant {
        self.now
    }

    fn next_command(&mut self, timeout: Duration) -> Result<Option<Command>, TerminalError> {
        let Some(&(at, command)) = self.keys.front() else {
            return Ok(Some(Command::Quit));
        };
        let pressed = self.start + at;
        if pressed <= self.now + timeout {
            self.now = self.now.max(pressed);
            self.keys.pop_front();
            Ok(Some(command))
        } else {
            self.now += timeout;
            Ok(None)
        }
    }

    fn draw(&mut self, render: impl FnOnce(&mut Frame<'_>)) -> Result<(), TerminalError> {
        match self.terminal.draw(render) {
            Ok(_frame) => Ok(()),
            Err(never) => match never {},
        }
    }

    fn ring_bell(&mut self) -> Result<(), TerminalError> {
        self.bells.push(self.since_start());
        Ok(())
    }

    fn set_title(&mut self, title: &str) -> Result<(), TerminalError> {
        self.titles.push(title.to_owned());
        Ok(())
    }

    fn set_taskbar(&mut self, taskbar: Taskbar) -> Result<(), TerminalError> {
        self.taskbars.push(taskbar);
        Ok(())
    }
}

/// A pause longer than the phase, driven through the whole loop: a one-minute work
/// phase runs 20 s, is paused for 90, then runs the 40 it has left and ends with
/// one bell, after which the short break waits (M09). The loop draws at least four
/// frames a second, so the phase is short to keep the test quick.
#[test]
fn loop_rings_one_bell_at_phase_end() -> Result<(), Box<dyn Error>> {
    let seconds = Duration::from_secs;
    let mut host = Scripted::new(&[
        (seconds(20), Command::StartPause),
        (seconds(110), Command::StartPause),
        (seconds(180), Command::Quit),
    ])?;
    let settings = Settings {
        work: "1".parse()?,
        ..Settings::default()
    };
    let appearance = Appearance {
        depth: ColorDepth::TrueColor,
        glyphs: GlyphTier::Emoji,
    };
    run_loop(&mut host, settings, appearance, Motion::Off)?;
    assert_eq!(
        host.bells,
        [seconds(150)],
        "one bell, as the work phase ends"
    );
    assert_eq!(
        host.since_start(),
        seconds(180),
        "the script ran to its end"
    );
    let paused = "💤 00:40 Work paused — pomodoro";
    assert!(host.titles.iter().any(|title| title == paused), "{paused}");
    assert_eq!(
        host.titles.last().map(String::as_str),
        Some("🔔 Short break ready — pomodoro")
    );
    let states: Vec<&str> = host
        .taskbars
        .iter()
        .map(|taskbar| match taskbar {
            Taskbar::Running(_) => "running",
            Taskbar::Paused(_) => "paused",
            Taskbar::Waiting => "waiting",
            Taskbar::Clear => "clear",
        })
        .collect();
    let mut changes = states.clone();
    changes.dedup();
    assert_eq!(changes, ["running", "paused", "running", "waiting"]);
    Ok(())
}

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
