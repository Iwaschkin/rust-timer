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
| C08 | `-h` or `--help` anywhere among the arguments, including beside invalid ones | usage on stdout listing every flag with its values and default, and the keys; stderr empty; exit 0; works without a terminal | `help_prints_usage` |
| C09 | valid arguments, stdout not a terminal | one line on stderr saying an interactive terminal is required; exit 1; stdout empty, so no escape sequence was written | `refuses_non_terminal_stdout` |
| C10 | invalid arguments, stdout not a terminal | the argument error is reported; exit 2 | `argument_errors_precede_terminal_check` |
| C11 | `--work 0 --wrok 5`, two wrong arguments | only the first in argument order, `--work 0`, is reported | `reports_first_wrong_argument` |
| C12 | `--color` with `auto`, `truecolor`, `256`, `16` or `none` | accepted; without the flag, `auto` | `accepts_appearance_choices` |
| C13 | `--glyphs` with `auto`, `emoji`, `symbols` or `ascii` | accepted; without the flag, `auto` | `accepts_appearance_choices` |
| C14 | `--motion` with `on` or `off` | accepted; without the flag, `on` | `accepts_appearance_choices` |
| C15 | `--color 8`, `--glyphs fancy`, `--motion maybe` | one line on stderr naming the flag, the rejected value and the accepted values; exit 2; stdout empty | `rejects_unknown_choice` |

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
| D04 | terminals of 0 × 0, 1 × 1, 10 × 3 and 30 × 8, in every state, with and without the Ready popup | a frame is drawn; no panic | `renders_in_tiny_terminals` |
| D05 | the break after W2 with N = 4, and the long break | `Round 2 of 4` and `Round 4 of 4`; a break shows the round it follows | `break_shows_round_it_follows` |
| D06 | a terminal of at least 80 × 24 | the large layout: big block digits of the time left, the braille dial, the progress bar, the cycle ribbon and the key caps | `large_layout_shows_every_panel` |
| D07 | at least 48 × 16 and less than the large size | the medium layout: smaller block digits, the bar, the ribbon and the keys; no dial | `medium_layout_drops_the_dial` |
| D08 | smaller than the medium size | the compact layout: phase, state, `MM:SS` as text, the bar and the keys | `compact_layout_keeps_essentials` |
| D09 | the progress bar at progress p over w cells | the filled length is p·w rounded down to an eighth of a cell, drawn with eighth-block glyphs; its colour ramps from a darker shade of the phase colour to the phase colour | `gradient_bar_fills_by_eighths` |
| D10 | the dial at progress p | the lit arc starts at twelve o'clock and runs clockwise over p of the circle; the rest is drawn in the track colour | `dial_arc_follows_progress` |
| D11 | the cycle ribbon with N work phases | 2N segments, one per phase of the cycle, widths in proportion to their lengths; passed phases filled, the current one marked | `ribbon_shows_the_cycle` |
| D12 | Ready | a popup over the dimmed screen, with the alert-colour border, a shadow, the loaded phase's name in block letters, its glyph and the space hint | `ready_popup_announces_next_phase` |
| D13 | Paused | a Paused badge, and the clock digits dimmed towards the background | `paused_screen_dims_the_clock` |

## Colour

The palette is designed in 24-bit colour. A tier decides what the terminal
receives.

| ID | Given | Then | Evidence |
| --- | --- | --- | --- |
| P01 | no `--color` flag, and the first matching environment in this order: `NO_COLOR` set and not empty; `TERM=dumb`; `COLORTERM` containing `truecolor` or `24bit`; `WT_SESSION` set; a Windows console with VT output; `TERM` containing `256`; anything else | tiers none, none, truecolor, truecolor, truecolor, 256, 16; an empty `NO_COLOR` is ignored | `color_depth_follows_environment` |
| P02 | `--color` with a tier, whatever the environment, including `NO_COLOR` | the flag's tier | `color_flag_overrides_environment` |
| P03 | each text colour on each background it is drawn on, and each border on its background | text at least 4.5:1 and borders at least 3:1 (WCAG 2.2), in truecolor and in the 256 tier | `palette_meets_contrast_targets` |
| P04 | any frame in the 256 tier, effects included | every colour is `Indexed` 16 to 255 or `Reset`; none is RGB | `frames_quantize_to_256` |
| P05 | any frame in the 16 tier | every colour is a named ANSI colour or `Reset` | `frames_quantize_to_16` |
| P06 | any frame in tier none | every foreground, background and underline colour is `Reset`, and bold and other modifiers stay | `no_color_keeps_modifiers` |
| P07 | any frame in the truecolor, 256 or 16 tier | every cell's background is painted; none is `Reset` | `background_is_painted` |
| P08 | the three phases | each has its own name, glyph and border type, so colour is never the only difference | `phases_differ_beyond_colour` |

## Glyphs

| ID | Given | Then | Evidence |
| --- | --- | --- | --- |
| G01 | no `--glyphs` flag, and the first match of: `TERM` is `linux` or `dumb`; Windows with neither `WT_SESSION` nor `TERM_PROGRAM`; anything else | tiers ascii, symbols, emoji; the flag, when given, wins | `glyph_tier_follows_environment` |
| G02 | every glyph of the emoji tier | a single code point, with no U+FE0F or U+200D, that ratatui measures as 2 cells | `emoji_glyphs_are_wide_and_single` |
| G03 | every glyph of the symbols and ascii tiers | 1 cell each; the ascii tier is ASCII only | `fallback_glyphs_are_narrow` |

## Motion

| ID | Given | Then | Evidence |
| --- | --- | --- | --- |
| E01 | `--motion on`, at start | an intro effect runs, and finishes within 1.5 s | `intro_effect_finishes` |
| E02 | a phase ends and the next waits | the popup enters with an effect, then its border pulses with a period of at least 1.5 s, under 3 flashes a second, until a key acts | `alert_pulse_is_slow_enough` |
| E03 | the loop choosing its next wake-up | 33 ms while an effect runs; otherwise the next whole second of the time left, and never more than 250 ms | `wake_interval_follows_effects` |
| E04 | `--motion off` | no effect ever runs; the popup and every screen appear without animation | `motion_off_runs_no_effects` |
| E05 | any effect at any terminal size, 0 × 0 included | no panic | `effects_render_in_tiny_terminals` |

## Terminal integration

| ID | Given | Then | Evidence |
| --- | --- | --- | --- |
| I01 | running, paused or ready | the window title is the phase glyph, `MM:SS` and the phase name while it counts, or the bell glyph, the phase name and "ready" while it waits; either ends with a dash and `pomodoro`; written only when its text changes | `title_follows_timer` |
| I02 | a terminal known to show OSC 9;4 progress: `WT_SESSION` set, `TERM_PROGRAM=vscode`, `TERM=xterm-kitty`, or `VTE_VERSION` at least 8000 | state 1 with the percentage while running, 4 while paused, 3 while ready; written when it changes and at least every 10 s | `progress_follows_state` |
| I03 | any other terminal | no OSC 9;4 is written | `progress_only_on_known_terminals` |
| I04 | every exit path | progress is cleared with state 0 and the saved title restored, before raw mode ends | `finish_clears_progress_and_title`, and review of the finish path |

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
| M06 | the demonstrator in Windows Terminal | colours, emoji, block digits, dial, effects, title and taskbar progress look as designed | manual check by the owner |
| M07 | the demonstrator in VS Code's terminal | the same, with emoji aligned; the title and progress only if the tab title settings include them | manual check by the owner |
| M08 | the demonstrator in a Linux terminal | the same as M06, as far as that terminal supports | manual check by the owner |

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
| V09 | Colour tier | truecolor, 256, 16, none | `--color`, or detection as P01 | every drawn colour passes through the tier before it reaches the terminal | `frames_quantize_to_256`, `frames_quantize_to_16`, `no_color_keeps_modifiers` |
| V10 | Glyph tier | emoji, symbols, ascii | `--glyphs`, or detection as G01 | every glyph comes from the tier's table | `emoji_glyphs_are_wide_and_single`, `fallback_glyphs_are_narrow` |
| V11 | Motion | on, off | `--motion` | with off, no effect is ever started | `motion_off_runs_no_effects` |

Prohibition sentences without an ID: 0.
