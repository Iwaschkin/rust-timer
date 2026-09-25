use super::{Choreography, Cue, Motion, cue_for, wake_interval};
use crate::theme;
use crate::timer::{Phase, State};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use std::time::Duration;

const FRAME: Duration = Duration::from_millis(50);

/// A small painted frame with some text and some amber, as the view would draw it.
fn frame(area: Rect) -> Buffer {
    let mut buffer = Buffer::empty(area);
    for (cell, index) in buffer.content.iter_mut().zip(0_u32..) {
        cell.set_symbol(if index % 3 == 0 { "█" } else { "x" });
        cell.set_fg(if index % 2 == 0 {
            theme::ALERT
        } else {
            theme::TEXT
        });
        cell.set_bg(theme::SURFACE);
    }
    buffer
}

/// Runs `choreography` for `frames` frames of `FRAME`, redrawing the base frame each
/// time as the loop does, and returns the last frame.
fn play(choreography: &mut Choreography, area: Rect, frames: u32) -> Buffer {
    let mut buffer = frame(area);
    for _ in 0..frames {
        buffer = frame(area);
        choreography.render(FRAME, &mut buffer);
    }
    buffer
}

#[test]
fn intro_effect_finishes() {
    let area = Rect::new(0, 0, 40, 10);
    let mut choreography = Choreography::new(Motion::On);
    choreography.cue(Cue::Start);
    assert!(choreography.running(), "the intro starts");
    play(&mut choreography, area, 30);
    assert!(!choreography.running(), "the intro is over within 1.5 s");
}

/// The amber cell's brightness, as the sum of its channels.
fn brightness(buffer: &Buffer) -> Option<u32> {
    let cell = buffer.content.first()?;
    let Color::Rgb(red, green, blue) = cell.fg else {
        return None;
    };
    Some(u32::from(red) + u32::from(green) + u32::from(blue))
}

#[test]
fn alert_pulse_is_slow_enough() {
    let area = Rect::new(0, 0, 6, 1);
    let mut choreography = Choreography::new(Motion::On);
    choreography.cue(Cue::Ready);
    let mut samples = Vec::new();
    for _ in 0..100_u32 {
        let buffer = play(&mut choreography, area, 1);
        samples.push(brightness(&buffer).unwrap_or(0));
    }
    assert!(choreography.running(), "the pulse lasts until a key acts");
    let peaks: Vec<usize> = samples
        .windows(3)
        .enumerate()
        .filter(
            |(_, window)| matches!(window, [before, peak, after] if peak > before && peak >= after),
        )
        .map(|(index, _)| index + 1)
        .collect();
    assert!(
        peaks.len() >= 2,
        "it pulses: peaks at {peaks:?} in {samples:?}"
    );
    for pair in peaks.windows(2) {
        if let [first, second] = pair {
            let gap = FRAME * u32::try_from(second - first).unwrap_or(0);
            assert!(
                gap >= Duration::from_millis(1500),
                "peaks {gap:?} apart: {peaks:?}"
            );
        }
    }
    let per_second = peaks.len() * 1000 / 5000;
    assert!(per_second < 3, "{} peaks in 5 s", peaks.len());
    choreography.cue(Cue::Begin);
    play(&mut choreography, area, 20);
    assert!(
        !choreography.running(),
        "a key ends the pulse, and the change settles"
    );
}

#[test]
fn wake_interval_follows_effects() {
    let seconds =
        |whole: u64, millis: u64| Duration::from_secs(whole) + Duration::from_millis(millis);
    assert_eq!(
        wake_interval(true, true, seconds(90, 300)),
        Duration::from_millis(33)
    );
    assert_eq!(
        wake_interval(false, true, seconds(90, 120)),
        Duration::from_millis(120)
    );
    assert_eq!(
        wake_interval(false, true, seconds(90, 900)),
        Duration::from_millis(250)
    );
    assert_eq!(
        wake_interval(false, true, seconds(90, 0)),
        Duration::from_millis(250)
    );
    assert_eq!(
        wake_interval(false, false, seconds(90, 120)),
        Duration::from_millis(250)
    );
}

#[test]
fn motion_off_runs_no_effects() {
    let area = Rect::new(0, 0, 20, 5);
    let mut choreography = Choreography::new(Motion::Off);
    for cue in [Cue::Start, Cue::Ready, Cue::Begin] {
        choreography.cue(cue);
        assert!(!choreography.running(), "{cue:?}");
    }
    assert_eq!(
        play(&mut choreography, area, 3),
        frame(area),
        "the frame is untouched"
    );
}

#[test]
fn effects_render_in_tiny_terminals() {
    for (width, height) in [(0, 0), (1, 1), (10, 3), (30, 8)] {
        let area = Rect::new(0, 0, width, height);
        let mut choreography = Choreography::new(Motion::On);
        for cue in [Cue::Start, Cue::Ready, Cue::Begin, Cue::Ready] {
            choreography.cue(cue);
            play(&mut choreography, area, 5);
        }
    }
}

#[test]
fn cues_follow_state_changes() {
    let (work, short) = (Phase::Work, Phase::ShortBreak);
    let cases = [
        (
            (work, State::Running),
            (short, State::Ready),
            Some(Cue::Ready),
        ),
        (
            (work, State::Paused),
            (short, State::Ready),
            Some(Cue::Ready),
        ),
        (
            (short, State::Ready),
            (work, State::Ready),
            Some(Cue::Ready),
        ),
        (
            (short, State::Ready),
            (short, State::Running),
            Some(Cue::Begin),
        ),
        (
            (work, State::Running),
            (short, State::Running),
            Some(Cue::Begin),
        ),
        ((work, State::Running), (work, State::Paused), None),
        ((work, State::Paused), (work, State::Running), None),
        ((short, State::Ready), (short, State::Ready), None),
    ];
    for (before, after, expected) in cases {
        assert_eq!(cue_for(before, after), expected, "{before:?} to {after:?}");
    }
}

#[test]
fn pulse_touches_only_the_alert_colour() {
    let area = Rect::new(0, 0, 6, 1);
    let mut choreography = Choreography::new(Motion::On);
    choreography.cue(Cue::Ready);
    // Past the popup's arrival, near the pulse's brightest point.
    let buffer = play(&mut choreography, area, 18);
    let colours: Vec<Color> = buffer.content.iter().map(|cell| cell.fg).collect();
    let text = colours.get(1).copied();
    assert_eq!(
        text,
        Some(theme::TEXT),
        "text keeps its colour: {colours:?}"
    );
    let alert = colours.first().copied();
    assert_ne!(alert, Some(theme::ALERT), "amber is mid-pulse: {colours:?}");
}
