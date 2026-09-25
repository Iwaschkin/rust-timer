//! Phase lengths, with their defaults.

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

/// The lengths a run uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Settings {
    /// Length of a work phase.
    pub(crate) work: Minutes,
}

impl Default for Settings {
    fn default() -> Self {
        Self { work: Minutes(25) }
    }
}

#[cfg(test)]
mod tests;
