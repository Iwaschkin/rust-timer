# Review: S7, terminal integration

| Slot | Value |
| --- | --- |
| Commit | `a16c84f` |
| Dirty scope | none; the checks ran on the tree committed as `a16c84f` |
| Toolchain | Rust 1.98.1, cargo-deny 0.20.2, git 2.55 |
| Host | Windows 11 x86_64 local; CI run <https://github.com/Iwaschkin/rust-timer/actions/runs/36136415024> on `a16c84f` |

| Command | Exit | Seconds | Log location |
| --- | --- | --- | --- |
| `cargo xtask guard` | 0 | 0 | local, not retained |
| `cargo xtask check` | 0 | 8 | local, not retained |
| `cargo deny --workspace --all-features --locked check advisories` | 0 | 1 | local, not retained |
| `cargo deny --manifest-path xtask/Cargo.toml --all-features --locked check advisories` | 0 | 2 | local, not retained |
| `cargo xtask gates` | 0 | 0 | 0 glob imports, 0 `pub` against 117 `pub(crate)`, 0 repeated names, 70 tests |
| Ownership checks O1 to O12 | O4 first hit tests quoting the contract's sequences; O4 now excludes test files like the others, then all printed nothing | 0 | local |
| Pseudo-console capture, `WT_SESSION` set | title stack pushed first; titles and progress follow the timer; farewell before the alternate screen is left | 4 | local, not retained |
| Hosted CI: Linux full check, Windows check, advisories, `quality` gate | success | about 180 | the CI run above |

## Findings (at most 5, one line each)

- ConPTY ignores an empty title and later sent the last countdown; the farewell now sets `pomodoro`, which the capture shows last.
- Progress is written only where OSC 9;4 is known; accepting every terminal failed `progress_only_on_known_terminals`.
- A failed farewell is reported as a failed restore; raw mode is left either way.
- Title and progress are written only on change, and progress again every 10 s for terminals that drop it.
- No `allow`, `expect` or formatting skip.

## Open limits (at most 3)

- Windows: M06 in Windows Terminal and M07 in VS Code's terminal pass (owner, 2026-09-25).
- Linux terminal: TBD. M08, and M01, M03 and M04 from S3, are not run yet; the owner plans them later on 2026-09-25.
- Windows CI skips release-mode tests; only Linux ran them.

Semantic review and hosted results are separate lines above; a passing command does not
certify the review, and a local pass does not certify another platform.
