//! Native configuration checks must reject Cargo's silent policy opt-outs.

mod common;

use common::Fixture;
use std::fs;

#[test]
fn empty_directory_cannot_claim_native_configuration() {
    let fixture = Fixture::new().unwrap();
    assert!(rust_quality::inspect_configuration(fixture.path()).is_err());
}

#[test]
fn python_dependency_alias_and_lock_entry_are_rejected() {
    for manifest in [
        "[dependencies]\nbindings = { package = 'pyo3', version = '0.26' }\n",
        "[[package]]\nname = 'pyo3-ffi'\nversion = '0.26.0'\n",
    ] {
        let fixture = Fixture::new().unwrap();
        fs::write(fixture.path().join("Cargo.toml"), manifest).unwrap();
        let error = rust_quality::inspect_artifacts(fixture.path()).unwrap_err();
        assert!(error.to_string().contains("Python dependency"));
    }
}

const ALIAS: &str = "[alias]\nxtask = 'run --locked --manifest-path xtask/Cargo.toml --'\n";

#[test]
fn included_configuration_is_rejected_before_cargo_discovery() {
    for include in [
        "include = ['shared.toml']\n",
        "include = [{ path = 'shared.toml' }]\n",
        "include = [{ path = 'optional.toml', optional = true }]\n",
        "include = []\n",
    ] {
        let fixture = Fixture::new().unwrap();
        fs::create_dir(fixture.path().join(".cargo")).unwrap();
        fs::write(
            fixture.path().join(".cargo/config.toml"),
            format!("{include}{ALIAS}"),
        )
        .unwrap();
        let error = rust_quality::inspect_configuration(fixture.path()).unwrap_err();
        assert!(
            error.to_string().contains("Cargo configuration includes"),
            "{error}"
        );
    }
}

#[test]
fn target_rustdoc_environment_is_rejected_in_a_child_process() {
    const CHILD: &str = "RUST_QUALITY_TARGET_RUSTDOC_TEST_CHILD";
    if std::env::var_os(CHILD).is_some() {
        let fixture = Fixture::new().unwrap();
        fs::create_dir(fixture.path().join(".cargo")).unwrap();
        fs::write(fixture.path().join(".cargo/config.toml"), ALIAS).unwrap();
        let error = rust_quality::inspect_configuration(fixture.path()).unwrap_err();
        assert!(error.to_string().contains("warning denial"), "{error}");
        return;
    }
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "target_rustdoc_environment_is_rejected_in_a_child_process",
            "--nocapture",
        ])
        .env(CHILD, "1")
        .env(
            "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTDOCFLAGS",
            "--cfg docsrs",
        )
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn rustflags_that_can_lower_a_lint_level_are_rejected() {
    for flags in [
        "[build]\nrustflags = ['-A', 'clippy::unwrap_used']\n",
        "[target.'cfg(all())']\nrustflags = '--cap-lints=allow'\n",
        "[env]\nRUSTFLAGS = '-W clippy::unwrap_used'\n",
        "[build]\nrustflags = ['--force-warn', 'clippy::unwrap_used']\n",
        "[build]\nrustdocflags = ['-Aclippy::unwrap_used']\n",
        "[profile.dev]\nrustflags = ['--allow', 'warnings']\n",
        "[env]\nCARGO_ENCODED_RUSTFLAGS = { value = \"-D\\u001fwarnings\\u001f--cap-lints\\u001fwarn\", force = true }\n",
    ] {
        let fixture = Fixture::new().unwrap();
        fs::create_dir(fixture.path().join(".cargo")).unwrap();
        fs::write(
            fixture.path().join(".cargo/config.toml"),
            format!("{ALIAS}{flags}"),
        )
        .unwrap();
        let error = rust_quality::inspect_configuration(fixture.path()).unwrap_err();
        assert!(
            error.to_string().contains("can lower a lint level"),
            "{flags}: {error}"
        );
    }
}

#[test]
fn rustflags_that_only_tune_or_raise_do_not_trip_the_flag_check() {
    for flags in [
        "[build]\nrustflags = ['-C', 'target-cpu=neoverse-n1']\n",
        "[build]\nrustflags = '-D warnings'\n",
        "[env]\nRUSTFLAGS = '-D warnings -C opt-level=2'\n",
        "[target.aarch64-unknown-linux-gnu]\nrustflags = ['-C', 'link-arg=-fuse-ld=lld']\n",
        "[profile.release]\nlto = true\ncodegen-units = 1\n",
    ] {
        let fixture = Fixture::new().unwrap();
        fs::create_dir(fixture.path().join(".cargo")).unwrap();
        fs::write(
            fixture.path().join(".cargo/config.toml"),
            format!("{ALIAS}{flags}"),
        )
        .unwrap();
        // The fixture is deliberately incomplete; it fails later, on the missing manifest.
        let error = rust_quality::inspect_configuration(fixture.path()).unwrap_err();
        assert!(
            !error.to_string().contains("can lower a lint level"),
            "{flags}: {error}"
        );
    }
}

/// Cargo lets configuration override a manifest's profile, so a manifest that says
/// `overflow-checks = true` proves nothing while one of these routes says otherwise.
#[test]
fn configuration_that_overrides_the_overflow_policy_is_rejected() {
    for route in [
        "[profile.release]\noverflow-checks = false\n",
        "[profile.release.package.'*']\noverflow-checks = false\n",
        "[profile.dist]\ninherits = 'release'\noverflow-checks = false\n",
        "[build]\nrustflags = ['-C', 'overflow-checks=off']\n",
        "[build]\nrustflags = '-Coverflow-checks=no'\n",
        "[target.'cfg(all())']\nrustflags = ['--codegen', 'overflow_checks=off']\n",
    ] {
        let fixture = Fixture::new().unwrap();
        fs::create_dir(fixture.path().join(".cargo")).unwrap();
        fs::write(
            fixture.path().join(".cargo/config.toml"),
            format!("{ALIAS}{route}"),
        )
        .unwrap();
        let error = rust_quality::inspect_configuration(fixture.path()).unwrap_err();
        assert!(
            error.to_string().contains("overflow policy belongs"),
            "{route}: {error}"
        );
    }
}

/// The check runner denies rustdoc warnings through `build.rustdocflags`, which Cargo
/// merges with a configured array. Cargo cannot merge a string with it, and ignores
/// it altogether once a target table sets rustdoc flags, so neither shape is accepted.
#[test]
fn rustdoc_flags_the_check_runner_cannot_merge_with_are_rejected() {
    for (flags, rejected) in [
        ("[build]\nrustdocflags = '--cfg docsrs'\n", true),
        (
            "[target.'cfg(all())']\nrustdocflags = ['--cfg', 'docsrs']\n",
            true,
        ),
        (
            "[target.x86_64-unknown-linux-gnu]\nrustdocflags = ['--cfg', 'docsrs']\n",
            true,
        ),
        (
            "[env]\nCARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTDOCFLAGS = '--cfg docsrs'\n",
            true,
        ),
        (
            "[env]\nCARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTDOCFLAGS = { value = '--cfg docsrs', force = true }\n",
            true,
        ),
        ("[build]\nrustdocflags = ['--cfg', 'docsrs']\n", false),
        ("[build]\nrustflags = '--cfg tokio_unstable'\n", false),
    ] {
        let fixture = Fixture::new().unwrap();
        fs::create_dir(fixture.path().join(".cargo")).unwrap();
        fs::write(
            fixture.path().join(".cargo/config.toml"),
            format!("{ALIAS}{flags}"),
        )
        .unwrap();
        // The fixture is deliberately incomplete; an accepted shape fails later, on the manifest.
        let error = rust_quality::inspect_configuration(fixture.path())
            .unwrap_err()
            .to_string();
        assert_eq!(
            error.contains("warning denial"),
            rejected,
            "{flags}: {error}"
        );
    }
}

/// Beside `config.toml`, Cargo reads the extensionless file and ignores the other.
#[test]
fn a_legacy_configuration_file_that_cargo_would_prefer_is_rejected() {
    let fixture = Fixture::new().unwrap();
    fs::create_dir(fixture.path().join(".cargo")).unwrap();
    fs::write(fixture.path().join(".cargo/config.toml"), ALIAS).unwrap();
    fs::write(
        fixture.path().join(".cargo/config"),
        "[profile.release]\noverflow-checks = false\n",
    )
    .unwrap();
    let error = rust_quality::inspect_configuration(fixture.path()).unwrap_err();
    assert!(
        error.to_string().contains("Legacy Cargo configuration"),
        "{error}"
    );
}

/// Cargo merges every ancestor's configuration into a command run at the root.
#[test]
fn an_ancestor_configuration_is_inspected_like_the_projects_own() {
    for (outer, rejected) in [
        (
            "include = ['shared.toml']\n",
            Some("Cargo configuration includes"),
        ),
        (
            "[build]\nrustflags = ['-A', 'clippy::unwrap_used']\n",
            Some("can lower a lint level"),
        ),
        (
            "[profile.release]\noverflow-checks = false\n",
            Some("overflow policy belongs"),
        ),
        ("[build]\nrustflags = ['-C', 'target-cpu=native']\n", None),
    ] {
        let fixture = Fixture::new().unwrap();
        let project = fixture.path().join("project");
        fs::create_dir_all(fixture.path().join(".cargo")).unwrap();
        fs::create_dir_all(project.join(".cargo")).unwrap();
        fs::write(fixture.path().join(".cargo/config.toml"), outer).unwrap();
        fs::write(project.join(".cargo/config.toml"), ALIAS).unwrap();
        // The fixture is deliberately incomplete; an accepted route fails later, on the manifest.
        let error = rust_quality::inspect_configuration(&project)
            .unwrap_err()
            .to_string();
        match rejected {
            Some(diagnostic) => assert!(error.contains(diagnostic), "{outer}: {error}"),
            None => assert!(error.contains("Cargo.toml"), "{outer}: {error}"),
        }
    }
}

#[test]
fn malformed_cargo_toml_is_not_ignored() {
    let fixture = Fixture::new().unwrap();
    fs::write(fixture.path().join("Cargo.toml"), "[package\n").unwrap();
    assert!(rust_quality::inspect_artifacts(fixture.path()).is_err());
}

#[test]
fn a_declared_custom_cfg_keeps_the_denied_level_and_a_lowered_one_does_not() {
    let canonical = include_str!("../lints.toml");
    for (entry, weakened) in [
        (
            "unexpected_cfgs = { level = \"deny\", check-cfg = ['cfg(tokio_unstable)'] }",
            false,
        ),
        (
            "unexpected_cfgs = { level = \"warn\", check-cfg = ['cfg(tokio_unstable)'] }",
            true,
        ),
    ] {
        let fixture = Fixture::new().unwrap();
        fs::create_dir(fixture.path().join(".cargo")).unwrap();
        fs::write(fixture.path().join(".cargo/config.toml"), ALIAS).unwrap();
        fs::create_dir(fixture.path().join("xtask")).unwrap();
        fs::write(fixture.path().join("xtask/lints.toml"), canonical).unwrap();
        fs::write(
            fixture.path().join("Cargo.toml"),
            canonical.replace("unexpected_cfgs = \"deny\"", entry),
        )
        .unwrap();
        // The fixture is deliberately incomplete; it fails later, on the missing tooling manifest.
        let error = rust_quality::inspect_configuration(fixture.path()).unwrap_err();
        assert_eq!(
            error
                .to_string()
                .contains("Weakened native lint rust::unexpected_cfgs"),
            weakened,
            "{entry}: {error}"
        );
    }
}
