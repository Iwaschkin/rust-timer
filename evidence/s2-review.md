# Review: S2, the pomodoro cycle

| Slot | Value |
| --- | --- |
| Commit | `a6a4092` |
| Dirty scope | none for code; afterwards AGENTS.md names `evidence/` for reviews, and the guard passed again |
| Toolchain | Rust 1.98.1, cargo-deny 0.20.2, git 2.55 |
| Host | Windows 11 x86_64 local; CI run <https://github.com/Iwaschkin/rust-timer/actions/runs/36085739965> on `e44cd37`, which contains `a6a4092` |

| Command | Exit | Seconds | Log location |
| --- | --- | --- | --- |
| `cargo xtask guard` | 0 | 1 | local, not retained |
| `cargo xtask check` | 0 | 7 | local, not retained |
| `cargo deny --workspace --all-features --locked check advisories` | 0 | 2 | local, not retained |
| `cargo deny --manifest-path xtask/Cargo.toml --all-features --locked check advisories` | 0 | 1 | local, not retained |
| `cargo xtask gates` | 0 | 0 | pasted in the slice 2 handoff |
| Hosted CI: Linux full check, Windows check, advisories, `quality` gate | success | 216 | the CI run above |
| Ownership checks O1 to O7 | O1 to O6 printed nothing; O7 printed a grep error for `src/cli.rs` | 0 | local |

## Findings (at most 5, one line each)

- A reading hours late ends one phase, and a key after an unobserved end reports the end, then acts on the next phase.
- Time left is banked time plus time since the last start, saturating and capped; production code never adds to an `Instant`.
- Gates: no glob imports, no `pub` against 35 `pub(crate)`, no repeated shape or test name, longest line 99.
- No `allow`, `expect`, formatting skip or platform `cfg` in `src/` or `tests/`.
- The move of the slice 1 review to `evidence/` went into `a6a4092` with the code (ledger SI-08).

## Open limits (at most 3)

- M01, M03 and M04 need the owner at Windows Terminal and a Linux terminal; not run.
- Windows CI skips release-mode tests and builds the release binary instead; only Linux ran them.
- T07 with N = 1 waits for slice 3's `--every` constructor (ledger SI-03).

Semantic review and hosted results are separate lines above; a passing command does not
certify the review, and a local pass does not certify another platform.
