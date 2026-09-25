//! The run loop: read the clock, end phases, run effects, draw, and act on keys
//! until the user quits.
//!
//! Each frame is drawn in three steps: the widgets, then the running effects, then
//! the fit to the terminal's colour tier.

use crate::motion::{self, Choreography, Cue, Motion};
use crate::options::Appearance;
use crate::settings::Settings;
use crate::terminal::{self, Command, RunFailure, Session, TerminalError};
use crate::theme;
use crate::timer::{Phase, State, Timer};
use crate::view;
use std::time::Instant;

/// Runs the timer in the terminal until the user quits.
///
/// # Errors
///
/// Fails when the terminal cannot be entered, used or restored.
pub(crate) fn run(
    settings: Settings,
    appearance: Appearance,
    motion: Motion,
) -> Result<(), RunFailure> {
    let mut session = Session::enter()?;
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
