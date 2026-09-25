//! The pomodoro cycle: which phase is on, whether it is running, and the time left.
//!
//! Every operation takes the clock reading as a parameter, so the timer never reads
//! the clock itself and tests control time exactly. Each operation first observes a
//! phase end that has already passed, then applies itself.

use crate::settings::Settings;
use std::time::{Duration, Instant};

/// A phase of the cycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Phase {
    /// Focused work.
    Work,
    /// The break after a work phase that is not the last of its cycle.
    ShortBreak,
    /// The break after the last work phase of a cycle.
    LongBreak,
}

/// Whether the current phase is counting down.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum State {
    /// Counting down.
    Running,
    /// Stopped part-way; space resumes it.
    Paused,
    /// Loaded at full length; space starts it.
    Ready,
}

#[derive(Clone, Copy, Debug)]
enum Clock {
    Running { since: Instant, banked: Duration },
    Paused { banked: Duration },
    Ready,
}

/// The cycle and the phase in progress.
#[derive(Debug)]
pub(crate) struct Timer {
    settings: Settings,
    phase: Phase,
    round: u8,
    clock: Clock,
}

impl Timer {
    /// Starts the first work phase at `now`.
    pub(crate) fn start(settings: Settings, now: Instant) -> Self {
        Self {
            settings,
            phase: Phase::Work,
            round: 1,
            clock: Clock::Running {
                since: now,
                banked: Duration::ZERO,
            },
        }
    }

    /// The current phase.
    pub(crate) fn phase(&self) -> Phase {
        self.phase
    }

    /// Whether the current phase is running, paused or ready.
    pub(crate) fn state(&self) -> State {
        match self.clock {
            Clock::Running { .. } => State::Running,
            Clock::Paused { .. } => State::Paused,
            Clock::Ready => State::Ready,
        }
    }

    /// The work phase's position in its cycle, from 1; a break shows the round it
    /// follows.
    pub(crate) fn round(&self) -> u8 {
        self.round
    }

    /// Work phases per cycle.
    pub(crate) fn rounds(&self) -> u8 {
        self.settings.every.get()
    }

    fn length(&self) -> Duration {
        match self.phase {
            Phase::Work => self.settings.work.duration(),
            Phase::ShortBreak => self.settings.short_break.duration(),
            Phase::LongBreak => self.settings.long_break.duration(),
        }
    }

    /// Time spent in the phase at `now`, from zero to the phase length.
    fn elapsed(&self, now: Instant) -> Duration {
        let elapsed = match self.clock {
            Clock::Running { since, banked } => {
                banked.saturating_add(now.saturating_duration_since(since))
            }
            Clock::Paused { banked } => banked,
            Clock::Ready => Duration::ZERO,
        };
        elapsed.min(self.length())
    }

    /// Time left in the phase at `now`.
    pub(crate) fn remaining(&self, now: Instant) -> Duration {
        self.length().saturating_sub(self.elapsed(now))
    }

    /// How much of the phase has passed at `now`.
    pub(crate) fn progress(&self, now: Instant) -> Progress {
        Progress::new(self.elapsed(now), self.length())
    }

    /// Ends the running phase if its time is up at `now`, loading the next one as
    /// ready. Returns whether a phase ended, which rings the bell.
    #[must_use = "a phase end rings the bell"]
    pub(crate) fn tick(&mut self, now: Instant) -> bool {
        let running = matches!(self.clock, Clock::Running { .. });
        let ended = running && self.elapsed(now) >= self.length();
        if ended {
            self.advance();
        }
        ended
    }

    /// Space: starts a ready phase, pauses a running one, resumes a paused one.
    /// Returns whether a phase ended first.
    #[must_use = "a phase end rings the bell"]
    pub(crate) fn toggle(&mut self, now: Instant) -> bool {
        let ended = self.tick(now);
        self.clock = match self.clock {
            Clock::Running { .. } => Clock::Paused {
                banked: self.elapsed(now),
            },
            Clock::Paused { banked } => Clock::Running { since: now, banked },
            Clock::Ready => Clock::Running {
                since: now,
                banked: Duration::ZERO,
            },
        };
        ended
    }

    /// `s`: loads the next phase of the cycle as ready, without a phase end of its
    /// own. Returns whether a phase ended first.
    #[must_use = "a phase end rings the bell"]
    pub(crate) fn skip(&mut self, now: Instant) -> bool {
        let ended = self.tick(now);
        self.advance();
        ended
    }

    /// Loads the phase that follows the current one, ready at full length.
    fn advance(&mut self) {
        (self.phase, self.round) = match self.phase {
            Phase::Work if self.round >= self.rounds() => (Phase::LongBreak, self.round),
            Phase::Work => (Phase::ShortBreak, self.round),
            Phase::ShortBreak => (Phase::Work, self.round.saturating_add(1)),
            Phase::LongBreak => (Phase::Work, 1),
        };
        self.clock = Clock::Ready;
    }
}

/// The filled share of the progress bar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Progress(f64);

impl Progress {
    /// `length` is never zero: a phase lasts at least a minute.
    fn new(elapsed: Duration, length: Duration) -> Self {
        Self(elapsed.min(length).div_duration_f64(length))
    }

    /// The share as a ratio from 0 to 1.
    pub(crate) fn ratio(self) -> f64 {
        self.0
    }
}

#[cfg(test)]
mod tests;
