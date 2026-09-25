//! Phase lengths and the long-break interval, with their defaults.

use std::time::Duration;

/// A phase length in whole minutes, never zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Minutes(u8);

impl Minutes {
    /// The length as a duration.
    pub(crate) fn duration(self) -> Duration {
        Duration::from_secs(u64::from(self.0) * 60)
    }
}

/// How many work phases make a cycle; the last is followed by the long break.
/// Never zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Every(u8);

impl Every {
    /// The count.
    pub(crate) fn get(self) -> u8 {
        self.0
    }
}

/// The lengths and interval a run uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Settings {
    /// Length of a work phase.
    pub(crate) work: Minutes,
    /// Length of a short break.
    pub(crate) short_break: Minutes,
    /// Length of a long break.
    pub(crate) long_break: Minutes,
    /// Work phases per cycle.
    pub(crate) every: Every,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            work: Minutes(25),
            short_break: Minutes(5),
            long_break: Minutes(15),
            every: Every(4),
        }
    }
}

#[cfg(test)]
mod tests;
