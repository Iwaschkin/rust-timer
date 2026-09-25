//! Phase lengths and the long-break interval: their range, defaults and parsing.

use std::error::Error;
use std::fmt;
use std::ops::RangeInclusive;
use std::str::FromStr;
use std::time::Duration;

/// The whole numbers every setting accepts. Two digits keep minutes on screen as
/// `MM:SS`.
pub(crate) const RANGE: RangeInclusive<u8> = 1..=99;

/// A setting's text that is not a whole number in [`RANGE`].
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct InvalidNumber {
    text: String,
}

impl fmt::Display for InvalidNumber {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:?} is not a whole number from {} to {}",
            self.text,
            RANGE.start(),
            RANGE.end()
        )
    }
}

impl Error for InvalidNumber {}

/// Reads `text` as a whole number in [`RANGE`]. Any failure, whether not a number,
/// too large for a byte or outside the range, is the same outcome for the user, so
/// the parser's own reason is not kept.
fn in_range(text: &str) -> Result<u8, InvalidNumber> {
    text.parse::<u8>()
        .ok()
        .filter(|number| RANGE.contains(number))
        .ok_or_else(|| InvalidNumber {
            text: text.to_owned(),
        })
}

/// A phase length in whole minutes, within [`RANGE`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Minutes(u8);

impl Minutes {
    /// The length as a duration.
    pub(crate) fn duration(self) -> Duration {
        Duration::from_secs(u64::from(self.0) * 60)
    }

    /// The length in minutes.
    pub(crate) fn get(self) -> u8 {
        self.0
    }
}

impl FromStr for Minutes {
    type Err = InvalidNumber;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        in_range(text).map(Self)
    }
}

/// How many work phases make a cycle; the last is followed by the long break.
/// Within [`RANGE`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Every(u8);

impl Every {
    /// The count.
    pub(crate) fn get(self) -> u8 {
        self.0
    }
}

impl FromStr for Every {
    type Err = InvalidNumber;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        in_range(text).map(Self)
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
