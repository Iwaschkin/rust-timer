# Review: S4, palette, glyphs and capabilities

| Slot | Value |
| --- | --- |
| Commit | `303cfb8` |
| Dirty scope | none; the checks ran on the tree committed as `303cfb8` |
| Toolchain | Rust 1.98.1, cargo-deny 0.20.2, git 2.55 |
| Host | Windows 11 x86_64 local; CI run <https://github.com/Iwaschkin/rust-timer/actions/runs/36098841318> on `303cfb8` |

| Command | Exit | Seconds | Log location |
| --- | --- | --- | --- |
| `cargo xtask guard` | 0 | 0 | local, not retained |
| `cargo xtask check` | 0 | 9 | local, not retained |
| `cargo deny --workspace --all-features --locked check advisories` | 0 | 2 | local, not retained |
| `cargo deny --manifest-path xtask/Cargo.toml --all-features --locked check advisories` | 0 | 1 | local, not retained |
| `cargo xtask gates` | 0 | 0 | 0 glob imports, 0 `pub` against 81 `pub(crate)`, 0 repeated names, longest line 141 (a test's HTML string) |
| Ownership checks O1 to O12 | O1 and O11 first hit doc comments in `options.rs` and `theme.rs`; reworded, then all printed nothing | 0 | local |
| Hosted CI: Linux full check, Windows check, advisories, `quality` gate | success | about 180 | the CI run above |

## Findings (at most 5, one line each)

- Tier none maps every colour to `Reset`, so bold survives; `no_color_keeps_modifiers` failed with `Rgb(230, 233, 242)` before the fix.
- Every text pair meets 4.5:1 and every border 3:1 in truecolor and in the 256 tier, computed from the palette constants.
- The 256 tier never uses indices 0 to 15, which the user's theme redefines; a ramp of arbitrary RGB stays in 16 to 255.
- A wide emoji's trailing cell is reset by ratatui and never drawn; `background_is_painted` checks only drawn cells.
- No `allow`, `expect` or formatting skip; the baseline assets are unchanged since `7ae17f6`.

## Open limits (at most 3)

- The design was reviewed as HTML renders in headless Edge, not in a real terminal; M06 to M08 remain.
- `--motion` (C14) is deferred to S6, because nothing reads it yet.
- Windows CI skips release-mode tests; only Linux ran them.

Semantic review and hosted results are separate lines above; a passing command does not
certify the review, and a local pass does not certify another platform.
