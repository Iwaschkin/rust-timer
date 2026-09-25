# Review: S3, the command line

| Slot | Value |
| --- | --- |
| Commit | `798069d` |
| Dirty scope | none; the checks ran on the tree committed as `798069d` |
| Toolchain | Rust 1.98.1, cargo-deny 0.20.2, git 2.55 |
| Host | Windows 11 x86_64 local; CI run <https://github.com/Iwaschkin/rust-timer/actions/runs/36092083660> on `f05eaf3`, which contains `798069d` |

| Command | Exit | Seconds | Log location |
| --- | --- | --- | --- |
| `cargo xtask guard` | 0 | 0 | local, not retained |
| `cargo xtask check` | 0 | 16 | local, not retained |
| `cargo deny --workspace --all-features --locked check advisories` | 0 | 7 | local, not retained |
| `cargo deny --manifest-path xtask/Cargo.toml --all-features --locked check advisories` | 0 | 2 | local, not retained |
| `cargo xtask gates` | 0 | 0 | pasted in the slice 3 handoff |
| Ownership checks O1 to O7 | all printed nothing | 0 | local |
| Hosted CI, first attempt, run 36091436371 | not started: account billing, while the repository was private | 2 to 5 | GitHub |
| Hosted CI: Linux full check, Windows check, advisories, `quality` gate | success | 108 | the CI run above |

## Findings (at most 5, one line each)

- The first wrong argument is reported, on one line with exit 2, before the terminal is touched; `-h` anywhere wins.
- The range and defaults are written once, in `settings`; the flag names once, in `cli`; the usage text reads both.
- Every one of the 31 tests the contract names exists; gates found no repeated test name among 33.
- The only platform `cfg` is C07's argument builder in `tests/cli.rs`: the Windows branch ran locally and in CI, the Unix branch in Linux CI.
- Deployed baseline assets match rust-skills `7ae17f6` file for file: tooling, configuration, CI and lint tables.

## Open limits (at most 3)

- Windows CI skips release-mode tests and builds the release binary instead; only Linux ran them.
- M03, the bell, is not run; `cargo run -- --work 1` reaches it in a minute.
- M01 and M04 were validated by the owner on 2026-09-25; the terminal used is not recorded.

Semantic review and hosted results are separate lines above; a passing command does not
certify the review, and a local pass does not certify another platform.
