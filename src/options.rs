//! What the command line chooses: the timer's settings and how the screen looks.

use crate::environment::Environment;
use crate::glyphs::GlyphTier;
use crate::motion::Motion;
use crate::settings::Settings;
use crate::theme::ColorDepth;

/// A choice the user can leave to detection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Choice<T> {
    /// Detect it from the environment.
    Auto,
    /// Use this, whatever the environment says.
    Fixed(T),
}

impl<T> Choice<T> {
    /// The fixed value, or what `detect` finds.
    fn resolve(self, detect: impl FnOnce() -> T) -> T {
        match self {
            Self::Auto => detect(),
            Self::Fixed(value) => value,
        }
    }
}

/// How the screen looks, once every choice is resolved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Appearance {
    /// The colour tier every frame is fitted to.
    pub(crate) depth: ColorDepth,
    /// The glyph set.
    pub(crate) glyphs: GlyphTier,
}

/// Everything the command line chooses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Options {
    /// Lengths and interval.
    pub(crate) settings: Settings,
    /// The colour choice.
    pub(crate) color: Choice<ColorDepth>,
    /// The glyph choice.
    pub(crate) glyphs: Choice<GlyphTier>,
    /// Whether effects run.
    pub(crate) motion: Motion,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            color: Choice::Auto,
            glyphs: Choice::Auto,
            motion: Motion::On,
        }
    }
}

impl Options {
    /// The appearance, with each `auto` choice detected from `environment`.
    pub(crate) fn appearance(&self, environment: &Environment) -> Appearance {
        Appearance {
            depth: self.color.resolve(|| ColorDepth::detect(environment)),
            glyphs: self.glyphs.resolve(|| GlyphTier::detect(environment)),
        }
    }
}

#[cfg(test)]
mod tests;
