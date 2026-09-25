# Review: S1, a bar that counts down

| Slot | Value |
| --- | --- |
| Commit | `a8c1722` |
| Dirty scope | none; the checks ran on the tree committed as `a8c1722` |
| Toolchain | Rust 1.98.1, cargo-deny 0.20.2, git 2.55 |
| Host | Windows 11 x86_64, local; no remote, so no CI job |

| Command | Exit | Seconds | Log location |
| --- | --- | --- | --- |
| `cargo xtask guard` | 0 | 0 | local, not retained |
| `cargo xtask check` | 0 | 33 | local, not retained |
| `cargo deny --workspace --all-features --locked check advisories` | 0 | 7 | local, not retained |
| `cargo deny --manifest-path xtask/Cargo.toml --all-features --locked check advisories` | 0 | 1 | local, not retained |
| Ownership checks O1 to O6 | printed nothing | 0 | local |

## Findings (at most 5, one line each)

- Progress is capped at 1: `progress_stays_within_unit_interval` read 8.2 three hours past the end before the cap.
- Without the stdout check the binary hung on the Windows console; the CLI test now kills it at 20 s (checked red, then restored).
- `Session` owns the terminal; `finish` reports run and restore failures in order, and `Drop` restores only if `finish` never ran.
- No `allow`, `expect`, formatting skip or platform `cfg` in `src/` or `tests/`.
- Plan deviations: the error types live in `terminal.rs`, not `app.rs`; C01 covers the work length only (ledger SI-03).

## Open limits (at most 3)

- M01, quitting cleanly on Windows Terminal, needs the owner at a terminal; not run.
- No hosted CI: the repository has no remote. Linux has not run.
- O7 errors until `src/cli.rs` exists in slice 3.

Semantic review and hosted results are separate lines above; a passing command does not
certify the review, and a local pass does not certify another platform.
