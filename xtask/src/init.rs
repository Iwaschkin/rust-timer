//! Bootstrap: apply the baseline's assets to a new Cargo project.
//!
//! `init` runs from the installed skill, because the assets it copies live beside the
//! tooling there and nowhere else. It writes only into a project that has none of the
//! files it would create, edits manifests only by adding whole tables and single keys,
//! and finishes by running the guard, so its result is checked the way every later
//! change is.

use crate::{Result, config};
use std::{
    collections::BTreeMap,
    fmt, fs,
    path::{Path, PathBuf},
    process::Command,
};

/// Configuration copied unchanged into the project root.
const COPIED: [&str; 4] = [
    "clippy.toml",
    "rustfmt.toml",
    "rust-toolchain.toml",
    "deny.toml",
];

const WORKFLOW: &str = ".github/workflows/quality.yml";

/// The alias the guard requires, byte for byte.
const ALIAS: &str = "[alias]\nxtask = \"run --locked --manifest-path xtask/Cargo.toml --\"\n";

/// What `init` did, for review before the project's first check.
#[derive(Debug, Default)]
pub struct InitReport {
    /// Files and directories that did not exist before, relative to the project.
    pub created: Vec<PathBuf>,
    /// Existing files that `init` added to, relative to the project.
    pub changed: Vec<PathBuf>,
    /// Decisions made on the project's behalf, and the steps left to it.
    pub notes: Vec<String>,
}

impl fmt::Display for InitReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (heading, paths) in [("Created", &self.created), ("Changed", &self.changed)] {
            if !paths.is_empty() {
                writeln!(formatter, "{heading}:")?;
                for path in paths {
                    writeln!(formatter, "  {}", path.display())?;
                }
            }
        }
        writeln!(formatter, "Notes:")?;
        for note in &self.notes {
            writeln!(formatter, "  - {note}")?;
        }
        write!(
            formatter,
            "The guard passes. Review the diff, fill AGENTS.md with the project's own commands \
             and promises, then run `cargo xtask check`."
        )
    }
}

/// Apply the baseline in `assets` to the new Cargo project at `project`.
///
/// The project already has its manifest, from `cargo new` or a workspace root, and none
/// of the files `init` creates. Existing projects and refreshes are merged by hand.
///
/// # Errors
/// Returns every file or manifest table that already exists, before anything is
/// written; a failed Cargo, git or file operation; or the guard's diagnostic when the
/// result does not pass it.
pub fn init(assets: &Path, project: &Path) -> Result<InitReport> {
    require_assets(assets)?;
    let project = project
        .canonicalize()
        .map_err(|error| format!("{}: {error}", project.display()))?;
    let root_manifest = project.join("Cargo.toml");
    if !root_manifest.is_file() {
        return Err(
            "init needs the project's Cargo.toml: create the project with `cargo new`, \
                    or write its workspace manifest, first"
                .into(),
        );
    }
    let metadata = config::metadata(&project, false)?;
    if metadata.workspace_root.canonicalize()? != project {
        return Err(format!(
            "{} belongs to the workspace at {}; init applies the baseline at a workspace root",
            project.display(),
            metadata.workspace_root.display()
        )
        .into());
    }
    let members: Vec<&config::Package> = metadata
        .workspace_members
        .iter()
        .filter_map(|id| metadata.packages.iter().find(|package| &package.id == id))
        .collect();
    let channel = config::read(&assets.join("rust-toolchain.toml"))?
        .get("toolchain")
        .and_then(|toolchain| toolchain.get("channel"))
        .and_then(toml::Value::as_str)
        .map(str::to_owned)
        .ok_or("The assets' rust-toolchain.toml names no channel")?;

    let mut report = InitReport::default();
    let mut conflicts: Vec<String> = ["xtask", ".cargo/config.toml", ".cargo/config", WORKFLOW]
        .into_iter()
        .chain(COPIED)
        .filter(|path| project.join(path).exists())
        .map(str::to_owned)
        .collect();
    let manifests = plan_manifests(
        assets,
        &root_manifest,
        &members,
        &channel,
        &mut conflicts,
        &mut report,
    )?;
    if !conflicts.is_empty() {
        return Err(format!(
            "init writes only into a new project, and found {}. Merge these by hand with \
             references/asset-application.md, or refresh with references/maintenance.md",
            conflicts.join(", ")
        )
        .into());
    }
    // Without conflicts, each planned edit must leave a manifest Cargo can read; a
    // failure here is this code's defect, reported before anything is written.
    for (path, text) in &manifests {
        toml::from_str::<toml::Value>(text)
            .map_err(|error| format!("init produced an unreadable {}: {error}", path.display()))?;
    }

    if !git(&project, &["rev-parse", "--is-inside-work-tree"])? {
        if !git(&project, &["init", "--quiet"])? {
            return Err("git init failed".into());
        }
        report.created.push(PathBuf::from(".git"));
        report
            .notes
            .push("Initialised a git repository: the guard reads files as git lists them".into());
    }
    copy_tree(&assets.join("xtask"), &project.join("xtask"))?;
    report.created.push(PathBuf::from("xtask"));
    for name in COPIED {
        fs::copy(assets.join(name), project.join(name))?;
        report.created.push(PathBuf::from(name));
    }
    fs::create_dir_all(project.join(".cargo"))?;
    fs::write(project.join(".cargo/config.toml"), ALIAS)?;
    report.created.push(PathBuf::from(".cargo/config.toml"));
    let template = fs::read_to_string(assets.join("agents-template.md"))?;
    extend(&project, "AGENTS.md", &template, &mut report)?;
    fs::create_dir_all(project.join(".github/workflows"))?;
    fs::copy(assets.join("ci-github-actions.yml"), project.join(WORKFLOW))?;
    report.created.push(PathBuf::from(WORKFLOW));
    if members.iter().any(|package| !package.features.is_empty()) {
        report.notes.push(format!(
            "A package declares [features]: render the `minimal` job described in {WORKFLOW} \
             and add it to the `quality` job's `needs`"
        ));
    }
    let ignore = match fs::read_to_string(project.join(".gitignore")) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(format!(".gitignore: {error}").into()),
    };
    if !ignores_build_output(&ignore) {
        extend(&project, ".gitignore", "target/\n", &mut report)?;
    }
    for (path, text) in manifests {
        fs::write(&path, text)?;
        report
            .changed
            .push(path.strip_prefix(&project)?.to_path_buf());
    }
    if !project.join("Cargo.lock").is_file() {
        cargo(&project, &["generate-lockfile"])?;
        report.created.push(PathBuf::from("Cargo.lock"));
        report
            .notes
            .push("Resolved Cargo.lock; review it and commit it".into());
    }
    report.notes.push(
        "Each crate root needs `//!` documentation for `missing_docs`; `cargo new`'s template \
         has none"
            .into(),
    );
    report.notes.push("Nothing is committed".into());
    crate::inspect(&project).map_err(|error| {
        format!("init applied the baseline, but the guard does not pass yet: {error}")
    })?;
    Ok(report)
}

/// Every asset `init` copies or reads.
fn require_assets(assets: &Path) -> Result<()> {
    for name in [
        "agents-template.md",
        "ci-github-actions.yml",
        "application-profile.toml",
        "xtask/Cargo.toml",
        "xtask/lints.toml",
    ]
    .into_iter()
    .chain(COPIED)
    {
        if !assets.join(name).is_file() {
            return Err(format!(
                "{} holds no {name}: init runs from the installed skill's assets, not from a \
                 project's own tooling",
                assets.display()
            )
            .into());
        }
    }
    Ok(())
}

/// The new text of every manifest `init` edits, keyed by path; anything already present
/// that `init` would have to overwrite goes to `conflicts` instead.
fn plan_manifests(
    assets: &Path,
    root_manifest: &Path,
    members: &[&config::Package],
    channel: &str,
    conflicts: &mut Vec<String>,
    report: &mut InitReport,
) -> Result<BTreeMap<PathBuf, String>> {
    let original = fs::read_to_string(root_manifest)?;
    let root: toml::Value = toml::from_str(&original)?;
    let workspace = root.get("workspace");
    if root.get("lints").is_some() || workspace.is_some_and(|table| table.get("lints").is_some()) {
        conflicts.push("a [lints] table in Cargo.toml".into());
    }
    let application = members.iter().any(|package| package.has_binary());
    if application
        && root
            .get("profile")
            .is_some_and(|profile| profile.get("release").is_some())
    {
        conflicts.push("a [profile.release] table in Cargo.toml".into());
    }
    let lints =
        without_leading_comments(&fs::read_to_string(assets.join("xtask/lints.toml"))?).to_owned();
    let mut text = original.clone();
    ensure_newline(&mut text);
    if let Some(workspace) = workspace {
        match workspace.get("exclude").and_then(toml::Value::as_array) {
            Some(exclude) if exclude.iter().any(|item| item.as_str() == Some("xtask")) => {}
            Some(_) => conflicts.push("a workspace exclude list without xtask".into()),
            None => text = insert_after_header(&text, "[workspace]", "exclude = [\"xtask\"]")?,
        }
        text.push_str("\n# The baseline's native lints; the project's guard checks them.\n");
        text.push_str(&lints.replace("[lints.", "[workspace.lints."));
    } else {
        text.push_str("\n# The baseline's native lints; the project's guard checks them.\n");
        text.push_str(&lints);
    }
    if application {
        let profile = fs::read_to_string(assets.join("application-profile.toml"))?;
        ensure_newline(&mut text);
        text.push_str("\n# An application's release build checks overflow.\n");
        text.push_str(without_leading_comments(&profile));
    }
    let root_manifest = root_manifest.to_path_buf();
    let mut manifests = BTreeMap::from([(root_manifest.clone(), text)]);
    let mut inserted = 0_usize;
    for package in members {
        let path = package.manifest_path.canonicalize()?;
        let mut text = match manifests.remove(&path) {
            Some(text) => text,
            None => {
                let text = fs::read_to_string(&path)?;
                let member: toml::Value = toml::from_str(&text)?;
                if member.get("lints").is_some() {
                    let shown = root_manifest
                        .parent()
                        .and_then(|project| path.strip_prefix(project).ok())
                        .unwrap_or(&path);
                    conflicts.push(format!("a [lints] table in {}", shown.display()));
                }
                text
            }
        };
        if package.rust_version.is_none() {
            text =
                insert_after_header(&text, "[package]", &format!("rust-version = \"{channel}\""))?;
            inserted = inserted.saturating_add(1);
        }
        if workspace.is_some() {
            ensure_newline(&mut text);
            text.push_str("\n[lints]\nworkspace = true\n");
        }
        manifests.insert(path, text);
    }
    if inserted > 0 {
        report.notes.push(format!(
            "Set rust-version = \"{channel}\", the validation pin, in {inserted} package(s); \
             lower it only together with a tested MSRV lane"
        ));
    }
    Ok(manifests)
}

/// Append `addition` to the file at `name` in `project`, creating it when absent.
fn extend(project: &Path, name: &str, addition: &str, report: &mut InitReport) -> Result<()> {
    let path = project.join(name);
    match fs::read_to_string(&path) {
        Ok(mut text) => {
            ensure_newline(&mut text);
            text.push('\n');
            text.push_str(addition);
            fs::write(&path, text)?;
            report.changed.push(PathBuf::from(name));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            fs::write(&path, addition)?;
            report.created.push(PathBuf::from(name));
        }
        Err(error) => return Err(format!("{name}: {error}").into()),
    }
    Ok(())
}

fn ensure_newline(text: &mut String) {
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
}

/// `text` with `line` added directly under the first `header` line.
fn insert_after_header(text: &str, header: &str, line: &str) -> Result<String> {
    let mut output = String::with_capacity(text.len().saturating_add(line.len()).saturating_add(1));
    let mut inserted = false;
    for current in text.split_inclusive('\n') {
        output.push_str(current);
        let rest = current.trim().strip_prefix(header);
        if !inserted
            && rest.is_some_and(|rest| rest.is_empty() || rest.trim_start().starts_with('#'))
        {
            if !current.ends_with('\n') {
                output.push('\n');
            }
            output.push_str(line);
            output.push('\n');
            inserted = true;
        }
    }
    if inserted {
        Ok(output)
    } else {
        Err(format!("No `{header}` line to add `{line}` under; add it by hand").into())
    }
}

/// `text` after its opening comment block: an asset's merge instructions, not policy.
fn without_leading_comments(text: &str) -> &str {
    let mut rest = text;
    while let Some((line, tail)) = rest.split_once('\n') {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            rest = tail;
        } else {
            break;
        }
    }
    rest
}

/// Whether an ignore file already ignores `target` at every depth; `/target`, as
/// `cargo new` writes it, covers only the root.
fn ignores_build_output(text: &str) -> bool {
    text.lines().any(|line| {
        matches!(
            line.trim(),
            "target" | "target/" | "**/target" | "**/target/"
        )
    })
}

fn copy_tree(from: &Path, to: &Path) -> Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        if entry.file_name() == "target" {
            continue;
        }
        let kind = entry.file_type()?;
        let destination = to.join(entry.file_name());
        if kind.is_dir() {
            copy_tree(&entry.path(), &destination)?;
        } else if kind.is_file() {
            fs::copy(entry.path(), destination)?;
        } else {
            return Err(format!("Unsupported asset entry: {}", entry.path().display()).into());
        }
    }
    Ok(())
}

/// Whether git succeeded; a git that cannot start is an error, not a `false`.
fn git(project: &Path, args: &[&str]) -> Result<bool> {
    let output = Command::new("git")
        .current_dir(project)
        .args(args)
        .output()
        .map_err(|error| {
            format!("Git unavailable: {error}; the guard reads the project as git lists it")
        })?;
    Ok(output.status.success())
}

fn cargo(project: &Path, args: &[&str]) -> Result<()> {
    let status = Command::new("cargo")
        .current_dir(project)
        .args(args)
        .status()
        .map_err(|error| format!("Cargo unavailable: {error}"))?;
    if !status.success() {
        return Err(format!("cargo {} failed: {status}", args.join(" ")).into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ignores_build_output, insert_after_header, without_leading_comments};

    #[test]
    fn a_key_goes_under_its_table_header_and_nowhere_else() {
        let manifest = "[package]\nname = \"demo\"\n\n[dependencies]\n";
        assert_eq!(
            insert_after_header(manifest, "[package]", "rust-version = \"1.98.1\"").unwrap(),
            "[package]\nrust-version = \"1.98.1\"\nname = \"demo\"\n\n[dependencies]\n"
        );
        assert!(insert_after_header("[package]  # the crate\n", "[package]", "x = 1").is_ok());
        assert!(insert_after_header("[package.metadata]\n", "[package]", "x = 1").is_err());
        assert!(insert_after_header("name = \"demo\"\n", "[package]", "x = 1").is_err());
    }

    #[test]
    fn an_asset_loses_its_merge_instructions_and_keeps_its_rules() {
        let asset =
            "# Merge into a package.\n# More.\n\n[lints.rust]\n# kept\nunsafe_code = \"forbid\"\n";
        assert_eq!(
            without_leading_comments(asset),
            "[lints.rust]\n# kept\nunsafe_code = \"forbid\"\n"
        );
    }

    #[test]
    fn only_an_unanchored_target_ignores_the_tooling_build_output() {
        assert!(!ignores_build_output("/target\n"));
        assert!(ignores_build_output("/target\ntarget/\n"));
        assert!(ignores_build_output("  target  \n"));
        assert!(!ignores_build_output("targets/\n"));
    }
}
