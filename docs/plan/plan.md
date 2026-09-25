# Pomodoro timer: plan

Reader: the agent implementing `pomodoro`.

Behaviour is fixed by the behaviour contract; row ids below refer to it. Commands,
lints and evidence rules come from `rust-quality-baseline`; this plan copies none
of them.

## 1. Structure and boundaries

The project root is the repository root. It holds one Cargo package, `pomodoro`: an
application with a binary target and no library target, `publish = false`. The
baseline's `xtask/` sits beside it as its own workspace with its own lock.

A library target is not justified: no consumer outside the binary uses these types.
Unit tests therefore live beside their module, in `src/<module>/tests.rs`, declared
with `#[cfg(test)] mod tests;`. Keeping tests in their own files lets the ownership
checks below scan production code only. The command-line rows run against the built
binary from `tests/cli.rs`.

| Module | Owns | Depends on |
| --- | --- | --- |
| `settings` | phase-length and interval value types, the range 1 to 99, the defaults | std |
| `cli` | the argument grammar, usage text, the usage error type | `settings` |
| `timer` | phase, round and timer state, transitions, remaining time, progress | `settings`, `std::time` |
| `view` | layout, the bar, phase and state labels, the remaining-time label | `timer`, ratatui widgets and layout |
| `terminal` | entering and restoring the terminal, the terminal check, key mapping, key help text, the bell, the command enum | ratatui's crossterm backend and re-export |
| `app` | the run loop: read the clock, tick, draw, apply commands; the run error type | `settings`, `timer`, `view`, `terminal` |
| `main` | reading `args_os`, exit codes, rendering error chains | `cli`, `app` |

Direction: `settings`, `cli` and `timer` never name ratatui or crossterm. `view` uses
only backend-independent ratatui items, so it does not depend on `terminal`; `app`
passes it the key help text. `terminal` is the only module that names crossterm.

### Values and ownership

| Value or resource | Owner | Borrowed by | Completion |
| --- | --- | --- | --- |
| Settings | copied into the timer at start | `view`, reading through the timer | none |
| Timer | a local in `app`'s run function | `view`, shared for one draw; command handling, exclusive for one call | dropped at quit; nothing persists |
| Terminal session (ratatui's `DefaultTerminal`) | `app`'s run function, created on entry | the loop, exclusively, for draw, input and bell | an explicit consuming `finish` that returns the restore result; `Drop` restores only when `finish` never ran; ratatui's panic hook restores on panic |
| Standard output | the session's backend | the bell writes through the backend writer | flushed after each draw and after the bell |
| Clock readings | `app`, the only caller of `Instant::now` | passed by value into every timer operation | none |

Timer state is an enum of Running, Paused and Ready. Running holds the instant it
last started and the time already banked; Paused holds the banked time; Ready holds
nothing. Elapsed time is `banked` plus `now.saturating_duration_since(since)`, added
with saturating arithmetic and capped at the phase length. No code adds a `Duration`
to an `Instant`, because that addition panics on overflow and the representation
does not need it. Every timer operation first observes a passed phase end, then
applies itself (T10). A reading at or after the end moves one phase and no further (T09).

Progress is a private-field type built only from elapsed time and a phase length,
with `Duration::div_duration_f64`. Phase length is never zero (value table), so the
ratio is defined, and the cap keeps it within 0 to 1. This type is what makes
ratatui's `Gauge::ratio` panic unreachable (D02).

The round counter is bounded by `N`: it resets to 1 by comparison after the long
break, rather than taking a remainder of an ever-growing count.

### Lint interactions decided here

- crossterm's `Event` and `KeyCode` are large foreign enums. A wildcard arm on them
  trips `wildcard_enum_match_arm`. Use `let … else`, `if let` and equality instead;
  a scratch probe with those forms passed Clippy clean. An expectation here is a
  review item, not a default.
- Ctrl-C reaches the program as a `c` key press with the Ctrl modifier; raw mode
  delivers no signal. The probe confirmed it. K01 and K03 pin both sides.
- Windows reports key releases as well as presses; only presses map to commands (K02).
- `main` returns `std::process::ExitCode`; the `exit` lint forbids `process::exit`.
- `std::env::args` panics on a non-Unicode argument; read `args_os` (C07).
- Split areas with `Layout::areas` into a fixed-size array; no indexing.
- Casts are avoided: minutes become seconds through `u64::from`, and progress
  comes from `div_duration_f64`.

### Ownership checks

| Document or section | Vocabulary | Only allowed in | Check |
| --- | --- | --- | --- |
| Contract C01–C11 | `--work`, `--short`, `--long`, `--every`, `--help` | `src/cli.rs` | O1 |
| Value table | the range bound 99 and the default 25 | `src/settings.rs` | O2 |
| Contract K01–K03 | `KeyCode`, `KeyModifiers`, `KeyEventKind` | `src/terminal.rs` | O3 |
| Contract M01–M05, C09 | `try_init`, `try_restore`, `event::poll`, `event::read`, `is_terminal`, the bell byte | `src/terminal.rs` | O4 |
| Contract D01–D05 | `Gauge`, `Layout`, and the on-screen labels | `src/view.rs` | O5 |
| Timer rows T01–T11 | `Instant::now` | `src/app.rs` | O6 |
| Direction rule above | `ratatui` | never in `settings`, `cli`, `timer` | O7 |

Each command below must print nothing. Run them from the repository root in a POSIX
shell.

~~~sh
# O1
grep -rnE -e '--(work|short|long|every|help)' src | grep -v -e 'src/cli.rs' -e '/tests.rs'
# O2
grep -rnwE '99|25' src | grep -v -e 'src/settings.rs' -e '/tests.rs'
# O3
grep -rnE 'KeyCode|KeyModifiers|KeyEventKind' src | grep -v -e 'src/terminal.rs' -e '/tests.rs'
# O4
grep -rnE 'try_init|try_restore|event::(poll|read)|is_terminal|x07' src | grep -v 'src/terminal.rs'
# O5
grep -rnE 'Gauge|Layout|"(Work|Short break|Long break|Running|Paused|Ready)"' src | grep -v -e 'src/view.rs' -e '/tests.rs'
# O6
grep -rn 'Instant::now' src | grep -v -e 'src/app.rs' -e '/tests.rs'
# O7
grep -ln 'ratatui' src/settings.rs src/cli.rs src/timer.rs
~~~

A hit is a prompt to look, not proof of a defect.

## 2. Errors, compatibility and operations

| Class | Examples | Shape | Outcome |
| --- | --- | --- | --- |
| Invalid input | C03–C07, C11 | one `cli` error enum: missing value, invalid value (flag, value, range), unknown argument, repeated flag, non-Unicode argument | one line on stderr, exit 2, before the terminal is touched |
| Unusable environment | C09 | a run-error variant with no cause | one line on stderr, exit 1, nothing on stdout |
| Terminal failure | M02 | a run-error variant holding the failed step (enter, draw, read input, ring bell, restore) and the `io::Error` as `source()` | terminal restored first; `main` prints the step, then the cause; exit 1 |
| Restore failure after a run failure | M05 | both errors kept | both reported, run error first; exit 1 |
| Programming defect | a dependency panic route | none: they are made unreachable, below | ratatui's panic hook restores the terminal |
| Cancellation | Ctrl-C | the quit command, not a signal | the normal quit path, exit 0 |

Each error's `Display` names only its own level; the cause travels through
`source()`, and `main` alone renders the chain. The `terminal` module owns the
restore sequence so every path reaches it: normal quit, a `?` return from the
loop (through `Drop`), and a panic (through ratatui's hook). Use `try_init`, never
`init` or `run`: those two panic when the terminal cannot be entered.

Dependency panic routes and their guards:

| Route | Guard | Evidence |
| --- | --- | --- |
| `Gauge::ratio` outside 0 to 1 | the progress type | D02 |
| `Layout::areas` with a constraint count unlike the array length | a programming defect | every render test in D03–D05 exercises it |
| `ratatui::init`, `ratatui::run` | not called | O4 plus review |
| `std::env::args` | not called | C07 |

Compatibility: an application with no published API, no semver promise and no
features. `rust-version` is `1.98.1`, the baseline pin that `init` writes. We make no
promise below it and run no MSRV lane. Supported targets are x86_64 Windows (MSVC)
and x86_64 Linux (GNU). There are no auto-trait commitments.

Operations: stdout carries only the alternate-screen interface or the `--help`
usage; stderr carries errors. Settings come from flags over defaults, with no
other source. There is one thread. The loop blocks in `event::poll` for at most
250 ms, then ticks and redraws. That bounds how stale the screen can be, and it
has no effect on timing accuracy (T03). There is no log output, because stdout
belongs to the interface. The release profile keeps the baseline's overflow checks.
Time arithmetic saturates, and minutes times 60 is at most 5,940, so no overflow
path exists to trip them.

Concurrency: none. There are no tasks, threads or async runtime, so the conditional
concurrency decisions do not apply.

### Dependencies below 1.0

| Crate | Pinned version | API items relied on | Documentation read |
| --- | --- | --- | --- |
| ratatui, `default-features = false`, `features = ["crossterm"]` | 0.30.2 | `try_init`, `try_restore`, `DefaultTerminal`, `Terminal::new`, `Terminal::draw`, `Terminal::backend_mut`, `Frame::area`, `Frame::render_widget`, `Layout::vertical`, `Layout::areas`, `Constraint::Length`, `Gauge::ratio`, `Gauge::label`, `Gauge::block`, `Block::bordered`, `Paragraph::new`, `backend::TestBackend`, the `ratatui::crossterm` re-export | 0.30.2 source rustdoc: `init.rs` (init panics; try_init enables raw mode before the alternate screen; restore order), `lib.rs` re-exports, ratatui-widgets 0.3.2 `gauge.rs` (ratio panics outside 0..=1), ratatui-core 0.1.2 `layout.rs` (`areas` panics on count mismatch), `terminal/backend.rs`; a scratch compile probe on 2026-09-24 |
| crossterm, through the ratatui re-export only | 0.29.0 | `event::poll`, `event::read`, `Event::Key`, `KeyEvent` fields `code`, `modifiers`, `kind`, `KeyEventKind::Press`, `KeyCode::Char`, `KeyCode::Esc`, `KeyModifiers::CONTROL` | 0.29.0 `event.rs`: `kind` is always set on Windows, so releases arrive there; `poll` returns early when an event is ready |
| ratatui-core, ratatui-widgets, ratatui-crossterm | 0.1.2, 0.3.2, 0.1.2 | reached through ratatui | as above |

Do not add crossterm as a direct dependency. A second crossterm version would
compile, but its event types would not match the backend's. Any version change to
these crates is a dependency update with its own review.

## 3. Verification matrix

| Item | Decision |
| --- | --- |
| Owned projects | package `pomodoro` at the root (application, binary only); the baseline's `xtask` workspace |
| Toolchain | 1.98.1 from the baseline pin; no separate MSRV row |
| Feature lanes | none; the package declares no features, so there is no minimal job |
| Linux, `ubuntu-24.04` | the baseline's full lane: executes every automated row, including the Linux branch of C07 and the release test lane |
| Windows, `windows-2025` | the baseline's Windows lane: executes every automated row, including the Windows branch of C07, plus the release build |
| macOS | none; no claim |
| Compile-only evidence | none claimed |
| Binary tests | `tests/cli.rs` spawns `CARGO_BIN_EXE_pomodoro` with stdout piped. Piped stdout is not a terminal, which is what makes C09 and C10 testable in CI |
| Unit tests | readings are offsets from one base instant; no test sleeps; D03–D05 draw into `TestBackend` |
| Manual checks | M01, M03, M04 on Windows Terminal on the owner's Windows 11 host and on one Linux terminal; run by the owner or a named reviewer; each result becomes a row in the slice's review record |
| Review-only | M02's single restore path, M05, any `#[expect]`, both `cfg` branches of C07, the lint interactions in section 1 |
| Advisories | the baseline's advisory commands for the product graph and the `xtask` graph; ratatui brings the product's first real dependency graph |
| Gate numbers | `cargo xtask gates` output and O1–O7 results at the stop after slice 2 |

C07 builds its argument with `OsStringExt::from_vec` on Unix and
`OsStringExt::from_wide` with an unpaired surrogate on Windows. Each branch is
active on its own lane only, so both lanes must be green before C07 counts as covered.

## 4. Implementation slices

Iterate with the baseline's fast check; each slice ends with its full check.

### S1: a bar that counts down

- Files: the baseline via `cargo init --name pomodoro` and the baseline's `init`;
  `Cargo.toml` with ratatui as above and the lock resolved at setup; `settings`
  (defaults only), `timer` (running state only), `view`, `terminal` (quit keys only),
  `app`, `main`, `tests/cli.rs`.
- Rows: C01, C09, T01–T04, D01, D02, D04, K01 (quit keys), K02, M01, M02.
- Failing test first: `progress_stays_within_unit_interval`. With an uncapped ratio,
  a reading after the end panics inside `Gauge::ratio`. Then `refuses_non_terminal_stdout`,
  which fails while the binary enters the alternate screen on a pipe.
- Done when the full check is green and `cargo run` on the owner's host shows a
  25-minute bar counting down that `q` leaves cleanly (M01, Windows).

### S2: the pomodoro cycle

- Files: `timer` (Paused, Ready, phase end, skip, rounds), `terminal` (space, `s`,
  bell), `view` (state, round, key help), `app` (phase-end bell, command dispatch).
- Rows: T05–T11, D03, D05, K01 (all keys), K03, M03, M04, M05.
- Failing test first: `late_reading_ends_only_one_phase`, which exposes a loop that
  replays missed phases. Then `command_after_phase_end_applies_to_next_phase`, which
  exposes a space press that pauses a finished phase at zero.
- Done when the full check is green and hosted CI is green. Then stop for review, as
  the kickoff says.

### S3: the command line

- Files: `cli` (new), `settings` (value types with fallible construction), `main`
  (exit code 2), `tests/cli.rs`.
- Rows: C02–C08, C10, C11.
- Failing test first: `rejects_non_unicode_argument`, which exposes a panic from
  `std::env::args`. Then `reports_first_wrong_argument`, which exposes a parser that
  collects every error or reports the last one.
- Done when every contract row has passing evidence, M01, M03 and M04 are recorded on
  both terminals, the full check is green and CI is green.

## 5. Rehearsals and risks

| Change | Files touched | Finding |
| --- | --- | --- |
| Rename an upstream field | Not applicable: no external service. The nearest equivalent, renaming a key, touches `src/terminal.rs` (binding and help text) and row K01 | one module; none |
| Swap the main external service | The nearest equivalent is replacing crossterm with ratatui's termwiz backend, which has the same raw mode, alternate screen and key events. It touches `Cargo.toml` features and `src/terminal.rs`; `view` uses backend-independent widgets | one module. K02 and M03 are re-checked because release events and the bell are backend behaviour |
| Add a sibling operation, restarting the phase on `r` | `src/timer.rs` and its tests (the transition), `src/terminal.rs` (binding, help text), `src/app.rs` (the dispatch arm, which the exhaustive match on the command enum forces), one contract row | each file changes for its own responsibility; none |

| Risk | Owner | Handling |
| --- | --- | --- |
| The clock counts sleep on some platforms and not on others | owner, accepted in the decision record | no correction; no row |
| Being killed by a signal leaves Linux raw mode on | owner, accepted in the decision record | `reset` repairs it; no signal handling |
| The ratatui and crossterm 0.x API changes on upgrade | owner | the pinned lock; upgrades are reviewed dependency updates |
| The Windows child process may not receive an unpaired surrogate unchanged | agent, in S3 | if the Windows branch of C07 cannot build such an argument, report it; do not drop the row |
| The bell is muted or visual on some terminals | owner | M03 records what each checked terminal did |
| Manual rows need a person at a terminal | owner | S3 does not close until M01, M03 and M04 are recorded |

Choices the owner may reverse are listed in the decision record. Each maps to rows:
waiting for space (T06, T11), skip semantics (T08), the 1-to-99 range (C02, C03),
starting immediately (T01), bar direction (D02), and exit codes (C03–C10).

Status: this plan reports planned verification only; no check listed here has run.
The one executed step was research. A scratch project outside this repository
compiled ratatui 0.30.2 with only the crossterm feature. Clippy passed with
`-D warnings` plus `wildcard_enum_match_arm`, `indexing_slicing` and
`cast_precision_loss`. Drawing a gauge at 0 × 0, 1 × 1 and 40 × 5 did not panic.
That ran on Windows with Rust 1.98.1 on 2026-09-24.
