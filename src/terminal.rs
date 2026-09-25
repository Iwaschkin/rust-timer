//! The terminal: entering and restoring it, reading keys, and its failures.
//!
//! This is the only module that names crossterm. Swapping the backend changes this
//! file and the dependency's features.

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{DefaultTerminal, Frame};
use std::error::Error;
use std::fmt;
use std::io::{self, IsTerminal, Write};
use std::time::Duration;

/// What a key press asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Command {
    /// Start, pause or resume the phase.
    StartPause,
    /// Move to the next phase.
    Skip,
    /// Leave the program.
    Quit,
}

/// The keys and what they do, as the screen and the usage text list them.
pub(crate) const KEYS: [(&str, &str); 3] = [("space", "start/pause"), ("s", "skip"), ("q", "quit")];

/// The keys on one line, for the usage text.
pub(crate) fn key_help() -> String {
    KEYS.iter()
        .map(|(key, action)| format!("{key} {action}"))
        .collect::<Vec<_>>()
        .join(" · ")
}

/// Whether the console takes 24-bit colour, as crossterm judges it. On Windows
/// that is whether virtual-terminal output could be enabled.
pub(crate) fn console_truecolor() -> bool {
    ratatui::crossterm::style::available_color_count() == u16::MAX
}

/// The command a terminal event asks for, if any. Only key presses count: Windows
/// also reports releases, which would otherwise act twice.
pub(crate) fn command(event: &Event) -> Option<Command> {
    let Event::Key(key) = event else {
        return None;
    };
    if key.kind != KeyEventKind::Press {
        return None;
    }
    let KeyCode::Char(character) = key.code else {
        return (key.code == KeyCode::Esc).then_some(Command::Quit);
    };
    if key.modifiers == KeyModifiers::CONTROL {
        return (character == 'c').then_some(Command::Quit);
    }
    if !key.modifiers.difference(KeyModifiers::SHIFT).is_empty() {
        return None;
    }
    match character {
        ' ' => Some(Command::StartPause),
        's' | 'S' => Some(Command::Skip),
        'q' | 'Q' => Some(Command::Quit),
        _ => None,
    }
}

/// The step of terminal handling that failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Step {
    /// Entering raw mode and the alternate screen.
    Enter,
    /// Drawing a frame.
    Draw,
    /// Waiting for or reading an event.
    ReadInput,
    /// Ringing the terminal bell.
    Bell,
    /// Leaving raw mode and the alternate screen.
    Restore,
}

impl fmt::Display for Step {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Enter => "enter the terminal",
            Self::Draw => "draw the screen",
            Self::ReadInput => "read input",
            Self::Bell => "ring the bell",
            Self::Restore => "restore the terminal",
        })
    }
}

/// Why the terminal could not be used.
#[derive(Debug)]
pub(crate) enum TerminalError {
    /// Standard output is not an interactive terminal.
    NotATerminal,
    /// A terminal operation failed.
    Failed {
        /// The operation.
        step: Step,
        /// Its cause.
        source: io::Error,
    },
}

impl TerminalError {
    fn at(step: Step) -> impl FnOnce(io::Error) -> Self {
        move |source| Self::Failed { step, source }
    }
}

impl fmt::Display for TerminalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotATerminal => formatter
                .write_str("an interactive terminal is required; standard output is not one"),
            Self::Failed { step, .. } => write!(formatter, "could not {step}"),
        }
    }
}

impl Error for TerminalError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NotATerminal => None,
            Self::Failed { source, .. } => Some(source),
        }
    }
}

/// How a run failed: while running, while restoring, or both.
#[derive(Debug)]
pub(crate) enum RunFailure {
    /// The run failed and the terminal was restored.
    Run(TerminalError),
    /// The run ended normally and restoring failed.
    Restore(TerminalError),
    /// The run failed, then restoring failed too.
    Both {
        /// The run's failure.
        run: TerminalError,
        /// The restore's failure.
        restore: TerminalError,
    },
}

impl RunFailure {
    /// Each failure, in the order it happened.
    pub(crate) fn errors(&self) -> Vec<&TerminalError> {
        match self {
            Self::Run(error) | Self::Restore(error) => vec![error],
            Self::Both { run, restore } => vec![run, restore],
        }
    }
}

/// Joins a run's outcome with the restore that followed it.
fn combine(run: Result<(), TerminalError>, restore: io::Result<()>) -> Result<(), RunFailure> {
    match run {
        Ok(()) => restore
            .map_err(TerminalError::at(Step::Restore))
            .map_err(RunFailure::Restore),
        Err(run) => Err(after(run, restore)),
    }
}

/// A failed run joined with the restore that followed it.
fn after(run: TerminalError, restore: io::Result<()>) -> RunFailure {
    match restore.map_err(TerminalError::at(Step::Restore)) {
        Ok(()) => RunFailure::Run(run),
        Err(restore) => RunFailure::Both { run, restore },
    }
}

/// The terminal in raw mode on the alternate screen, until [`Session::finish`].
pub(crate) struct Session {
    terminal: DefaultTerminal,
    finished: bool,
}

impl Session {
    /// Enters raw mode and the alternate screen.
    ///
    /// # Errors
    ///
    /// Fails before changing anything when standard output is not a terminal, and
    /// when the terminal cannot be entered; whatever was already changed is restored
    /// first.
    pub(crate) fn enter() -> Result<Self, RunFailure> {
        if !io::stdout().is_terminal() {
            return Err(RunFailure::Run(TerminalError::NotATerminal));
        }
        match ratatui::try_init() {
            Ok(terminal) => Ok(Self {
                terminal,
                finished: false,
            }),
            Err(source) => Err(after(
                TerminalError::Failed {
                    step: Step::Enter,
                    source,
                },
                ratatui::try_restore(),
            )),
        }
    }

    /// Draws one frame.
    ///
    /// # Errors
    ///
    /// Fails when the frame cannot be written to the terminal.
    pub(crate) fn draw(
        &mut self,
        render: impl FnOnce(&mut Frame<'_>),
    ) -> Result<(), TerminalError> {
        self.terminal
            .draw(render)
            .map(|_frame| ())
            .map_err(TerminalError::at(Step::Draw))
    }

    /// Waits up to `timeout` for an event and returns the command it asks for.
    ///
    /// # Errors
    ///
    /// Fails when events cannot be read.
    pub(crate) fn next_command(
        &mut self,
        timeout: Duration,
    ) -> Result<Option<Command>, TerminalError> {
        if !event::poll(timeout).map_err(TerminalError::at(Step::ReadInput))? {
            return Ok(None);
        }
        let event = event::read().map_err(TerminalError::at(Step::ReadInput))?;
        Ok(command(&event))
    }

    /// Rings the terminal bell once.
    ///
    /// # Errors
    ///
    /// Fails when the bell cannot be written to the terminal.
    pub(crate) fn ring_bell(&mut self) -> Result<(), TerminalError> {
        let backend = self.terminal.backend_mut();
        backend
            .write_all(b"\x07")
            .and_then(|()| backend.flush())
            .map_err(TerminalError::at(Step::Bell))
    }

    /// Restores the terminal, reporting `run`'s outcome together with the restore's.
    ///
    /// # Errors
    ///
    /// Returns every failure, the run's first.
    pub(crate) fn finish(mut self, run: Result<(), TerminalError>) -> Result<(), RunFailure> {
        self.finished = true;
        combine(run, ratatui::try_restore())
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        if !self.finished {
            // Fallback for a path that skipped `finish`; ratatui reports its own failure.
            ratatui::restore();
        }
    }
}

#[cfg(test)]
mod tests;
