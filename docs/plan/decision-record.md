# Pomodoro timer: decision record

Reader: the owner of rust-timer.

The installed skills decide structure, process and evidence. This pack decides
behaviour. Where they conflict on structure, the skill wins and the conflict is
reported.

## What we are building

`pomodoro` is a terminal pomodoro timer.
Work phases alternate with short breaks, and every fourth work phase is followed by a long break.
When a phase ends, the bell rings once and the next phase waits until you press space.
Space starts, pauses and resumes; `s` skips; `q`, Esc and Ctrl-C quit.
Four flags set the lengths and the interval, each a whole number from 1 to 99.[^1]
Slices 1 to 3 built this, and it passed on Windows.

## The visual overhaul

On 2026-09-25 you asked for a visual overhaul as a technology demonstrator.
It should show what ratatui can do, not stop at what is sensible.
The phase-end alert must be visible as well as audible, because a terminal bell is easy to miss.[^2]

- The whole screen gets a designed dark palette, painted by the program.
- The time left is drawn in large block digits, in the colour of the phase.
- A round dial drawn in braille dots fills as the phase runs.
- A progress bar fills smoothly in eighths of a cell, with a colour ramp.
- A ribbon shows the whole cycle, with the current phase marked.
- When a phase ends, a popup with a shadow names the next phase, and its border pulses slowly.
- Pausing dims the clock and shows a Paused badge.
- Effects animate the start, phase changes and the alert.
- The window title shows the time left.
- In terminals that support it, the taskbar and tab show progress.
- Emoji mark the phases: 🍅 for work, ☕ for a short break, 🌙 for a long break.

The layout adapts to the window size, and a very small window still works.[^3]

## Colour, emoji and motion everywhere

Terminals differ, so the program adapts.
It detects how many colours the terminal shows and converts the palette to match.
`NO_COLOR` turns colour off, as the no-color.org convention asks.
Terminals that cannot show emoji get plain symbols instead.
Three new flags override the detection: `--color`, `--glyphs` and `--motion`.
`--motion off` turns every animation off.
Colour never carries meaning alone: each phase also has its own name, emoji and border style.
Every text colour meets the WCAG contrast level for normal text.
The alert pulse stays well below three flashes a second.[^4]

## What we are not building

- No saved history, statistics or configuration file.
- No desktop notifications or sound files.
- No mouse support, and no choice of themes: there is one designed palette.
- No published crate.
- No macOS promise.

## Technology choices

- Rust 1.98.1, edition 2024, as the quality baseline pins.
- ratatui 0.30.2 with only its crossterm backend.
- tachyonfx 0.25.2 for effects, and tui-big-text 0.8.10 for the large digits.
- Both were checked against our exact ratatui: one copy of its core, a clean build and no advisory.
- The standard library reads the command line and the environment.
- One thread, and no async runtime.

## Policies

- The quality baseline's lints, checks and evidence rules apply unchanged.
- Remaining time comes from the clock, not from counting redraws.
- The terminal, its title and its taskbar progress are restored on every exit.
- The screen redraws at least four times a second, and about thirty times a second while an effect runs.
- Windows and Linux are the supported platforms, and CI runs the tests on both.
- Appearance is checked by hand in Windows Terminal, VS Code and a Linux terminal.

## Decided for you, easy to reverse

1. The next phase waits for space instead of starting by itself.
2. A skipped work phase still counts towards the long break.
3. Lengths are whole minutes from 1 to 99.
4. Argument mistakes exit with 2, runtime failures with 1, and quitting with 0.
5. The palette is "Tomato Night": tomato for work, mint for short breaks, lavender for long breaks, amber for the alert.
6. The taskbar shows a paused state while paused, and an animated state while a phase waits.

## Risks you own

- The standard clock may or may not count time the computer spends asleep.
- If another program kills `pomodoro`, a Linux terminal can stay in raw mode; `reset` repairs it.
- ratatui, tachyonfx and tui-big-text are below version 1.0, so upgrades are deliberate reviews.
- tachyonfx contains `unsafe` code of its own; our lints cover only our code.
- VS Code shows the title and taskbar progress only if its tab title settings include them.
- Some terminals show emoji at a different width, and the program cannot detect that.
- Hand checks need a person at each terminal before the last slice closes.

[^1]: Behaviour contract rows C01 to C11, T01 to T11 and K01 to K03.
[^2]: Rows D12, E02, I01 and I02.
[^3]: Rows D06 to D11 and D04.
[^4]: Rows P01 to P08, G01 to G03, C12 to C15 and E02 to E04.
