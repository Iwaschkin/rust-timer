# Pomodoro timer: decision record

Reader: the owner of rust-timer.

The installed skills decide structure, process and evidence. This pack decides
behaviour. Where they conflict on structure, the skill wins and the conflict is
reported.

## What we are building

The repository is empty today, so no existing code constrains these choices.
We are building a terminal pomodoro timer, run as `pomodoro`.
It shows the current phase and a progress bar that fills as the phase runs.
The bar carries the remaining time as minutes and seconds.
Work phases alternate with short breaks.
Every fourth work phase is followed by a long break instead.
When a phase ends, the terminal bell rings once.
The next phase then waits until you press space.[^1]

Space starts, pauses and resumes the timer.
`s` skips to the next phase.
`q`, Esc and Ctrl-C quit.[^2]

Four flags change the cycle when the program starts.
`--work`, `--short` and `--long` set phase lengths in whole minutes.
`--every` sets how many work phases come before a long break.
Each value is a whole number from 1 to 99.
The defaults are 25, 5, 15 and 4.[^3]
A mistyped flag stops the program with a one-line reason before the screen changes.

## What we are not building

- No saved history, statistics or streaks.
- No configuration file and no environment variables.
- No desktop notifications or sound files; the terminal bell is the only alert.
- No mouse support, colour themes or task names.
- No published crate and no promises to library users.
- No macOS support promise; it may work there, but nobody checks it.

Deferred: a key that restarts the current phase, and lengths in seconds for quick demos.

## Technology choices

- Rust 1.98.1 with edition 2024, the compiler the quality baseline pins.
- ratatui 0.30.2 draws the screen, with only its crossterm backend enabled.
- crossterm 0.29.0 comes through ratatui rather than as its own dependency.
  That keeps exactly one crossterm version in the build.
- The standard library reads the command line.
  Four flags do not justify a parsing crate.
- One thread and no async runtime.
  The program wakes at least four times a second to redraw.
- One package holding a binary and no library.
  Nothing outside the program uses its types.

## Policies

- The quality baseline's lints, checks and evidence rules apply unchanged.
- Remaining time comes from a monotonic clock, not from counting redraws.
  A slow or late redraw never shortens or lengthens a phase.
- The terminal is restored on every quit and every error.
- Windows and Linux are the supported platforms, and CI runs the tests on both.
- Terminal restore, the bell and resizing are checked by hand, because CI has no terminal.

## Choices made for you, easy to reverse

1. The next phase waits for space instead of starting by itself.
   Reason: an unattended timer would keep cycling and count pomodoros nobody worked.
2. Skipping moves on exactly as if the phase had ended, but without the bell.
   A skipped work phase still counts towards the long break.
3. Lengths are whole minutes from 1 to 99.
   The limit keeps the display to two minute digits.
   The shortest phase therefore takes one minute.
4. The timer runs as soon as the program opens.
5. The bar fills as time passes rather than emptying.
6. Argument mistakes exit with code 2, runtime failures with 1, and quitting with 0.
7. The command is `pomodoro`, although the repository is `rust-timer`.

## Open questions

None blocks the first slice.
Choices 1, 2 and 4 take effect in the second slice.
Choices 3 and 6 take effect in the third slice.
Change any of them before its slice starts and the contract row changes with it.

## Risks you own

- Laptop sleep: the standard clock counts sleep time on some platforms and not on others.
  After waking, a phase may have ended or may carry on where it stopped.
  We accept either outcome and do not correct for it.
- Some terminals mute the bell or flash the window instead.
  We check it by hand on Windows Terminal and one Linux terminal.
- ratatui and crossterm are below version 1.0.
  Their next minor release can change what the code calls.
  Upgrading them is a deliberate change with its own review, never a routine refresh.
- If another program kills `pomodoro`, a Linux terminal can stay in raw mode.
  Typing `reset` repairs it.
  Handling that case needs a signal-handling dependency, which we leave out.
- Hand checks need a person at a real terminal.
  You, or a reviewer you name, run them before the last slice closes.

[^1]: Behaviour contract rows T06, T07, T11 and M03.
[^2]: Rows K01 to K03 and M01.
[^3]: Rows C01 to C08.
