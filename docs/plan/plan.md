# Pomodoro timer: plan

Reader: the agent implementing `pomodoro`.

Behaviour is fixed by the behaviour contract; row ids below refer to it. Commands,
lints and evidence rules come from `rust-quality-baseline`; this plan copies none
of them. The research behind the overhaul is in `docs/research/visual-overhaul.md`.

## 1. Structure and boundaries

The project root is the repository root. It holds one Cargo package, `pomodoro`: an
application with a binary target and no library target, `publish = false`. The
baseline's `xtask/` sits beside it as its own workspace with its own lock.

Unit tests live beside their module, in `src/<module>/tests.rs`, declared with
`#[cfg(test)] mod tests;`, so the ownership checks can exclude them by path. The
command-line rows run against the built binary from `tests/cli.rs`, which bounds
every child it spawns.

| Module | Owns | Depends on |
| --- | --- | --- |
| `settings` | phase lengths and interval, their range, defaults and parsing | std |
| `options` | everything the command line chooses: the settings, the colour and glyph choices (`auto` or a fixed tier) and motion on or off | `settings`, `theme`, `glyphs` |
| `cli` | the flags and their choices, declared for clap, which writes the usage text and argument errors | `options`, clap |
| `environment` | one snapshot of the environment variables that decide colour, glyphs and progress support; which terminal this is | std |
| `theme` | the palette (every RGB value), colour tiers and their detection, the per-frame quantize pass, the contrast maths | ratatui style and buffer, `environment` |
| `glyphs` | glyph roles, the three tier tables, tier detection | `environment` |
| `timer` | phases, rounds, states, transitions, remaining time, progress, the cycle's phase list | `settings` |
| `view` | the three layouts and their composition, labels | `timer`, `theme`, `glyphs`, ratatui widgets |
| `view/clock`, `view/bar`, `view/dial`, `view/ribbon`, `view/popup` | the big digits, the gradient bar, the braille dial, the cycle ribbon, the Ready popup | `theme`, ratatui; `view/clock` also tui-big-text |
| `motion` | which effect runs on which event, the effect manager, the wake-up interval | tachyonfx, `theme`, ratatui buffer |
| `terminal` | entering and restoring the terminal, keys, the bell, the window title, OSC 9;4 progress, synchronized output | ratatui's crossterm |
| `app` | the loop: read the clock, tick, run effects, draw, update title and progress, act on keys | all of the above |
| `main` | reading `args_os` and the environment, exit codes, error chains | `cli`, `app`, `environment` |

Direction: `settings`, `options`, `cli`, `timer` and `environment` never name
ratatui. `view` never names crossterm or tachyonfx. The timer's cycle rule (which
phase follows which) is written once, in `timer`, and the ribbon reads it.

### Values and ownership

| Value or resource | Owner | Borrowed by | Completion |
| --- | --- | --- | --- |
| Options | `main`, moved into `app::run` | the loop reads them | none |
| Environment snapshot | `main`, read once at start | tier detection, progress detection | none |
| Timer | a local in `app`'s run function | `view` shared for one draw; commands exclusive for one call | nothing persists |
| Effect manager | `app`'s run function, through `motion` | exclusive inside each draw, after the widgets render | dropped at quit |
| Terminal session | `app`'s run function | the loop, exclusively | explicit `finish` clears progress, restores the title, then leaves raw mode; `Drop` restores only if `finish` never ran |
| Clock readings | `app`, the only caller of `Instant::now` | passed by value | none |

The frame pipeline is fixed: render the widgets in 24-bit palette colours, run the
effects, then quantize the whole buffer to the tier. Effects turn every colour into
RGB mid-flight, so quantizing last is what keeps P04 to P06 true.

### Lint and library traps decided here

- `Gauge::ratio`, `Gauge::percent` and tachyonfx's `hsl_shift(None, None, ..)`
  panic: clamp every ratio; use `hsl_shift_fg`.
- Use `Buffer::cell_mut`, never `buf[(x, y)]`; `Layout::areas::<N>` stays in step
  with its constraint array.
- `symbols::Marker` and crossterm's enums are foreign and large: use `let … else`
  and equality, or a reasoned expectation at a foreign non-exhaustive match.
- In tier none, every colour becomes `Reset` before drawing; crossterm's own
  NO_COLOR path would write `ESC[;m` and drop bold (research 3.1).
- Only the safe emoji set of research 4.4 is used; G02 tests it.
- clap reads a leading `-` as a flag: the numbers set `allow_negative_numbers`, so
  `-1` fails its range check instead of being an unknown flag.

### Ownership checks

| Document or section | Vocabulary | Only allowed in | Check |
| --- | --- | --- | --- |
| Contract C01–C15 | the flag names | `src/cli.rs` | O1 |
| Value table | the range bound 99 and the default 25 | `src/settings.rs` | O2 |
| Contract K01–K03 | `KeyCode`, `KeyModifiers`, `KeyEventKind` | `src/terminal.rs` | O3 |
| Contract M01–M05, C09, I01–I04 | `try_init`, `try_restore`, event polling, `is_terminal`, the bell byte, escape sequences | `src/terminal.rs` | O4 |
| Contract D01–D13 | `Gauge`, `Layout`, `Canvas`, the on-screen labels | `src/view` | O5 |
| Timer rows T01–T11 | `Instant::now` | `src/app.rs` | O6 |
| Direction rule above | `ratatui` | never in `settings`, `options`, `cli`, `timer`, `environment` | O7 |
| Contract E01–E05 | `tachyonfx` | `src/motion.rs` | O8 |
| Contract D06, D07, D12 | `tui_big_text` | `src/view/clock.rs` | O9 |
| Contract P01–P08 | RGB colour values | `src/theme.rs` | O10 |
| Contract P01, G01, I02 | environment variable names | `src/environment.rs` | O11 |
| Contract G01–G03 | emoji | `src/glyphs.rs` | O12 |
| Technology choice: clap | `clap::`, `use clap` | `src/cli.rs` | O13 |

Each command below must print nothing. Run them from the repository root in a POSIX
shell.

~~~sh
# O1
grep -rnE -e '--(work|short|long|every|help|color|glyphs|motion)' src | grep -v -e 'src/cli.rs' -e '/tests.rs'
# O2
grep -rnwE '99|25' src | grep -v -e 'src/settings.rs' -e '/tests.rs'
# O3
grep -rnE 'KeyCode|KeyModifiers|KeyEventKind' src | grep -v -e 'src/terminal.rs' -e '/tests.rs'
# O4
grep -rnE 'try_init|try_restore|event::(poll|read)|is_terminal|x07|x1b' src | grep -v -e 'src/terminal.rs' -e '/tests.rs'
# O5
grep -rnE 'Gauge|Layout::|Canvas|"(Work|Short break|Long break|Running|Paused|Ready)"' src | grep -v -e 'src/view' -e '/tests.rs'
# O6
grep -rn 'Instant::now' src | grep -v -e 'src/app.rs' -e '/tests.rs'
# O7
grep -rln 'ratatui' src | grep -E 'src/(settings|options|cli|timer|environment)\.rs'
# O8
grep -rn 'tachyonfx' src | grep -v -e 'src/motion.rs' -e '/tests.rs'
# O9
grep -rn 'tui_big_text' src | grep -v 'src/view/clock.rs'
# O10
grep -rnE 'from_u32|Color::Rgb' src | grep -v -e 'src/theme.rs' -e '/tests.rs'
# O11
grep -rnE 'NO_COLOR|COLORTERM|WT_SESSION|TERM_PROGRAM|VTE_VERSION' src | grep -v -e 'src/environment.rs' -e '/tests.rs'
# O12
grep -rn '🍅\|☕\|🌙\|🔔' src | grep -v -e 'src/glyphs.rs' -e '/tests.rs'
# O13
grep -rnE 'clap::|use clap' src | grep -v -e 'src/cli.rs' -e '/tests.rs'
~~~

A hit is a prompt to look, not proof of a defect.

## 2. Errors, compatibility and operations

| Class | Examples | Outcome |
| --- | --- | --- |
| Invalid input | C03–C07, C11, C15 | one `cli` error enum; one line on stderr, exit 2, before the terminal is touched |
| Unusable environment | C09 | one line on stderr, exit 1, nothing on stdout |
| Terminal failure | M02 | terminal restored first; `main` prints the step, then the cause; exit 1 |
| Restore failure after a run failure | M05 | both reported, run error first; exit 1 |
| Programming defect | a library panic route | made unreachable (section 1); ratatui's panic hook restores the terminal |
| Cancellation | Ctrl-C | the quit command, exit 0 |

Writing the title, progress or synchronized-output sequences is a terminal step
like drawing: a failure there is a terminal failure with its own step name.

Compatibility: an application, no published API, no features. `rust-version` is
`1.98.1`, the baseline pin. Supported targets are x86_64 Windows (MSVC) and x86_64
Linux (GNU).

Operations: stdout carries only the interface or the usage. The loop wakes at the
interval E03 chooses: 33 ms while an effect runs, otherwise the next whole second
of the time left, and never more than 250 ms. Each frame is wrapped in
synchronized output, which unsupporting terminals ignore. The research measured an
idle frame at 111 bytes and the worst gradient effect near 41 KB. Concurrency:
none; one thread, no async runtime.

### Dependencies below 1.0

| Crate | Pinned version | API items relied on | Documentation read |
| --- | --- | --- | --- |
| ratatui, default features off, `crossterm` | 0.30.2 | as before, plus `Block` titles, `BorderType`, `Shadow`, `Padding`, `Clear`, `Canvas` with `Marker::Braille`, `Flex`, `Buffer::content`, `Buffer::cell_mut`, `style::Color` | research sections 1.1–1.10, with file and line |
| crossterm, through ratatui only | 0.29.0 | as before, plus `terminal::SetTitle`, `Begin/EndSynchronizedUpdate`, `style::available_color_count`, the `Command` trait | research 3.1, 5.1, 7.2 |
| tachyonfx, default features off, `std` and `std-duration` | 0.25.2 | `EffectManager::{add_unique_effect, cancel_unique_effect, process_effects, is_running}`, `fx::{coalesce, sweep_in, dissolve, fade_from, hsl_shift_fg, ping_pong, repeating, parallel, sequence}`, `Interpolation`, `Motion`, `CellFilter` | research 2.2 and 7.3; the crate's README and `fx/mod.rs` |
| tui-big-text | 0.8.10 | `BigText::builder`, `PixelSize::{Full, HalfHeight, Quadrant}`, `lines`, `style`, `centered` | research 2.3 |

Research 2.1 proved each against our exact graph: one `ratatui-core 0.1.2`, a
clean build and no advisory. Adding them to this project repeats that proof with
`cargo tree -d` and the advisory check. Known costs: tui-big-text compiles `time`
through ratatui-widgets' calendar feature; `compact_str` 0.9 and 0.10 both
compile; tachyonfx has internal `unsafe`. Each is a line in the S5 or S6 review.

## 3. Verification matrix

| Item | Decision |
| --- | --- |
| Toolchain and lanes | 1.98.1; Linux full lane and Windows lane from the baseline; no features, no MSRV lane, no macOS |
| Binary tests | `tests/cli.rs`, stdout piped, every child killed at 20 s |
| Unit tests | readings are offsets from one base instant; screens draw into `TestBackend`; colour rows inspect the buffer after the quantize pass; environment rows build the snapshot directly, never from the real environment |
| Previews | an ignored test, `preview_screens`, renders named scenes to HTML under `target/preview/` with their colours; a headless browser screenshot of it is the design review of S4 and S5. It is not evidence for any row. |
| Program output | a pseudo-console capture (as used for M03) shows what the program writes: bell, title, progress; it is research evidence, not a CI test |
| Manual checks | M01–M05 were recorded in Windows Terminal; M06–M08 by the owner in Windows Terminal, VS Code and a Linux terminal |
| Review-only | M02, M05, I04's order, any `#[expect]`, both `cfg` branches of C07, the dependency costs in section 2 |
| Gate numbers | `cargo xtask gates` and O1–O12 at each stop |

## 4. Implementation slices

Iterate with the baseline's fast check; each slice ends with its full check, a
review in `evidence/`, and hosted CI.

Done: S1 (a bar that counts down), S2 (the pomodoro cycle), S3 (the command line),
S4 (palette, glyphs and capabilities), S5 (the showpiece), S6 (motion) and S7
(terminal integration). Their reviews are `evidence/s1-review.md` to
`evidence/s7-review.md`; M08 and the Linux runs of M01, M03 and M04 are still open.

### S9: a parser crate

- From an external review, and the rust-skills rule that followed: a maintained
  crate for a solved problem. clap 4.6.7, with its derive, replaces the 9 KB
  hand-written parser; the lengths keep their own range check, which clap calls.
- Files: `cli` (the flags declared for clap), `main` (clap prints help or the
  error), `settings` (a length shows its number, for clap's defaults).
- Rows: C02 to C08, C11 and C15 now state what clap guarantees. `--work=50` is
  accepted; errors are clap's, over several lines; help after an invalid argument
  is not reached; of two wrong arguments, clap reports one.
- Failing tests first: the old binary tests failed against clap on one-line
  errors and on `--work=5`; each row was rewritten before its test.

## 5. Rehearsals and risks

| Change | Files touched | Finding |
| --- | --- | --- |
| Rename an upstream field | not applicable; the nearest, renaming a key, touches `src/terminal.rs` and K01 | one module |
| Swap the main external service | the nearest are the terminal backend (`src/terminal.rs` and `Cargo.toml`) and the effects library (`src/motion.rs` and `Cargo.toml`); `view` draws plain buffers | one module each |
| Add a sibling operation, restarting a phase on `r` | `timer` and its tests, `terminal` (binding, help), `app` (the dispatch arm), one contract row | each file for its own responsibility |

| Risk | Owner | Handling |
| --- | --- | --- |
| Sleep and the monotonic clock; signals leaving raw mode | owner, accepted | as recorded |
| 0.x upgrades of ratatui, tachyonfx, tui-big-text | owner | pinned lock; reviewed updates |
| tachyonfx's internal `unsafe` | owner | dependency review line in the S6 review |
| Emoji width differs in a terminal with old width tables | owner | the safe set; `--glyphs symbols` as the escape hatch |
| OSC 9 collides with notification OSC 9 elsewhere | agent | progress only on positive detection (I02, I03) |
| The title stack is unverified in Windows Terminal | agent | restore by stack where supported; record what M06 shows |
| A slow terminal drops frames | agent | effects are time-based; frames are area-limited |

Status: slices 1 to 7 have passed their checks and hosted CI; S9 is in review.
