use super::Timer;
use crate::settings::Settings;
use std::time::{Duration, Instant};

const WORK: Duration = Duration::from_secs(25 * 60);
const HOUR: Duration = Duration::from_secs(60 * 60);

#[test]
fn starts_running_first_work_phase() {
    let base = Instant::now();
    let timer = Timer::start(Settings::default(), base);
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
    let timer = Timer::start(Settings::default(), base);
    let mut reading = base;
    let mut total = Duration::ZERO;
    for step in 0..1_000_u64 {
        let delta = Duration::from_micros(step * 997 % 1_300);
        reading += delta;
        total += delta;
    }
    assert_eq!(timer.remaining(reading), timer.remaining(base + total));
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
