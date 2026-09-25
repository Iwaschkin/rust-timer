//! The dial: a ring of braille dots that lights up clockwise from twelve o'clock
//! as the phase runs, with the percentage in the middle.
//!
//! A braille cell holds 2 × 4 dots and a terminal cell is about twice as tall as it
//! is wide, so each dot is square. The canvas is sized in dots so the ring is round.

use crate::theme;
use crate::timer::Progress;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::Marker;
use ratatui::text::{Line, Span};
use ratatui::widgets::canvas::{Canvas, Points};
use std::f64::consts::TAU;

/// Points sampled around the ring; enough that no braille dot on it is missed.
const SAMPLES: u16 = 1440;

/// Positions of braille dots, in dots from the canvas's lower left.
type Dots = Vec<(f64, f64)>;

/// Draws the dial for `progress` in `area`, with `glyph` above the percentage.
pub(super) fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    progress: Progress,
    accent: Color,
    glyph: &str,
) {
    if area.width < 4 || area.height < 3 {
        return;
    }
    let width = f64::from(area.width) * 2.0;
    let height = f64::from(area.height) * 4.0;
    let centre = (width / 2.0, height / 2.0);
    let radius = (width / 2.0).min(height / 2.0) - 1.0;
    let lit = progress.ratio();
    let (track, arc) = ring(centre, radius, lit);
    let label = format!("{}%", progress.scaled(100));
    // Each character is two dots wide, so half the label is its length in dots.
    let half_label = f64::from(u16::try_from(label.len()).unwrap_or(0));
    let canvas = Canvas::default()
        .marker(Marker::Braille)
        .background_color(theme::SURFACE)
        .x_bounds([0.0, width])
        .y_bounds([0.0, height])
        .paint(move |context| {
            context.draw(&Points {
                coords: &track,
                color: theme::TRACK,
            });
            context.layer();
            context.draw(&Points {
                coords: &arc,
                color: accent,
            });
            context.print(
                centre.0 - 2.0,
                centre.1 + 3.0,
                Line::from(Span::raw(glyph.to_owned())),
            );
            context.print(
                centre.0 - half_label,
                centre.1 - 2.0,
                Line::from(Span::styled(
                    label.clone(),
                    Style::new().fg(accent).add_modifier(Modifier::BOLD),
                )),
            );
        });
    frame.render_widget(canvas, area);
}

/// The ring's dots, three deep: the unlit track, and the arc lit clockwise from
/// twelve o'clock over `lit` of the turn.
fn ring(centre: (f64, f64), radius: f64, lit: f64) -> (Dots, Dots) {
    let mut track = Vec::new();
    let mut arc = Vec::new();
    for step in 0..SAMPLES {
        let turn = f64::from(step) / f64::from(SAMPLES);
        let angle = turn * TAU;
        for depth in [0.0, 1.0, 2.0] {
            let reach = radius - depth;
            let point = (
                centre.0 + reach * angle.sin(),
                centre.1 + reach * angle.cos(),
            );
            if turn < lit {
                arc.push(point);
            } else {
                track.push(point);
            }
        }
    }
    (track, arc)
}
