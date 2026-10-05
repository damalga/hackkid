use hackkid_core::engine::renderer::Renderer;
use hackkid_core::game::world::World;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};

use super::put;

/// The first-person view: two pixels per cell, the top one as the background
/// colour and the bottom one as the colour of a lower half block (`▄`).
pub fn world(buf: &mut Buffer, view: Rect, world: &World, now_ms: u64) {
    let (w, rows) = (view.width as usize, view.height as usize);
    let frame = world.render_view(&Renderer::new(w, rows * 2), now_ms);
    let px = |x: usize, y: usize| {
        let i = (y * w + x) * 4;
        Color::Rgb(frame[i], frame[i + 1], frame[i + 2])
    };
    for ty in 0..rows {
        for tx in 0..w {
            if let Some(cell) = buf.cell_mut((view.x + tx as u16, view.y + ty as u16)) {
                cell.set_char('▄').set_bg(px(tx, ty * 2)).set_fg(px(tx, ty * 2 + 1));
            }
        }
    }
}

/// Lying on your back: a ceiling tile and the glow of a fluorescent light.
pub fn ceiling(buf: &mut Buffer, view: Rect, wake_text: bool) {
    let cx = view.width as f64 / 2.0;
    let cy = view.height as f64 / 2.0;
    let max_d = (cx * cx + cy * cy).sqrt();
    for ty in 0..view.height {
        for tx in 0..view.width {
            let d = ((tx as f64 - cx).powi(2) + (ty as f64 - cy).powi(2)).sqrt();
            let t = (1.0 - d / max_d).max(0.0);
            let glow = (t.powf(2.5) * 220.0) as u8;
            let bg = Color::Rgb(25u8.saturating_add(glow), 25u8.saturating_add(glow), 25u8.saturating_add((glow as f64 * 0.85) as u8));
            if let Some(cell) = buf.cell_mut((view.x + tx, view.y + ty)) {
                cell.reset();
                cell.set_bg(bg);
            }
        }
    }
    if !wake_text {
        return;
    }
    let width = (view.width as usize).saturating_sub(8).min(90);
    let mut lines = super::text::wrap(
        "You open your eyes. Only the ceiling tile, a humming fluorescent light, and absolute silence. The Fanfare has left the hospital completely empty.",
        width,
    );
    lines.push(String::new());
    lines.push("[Press any key]".into());
    let style = Style::new().fg(Color::Black).bg(Color::Rgb(240, 235, 210));
    let top = (view.y + view.height / 2).saturating_sub(lines.len() as u16 / 2);
    for (i, line) in lines.iter().enumerate() {
        if line.is_empty() {
            continue;
        }
        let x = view.x + view.width.saturating_sub(line.chars().count() as u16) / 2;
        put(buf, x, top + i as u16, line, style);
    }
}
