//! Review numbers must come from the tool, not from an agent counting its own.

mod common;

use common::Fixture;
use std::fs;

fn write(fixture: &Fixture, relative: &str, text: &str) {
    let path = fixture.path().join(relative);
    if let Some(parent) = path.parent() {
        let created = fs::create_dir_all(parent);
        assert!(created.is_ok(), "fixture directory: {created:?}");
    }
    let written = fs::write(&path, text);
    assert!(written.is_ok(), "fixture file: {written:?}");
}

#[test]
fn every_gate_counts_what_its_name_says() {
    let fixture = Fixture::new().unwrap();
    write(
        &fixture,
        "src/lib.rs",
        "use std::fmt::*;\npub struct One;\npub(crate) fn two() {}\npub enum Three { A }\n",
    );
    write(
        &fixture,
        "src/other.rs",
        "use std::io::*;\npub struct One;\npub(crate) const FOUR: u8 = 4;\n",
    );
    write(&fixture, "evidence/S1-review.md", "one\ntwo\n");
    let gates = rust_quality::gates(fixture.path()).unwrap();
    assert_eq!(gates.glob_imports, 2);
    assert_eq!(gates.public_items, 3);
    assert_eq!(gates.crate_items, 2);
    assert_eq!(gates.evidence_files, 1);
    assert_eq!(gates.evidence_bytes, 8);
    assert_eq!(gates.repeated_shapes, vec!["One".to_owned()]);
}

/// An asynchronous project writes these forms on every other line; a counter that
/// knows only the bare ones under-reports exactly the projects that use a runtime.
#[test]
fn qualified_functions_attribute_arguments_and_grouped_globs_are_counted() {
    let fixture = Fixture::new().unwrap();
    write(
        &fixture,
        "src/lib.rs",
        "use std::{fmt::*, io};\npub async fn fetch() {}\npub const fn limit() -> u8 { 1 }\n\
         pub(crate) async fn inner() {}\npub extern \"C\" fn exported() {}\npub union Bits { a: u8 }\n\
         pub use std::io::Read;\n",
    );
    write(
        &fixture,
        "tests/flow.rs",
        "#[tokio::test(flavor = \"current_thread\")]\nasync fn paused() {}\n\
         #[tokio::test(start_paused = true)]\nasync fn paused() {}\n#[test_case(1)]\nfn not_a_test() {}\n",
    );
    let gates = rust_quality::gates(fixture.path()).unwrap();
    assert_eq!(gates.glob_imports, 1);
    assert_eq!(gates.public_items, 4);
    assert_eq!(gates.crate_items, 1);
    assert_eq!(gates.test_functions, 2);
    assert_eq!(gates.repeated_test_names, vec!["paused".to_owned()]);
}

#[test]
fn the_longest_line_names_where_it_is() {
    let fixture = Fixture::new().unwrap();
    write(&fixture, "src/lib.rs", "pub struct Short;\n");
    let long = format!("//{}\n", "x".repeat(140));
    write(&fixture, "src/wide.rs", &long);
    let gates = rust_quality::gates(fixture.path()).unwrap();
    assert_eq!(gates.longest_line, 142);
    assert!(
        gates.longest_line_at.contains("wide.rs:1"),
        "{}",
        gates.longest_line_at
    );
}

#[test]
fn build_output_tooling_and_installed_skills_are_not_this_project() {
    let fixture = Fixture::new().unwrap();
    write(&fixture, "src/lib.rs", "pub struct Only;\n");
    write(&fixture, ".gitignore", "target/\n");
    for elsewhere in [
        "target/debug/build.rs",
        "xtask/src/main.rs",
        ".claude/skills/pack/examples/demo/src/lib.rs",
        ".agents/skills/pack/examples/demo/src/lib.rs",
    ] {
        write(&fixture, elsewhere, "use std::fmt::*;\npub struct Only;\n");
    }
    let gates = rust_quality::gates(fixture.path()).unwrap();
    assert_eq!(gates.glob_imports, 0);
    assert_eq!(gates.public_items, 1);
    assert!(gates.repeated_shapes.is_empty());
}

#[test]
fn a_test_name_used_twice_is_reported_because_a_contract_row_names_it() {
    let fixture = Fixture::new().unwrap();
    write(
        &fixture,
        "tests/values.rs",
        "#[test]\nfn bounds() {}\n#[test]\nfn only_here() {}\n",
    );
    write(
        &fixture,
        "tests/dates.rs",
        "#[tokio::test]\nasync fn bounds() {}\nfn bounds_helper() {}\n",
    );
    let gates = rust_quality::gates(fixture.path()).unwrap();
    assert_eq!(gates.repeated_test_names, vec!["bounds".to_owned()]);
    assert_eq!(gates.test_functions, 3);
}

#[test]
fn a_project_without_evidence_reports_none_rather_than_failing() {
    let fixture = Fixture::new().unwrap();
    write(&fixture, "src/lib.rs", "pub struct Only;\n");
    let gates = rust_quality::gates(fixture.path()).unwrap();
    assert_eq!(gates.evidence_files, 0);
    assert_eq!(gates.evidence_bytes, 0);
    assert!(gates.to_string().contains("evidence"));
}
