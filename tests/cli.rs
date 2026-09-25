//! The command line, run as a built binary with its output captured.
//!
//! Captured output is not a terminal, which is what these tests rely on. The binary
//! is interactive, so a regression hangs rather than fails: every run is bounded and
//! killed at the limit.

use std::io;
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const LIMIT: Duration = Duration::from_secs(20);

fn pomodoro(arguments: &[&str]) -> io::Result<Output> {
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

#[test]
fn refuses_non_terminal_stdout() -> io::Result<()> {
    let output = pomodoro(&[])?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "stderr: {stderr}");
    assert!(output.stdout.is_empty(), "stdout: {:?}", output.stdout);
    assert_eq!(stderr.lines().count(), 1, "stderr: {stderr}");
    assert!(stderr.contains("interactive terminal"), "stderr: {stderr}");
    Ok(())
}
