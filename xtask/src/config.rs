use crate::Result;
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Deserialize)]
pub(crate) struct Metadata {
    pub(crate) packages: Vec<Package>,
    pub(crate) workspace_members: Vec<String>,
    pub(crate) workspace_root: PathBuf,
}

#[derive(Deserialize)]
pub(crate) struct Package {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) manifest_path: PathBuf,
    pub(crate) rust_version: Option<String>,
    pub(crate) targets: Vec<Target>,
    #[serde(default)]
    pub(crate) features: BTreeMap<String, Vec<String>>,
}

impl Package {
    pub(crate) fn has_binary(&self) -> bool {
        self.targets
            .iter()
            .any(|target| target.kind.iter().any(|kind| kind == "bin"))
    }
}

/// What Cargo says about the workspace at `root`, without resolving dependencies.
/// `locked` refuses a missing or outdated lockfile; bootstrap runs before one exists.
pub(crate) fn metadata(root: &Path, locked: bool) -> Result<Metadata> {
    let mut command = Command::new("cargo");
    command
        .current_dir(root)
        .args(["metadata", "--no-deps", "--format-version", "1"]);
    if locked {
        command.arg("--locked");
    }
    let output = command
        .output()
        .map_err(|error| format!("Cargo discovery unavailable: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "Cargo discovery failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}

#[derive(Deserialize)]
pub(crate) struct Target {
    pub(crate) kind: Vec<String>,
    pub(crate) doctest: bool,
    doc: bool,
    test: bool,
}

pub(crate) fn read(path: &Path) -> Result<toml::Value> {
    let source =
        fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    toml::from_str(&source).map_err(|error| format!("{}: {error}", path.display()).into())
}

pub(crate) fn inspect(root: &Path) -> Result<Metadata> {
    let root = root.canonicalize()?;
    // Configuration routes come first: they can silently override everything checked below.
    let config = read(&root.join(".cargo/config.toml"))?;
    let alias = config
        .get("alias")
        .and_then(|value| value.get("xtask"))
        .and_then(toml::Value::as_str);
    if alias != Some("run --locked --manifest-path xtask/Cargo.toml --") {
        return Err("Missing native xtask alias with its locked independent manifest".into());
    }
    if root.join(".cargo/config").is_file() {
        return Err(
            "Legacy Cargo configuration: Cargo reads .cargo/config and ignores the \
             .cargo/config.toml beside it; merge it into config.toml"
                .into(),
        );
    }
    inspect_cargo_configuration(&root)?;
    inspect_environment()?;
    let manifest = read(&root.join("Cargo.toml"))?;
    let expected = read(&root.join("xtask/lints.toml"))?;
    let canonical: toml::Value = toml::from_str(include_str!("../lints.toml"))?;
    require_lints(&expected, &canonical, Path::new("xtask/lints.toml"))?;
    let workspace = manifest.get("workspace").is_some();
    let policy = manifest.get("workspace").unwrap_or(&manifest);
    require_lints(policy, &canonical, &root.join("Cargo.toml"))?;
    let tooling = read(&root.join("xtask/Cargo.toml"))?;
    require_lints(&tooling, &canonical, &root.join("xtask/Cargo.toml"))?;
    if tooling.get("workspace").is_none()
        || tooling
            .get("package")
            .and_then(|package| package.get("publish"))
            .and_then(toml::Value::as_bool)
            != Some(false)
    {
        return Err("Tooling must be an independent, non-published workspace".into());
    }
    for file in ["Cargo.lock", "xtask/Cargo.lock", "AGENTS.md"] {
        if !root.join(file).is_file() {
            return Err(format!("Missing required file: {file}").into());
        }
    }
    for directory in [root.clone(), root.join("xtask")] {
        for (file, baseline) in [
            ("clippy.toml", include_str!("../clippy.toml")),
            ("rustfmt.toml", include_str!("../rustfmt.toml")),
        ] {
            let current = read(&directory.join(file))?;
            let required: toml::Value = toml::from_str(baseline)?;
            require_values(&current, &required, file)?;
        }
    }
    let toolchain = read(&root.join("rust-toolchain.toml"))?;
    let channel = toolchain
        .get("toolchain")
        .and_then(|value| value.get("channel"))
        .and_then(toml::Value::as_str)
        .ok_or("Missing pinned toolchain")?;
    if channel.split('.').count() != 3
        || !channel.chars().all(|ch| ch.is_ascii_digit() || ch == '.')
    {
        return Err("Toolchain must be an exact stable release".into());
    }
    let deny = read(&root.join("deny.toml"))?;
    let advisory = deny.get("advisories").ok_or("Missing advisory policy")?;
    if advisory.get("yanked").and_then(toml::Value::as_str) != Some("deny") {
        return Err("Advisory policy must deny yanked dependencies".into());
    }
    let metadata = metadata(&root, true)?;
    if metadata.workspace_root.canonicalize()? != root || metadata.workspace_members.is_empty() {
        return Err("Cargo discovery must describe this nonempty project".into());
    }
    let mut manifests = BTreeSet::from([root.join("Cargo.toml"), root.join("xtask/Cargo.toml")]);
    for id in &metadata.workspace_members {
        let package = metadata
            .packages
            .iter()
            .find(|package| &package.id == id)
            .ok_or("Cargo omitted a member package")?;
        let path = package.manifest_path.canonicalize()?;
        if !path.starts_with(&root) || path.starts_with(root.join("xtask")) {
            return Err("Product members must be inside the project and outside tooling".into());
        }
        let member = read(&path)?;
        if workspace
            && member
                .get("lints")
                .and_then(|value| value.get("workspace"))
                .and_then(toml::Value::as_bool)
                != Some(true)
        {
            return Err(format!("Missing lint inheritance: {}", path.display()).into());
        }
        if package.rust_version.is_none() {
            return Err(format!("Missing rust-version: {}", package.name).into());
        }
        for target in &package.targets {
            if target
                .kind
                .iter()
                .any(|kind| matches!(kind.as_str(), "lib" | "rlib" | "proc-macro"))
                && (!target.doctest || !target.doc || !target.test)
            {
                return Err(format!("Disabled library verification: {}", package.name).into());
            }
            if target.kind.iter().any(|kind| kind == "bin") && !target.test {
                return Err(format!("Disabled binary verification: {}", package.name).into());
            }
        }
        if package.has_binary() {
            require_overflow_checks(&manifest)?;
        }
        manifests.insert(path);
    }
    // Every manifest git lists belongs to this workspace or to the tooling. A nested
    // repository, such as a worktree, is another project and is not listed.
    for relative in crate::files::list(&root)? {
        if relative
            .file_name()
            .is_some_and(|name| name == "Cargo.toml")
            && !manifests.contains(&root.join(&relative).canonicalize()?)
        {
            return Err(format!("Unaccounted Cargo project: {}", relative.display()).into());
        }
    }
    Ok(metadata)
}

/// Flag variables Cargo reads for rustc and rustdoc, in any `[env]` table or the process.
const FLAG_VARIABLES: [&str; 6] = [
    "RUSTFLAGS",
    "RUSTDOCFLAGS",
    "CARGO_ENCODED_RUSTFLAGS",
    "CARGO_ENCODED_RUSTDOCFLAGS",
    "CARGO_BUILD_RUSTFLAGS",
    "CARGO_BUILD_RUSTDOCFLAGS",
];

fn flag_variable(name: &str) -> bool {
    FLAG_VARIABLES.contains(&name)
        || (name.starts_with("CARGO_TARGET_")
            && (name.ends_with("_RUSTFLAGS") || name.ends_with("_RUSTDOCFLAGS")))
}

fn target_rustdoc_variable(name: &str) -> bool {
    name.starts_with("CARGO_TARGET_") && name.ends_with("_RUSTDOCFLAGS")
}

fn target_rustdoc_error(location: &str) -> String {
    format!(
        "{location} replaces build.rustdocflags, where the check runner adds its warning \
         denial; move these flags to [build] rustdocflags, RUSTDOCFLAGS or CARGO_ENCODED_RUSTDOCFLAGS"
    )
}

/// Where the overflow policy may be written, and the phrase every other route is refused with.
const OVERFLOW_OWNER: &str = "the overflow policy belongs in Cargo.toml, under [profile]";

/// An application's release profile checks overflow, and no profile entry, package
/// override or custom profile in the manifest switches the checks off again.
fn require_overflow_checks(manifest: &toml::Value) -> Result<()> {
    let profiles = manifest.get("profile");
    if profiles
        .and_then(|value| value.get("release"))
        .and_then(|value| value.get("overflow-checks"))
        .and_then(toml::Value::as_bool)
        != Some(true)
    {
        return Err("Application release profile must enable overflow-checks".into());
    }
    let mut entries = Vec::new();
    overflow_entries(profiles, &mut vec!["profile".to_owned()], &mut entries);
    if let Some((location, _)) = entries
        .iter()
        .find(|(_, value)| value.as_bool() != Some(true))
    {
        return Err(format!("Application profile disables overflow-checks: {location}").into());
    }
    Ok(())
}

/// Every `overflow-checks` entry at or below `value`, with its dotted location.
fn overflow_entries<'a>(
    value: Option<&'a toml::Value>,
    trail: &mut Vec<String>,
    entries: &mut Vec<(String, &'a toml::Value)>,
) {
    let Some(toml::Value::Table(table)) = value else {
        return;
    };
    for (key, child) in table {
        trail.push(key.clone());
        if key == "overflow-checks" {
            entries.push((trail.join("."), child));
        }
        overflow_entries(Some(child), trail, entries);
        trail.pop();
    }
}

/// A token that can lower a lint level. `-D`, `-F` and linker flags only raise or tune;
/// `-C` tunes too, except for the one codegen option the baseline has a policy for.
/// These are rustc's lint-level options; any other route to a level is review work.
fn lowers_a_level(token: &str) -> bool {
    let token = token.trim();
    token.starts_with("-A")
        || token.starts_with("-W")
        || token.starts_with("--allow")
        || token.starts_with("--warn")
        // A force-warned lint stays a warning under `-D warnings`.
        || token.starts_with("--force-warn")
        || token.starts_with("--cap-lints")
}

fn flag_tokens(value: &toml::Value) -> Vec<String> {
    match value {
        toml::Value::String(text) => text
            .split(|ch: char| ch.is_whitespace() || ch == '\u{1f}')
            .map(str::to_owned)
            .collect(),
        toml::Value::Array(values) => values.iter().flat_map(flag_tokens).collect(),
        // `[env]` entries may be tables such as `{ value = "...", force = true }`.
        toml::Value::Table(table) => table.get("value").map(flag_tokens).unwrap_or_default(),
        toml::Value::Integer(_)
        | toml::Value::Float(_)
        | toml::Value::Boolean(_)
        | toml::Value::Datetime(_) => Vec::new(),
    }
}

/// `-C overflow-checks=…`, in either spelling rustc accepts, replaces the profile's answer.
fn sets_overflow_checks(token: &str) -> bool {
    token.contains("overflow-checks") || token.contains("overflow_checks")
}

fn require_no_lowering(value: &toml::Value, location: &str) -> Result<()> {
    let tokens = flag_tokens(value);
    if let Some(token) = tokens.iter().find(|token| lowers_a_level(token)) {
        return Err(format!(
            "{location} can lower a lint level ({token}); lint levels belong in [lints]"
        )
        .into());
    }
    if let Some(token) = tokens.iter().find(|token| sets_overflow_checks(token)) {
        return Err(format!("{location} sets overflow checks ({token}); {OVERFLOW_OWNER}").into());
    }
    Ok(())
}

/// The check runner adds its rustdoc warning denial to `build.rustdocflags`, and Cargo
/// merges that with a configured array. Cargo refuses to merge it with a string, and
/// ignores `build.rustdocflags` altogether once a target table sets rustdoc flags,
/// which would drop the denial without a word.
fn require_mergeable(value: &toml::Value, trail: &[String], location: &str) -> Result<()> {
    if trail.first().is_some_and(|table| table == "target") {
        return Err(target_rustdoc_error(location).into());
    }
    if !value.is_array() {
        return Err(format!(
            "{location} must be an array, so that Cargo can merge the check runner's warning \
             denial with it"
        )
        .into());
    }
    Ok(())
}

/// Inspect every configuration file Cargo merges into a command run at the root:
/// the project's, each ancestor directory's and Cargo home's.
///
/// This rejects includes and policy-overriding values in each file. It does not resolve
/// Cargo's precedence between them: a route that could override the policy is
/// refused wherever it is written.
fn inspect_cargo_configuration(root: &Path) -> Result<()> {
    let home = std::env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::home_dir().map(|home| home.join(".cargo")));
    let directories = root
        .ancestors()
        .map(|directory| directory.join(".cargo"))
        .chain(home);
    for directory in directories {
        // Elsewhere than the root a legacy file is still read, so it is inspected too.
        for name in ["config.toml", "config"] {
            let path = directory.join(name);
            if !path.is_file() {
                continue;
            }
            let config = read(&path)?;
            let shown = path.strip_prefix(root).unwrap_or(&path);
            if config.get("include").is_some() {
                return Err(format!(
                    "{}: Cargo configuration includes are unsupported; merge the included \
                     settings into the inspected configuration files",
                    shown.display()
                )
                .into());
            }
            inspect_flags(&config, shown)?;
            let mut entries = Vec::new();
            overflow_entries(
                config.get("profile"),
                &mut vec!["profile".to_owned()],
                &mut entries,
            );
            if let Some((location, _)) = entries.first() {
                return Err(format!(
                    "{}: {location} overrides the manifest; {OVERFLOW_OWNER}",
                    shown.display()
                )
                .into());
            }
        }
    }
    Ok(())
}

/// Reject `rustflags`/`rustdocflags` values and flag variables that could lower a lint level
/// or replace the overflow policy.
///
/// Tuning and warning promotion stay legal through supported flag routes.
fn inspect_flags(config: &toml::Value, path: &Path) -> Result<()> {
    fn walk(value: &toml::Value, trail: &mut Vec<String>, path: &Path) -> Result<()> {
        let toml::Value::Table(table) = value else {
            return Ok(());
        };
        let in_env = trail.last().is_some_and(|key| key == "env");
        for (key, child) in table {
            let flag_key = matches!(key.as_str(), "rustflags" | "rustdocflags")
                || (in_env && flag_variable(key));
            if flag_key {
                let location = format!("{}: {}.{key}", path.display(), trail.join("."));
                require_no_lowering(child, &location)?;
                if key == "rustdocflags" {
                    require_mergeable(child, trail, &location)?;
                } else if in_env && target_rustdoc_variable(key) {
                    return Err(target_rustdoc_error(&location).into());
                }
            }
            trail.push(key.clone());
            walk(child, trail, path)?;
            trail.pop();
        }
        Ok(())
    }
    walk(config, &mut Vec::new(), path)
}

/// The same routes in the guard's own environment; CI and shells set them too.
fn inspect_environment() -> Result<()> {
    for (name, value) in std::env::vars_os() {
        let Some(name) = name.to_str() else {
            continue;
        };
        if name.starts_with("CARGO_PROFILE_") && name.ends_with("_OVERFLOW_CHECKS") {
            return Err(format!(
                "environment variable {name} overrides the manifest; {OVERFLOW_OWNER}"
            )
            .into());
        }
        if flag_variable(name) {
            let value = toml::Value::String(value.to_string_lossy().into_owned());
            require_no_lowering(&value, &format!("environment variable {name}"))?;
            if target_rustdoc_variable(name) {
                return Err(target_rustdoc_error(&format!("environment variable {name}")).into());
            }
        }
    }
    Ok(())
}

fn require_lints(actual: &toml::Value, canonical: &toml::Value, path: &Path) -> Result<()> {
    let actual = actual
        .get("lints")
        .ok_or_else(|| format!("Missing native lints: {}", path.display()))?;
    let groups = canonical["lints"]
        .as_table()
        .ok_or("Invalid canonical lints")?;
    for (group, rules) in groups {
        for (name, value) in rules.as_table().ok_or("Invalid canonical lint group")? {
            let configured = actual.get(group).and_then(|value| value.get(name));
            let expected = lint_level(value);
            let level = configured.and_then(lint_level);
            if level != expected && !(level == Some("forbid") && expected == Some("deny")) {
                return Err(
                    format!("Weakened native lint {group}::{name}: {}", path.display()).into(),
                );
            }
            if let Some(priority) = value.get("priority")
                && configured.and_then(|value| value.get("priority")) != Some(priority)
            {
                return Err(format!("Wrong lint group priority: {group}::{name}").into());
            }
        }
    }
    for (group, rules) in actual.as_table().ok_or("Invalid native lint table")? {
        for (name, value) in rules.as_table().ok_or("Invalid native lint group")? {
            if !matches!(lint_level(value), Some("deny" | "forbid")) {
                return Err(format!(
                    "Weakened additional lint {group}::{name}: {}",
                    path.display()
                )
                .into());
            }
        }
    }
    Ok(())
}

fn lint_level(value: &toml::Value) -> Option<&str> {
    value
        .as_str()
        .or_else(|| value.get("level").and_then(toml::Value::as_str))
}

fn require_values(actual: &toml::Value, required: &toml::Value, context: &str) -> Result<()> {
    for (name, value) in required
        .as_table()
        .ok_or("Invalid canonical configuration")?
    {
        if actual.get(name) != Some(value) {
            return Err(format!("Changed required setting: {context}: {name}").into());
        }
    }
    Ok(())
}
