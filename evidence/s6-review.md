# Review: S6, motion

| Slot | Value |
| --- | --- |
| Commit | `5b65158` |
| Dirty scope | none; the checks ran on the tree committed as `5b65158` |
| Toolchain | Rust 1.98.1, cargo-deny 0.20.2, git 2.55 |
| Host | Windows 11 x86_64 local; CI run <https://github.com/Iwaschkin/rust-timer/actions/runs/36135628084> on `5b65158` |

| Command | Exit | Seconds | Log location |
| --- | --- | --- | --- |
| `cargo xtask guard` | 0 | 0 | local, not retained |
| `cargo xtask check` | 0 | 25 | local, not retained |
| `cargo deny --workspace --all-features --locked check advisories` | 0 | 1 | local; one `ratatui-core 0.1.2` after adding tachyonfx |
| `cargo deny --manifest-path xtask/Cargo.toml --all-features --locked check advisories` | 0 | 1 | local, not retained |
| `cargo xtask gates` | 0 | 0 | 0 glob imports, 0 `pub` against 109 `pub(crate)`, 0 repeated names, 66 tests |
| Ownership checks O1 to O12 | all printed nothing | 0 | local |
| Hosted CI: Linux full check, Windows check, advisories, `quality` gate | success | about 180 | the CI run above |

## Findings (at most 5, one line each)

- A filter on tachyonfx's `repeating` wrapper does not reach the shift inside; every colour pulsed white until it moved to the shift.
- The research recommended that wrapper-level filter; a test of an untouched text cell caught it, not the research.
- The pulse is measured, not restated: peaks 1.8 s apart, and a 200 ms mutation failed with peaks 500 ms apart.
- Sweeps blend in RGB; in HSL the tomato passed through purple on its way to the background.
- tachyonfx adds `compact_str` 0.10 beside 0.9 and carries `unsafe` of its own; no advisory, no suppression in our code.

## Open limits (at most 3)

- The animation has been seen only as frames rendered to HTML; the owner has not run it in a terminal.
- Frame pacing on a slow terminal is untested; effects are time-based, so a slow terminal drops frames, not time.
- Windows CI skips release-mode tests; only Linux ran them.

Semantic review and hosted results are separate lines above; a passing command does not
certify the review, and a local pass does not certify another platform.
