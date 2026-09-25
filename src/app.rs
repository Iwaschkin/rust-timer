//! The run loop: read the clock, draw, and act on keys until the user quits.

use crate::settings::Settings;
use crate::terminal::{self, Command, RunFailure, Session, TerminalError};
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
pub(crate) fn run(settings: Settings) -> Result<(), RunFailure> {
    let mut session = Session::enter()?;
    let outcome = run_loop(&mut session, settings);
    session.finish(outcome)
}

fn run_loop(session: &mut Session, settings: Settings) -> Result<(), TerminalError> {
    let timer = Timer::start(settings, Instant::now());
    loop {
        let now = Instant::now();
        session.draw(|frame| view::render(frame, &timer, now, terminal::KEY_HELP))?;
        match session.next_command(REDRAW)? {
            Some(Command::Quit) => return Ok(()),
            None => {}
        }
    }
}
