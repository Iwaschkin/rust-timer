# Review: S9, a parser crate

| Slot | Value |
| --- | --- |
| Commit | `67af458` |
| Dirty scope | none; the checks ran on the tree committed as `67af458` |
| Toolchain | Rust 1.98.1, cargo-deny 0.20.2, git 2.55 |
| Host | Windows 11 x86_64 local; CI run <https://github.com/Iwaschkin/rust-timer/actions/runs/36156522954> on `67af458` |

| Command | Exit | Seconds | Log location |
| --- | --- | --- | --- |
| `cargo xtask guard` | 0 | 0 | local, not retained |
| `cargo xtask check` | 0 | 19 | local, not retained |
| `cargo deny --workspace --all-features --locked check advisories` | 0 | 1 | local, not retained, with clap's eight new crates |
| `cargo deny --manifest-path xtask/Cargo.toml --all-features --locked check advisories` | 0 | 1 | local, not retained |
| `cargo xtask gates` | 0 | 0 | 0 glob imports, 0 `pub` against 112 `pub(crate)`, 0 repeated names, 70 tests |
| Ownership checks O1 to O13, O13 new for clap | all printed nothing | 0 | local |
| The binary run by hand for every contract case, stdout and stderr captured | exit 2 for each error, 0 for help, 1 for `--work=5` without a terminal | 10 | local, not retained |
| Hosted CI: Linux full check, Windows check, advisories, `quality` gate | success | about 180 | the CI run above |

## Findings (at most 5, one line each)

- clap is named only in `cli`; value enums map onto `Choice`, `GlyphTier` and `Motion`, so no other module changed type.
- The range and defaults stay in `settings`: clap calls `Minutes`'s and `Every`'s `FromStr`, and shows defaults through their `Display`.
- clap read `--every -1` as an unknown flag; `allow_negative_numbers` makes it a value that fails its range check, as before.
- The contract rows were rewritten from clap's captured output before the tests; C11 now promises one error, since clap checks names before values.
- The parser source halved, 8,690 to 4,545 bytes; the release binary grew from 887 KB to 1,431 KB. No `allow`, `expect` or formatting skip.

## Open limits (at most 3)

- clap's error text belongs to clap: the tests pin its key phrases, so a clap upgrade that rewords one fails a test by design.
- The non-Unicode cases ran on Windows only; the Unix branch runs in Linux CI.
- Windows CI skips release-mode tests; only Linux ran them.

Semantic review and hosted results are separate lines above; a passing command does not
certify the review, and a local pass does not certify another platform.
