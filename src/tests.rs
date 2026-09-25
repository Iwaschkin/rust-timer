use super::report;
use crate::terminal::{Step, TerminalError};
use std::io;

#[test]
fn run_error_names_step_then_cause() {
    let error = TerminalError::Failed {
        step: Step::Draw,
        source: io::Error::other("broken pipe"),
    };
    assert_eq!(report(&error), "could not draw the screen: broken pipe");
}
