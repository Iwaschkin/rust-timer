//! The terminal: entering and restoring it, reading keys, and its failures.
//!
//! This is the only module that names crossterm. Swapping the backend changes this
//! file and the dependency's features.

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::crossterm::terminal::{BeginSynchronizedUpdate, EndSynchronizedUpdate};
use ratatui::crossterm::{ExecutableCommand, QueueableCommand};
use ratatui::{DefaultTerminal, Frame};
use std::error::Error;
use std::fmt;
use std::io::{self, IsTerminal, Write};
use std::time::{Duration, Instant};

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

/// What the taskbar button and tab show, in terminals that support OSC 9;4.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Taskbar {
    /// No indicator.
    Clear,
    /// A running phase, with its percentage done.
    Running(u8),
    /// A paused phase, with its percentage done.
    Paused(u8),
    /// A phase waiting for a key: animated, to draw the eye.
    Waiting,
}

/// The OSC 9;4 sequence for `taskbar`: `ESC ] 9 ; 4 ; state ; value BEL`, where
/// state 1 is normal, 4 paused (warning), 3 indeterminate and 0 cleared.
pub(crate) fn progress_sequence(taskbar: Taskbar) -> String {
    let (state, value) = match taskbar {
        Taskbar::Clear => (0, 0),
        Taskbar::Running(percent) => (1, percent),
        Taskbar::Paused(percent) => (4, percent),
        Taskbar::Waiting => (3, 0),
    };
    format!("\x1b]9;4;{state};{value}\x07")
}

/// The OSC 0 sequence that sets the window and tab title.
pub(crate) fn title_sequence(title: &str) -> String {
    format!("\x1b]0;{title}\x07")
}

/// Saves the current title on xterm's title stack.
const PUSH_TITLE: &str = "\x1b[22;0t";

/// Restores the title saved on xterm's title stack.
const POP_TITLE: &str = "\x1b[23;0t";

/// The title left behind where the terminal keeps no title stack.
const PARTING_TITLE: &str = "pomodoro";

/// What leaving writes before raw mode ends: the progress cleared, where it was
/// shown; a neutral title; then the saved title restored, where the terminal keeps
/// a title stack.
///
/// The neutral title is not empty, because Windows' pseudo-console ignores an empty
/// title and would keep, and later send, the last countdown written. Without a
/// stack, as there, the terminal is left showing the program's name rather than a
/// stopped clock.
pub(crate) fn farewell(taskbar: bool) -> String {
    let mut sequences = String::new();
    if taskbar {
        sequences.push_str(&progress_sequence(Taskbar::Clear));
    }
    sequences.push_str(&title_sequence(PARTING_TITLE));
    sequences.push_str(POP_TITLE);
    sequences
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
    /// Setting the window title or the taskbar progress.
    Signal,
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
            Self::Signal => "update the window title or taskbar",
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
    taskbar: bool,
}

impl Session {
    /// Enters raw mode and the alternate screen.
    ///
    /// # Errors
    ///
    /// Fails before changing anything when standard output is not a terminal, and
    /// when the terminal cannot be entered; whatever was already changed is restored
    /// first.
    ///
    /// The window title is saved on the terminal's title stack. `taskbar` says
    /// whether the terminal shows OSC 9;4 progress; without it none is written.
    pub(crate) fn enter(taskbar: bool) -> Result<Self, RunFailure> {
        if !io::stdout().is_terminal() {
            return Err(RunFailure::Run(TerminalError::NotATerminal));
        }
        let entered = |source| TerminalError::Failed {
            step: Step::Enter,
            source,
        };
        let terminal = match ratatui::try_init() {
            Ok(terminal) => terminal,
            Err(source) => return Err(after(entered(source), ratatui::try_restore())),
        };
        let mut session = Self {
            terminal,
            finished: false,
            taskbar,
        };
        if let Err(source) = session.write(PUSH_TITLE) {
            session.finished = true;
            return Err(after(entered(source), ratatui::try_restore()));
        }
        Ok(session)
    }

    /// Writes `sequence` straight to the terminal and flushes it.
    fn write(&mut self, sequence: &str) -> io::Result<()> {
        let backend = self.terminal.backend_mut();
        backend
            .write_all(sequence.as_bytes())
            .and_then(|()| backend.flush())
    }

    /// Clears the progress and restores the title, then the terminal, reporting
    /// `run`'s outcome together with the restore's.
    ///
    /// A farewell that cannot be written is a failed restore: the terminal is left
    /// showing this program's title or progress. Raw mode is left either way.
    ///
    /// # Errors
    ///
    /// Returns every failure, the run's first.
    pub(crate) fn finish(mut self, run: Result<(), TerminalError>) -> Result<(), RunFailure> {
        self.finished = true;
        let said = self.write(&farewell(self.taskbar));
        let restored = ratatui::try_restore();
        combine(run, said.and(restored))
    }
}

/// What the run loop needs from outside it: the clock, the keys and the terminal.
/// [`Session`] is the live host; a test drives the loop through a scripted one.
pub(crate) trait Host {
    /// The time now.
    fn now(&self) -> Instant;

    /// Waits up to `timeout` for an event and returns the command it asks for.
    ///
    /// # Errors
    ///
    /// Fails when events cannot be read.
    fn next_command(&mut self, timeout: Duration) -> Result<Option<Command>, TerminalError>;

    /// Draws one frame.
    ///
    /// # Errors
    ///
    /// Fails when the frame cannot be written to the terminal.
    fn draw(&mut self, render: impl FnOnce(&mut Frame<'_>)) -> Result<(), TerminalError>;

    /// Rings the terminal bell once.
    ///
    /// # Errors
    ///
    /// Fails when the bell cannot be written to the terminal.
    fn ring_bell(&mut self) -> Result<(), TerminalError>;

    /// Sets the window and tab title.
    ///
    /// # Errors
    ///
    /// Fails when the title cannot be written to the terminal.
    fn set_title(&mut self, title: &str) -> Result<(), TerminalError>;

    /// Shows `taskbar` as OSC 9;4 progress, in a terminal that supports it.
    ///
    /// # Errors
    ///
    /// Fails when the progress cannot be written to the terminal.
    fn set_taskbar(&mut self, taskbar: Taskbar) -> Result<(), TerminalError>;
}

impl Host for Session {
    fn now(&self) -> Instant {
        Instant::now()
    }

    fn set_title(&mut self, title: &str) -> Result<(), TerminalError> {
        self.write(&title_sequence(title))
            .map_err(TerminalError::at(Step::Signal))
    }

    fn set_taskbar(&mut self, taskbar: Taskbar) -> Result<(), TerminalError> {
        if !self.taskbar {
            return Ok(());
        }
        self.write(&progress_sequence(taskbar))
            .map_err(TerminalError::at(Step::Signal))
    }

    fn draw(&mut self, render: impl FnOnce(&mut Frame<'_>)) -> Result<(), TerminalError> {
        // Synchronized output: the terminal shows the whole frame at once, so an
        // effect never tears. A terminal that lacks it ignores both markers. The end
        // marker is written even when drawing fails, so updates are never held back.
        let began = self
            .terminal
            .backend_mut()
            .queue(BeginSynchronizedUpdate)
            .map(|_backend| ());
        let drawn = self.terminal.draw(render).map(|_frame| ());
        let ended = self
            .terminal
            .backend_mut()
            .execute(EndSynchronizedUpdate)
            .map(|_backend| ());
        began
            .and(drawn)
            .and(ended)
            .map_err(TerminalError::at(Step::Draw))
    }

    fn next_command(&mut self, timeout: Duration) -> Result<Option<Command>, TerminalError> {
        if !event::poll(timeout).map_err(TerminalError::at(Step::ReadInput))? {
            return Ok(None);
        }
        let event = event::read().map_err(TerminalError::at(Step::ReadInput))?;
        Ok(command(&event))
    }

    fn ring_bell(&mut self) -> Result<(), TerminalError> {
        self.write("\x07").map_err(TerminalError::at(Step::Bell))
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
