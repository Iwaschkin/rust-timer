# Skill issues found while building pomodoro

Reader: the maintainer of rust-skills.

This ledger records every problem met while building rust-timer with
`rust-quality-baseline`, from rust-skills main `7ae17f6` (0.5.0-rc.5). The build
follows the plan pack in [plan/](plan/). Each entry says what happened, what the skill
led me to expect, the evidence, and what I did about it. Severity is one of: blocks
(the documented route fails), misleads (the text leads to a wrong action), friction
(extra work, right result), or note.

Host: Windows 11, Git Bash, Rust 1.98.1, cargo-deny 0.20.2, git 2.55. A GitHub remote
arrived after slice 2; hosted CI evidence starts there (SI-07, SI-11).

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
  constructor would be inactive `cfg` code. That case moves to slice 3. It recurred
  three times in slice 4: the `--motion` flag, the colour `mix` function and the
  `VTE_VERSION` field each waited for the slice whose production code reads them.

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

### SI-09 "Apply the baseline at the start of the slice" has no defined meaning mid-project (note)

- **Where:** kickoff-template.md step 1, at the start of slice 3.
- **What happened:** the project was bootstrapped with `init` in slice 1, so later
  slices had nothing to bootstrap. maintenance.md covers a refresh and says equal
  version markers are not evidence that nothing changed; the diff against the
  installed assets is. The kickoff does not say that "apply" means that diff. I first
  compared markers and ran the guard, and only did the diff after reading
  maintenance.md.
- **Expected:** the kickoff step to say "diff the deployed assets against the
  installed skill, as maintenance.md describes; refresh if they differ".
- **Done:** diffed 19 tooling files, four configuration files, the CI workflow and
  the lint tables against rust-skills `7ae17f6`; all matched.

### SI-10 Nothing checks that the tests the contract names exist (friction)

- **Where:** plan-format.md's evidence column and `cargo xtask gates`, at the end of
  slice 3.
- **What happened:** the contract names a test for each row, and `gates` reports a
  test name used more than once. Nothing reports a named test that does not exist,
  so renaming a test orphans its row silently. I checked by hand: every backticked
  name in the contract's rows against every `fn` in `src/` and `tests/`, and all 31
  exist.
- **Expected:** gates, or a check the plan generates, to list contract names with no
  matching test.
- **Done:** the hand check, recorded in the slice 3 review.

### SI-11 Hosted CI stopped on account billing; the template does not mention cost (note)

- **Where:** the CI template, on PR #2.
- **What happened:** every job of run 36091436371 failed in 2 to 5 s with "The job
  was not started because recent account payments have failed or your spending
  limit needs to be increased." The repository is private. On a private repository
  every job spends the plan's included Actions minutes, and beyond those GitHub's
  billing page charges $0.010 a minute on Windows against $0.006 on Linux; the
  template also runs the whole workflow weekly. Whether this workflow used up the
  allowance is not established here; the template does not mention the cost at all.
- **Expected:** a line in asset-application.md or the template about minutes on
  private repositories and the weekly run, so an owner can budget for it.
- **Done:** the owner made the repository public, where standard runners are free.
  The run queued for `f05eaf3` then started and passed on every job (run
  36092083660, 108 s). Slice 3's review records both runs.

### SI-12 A manual check that tests the terminal, not the program (misleads)

- **Where:** rust-project-plan's acceptance rows, met at M03 after slice 3.
- **What happened:** M03 said "a phase ends: the terminal bell rings once", checked
  by hand. The owner heard nothing in VS Code's terminal or in Windows Terminal. The
  row mixed two claims: that pomodoro sends a bell, which is the program's
  behaviour, and that the terminal makes a sound, which is the terminal's
  configuration. VS Code's `accessibility.signals.terminalBell` defaults to sound
  `"auto"`, which plays only when a screen reader is attached. So a correct program
  fails the row by default, and the manual check alone could not tell the two
  apart. Neither skill offers a way to capture what a terminal program actually
  writes. The same holds for M01 and M04.
- **Expected:** plan-format.md to split such a row into what the program emits,
  tested through a pseudo-terminal, and what the terminal does with it, a manual
  check that records the terminal's settings. The baseline could name a
  pseudo-terminal capture as the route to test terminal output.
- **Done:** a throwaway harness ran pomodoro in a ConPTY, the pseudo-console both
  terminals use, for 65 s with `--work 1`. It captured "Work · Running" becoming
  "Short break · Ready" and exactly one standalone bell at that point; a PowerShell
  control that writes a bell gave one standalone bell the same way. The README now
  names the settings that make the bell audible. The owner then heard it in Windows
  Terminal, quiet and about a second late, and never in VS Code. A timed capture put
  the bell 0.16 s after the phase end and 15 ms before the Ready screen, so the delay
  is in the terminal's sound playback, not in pomodoro. Linux is not yet checked.

### SI-13 The plan format has no route for extending a pack after its slices are done (friction)

- **Where:** rust-project-plan's plan-format.md, when the owner asked for the visual
  overhaul after slice 3.
- **What happened:** plan-format.md says to keep an existing good plan and amend it
  rather than reformat it, and caps the decision record at 900 words and the plan at
  3,000. The decision record was at 803 words, so the overhaul could not be added as
  an amendment. It had to be rewritten, folding the confirmed "choices made for you"
  into the description. The plan made room by collapsing the finished slices 1 to 3
  into one paragraph that points at their reviews. Nothing in the skill says what
  happens to finished slices, answered open questions or confirmed choices, or where
  a mid-project research file belongs.
- **Expected:** a short rule: finished slices shrink to a pointer at their evidence;
  confirmed choices move out of "choices made for you"; research lives beside the
  pack and the plan cites it.
- **Done:** the decision record was rewritten at 864 words and the plan at 2,254,
  with 23 new contract rows (C12–C15, D06–D13, P01–P08, G01–G03, E01–E05, I01–I04,
  M06–M08, V09–V11). The research is `docs/research/visual-overhaul.md`.

### SI-14 No route to review what a terminal UI looks like (friction)

- **Where:** rust-quality-baseline's verification.md, and rust-project-plan's
  verification matrix, in slices 4 and 5.
- **What happened:** the baseline's evidence is commands, tests and CI; a visual
  design has none of those for "does it look right". Tests could check that the dial
  lights the right quadrant or that no digit keeps the full accent while paused, but
  not that the bar looked muddy at 29 %, that the drop shadow let a digit show
  through in a pale grey, or that the paused badge sat on top of the digits. Those
  three defects were found only by looking.
- **Expected:** a named route for rendered-output review: render scenes from
  `TestBackend` buffers to a viewable form, outside the evidence count, and look at
  them before a slice closes.
- **Done:** an ignored test, `preview_screens`, writes each scene as HTML (every
  cell with its colours, in the owner's terminal font) and as plain text under
  `target/preview/`. Headless Edge screenshots of the HTML were the design review.
  The README first showed a text render, which misaligned on GitHub, whose fonts
  do not give braille, block elements and emoji whole cells; it now shows a
  screenshot of the HTML render. The plan records the previews as review aids, not
  evidence for any row.

### SI-15 A stacked branch was rebased; the reviews name commits main does not contain (misleads)

- **Where:** the kickoff's branch instructions and the review template, when PRs #2
  and #3 merged.
- **What happened:** the overhaul branch started from the slice 3 branch while PR #2
  was open, so PR #3 was stacked on it. When PR #2 merged, GitHub moved PR #3's base
  to main, and three seconds later the branch was force-pushed with its eleven
  commits rebased onto the merge commit, each with a new hash. PR #3 then merged
  with a merge commit. The commits the S4 to S7 reviews name, `303cfb8`, `c613231`,
  `5b65158` and `a16c84f`, are not on main.
- **Expected:** the kickoff or the project's AGENTS.md to say that work goes on a
  feature branch from the current main and merges through a pull request with a
  merge commit, and that later work starts on a new branch from the updated main,
  never on a merged branch or stacked on an unmerged one.
- **Done:** the owner set that workflow, and rust-skills 0.5.0-rc.6 adds it to the
  AGENTS.md template and the kickoff. Each rebased commit has the same tree as the
  one it replaced, so the reviews' results hold for main's `0a6260b`, `bf09b4f`,
  `432bba8` and `6c7f740`, in that order.

### SI-16 Both skills show only hand-written parsers, and the contract ruled out a library (misleads)

- **Where:** rust-project-plan's example plan and workflow, the baseline's
  contracts example, and the contract rows C02 to C11; found by an external
  review of the code after slice 7.
- **What happened:** the reviewer found 8.7 KB of bespoke argument parsing where
  clap, lexopt or pico-args would do. The plan wrote "the standard library reads
  the command line" with no reason given: the plan skill's only example is a
  "standard library only" CLI, and the baseline points at its dependency-free
  contracts example for CLI behaviour. The contract, written before any crate
  was chosen, then fixed edge cases a parser crate handles its own way: it
  rejected `--work=5`, accepted `--help` beside an invalid argument, and reported
  the first wrong argument in order. Only hand-written code could meet it.
- **Expected:** a default to reuse a maintained crate for a solved problem, with
  the choice made before the contract rows it affects, which then state what the
  crate guarantees.
- **Done:** rust-skills 0.5.0-rc.6 adds that default to rust-design and the plan
  format, and the two examples now say their one positional argument is why they
  parse by hand. Slice S9 (PR #9) replaces the parser with clap and rewrites C02
  to C08, C11 and C15 to clap's behaviour.

### SI-17 No rule for when a loop needs its own seam (friction)

- **Where:** the baseline's rust-design reference; found by the same review.
- **What happened:** the timer took the time as a parameter, as the plan intended,
  but the loop read `Instant::now` and the keys itself. Sequences through the
  loop, such as run, pause, resume and a phase end with one bell, could be tested
  only inside `Timer`, and the program's half of M03 needed a pseudo-console
  capture (SI-12).
- **Expected:** a rule saying when the loop takes a clock and an event source:
  when a promised outcome comes from the loop's own sequencing.
- **Done:** rust-skills 0.5.0-rc.6 adds that rule as a rust-design row. Slice S8
  (PR #8) gives the loop a `Host`, and `loop_rings_one_bell_at_phase_end` drives a
  one-minute phase through a 90 s pause to exactly one bell (row M09).
