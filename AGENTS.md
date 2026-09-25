# Rust project instructions

<!-- rust-quality-baseline: 0.5.0-rc.5 -->

Keep automation and dependencies Rust-native; Python tooling, scripts and config
are prohibited.

Run from the project root:

~~~sh
cargo xtask guard
cargo xtask check
cargo deny --workspace --all-features --locked check advisories
cargo deny --manifest-path xtask/Cargo.toml --all-features --locked check advisories
~~~

The project-owned xtask is an independent workspace; its compiler floor is not
the product MSRV. Resolve locks only in deliberate dependency changes; checks
preserve them. Iterate with `cargo xtask check --fast`; run the full check before
calling a slice done.

`pomodoro` is an application, not published, with no features and no MSRV below
the pinned 1.98.1, so there are no feature, MSRV or publication lanes. CI runs the
check on Linux and Windows. The plan pack in `docs/plan/` fixes behaviour; the
behaviour contract names the test for every rule, and the plan's ownership checks
O1 to O7 each print nothing while their rule holds. Terminal restore, the bell and
resizing (contract rows M01, M03, M04) need a person at a real terminal on Windows
and on Linux; record each result in the slice review under `evidence/`. Tests
that run the binary bound it and kill it at the limit, because an interactive
binary hangs instead of failing.

Keep invariant-bearing representation private and every construction/mutation
route valid. Identify resource owners, preserve error causes and perform
fallible completion explicitly. For concurrent work, bound admission, supervise
tasks, observe failures and specify cancellation/shutdown outcomes. Prefer
cohesive modules and concrete types; introduce traits/crates for useful boundaries.

Public documentation explains guarantees and errors. Tests protect observable
behavior and failure paths; assertions belong in tests, never as operational
shortcuts.

## Root-cause discipline

Fix code before changing policy.

| Situation | Do | Evidence |
| --- | --- | --- |
| A failure | Reproduce, trace the input to the flaw, fix that cause, verify, delete obsolete code | The reproducing test passes |
| A diagnostic you would rather silence, swallow or retry past | Fix its cause first | The cause is named in the commit |
| A narrow native expectation | A concrete reason at the site, reviewed | `#[expect(..., reason = "...")]` |
| A temporary workaround | Cause, why the proper fix is unavailable, that fix and its removal condition at the site, marked `WORKAROUND:`; a blocked fix is surfaced | The four parts at the site |
| A compatibility path | Only for an actual consumer | The consumer is named |
| An allow/warn attribute, broad expectation, formatting skip, inactive cfg branch, or a guard, lint, build or CI change | Review as a policy change; tools miss these and the guard sees only literal routes | Listed in the review record |

Report commands and outcomes, distinguishing local checks, hosted CI and semantic
review; a missing tool, timeout or skipped check counts as unexecuted.
