# Pomodoro timer: behaviour contract

Reader: the agent implementing `pomodoro`. This is the source of truth for tests.

Provider contracts: none. The program talks to no external service; the terminal
is a local resource owned as described in the plan.

Notation. `L` is the length of the current phase. `N` is the `--every` value.
`W`, `S` and `B` are a work phase, a short break and a long break. A reading is
a monotonic clock instant.

## Command line

| ID | Given | Then | Evidence |
| --- | --- | --- | --- |
| C01 | no arguments | work 25 min, short break 5 min, long break 15 min, a long break after every 4th work phase | `defaults_are_classic_pomodoro` |
| C02 | `--work 1`, `--work 99`, `--short 1`, `--long 99`, `--every 1`, `--every 99`, `--work 025` | accepted as 1, 99, 1, 99, 1, 99 and 25 | `accepts_bounds_and_leading_zeros` |
| C03 | `--work 0`, `--short 100`, `--long abc`, `--every -1`, `--work ""`, `--work " 5"` | one line on stderr naming the flag, the rejected value and the range 1 to 99; exit 2; stdout empty | `rejects_out_of_range_or_non_numeric` |
| C04 | a flag as the last argument, such as `--work` | one line on stderr naming the flag and saying its value is missing; exit 2; stdout empty | `rejects_missing_value` |
| C05 | `--wrok 5`, `--work=5`, or a bare `5` | one line on stderr naming the unrecognised argument; exit 2; stdout empty | `rejects_unknown_argument` |
| C06 | `--work 10 --work 20` | one line on stderr naming the repeated flag; exit 2; stdout empty | `rejects_repeated_flag` |
| C07 | an argument that is not valid Unicode, on Linux and on Windows | one line on stderr; exit 2; no panic | `rejects_non_unicode_argument` |
| C08 | `-h` or `--help` anywhere among the arguments, including beside invalid ones | usage on stdout listing the four flags with their ranges and defaults, and the keys; stderr empty; exit 0; works without a terminal | `help_prints_usage` |
| C09 | valid arguments, stdout not a terminal | one line on stderr saying an interactive terminal is required; exit 1; stdout empty, so no escape sequence was written | `refuses_non_terminal_stdout` |
| C10 | invalid arguments, stdout not a terminal | the argument error is reported; exit 2 | `argument_errors_precede_terminal_check` |
| C11 | `--work 0 --wrok 5`, two wrong arguments | only the first in argument order, `--work 0`, is reported | `reports_first_wrong_argument` |

## Timer

| ID | Given | Then | Evidence |
| --- | --- | --- | --- |
| T01 | the program starts | phase W, round 1 of N, running, remaining `L`, progress 0 | `starts_running_first_work_phase` |
| T02 | running, a reading `t` after the start with `t < L` | remaining `L − t` | `remaining_counts_down` |
| T03 | running, the same total time elapsing in one step or in 1,000 uneven steps | identical remaining time | `uneven_ticks_do_not_drift` |
| T04 | a reading earlier than the phase start | remaining `L`, progress 0, no panic | `clock_before_start_reads_as_no_elapsed_time` |
| T05 | running; space; any time passes; space | pausing freezes remaining; the paused phase shows Paused; resuming counts down from the frozen value | `pause_freezes_remaining` |
| T06 | running, and a reading at or after the phase end | the phase end is reported exactly once; the next phase is loaded at full length and shows Ready, not running | `phase_end_reported_once` |
| T07 | N = 4, then N = 1, phases ended one after another | N = 4: W1 S W2 S W3 S W4 B W1 …; N = 1: W1 B W1 B … | `cycle_follows_long_break_interval` |
| T08 | any phase, whether running, paused or Ready; `s` | the next phase in the cycle, at full length, Ready; no phase end is reported, so no bell | `skip_moves_to_next_phase_silently` |
| T09 | running, next reading 3 hours after the phase end | exactly one phase end; the following phase is Ready at full length | `late_reading_ends_only_one_phase` |
| T10 | running, phase end passed but not yet observed; space | the phase end is reported first; space then starts the next phase | `command_after_phase_end_applies_to_next_phase` |
| T11 | Ready; space | running, counting down from full length | `space_starts_ready_phase` |

## Screen

| ID | Given | Then | Evidence |
| --- | --- | --- | --- |
| D01 | remaining 25:00.000, 24:59.001, 24:59.000, 0:00.001, 0:00.000, 99:00.000 | bar label `25:00`, `25:00`, `24:59`, `00:01`, `00:00`, `99:00` | `remaining_label_rounds_up_to_whole_seconds` |
| D02 | elapsed 0, `L/2`, `L`; a reading before the start; a reading 3 hours after the end | progress 0, 0.5, 1, 0, 1; progress is never outside 0 to 1 | `progress_stays_within_unit_interval` |
| D03 | a 60 × 10 terminal | the frame shows the phase name (`Work`, `Short break` or `Long break`), the state (`Running`, `Paused` or `Ready`), `Round k of N`, the bar with its label, and the key help | `frame_shows_phase_state_round_and_keys` |
| D04 | terminals of 0 × 0, 1 × 1 and 10 × 3 | a frame is drawn; no panic | `renders_in_tiny_terminals` |
| D05 | the break after W2 with N = 4, and the long break | `Round 2 of 4` and `Round 4 of 4`; a break shows the round it follows | `break_shows_round_it_follows` |

## Keys

| ID | Given | Then | Evidence |
| --- | --- | --- | --- |
| K01 | a key press of space; `s` or `S`; `q`, `Q`, Esc or Ctrl-C | start or pause; skip; quit | `keys_map_to_commands` |
| K02 | a key release or key repeat of any key in K01 | no command | `ignores_key_release_and_repeat` |
| K03 | `c` without Ctrl, Ctrl-S, any other key, mouse, focus, paste and resize events | no command | `unbound_input_is_ignored` |

## Terminal lifecycle

| ID | Given | Then | Evidence |
| --- | --- | --- | --- |
| M01 | quitting with `q`, Esc or Ctrl-C | normal screen back, cursor visible, typed characters echo again; exit 0 | manual check on Windows Terminal and one Linux terminal |
| M02 | a terminal read or write failure while running | the terminal is restored first; then one line on stderr naming the failed step, followed by its cause; exit 1 | `run_error_names_step_then_cause`, and review: every return from the run loop passes one restore |
| M03 | a phase ends | the terminal bell rings once | manual check, same terminals as M01 |
| M04 | the window is resized while running | the next frame fills the new size | manual check, same terminals as M01 |
| M05 | restoring the terminal fails | its error is reported on stderr after any run error; exit 1 | review of the session's finish path |

## Value distinctions

| ID | Value | Valid values | Routes in | Invariant and reason | Evidence |
| --- | --- | --- | --- | --- | --- |
| V01 | Phase length | whole minutes 1 to 99 | defaults; the `--work`, `--short` and `--long` values | never zero, so progress is always defined; two digits of minutes on screen | `defaults_are_classic_pomodoro`, `accepts_bounds_and_leading_zeros`, `rejects_out_of_range_or_non_numeric`; review: private field, fallible construction is the only route |
| V02 | Long-break interval `N` | 1 to 99 | default; the `--every` value | never zero, so the cycle always contains a work phase | as V01 |
| V03 | Round `k` | 1 to `N` | timer transitions only | the work phase's position in the cycle; never grows past `N` | `cycle_follows_long_break_interval`, `break_shows_round_it_follows` |
| V04 | Progress | 0 to 1 inclusive | computed from elapsed time and phase length only | the bar's precondition: ratatui panics on a ratio outside 0 to 1 | `progress_stays_within_unit_interval` |
| V05 | Elapsed time | 0 to `L` | clock readings passed to timer operations | never exceeds `L`, whatever the reading | `late_reading_ends_only_one_phase`, `clock_before_start_reads_as_no_elapsed_time` |
| V06 | Timer state | Running, Paused, Ready | start, space, `s`, phase end | transitions only as T05, T06, T08, T11 | the tests of those rows |
| V07 | Phase | W, S, B | timer transitions only | order as T07 | `cycle_follows_long_break_interval` |
| V08 | Command | start or pause, skip, quit | key presses only | mapping as K01 to K03 | the tests of those rows |

Prohibition sentences without an ID: 0.
