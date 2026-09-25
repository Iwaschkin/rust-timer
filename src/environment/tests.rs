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

#[test]
fn progress_only_on_known_terminals() {
    let known = [
        terminal(&[("WT_SESSION", "x")], true, true),
        terminal(&[("TERM_PROGRAM", "vscode")], false, false),
        terminal(&[("TERM", "xterm-kitty")], false, false),
        terminal(&[("VTE_VERSION", "8000")], false, false),
        terminal(&[("VTE_VERSION", "8203")], false, false),
    ];
    for environment in known {
        assert!(environment.shows_progress(), "{environment:?}");
    }
    let unknown = [
        terminal(&[], true, true),
        terminal(&[("TERM", "xterm-256color")], false, false),
        terminal(&[("VTE_VERSION", "7800")], false, false),
        terminal(&[("VTE_VERSION", "new")], false, false),
        terminal(&[("TERM_PROGRAM", "iTerm.app")], false, false),
        terminal(&[("TERM_PROGRAM", "WezTerm")], false, false),
    ];
    for environment in unknown {
        assert!(!environment.shows_progress(), "{environment:?}");
    }
}
