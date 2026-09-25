//! Project-owned checks for the strict, safe Rust baseline.

use std::path::Path;

mod artifacts;
mod check;
mod config;
mod files;
mod gates;
mod init;

/// A failed check or unavailable prerequisite; neither constitutes success.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub use gates::{Gates, gates};
pub use init::{InitReport, init};

/// Inspect the files git lists without executing or modifying project code.
///
/// # Errors
/// Returns a diagnostic for forbidden artifacts, listed build output, a symlink out
/// of the project, unreadable input, or a project git cannot list.
pub fn inspect_artifacts(root: &Path) -> Result<()> {
    artifacts::inspect(root)
}

/// Check native configuration and package policy without changing it.
///
/// # Errors
/// Returns the missing or weakened requirement, or a Cargo discovery failure.
pub fn inspect_configuration(root: &Path) -> Result<()> {
    config::inspect(root).map(|_| ())
}

/// Inspect the complete project before running its build or tests.
///
/// # Errors
/// Returns the first artifact or configuration violation.
pub fn inspect(root: &Path) -> Result<()> {
    inspect_artifacts(root)?;
    inspect_configuration(root)
}

/// Run native checks for one explicit feature selection on the current host.
///
/// `--fast` runs the guard, product formatting, Clippy and debug tests only;
/// `--no-release-tests` runs everything except the release test lane. Both print
/// what they skipped and neither is release evidence; the full run is.
///
/// # Errors
/// Returns any failed check or missing prerequisite; does not install tools.
pub fn check(root: &Path, arguments: &[String]) -> Result<()> {
    check::run(root, arguments)
}
