use super::Environment;
use std::ffi::OsString;

/// A terminal described by its variables, for the tests of every module that
/// detects something.
pub(crate) fn terminal(variables: &[(&str, &str)], windows: bool, console: bool) -> Environment {
    Environment::from_lookup(
        |name| {
            variables
                .iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| OsString::from(value))
        },
        windows,
        console,
    )
}

#[test]
fn empty_values_count_as_unset() {
    let described = terminal(&[("NO_COLOR", ""), ("COLORTERM", "")], false, false);
    assert!(!described.no_color());
    assert!(!described.announces_truecolor());
    assert!(terminal(&[("NO_COLOR", "1")], false, false).no_color());
}

#[test]
fn legacy_console_is_windows_without_a_terminal_program() {
    assert!(terminal(&[], true, true).legacy_windows_console());
    assert!(!terminal(&[("WT_SESSION", "x")], true, true).legacy_windows_console());
    assert!(!terminal(&[("TERM_PROGRAM", "vscode")], true, true).legacy_windows_console());
    assert!(!terminal(&[], false, false).legacy_windows_console());
}
