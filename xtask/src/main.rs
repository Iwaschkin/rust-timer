//! Local quality commands; native check failures remain visible to the caller.

use std::{path::Path, process::ExitCode};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> rust_quality::Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("Missing project root")?;
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    match arguments.split_first() {
        Some((command, rest)) if command == "guard" && rest.is_empty() => {
            rust_quality::inspect(root)
        }
        Some((command, rest)) if command == "check" => rust_quality::check(root, rest),
        Some((command, rest)) if command == "gates" && rest.is_empty() => {
            println!("{}", rust_quality::gates(root)?);
            Ok(())
        }
        // Run from the installed skill, whose assets sit beside this crate.
        Some((command, [project])) if command == "init" => {
            println!("{}", rust_quality::init(root, Path::new(project))?);
            Ok(())
        }
        _ => Err(
            "Usage: cargo xtask guard | gates | check [--fast | --no-release-tests] \
                  [--no-default-features] [--features names] [--all-features]; \
                  from the installed skill: init <project>"
                .into(),
        ),
    }
}
