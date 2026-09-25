use super::Phase::{LongBreak, ShortBreak, Work};
use super::{Phase, State, Timer};
use crate::settings::Settings;
use std::time::{Duration, Instant};

const WORK: Duration = Duration::from_secs(25 * 60);
const SHORT: Duration = Duration::from_secs(5 * 60);
const LONG: Duration = Duration::from_secs(15 * 60);
const MINUTE: Duration = Duration::from_secs(60);
const HOUR: Duration = Duration::from_secs(60 * 60);

/// Runs the current phase to its end: starts it if ready, then reads the clock at
/// its last instant. Returns the reading.
fn finish_phase(timer: &mut Timer, now: Instant) -> Instant {
    if timer.state() == State::Ready {
        assert!(!timer.toggle(now));
    }
    let end = now + timer.remaining(now);
    assert!(timer.tick(end), "the phase ends at {end:?}");
    end
}

#[test]
fn starts_running_first_work_phase() {
    let base = Instant::now();
    let timer = Timer::start(Settings::default(), base);
    assert_eq!(timer.phase(), Phase::Work);
    assert_eq!((timer.round(), timer.rounds()), (1, 4));
    assert_eq!(timer.state(), State::Running);
    assert_eq!(timer.remaining(base), WORK);
    assert_eq!(timer.progress(base).ratio(), 0.0);
}

#[test]
fn remaining_counts_down() {
    let base = Instant::now();
    let timer = Timer::start(Settings::default(), base);
    let seven = Duration::from_secs(7 * 60 + 3);
    assert_eq!(timer.remaining(base + seven), WORK - seven);
}

#[test]
fn uneven_ticks_do_not_drift() {
    let base = Instant::now();
    let mut timer = Timer::start(Settings::default(), base);
    let mut reading = base;
    let mut total = Duration::ZERO;
    for step in 0..1_000_u64 {
        let delta = Duration::from_micros(step * 997 % 1_300);
        reading += delta;
        total += delta;
        assert!(!timer.tick(reading));
    }
    let untouched = Timer::start(Settings::default(), base);
    assert_eq!(timer.remaining(reading), untouched.remaining(base + total));
}

#[test]
fn clock_before_start_reads_as_no_elapsed_time() {
    let base = Instant::now() + HOUR;
    let timer = Timer::start(Settings::default(), base);
    let earlier = base - Duration::from_secs(90);
    assert_eq!(timer.remaining(earlier), WORK);
    assert_eq!(timer.progress(earlier).ratio(), 0.0);
}

#[test]
fn progress_stays_within_unit_interval() {
    let base = Instant::now() + HOUR;
    let timer = Timer::start(Settings::default(), base);
    let readings = [
        (base, 0.0),
        (base + WORK / 2, 0.5),
        (base + WORK, 1.0),
        (base - Duration::from_secs(1), 0.0),
        (base + WORK + 3 * HOUR, 1.0),
    ];
    for (reading, expected) in readings {
        assert_eq!(timer.progress(reading).ratio(), expected);
    }
}

#[test]
fn pause_freezes_remaining() {
    let base = Instant::now();
    let mut timer = Timer::start(Settings::default(), base);
    assert!(!timer.toggle(base + 5 * MINUTE));
    assert_eq!(timer.state(), State::Paused);
    assert_eq!(timer.remaining(base + 2 * HOUR), WORK - 5 * MINUTE);
    assert!(!timer.toggle(base + 2 * HOUR));
    assert_eq!(timer.state(), State::Running);
    assert_eq!(timer.remaining(base + 2 * HOUR + MINUTE), WORK - 6 * MINUTE);
}

#[test]
fn phase_end_reported_once() {
    let base = Instant::now();
    let mut timer = Timer::start(Settings::default(), base);
    assert!(!timer.tick(base + WORK - Duration::from_nanos(1)));
    assert!(timer.tick(base + WORK));
    assert_eq!(
        (timer.phase(), timer.state()),
        (Phase::ShortBreak, State::Ready)
    );
    assert_eq!(timer.remaining(base + WORK + HOUR), SHORT);
    assert!(!timer.tick(base + WORK + Duration::from_secs(1)));
    assert!(!timer.tick(base + WORK + 5 * HOUR));
}

#[test]
fn cycle_follows_long_break_interval() {
    let mut now = Instant::now();
    let mut timer = Timer::start(Settings::default(), now);
    let mut seen = vec![(timer.phase(), timer.round())];
    for _ in 0..8 {
        now = finish_phase(&mut timer, now);
        seen.push((timer.phase(), timer.round()));
    }
    let expected = [
        (Work, 1),
        (ShortBreak, 1),
        (Work, 2),
        (ShortBreak, 2),
        (Work, 3),
        (ShortBreak, 3),
        (Work, 4),
        (LongBreak, 4),
        (Work, 1),
    ];
    assert_eq!(seen, expected);
}

#[test]
fn skip_moves_to_next_phase_silently() {
    let base = Instant::now();
    let mut running = Timer::start(Settings::default(), base);
    assert!(!running.skip(base + MINUTE));
    assert_eq!(
        (running.phase(), running.state()),
        (Phase::ShortBreak, State::Ready)
    );
    assert_eq!(running.remaining(base + HOUR), SHORT);

    let mut paused = Timer::start(Settings::default(), base);
    assert!(!paused.toggle(base + MINUTE));
    assert!(!paused.skip(base + 2 * MINUTE));
    assert_eq!(
        (paused.phase(), paused.state()),
        (Phase::ShortBreak, State::Ready)
    );

    assert!(!running.skip(base + HOUR));
    assert_eq!((running.phase(), running.round()), (Phase::Work, 2));
    assert_eq!(running.state(), State::Ready);
}

#[test]
fn late_reading_ends_only_one_phase() {
    let base = Instant::now();
    let mut timer = Timer::start(Settings::default(), base);
    assert!(timer.tick(base + WORK + 3 * HOUR));
    assert_eq!((timer.phase(), timer.round()), (Phase::ShortBreak, 1));
    assert_eq!(timer.state(), State::Ready);
    assert_eq!(timer.remaining(base + WORK + 4 * HOUR), SHORT);
}

#[test]
fn command_after_phase_end_applies_to_next_phase() {
    let base = Instant::now();
    let mut timer = Timer::start(Settings::default(), base);
    let late = base + WORK + Duration::from_secs(10);
    assert!(timer.toggle(late), "the passed end is reported first");
    assert_eq!(
        (timer.phase(), timer.state()),
        (Phase::ShortBreak, State::Running)
    );
    assert_eq!(timer.remaining(late + MINUTE), SHORT - MINUTE);
}

#[test]
fn space_starts_ready_phase() {
    let base = Instant::now();
    let mut timer = Timer::start(Settings::default(), base);
    let mut now = base;
    for _ in 0..7 {
        now = finish_phase(&mut timer, now);
    }
    assert_eq!(
        (timer.phase(), timer.state()),
        (Phase::LongBreak, State::Ready)
    );
    let start = now + HOUR;
    assert!(!timer.toggle(start));
    assert_eq!(timer.state(), State::Running);
    let half_minute = Duration::from_secs(30);
    assert_eq!(timer.remaining(start + half_minute), LONG - half_minute);
}
