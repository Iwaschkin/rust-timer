# Visual overhaul: research findings

Reader: the agent that will design and build the visual overhaul of `pomodoro` (the phase-end alert, the palette, emoji and effects).

Researched 2026-09-25 against the project's resolved graph: ratatui 0.30.2, ratatui-core 0.1.2, ratatui-widgets 0.3.2, ratatui-crossterm 0.1.2, crossterm 0.29.0, unicode-width 0.2.2 (`E:\DEV\rust-timer\Cargo.lock`). Toolchain 1.98.1, cargo-deny 0.20.2.

Citation shorthand:

- `REG/` is `C:\Users\david\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\`, the unpacked crate sources. Each cite names crate, version, file and line.
- `PROBE/` is the throwaway probe workspace `C:\Users\david\AppData\Local\Temp\claude\e--DEV-rust-skills\df5e0b76-7550-4a54-9173-9e6cfce90e9c\scratchpad\research-probe\`. `scratchpad/` is its parent, which holds the awk scripts used for the colour maths. Both may be deleted, so their results are copied here. "Probe" means it was compiled and run locally on Windows 11 with Rust 1.98.1. It was not run in a real terminal emulator.
- "Unverified" marks anything I could not confirm from a primary source or a probe.

## 1. ratatui 0.30.2 rendering capabilities

### 1.1 What our feature line actually enables

The manifest line is `ratatui = { version = "0.30.2", default-features = false, features = ["crossterm"] }`. Here is what it turns on after Cargo's feature unification:

| Feature | State | Evidence |
| --- | --- | --- |
| `crossterm` (backend, `ratatui::crossterm` re-export) | on | `REG/ratatui-0.30.2/Cargo.toml` `[features] crossterm = ["dep:ratatui-crossterm", "std"]` |
| `underline-color` (`Style::underline_color`, SGR 58) | **on anyway** | ratatui depends on ratatui-crossterm without `default-features = false`. ratatui-crossterm's defaults are `["crossterm_0_29", "underline-color"]` (`REG/ratatui-crossterm-0.1.2/Cargo.toml`). The probe `cargo tree -e features -i ratatui-core` on the baseline shows `"default" "std" "underline-color"` (`PROBE/per-crate/baseline`). |
| `layout-cache` | off | `Layout::init_cache` is `#[cfg(feature = "layout-cache")]` (`REG/ratatui-core-0.1.2/src/layout/layout.rs:310`). Without it, `split` solves every call (`layout.rs:739-746`). |
| `widget-calendar` (`widgets::calendar`) | off | `REG/ratatui-0.30.2/src/widgets.rs:673-674` |
| `macros` (`ratatui::macros`) | off | `REG/ratatui-0.30.2/src/lib.rs:484-485` |
| `palette` (the `palette` crate: `Color::from_hsl`, `Color::from_hsluv`, `ratatui::palette`) | off | `REG/ratatui-core-0.1.2/src/style/color.rs:415-416` (`#[cfg(feature = "palette")]`); `REG/ratatui-0.30.2/src/lib.rs:477-478` |
| `unstable-widget-ref` (`WidgetRef`, `StatefulWidgetRef`) | off | `REG/ratatui-0.30.2/src/widgets.rs:690-691` |
| `unstable-rendered-line-info` (`Paragraph::line_count`, `line_width`) | off | `REG/ratatui-widgets-0.3.2/src/paragraph.rs:328-331` |
| `serde`, `scrolling-regions`, `termion`/`termwiz`/`termina` | off | `REG/ratatui-0.30.2/Cargo.toml` |

Only the calendar widget needs a feature we lack. All other widgets are re-exported unconditionally: BarChart, Block, Canvas, Chart, Clear, Fill, Gauge, LineGauge, List, RatatuiLogo, RatatuiMascot, Paragraph, Scrollbar, Sparkline, Table and Tabs (`REG/ratatui-0.30.2/src/widgets.rs:667-689`).

### 1.2 Colours

- `Color` variants (`REG/ratatui-core-0.1.2/src/style/color.rs:69-127`):
  - `Reset`;
  - the 16 named ANSI colours: Black, Red, Green, Yellow, Blue, Magenta, Cyan, Gray, DarkGray, LightRed, LightGreen, LightYellow, LightBlue, LightMagenta, LightCyan and White;
  - `Rgb(u8, u8, u8)`;
  - `Indexed(u8)`.
- The `Rgb` docs warn that crossterm has no truecolor fallback, so on a non-truecolor terminal "the display will be unpredictable" (`color.rs:113-122`).
- `Color::from_u32(0x00RRGGBB)` is a `const fn` (`color.rs:133`), so the whole palette can be `const`.
- `FromStr` accepts names, `"0".."255"` (as `Indexed`) and `"#rrggbb"` (`color.rs:286-343`).
- Wire format (probe `PROBE/nocolor`):
  - named colours are emitted as `38;5;n`, for example `Color::Red` gives `ESC[38;5;1m`, not SGR 31;
  - `Rgb` is emitted as `38;2;r;g;b`;
  - `Indexed` is emitted as `38;5;n`.

### 1.3 Modifiers and `Stylize`

- `Modifier` bitflags (`REG/ratatui-core-0.1.2/src/style.rs:104-114`): BOLD, DIM, ITALIC, UNDERLINED, SLOW_BLINK, RAPID_BLINK, REVERSED, HIDDEN and CROSSED_OUT.
- `Style::underline_color` is `#[cfg(feature = "underline-color")]` (`style.rs:385-390`). It is available, see 1.1.
- `Stylize` (`REG/ratatui-core-0.1.2/src/style/stylize.rs:233-270`) provides:
  - `fg`/`bg`/`reset`/`add_modifier`/`remove_modifier`;
  - one method per named colour and its `on_*` background form, for example `red()` and `on_red()`;
  - `bold()`/`not_bold()` and the same pair for every modifier.
- `Stylize` is a blanket impl over every `Styled` type (`stylize.rs:273`), including `&str`, `String`, `Span`, `Line`, `Text`, `Block`, `Paragraph` and `Gauge`. Primitives get it through `styled!` (`stylize.rs:354-369`).

### 1.4 `Block`

Source: `REG/ratatui-widgets-0.3.2/src/block.rs`.

- Titles:
  - `title`, `title_top` and `title_bottom` take `Into<Line>` (`block.rs:366, 397, 426`);
  - alignment is per `Line`, via `.left_aligned()`, `.centered()` or `.right_aligned()`, and a block can hold several titles on each edge (`block.rs:380-395`);
  - `title_alignment` sets the default alignment and `title_position` the default edge (`block.rs:471, 494`);
  - `title_style` applies after `style` and `border_style` (`block.rs:436-450`).
- Borders:
  - `borders(Borders)`, `border_type(BorderType)`, `border_set(border::Set)` for a custom set, and `border_style` (`block.rs:520-626`);
  - `BorderType` variants: Plain, Rounded, Double, Thick, LightDoubleDashed, HeavyDoubleDashed, LightTripleDashed, HeavyTripleDashed, LightQuadrupleDashed, HeavyQuadrupleDashed, QuadrantInside and QuadrantOutside (`REG/ratatui-widgets-0.3.2/src/borders.rs:36-150`);
  - extra sets in `ratatui::symbols::border` include `ONE_EIGHTH_WIDE`, `ONE_EIGHTH_TALL`, `PROPORTIONAL_WIDE`, `PROPORTIONAL_TALL`, `FULL` and `EMPTY` (`REG/ratatui-core-0.1.2/src/symbols/border.rs:239-355`).
- Padding: `Padding::new`, `zero`, `horizontal`, `vertical`, `uniform`, `proportional`, `symmetric`, `left`, `right`, `top` and `bottom` (`REG/ratatui-widgets-0.3.2/src/block/padding.rs:50-152`).
- Border merging: `merge_borders(MergeStrategy::{Replace, Exact, Fuzzy})` collapses adjacent borders into junctions (`block.rs:707`; `REG/ratatui-core-0.1.2/src/symbols/merge.rs:62-271`).
- Shadows, new in this line (`block.rs:731`; `REG/ratatui-widgets-0.3.2/src/block/shadow.rs`):
  - `Block::shadow(Shadow)`;
  - presets `Shadow::overlay()` (style only), `block()` (█), `light_shade()`, `medium_shade()` and `dark_shade()` (░▒▓);
  - `Shadow::custom(impl CellEffect)`, `.style(..)`, and `.offset(Offset)` with a default of (1, 1);
  - the ready-made `dimmed()` `CellEffect` adds DIM and halves an RGB background, or sets `Black` on a non-RGB one (`shadow.rs:320-344`).
- A popup with a real drop shadow is a single builder chain: `Block::bordered().title(..).shadow(Shadow::dark_shade().offset(Offset::new(2, 1)))`.

### 1.5 Gauge and LineGauge

Source: `REG/ratatui-widgets-0.3.2/src/gauge.rs`.

| | `Gauge` | `LineGauge` |
| --- | --- | --- |
| Height used | the whole area (fills every row) | one row |
| Value | `ratio(f64)` or `percent(u16)`. **Both `assert!` and panic** outside 0..=1 and 0..=100 (`gauge.rs:75-79, 97-101`). | `ratio(f64)` (`gauge.rs:312`) |
| Fill style | `gauge_style`: fg = filled colour, bg = unfilled colour (`gauge.rs:137, 195-205`) | `filled_style`, `unfilled_style` (`gauge.rs:402, 412`). `gauge_style` is deprecated since 0.27.0 (`gauge.rs:385`). |
| Symbols | full block. `use_unicode(true)` adds 1/8-cell partial blocks at the edge (`gauge.rs:148, 187-212`). | `filled_symbol`, `unfilled_symbol`, any `&str`, for example "━"/"─" or "▰"/"▱" (`gauge.rs:341-351`) |
| Label | centred `Span`. Defaults to `NN%`. In filled cells it is drawn with fg/bg swapped (`gauge.rs:180-206`). | left-aligned `Line`, then the bar. Defaults to `NNN%` (`gauge.rs:432-441`). |
| Widget style | `style` (area and block), plus an optional `block` | same |

- Design consequence: the Gauge label's text colour over the filled part is the unfilled colour. Pick filled and unfilled colours that also contrast as text; see section 6.
- Clamp every ratio before calling `ratio`, because the widget panics outside its range.

### 1.6 Sparkline, BarChart, Chart

- `Sparkline` (`REG/ratatui-widgets-0.3.2/src/sparkline.rs`):
  - `data`, `max`, `bar_set` (`THREE_LEVELS`/`NINE_LEVELS` or a custom set) and `direction(RenderDirection::{LeftToRight, RightToLeft})`;
  - `absent_value_style` and `absent_value_symbol`;
  - per-bar styling through `SparklineBar` (`sparkline.rs:66-280`).
  - Useful for a "today's sessions" history strip.
- `BarChart` (`REG/ratatui-widgets-0.3.2/src/barchart.rs`, `barchart/bar.rs`, `barchart/bar_group.rs`):
  - `BarChart::vertical`/`horizontal`/`grouped`;
  - `bar_width`, `bar_gap`, `group_gap`, `bar_set`, `bar_style`, `value_style`, `label_style` and `max`;
  - per-`Bar` `style`, `value_style`, `text_value` and `label`.
- `Chart` (`REG/ratatui-widgets-0.3.2/src/chart.rs`):
  - `Dataset` with `marker`, `graph_type(GraphType::{Scatter, Line, Bar, Area})` and `fill_to_y` for Area;
  - `Axis` with `bounds`, `labels`, `labels_alignment`, `title` and `style`;
  - `legend_position(Option<LegendPosition>)` (`chart.rs:158-199, 325-440, 530-745`).

### 1.7 Canvas

- Markers: `symbols::Marker` is `#[non_exhaustive]` and has Dot, Block, Bar, Braille, HalfBlock, Quadrant, Sextant, Octant and `Custom(char)` (`REG/ratatui-core-0.1.2/src/symbols/marker.rs`). All five sub-cell markers exist in this version.
- Grid resolution per marker (`REG/ratatui-widgets-0.3.2/src/canvas.rs:583-591`):
  - Braille: 2×4 dots, `PatternGrid<2,4>`;
  - Quadrant: 2×2;
  - Sextant: 2×3;
  - Octant: 2×4;
  - HalfBlock: 1×2, with its own grid that keeps an fg and a bg per cell.
  - Pattern grids hold one fg per cell (`canvas.rs:806-816`).
  - The default marker is Braille (`canvas.rs:745`).
- Glyph support:
  - Sextants (U+1FB00 block) and octants (U+1CD00 block, the Legacy Computing Supplement) need font support (`marker.rs` doc comments).
  - VS Code draws U+1FB00–U+1FBFF itself (`terminal.integrated.customGlyphs`, GPU renderer only). U+1CDxx octants are not in that list (<https://raw.githubusercontent.com/microsoft/vscode/main/src/vs/workbench/contrib/terminal/common/terminalConfiguration.ts>, `CustomGlyphs` setting).
  - Octant rendering in Windows Terminal or Linux fonts is unverified.
- Shapes:
  - `Circle` is an outline sampled at 360 points (`REG/ratatui-widgets-0.3.2/src/canvas/circle.rs:32-43`);
  - `Line`, `FilledLine`, `Rectangle`, `Points`, and `Map` (`MapResolution::{Low, High}`) (`canvas.rs:32-36`).
- `Context` provides:
  - `draw`, `layer()`, `print(x, y, line)` for text labels that always sit on top, and `marker()` (`canvas.rs:598-640`);
  - `Canvas::background_color` (`canvas.rs:800`).
- A Braille or HalfBlock "tomato ring" progress dial is possible with `Circle` plus `Points` computed in `f64`.

### 1.8 Layout

- `Flex` values (`REG/ratatui-core-0.1.2/src/layout/flex.rs:26-209`): Legacy, Start, End, Center, SpaceBetween, SpaceEvenly and SpaceAround.
- `Layout::spacing(impl Into<Spacing>)` takes `Spacing::Space(u16)` or `Spacing::Overlap(u16)` (`layout.rs:89-108, 546`).
- Constraints: `Min`, `Max`, `Length`, `Percentage`, `Ratio` and `Fill` (`REG/ratatui-core-0.1.2/src/layout/constraint.rs`).
- Centring helpers: `Rect::centered(h, v)`, `centered_horizontally` and `centered_vertically` (`REG/ratatui-core-0.1.2/src/layout/rect.rs:513-560`).
- `Layout::areas::<N>` and `Rect::layout::<N>` panic when N differs from the constraint count (`layout.rs:562-586`; `rect.rs:588-596`). Destructuring an N-element constraint array keeps them in step.

### 1.9 `Clear`, `Fill` and direct `Buffer` access

- `Clear`:
  - resets every cell in the area (`REG/ratatui-widgets-0.3.2/src/clear.rs`), so the area falls back to `Color::Reset`;
  - in a painted-background app, follow `Clear` with a styled `Block` or a `Fill`.
- `Fill::new(symbol).style(..)` fills an area with one symbol and style (`REG/ratatui-widgets-0.3.2/src/fill.rs:9-90`). It is new in this widget set.
- Direct buffer access:
  - `Frame::buffer_mut()` (`REG/ratatui-core-0.1.2/src/terminal/frame.rs:207`) and `Frame::count()`, a frame counter (`frame.rs:235`);
  - `Buffer::content` is a public `Vec<Cell>` (`REG/ratatui-core-0.1.2/src/buffer/buffer.rs:67-73`), so a whole-screen post-process pass can be `for cell in &mut buf.content { .. }`, with no indexing;
  - `Buffer::cell_mut(Position)` returns `Option<&mut Cell>` (`buffer.rs:210`), while `buf[(x, y)]` and the deprecated `get`/`get_mut` panic out of range (`buffer.rs:120-160, 511-560`);
  - `Cell` has public `fg`, `bg`, `underline_color`, `modifier` and `diff_option` (`REG/ratatui-core-0.1.2/src/buffer/cell.rs:49-62`);
  - `CellDiffOption::{None, Skip, AlwaysUpdate, ForcedWidth(NonZeroU16)}` (`cell.rs`) can force a cell's width.

### 1.10 Built-in palettes

- `ratatui::style::palette` is compiled unconditionally: `pub mod palette;` at `REG/ratatui-core-0.1.2/src/style.rs:81` has no `cfg`. It is re-exported as `ratatui::style` (`REG/ratatui-0.30.2/src/lib.rs:517`).
  - `palette::tailwind` has 22 palettes (SLATE … ROSE). Each is a `Palette` with `c50`–`c950` as `Color::Rgb`, plus `BLACK` and `WHITE` (`REG/ratatui-core-0.1.2/src/style/palette/tailwind.rs:281-641`).
  - `palette::material` has 16 `AccentedPalette`s (`c50`–`c900`, `a100`/`a200`/`a400`/`a700`), three `NonAccentedPalette`s (BROWN, GRAY, BLUE_GRAY), plus `BLACK`/`WHITE` (`REG/ratatui-core-0.1.2/src/style/palette/material.rs:423-456`).
  - All of this is available with our feature set.
- The `palette` *feature* is unrelated. It pulls in the `palette` crate for HSL/HSLuv construction and is off.

## 2. Crates compatible with our exact graph

### 2.1 Method and results

- Each candidate got its own Cargo project (`PROBE/per-crate/<name>`). It contained our exact ratatui line plus the candidate, `edition = "2024"` and the 1.98.1 toolchain pin.
- The script `PROBE/per-crate/run.sh` ran `cargo generate-lockfile`, `cargo tree -d -e normal,build`, `cargo build --locked` and `cargo deny --all-features --locked check advisories`, using a copy of `E:\DEV\rust-timer\deny.toml` (`unmaintained = "all"`, `yanked = "deny"`).
- The advisory DB was RustSec advisory-db at commit `593df8c1` (2026-09-24).
- "Compiled crates" comes from `cargo tree -e normal,build`. The lockfile also lists optional dependencies that are never compiled, such as `time` and `palette` in the baseline.
- The baseline (ratatui alone) resolves 91 lock packages, the same count as the project's `Cargo.lock`. It compiles 59 crates. Its duplicates are hashbrown 0.16.1/0.17.1 and syn 2/3.

| Crate (latest on crates.io) | Depends on | ratatui-core copies | Compiled crates added | New duplicates | Build | Advisories |
| --- | --- | --- | --- | --- | --- | --- |
| tachyonfx 0.25.2, `default-features = false, features = ["std", "std-duration"]` | `ratatui-core = "0.1.2"`, no default features (`REG/tachyonfx-0.25.2/Cargo.toml`) | 1 | tachyonfx, bon, bon-macros, prettyplease, micromath, compact_str 0.10.0, zmij | compact_str 0.9.1 (ratatui-core) + 0.10.0 (tachyonfx) | ok, 0 warnings | ok |
| tachyonfx 0.25.2, default features (`std`, `dsl`) | same | 1 | as above plus anpa (DSL parser) | same | ok | ok |
| tui-big-text 0.8.10 | `ratatui-core = "0.1"`, `ratatui-widgets = "0.3"` with **default features** (`REG/tui-big-text-0.8.10/Cargo.toml`) | 1 | tui-big-text, font8x8, derive_builder (+core, +macro), darling 0.20, fnv, itertools 0.15, **time 0.3.55, time-core, deranged, powerfmt, num-conv** | itertools 0.14/0.15; darling 0.20/0.24 (proc-macro only) | ok | ok |
| tui-popup 0.7.7 | `ratatui-core = "0.1"`, `ratatui-widgets = "0.3"` (defaults), optional crossterm 0.29 | 1 | tui-popup, derive-getters, derive_setters, darling 0.21, **time and its four deps** | darling 0.21/0.24 (proc-macro) | ok | ok |
| tui-bar-graph 0.3.6 | `ratatui-core = "0.1"`, colorgrad 0.8 | 1 | tui-bar-graph, colorgrad, csscolorparser, num-traits, libm, autocfg | none | ok | ok |
| tui-box-text 0.3.5 | `ratatui-core = "0.1"`, plus **color-eyre as a normal dependency** | 1 | 19 crates, including color-eyre, eyre, backtrace, tracing, tracing-subscriber, sharded-slab | none | ok | ok |
| throbber-widgets-tui 0.11.1 | whole `ratatui = "0.30"`, default features off | 1 | throbber-widgets-tui only | none | ok | ok |
| **Negative control:** tui-rain 1.0.1 | `ratatui = "0.29.0"` | a second ratatui (0.29.0) | also forces unicode-width down to 0.2.0 for the whole graph ("Adding unicode-width v0.2.0 (available: v0.2.2)") | ratatui 0.29/0.30, unicode-width 0.1/0.2 | not built | **fails**: `error[unmaintained]` RUSTSEC-2024-0436 `paste` via ratatui 0.29 |

- All candidates were also built together in one project with a demo that renders them all into a `TestBackend`, including tachyonfx effects over them. The combined graph also has exactly one `ratatui-core v0.1.2` (`cargo tree -i ratatui-core`), builds with 0 warnings, and 133 crates pass `cargo deny` advisories (`PROBE/src/main.rs`, `PROBE/Cargo.toml`).
- No crate produced an unmaintained or yanked finding except the negative control.
- Crates.io latest versions were checked through `https://crates.io/api/v1/crates/<name>` on 2026-09-25:
  - tachyonfx 0.25.2 (updated 2026-09-06), tui-big-text 0.8.10, tui-popup 0.7.7, tui-bar-graph 0.3.6, tui-box-text 0.3.5 and tui-widgets 0.7.12 (all 2026-09-24);
  - throbber-widgets-tui 0.11.1 (2026-06-19) and tui-rain 1.0.1 (2024-11-29);
  - ratatui 0.30.2 and ratatui-core 0.1.2 are still the newest (2026-06-19).

Side effects to record in the dependency review:

1. **tui-big-text and tui-popup turn on ratatui-widgets' `calendar` feature.** That compiles `time 0.3.55` and four helper crates. It does not expose `ratatui::widgets::calendar`, which stays gated by ratatui's own `widget-calendar` (probe feature trees in `PROBE/per-crate/tui-big-text`; cause in `REG/ratatui-widgets-0.3.2/Cargo.toml`: `default = ["all-widgets"]`, `all-widgets = ["calendar"]`, `calendar = ["dep:time"]`). This is harmless to behaviour but adds compile time and review surface.
2. tachyonfx needs `compact_str 0.10` while ratatui-core uses 0.9, so both are compiled.
3. tachyonfx contains `unsafe` (a lifetime `transmute` in `REG/tachyonfx-0.25.2/src/cell_iter.rs:181`). The project's `unsafe_code = "forbid"` covers only its own crate, but this belongs in the dependency review.
4. tui-big-text's renderer calls `.unwrap()` on the first char of a grapheme, which is never empty (`REG/tui-big-text-0.8.10/src/big_text.rs:210`). Our lints do not reach dependencies.

### 2.2 tachyonfx 0.25.2 API

tachyonfx is a post-processor. You render widgets first, then effects mutate the rendered cells in the buffer (`REG/tachyonfx-0.25.2/src/fx/mod.rs:1-7`).

Feature flags (`REG/tachyonfx-0.25.2/Cargo.toml`; `README.md:295-303`):

| Flag | Default | Meaning |
| --- | --- | --- |
| `std` | yes | std support |
| `dsl` | yes | string DSL for effects (pulls `anpa`) |
| `std-duration` | no | `tachyonfx::Duration` becomes `std::time::Duration`. Without it, `Duration` is a custom `u32` milliseconds type (`src/duration.rs:5-11`). |
| `sendable` | no | effects become `Send` (`Arc<Mutex>` instead of `Rc<RefCell>`) (`src/features.rs`) |
| `ratatui-next-cell` | no | compatibility marker for `Cell::diff_option` (CHANGELOG 0.25.1) |
| `wasm`, `web-time` | no | WebAssembly |

- Use `std-duration`. Without it, `From<std::time::Duration>` truncates to whole milliseconds (`d.as_millis() as u32`, `src/duration.rs:150-156`), so a 16.67 ms frame counts as 16 ms and effects run about 4% slow.
- The probe `PROBE/min-tachyonfx` confirms that `let tick: tachyonfx::Duration = std::time::Duration::from_micros(16_667);` compiles with `std-duration`.

Core types:

- `Effect`:
  - `process(duration, &mut Buffer, area) -> Option<Duration>` returns the overflow time (`src/effect.rs:246-257`);
  - `done()`, `running()`, `reset()` and `reversed()`;
  - `with_area`, `with_filter(CellFilter)`, `with_pattern(P)`, `with_color_space(ColorSpace::{Rgb, Hsl, Hsv})` (default Hsl) and `with_rng` (`src/effect.rs`; `src/color_space.rs:7-15`).
- `EffectManager<K: Clone + Ord>` (`src/effect_manager.rs`):
  - `add_effect` and `add_unique_effect(key, effect)`, where a new effect with the same key cancels the running one;
  - `cancel_unique_effect(key)`;
  - `is_running()` says whether any effect is active;
  - `process_effects(duration, &mut Buffer, area)` runs every effect and drops finished ones (`effect_manager.rs:43-140`).
- `EffectRenderer` is implemented for `Frame` and `Buffer`: `frame.render_effect(&mut effect, area, elapsed)` (`src/render_effect.rs`).
- `EffectTimer` converts from `u32` ms, `(u32, Interpolation)`, `(Duration, Interpolation)` and `Duration` (`src/effect_timer.rs:242-265`).
- `Interpolation` has 34 variants:
  - Linear and Reverse;
  - In, Out and InOut forms of Back, Bounce, Circ, Cubic, Elastic, Expo, Quad, Quart, Quint and Sine;
  - SmoothStep and Spring (`src/interpolation.rs:240-317`).
- `Shader` is the trait for custom effects; `.into_effect()` wraps one (`src/shader.rs`). `fx::effect_fn(state, timer, closure)` is the quick way to write a custom effect (`fx/mod.rs:334`).

Effects the brief asked about (all in `REG/tachyonfx-0.25.2/src/fx/mod.rs`):

| Want | Constructor | Line |
| --- | --- | --- |
| Fade | `fade_to(fg, bg, timer)`, `fade_from`, `fade_to_fg`, `fade_from_fg` | 1903, 1928, 1585, 1611 |
| Dissolve and reform | `dissolve(timer)`, `dissolve_to(style, timer)`, `coalesce(timer)`, `coalesce_from` | 1476-1555 |
| Sweep | `sweep_in(Motion, gradient_len, randomness, colour, timer)`, `sweep_out`; `slide_in`, `slide_out` | 837, 741, 891, 943 |
| HSL shift | `hsl_shift(Option<[h,s,l]>, Option<[h,s,l]>, timer)`. **Panics if both are `None`.** Use `hsl_shift_fg([h, s, l], timer)`. | 445-485 |
| Glitch | `fx::Glitch::builder().cell_glitch_ratio(f32).action_start_delay_ms(Range<u32>).action_ms(Range<u32>).build().into_effect()` | `src/fx/glitch.rs:37-48` |
| Ping-pong, repeat | `ping_pong(e)`, `repeat(e, RepeatMode::{Forever, Times(n), Duration(d)})`, `repeating(e)` | 687, 658, 716; `fx/repeat.rs:146-153` |
| Composition | `sequence(&[..])`, `parallel(&[..])`, `delay`, `sleep`, `prolong_start`/`prolong_end`, `never_complete`, `with_duration`, `run_once`, `freeze_at`, `remap_alpha` | 1404-2204 |
| Colour | `paint`, `paint_fg`/`paint_bg`, `lighten`/`darken`, `saturate` | 1639-1878 |
| Geometry and text | `expand`, `stretch`, `translate`, `explode`, `evolve`/`evolve_into`/`evolve_from` | 1006-1313, 545 |

- Spatial patterns (`REG/tachyonfx-0.25.2/src/pattern/`):
  - `RadialPattern::center()`, `DiamondPattern::center()`, `SpiralPattern::center()` (all with `.with_transition_width(f32)`);
  - `SweepPattern::left_to_right(n)`/`right_to_left`/`up_to_down`/`down_to_up`, and `DiagonalPattern`;
  - `CheckerboardPattern`, `WavePattern`, `DissolvePattern`, `CoalescePattern`, `BlendPattern`, `CombinedPattern` and `InvertedPattern`.
- `CellFilter` restricts which cells an effect touches. Variants include `All`, `Area`, `Text`, `NonEmpty`, `Inner(Margin)`, `FgColor` and `AllOf`, with `.negated()` (`src/cell_filter/filter.rs`; `fx/mod.rs` module doc).
- **Effects convert every colour to RGB.**
  - `ToRgbComponents` maps named, `Indexed` and `Reset` colours to RGB. `Reset` becomes black (0,0,0) (`REG/tachyonfx-0.25.2/src/color_ext.rs`).
  - Probe: a 50% `fade_to_fg` from `Indexed(208)` to `Indexed(46)` produced `Rgb(188, 255, 0)` (`PROBE/src/main.rs` output).
  - So a 256- or 16-colour terminal will receive truecolor codes mid-effect unless the app quantizes afterwards (see 3.5).
  - A fade that starts from a `Reset` background fades from black, which is another reason to paint our own background.
- How to drive it: see section 7.

### 2.3 tui-big-text 0.8.10 API

Source: `REG/tui-big-text-0.8.10/src/big_text.rs`, `src/pixel_size.rs`.

- Build: `BigText::builder()` then `.lines(Vec<Line>)`, `.style(Style)`, `.pixel_size(PixelSize)`, `.alignment(Alignment)` or `.left_aligned()`/`.centered()`/`.right_aligned()`, `.block(Block)` and `.build()`.
  - `build()` returns `BigText` directly, not a `Result` (`big_text.rs:117-126`).
  - `BigText` is `#[non_exhaustive]` with public fields.
  - Per-span styles in each `Line` are honoured, so each digit can be coloured differently.
- Font: there is only one, the 8×8 bitmaps of the `font8x8` crate. Its BASIC, LATIN, HIRAGANA, GREEK, BLOCK, BOX, MISC and SGA sets are searched in order (`big_text.rs:195-204`). A character with no glyph renders blank, so emoji cannot be rendered big.
- `PixelSize` sets how many font pixels one cell shows (`pixel_size.rs:43-53`). The glyph size is `ceil(8/x) × ceil(8/y)` cells (`big_text.rs:158-160`). Sizes for the five-glyph string "25:00":

| PixelSize | Pixels per cell | Cells per glyph (w×h) | "25:00" needs |
| --- | --- | --- | --- |
| Full (default) | 1×1 | 8×8 | 40×8 |
| HalfHeight | 1×2 | 8×4 | 40×4 |
| HalfWidth | 2×1 | 4×8 | 20×8 |
| Quadrant | 2×2 | 4×4 | 20×4 |
| ThirdHeight | 1×3 | 8×3 | 40×3 |
| Sextant | 2×3 | 4×3 | 20×3 |
| QuarterHeight | 1×4 | 8×2 | 40×2 |
| Octant | 2×4 | 4×2 | 20×2 |

- The crate warns that ThirdHeight, Sextant, QuarterHeight and Octant "might look very strange" depending on the terminal (`pixel_size.rs:17-33`).
- Quadrant and HalfHeight use only Block Elements (U+2580–U+259F). Those are widely supported, and VS Code draws them itself (`CustomGlyphs` setting, section 1.7 source).
- The probe rendered "25:00" in Quadrant at 20×4 cells (`PROBE/src/main.rs` output).

### 2.4 Other showpiece crates on ratatui-core 0.1

All of these depend on `ratatui-core = "0.1"` (manifests under `REG/`) and are in the tui-widgets suite (<https://github.com/ratatui/tui-widgets>):

- tui-popup 0.7.7: a centred popup with a title.
- tui-bar-graph 0.3.6: gradient bar graphs using Braille, solid, quadrant or octant cells and colorgrad gradients.
- tui-box-text 0.3.5: box-drawing big letters. Its dependency cost is high (color-eyre and tracing).
- tui-cards 0.3.6: playing cards.
- tui-equalizer 0.2.4: an equalizer.
- tui-qrcode 0.2.7: QR codes.
- tui-scrollbar 0.2.8: fractional scrollbars.
- tui-scrollview 0.6.8: scroll view.
- tui-prompts 0.6.8: prompts. It has a hard dependency on crossterm 0.29, the same version as ours.

Outside the suite:

- throbber-widgets-tui 0.11.1: spinners, with sets such as `BRAILLE_SIX`, `CLOCK` and `QUADRANT_BLOCK` (`REG/throbber-widgets-tui-0.11.1/src/symbols.rs`). It adds only itself.
- ratatui-image 12.0.0-rc.0: a pre-release on `ratatui ^0.30.1`. It brings in the `image` crate and chafa by default. Not probed.
- Avoid anything pinned to ratatui 0.29 (tui-rain 1.0.1, shown above) and ratatui-splash-screen 0.1.5. The latter depends on `ratatui >= 0.25` with default features (`REG/ratatui-splash-screen-0.1.5/Cargo.toml`), which would switch on all of ratatui's default features in our graph. Not probed.

## 3. Terminal colour capability detection and fallback

### 3.1 How crossterm 0.29 decides

`crossterm::style::available_color_count()` (`REG/crossterm-0.29.0/src/style.rs:163-181`):

1. On Windows, if `supports_ansi()` is true, it returns `u16::MAX` (truecolor). `supports_ansi()` is true when `ENABLE_VIRTUAL_TERMINAL_PROCESSING` can be set on stdout, or when `TERM` is set and is not `dumb` (`REG/crossterm-0.29.0/src/ansi_support.rs`).
2. Otherwise it reads `COLORTERM`, and only if that is **unset** does it read `TERM`. A value containing `24bit` or `truecolor` means `u16::MAX`; a value containing `256` means 256; anything else means 8.
   - Gap: a `COLORTERM` value such as `yes` returns 8 without consulting `TERM`.
   - The doc itself says "This does not always provide a good result."
3. It ignores `NO_COLOR`. That is handled separately, below.

- In the probe on Windows 11 it returned 65535 (`PROBE/src/main.rs` output).
- Under WSL inside Windows Terminal the Linux build takes step 2. Windows Terminal does not set `COLORTERM` (3.2), so the answer depends on the distro's `TERM`, typically 256.

NO_COLOR in crossterm:

- `Colored::ansi_color_disabled()` is true when `NO_COLOR` is set and non-empty. The result is memoised, and `force_color_output(bool)` overrides it (`REG/crossterm-0.29.0/src/style/types/colored.rs:75-92`; `style.rs:183-193`).
- When colour is disabled, the colour part of each SGR is written as an empty string (`colored.rs:98-101`).
- **Pitfall, verified by probe:**
  - ratatui-crossterm queues the modifier change before `SetColors` (`REG/ratatui-crossterm-0.1.2/src/lib.rs:248-266`), and with colour disabled `SetColors` becomes `ESC[;m`, which resets all attributes.
  - The probe output (`PROBE/nocolor`) was:
    - colour on, BOLD red: `ESC[1m ESC[38;5;1;49m X`
    - colour off, BOLD red: `ESC[1m ESC[;m X` (**bold lost**)
    - colour off, BOLD with `Color::Reset`: `ESC[1m X` (bold kept)
  - So in no-colour mode the app must map every colour to `Color::Reset` itself. Then ratatui never emits `SetColors`, because the colours never change from `Reset` (`lib.rs:256`).

### 3.2 Environment variables set by the target terminals

| Terminal | Variables it sets | Source |
| --- | --- | --- |
| Windows Terminal | `WT_SESSION` (GUID), `WT_PROFILE_ID`, also forwarded into WSL through `WSLENV`. Does **not** set `COLORTERM`; issue #11057 "WT should set COLORTERM" is still open. | <https://raw.githubusercontent.com/microsoft/terminal/main/src/cascadia/TerminalConnection/ConptyConnection.cpp> lines 61-90; <https://github.com/microsoft/terminal/issues/11057> |
| VS Code integrated terminal | `TERM_PROGRAM=vscode`, `TERM_PROGRAM_VERSION`, `COLORTERM=truecolor` | <https://raw.githubusercontent.com/microsoft/vscode/main/src/vs/workbench/contrib/terminal/common/terminalEnvironment.ts> lines 62-71 |
| GNOME Terminal and other VTE terminals | `TERM` (VTE's terminfo name), `VTE_VERSION` (numeric, for example 8000 for 0.80), `COLORTERM=truecolor` | <https://raw.githubusercontent.com/GNOME/vte/master/src/spawn.cc> lines 252-281 |
| Konsole | default profile environment `TERM=xterm-256color`, `COLORTERM=truecolor`; `KONSOLE_VERSION` | <https://raw.githubusercontent.com/KDE/konsole/master/src/profile/Profile.cpp> line 71; `src/session/SessionManager.cpp` line 201 |
| kitty | `TERM` (default `xterm-kitty`), `COLORTERM=truecolor`, `KITTY_WINDOW_ID` | <https://raw.githubusercontent.com/kovidgoyal/kitty/master/kitty/child.py> lines 382-403 |
| Alacritty | `TERM=alacritty` if that terminfo exists, else `xterm-256color`; `COLORTERM=truecolor` | <https://raw.githubusercontent.com/alacritty/alacritty/master/alacritty_terminal/src/tty/mod.rs> lines 100-108 |
| WezTerm | `TERM` (configurable), `COLORTERM=truecolor`, `TERM_PROGRAM=WezTerm`, `TERM_PROGRAM_VERSION` | <https://raw.githubusercontent.com/wezterm/wezterm/main/config/src/config.rs> lines 1611-1622 |
| Legacy Windows console host | none of the above. No `WT_SESSION` and no `TERM_PROGRAM` is the heuristic. | absence, from the rows above |

### 3.3 NO_COLOR semantics

Source: <https://no-color.org/>.

- The rule: "Command-line software which adds ANSI color to its output by default should check for a `NO_COLOR` environment variable that, when present and not an empty string (regardless of its value), prevents the addition of ANSI color."
- FAQ 2: "User-level configuration files and per-instance command-line arguments should override the `NO_COLOR` environment variable." A `--color=always` flag may override it.
- FAQ 3: NO_COLOR does not disable bold, underline or italic. "This standard only signals the user's intention regarding adding ANSI color to text output."
- So keep modifiers in no-colour mode, and use them to replace what colour conveyed.

### 3.4 Truecolor support

| Terminal | 24-bit colour | Source |
| --- | --- | --- |
| Windows Terminal | yes | renders RGB SGR. crossterm treats any VT-capable Windows console as truecolor (3.1). The README links 24-bit colour for the console family: <https://raw.githubusercontent.com/microsoft/terminal/main/README.md>. |
| Legacy console host (conhost) | yes since Windows 10 Insider build 14931 | <https://devblogs.microsoft.com/commandline/24-bit-color-in-the-windows-console/>. **Conflict:** the Console VT reference still says the console "will choose the nearest appropriate color from the existing 16 color table" for extended colours (<https://learn.microsoft.com/windows/console/console-virtual-terminal-sequences>, "Extended Colors"). The page looks outdated, but this is unverified on current conhost. |
| VS Code (xterm.js) | yes; advertises `COLORTERM=truecolor` | `terminalEnvironment.ts` (3.2) |
| VTE, Konsole, kitty, Alacritty, WezTerm | each advertises `COLORTERM=truecolor` | table 3.2 |

- VS Code will **recolour low-contrast text**. `terminal.integrated.minimumContrastRatio` defaults to 4.5: "the foreground color of each cell will change to try meet the contrast ratio specified" (<https://raw.githubusercontent.com/microsoft/vscode/main/src/vs/workbench/contrib/terminal/common/terminalConfiguration.ts>, `MinimumContrastRatio`). Every text pair below 4.5:1 will look different in VS Code; section 6 keeps all text pairs above it.
- VS Code also defaults `drawBoldTextInBrightColors` to true, so bold named colours become their bright variants there (same file).

### 3.5 Recommended fallback: truecolor → 256 → 16 → none

Detection, in order, with the first match winning:

| Step | Condition | Tier | Why |
| --- | --- | --- | --- |
| 1 | CLI flag or config: `--color` set to `truecolor`, `256`, `16` or `never` | as given | NO_COLOR FAQ 2 |
| 2 | `NO_COLOR` set and non-empty | none | no-color.org |
| 3 | `TERM=dumb` | none | crossterm also treats `dumb` as no ANSI (`ansi_support.rs`) |
| 4 | `COLORTERM` contains `truecolor` or `24bit` | truecolor | convention crossterm uses (3.1) |
| 5 | `WT_SESSION` set | truecolor | WT does not set `COLORTERM`; this matters in WSL (3.2) |
| 6 | Windows build and `available_color_count() == u16::MAX` | truecolor | crossterm's Windows path (3.1) |
| 7 | `TERM` contains `256` | 256 | |
| 8 | anything else | 16 | |

Implementation that fits the lint policy and the effects library:

- Design everything as RGB constants. Render widgets, then tachyonfx effects.
- As the **last step of every frame**, run one quantize pass over `frame.buffer_mut().content`. It maps each `Color::Rgb` in `fg`, `bg` and `underline_color` to the active tier:
  - truecolor: unchanged;
  - 256: nearest xterm index 16–255. Use a precomputed table for palette colours and nearest-in-cube or grey for effect intermediates;
  - 16: the semantic map in 6.6;
  - none: `Color::Reset`, keeping modifiers.
- This is one exhaustive `match` on the project's own tier enum, with no indexing.
- Avoid indices 0–15 in the 256 tier, because users' themes redefine them.
- Indices 16–231 are the 6×6×6 cube with channel levels 0, 95, 135, 175, 215 and 255; 232–255 are greys `8 + 10·i` (<https://raw.githubusercontent.com/ThomasDickey/xterm-snapshots/master/256colres.pl>).
- tachyonfx's own `term256_colors()` downsampler is deprecated since 0.16.0 (`REG/tachyonfx-0.25.2/src/fx/mod.rs:490-497`), so the quantize pass is ours to own.

## 4. Emoji in terminals

### 4.1 How ratatui 0.30 measures cells

- Text is split into extended grapheme clusters by `unicode-segmentation`:
  - `Span::styled_graphemes` uses `.graphemes(true)` (`REG/ratatui-core-0.1.2/src/text/span.rs:306-313`);
  - so does `Buffer::set_stringn` (`buffer.rs:348-351`).
- Each grapheme's width is `CellWidth::cell_width` (`REG/ratatui-core-0.1.2/src/buffer/cell_width.rs`):
  - one byte: 1;
  - otherwise `UnicodeWidthStr::width` of the whole grapheme, plus 1 for each U+FF9E or U+FF9F;
  - it does not use `width_cjk`.
- A grapheme of width 2 occupies one cell. The following cells are reset (`buffer.rs:363-369`).
- `ratatui-core` requires `unicode-width >=0.2.0`. The lock resolves **0.2.2, built on Unicode 17.0.0** (`REG/unicode-width-0.2.2/src/tables.rs:165`).
- The string-level rules in unicode-width 0.2.2 (`REG/unicode-width-0.2.2/src/lib.rs` module docs) give width 2 to:
  - well-formed, fully-qualified emoji ZWJ sequences;
  - emoji modifier sequences;
  - emoji presentation sequences (base + U+FE0F).
- This follows UAX #11 revision 31 (Unicode 9.0). That revision made `Emoji_Presentation` characters East Asian Wide and said emoji-style variation sequences should be treated as Wide (<https://www.unicode.org/reports/tr11/tr11-31.html>, Modifications).
- The buffer diff has a special path for VS16 graphemes. It re-checks their trailing cell "to work around terminals that fail to clear the trailing cell of certain emoji presentation sequences" (`REG/ratatui-core-0.1.2/src/buffer/diff.rs:157-171`).
- Probe widths through ratatui's own `CellWidth` are in the table in 4.4 (`PROBE/src/main.rs` output).

### 4.2 Where terminals disagree

| Terminal | Width model | VS16 after a text-default base (✔️) | ZWJ or modifier sequences | Source |
| --- | --- | --- | --- | --- |
| unicode-width 0.2.2 (ratatui) | per grapheme, Unicode 17 | 2 | 2 | 4.1 |
| Windows Terminal (`compatibility.textMeasurement` default `graphemes`) | grapheme clusters, Unicode 16 tables, width capped at 2 | 2: "U+FE0F … turn[s] them from being ambiguous width (= narrow) into wide" | 2 | <https://raw.githubusercontent.com/microsoft/terminal/main/doc/cascadia/profiles.schema.json> lines 2430-2446; <https://raw.githubusercontent.com/microsoft/terminal/main/src/types/CodepointWidthDetector.cpp> lines 46, 850-858 |
| VS Code / xterm.js (`terminal.integrated.unicodeVersion`: `"6"` or `"11"`, default `"11"`) | per code point. U+FE0F is a zero-width combiner and does not widen. The v11 wide table ends at U+1FA95, so emoji from Unicode 13 on are 1 cell. | **1**, a mismatch | 🧘‍♀️ = 2+0+1+0 = **3**; 💪🏽 = 2+2 = **4**; both mismatches | `terminalConfiguration.ts` `UnicodeVersion`; <https://raw.githubusercontent.com/xtermjs/xterm.js/master/addons/addon-unicode11/src/UnicodeV11.ts> |
| VS Code with `unicodeVersion: "6"` | every non-CJK supplementary code point is 1; BMP emoji are 1 | 1 | mismatch | <https://raw.githubusercontent.com/xtermjs/xterm.js/master/src/common/input/UnicodeV6.ts> `wcwidth` |
| Alacritty | per code point via `unicode-width` 0.2 `char` widths. Zero-width chars attach to the previous cell. | **1** | **3 / 4** | <https://raw.githubusercontent.com/alacritty/alacritty/master/alacritty_terminal/src/term/mod.rs> lines 1062-1085 |
| WezTerm (`unicode_version` default 9) | VS16 is only respected at `unicode_version >= 14` | **1** by default | unverified | <https://wezterm.org/config/lua/config/unicode_version.html> |
| kitty | full grapheme segmentation (Unicode 16) since 0.42.0; VS16 makes emoji two cells | 2 | 2 (unverified in detail) | <https://raw.githubusercontent.com/kovidgoyal/kitty/master/docs/changelog.rst> (0.42.0, 0.13.0, 0.43.0 entries) |
| Konsole (master) | Unicode 15.0 width tables; U+FE0F widens a width-1 base to 2 | 2 | unverified | <https://raw.githubusercontent.com/KDE/konsole/master/src/Screen.cpp> lines 1161-1169; `src/characters/CharacterWidth.cpp` line 19 |
| GNOME Terminal / VTE | no VS16 widening found in a code search of the GNOME/vte mirror | **unverified**; assume 1 | unverified | code search, 2026-09-25 |
| Legacy console host | Microsoft says conhost could not gain "unicode text, and emoji" support because of backward compatibility | unusable | unusable | <https://raw.githubusercontent.com/microsoft/terminal/main/README.md> lines 212-217 |

Rendering in colour:

- Windows Terminal's Atlas renderer uses DirectWrite `TranslateColorGlyphRun`, meaning colour glyphs (code search: `src/renderer/atlas/BackendD3D.cpp`, `BackendD2D.cpp` in microsoft/terminal).
- WezTerm bundles Noto Color Emoji as a default fallback (<https://wezterm.org/config/fonts.html>).
- Alacritty has had coloured emoji on Linux/BSD since 0.4.1 (<https://raw.githubusercontent.com/alacritty/alacritty/master/CHANGELOG.md>).
- kitty renders colour emoji (changelog entries about VS16 rendering).
- VTE, Konsole and VS Code render with whatever colour emoji font the system provides. Colour emoji there is unverified and font-dependent.

Problem classes:

1. **Text-default emoji plus VS16** (✔️ ⏸️ ▶️ ⏱️ ⚠️ ❤️). ratatui says 2 cells, but xterm.js, Alacritty and default WezTerm say 1. Every following cell on that row shifts or leaves stale content.
2. **ZWJ and skin-tone sequences.** ratatui says 2, but per-code-point terminals say 3–4.
3. **Text-default emoji without VS16** (✔ ⏸ ⏱ ⚠). Everyone agrees on 1, but many terminals draw them as monochrome text glyphs, and ▶ is East Asian Ambiguous. Windows Terminal widens Ambiguous characters when `compatibility.ambiguousWidth` is `wide` (profiles.schema.json lines 2439-2446).
4. **Pre-Unicode-9 width tables.** They make every emoji 1 cell, because emoji only became East Asian Wide in Unicode 9.0 (UAX #11 r31). VS Code's `"6"` mode behaves this way.
5. **Post-Unicode-12 emoji.** They are narrow in VS Code, whose v11 table ends at U+1FA95.

### 4.3 Safe emoji

A safe emoji is a single code point that meets all of these:

- `Emoji_Presentation=Yes` and `East_Asian_Width=W`;
- no selector appended;
- present in xterm.js's v11 table.

Such emoji measure 2 in every width model above except VS Code's non-default `"6"` mode and the legacy console host.

### 4.4 Candidate table

Emoji_Presentation comes from `emoji-data.txt` 17.0, East_Asian_Width from `EastAsianWidth-17.0.0.txt` (<https://www.unicode.org/Public/17.0.0/ucd/emoji/emoji-data.txt>, <https://www.unicode.org/Public/17.0.0/ucd/EastAsianWidth.txt>). The width column is what ratatui computed in the probe (`CellWidth::cell_width` and `Span::width`, `PROBE/src/main.rs`). "v11" means inside xterm.js's v11 wide ranges.

| Glyph | Name | Code point(s) | Emoji_Presentation | EAW | ratatui width | Safe? |
| --- | --- | --- | --- | --- | --- | --- |
| 🍅 | tomato | U+1F345 | Yes | W | 2 | **yes** |
| ☕ | hot beverage | U+2615 | Yes | W | 2 | yes (BMP; monochrome glyph possible, unverified) |
| 🍵 | teacup without handle | U+1F375 | Yes | W | 2 | **yes** |
| 🔔 | bell | U+1F514 | Yes | W | 2 | **yes** |
| 🔕 | bell with slash | U+1F515 | Yes | W | 2 | **yes** |
| ✅ | check mark button | U+2705 | Yes | W | 2 | yes (BMP) |
| ✔ | heavy check mark | U+2714 | No | N | 1 | text symbol only |
| ✔️ | heavy check mark + VS16 | U+2714 U+FE0F | n/a | n/a | 2 | **no** (1 in xterm.js, Alacritty, default WezTerm) |
| ✓ | check mark | U+2713 | not emoji | N | 1 | safe text fallback |
| ✨ | sparkles | U+2728 | Yes | W | 2 | yes (BMP) |
| 🌙 | crescent moon | U+1F319 | Yes | W | 2 | **yes** |
| 😴 | sleeping face | U+1F634 | Yes | W | 2 | **yes** |
| 💤 | zzz | U+1F4A4 | Yes | W | 2 | **yes** |
| 🔥 | fire | U+1F525 | Yes | W | 2 | **yes** |
| ⌛ | hourglass done | U+231B | Yes | W | 2 | yes (BMP) |
| ⏳ | hourglass not done | U+23F3 | Yes | W | 2 | yes (BMP) |
| ⏰ | alarm clock | U+23F0 | Yes | W | 2 | yes (BMP) |
| ⏱ | stopwatch | U+23F1 | No | N | 1 | no as emoji |
| ⏱️ | stopwatch + VS16 | U+23F1 U+FE0F | n/a | n/a | 2 | **no** |
| ⏲️ | timer clock + VS16 | U+23F2 U+FE0F | n/a | n/a | 2 | **no** |
| ⏸ / ⏸️ | pause | U+23F8 (+FE0F) | No | N | 1 / 2 | **no** |
| ▶ / ▶️ | play | U+25B6 (+FE0F) | No | **A** | 1 / 2 | **no** |
| ⏹️ ⏭️ | stop, next | U+23F9, U+23ED + FE0F | No | N | 2 | **no** |
| 🎯 | direct hit | U+1F3AF | Yes | W | 2 | **yes** |
| 🧠 | brain | U+1F9E0 | Yes | W | 2 | **yes** (v11) |
| 🎉 | party popper | U+1F389 | Yes | W | 2 | **yes** |
| 🏆 | trophy | U+1F3C6 | Yes | W | 2 | **yes** |
| ⭐ | star | U+2B50 | Yes | W | 2 | yes (BMP) |
| 🌟 | glowing star | U+1F31F | Yes | W | 2 | **yes** |
| ⚡ | high voltage | U+26A1 | Yes | W | 2 | yes (BMP) |
| ⚠ / ⚠️ | warning | U+26A0 (+FE0F) | No | N | 1 / 2 | **no** |
| 🚨 | police light | U+1F6A8 | Yes | W | 2 | **yes** |
| 🌿 🌱 🍃 | herb, seedling, leaf | U+1F33F, U+1F331, U+1F343 | Yes | W | 2 | **yes** |
| 🛋️ | couch + VS16 | U+1F6CB U+FE0F | No (base) | N | 2 | **no** |
| 🧘 | person in lotus position | U+1F9D8 | Yes | W | 2 | **yes** |
| 🧘‍♀️ | woman in lotus position | U+1F9D8 U+200D U+2640 U+FE0F | ZWJ sequence | n/a | 2 | **no** (3 in xterm.js and Alacritty) |
| 💪🏽 | biceps + skin tone | U+1F4AA U+1F3FD | modifier sequence | n/a | 2 | **no** (4 in xterm.js and Alacritty) |
| 🔄 🔁 | arrows, repeat | U+1F504, U+1F501 | Yes | W | 2 | **yes** |
| 🔴 🟢 🟥 | coloured circles and square | U+1F534, U+1F7E2, U+1F7E5 | Yes | W | 2 | yes (Unicode 12; inside the v11 table) |
| 📊 | bar chart | U+1F4CA | Yes | W | 2 | **yes** |
| 🕐 | one o'clock | U+1F550 | Yes | W | 2 | **yes** |
| ☀️ | sun + VS16 | U+2600 U+FE0F | No (base) | N | 2 | **no** |
| ● • ◆ | geometric symbols | U+25CF, U+2022, U+25C6 | not emoji | A | 1 | text fallback. Ambiguous, so 2 cells under WT's `ambiguousWidth: wide`. |

### 4.5 Fallback strategy and detection

- Use three glyph tiers from one role table:
  - **Emoji**, from the safe set above;
  - **Symbols**, from BMP text glyphs such as ✓ ● ◆ ◷ █ and box drawing;
  - **ASCII**, for example `[W] [B] [L] * > ||`.
- Every role must also carry a text label. Glyphs never carry meaning alone; see 6.2.
- Pick the tier:
  1. from a flag (`--glyphs=emoji|symbols|ascii`) if given;
  2. otherwise `ascii` if `TERM` is `linux` or `dumb`;
  3. otherwise `symbols` on Windows when neither `WT_SESSION` nor `TERM_PROGRAM` is set (the legacy console host);
  4. otherwise `emoji`.
- No terminal query reports "can draw colour emoji". A width self-test is possible:
  - print a safe emoji at column 0 in the alternate screen, then read the cursor column;
  - on Unix, `crossterm::cursor::position()` sends `ESC[6n` and parses the reply (`REG/crossterm-0.29.0/src/cursor/sys/unix.rs:17-30`);
  - on Windows it reads the console screen buffer instead (`cursor/sys/windows.rs:38-45`). Whether that reflects Windows Terminal's grapheme widths is unverified.
  - The test proves width only, not that the glyph renders in colour. Treat it as optional.
- Add a unit test asserting `cell_width() == 2` for every glyph in the emoji tier, and 1 for the symbol and ASCII tiers. It catches an accidental VS16 or ZWJ.

## 5. Terminal integration escape sequences

### 5.1 Window and tab title

- `crossterm::terminal::SetTitle(t)` writes `ESC ] 0 ; t BEL`. Its Windows fallback is `SetConsoleTitle` (`REG/crossterm-0.29.0/src/terminal.rs:386-397`).
- xterm defines OSC 0 (icon name and window title), OSC 1 (icon name) and OSC 2 (window title) (<https://invisible-island.net/xterm/ctlseqs/ctlseqs.txt>, lines 2034-2036, XTerm patch 411).
- The Windows console accepts OSC 0 and OSC 2 titles under 255 characters (<https://learn.microsoft.com/windows/console/console-virtual-terminal-sequences>, "Window Title").
- Windows Terminal ignores application titles when the profile sets `suppressApplicationTitle: true` (<https://learn.microsoft.com/windows/terminal/customize-settings/profile-advanced>).
- VS Code's default tab title template is `${process}`. The title an app sets arrives as `${sequence}`, so a VS Code user sees our title only after adding `${sequence}` to `terminal.integrated.tabs.title` (`terminalConfiguration.ts`, `TerminalTitle` default and descriptor list). The UI behaviour is unverified.
- Restoring the title on exit:
  - xterm has a title stack, `CSI 22 ; 0 t` to save and `CSI 23 ; 0 t` to restore (ctlseqs lines 1691-1695);
  - no Windows Terminal support was found in code search, so treat it as unverified there;
  - push on entry and pop on exit, and accept that some terminals keep our last title.

### 5.2 OSC 9;4 progress

- Format (Windows Terminal ≥ 1.6): `ESC ] 9 ; 4 ; <state> ; <progress> BEL`. Shown as a tab progress ring and in the Windows taskbar.
  - States: 0 hide/clear; 1 normal with value; 2 error; 3 indeterminate, value ignored; 4 warning.
  - Progress is 0–100 (<https://learn.microsoft.com/windows/terminal/tutorials/progress-bar-sequences>).
  - The taskbar animation needs "Show animations in Windows" enabled (same page).
- ConEmu defines state 4 as "paused" and allows either terminator, ESC `\` or BEL (<https://conemu.github.io/en/AnsiEscapeCodes.html>, ConEmu-specific OSC).

| Terminal | OSC 9;4 | Source |
| --- | --- | --- |
| Windows Terminal | yes, tab ring and taskbar | Microsoft Learn tutorial above |
| ConEmu | yes, the origin of the sequence | ConEmu page above |
| VS Code | parsed by `@xterm/addon-progress`, which VS Code loads (`xtermTerminal.ts` lines 350-355, VS Code 1.140.0 with `@xterm/xterm ^6.1.0-beta.304`). Exposed as the `${progress}` variable for tab title and description templates, which are not in the defaults. | <https://raw.githubusercontent.com/microsoft/vscode/main/src/vs/workbench/contrib/terminal/browser/xterm/xtermTerminal.ts>; <https://raw.githubusercontent.com/xtermjs/xterm.js/master/addons/addon-progress/README.md>; `terminalConfiguration.ts` line 28 |
| Ghostty | yes, as a bar over each split. **Resets after about 15 s** without a new report. | <https://ghostty.org/docs/vt/osc/conemu> |
| kitty | since 0.47.0 (2026-05-19) it draws a bar (`progress_bar` option, default `top`). Since 0.38.0 it discards OSC 9 notifications starting with `4;`. | kitty changelog; <https://raw.githubusercontent.com/kovidgoyal/kitty/master/kitty/options/definition.py> lines 693-700 |
| WezTerm | parsed in `main` (`ConEmuProgress`). Release support is unverified; the WezTerm escape-sequence docs list OSC 9 only as a toast notification. | <https://raw.githubusercontent.com/wezterm/wezterm/main/wezterm-escape-parser/src/osc.rs> lines 327-346; <https://wezterm.org/escape-sequences.html> |
| VTE (GNOME Terminal) | exposed as termprops `vte.progress.hint` and `vte.progress.value` since VTE 0.80. Whether GNOME Terminal's UI shows it is unverified. | <https://raw.githubusercontent.com/GNOME/vte/master/src/vte/vteglobals.h> lines 183-210; `src/vteseq.cc` lines 2140-2228 |
| Alacritty, foot | not supported (reported) | <https://github.com/jdx/mise/discussions/6654> (secondary, unverified) |

**Unknown OSC is not always ignored.**

- OSC 9 collides with iTerm2's "show desktop notification" OSC 9. Ghostty resolves this by checking for a ConEmu sub-ID; kitty added its `4;` filter in 0.38.0 (both sources above).
- A terminal that implements OSC 9 notifications but not 9;4 may raise a toast reading "4;1;50". A user report also blames garbled output on an old VTE (0.52). Both are in mise discussion #6654, a secondary source.
- So send OSC 9;4 **only on positive detection**:
  - `WT_SESSION`;
  - `TERM_PROGRAM=vscode`;
  - `TERM=xterm-kitty`;
  - `TERM_PROGRAM=ghostty` (unverified variable);
  - `VTE_VERSION >= 8000`.
- Clearing:
  - send `ESC ] 9 ; 4 ; 0 BEL` on every exit path, beside the existing terminal restore;
  - for Ghostty, re-send the current state at least every 10 s while it should stay visible.
- crossterm's `Command` trait lets us define this as a typed command with no `unsafe`: implement `write_ansi`, and on Windows also `execute_winapi`, which is required under `cfg(windows)` (`REG/crossterm-0.29.0/src/command.rs:12-37`).

### 5.3 Request attention

- **Windows Terminal**:
  - on BEL, `bellStyle` accepts `"all"`, `"audible"`, `"window"`, `"taskbar"` or `"none"`, and the default is `"audible"` (<https://learn.microsoft.com/windows/terminal/customize-settings/profile-advanced>);
  - the schema says `"all"` "will play a sound, flash the taskbar icon (if the terminal window is not in focus) and flash the window" (`profiles.schema.json` line 2898);
  - so the taskbar only flashes if the user opted in;
  - the app-controlled attention cue is OSC 9;4. State 4 (warning/paused) or state 2 (error) colours the taskbar button while the next phase waits.
- **VS Code**:
  - BEL shows a bell next to the terminal name only when `terminal.integrated.enableVisualBell` is on, and it defaults to `false`;
  - it can play the `terminalBell` accessibility signal (`terminalConfiguration.ts`; <https://raw.githubusercontent.com/microsoft/vscode/main/src/vs/platform/accessibilitySignal/browser/accessibilitySignalService.ts>, `terminalBell`). Its default is unverified.
- **kitty**: `window_alert_on_bell yes` is the default. It will "make the dock icon bounce on macOS or the taskbar flash on Linux" (<https://raw.githubusercontent.com/kovidgoyal/kitty/master/kitty/options/definition.py> lines 1511-1517).
- **Alacritty**:
  - sets the window urgency hint on BEL when unfocused and DEC mode 1042 is on;
  - 1042 is on by default (`alacritty/src/event.rs` lines 1879-1884; `alacritty_terminal/src/term/mod.rs` `TermMode::default` includes `URGENCY_HINTS`);
  - xterm documents 1042 as "Enable Urgency window manager hint when Control-G is received" (ctlseqs lines 1010-1012).
- GNOME Terminal and Konsole bell or urgency behaviour is unverified.
- Recommendation: emit exactly one BEL when a phase completes. The app already has a `Step::Bell` path (`src/terminal.rs`).

## 6. Palette design

### 6.1 WCAG formulas and thresholds

Source: <https://www.w3.org/TR/WCAG22/>, definitions and success criteria.

- Relative luminance: `L = 0.2126 R + 0.7152 G + 0.0722 B`. Each channel is `C = C8/255`, then `C/12.92` if `C <= 0.04045`, else `((C + 0.055)/1.055)^2.4`. WCAG notes that the older threshold, 0.03928, "has no practical effect".
- Contrast ratio: `(L1 + 0.05)/(L2 + 0.05)`, where L1 is the lighter colour. It ranges from 1 to 21.
- The same definition says it is a failure to specify a text colour without a background colour, "because the user's default background color is unknown". This supports painting our own background.
- Thresholds:
  - 1.4.3 (AA): text 4.5:1 and large text 3:1. Large means at least 18 pt, or 14 pt bold.
  - 1.4.6 (AAA): 7:1 and 4.5:1.
  - 1.4.11: UI components and graphical objects 3:1 against adjacent colours.
  - 1.4.1: colour must not be "the only visual means of conveying information".
  - 2.3.1: no more than **three flashes in any one-second period**. This is binding for the alert.

### 6.2 Colour-blind safety

- "The most common type of color vision deficiency makes it hard to tell the difference between red and green"; deuteranomaly is the most common red-green type. Tritanopia confuses "blue and green, purple and red, and yellow and pink" (<https://www.nei.nih.gov/learn-about-eye-health/eye-conditions-and-diseases/color-blindness/types-color-vision-deficiency>).
- Hue alone therefore cannot separate work, short break and long break, and it must never be the only cue (WCAG 1.4.1).
- Luminance cannot separate three accents either. If every accent keeps at least 4.5:1 on our background (L = 0.0071), the dimmest accent has L at least 0.2071. The best possible accent-to-accent ratio is then (1+0.05)/(0.2071+0.05) = 4.08. So no three accents can be pairwise 3:1 apart, which would need a span of 9:1.
- Every phase therefore also carries:
  - its **label** (Work, Short break, Long break);
  - its **glyph** (🍅, ☕, 🌙);
  - its **position** or border style, for example a Thick border for work and a Rounded one for breaks.
- Simulated separation, using CIE76 ΔE with Machado et al. 2009 severity-1.0 matrices applied to linear sRGB. Matrices are from <https://www.inf.ufrgs.br/~oliveira/pubs_files/CVD_Simulation/CVD_Simulation.html>; the page does not state linear versus gamma, so that is unverified. Computed in `scratchpad/cvd.awk`.

| Pair | normal | protan | deutan | tritan |
| --- | --- | --- | --- | --- |
| work / short | 108.6 | 43.4 | 60.4 | 121.6 |
| work / long | 89.2 | 76.2 | 90.7 | 76.9 |
| short / long | 78.9 | 43.2 | 32.1 | 48.8 |
| work / success | 100.2 | **25.8** | **22.7** | 108.1 |
| short / success | 38.5 | 39.0 | 39.1 | **14.1** |
| work / alert | 58.6 | 47.4 | **27.4** | 49.9 |

- The weak pairs are exactly the classic pitfalls: red against green, and cyan against green for tritan.
- Never show success-green next to work-red or short-break-cyan as the only difference.

### 6.3 Dark or light background, and detection

- OSC 11 with `?` asks xterm to reply with the current background ("xterm replies with a control sequence of the same form", ctlseqs lines 2108-2117). It is not practical here, for four reasons.
- **Reason 1:** crossterm 0.29 has no event type for OSC replies. Its internal events are only `Event`, `CursorPosition`, `KeyboardEnhancementFlags` and `PrimaryDeviceAttributes` (`REG/crossterm-0.29.0/src/event.rs:1475-1487`).
- **Reason 2:** on Unix, `ESC ]` is parsed as Alt+`]` and the rest as ordinary key presses (`REG/crossterm-0.29.0/src/event/sys/unix/parse.rs`, `parse_event` fallback arm). A reply would reach the app as keystrokes.
- **Reason 3:** on Windows, query replies go into the console input stream (<https://learn.microsoft.com/windows/console/console-virtual-terminal-sequences>, "Query State").
- **Reason 4:** there is no timeout path for terminals that never answer.
- Therefore **paint our own background** on every cell each frame, for example `Block::new().style(Style::new().bg(BG))` over `frame.area()`. The palette then does not depend on the user's theme, and tachyonfx fades interpolate from a known RGB instead of `Reset`, which it treats as black.
- In the none tier do not paint; use the terminal's default colours.

### 6.4 Proposed palette ("Tomato Night")

Luminance and ratios were computed with the WCAG 2.2 formula (`scratchpad/wcag.awk`, `final.awk`).

| Role | Hex | L | Use |
| --- | --- | --- | --- |
| Background (painted) | `#12141C` | 0.0071 | every cell |
| Surface | `#1A1E2A` | 0.0132 | popup and panel bodies |
| Track (gauge unfilled) | `#262B3B` | 0.0246 | gauge background, and label text over the filled bar |
| Border | `#646D8A` | 0.1548 | block borders |
| Text | `#E6E9F2` | 0.8151 | primary text |
| Dim text | `#9AA3B8` | 0.3653 | key help, secondary labels |
| Work accent (tomato) | `#FF6B57` | 0.3246 | work phase: big digits, gauge fill, border |
| Short-break accent (mint) | `#3DDBD9` | 0.5666 | short break |
| Long-break accent (lavender) | `#B69CFF` | 0.4094 | long break |
| Alert / attention (amber) | `#FFC94D` | 0.6357 | phase-end banner and border, pulse |
| Success (green) | `#7BD88F` | 0.5531 | round completed, check marks |

Gauge: filled = the phase accent, unfilled = Track. Ratatui draws the label over the filled part in the Track colour (1.5).

Worked example, work accent on background:

- `#FF6B57`:
  - R: 255/255 = 1.0, linearised 1.0;
  - G: 107/255 = 0.41961, linearised ((0.41961+0.055)/1.055)^2.4 = 0.14704;
  - B: 87/255 = 0.34118, linearised 0.09531;
  - L = 0.2126·1.0 + 0.7152·0.14704 + 0.0722·0.09531 = 0.32464.
- `#12141C`: the channels linearise to 0.006050, 0.006995 and 0.011611, so L = 0.007127.
- Ratio = (0.32464 + 0.05)/(0.007127 + 0.05) = 0.37464/0.057127 = **6.56**.

| Foreground on background | 24-bit | 256-tier | Target |
| --- | --- | --- | --- |
| Text on Background | 15.14 | 16.15 | 4.5 (AAA 7) |
| Text on Surface | 13.70 | 14.69 | 4.5 |
| Text on Track | 11.60 | 13.04 | 4.5 |
| Dim on Background | 7.27 | 7.88 | 4.5 |
| Dim on Surface | 6.58 | 7.17 | 4.5 |
| Dim on Track | 5.57 | 6.36 | 4.5 |
| Work on Background | 6.56 | 6.29 | 4.5 |
| Short on Background | 10.79 | 10.87 | 4.5 |
| Long on Background | 8.04 | 9.28 | 4.5 |
| Alert on Background | 12.00 | 13.50 | 4.5 |
| Success on Background | 10.56 | 10.79 | 4.5 |
| Work on Surface | 5.93 | 5.72 | 4.5 |
| Short on Surface | 9.76 | 9.89 | 4.5 |
| Long on Surface | 7.27 | 8.44 | 4.5 |
| Alert on Surface | 10.86 | 12.28 | 4.5 |
| Success on Surface | 9.55 | 9.82 | 4.5 |
| Background on Alert (banner text) | 12.00 | 13.50 | 4.5 |
| Track on Work (gauge label over fill; also fill vs unfilled) | 5.02 | 5.08 | 4.5 and 3 |
| Track on Short | 8.27 | 8.78 | 4.5 and 3 |
| Track on Long | 6.16 | 7.49 | 4.5 and 3 |
| Border on Background | 3.59 | 4.12 | 3 (1.4.11) |
| Border on Surface | 3.24 | 3.75 | 3 |
| Track on Background | 1.31 | 1.24 | decorative. The gauge extent is shown by its border. |

- Every text pair is at least 5:1 and every UI pair at least 3:1 in both tiers.
- So VS Code's default `minimumContrastRatio` of 4.5 (3.4) will not recolour anything.

### 6.5 256-colour mapping

- Mostly the nearest xterm index by RGB distance over 16–255, computed from `256colres.pl`.
- Two deliberate overrides:
  - Track uses 235, not the nearest 236, to keep the gauge label at 5.08:1; 236 gives 4.43:1;
  - Border uses 243, not the nearest 60, to stay at least 3:1 on the surface; 60 gives 2.82:1.

| Role | 24-bit | 256 index (RGB) |
| --- | --- | --- |
| Background | `#12141C` | 233 (`#121212`) |
| Surface | `#1A1E2A` | 234 (`#1C1C1C`) |
| Track | `#262B3B` | 235 (`#262626`) |
| Border | `#646D8A` | 243 (`#767676`) |
| Text | `#E6E9F2` | 255 (`#EEEEEE`) |
| Dim | `#9AA3B8` | 248 (`#A8A8A8`) |
| Work | `#FF6B57` | 203 (`#FF5F5F`) |
| Short break | `#3DDBD9` | 80 (`#5FD7D7`) |
| Long break | `#B69CFF` | 147 (`#AFAFFF`) |
| Alert | `#FFC94D` | 221 (`#FFD75F`) |
| Success | `#7BD88F` | 114 (`#87D787`) |

### 6.6 16-colour and no-colour mapping

The 16 ANSI colours are defined by the user's theme. The ratios below are for Windows Terminal's default Campbell scheme on its `#0C0C0C` black (values from <https://learn.microsoft.com/windows/terminal/customize-settings/color-schemes>) and are illustrative only.

| Role | 16-colour (`ratatui::style::Color`) | Campbell ratio on Black | No colour (NO_COLOR) |
| --- | --- | --- | --- |
| Background | `Black` (painted) | n/a | `Reset`, not painted |
| Surface, Track | `Black` | n/a | `Reset` |
| Border | `DarkGray` (`#767676`) | 4.31 | `Reset` |
| Text | `White` (bright white `#F2F2F2`) | 17.47 | `Reset` |
| Dim | `Gray` (`#CCCCCC`). `DarkGray` would be 4.31, below 4.5. | 12.18 | `Reset` (optionally DIM) |
| Work | `LightRed` (`#E74856`) | 5.09 | `Reset` + BOLD + label + 🍅/[W] |
| Short break | `LightCyan` (`#61D6D6`) | 11.27 | `Reset` + label + glyph |
| Long break | `LightBlue` (`#3B78FF`). Not `LightMagenta` (`#B4009E`, 3.20). | 4.95 | `Reset` + label + glyph |
| Alert | `LightYellow` (`#F9F1A5`) | 16.91 | REVERSED + BOLD banner |
| Success | `LightGreen` (`#16C60C`) | 8.49 | `Reset` + ✓ |
| Gauge | filled = phase light colour on `Black` | ≥ 4.95 | the Gauge's full-block fill still shows with default fg |

- In the none tier, keep every colour `Reset`, so crossterm never emits the attribute-resetting `ESC[;m` (3.1).
- Turn tachyonfx colour effects off in that tier. Character effects such as `dissolve`/`coalesce` still work.

## 7. Animation pacing

### 7.1 What a frame costs

- `Terminal::draw` renders into the current buffer. `flush` then diffs it against the previous buffer and passes only changed cells to the backend, and `swap_buffers` follows (`REG/ratatui-core-0.1.2/src/terminal/buffers.rs:75-124`).
- The crossterm backend moves the cursor only when a cell is not adjacent to the previous one, and emits `SetColors` only when fg or bg changes (`REG/ratatui-crossterm-0.1.2/src/lib.rs:242-274`).
- Measured (probe `PROBE/framecost`, 80×24, our palette, CrosstermBackend writing to a `Vec<u8>`):

| Frame | Cells changed | Bytes written |
| --- | --- | --- |
| First full frame | 1920 | 2,564 |
| Idle tick (the seconds digit and gauge edge change) | 5 | 111 |
| Uniform full-screen `fade_from` (runs of equal colour) | 1920 | about 2,550 |
| Radial-gradient `fade_from` + `RadialPattern` (every cell a new colour) | 1920 | mean 17,464; worst 40,645 |

- CPU for render + effect + diff + encode (debug build): about 0.9 ms per frame for the uniform fade and about 1.4 ms for the gradient. The release build was about 0.22 ms per gradient frame.
- Bytes are not the bottleneck on our side. A worst-case gradient at 30 fps is about 1.2 MB/s into the terminal.
- The terminal's own render cost was not measured. For scale, crossterm's source notes 20–24 fps for a 171×51 full-screen app that changes every cell's colours, on iTerm2 (`REG/crossterm-0.29.0/src/style.rs:296-300`).

### 7.2 Recommended intervals

These are engineering judgement; no primary source prescribes a TUI frame rate.

- **Idle** (no effect running): keep the current model. The loop waits for input with a timeout and redraws at least every 250 ms (`src/app.rs:12`).
  - The display changes once a second. With `Gauge::use_unicode(true)` on a 60-column bar in a 25-minute phase, one eighth-cell step is 1500 s / 480 ≈ 3.1 s.
  - Better still: sleep until the next whole-second boundary of the remaining time. That gives at most about 1 redraw/s and removes visible jitter in the seconds digit.
- **While an effect runs** (`EffectManager::is_running()` is true): poll with a **33 ms** timeout, about 30 fps.
  - Allow 16 ms (60 fps) for short, small-area entry effects.
  - Drop back to the idle cadence as soon as `is_running()` is false.
- Restrict effects to the area that needs them (`with_area` or `CellFilter::Area`), so the diff stays small.
- Wrap each draw in synchronized output: crossterm's `BeginSynchronizedUpdate` writes `CSI ? 2026 h` and `EndSynchronizedUpdate` writes `CSI ? 2026 l` (`REG/crossterm-0.29.0/src/terminal.rs:399-470`).
  - Supported by Windows Terminal (`SO_SynchronizedOutput = DECPrivateMode(2026)` in <https://raw.githubusercontent.com/microsoft/terminal/main/src/terminal/adapter/DispatchTypes.hpp>) and xterm.js (`InputHandler.ts` mode 2026).
  - Also listed as supported by kitty, WezTerm, Alacritty and Ghostty; VTE is "not yet implemented" (<https://github.com/contour-terminal/vt-extensions/blob/master/synchronized-output.md>).
  - An unsupporting terminal just shows updates immediately (same page).
  - Which xterm.js version a given VS Code release bundles, and whether it includes 2026, is unverified.
- Alert pulses must stay at or below 3 flashes per second (WCAG 2.3.1). Use a ping-pong period of 1.5–2 s.

### 7.3 How tachyonfx expects to be driven

- Measure the real elapsed time each frame with `Instant`.
- Render widgets, then call `effects.process_effects(elapsed, frame.buffer_mut(), frame.area())` inside the same `draw` closure.
- Then poll input with a short timeout. The README uses 16 ms (`REG/tachyonfx-0.25.2/README.md:37-73`).
- Effects are time-based, not frame-based, so a slow terminal drops frames but keeps the duration.
- Pass the true elapsed time, not the nominal interval, because `poll` can return early on a key press.

## Recommendations

1. **Dependencies.** Add these; all were proven to share the single `ratatui-core 0.1.2`, build with 0 warnings and pass the project's advisory policy:
   - `tachyonfx = { version = "0.25.2", default-features = false, features = ["std", "std-duration"] }`. Leave out the `dsl` feature; we do not parse effect strings.
   - `tui-big-text = "0.8.10"`.
   - Optionally `throbber-widgets-tui = "0.11.1"`, which adds one crate.
   - Record in the dependency review:
     - `time 0.3.55` becomes compiled, via ratatui-widgets' default `calendar` feature, when tui-big-text is added;
     - compact_str 0.9 and 0.10 are both compiled;
     - tachyonfx's internal `unsafe`.
   - Do not add tui-box-text (it pulls in color-eyre and tracing), anything on ratatui 0.29 (a second ratatui, plus the RUSTSEC-2024-0436 `paste` finding), or crates that turn on ratatui's default features.
2. **Theme module.**
   - Semantic roles map to the 24-bit palette in 6.4, as `const Color::from_u32` values.
   - Add a `ColorDepth { TrueColor, Ansi256, Ansi16, NoColor }`, detected by the table in 3.5, with a `--color` override.
   - Run one quantize pass over `frame.buffer_mut().content` after effects, using the 6.5 and 6.6 tables.
   - Paint the background in every tier except `NoColor`.
3. **Glyph tiers** (4.5): emoji, symbols, ASCII. Use only safe emoji:
   - 🍅 work, ☕ short break, 🌙 long break;
   - 🔔 phase end, ✅ round done, 🔥 streak, ✨ celebration, ⏳ running, 💤 paused, 🎯 goal, 🏆 all rounds done.
   - Do not use ✔️, ⏸️, ▶️, ⏱️, ⚠️ or any ZWJ or skin-tone sequence.
   - Unit-test that every emoji-tier glyph has `cell_width() == 2`.
4. **Phase-end alert** (goal a). Everything happens while the timer waits for a key.
   - Draw `Clear` plus a Surface-styled `Block::bordered().border_type(BorderType::Thick)` with an Alert border and `.shadow(Shadow::new(dimmed()))` or `Shadow::dark_shade()`.
   - Inside it, put `BigText` (Quadrant) naming the next phase in its accent, a glyph line (for example "☕ Short break ready") and the key hint.
   - Effects:
     - entry: `fx::coalesce((500, Interpolation::QuadOut))` in parallel with `fx::sweep_in(Motion::LeftToRight, 12, 0, BG, (500, Interpolation::QuadOut))`;
     - waiting: `fx::repeating(fx::ping_pong(fx::hsl_shift_fg([0.0, 0.0, 18.0], (900, Interpolation::SineInOut))))` on the popup border only, via `CellFilter`. The period is 1.8 s, well under 3 flashes/s;
     - register the waiting effect with `add_unique_effect("alert", ..)` and cancel it with `cancel_unique_effect("alert")` on the key.
   - Terminal cues:
     - one BEL;
     - title `🔔 <phase> ready: press space`;
     - OSC 9;4 state 4 at 100, only on detected terminals (5.2), re-sent every 10 s;
     - on the key, set state 1 with live progress; on exit, state 0.
   - During phases, set OSC 9;4 state 1 to the phase percentage, sent only when the integer percent changes.
5. **Visual overhaul** (goal b).
   - Painted background, and a big `BigText` clock (Quadrant, or HalfHeight when rows allow) in the phase accent.
   - A `Gauge` with `use_unicode(true)`, fill = accent, unfilled = Track.
   - A `Block` per panel with `title_top` (phase + glyph) and `title_bottom` (round dots such as ●●○○ and key help) using `Flex::Center` and `Spacing`.
   - A `Sparkline` or `BarChart` of completed rounds.
   - A phase-change transition: `fx::sequence(&[fx::dissolve(250), fx::coalesce(250)])` over the clock, or `fx::fade_from` from the old accent.
   - A slow ambient `hsl_shift_fg` shimmer on the running gauge edge, at about 10 fps, is optional and off in `NoColor`.
6. **Pacing** (7.2): idle means redraw at the next second boundary, or within 250 ms; effects mean 33 ms. Bracket frames with synchronized output. Use `std-duration` so elapsed time is not truncated.
7. **Panic traps under our lints.** These are library panics that no lint catches:
   - clamp before `Gauge::ratio` or `percent`;
   - never call `fx::hsl_shift(None, None, ..)`;
   - use `Buffer::cell_mut`, not `buf[(x, y)]`;
   - keep `Layout::areas::<N>` in step with the constraint arrays;
   - `symbols::Marker` is `#[non_exhaustive]`, so a match on it needs the expectation described in the project's lint notes.

## Sources

- Crate sources, unpacked: `REG/ratatui-0.30.2`, `REG/ratatui-core-0.1.2`, `REG/ratatui-widgets-0.3.2`, `REG/ratatui-crossterm-0.1.2`, `REG/crossterm-0.29.0`, `REG/unicode-width-0.2.2`, `REG/tachyonfx-0.25.2`, `REG/tui-big-text-0.8.10`, `REG/tui-popup-0.7.7`, `REG/tui-bar-graph-0.3.6`, `REG/tui-box-text-0.3.5`, `REG/throbber-widgets-tui-0.11.1`, `REG/tui-equalizer-0.2.4`, `REG/tui-scrollbar-0.2.8`, `REG/tui-cards-0.3.6`, `REG/tui-qrcode-0.2.7`, `REG/tui-prompts-0.6.8`, `REG/tui-scrollview-0.6.8`, `REG/tui-rain-1.0.1`, `REG/ratatui-splash-screen-0.1.5`.
- Probes (2026-09-25, Rust 1.98.1, cargo-deny 0.20.2, advisory-db `593df8c1`): `PROBE/` (combined demo), `PROBE/per-crate/*` (one project per crate, plus `run.sh`), `PROBE/neg-tui-rain`, `PROBE/min-tachyonfx`, `PROBE/nocolor`, `PROBE/framecost`.
- crates.io API: <https://crates.io/api/v1/crates/tachyonfx> and the same path for each crate named in 2.1.
- Unicode: <https://www.unicode.org/Public/17.0.0/ucd/emoji/emoji-data.txt>, <https://www.unicode.org/Public/17.0.0/ucd/EastAsianWidth.txt>, <https://www.unicode.org/Public/17.0.0/ucd/emoji/emoji-variation-sequences.txt>, <https://www.unicode.org/reports/tr11/tr11-31.html>.
- W3C: <https://www.w3.org/TR/WCAG22/>.
- NO_COLOR: <https://no-color.org/>.
- NEI colour vision: <https://www.nei.nih.gov/learn-about-eye-health/eye-conditions-and-diseases/color-blindness/types-color-vision-deficiency>, <https://www.nei.nih.gov/learn-about-eye-health/eye-conditions-and-diseases/color-blindness>.
- CVD matrices: <https://www.inf.ufrgs.br/~oliveira/pubs_files/CVD_Simulation/CVD_Simulation.html> (Machado, Oliveira, Fernandes, IEEE TVCG 15(6), 2009).
- xterm: <https://invisible-island.net/xterm/ctlseqs/ctlseqs.txt> (patch 411), <https://raw.githubusercontent.com/ThomasDickey/xterm-snapshots/master/256colres.pl>.
- Microsoft: <https://learn.microsoft.com/windows/terminal/tutorials/progress-bar-sequences>, <https://learn.microsoft.com/windows/terminal/customize-settings/profile-advanced>, <https://learn.microsoft.com/windows/terminal/customize-settings/color-schemes>, <https://learn.microsoft.com/windows/console/console-virtual-terminal-sequences>, <https://devblogs.microsoft.com/commandline/24-bit-color-in-the-windows-console/>.
- microsoft/terminal source: `README.md`, `doc/cascadia/profiles.schema.json`, `src/types/CodepointWidthDetector.cpp`, `src/cascadia/TerminalConnection/ConptyConnection.cpp`, `src/terminal/adapter/DispatchTypes.hpp`, issue 11057.
- VS Code source: `src/vs/workbench/contrib/terminal/common/terminalEnvironment.ts`, `.../common/terminalConfiguration.ts`, `.../browser/xterm/xtermTerminal.ts`, `package.json` (1.140.0).
- xterm.js source: `src/common/input/UnicodeV6.ts`, `addons/addon-unicode11/src/UnicodeV11.ts`, `addons/addon-progress/README.md`, `src/common/InputHandler.ts`.
- ConEmu: <https://conemu.github.io/en/AnsiEscapeCodes.html>. Ghostty: <https://ghostty.org/docs/vt/osc/conemu>.
- kitty source: <https://raw.githubusercontent.com/kovidgoyal/kitty/master/docs/changelog.rst>, <https://raw.githubusercontent.com/kovidgoyal/kitty/master/kitty/child.py>, <https://raw.githubusercontent.com/kovidgoyal/kitty/master/kitty/options/definition.py>.
- WezTerm: <https://wezterm.org/escape-sequences.html>, <https://wezterm.org/config/lua/config/unicode_version.html>, <https://wezterm.org/config/fonts.html>; source `wezterm-escape-parser/src/osc.rs`, `term/src/terminalstate/performer.rs`, `config/src/config.rs`.
- VTE source: `src/spawn.cc`, `src/vte/vteglobals.h`, `src/vteseq.cc`.
- Konsole source: `src/profile/Profile.cpp`, `src/session/SessionManager.cpp`, `src/Screen.cpp`, `src/characters/CharacterWidth.cpp`.
- Alacritty source: `alacritty_terminal/src/tty/mod.rs`, `alacritty_terminal/src/term/mod.rs`, `alacritty/src/event.rs`, `CHANGELOG.md`; vte parser `src/ansi.rs`.
- Synchronized output: <https://github.com/contour-terminal/vt-extensions/blob/master/synchronized-output.md>.
- Secondary, flagged where used: <https://github.com/jdx/mise/discussions/6654>.
