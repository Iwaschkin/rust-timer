use super::Settings;
use std::time::Duration;

#[test]
fn defaults_are_classic_pomodoro() {
    let settings = Settings::default();
    assert_eq!(settings.work.duration(), Duration::from_secs(25 * 60));
}
