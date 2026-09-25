//! The command line, run as a built binary with its output captured.
//!
//! Captured output is not a terminal, which is what these tests rely on. The binary
//! is interactive, so a regression hangs rather than fails: every run is bounded and
//! killed at the limit.

use std::ffi::{OsStr, OsString};
use std::io;
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const LIMIT: Duration = Duration::from_secs(20);

fn pomodoro<A: AsRef<OsStr>>(arguments: &[A]) -> io::Result<Output> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_pomodoro"))
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let started = Instant::now();
    while child.try_wait()?.is_none() {
        if started.elapsed() > LIMIT {
            child.kill()?;
            child.wait()?;
            return Err(io::Error::other(format!(
                "pomodoro was still running after {LIMIT:?} and was killed"
            )));
        }
        thread::sleep(Duration::from_millis(20));
    }
    child.wait_with_output()
}

/// Runs with `arguments` and requires a usage error, as clap reports one: exit 2,
/// nothing on stdout, and stderr opening with an `error:` line that contains every
/// one of `mentions`. Returns stderr.
fn usage_error<A: AsRef<OsStr>>(arguments: &[A], mentions: &[&str]) -> io::Result<String> {
    let shown: Vec<_> = arguments.iter().map(AsRef::as_ref).collect();
    let output = pomodoro(arguments)?;
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    assert_eq!(output.status.code(), Some(2), "{shown:?}: {stderr}");
    assert!(
        output.stdout.is_empty(),
        "{shown:?}: stdout {:?}",
        output.stdout
    );
    let first = stderr.lines().next().unwrap_or_default();
    assert!(first.starts_with("error: "), "{shown:?}: {stderr}");
    for mention in mentions {
        assert!(
            stderr.contains(mention),
            "{shown:?}: {mention:?} in {stderr}"
        );
    }
    Ok(stderr)
}

#[test]
fn refuses_non_terminal_stdout() -> io::Result<()> {
    let output = pomodoro::<&str>(&[])?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "stderr: {stderr}");
    assert!(output.stdout.is_empty(), "stdout: {:?}", output.stdout);
    assert_eq!(stderr.lines().count(), 1, "stderr: {stderr}");
    assert!(stderr.contains("interactive terminal"), "stderr: {stderr}");
    Ok(())
}

#[test]
fn rejects_out_of_range_or_non_numeric() -> io::Result<()> {
    let cases = [
        ["--work", "0"],
        ["--short", "100"],
        ["--long", "abc"],
        ["--every", "-1"],
        ["--work", ""],
        ["--work", " 5"],
    ];
    for [flag, value] in cases {
        let quoted = format!("'{value}'");
        usage_error(&[flag, value], &[flag, &quoted, "1 to 99"])?;
    }
    Ok(())
}

#[test]
fn rejects_missing_value() -> io::Result<()> {
    usage_error(&["--work"], &["--work", "value is required"])?;
    usage_error(
        &["--short", "5", "--every"],
        &["--every", "value is required"],
    )?;
    Ok(())
}

#[test]
fn rejects_unknown_argument() -> io::Result<()> {
    let stderr = usage_error(&["--wrok", "5"], &["unexpected argument '--wrok'"])?;
    assert!(
        stderr.contains("'--work'"),
        "the near miss is named: {stderr}"
    );
    usage_error(&["5"], &["unexpected argument '5'"])?;
    Ok(())
}

#[test]
fn rejects_repeated_flag() -> io::Result<()> {
    usage_error(
        &["--work", "10", "--work", "20"],
        &["--work", "cannot be used multiple times"],
    )?;
    Ok(())
}

#[test]
fn rejects_non_unicode_argument() -> io::Result<()> {
    usage_error(
        &[OsString::from("--work"), not_unicode()],
        &["invalid UTF-8"],
    )?;
    usage_error(&[not_unicode()], &["unexpected argument"])?;
    Ok(())
}

/// An argument no Unicode string can hold: a lone surrogate on Windows, a byte
/// that starts no UTF-8 sequence elsewhere.
#[cfg(windows)]
fn not_unicode() -> OsString {
    use std::os::windows::ffi::OsStringExt;
    OsString::from_wide(&[0xD800])
}

#[cfg(unix)]
fn not_unicode() -> OsString {
    use std::os::unix::ffi::OsStringExt;
    OsString::from_vec(vec![0xFF])
}

#[test]
fn help_prints_usage() -> io::Result<()> {
    for arguments in [&["--help"][..], &["-h"], &["--help", "--work", "0"]] {
        let output = pomodoro(arguments)?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(output.status.code(), Some(0), "{arguments:?}");
        assert!(
            output.stderr.is_empty(),
            "{arguments:?}: {:?}",
            output.stderr
        );
        for expected in [
            "--work",
            "--short",
            "--long",
            "--every",
            "1 to 99",
            "[default: 25]",
            "[default: 5]",
            "[default: 15]",
            "[default: 4]",
            "--color",
            "--glyphs",
            "--motion",
            "truecolor",
            "emoji",
            "[default: auto]",
            "[default: on]",
            "space",
        ] {
            assert!(stdout.contains(expected), "{expected:?} in\n{stdout}");
        }
    }
    Ok(())
}

#[test]
fn argument_errors_precede_terminal_check() -> io::Result<()> {
    let stderr = usage_error(&["--work", "0"], &["--work"])?;
    assert!(!stderr.contains("interactive terminal"), "{stderr}");
    Ok(())
}

#[test]
fn stops_at_one_argument_error() -> io::Result<()> {
    let stderr = usage_error(&["--work", "0", "--wrok", "5"], &[])?;
    let errors = stderr
        .lines()
        .filter(|line| line.starts_with("error: "))
        .count();
    assert_eq!(errors, 1, "{stderr}");
    Ok(())
}

#[test]
fn rejects_unknown_choice() -> io::Result<()> {
    usage_error(
        &["--color", "8"],
        &["--color", "'8'", "auto, truecolor, 256, 16, none"],
    )?;
    usage_error(
        &["--glyphs", "fancy"],
        &["--glyphs", "'fancy'", "auto, emoji, symbols, ascii"],
    )?;
    usage_error(&["--motion", "maybe"], &["--motion", "'maybe'", "on, off"])?;
    Ok(())
}
