# pomodoro

A pomodoro timer for the terminal. It runs a cycle of work phases and breaks, and
shows the current phase as a progress bar with the time left.

```text
 🍅 pomodoro   🍅⚪⚪⚪ Round 2 of 4
┏ 🍅 Work ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ ⏳ Running ┓
┃                                                                              ┃
┃                                                                              ┃
┃                                                                              ┃
┃                                                                              ┃
┃            ⣀⣤⣴⣶⣶⣶⣶⣦⣤⣀                                                        ┃
┃         ⢀⣴⣾⠿⠋⠉    ⠉⠙⠿⣷⣦⡀          ██    ██████             ███   ████        ┃
┃        ⢠⣾⡟⠁          ⠈⢻⣷⡄        ███    ██  ██    ██      ████  ██  ██       ┃
┃       ⢀⣿⡏              ⢹⣿⡀        ██        ██    ██     ██ ██      ██       ┃
┃       ⢸⣿      29%       ⣿⡇        ██       ██           ██  ██    ███        ┃
┃       ⢸⣿     Work       ⣿⡇        ██      ██            ███████  ██          ┃
┃       ⠈⣿⣇              ⣸⠉⠁        ██      ██      ██        ██  ██  ██       ┃
┃        ⠘⢿⣧⡀          ⢀⣼⡿⠃       ██████    ██      ██       ████ ██████       ┃
┃         ⠈⠻⢿⣶⣄⣀    ⣀⣠⣶⡿⠟⠁                                                     ┃
┃            ⠉⠛⠻⠿⠿⠿⠿⠟⠛⠉                                                        ┃
┃                                                                              ┃
┃  █████████████████████▌                                                      ┃
┃                                07:18 of 25:00                                ┃
┃                                                                              ┃
┃  ████████████ ███ ███▊                                                       ┃
┃                                                                              ┃
┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
                     space  start/pause   s  skip   q  quit
```

## How it works

- **The cycle:** a work phase is followed by a short break. After the fourth work
  phase comes a long break instead, and the cycle starts again at round 1.
  With the defaults that's 25 minutes of work, 5-minute short breaks and a 15-minute
  long break.
- **The first phase** starts running as soon as the program opens.
- **When a phase ends,** the terminal bell rings once and a popup names the next
  phase over the dimmed screen. That phase is loaded at full length and waits until
  you press space. Nothing moves on while you are away.
- **The time left** is shown in large block digits as minutes and seconds, rounded
  up, so it reads `00:00` only when the phase is over. A dial of braille dots and a
  bar that fills in eighths of a cell show how much of the phase has passed.
- **Motion:** the screen assembles itself when the program opens, a phase that
  starts coalesces into view, and the popup drops in and its amber frame pulses
  slowly, about once every two seconds, until you press a key. `--motion off`
  turns every animation off.
- **Outside the window:** the tab and window title show the phase and the time
  left, or which phase is ready. In Windows Terminal, VS Code, kitty and GNOME
  Terminal (VTE 0.80 or later), the taskbar button and tab also show progress:
  normal while running, paused while paused, and animated while a phase waits.
  VS Code shows these only if its tab title settings include `${sequence}` and
  `${progress}`. On exit the progress is cleared, and the title comes back where
  the terminal keeps a title stack; elsewhere, such as Windows Terminal, it reads
  `pomodoro`.
- **Timing** comes from the system's monotonic clock, not from counting screen
  updates. The screen redraws at least four times a second, and a slow redraw
  never changes the length of a phase.

## Requirements

- Windows or Linux. macOS is not tested.
- An interactive terminal: Windows Terminal on Windows, or any terminal emulator
  on Linux. The full layout, with the dial, needs 80 columns by 24 rows; from 48 by
  16 the digits get smaller and the dial goes; below that the time is plain text. A
  tiny window still works, with parts cut off.
- To build it: [rustup](https://rustup.rs). The repository pins Rust 1.98.1 in
  `rust-toolchain.toml`, and rustup installs that version the first time you run
  `cargo` here.

## Install

From the repository root, either install the binary into Cargo's bin directory
(usually `~/.cargo/bin`, which rustup puts on your `PATH`):

```sh
cargo install --path . --locked
```

or build it in place and run `target/release/pomodoro` (`pomodoro.exe` on
Windows):

```sh
cargo build --release --locked
```

To try it without installing, use `cargo run --release --`, followed by any
options.

## Running it

```sh
pomodoro                      # 25 / 5 / 15 minutes, long break every 4th round
pomodoro --work 50 --short 10 # longer work phases and short breaks
pomodoro --every 2            # a long break after every second work phase
pomodoro --work 1 --short 1   # a quick run to see the cycle and hear the bell
pomodoro --help               # the options, their ranges and defaults, and the keys
```

### Options

| Option | Sets | Values | Default |
| --- | --- | --- | --- |
| `--work MIN` | the length of a work phase, in minutes | 1 to 99 | 25 |
| `--short MIN` | the length of a short break, in minutes | 1 to 99 | 5 |
| `--long MIN` | the length of a long break, in minutes | 1 to 99 | 15 |
| `--every N` | how many work phases come before a long break | 1 to 99 | 4 |
| `--color WHEN` | the colours to use | `auto`, `truecolor`, `256`, `16`, `none` | `auto` |
| `--glyphs SET` | the pictures beside the text | `auto`, `emoji`, `symbols`, `ascii` | `auto` |
| `--motion ON` | whether the screen animates | `on`, `off` | `on` |
| `-h`, `--help` | print the usage and exit | | |

Each option's value is the next argument (`--work 50`, not `--work=50`), and each
option can appear once. `--help` wins wherever it appears.

With `auto`, pomodoro picks the colours and pictures from the terminal it runs in.
It uses 24-bit colour in Windows Terminal, VS Code and any terminal that sets
`COLORTERM=truecolor`, and falls back to 256 or 16 colours elsewhere. Setting
`NO_COLOR` to any non-empty value turns colour off, and `--color` overrides it.
Emoji are used everywhere except the old Windows console, which gets symbols, and
the Linux console, which gets plain ASCII.

### Keys

| Key | Does |
| --- | --- |
| Space | Starts a phase that is Ready, pauses one that is Running, resumes one that is Paused |
| `s` | Skips to the next phase, loaded as Ready; no bell. A skipped work phase still counts towards the long break |
| `q`, Esc or Ctrl-C | Quits and restores the terminal |

Keys work with or without Shift. Any other key does nothing.

### The screen

- **The header** shows a tomato for each work phase done in this cycle, a circle
  for each still to come, and the round (a break shows the round it follows).
- **The panel** is framed in the phase's colour: tomato red with a thick border for
  work, mint with a rounded border for a short break, lavender with a double border
  for a long break. Its title gives the phase and its state (`Running`, `Paused` or
  `Ready`).
- **Inside**, from the top: a Paused badge when paused, the dial and the large
  digits (dimmed while paused), the bar with the time passed out of the phase
  length, and the ribbon: one segment per phase of the cycle, as wide as the phase
  is long, filled as the cycle goes on.
- **The keys** are along the bottom.

Quitting ends the run. Nothing is saved, and the next run starts again at round 1.

## Exit status and messages

| Status | Meaning |
| --- | --- |
| 0 | You quit, or `--help` printed the usage |
| 1 | The terminal could not be used, or the usage could not be printed; the message on stderr says why |
| 2 | An option was wrong; nothing was started |

Every message starts with `pomodoro:` and fits on one line. For an option error,
the message names the first wrong argument in the order given, for example:

```text
pomodoro: invalid value for --work: "0" is not a whole number from 1 to 99
pomodoro: --work needs a value
pomodoro: unrecognised argument "--work=50"
pomodoro: --work given more than once
```

A terminal failure names the step that failed, then the system's reason, for
example `pomodoro: could not draw the screen: ...`. The terminal is restored before
the message is printed.

## Troubleshooting

- **`an interactive terminal is required; standard output is not one`**: the
  output is redirected to a file or a pipe. Run `pomodoro` directly in a terminal
  window. `--help` works either way.
- **No bell**: pomodoro sends the terminal's bell once when a phase ends; whether
  you hear it is up to the terminal. The phase change still shows on screen as
  `Ready`.
  - VS Code's terminal is silent by default: its `accessibility.signals.terminalBell`
    setting plays a sound only when a screen reader is attached. Set it to
    `{ "sound": "on" }`, or turn on `terminal.integrated.enableVisualBell` for a
    bell icon instead.
  - Windows Terminal plays a sound by default (the profile's `bellStyle` is
    `"audible"`). If it's silent, check that the profile doesn't change `bellStyle`
    and that System sounds isn't muted in the Windows volume mixer. If it's too
    quiet, point the profile's `bellSound` at a louder audio file. `"all"` also
    flashes the taskbar.
  - To test a terminal on its own, run `[Console]::Write([char]7)` in PowerShell,
    or `printf '\a'` on Linux.
- **The terminal stops echoing after pomodoro was killed**: quitting with `q`,
  Esc or Ctrl-C always restores the terminal, but a process killed from outside
  (for example with `kill` on Linux) cannot. Type `reset` and press Enter, even if
  the letters don't appear.
- **The time looks wrong after the computer slept**: whether the clock counts time
  asleep depends on the platform. After waking, the phase may have ended or may
  carry on where it stopped.
- **Odd characters in place of the `·`, the bar or the emoji**: use a terminal with
  UTF-8 and a font that has box-drawing characters, such as Windows Terminal. If
  emoji look misaligned, `--glyphs symbols` swaps them for one-cell symbols.
- **Animations are distracting or slow**: `--motion off` draws every screen still.
- **Colours look wrong**: `--color 256` or `--color 16` forces a smaller palette,
  and `--color none` uses the terminal's own colours.

## Development

[AGENTS.md](AGENTS.md) has the checks to run before a change is done. It is built
on the `rust-quality-baseline` skill, with `cargo xtask check` as the main one.
The intended behaviour, and the test that covers each rule, are in
[docs/plan/](docs/plan/).
