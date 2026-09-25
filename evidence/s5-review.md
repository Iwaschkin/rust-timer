# Review: S5, the showpiece

| Slot | Value |
| --- | --- |
| Commit | `c613231` |
| Dirty scope | none; the checks ran on the tree committed as `c613231` |
| Toolchain | Rust 1.98.1, cargo-deny 0.20.2, git 2.55 |
| Host | Windows 11 x86_64 local; CI run <https://github.com/Iwaschkin/rust-timer/actions/runs/36099646191> on `c613231` |

| Command | Exit | Seconds | Log location |
| --- | --- | --- | --- |
| `cargo xtask guard` | 0 | 0 | local, not retained |
| `cargo xtask check` | 1, then 0 | 6, then about 20 | Clippy `type_complexity` on the dial's return type; named `Dots`, then clean |
| `cargo deny --workspace --all-features --locked check advisories` | 0 | 1 | local; one `ratatui-core 0.1.2` after adding tui-big-text |
| `cargo deny --manifest-path xtask/Cargo.toml --all-features --locked check advisories` | 0 | 2 | local, not retained |
| `cargo xtask gates` | 0 | 0 | 0 glob imports, 0 `pub` against 100 `pub(crate)`, 0 repeated names, 59 tests |
| Ownership checks O1 to O12 | all printed nothing | 0 | local |
| Mutation checks | whole-cell fill failed D09; no popup failed D12; both restored and green | 60 | local |
| Hosted CI: Linux full check, Windows check, advisories, `quality` gate | success | about 180 | the CI run above |

## Findings (at most 5, one line each)

- The bar counts eighths in integers from an exact elapsed/length ratio; no float-to-integer cast exists.
- Looking at renders found three defects no test named: a muddy bar ramp, a shadow showing a digit through, a crowded badge.
- tui-big-text adds `time` (via the calendar feature), darling 0.20 and itertools 0.15 beside 0.14; no advisory.
- The cycle rule is one function, `following`, shared by `advance` and the ribbon's `cycle`.
- No `allow`, `expect` or formatting skip.

## Open limits (at most 3)

- The design has been seen only as headless-browser renders; the owner has not run it in a terminal.
- Motion (S6) and the title and progress sequences (S7) are not built.
- Windows CI skips release-mode tests; only Linux ran them.

Semantic review and hosted results are separate lines above; a passing command does not
certify the review, and a local pass does not certify another platform.
