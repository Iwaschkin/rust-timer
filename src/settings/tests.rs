use super::Settings;
use std::time::Duration;

#[test]
fn defaults_are_classic_pomodoro() {
    let settings = Settings::default();
    assert_eq!(settings.work.duration(), Duration::from_secs(25 * 60));
    assert_eq!(settings.short_break.duration(), Duration::from_secs(5 * 60));
    assert_eq!(settings.long_break.duration(), Duration::from_secs(15 * 60));
    assert_eq!(settings.every.get(), 4);
}
