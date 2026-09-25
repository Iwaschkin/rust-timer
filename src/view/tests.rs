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

/// The frame's text, one line per row.
fn text(buffer: &Buffer) -> String {
    buffer
        .content
        .chunks(usize::from(buffer.area.width.max(1)))
        .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
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

/// The cells a terminal draws: a wide glyph covers the cell after it, which ratatui
/// resets and the terminal never shows.
fn drawn_cells(buffer: &Buffer) -> impl Iterator<Item = &ratatui::buffer::Cell> + '_ {
    let width = usize::from(buffer.area.width.max(1));
    buffer.content.chunks(width).flat_map(|row| {
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
    })
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
    for wanted in ["Short break", "Ready", "Round 2 of 4", "05:00"] {
        assert!(short.contains(wanted), "{wanted:?} in\n{short}");
    }
    let long = text(&draw((60, 10), &skipped(7, base), base, TRUECOLOR_EMOJI)?);
    for wanted in ["Long break", "Ready", "Round 4 of 4", "15:00"] {
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
