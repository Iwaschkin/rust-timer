# Review: S8, the loop seam

| Slot | Value |
| --- | --- |
| Commit | `1b6aa5e` |
| Dirty scope | none; the checks ran on the tree committed as `1b6aa5e` |
| Toolchain | Rust 1.98.1, cargo-deny 0.20.2, git 2.55 |
| Host | Windows 11 x86_64 local; CI run <https://github.com/Iwaschkin/rust-timer/actions/runs/36155607056> on `1b6aa5e` |

| Command | Exit | Seconds | Log location |
| --- | --- | --- | --- |
| `cargo xtask guard` | 0 | 0 | local, not retained |
| `cargo xtask check` | 0 | 19 | local, not retained |
| `cargo deny --workspace --all-features --locked check advisories` | 0 | 1 | local, not retained |
| `cargo deny --manifest-path xtask/Cargo.toml --all-features --locked check advisories` | 0 | 1 | local, not retained |
| `cargo xtask gates` | 0 | 0 | 0 glob imports, 0 `pub` against 113 `pub(crate)`, 0 repeated names, 71 tests |
| Ownership checks O1 to O12, O6 now naming the live host | all printed nothing | 0 | local |
| Mutations of the loop, each restored | no bell, the real clock, a bell on every key: each failed `loop_rings_one_bell_at_phase_end` | 30 | local, not retained |
| Hosted CI: Linux full check, Windows check, advisories, `quality` gate | success | about 180 | the CI run above |

## Findings (at most 5, one line each)

- The loop reads the time only from its `Host`; `Session` is the live host and the only reader of `Instant::now`.
- `Session` implements `Host` itself, so the seam adds no pass-through layer; `enter` and `finish` stay inherent.
- The scripted host moves its clock only while the loop waits, so a 20 s run, 90 s pause and 40 s run end with one bell at 150 s.
- The 25-minute version of the sequence took 84 s in a debug build at four frames a second; a one-minute phase tests the same property in 3 s.
- No `allow`, `expect` or formatting skip; the test backend's error type is `Infallible`, matched away rather than mapped.

## Open limits (at most 3)

- M03 now covers only what the terminal does with the bell; its Linux run is still open.
- The scripted host draws into a 20 by 6 buffer, so the sequence test exercises the compact layout only.
- Windows CI skips release-mode tests; only Linux ran them.

Semantic review and hosted results are separate lines above; a passing command does not
certify the review, and a local pass does not certify another platform.
