//! Regressions for Python entering otherwise native project output.

mod common;

use common::Fixture;
use std::fs;

fn write(fixture: &Fixture, name: &str, content: &str) -> rust_quality::Result<()> {
    let path = fixture.path().join(name);
    fs::create_dir_all(path.parent().ok_or("missing fixture parent")?)?;
    fs::write(path, content)?;
    Ok(())
}

#[test]
fn rejects_python_artifacts_even_in_hidden_directories() {
    for name in [
        "build.py",
        "api.pyi",
        "cache.pyc",
        "pyproject.toml",
        "uv.lock",
        "uv.toml",
        "pylock.toml",
        "requirements.in",
        ".pypirc",
        "requirements-dev.txt",
        "Pipfile",
        "poetry.lock",
        "tox.ini",
        ".python-version",
        ".venv/marker",
        "hidden/__pycache__/marker",
        ".hidden/helper.py",
        ".pre-commit-config.yaml",
    ] {
        let fixture = Fixture::new().unwrap();
        write(&fixture, name, "").unwrap();
        let error = rust_quality::inspect_artifacts(fixture.path()).unwrap_err();
        assert!(
            error.to_string().contains("Python artifact"),
            "{name}: {error}"
        );
    }
}

#[test]
fn rejects_interpreters_and_installers_without_relying_on_file_extensions() {
    for (name, text) in [
        ("build", "#!/usr/bin/env python3\n"),
        (
            ".github/workflows/check.yml",
            "steps:\n  - uses: actions/setup-python@v6\n",
        ),
        (".github/workflows/check.yml", "run: uv run ruff check .\n"),
        ("tools/build.ps1", "& python.exe build\n"),
        ("Dockerfile", "RUN pip3 install build\n"),
        (".github/workflows/check.yml", "run: ruff check .\n"),
        (".github/workflows/check.yml", "run: pytest\n"),
        (
            ".github/workflows/check.yml",
            "steps:\n  - run: pip install x\n",
        ),
        (
            ".github/workflows/check.yml",
            "steps:\n  - name: prepare\n    run: |\n      cargo build\n      pip install x\n",
        ),
        ("tools/check.sh", "set -e\npython3 script.py\n"),
        ("tools/check.sh", "cargo build && python3 script.py\n"),
        ("tools/check.sh", "FOO=1 env python3 -m build\n"),
        ("tools/check.sh", "sh -c 'pip install x'\n"),
        ("tools/run.cmd", "cmd /c python tool\n"),
        ("Makefile", "lint:\n\t@ruff check .\n"),
        ("Dockerfile", "RUN [\"python3\", \"tool.py\"]\n"),
        (".github/workflows/check.yml", "steps:\n  - shell: python\n"),
        (
            "build.rs",
            "fn main() { std::process::Command::new(\"python\").status(); }",
        ),
    ] {
        let fixture = Fixture::new().unwrap();
        write(&fixture, name, text).unwrap();
        let error = rust_quality::inspect_artifacts(fixture.path()).unwrap_err();
        assert!(
            error.to_string().contains("Python tooling"),
            "{name}: {error}"
        );
    }
}

#[test]
fn ordinary_rust_configuration_and_policy_prose_pass() {
    let fixture = Fixture::new().unwrap();
    for (name, text) in [
        (
            "Cargo.toml",
            "[package]\nname = 'native'\nversion = '0.1.0'\n",
        ),
        ("src/lib.rs", "//! A native library.\n"),
        (
            "AGENTS.md",
            "Keep generated projects Rust-native; Python tooling is prohibited.\n",
        ),
        (".github/workflows/check.yml", "run: cargo test --locked\n"),
    ] {
        write(&fixture, name, text).unwrap();
    }
    rust_quality::inspect_artifacts(fixture.path()).unwrap();
}

#[test]
fn prose_and_data_that_merely_mention_python_pass() {
    for (name, text) in [
        (
            "README.md",
            "Install the notebook with `python` before running the demo.\n",
        ),
        (
            "template.yaml",
            "Resources:\n  Fn:\n    Properties:\n      Runtime: python3.12\n",
        ),
        (
            "docs/notes.md",
            "Some teams use pip install for unrelated tooling.\n",
        ),
        (
            ".github/workflows/check.yml",
            "steps:\n  - name: black box test\n    run: cargo test --locked\n",
        ),
    ] {
        let fixture = Fixture::new().unwrap();
        write(&fixture, name, text).unwrap();
        rust_quality::inspect_artifacts(fixture.path())
            .unwrap_or_else(|error| panic!("{name}: {error}"));
    }
}

/// Naming or discussing the forbidden tool is not running it: only a command
/// position is an invocation. This project's own tests are named after Python.
#[test]
fn python_words_outside_a_command_position_pass() {
    for (name, text) in [
        (
            ".github/workflows/check.yml",
            "steps:\n  - run: cargo test --manifest-path xtask/Cargo.toml rejects_python_artifacts\n",
        ),
        (
            ".github/workflows/check.yml",
            "steps:\n  - run: |\n      # native only: no python here\n      cargo test --locked\n",
        ),
        (
            ".github/workflows/check.yml",
            "steps:\n  - run: cargo bench --locked -- black_box\n",
        ),
        (
            "justfile",
            "# Rust-native project; python is not used.\ncheck:\n\tcargo xtask check\n",
        ),
        (
            "tools/check.sh",
            "set -e\necho \"no python or pytest here\"\ncargo test # not pytest\n",
        ),
        ("Dockerfile", "RUN cargo test rejects_python_artifacts\n"),
        ("tools/check.ps1", "cargo test mypy_config_is_absent\n"),
    ] {
        let fixture = Fixture::new().unwrap();
        write(&fixture, name, text).unwrap();
        rust_quality::inspect_artifacts(fixture.path())
            .unwrap_or_else(|error| panic!("{name}: {text:?}: {error}"));
    }
}

/// The rule is about what this project owns and ships. A skill the agent host
/// installed may carry any language; the same file anywhere else is still rejected.
#[test]
fn skills_the_agent_host_installed_are_not_this_projects_artifacts() {
    let long_line = format!("// {}\n", "x".repeat(200));
    let installed = [
        (".claude/skills/helper/scripts/validate.py", "print('x')\n"),
        (
            ".claude/skills/pdf/scripts/run.sh",
            "#!/bin/sh\npython3 -m pdfplumber \"$1\"\n",
        ),
        (".agents/skills/other/pyproject.toml", "[project]\n"),
        (".agents/skills/other/examples/demo.rs", long_line.as_str()),
    ];
    let fixture = Fixture::new().unwrap();
    for (name, text) in installed {
        write(&fixture, name, text).unwrap();
    }
    rust_quality::inspect_artifacts(fixture.path()).unwrap();

    for name in [
        ".claude/hooks/validate.py",
        ".claude/skills.py",
        "vendor/.claude/skills/helper/validate.py",
        "skills/helper/validate.py",
    ] {
        let fixture = Fixture::new().unwrap();
        write(&fixture, name, "").unwrap();
        let error = rust_quality::inspect_artifacts(fixture.path()).unwrap_err();
        assert!(
            error.to_string().contains("Python artifact"),
            "{name}: {error}"
        );
    }
}

/// Git decides what belongs to the project. An ignored environment or package cache
/// is the developer's; the same files, not ignored, are the project's.
#[test]
fn files_git_ignores_are_not_the_projects() {
    for (ignore, name) in [
        (".venv/\n", ".venv/pyvenv.cfg"),
        ("node_modules/\n", "node_modules/node-gyp/gyp/gyp_main.py"),
    ] {
        let fixture = Fixture::new().unwrap();
        write(&fixture, name, "").unwrap();
        let error = rust_quality::inspect_artifacts(fixture.path()).unwrap_err();
        assert!(
            error.to_string().contains("Python artifact"),
            "{name}: {error}"
        );
        write(&fixture, ".gitignore", ignore).unwrap();
        rust_quality::inspect_artifacts(fixture.path())
            .unwrap_or_else(|error| panic!("{name}: {error}"));
    }
}

/// A worktree, a submodule or any repository nested in the checkout is another
/// project, whatever it contains.
#[test]
fn a_nested_repository_is_another_project() {
    let fixture = Fixture::new().unwrap();
    write(&fixture, "nested/tool.py", "print('another project')\n").unwrap();
    common::repository(&fixture.path().join("nested")).unwrap();
    rust_quality::inspect_artifacts(fixture.path()).unwrap();
}

/// `cargo new` ignores `/target`, which leaves the tooling's `xtask/target` listed,
/// and listed build output would be read as the project's own files.
#[test]
fn build_output_git_lists_is_rejected_until_ignored() {
    for name in ["target/debug/marker", "xtask/target/debug/marker"] {
        let fixture = Fixture::new().unwrap();
        write(&fixture, name, "").unwrap();
        let error = rust_quality::inspect_artifacts(fixture.path()).unwrap_err();
        assert!(
            error.to_string().contains("Build output is not ignored"),
            "{name}: {error}"
        );
        write(&fixture, ".gitignore", "target/\n").unwrap();
        rust_quality::inspect_artifacts(fixture.path())
            .unwrap_or_else(|error| panic!("{name}: {error}"));
    }
}

#[test]
fn rust_lines_the_formatter_silently_skips_are_rejected() {
    let long_literal = "x".repeat(150);
    let over = format!(
        "//! Probe.\nfn main() {{ let value = \"{long_literal}\"; println!(\"{{value}}\"); }}\n"
    );
    let fixture = Fixture::new().unwrap();
    write(&fixture, "src/main.rs", &over).unwrap();
    let error = rust_quality::inspect_artifacts(fixture.path()).unwrap_err();
    assert!(
        error.to_string().contains("rustfmt cannot format")
            && error.to_string().contains("main.rs:2"),
        "{error}"
    );
    // Splitting the literal with `\` continuation brings every line under the cap.
    let wrapped = format!(
        "//! Probe.\nfn main() {{\n    let value = \"{}\\\n        {}\";\n    println!(\"{{value}}\");\n}}\n",
        "x".repeat(75),
        "x".repeat(75)
    );
    write(&fixture, "src/main.rs", &wrapped).unwrap();
    rust_quality::inspect_artifacts(fixture.path()).unwrap();
    // Exactly at the cap passes; one more character fails.
    for (width, accepted) in [(160, true), (161, false)] {
        let line = format!("// {}", "y".repeat(width - 3));
        assert_eq!(line.chars().count(), width);
        write(
            &fixture,
            "src/main.rs",
            &format!("//! Probe.\n{line}\nfn main() {{}}\n"),
        )
        .unwrap();
        assert_eq!(
            rust_quality::inspect_artifacts(fixture.path()).is_ok(),
            accepted,
            "{width}"
        );
    }
}

#[test]
fn missing_directory_is_not_a_clean_project() {
    let fixture = Fixture::new().unwrap();
    assert!(rust_quality::inspect_artifacts(&fixture.path().join("missing")).is_err());
}

#[test]
fn quoted_shell_data_is_not_an_interpreter_command() {
    for (name, text) in [
        (
            "check.sh",
            "printf '%s\\n' 'Native build; python is not required'\n",
        ),
        ("check.sh", "echo \"native | python is not needed\"\n"),
        ("check.sh", "echo 'native & python is not needed'\n"),
        ("check.sh", "echo 'literal $(python -V)'\n"),
        ("check.sh", "echo 'literal `python -V`'\n"),
        (
            "check.sh",
            "sh -c 'echo \"native; python is not required\"'\n",
        ),
        (
            "check.ps1",
            "Write-Output 'Native build; python is not required'\n",
        ),
        ("check.cmd", "echo \"native & python is not needed\"\n"),
    ] {
        let fixture = Fixture::new().unwrap();
        write(&fixture, name, text).unwrap();
        rust_quality::inspect_artifacts(fixture.path())
            .unwrap_or_else(|error| panic!("{name}: {text:?}: {error}"));
    }
}

#[test]
fn quoted_hashes_do_not_hide_later_interpreter_commands() {
    for (name, text) in [
        ("check.sh", "echo \"value # quoted\"; python -V\n"),
        ("check.sh", "printf '%s' 'value # quoted'; python -V\n"),
        (
            "check.ps1",
            "Write-Output 'value # quoted'; & python.exe -V\n",
        ),
        ("check.cmd", "echo \"value # quoted\" & python -V\n"),
    ] {
        let fixture = Fixture::new().unwrap();
        write(&fixture, name, text).unwrap();
        let error = rust_quality::inspect_artifacts(fixture.path()).unwrap_err();
        assert!(
            error.to_string().contains("Python tooling"),
            "{name}: {text:?}: {error}"
        );
    }
}

#[test]
fn executable_shell_strings_and_substitutions_are_inspected() {
    for (name, text) in [
        ("check.sh", "sh -c 'echo ready; python -V'\n"),
        (
            "check.sh",
            "bash -lc 'echo \"value # quoted\"; python -V'\n",
        ),
        ("check.sh", "echo \"$(python -V)\"\n"),
        ("check.sh", "echo \"`python -V`\"\n"),
        ("check.sh", "echo \"$(printf '%s' ')'; python -V)\"\n"),
        (
            "check.ps1",
            "pwsh -NoProfile -Command 'Write-Output ready; python -V'\n",
        ),
        ("check.ps1", "Write-Output \"$(python -V)\"\n"),
        ("check.cmd", "cmd /c \"echo ready & python -V\"\n"),
    ] {
        let fixture = Fixture::new().unwrap();
        write(&fixture, name, text).unwrap();
        let error = rust_quality::inspect_artifacts(fixture.path()).unwrap_err();
        assert!(
            error.to_string().contains("Python tooling"),
            "{name}: {text:?}: {error}"
        );
    }
}
