//! The running phase and the time left in it.
//!
//! Every operation takes the clock reading as a parameter, so the timer never reads
//! the clock itself and tests control time exactly.

use crate::settings::Settings;
use std::time::{Duration, Instant};

/// A phase in progress.
#[derive(Debug)]
pub(crate) struct Timer {
    length: Duration,
    since: Instant,
}

impl Timer {
    /// Starts the first work phase at `now`.
    pub(crate) fn start(settings: Settings, now: Instant) -> Self {
        Self {
            length: settings.work.duration(),
            since: now,
        }
    }

    /// Time spent in the phase at `now`; zero for a reading before the start.
    fn elapsed(&self, now: Instant) -> Duration {
        now.saturating_duration_since(self.since)
    }

    /// Time left in the phase at `now`.
    pub(crate) fn remaining(&self, now: Instant) -> Duration {
        self.length.saturating_sub(self.elapsed(now))
    }

    /// How much of the phase has passed at `now`.
    pub(crate) fn progress(&self, now: Instant) -> Progress {
        Progress::new(self.elapsed(now), self.length)
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
