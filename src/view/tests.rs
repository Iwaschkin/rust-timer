use super::{border_type, phase_glyph, phase_label, remaining_label, render};
use crate::glyphs::GlyphTier;
use crate::options::Appearance;
use crate::settings::Settings;
use crate::terminal::KEYS;
use crate::theme::{self, ColorDepth};
use crate::timer::{Phase, Timer};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::style::{Color, Modifier};
use std::error::Error;
use std::time::{Duration, Instant};

const TRUECOLOR_EMOJI: Appearance = Appearance {
    depth: ColorDepth::TrueColor,
    glyphs: GlyphTier::Emoji,
};

/// The frame the program draws for `timer` at `now`: rendered, then fitted to the
/// colour tier, as the run loop does.
fn draw(
    size: (u16, u16),
    timer: &Timer,
    now: Instant,
    appearance: Appearance,
) -> Result<Buffer, Box<dyn Error>> {
    let mut terminal = Terminal::new(TestBackend::new(size.0, size.1))?;
    terminal.draw(|frame| {
        render(frame, timer, now, appearance, &KEYS);
        theme::quantize(frame.buffer_mut(), appearance.depth);
    })?;
    Ok(terminal.backend().buffer().clone())
}

/// The frame's text as a terminal shows it, one line per row.
fn text(buffer: &Buffer) -> String {
    rows(buffer)
        .map(|row| {
            drawn(row)
                .map(ratatui::buffer::Cell::symbol)
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn rows(buffer: &Buffer) -> impl Iterator<Item = &[ratatui::buffer::Cell]> + '_ {
    buffer.content.chunks(usize::from(buffer.area.width.max(1)))
}

/// The cells of `row` a terminal draws: a wide glyph covers the cell after it,
/// which ratatui resets and the terminal never shows.
fn drawn(row: &[ratatui::buffer::Cell]) -> impl Iterator<Item = &ratatui::buffer::Cell> + '_ {
    let mut hidden = 0_usize;
    row.iter().filter(move |cell| {
        if hidden > 0 {
            hidden -= 1;
            return false;
        }
        hidden = ratatui::text::Span::raw(cell.symbol())
            .width()
            .saturating_sub(1);
        true
    })
}

/// Skips `count` phases ahead of a fresh timer.
fn skipped(count: usize, now: Instant) -> Timer {
    let mut timer = Timer::start(Settings::default(), now);
    for _ in 0..count {
        assert!(!timer.skip(now));
    }
    timer
}

/// A running work phase, a paused one, a ready short break and a ready long break.
fn scenes(now: Instant) -> Vec<Timer> {
    let running = Timer::start(Settings::default(), now);
    let mut paused = Timer::start(Settings::default(), now);
    assert!(!paused.toggle(now + Duration::from_secs(90)));
    vec![running, paused, skipped(1, now), skipped(7, now)]
}

/// Every cell a terminal draws.
fn drawn_cells(buffer: &Buffer) -> impl Iterator<Item = &ratatui::buffer::Cell> + '_ {
    rows(buffer).flat_map(drawn)
}

fn colors(buffer: &Buffer) -> impl Iterator<Item = Color> + '_ {
    buffer
        .content
        .iter()
        .flat_map(|cell| [cell.fg, cell.bg, cell.underline_color])
}

#[test]
fn remaining_label_rounds_up_to_whole_seconds() {
    let cases = [
        (Duration::from_secs(25 * 60), "25:00"),
        (Duration::from_millis(24 * 60_000 + 59_001), "25:00"),
        (Duration::from_secs(24 * 60 + 59), "24:59"),
        (Duration::from_millis(1), "00:01"),
        (Duration::ZERO, "00:00"),
        (Duration::from_secs(99 * 60), "99:00"),
    ];
    for (remaining, expected) in cases {
        assert_eq!(remaining_label(remaining), expected, "{remaining:?}");
    }
}

#[test]
fn renders_in_tiny_terminals() -> Result<(), Box<dyn Error>> {
    let base = Instant::now();
    for timer in scenes(base) {
        for size in [(0, 0), (1, 1), (10, 3), (30, 8)] {
            for glyphs in [GlyphTier::Emoji, GlyphTier::Symbols, GlyphTier::Ascii] {
                let appearance = Appearance {
                    depth: ColorDepth::Ansi256,
                    glyphs,
                };
                draw(size, &timer, base, appearance)?;
            }
        }
    }
    Ok(())
}

#[test]
fn frame_shows_phase_state_round_and_keys() -> Result<(), Box<dyn Error>> {
    let base = Instant::now();
    let mut timer = Timer::start(Settings::default(), base);
    let running = text(&draw((60, 10), &timer, base, TRUECOLOR_EMOJI)?);
    let mut expected = vec!["Work", "Running", "Round 1 of 4", "25:00"];
    for (key, action) in KEYS {
        expected.extend([key, action]);
    }
    for wanted in expected {
        assert!(running.contains(wanted), "{wanted:?} in\n{running}");
    }
    assert!(!timer.toggle(base + Duration::from_secs(61)));
    let paused = text(&draw(
        (60, 10),
        &timer,
        base + Duration::from_secs(61),
        TRUECOLOR_EMOJI,
    )?);
    for wanted in ["Work", "Paused", "23:59"] {
        assert!(paused.contains(wanted), "{wanted:?} in\n{paused}");
    }
    Ok(())
}

#[test]
fn break_shows_round_it_follows() -> Result<(), Box<dyn Error>> {
    let base = Instant::now();
    let short = text(&draw((60, 10), &skipped(3, base), base, TRUECOLOR_EMOJI)?);
    for wanted in ["Short break", "Ready", "Round 2 of 4"] {
        assert!(short.contains(wanted), "{wanted:?} in\n{short}");
    }
    let long = text(&draw((60, 10), &skipped(7, base), base, TRUECOLOR_EMOJI)?);
    for wanted in ["Long break", "Ready", "Round 4 of 4"] {
        assert!(long.contains(wanted), "{wanted:?} in\n{long}");
    }
    Ok(())
}

#[test]
fn frames_quantize_to_256() -> Result<(), Box<dyn Error>> {
    let base = Instant::now();
    let appearance = Appearance {
        depth: ColorDepth::Ansi256,
        glyphs: GlyphTier::Emoji,
    };
    for timer in scenes(base) {
        let buffer = draw((80, 24), &timer, base, appearance)?;
        for color in colors(&buffer) {
            let fits = matches!(color, Color::Reset | Color::Indexed(16..=255));
            assert!(fits, "{color:?}");
        }
    }
    let mut ramp = Buffer::empty(ratatui::layout::Rect::new(0, 0, 64, 4));
    for (cell, step) in ramp.content.iter_mut().zip(0_u8..=255) {
        cell.fg = Color::Rgb(step.wrapping_mul(4), 255 - step, step.wrapping_mul(7));
        cell.bg = Color::Rgb(step, step, step);
    }
    theme::quantize(&mut ramp, ColorDepth::Ansi256);
    for color in colors(&ramp) {
        assert!(
            matches!(color, Color::Reset | Color::Indexed(16..=255)),
            "{color:?}"
        );
    }
    Ok(())
}

#[test]
fn frames_quantize_to_16() -> Result<(), Box<dyn Error>> {
    let base = Instant::now();
    let appearance = Appearance {
        depth: ColorDepth::Ansi16,
        glyphs: GlyphTier::Symbols,
    };
    for timer in scenes(base) {
        let buffer = draw((80, 24), &timer, base, appearance)?;
        for color in colors(&buffer) {
            let named = !matches!(color, Color::Rgb(..) | Color::Indexed(_));
            assert!(named, "{color:?}");
        }
    }
    Ok(())
}

#[test]
fn no_color_keeps_modifiers() -> Result<(), Box<dyn Error>> {
    let base = Instant::now();
    let appearance = Appearance {
        depth: ColorDepth::None,
        glyphs: GlyphTier::Ascii,
    };
    for timer in scenes(base) {
        let buffer = draw((80, 24), &timer, base, appearance)?;
        for color in colors(&buffer) {
            assert_eq!(color, Color::Reset);
        }
        let bold = buffer
            .content
            .iter()
            .any(|cell| cell.modifier.contains(Modifier::BOLD));
        assert!(bold, "labels stay bold without colour");
    }
    Ok(())
}

#[test]
fn background_is_painted() -> Result<(), Box<dyn Error>> {
    let base = Instant::now();
    for depth in [
        ColorDepth::TrueColor,
        ColorDepth::Ansi256,
        ColorDepth::Ansi16,
    ] {
        let appearance = Appearance {
            depth,
            glyphs: GlyphTier::Emoji,
        };
        for timer in scenes(base) {
            let buffer = draw((80, 24), &timer, base, appearance)?;
            let unpainted = drawn_cells(&buffer)
                .filter(|cell| cell.bg == Color::Reset)
                .count();
            assert_eq!(unpainted, 0, "{depth:?}");
        }
    }
    Ok(())
}

#[test]
fn phases_differ_beyond_colour() {
    let phases = [Phase::Work, Phase::ShortBreak, Phase::LongBreak];
    for (position, first) in phases.iter().enumerate() {
        for second in phases.iter().skip(position + 1) {
            assert_ne!(phase_label(*first), phase_label(*second));
            assert_ne!(border_type(*first), border_type(*second));
            for tier in [GlyphTier::Emoji, GlyphTier::Symbols, GlyphTier::Ascii] {
                assert_ne!(
                    tier.glyph(phase_glyph(*first)),
                    tier.glyph(phase_glyph(*second)),
                    "{tier:?}"
                );
            }
        }
    }
}

/// Writes HTML renders of the main scenes to `target/preview/`, for design review
/// in a browser. Run with `cargo test -- --ignored preview_screens`.
#[test]
#[ignore = "writes files for a person to look at; not evidence for any row"]
fn preview_screens() -> Result<(), Box<dyn Error>> {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/preview");
    std::fs::create_dir_all(&directory)?;
    let base = Instant::now();
    let later = base + Duration::from_secs(7 * 60 + 18);
    let mut running = skipped(2, base);
    assert!(!running.toggle(base));
    let mut paused = Timer::start(Settings::default(), base);
    assert!(!paused.toggle(later));
    let named = [
        ("work-running", running, later),
        ("work-paused", paused, later),
        ("short-ready", skipped(1, base), base),
        ("long-ready", skipped(7, base), base),
    ];
    for (name, timer, now) in &named {
        for (size_name, size) in [
            ("large", (100, 30)),
            ("readme", (80, 24)),
            ("medium", (60, 18)),
            ("small", (36, 10)),
        ] {
            for (tier_name, depth) in [
                ("truecolor", ColorDepth::TrueColor),
                ("256", ColorDepth::Ansi256),
            ] {
                let appearance = Appearance {
                    depth,
                    glyphs: GlyphTier::Emoji,
                };
                let buffer = draw(size, timer, *now, appearance)?;
                let file = directory.join(format!("{name}-{size_name}-{tier_name}.html"));
                std::fs::write(file, html(&buffer))?;
                let plain = directory.join(format!("{name}-{size_name}-{tier_name}.txt"));
                std::fs::write(plain, text(&buffer))?;
            }
        }
    }
    Ok(())
}

/// The buffer as an HTML page of fixed-width cells, in the font the owner's
/// terminal uses.
fn html(buffer: &Buffer) -> String {
    let mut page = String::from(
        "<!doctype html><meta charset=\"utf-8\"><body style=\"margin:0;background:#000\">\
         <div style=\"font:18px/1.25 'JetBrains Mono','Segoe UI Emoji',monospace;padding:0\">",
    );
    let width = usize::from(buffer.area.width.max(1));
    for row in buffer.content.chunks(width) {
        page.push_str("<div style=\"display:flex;height:1.25em\">");
        let mut skip = 0_usize;
        for cell in row {
            if skip > 0 {
                skip -= 1;
                continue;
            }
            let symbol = cell.symbol();
            let cells = unicode_cells(symbol);
            skip = cells.saturating_sub(1);
            let weight = if cell.modifier.contains(Modifier::BOLD) {
                "bold"
            } else {
                "normal"
            };
            page.push_str(&format!(
                "<span style=\"display:inline-block;width:{cells}ch;white-space:pre;color:{};background:{};font-weight:{weight}\">{}</span>",
                css(cell.fg, "#e6e9f2"),
                css(cell.bg, "#0c0c0c"),
                escape(symbol),
            ));
        }
        page.push_str("</div>");
    }
    page.push_str("</div></body>");
    page
}

fn unicode_cells(symbol: &str) -> usize {
    ratatui::text::Span::raw(symbol).width().max(1)
}

fn escape(symbol: &str) -> String {
    symbol
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn css(color: Color, default: &str) -> String {
    let rgb = match color {
        Color::Rgb(red, green, blue) => [red, green, blue],
        Color::Indexed(index) => theme::indexed_rgb(index),
        Color::Reset
        | Color::Black
        | Color::Red
        | Color::Green
        | Color::Yellow
        | Color::Blue
        | Color::Magenta
        | Color::Cyan
        | Color::Gray
        | Color::DarkGray
        | Color::LightRed
        | Color::LightGreen
        | Color::LightYellow
        | Color::LightBlue
        | Color::LightMagenta
        | Color::LightCyan
        | Color::White => return default.to_owned(),
    };
    let [red, green, blue] = rgb;
    format!("#{red:02x}{green:02x}{blue:02x}")
}

const WORK_LENGTH: Duration = Duration::from_secs(25 * 60);

fn is_braille(cell: &ratatui::buffer::Cell) -> bool {
    cell.symbol()
        .chars()
        .next()
        .is_some_and(|character| ('\u{2801}'..='\u{28FF}').contains(&character))
}

fn is_block(cell: &ratatui::buffer::Cell) -> bool {
    "█▀▄▌▐▖▗▘▙▚▛▜▝▞▟".contains(cell.symbol()) && !cell.symbol().trim().is_empty()
}

fn count(buffer: &Buffer, wanted: impl Fn(&ratatui::buffer::Cell) -> bool) -> usize {
    buffer.content.iter().filter(|cell| wanted(cell)).count()
}

#[test]
fn large_layout_shows_every_panel() -> Result<(), Box<dyn Error>> {
    let base = Instant::now();
    let timer = Timer::start(Settings::default(), base);
    let now = base + WORK_LENGTH / 3;
    let buffer = draw((80, 24), &timer, now, TRUECOLOR_EMOJI)?;
    let shown = text(&buffer);
    assert!(count(&buffer, is_braille) > 20, "the dial\n{shown}");
    let digits = count(&buffer, |cell| cell.fg == theme::WORK && is_block(cell));
    assert!(digits > 40, "block digits, {digits} cells\n{shown}");
    let track = count(&buffer, |cell| cell.bg == theme::TRACK);
    assert!(track > 100, "bar and ribbon, {track} cells\n{shown}");
    for wanted in ["08:20 of 25:00", "33%", "Round 1 of 4", "space", "quit"] {
        assert!(shown.contains(wanted), "{wanted:?} in\n{shown}");
    }
    Ok(())
}

#[test]
fn medium_layout_drops_the_dial() -> Result<(), Box<dyn Error>> {
    let base = Instant::now();
    let timer = Timer::start(Settings::default(), base);
    let buffer = draw((60, 18), &timer, base, TRUECOLOR_EMOJI)?;
    let shown = text(&buffer);
    assert_eq!(count(&buffer, is_braille), 0, "no dial\n{shown}");
    let digits = count(&buffer, |cell| cell.fg == theme::WORK && is_block(cell));
    assert!(digits > 20, "block digits, {digits} cells\n{shown}");
    assert!(shown.contains("00:00 of 25:00"), "the bar's label\n{shown}");
    Ok(())
}

#[test]
fn compact_layout_keeps_essentials() -> Result<(), Box<dyn Error>> {
    let base = Instant::now();
    let timer = Timer::start(Settings::default(), base);
    let buffer = draw((40, 10), &timer, base, TRUECOLOR_EMOJI)?;
    let shown = text(&buffer);
    assert_eq!(count(&buffer, is_braille), 0, "no dial\n{shown}");
    for wanted in ["Work", "Running", "25:00", "space", "skip", "quit"] {
        assert!(shown.contains(wanted), "{wanted:?} in\n{shown}");
    }
    Ok(())
}

#[test]
fn gradient_bar_fills_by_eighths() {
    let base = Instant::now();
    let timer = Timer::start(Settings::default(), base);
    let area = ratatui::layout::Rect::new(0, 0, 10, 1);
    let mut buffer = Buffer::empty(area);
    super::bar::render(
        &mut buffer,
        area,
        timer.progress(base + WORK_LENGTH * 33 / 80),
        theme::WORK,
    );
    let symbols: Vec<&str> = buffer.content.iter().map(|cell| cell.symbol()).collect();
    assert_eq!(symbols, ["█", "█", "█", "█", "▏", " ", " ", " ", " ", " "]);
    let first = buffer.content.first().map(|cell| cell.fg);
    assert_eq!(
        first,
        Some(theme::faded(theme::WORK)),
        "the ramp starts faded"
    );
    let edge = buffer.content.get(4).map(|cell| cell.fg);
    assert_eq!(
        edge,
        Some(theme::WORK),
        "the leading edge shows the accent in full"
    );
    let mut full = Buffer::empty(area);
    super::bar::render(
        &mut full,
        area,
        timer.progress(base + WORK_LENGTH),
        theme::WORK,
    );
    let last = full
        .content
        .last()
        .map(|cell| (cell.symbol().to_owned(), cell.fg));
    assert_eq!(
        last,
        Some(("█".to_owned(), theme::WORK)),
        "the ramp ends in the accent"
    );
}

/// The dial's lit braille cells, as (column, row), at `share` of a work phase.
fn lit_cells(share: u32) -> Result<Vec<(u16, u16)>, Box<dyn Error>> {
    let base = Instant::now();
    let timer = Timer::start(Settings::default(), base);
    let progress = timer.progress(base + WORK_LENGTH * share / 100);
    let mut terminal = Terminal::new(TestBackend::new(24, 10))?;
    terminal.draw(|frame| super::dial::render(frame, frame.area(), progress, theme::WORK, "W"))?;
    let buffer = terminal.backend().buffer();
    Ok(buffer
        .content
        .iter()
        .zip(0_u16..)
        .filter(|(cell, _)| is_braille(cell) && cell.fg == theme::WORK)
        .map(|(_, index)| (index % 24, index / 24))
        .collect())
}

#[test]
fn dial_arc_follows_progress() -> Result<(), Box<dyn Error>> {
    let quarter = lit_cells(25)?;
    assert!(!quarter.is_empty());
    for (column, row) in &quarter {
        assert!(
            *column >= 11 && *row <= 5,
            "a quarter lights the top right: {column},{row}"
        );
    }
    let three_quarters = lit_cells(75)?;
    assert!(three_quarters.len() > quarter.len() * 2);
    for (column, row) in &three_quarters {
        assert!(
            !(*column < 10 && *row < 4),
            "the top left stays unlit: {column},{row}"
        );
    }
    assert!(lit_cells(0)?.is_empty(), "nothing is lit at the start");
    Ok(())
}

#[test]
fn ribbon_shows_the_cycle() -> Result<(), Box<dyn Error>> {
    let base = Instant::now();
    let mut timer = skipped(2, base);
    assert!(!timer.toggle(base));
    let segments = timer.cycle();
    assert_eq!(segments.len(), 8);
    let widths = super::ribbon::widths(&segments, 67).ok_or("67 cells fit")?;
    assert_eq!(widths.iter().sum::<u16>() + 7, 67);
    // 60 cells after the 7 gaps, over a 130-minute cycle.
    for (segment, width) in segments.iter().zip(&widths) {
        let exact = segment.length.as_secs() * 60 / (130 * 60);
        assert!(
            u64::from(*width).abs_diff(exact) <= 1,
            "{segment:?}: {width} cells"
        );
    }
    let (work, short) = (widths.first().copied(), widths.get(1).copied());
    assert!(work > short.map(|cells| cells * 3), "{widths:?}");
    let area = ratatui::layout::Rect::new(0, 0, 67, 1);
    let mut buffer = Buffer::empty(area);
    let half = timer.progress(base + WORK_LENGTH / 2);
    assert!(super::ribbon::render(
        &mut buffer,
        area,
        &segments,
        timer.position(),
        half
    ));
    let first = buffer
        .content
        .first()
        .map(|cell| (cell.symbol().to_owned(), cell.fg));
    assert_eq!(
        first,
        Some(("█".to_owned(), theme::faded(theme::WORK))),
        "W1 passed"
    );
    let last = buffer.content.last().map(|cell| cell.bg);
    assert_eq!(last, Some(theme::TRACK), "the long break is still to come");
    let many = Timer::start(
        Settings {
            every: "99".parse()?,
            ..Settings::default()
        },
        base,
    );
    assert!(
        super::ribbon::widths(&many.cycle(), 60).is_none(),
        "198 phases need more room"
    );
    Ok(())
}

#[test]
fn ready_popup_announces_next_phase() -> Result<(), Box<dyn Error>> {
    let base = Instant::now();
    let buffer = draw((80, 24), &skipped(1, base), base, TRUECOLOR_EMOJI)?;
    let shown = text(&buffer);
    for wanted in [
        "Time's up",
        "Short break is ready",
        "5 min · round 1 of 4",
        "space",
        "start",
        "skip",
    ] {
        assert!(shown.contains(wanted), "{wanted:?} in\n{shown}");
    }
    assert!(
        count(&buffer, |cell| cell.fg == theme::ALERT
            && cell.symbol() == "━")
            > 20,
        "{shown}"
    );
    assert!(
        count(&buffer, |cell| cell.bg == theme::SHADOW) > 10,
        "the drop shadow"
    );
    let letters = count(&buffer, |cell| {
        cell.fg == theme::SHORT_BREAK && is_block(cell)
    });
    assert!(
        letters > 10,
        "BREAK in block letters, {letters} cells\n{shown}"
    );
    Ok(())
}

#[test]
fn paused_screen_dims_the_clock() -> Result<(), Box<dyn Error>> {
    let base = Instant::now();
    let mut timer = Timer::start(Settings::default(), base);
    assert!(!timer.toggle(base + Duration::from_secs(90)));
    let buffer = draw(
        (80, 24),
        &timer,
        base + Duration::from_secs(90),
        TRUECOLOR_EMOJI,
    )?;
    let shown = text(&buffer);
    assert!(shown.contains("PAUSED"), "{shown}");
    let dimmed = count(&buffer, |cell| {
        cell.fg == theme::faded(theme::WORK) && is_block(cell)
    });
    assert!(dimmed > 40, "dimmed digits, {dimmed} cells\n{shown}");
    let bright = count(&buffer, |cell| {
        cell.fg == theme::WORK && cell.symbol() == "█"
    });
    assert_eq!(bright, 0, "no digit keeps the full accent\n{shown}");
    Ok(())
}
