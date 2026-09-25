//! `pomodoro`: a terminal pomodoro timer with a progress bar.

mod app;
mod cli;
mod environment;
mod glyphs;
mod motion;
mod options;
mod settings;
mod terminal;
mod theme;
mod timer;
mod view;

use environment::Environment;
use std::error::Error;
use std::process::ExitCode;

/// The exit status for arguments that were rejected before anything ran.
const USAGE: u8 = 2;

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
    let options = match cli::parse(std::env::args_os(), &terminal::key_help()) {
        Ok(options) => options,
        // Help and argument errors both stop here: clap prints help to stdout and an
        // error to stderr, in colour where that is a terminal.
        Err(stop) => {
            return match stop.print() {
                Err(error) => {
                    eprintln!("pomodoro: could not print the usage: {}", report(&error));
                    ExitCode::FAILURE
                }
                Ok(()) if stop.use_stderr() => ExitCode::from(USAGE),
                Ok(()) => ExitCode::SUCCESS,
            };
        }
    };
    let environment = Environment::from_lookup(
        |name| std::env::var_os(name),
        cfg!(windows),
        terminal::console_truecolor(),
    );
    let appearance = options.appearance(&environment);
    match app::run(
        options.settings,
        appearance,
        options.motion,
        environment.shows_progress(),
    ) {
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
