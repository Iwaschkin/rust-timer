//! Reduced check lanes are accepted, exclusive, and never mistaken for the full check.

mod common;

use common::Fixture;

#[test]
fn reduced_lanes_are_recognised_arguments() {
    // An empty directory fails later, on the missing configuration, never on the flag.
    for lane in ["--fast", "--no-release-tests"] {
        let fixture = Fixture::new().unwrap();
        let error = rust_quality::check(fixture.path(), &[lane.into()]).unwrap_err();
        assert!(
            !error.to_string().contains("Unsupported check argument"),
            "{lane}: {error}"
        );
    }
}

#[test]
fn only_one_reduced_lane_may_be_selected() {
    let fixture = Fixture::new().unwrap();
    let error = rust_quality::check(
        fixture.path(),
        &["--fast".into(), "--no-release-tests".into()],
    )
    .unwrap_err();
    assert!(error.to_string().contains("one reduced lane"), "{error}");
}

#[test]
fn unknown_arguments_are_still_rejected() {
    let fixture = Fixture::new().unwrap();
    let error = rust_quality::check(fixture.path(), &["--quick".into()]).unwrap_err();
    assert!(
        error.to_string().contains("Unsupported check argument"),
        "{error}"
    );
}
