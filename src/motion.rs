//! Motion: the effects that animate the screen, and how often the screen wakes.
//!
//! Effects run on each frame after the widgets are drawn and before the frame is
//! fitted to the colour tier, so they work in 24-bit colour whatever the terminal
//! shows. This is the only module that uses tachyonfx.

use crate::theme;
use crate::timer::{Phase, State};
use ratatui::buffer::Buffer;
use std::time::Duration;
use tachyonfx::{
    CellFilter, ColorSpace, Effect, EffectManager, Interpolation, Motion as Direction, fx,
};

/// Whether effects run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Motion {
    /// Animate.
    On,
    /// Never start an effect.
    Off,
}

/// Something that happened, which may start an effect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Cue {
    /// The program opened.
    Start,
    /// A phase ended or was skipped, and the next one waits.
    Ready,
    /// A phase started after waiting, or straight after a late phase end.
    Begin,
}

/// Where each running effect lives; a new effect in a slot replaces the old one.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
enum Slot {
    #[default]
    Intro,
    Alert,
    Change,
}

/// The frame interval while an effect runs: about thirty frames a second.
const EFFECT_FRAME: Duration = Duration::from_millis(33);
/// The longest the screen waits between frames.
const IDLE_FRAME: Duration = Duration::from_millis(250);

/// The running effects, and whether any may start.
pub(crate) struct Choreography {
    motion: Motion,
    effects: EffectManager<Slot>,
}

impl Choreography {
    /// No effect running yet.
    pub(crate) fn new(motion: Motion) -> Self {
        Self {
            motion,
            effects: EffectManager::default(),
        }
    }

    /// Starts the effects `cue` calls for, unless motion is off.
    pub(crate) fn cue(&mut self, cue: Cue) {
        if self.motion == Motion::Off {
            return;
        }
        match cue {
            Cue::Start => self.effects.add_unique_effect(Slot::Intro, intro()),
            Cue::Ready => {
                self.effects.add_unique_effect(Slot::Change, arrival());
                self.effects.add_unique_effect(Slot::Alert, pulse());
            }
            Cue::Begin => {
                self.effects.cancel_unique_effect(Slot::Alert);
                self.effects.add_unique_effect(Slot::Change, departure());
            }
        }
    }

    /// Advances every effect by `elapsed` over the frame in `buffer`.
    pub(crate) fn render(&mut self, elapsed: Duration, buffer: &mut Buffer) {
        let area = buffer.area;
        self.effects.process_effects(elapsed, buffer, area);
    }

    /// Whether any effect is running.
    pub(crate) fn running(&self) -> bool {
        self.effects.is_running()
    }
}

/// The screen assembles itself: cells coalesce while a wave sweeps in from the left.
/// Colours blend in RGB, straight to the background, so the tomato never passes
/// through purple on its way.
fn intro() -> Effect {
    fx::parallel(&[
        fx::coalesce((700, Interpolation::QuadOut)),
        fx::sweep_in(
            Direction::LeftToRight,
            24,
            0,
            theme::BACKGROUND,
            (700, Interpolation::QuadOut),
        )
        .with_color_space(ColorSpace::Rgb),
    ])
}

/// The popup drops in from above out of the shadow colour.
fn arrival() -> Effect {
    fx::sweep_in(
        Direction::UpToDown,
        12,
        0,
        theme::SHADOW,
        (450, Interpolation::QuadOut),
    )
    .with_color_space(ColorSpace::Rgb)
}

/// The alert's pulse: everything amber brightens and fades back, once every 1.8 s,
/// far below the three flashes a second that WCAG 2.3.1 allows.
///
/// The filter sits on the innermost effect: set on the `repeating` wrapper it does
/// not reach the shift inside, and every colour on the screen would pulse.
fn pulse() -> Effect {
    let brighten = fx::hsl_shift_fg([0.0, 0.0, 18.0], (900, Interpolation::SineInOut))
        .with_filter(CellFilter::FgColor(theme::ALERT));
    fx::repeating(fx::ping_pong(brighten))
}

/// A phase starts: the screen coalesces into it.
fn departure() -> Effect {
    fx::coalesce((350, Interpolation::QuadOut))
}

/// The cue for going from `before` to `after`, each a phase and its state.
pub(crate) fn cue_for(before: (Phase, State), after: (Phase, State)) -> Option<Cue> {
    let (was, now) = (before.1, after.1);
    if now == State::Ready && before != after {
        Some(Cue::Ready)
    } else if now == State::Running && (was == State::Ready || before.0 != after.0) {
        Some(Cue::Begin)
    } else {
        None
    }
}

/// How long the loop waits before the next frame (E03): a frame's worth while an
/// effect runs; otherwise until the time shown changes, and never more than 250 ms.
pub(crate) fn wake_interval(
    effects_running: bool,
    counting: bool,
    remaining: Duration,
) -> Duration {
    if effects_running {
        return EFFECT_FRAME;
    }
    let to_next_second = Duration::from_nanos(u64::from(remaining.subsec_nanos()));
    if !counting || to_next_second.is_zero() {
        IDLE_FRAME
    } else {
        to_next_second.min(IDLE_FRAME)
    }
}

#[cfg(test)]
mod tests;
