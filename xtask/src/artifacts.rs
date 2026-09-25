use crate::{Result, files};
use std::{
    fs,
    path::{Path, PathBuf},
};

mod shell;

pub(crate) fn inspect(root: &Path) -> Result<()> {
    let listed = files::list(root)?;
    let canonical = root.canonicalize()?;
    for relative in listed {
        if let Some(artifact) = python_component(&relative) {
            return Err(format!("Python artifact: {}", artifact.display()).into());
        }
        let path = root.join(&relative);
        if fs::symlink_metadata(&path)?.file_type().is_symlink() {
            let target = path.canonicalize().map_err(|error| {
                format!("Unresolvable symlink: {}: {error}", relative.display())
            })?;
            if !target.starts_with(&canonical) {
                return Err(format!(
                    "Symlink leaves the project: {} -> {}",
                    relative.display(),
                    target.display()
                )
                .into());
            }
            // A directory inside the project: git lists its files in their own right.
            if !target.is_file() {
                continue;
            }
        } else if !path.is_file() {
            return Err(format!("Unsupported filesystem entry: {}", relative.display()).into());
        }
        inspect_file(&path, &relative)?;
    }
    Ok(())
}

/// The leading part of `relative` that ends in a Python file or directory name.
fn python_component(relative: &Path) -> Option<PathBuf> {
    let mut prefix = PathBuf::new();
    for component in relative.components() {
        prefix.push(component);
        let name = component.as_os_str().to_string_lossy().to_ascii_lowercase();
        if python_name(&name) {
            return Some(prefix);
        }
    }
    None
}

fn python_name(name: &str) -> bool {
    let suffix = name.rsplit('.').next().unwrap_or("");
    matches!(
        suffix,
        "py" | "pyi" | "pyc" | "pyo" | "pyd" | "pyz" | "ipynb" | "whl"
    ) || matches!(
        name,
        "pyproject.toml"
            | "uv.lock"
            | "uv.toml"
            | "pylock.toml"
            | ".pypirc"
            | ".coveragerc"
            | "poetry.lock"
            | "pdm.lock"
            | "pipfile"
            | "pipfile.lock"
            | "setup.cfg"
            | "tox.ini"
            | "pytest.ini"
            | "pyrightconfig.json"
            | "mypy.ini"
            | ".mypy.ini"
            | "ruff.toml"
            | ".ruff.toml"
            | ".python-version"
            | ".python-versions"
            | "pyvenv.cfg"
            | ".venv"
            | "venv"
            | "__pycache__"
            | ".pytest_cache"
            | ".mypy_cache"
            | ".ruff_cache"
            | ".pre-commit-config.yaml"
            | ".pre-commit-config.yml"
    ) || (name.starts_with("requirements") && (name.ends_with(".txt") || name.ends_with(".in")))
        || (name.starts_with("constraints") && (name.ends_with(".txt") || name.ends_with(".in")))
        || (name.starts_with("pylock.") && name.ends_with(".toml"))
        || name.ends_with(".egg-info")
        || name.ends_with(".dist-info")
}

fn inspect_file(path: &Path, relative: &Path) -> Result<()> {
    let bytes = fs::read(path)?;
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    let text = match std::str::from_utf8(&bytes) {
        Ok(text) => text,
        Err(error)
            if matches!(
                extension,
                "rs" | "toml" | "yaml" | "yml" | "sh" | "ps1" | "md"
            ) =>
        {
            return Err(error.into());
        }
        Err(_) => return Ok(()), // Opaque assets are not executable text.
    };
    if path
        .file_name()
        .is_some_and(|name| name == "Cargo.toml" || name == "Cargo.lock")
    {
        let value: toml::Value = toml::from_str(text)?;
        inspect_dependencies(&value, relative)?;
    }
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    if let Some(line) = python_route(text, name, extension) {
        return Err(format!("Python tooling: {}:{}", relative.display(), line).into());
    }
    if extension == "rs"
        && let Some(line) = text
            .lines()
            .position(|line| line.chars().count() > LINE_CAP)
    {
        return Err(format!(
            "Line exceeds {LINE_CAP} columns, and rustfmt cannot format over-long statements or \
             macro bodies (cargo fmt --check still passes): {}:{}. Split the statement; give a \
             long literal a `\\` continuation or concat!.",
            relative.display(),
            line + 1
        )
        .into());
    }
    Ok(())
}

/// Raw column cap for Rust source. Lines past it are exactly the ones the formatter skips.
const LINE_CAP: usize = 160;

/// Executable contexts only: Rust subprocess literals, scripts and shebang files,
/// container build steps and workflow commands. Prose and data are never scanned,
/// so a README, a Lambda runtime declaration or a step name cannot fail the guard.
fn python_route(text: &str, name: &str, extension: &str) -> Option<usize> {
    let lower_name = name.to_ascii_lowercase();
    let numbered = || {
        text.lines()
            .enumerate()
            .map(|(index, line)| (index + 1, line))
    };
    if extension == "rs" {
        return numbered()
            .find(|(_, line)| rust_subprocess(line))
            .map(|(number, _)| number);
    }
    let script = matches!(extension, "sh" | "ps1" | "cmd" | "bat" | "mk")
        || matches!(lower_name.as_str(), "makefile" | "justfile")
        || text.starts_with("#!");
    if script {
        return numbered()
            .find(|(_, line)| shell::runs_python(line, shell::Dialect::for_extension(extension)))
            .map(|(number, _)| number);
    }
    let container = matches!(lower_name.as_str(), "dockerfile" | "containerfile")
        || lower_name.ends_with(".dockerfile");
    if container {
        return numbered()
            .find(|(_, line)| {
                let step = line.trim_start();
                step.get(..4)
                    .is_some_and(|keyword| keyword.eq_ignore_ascii_case("run "))
                    && step.get(4..).is_some_and(shell_route)
            })
            .map(|(number, _)| number);
    }
    if matches!(extension, "yml" | "yaml") {
        return workflow_route(text);
    }
    None
}

/// Literal subprocess routes only; this is not Rust data-flow analysis.
fn rust_subprocess(line: &str) -> bool {
    let compact: String = line.chars().filter(|ch| !ch.is_whitespace()).collect();
    [
        "python",
        "python3",
        "python.exe",
        "py",
        "pip",
        "pip3",
        "uv",
        "ruff",
        "pytest",
        "pyright",
        "mypy",
        "black",
    ]
    .iter()
    .any(|name| compact.contains(&format!("Command::new(\"{name}\")")))
}

/// A shell line that starts an interpreter, installer or Python-only tool.
///
/// Only command positions are read: the first word of each simple command, after
/// assignments, shell keywords and wrappers such as `env` or `sh -c`. Arguments
/// and comments name a tool without running it, so the test filter
/// `rejects_python_artifacts` is not an invocation. This is not a shell parser:
/// a command assembled at run time, or reached through `find -exec`, is review's.
fn shell_route(line: &str) -> bool {
    shell::runs_python(line, shell::Dialect::Posix)
}

/// Only `run`, `shell`, `script` and `uses` values are commands in a workflow file,
/// including the lines of a block scalar under such a key.
fn workflow_route(text: &str) -> Option<usize> {
    let mut block: Option<usize> = None;
    for (index, line) in text.lines().enumerate() {
        let number = index + 1;
        let indent = line.len() - line.trim_start().len();
        if let Some(column) = block {
            if line.trim().is_empty() {
                continue;
            }
            if indent > column {
                if shell_route(line) {
                    return Some(number);
                }
                continue;
            }
            block = None;
        }
        let item = line.trim_start();
        let item = item.strip_prefix("- ").map_or(item, str::trim_start);
        let column = line.len() - item.len();
        let Some((key, value)) = item.split_once(':') else {
            continue;
        };
        let (key, value) = (key.trim(), value.trim());
        if key == "uses" {
            let lower = value.to_ascii_lowercase();
            if lower.contains("actions/setup-python@") || lower.contains("astral-sh/setup-uv@") {
                return Some(number);
            }
            continue;
        }
        if !matches!(key, "run" | "shell" | "script") {
            continue;
        }
        if value.starts_with('|') || value.starts_with('>') {
            block = Some(column);
        } else if shell_route(value) {
            return Some(number);
        }
    }
    None
}

fn python_dependency(name: &str) -> bool {
    matches!(
        name,
        "python"
            | "python3"
            | "pyo3"
            | "pyo3-ffi"
            | "pyo3-build-config"
            | "cpython"
            | "maturin"
            | "rustpython"
            | "rustpython-vm"
            | "python3-dll-a"
    )
}

fn inspect_dependencies(value: &toml::Value, path: &Path) -> Result<()> {
    match value {
        toml::Value::Table(table) => {
            for (key, value) in table {
                if python_dependency(key)
                    || (matches!(key.as_str(), "package" | "name")
                        && value.as_str().is_some_and(python_dependency))
                {
                    return Err(format!("Python dependency: {} ({key})", path.display()).into());
                }
                inspect_dependencies(value, path)?;
            }
        }
        toml::Value::Array(values) => {
            for value in values {
                inspect_dependencies(value, path)?;
            }
        }
        toml::Value::String(_)
        | toml::Value::Integer(_)
        | toml::Value::Float(_)
        | toml::Value::Boolean(_)
        | toml::Value::Datetime(_) => {}
    }
    Ok(())
}
