//! The project's files, as git lists them.

use crate::Result;
use std::{
    collections::BTreeSet,
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};

/// Every file this project owns, relative to `root`, in order: the tracked files and
/// the new files git does not ignore.
///
/// Git decides, so an ignored `.venv`, `node_modules` or build directory is not the
/// project's, and neither is a nested repository: a worktree under `.claude/worktrees`
/// or a submodule is another project. A file deleted from the working tree is not
/// listed, and neither is anything under a directory [`outside_project`] names. Build
/// output that git lists is an error, because it would be read as the project's own.
pub(crate) fn list(root: &Path) -> Result<Vec<PathBuf>> {
    if !root.is_dir() {
        return Err(format!("Not a project directory: {}", root.display()).into());
    }
    let output = Command::new("git")
        .current_dir(root)
        .args([
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ])
        .output()
        .map_err(|error| {
            format!("Git unavailable: {error}; the guard reads the project as git lists it")
        })?;
    if !output.status.success() {
        return Err(format!(
            "Git cannot list this project, and the guard reads the project as git lists it, \
             so it must be inside a git repository: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into());
    }
    let mut files = BTreeSet::new();
    for entry in output.stdout.split(|byte| *byte == 0) {
        let entry = std::str::from_utf8(entry)
            .map_err(|error| format!("Git listed a path that is not UTF-8: {error}"))?;
        // An untracked nested repository is listed as its directory, with a slash.
        if entry.is_empty() || entry.ends_with('/') {
            continue;
        }
        let relative: PathBuf = entry.split('/').collect();
        if outside_project(&relative) {
            continue;
        }
        match fs::symlink_metadata(root.join(&relative)) {
            // A submodule is listed as its directory: another project.
            Ok(metadata) if metadata.is_dir() => {}
            Ok(_) => {
                files.insert(relative);
            }
            // Tracked, but deleted from the working tree.
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(format!("{}: {error}", relative.display()).into()),
        }
    }
    if let Some(output) = files.iter().find(|relative| build_output(relative)) {
        return Err(format!(
            "Build output is not ignored: {}. Ignore `target/` at any depth; `/target`, as \
             `cargo new` writes it, leaves the tooling's xtask/target listed",
            output.display()
        )
        .into());
    }
    Ok(files.into_iter().collect())
}

/// Cargo's build directories for the product workspace and the tooling workspace.
fn build_output(relative: &Path) -> bool {
    relative.starts_with("target") || relative.starts_with("xtask/target")
}

/// Trees under the project root that are not this project's to check even when git
/// lists them: skills the agent host installed.
///
/// The baseline answers for what it puts into a project. A skill installed beside it
/// may carry any language and its own example manifests. This is the one definition,
/// so the artifact rules, project discovery and review numbers agree; the same file
/// anywhere else, including elsewhere under a host directory, is the project's.
fn outside_project(relative: &Path) -> bool {
    [".claude/skills", ".agents/skills"]
        .iter()
        .any(|tree| relative.starts_with(tree))
}
