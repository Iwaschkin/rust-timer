//! `pomodoro`: a terminal pomodoro timer with a progress bar.

mod app;
mod settings;
mod terminal;
mod timer;
mod view;

use settings::Settings;
use std::error::Error;
use std::process::ExitCode;

/// Every level of a failure on one line, outermost first.
fn report(error: &dyn Error) -> String {
    let mut text = error.to_string();
    let mut cause = error.source();
    while let Some(source) = cause {
        text.push_str(": ");
        text.push_str(&source.to_string());
        cause = source.source();
    }
    text
}

fn main() -> ExitCode {
    match app::run(Settings::default()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(failure) => {
            for error in failure.errors() {
                eprintln!("pomodoro: {}", report(error));
            }
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests;
