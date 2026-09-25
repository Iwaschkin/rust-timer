//! A disposable project directory that git manages, shared by the guard's tests.

use std::{
    fs, io,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);

/// A fresh repository in the temporary directory, removed when dropped.
///
/// The guard reads a project as git lists it, so every fixture is a repository.
pub struct Fixture(PathBuf);

impl Fixture {
    /// Creates the directory and its repository.
    ///
    /// # Errors
    /// Returns the I/O or git failure.
    pub fn new() -> io::Result<Self> {
        let path = std::env::temp_dir().join(format!(
            "rust-quality-fixture-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path)?;
        let fixture = Self(path);
        repository(fixture.path())?;
        Ok(fixture)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let removed = fs::remove_dir_all(&self.0);
        assert!(
            removed.is_ok(),
            "fixture cleanup failed: {}: {removed:?}",
            self.0.display()
        );
    }
}

/// Makes `directory` a repository whose ignore rules are its own.
///
/// The excludes file points at nothing, so a developer's global ignore rules cannot
/// hide a fixture's files from the guard.
///
/// # Errors
/// Returns the I/O or git failure.
pub fn repository(directory: &Path) -> io::Result<()> {
    let excludes = directory.join(".git").join("no-global-excludes");
    for args in [
        vec!["init", "--quiet"],
        vec![
            "config",
            "core.excludesFile",
            excludes
                .to_str()
                .ok_or_else(|| io::Error::other("fixture path is not UTF-8"))?,
        ],
    ] {
        let output = Command::new("git")
            .current_dir(directory)
            .args(&args)
            .output()?;
        if !output.status.success() {
            return Err(io::Error::other(format!(
                "git {args:?} failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }
    }
    Ok(())
}
