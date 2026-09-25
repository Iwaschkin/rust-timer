# Skill issues found while building pomodoro

Reader: the maintainer of rust-skills.

This ledger records every problem met while building rust-timer with
`rust-quality-baseline`, from rust-skills main `7ae17f6` (0.5.0-rc.5). The build
follows the plan pack in [plan/](plan/). Each entry says what happened, what the skill
led me to expect, the evidence, and what I did about it. Severity is one of: blocks
(the documented route fails), misleads (the text leads to a wrong action), friction
(extra work, right result), or note.

Host: Windows 11, Git Bash, Rust 1.98.1, cargo-deny 0.20.2, git 2.55. No remote, so
there is no hosted CI.

## Entries

### SI-01 `init` puts `rust-version` above `name` (note)

- **Where:** `init`, slice 1 setup.
- **What happened:** `init` inserted `rust-version = "1.98.1"` as the first key of
  `[package]`, above `name`. Cargo accepts it, but it reads oddly next to the order
  `cargo new` writes.
- **Expected:** the key placed after `edition`, or anywhere a reader looks for it.
- **Done:** left as written.

### SI-02 `init` leaves `/target` and `target/` both in `.gitignore` (note)

- **Where:** `init`, slice 1 setup.
- **What happened:** `cargo init` wrote `/target`; `init` appended a blank line and
  `target/`. The second covers the first.
- **Expected:** one entry. asset-application.md explains why `target/` is needed, so
  `init` could replace `/target` rather than add beside it.
- **Done:** left as written.

### SI-03 Strict lints decide what a slice can contain; the plan skill does not say so (friction)

- **Where:** rust-project-plan's slice format, met in slice 1.
- **What happened:** under `-D warnings`, a value, field, variant or enum arm that
  production code does not yet read is a `dead_code` error. The plan put all four
  defaults (C01) and a running-only timer in slice 1. The short-break, long-break and
  interval settings, and the non-work phases, cannot exist until slice 2 reads them.
  The same lint stops a planned red: a binary without the terminal check leaves
  `TerminalError::NotATerminal` unconstructed, so `cargo xtask check --fast` fails in
  Clippy before any test runs. The red is only visible through plain `cargo test`.
- **Expected:** plan-format.md section 7 to say that a slice introduces only what its
  own production code reads, and that a red is observed with `cargo test` when the
  lint gate would stop it first.
- **Done:** slice 1 has the work length only; C01's other three values move to
  slice 2. Both reds were observed with `cargo test --locked`.

### SI-04 Nothing bounds a spawned binary; an interactive one hangs the test (misleads)

- **Where:** the contracts example's binary tests and verification.md, met in slice 1.
- **What happened:** the planned red for C09 ran the binary without its terminal
  check. On Windows, crossterm reads keys from the console, not from stdin, so the
  binary entered the loop with stdout piped and waited for input. `cargo test` hung
  for 600 s until the outer `timeout` killed it (exit 143). The example's tests call
  `Command::output()` with no bound, and verification.md mentions only the caller's
  timeout. In CI, the job's 30 or 45 minute cap would have been the only bound.
- **Expected:** the example, or rust-design's CLI guidance, to bound any spawned child
  that can wait on a terminal, and to name the hang as its failure mode.
- **Done:** tests/cli.rs polls the child and kills it after 20 s. Negative check: with
  the terminal check disabled, the test failed in 22 s with "was still running after
  20s and was killed", no process was left, and the restored source passed.
