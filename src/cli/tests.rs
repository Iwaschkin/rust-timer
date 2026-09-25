use super::{Invocation, parse};
use crate::settings::Settings;
use std::ffi::OsString;

fn settings(arguments: &[&str]) -> Settings {
    let invocation = parse(arguments.iter().map(OsString::from));
    let Ok(Invocation::Run(settings)) = invocation else {
        panic!("{arguments:?} should run, got {invocation:?}");
    };
    settings
}

#[test]
fn accepts_bounds_and_leading_zeros() {
    assert_eq!(settings(&["--work", "1"]).work.get(), 1);
    assert_eq!(settings(&["--work", "99"]).work.get(), 99);
    assert_eq!(settings(&["--short", "1"]).short_break.get(), 1);
    assert_eq!(settings(&["--long", "99"]).long_break.get(), 99);
    assert_eq!(settings(&["--every", "1"]).every.get(), 1);
    assert_eq!(settings(&["--every", "99"]).every.get(), 99);
    assert_eq!(settings(&["--work", "025"]).work.get(), 25);
    let all = settings(&[
        "--every", "2", "--long", "20", "--short", "3", "--work", "50",
    ]);
    let chosen = (
        all.work.get(),
        all.short_break.get(),
        all.long_break.get(),
        all.every.get(),
    );
    assert_eq!(chosen, (50, 3, 20, 2));
    assert_eq!(settings(&[]), Settings::default());
}
