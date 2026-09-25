use crate::{Result, config};
use std::{path::Path, process::Command};

/// Which checks a run performs. Only `Full` is evidence that a slice is done.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Lane {
    Full,
    /// Everything except the release test lane; for hosts where release compiles are slow.
    NoReleaseTests,
    /// Guard, product formatting, Clippy and debug tests; the inner development loop.
    Fast,
}

pub(crate) fn run(root: &Path, arguments: &[String]) -> Result<()> {
    let (lane, features) = split_lane(arguments)?;
    validate_features(&features)?;
    crate::inspect_artifacts(root)?;
    let metadata = config::inspect(root)?;
    if lane == Lane::Fast {
        cargo(root, &["fmt", "--all", "--check"], &[], Deny::Nothing)?;
    } else {
        for args in [
            vec![
                "fmt",
                "--manifest-path",
                "xtask/Cargo.toml",
                "--all",
                "--check",
            ],
            vec![
                "clippy",
                "--manifest-path",
                "xtask/Cargo.toml",
                "--all-targets",
                "--locked",
                "--",
                "-D",
                "warnings",
            ],
            vec![
                "test",
                "--manifest-path",
                "xtask/Cargo.toml",
                "--all-targets",
                "--locked",
            ],
            vec!["fmt", "--all", "--check"],
        ] {
            cargo(root, &args, &[], Deny::Nothing)?;
        }
    }
    cargo(
        root,
        &["clippy", "--workspace", "--all-targets", "--locked"],
        &features,
        Deny::Clippy,
    )?;
    cargo(
        root,
        &["test", "--workspace", "--all-targets", "--locked"],
        &features,
        Deny::Nothing,
    )?;
    if lane == Lane::Fast {
        eprintln!(
            "Skipped: tooling format/Clippy/tests, doctests, rustdoc and release tests. \
             A fast lane is not release evidence; run `cargo xtask check` before calling a slice done."
        );
        return Ok(());
    }
    if metadata.packages.iter().any(|package| {
        package.targets.iter().any(|target| {
            target.doctest
                && target
                    .kind
                    .iter()
                    .any(|kind| matches!(kind.as_str(), "lib" | "rlib" | "proc-macro"))
        })
    }) {
        cargo(
            root,
            &["test", "--workspace", "--doc", "--locked"],
            &features,
            Deny::Rustdoc,
        )?;
    }
    cargo(
        root,
        &["doc", "--workspace", "--no-deps", "--locked"],
        &features,
        Deny::Rustdoc,
    )?;
    let has_binary = metadata.packages.iter().any(config::Package::has_binary);
    if lane == Lane::NoReleaseTests {
        eprintln!(
            "Skipped: release tests{}. This lane is not release evidence; run `cargo xtask check` \
             before calling a slice done.",
            if has_binary { "" } else { " (none declared)" }
        );
        return Ok(());
    }
    if has_binary {
        cargo(
            root,
            &[
                "test",
                "--workspace",
                "--all-targets",
                "--release",
                "--locked",
            ],
            &features,
            Deny::Nothing,
        )?;
    }
    Ok(())
}

/// Separate the lane flag from feature arguments; at most one reduced lane is meaningful.
fn split_lane(arguments: &[String]) -> Result<(Lane, Vec<String>)> {
    let mut lane = Lane::Full;
    let mut features = Vec::new();
    for argument in arguments {
        let selected = match argument.as_str() {
            "--fast" => Lane::Fast,
            "--no-release-tests" => Lane::NoReleaseTests,
            _ => {
                features.push(argument.clone());
                continue;
            }
        };
        if lane != Lane::Full {
            return Err("Choose one reduced lane: --fast or --no-release-tests".into());
        }
        lane = selected;
    }
    Ok((lane, features))
}

/// Whose warnings an invocation turns into errors, beyond what `[lints]` already denies.
#[derive(Clone, Copy)]
enum Deny {
    Nothing,
    Clippy,
    Rustdoc,
}

fn cargo(root: &Path, args: &[&str], features: &[String], deny: Deny) -> Result<()> {
    let mut command = Command::new("cargo");
    command.current_dir(root).args(args);
    match deny {
        Deny::Nothing => command.args(features),
        Deny::Clippy => command.args(features).args(["--", "-D", "warnings"]),
        Deny::Rustdoc => deny_rustdoc_warnings(&mut command).args(features),
    };
    eprintln!("Running {command:?}");
    let status = command.status()?;
    if !status.success() {
        return Err(format!("Native check failed ({status}): {command:?}").into());
    }
    Ok(())
}

/// Deny rustdoc warnings without replacing the project's own rustdoc flags.
///
/// Cargo takes rustdoc flags from the first source that has any: the two flag
/// variables, then target tables, then `build.rustdocflags`. Setting a variable
/// here would therefore discard flags the project configured. A variable the
/// caller already set is extended, since Cargo is already ignoring the rest.
/// Otherwise the denial is a command-line `build.rustdocflags` value, which Cargo
/// merges with the configured array; the guard refuses the shapes that cannot merge.
fn deny_rustdoc_warnings(command: &mut Command) -> &mut Command {
    // Command-local environment works in Windows and POSIX shells alike.
    if let Some(mut encoded) = std::env::var_os("CARGO_ENCODED_RUSTDOCFLAGS") {
        encoded.push("\u{1f}-Dwarnings");
        command.env("CARGO_ENCODED_RUSTDOCFLAGS", encoded)
    } else if let Some(mut flags) = std::env::var_os("RUSTDOCFLAGS") {
        flags.push(" -D warnings");
        command.env("RUSTDOCFLAGS", flags)
    } else {
        command.args(["--config", "build.rustdocflags=['-Dwarnings']"])
    }
}

fn validate_features(features: &[String]) -> Result<()> {
    let mut arguments = features.iter();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--no-default-features" | "--all-features" => {}
            "--features" => {
                let names = arguments.next().ok_or("--features requires names")?;
                if names.is_empty() || names.starts_with('-') {
                    return Err("Invalid feature names".into());
                }
            }
            _ => return Err(format!("Unsupported check argument: {argument}").into()),
        }
    }
    Ok(())
}
