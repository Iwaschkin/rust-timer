use super::{Every, Minutes, Settings};
use std::time::Duration;

#[test]
fn defaults_are_classic_pomodoro() {
    let settings = Settings::default();
    assert_eq!(settings.work.duration(), Duration::from_secs(25 * 60));
    assert_eq!(settings.short_break.duration(), Duration::from_secs(5 * 60));
    assert_eq!(settings.long_break.duration(), Duration::from_secs(15 * 60));
    assert_eq!(settings.every.get(), 4);
}

#[test]
fn values_outside_range_are_rejected() {
    for text in ["0", "100", "256", "-1", "", " 5", "5 ", "abc", "1.5"] {
        let minutes = text.parse::<Minutes>();
        assert!(minutes.is_err(), "{text:?} as minutes: {minutes:?}");
        let every = text.parse::<Every>();
        assert!(every.is_err(), "{text:?} as a count: {every:?}");
    }
    let message = "0".parse::<Minutes>().map_err(|error| error.to_string());
    assert_eq!(
        message,
        Err("\"0\" is not a whole number from 1 to 99".to_owned())
    );
}
