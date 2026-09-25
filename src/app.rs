//! The run loop: read the clock, end phases, run effects, draw, signal the window
//! title and taskbar, and act on keys until the user quits.
//!
//! Each frame is drawn in three steps: the widgets, then the running effects, then
//! the fit to the terminal's colour tier.

use crate::motion::{self, Choreography, Cue, Motion};
use crate::options::Appearance;
use crate::settings::Settings;
use crate::terminal::{self, Command, RunFailure, Session, Taskbar, TerminalError};
use crate::theme;
use crate::timer::{Phase, State, Timer};
use crate::view;
use std::time::{Duration, Instant};

/// Progress is written again at least this often, because some terminals drop it
/// after about fifteen seconds without a report.
const PROGRESS_REFRESH: Duration = Duration::from_secs(10);

/// Runs the timer in the terminal until the user quits. `taskbar` says whether the
/// terminal shows OSC 9;4 progress.
///
/// # Errors
///
/// Fails when the terminal cannot be entered, used or restored.
pub(crate) fn run(
    settings: Settings,
    appearance: Appearance,
    motion: Motion,
    taskbar: bool,
) -> Result<(), RunFailure> {
    let mut session = Session::enter(taskbar)?;
    let outcome = run_loop(&mut session, settings, appearance, motion);
    session.finish(outcome)
}

fn run_loop(
    session: &mut Session,
    settings: Settings,
    appearance: Appearance,
    motion: Motion,
) -> Result<(), TerminalError> {
    let started = Instant::now();
    let mut timer = Timer::start(settings, started);
    let mut choreography = Choreography::new(motion);
    choreography.cue(Cue::Start);
    let mut signals = Signals::default();
    let mut last_frame = started;
    loop {
        let now = Instant::now();
        let before = (timer.phase(), timer.state());
        if timer.tick(now) {
            session.ring_bell()?;
        }
        cue(&mut choreography, before, &timer);
        let elapsed = now.saturating_duration_since(last_frame);
        last_frame = now;
        session.draw(|frame| {
            view::render(frame, &timer, now, appearance, &terminal::KEYS);
            choreography.render(elapsed, frame.buffer_mut());
            theme::quantize(frame.buffer_mut(), appearance.depth);
        })?;
        if let Some(title) = signals.title(view::window_title(&timer, now, appearance.glyphs)) {
            session.set_title(&title)?;
        }
        if let Some(progress) = signals.taskbar(taskbar(&timer, now), now) {
            session.set_taskbar(progress)?;
        }
        let wait = motion::wake_interval(
            choreography.running(),
            timer.state() == State::Running,
            timer.remaining(now),
        );
        let Some(command) = session.next_command(wait)? else {
            continue;
        };
        let now = Instant::now();
        let before = (timer.phase(), timer.state());
        let ended = match command {
            Command::StartPause => timer.toggle(now),
            Command::Skip => timer.skip(now),
            Command::Quit => return Ok(()),
        };
        if ended {
            session.ring_bell()?;
        }
        cue(&mut choreography, before, &timer);
    }
}

/// Starts whatever effect the change from `before` to the timer's state calls for.
fn cue(choreography: &mut Choreography, before: (Phase, State), timer: &Timer) {
    if let Some(cue) = motion::cue_for(before, (timer.phase(), timer.state())) {
        choreography.cue(cue);
    }
}

/// What the taskbar shows for `timer` at `now` (I02).
fn taskbar(timer: &Timer, now: Instant) -> Taskbar {
    let percent = u8::try_from(timer.progress(now).scaled(100)).unwrap_or(100);
    match timer.state() {
        State::Running => Taskbar::Running(percent),
        State::Paused => Taskbar::Paused(percent),
        State::Ready => Taskbar::Waiting,
    }
}

/// What was last written outside the window, so each is written only when it
/// changes, and progress also when it has not been written for a while.
#[derive(Debug, Default)]
struct Signals {
    title: String,
    taskbar: Option<(Taskbar, Instant)>,
}

impl Signals {
    /// `title`, if it differs from the last one written.
    fn title(&mut self, title: String) -> Option<String> {
        if title == self.title {
            return None;
        }
        self.title.clone_from(&title);
        Some(title)
    }

    /// `taskbar`, if it differs from the last one written or that was written at
    /// least [`PROGRESS_REFRESH`] before `now`.
    fn taskbar(&mut self, taskbar: Taskbar, now: Instant) -> Option<Taskbar> {
        let due = match self.taskbar {
            Some((last, written)) => {
                last != taskbar || now.saturating_duration_since(written) >= PROGRESS_REFRESH
            }
            None => true,
        };
        if due {
            self.taskbar = Some((taskbar, now));
            Some(taskbar)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests;
