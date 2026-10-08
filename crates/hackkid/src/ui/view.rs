use hackkid_core::engine::renderer::Viewport;
use hackkid_core::game::world::World;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};

use super::put;

/// Copies a rendered frame into the cells: two pixels per cell, the top one as the
/// background colour and the bottom one as the colour of a lower half block (`▄`).
fn blit(buf: &mut Buffer, view: Rect, vp: &Viewport) {
    let (w, _) = vp.size();
    let px = |x: usize, y: usize| {
        let i = (y * w + x) * 4;
        Color::Rgb(vp.rgba[i], vp.rgba[i + 1], vp.rgba[i + 2])
    };
    for ty in 0..view.height as usize {
        for tx in 0..view.width as usize {
            if let Some(cell) = buf.cell_mut((view.x + tx as u16, view.y + ty as u16)) {
                cell.set_char('▄').set_bg(px(tx, ty * 2)).set_fg(px(tx, ty * 2 + 1));
            }
        }
    }
}

/// The first-person view.
pub fn world(buf: &mut Buffer, view: Rect, world: &World, vp: &mut Viewport, now_ms: u64) {
    vp.resize(view.width as usize, view.height as usize * 2);
    world.render_view(vp, now_ms);
    blit(buf, view, vp);
}

/// Lying on your back, looking up at the ceiling.
pub fn ceiling(buf: &mut Buffer, view: Rect, world: &World, vp: &mut Viewport, now_ms: u64, wake_text: bool) {
    vp.resize(view.width as usize, view.height as usize * 2);
    world.render_ceiling(vp, now_ms);
    blit(buf, view, vp);
    if !wake_text {
        return;
    }
    let width = (view.width as usize).saturating_sub(8).min(90);
    let mut lines = super::text::wrap(
        "You open your eyes. Only the ceiling tiles, a humming fluorescent light, and absolute silence. The Fanfare has left the hospital completely empty.",
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
