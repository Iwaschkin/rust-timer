//! The run loop: read the clock, end phases, draw, and act on keys until the user
//! quits.

use crate::options::Appearance;
use crate::settings::Settings;
use crate::terminal::{self, Command, RunFailure, Session, TerminalError};
use crate::theme;
use crate::timer::Timer;
use crate::view;
use std::time::{Duration, Instant};

/// The longest the screen waits before redrawing. Timing never depends on it: the
/// time left comes from the clock, not from counting redraws.
const REDRAW: Duration = Duration::from_millis(250);

/// Runs the timer in the terminal until the user quits.
///
/// # Errors
///
/// Fails when the terminal cannot be entered, used or restored.
pub(crate) fn run(settings: Settings, appearance: Appearance) -> Result<(), RunFailure> {
    let mut session = Session::enter()?;
    let outcome = run_loop(&mut session, settings, appearance);
    session.finish(outcome)
}

fn run_loop(
    session: &mut Session,
    settings: Settings,
    appearance: Appearance,
) -> Result<(), TerminalError> {
    let mut timer = Timer::start(settings, Instant::now());
    loop {
        let now = Instant::now();
        if timer.tick(now) {
            session.ring_bell()?;
        }
        session.draw(|frame| {
            view::render(frame, &timer, now, appearance, &terminal::KEYS);
            theme::quantize(frame.buffer_mut(), appearance.depth);
        })?;
        let Some(command) = session.next_command(REDRAW)? else {
            continue;
        };
        let now = Instant::now();
        let ended = match command {
            Command::StartPause => timer.toggle(now),
            Command::Skip => timer.skip(now),
            Command::Quit => return Ok(()),
        };
        if ended {
            session.ring_bell()?;
        }
    }
}
