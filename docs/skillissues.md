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
  slice 2. Both reds were observed with `cargo test --locked`. In slice 2 the same
  rule held back T07's N = 1 case: building settings with N = 1 needs the `--every`
  constructor, which only slice 3's production code reads, and a test-only
  constructor would be inactive `cfg` code. That case moves to slice 3.

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

### SI-05 The review record has no stated home (friction)

- **Where:** quality-review-template.md and the kickoff template, at the end of slice 1.
- **What happened:** the kickoff says a slice's evidence is "a review of at most 40
  lines, the commit hash and the CI run URL", and the template says logs and captured
  output stay out of the repository. Neither says whether the review itself is
  committed, or where. It also has to name the commit it reviews, so it cannot sit in
  that commit.
- **Expected:** one sentence giving the path and the order: slice commit, then the
  review in a following commit.
- **Done:** a review file per slice, committed after the slice commit it names;
  first under `docs/reviews/`, then under `evidence/` (SI-08).

### SI-06 A plan check that names files fails until the files exist (note)

- **Where:** rust-project-plan's ownership checks, met in slice 1.
- **What happened:** O7, `grep -ln 'ratatui' src/settings.rs src/cli.rs src/timer.rs`,
  printed `grep: src/cli.rs: No such file or directory` until slice 3 creates the file.
  plan-format.md says a check prints nothing while its rule holds, but not from which
  slice.
- **Expected:** checks written over directories, or with `grep -s`, so they hold from
  the first slice; or a stated first slice for each.
- **Done:** recorded the error in the slice 1 review; the check is correct from slice 3.

### SI-07 The kickoff assumes a remote and hosted CI (friction)

- **Where:** kickoff-template.md, after slice 1.
- **What happened:** the kickoff says to push the branch, open a pull request and use
  the CI run URL as evidence from slice 2 on. rust-timer has no remote, and neither
  skill asks whether one exists before the kickoff is filled in.
- **Expected:** the plan's verification matrix to record whether hosted CI exists,
  and the kickoff to say what replaces the CI URL when it does not.
- **Done:** slices 1 and 2 ran on local checks, labelled local. The owner then added
  a GitHub remote; its first push made `feat/pomodoro` the default branch, and the
  workflow only runs on pushes to `main` and on pull requests, so nothing ran until
  `main` was pushed and PR #1 opened. Its first run passed. The kickoff could say
  that the base branch must exist on the remote and be the default.

### SI-08 `gates` counts evidence in a directory no document names (misleads)

- **Where:** `cargo xtask gates`, at the slice 2 stop.
- **What happened:** gates reported "evidence: 0 bytes in 0 files" with the slice 1
  review committed under `docs/reviews/`. xtask/src/gates.rs counts only files whose
  path starts with a top-level `evidence/` directory. verification.md says gates
  reports "evidence size and file count" but not where evidence lives, and neither
  the review template nor the kickoff names the directory. Together with SI-05, the
  only statement of where a review goes is a string in the tool's source.
- **Expected:** verification.md, the review template or `init` (by creating
  `evidence/`) to name the directory that gates measures.
- **Done:** moved the reviews to `evidence/` and updated AGENTS.md. Gates then
  reported "evidence: 1754 bytes in 1 files". The move went into the slice 2 code
  commit `a6a4092` by accident, because `git mv` had staged it.
