//! The hospital map (M): the whole floor plan over the view, two map pixels per cell.

use hackkid_core::engine::map::{Material, Tile};
use hackkid_core::game::world::World;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Widget};

use super::{put, shade};

const BORDER: Color = Color::Rgb(150, 160, 180);
const YOU: (u8, u8, u8) = (255, 70, 60);
const AHEAD: (u8, u8, u8) = (255, 210, 90);
const SURVIVOR: (u8, u8, u8) = (90, 220, 230);

/// (priority, colour): walls and doors win when several tiles share a map pixel, so
/// thin walls never disappear when the map is shrunk to fit.
fn tile_color(t: Tile, m: Material) -> (u8, (u8, u8, u8)) {
    match t {
        t if t.is_any_door() => (5, (230, 190, 70)),
        Tile::WindowWall => (4, (130, 180, 220)),
        Tile::Wall | Tile::BathroomWall | Tile::HospitalOuterWall | Tile::Column | Tile::PushBox => (4, (205, 208, 214)),
        Tile::Hedge => (3, (34, 70, 34)),
        _ => (
            1,
            match m {
                Material::Corridor => (118, 118, 108),
                Material::Room => (92, 106, 116),
                Material::Tile => (104, 120, 130),
                Material::Terrazzo => (130, 116, 96),
                Material::Epoxy => (78, 122, 106),
                Material::Carpet => (62, 68, 94),
                Material::Concrete => (98, 98, 94),
                Material::Grass => (48, 88, 42),
                Material::Sidewalk => (132, 130, 124),
                Material::Asphalt => (42, 42, 46),
                Material::Parking => (56, 56, 62),
                Material::Pavers => (122, 116, 106),
            },
        ),
    }
}

pub fn draw(buf: &mut Buffer, area: Rect, world: &World) {
    let panel = Rect::new(area.x + 2, area.y + 1, area.width.saturating_sub(4), area.height.saturating_sub(2));
    if panel.width < 10 || panel.height < 6 {
        return;
    }
    shade(buf, panel, 0.25);
    let block = Block::bordered()
        .border_style(Style::new().fg(BORDER))
        .title(Line::styled("─ Hospital map ", Style::new().fg(BORDER)));
    let inner = block.inner(panel);
    block.render(panel, buf);
    let legend_h = 1;
    let (cols, px_rows) = (inner.width as f64, (inner.height - legend_h) as f64 * 2.0);
    let map = &world.map;
    let step = (map.width as f64 / cols).max(map.height as f64 / px_rows).max(0.5);
    let (used_w, used_h) = ((map.width as f64 / step).ceil(), (map.height as f64 / step).ceil());
    let (ox, oy) = (((cols - used_w) / 2.0).floor().max(0.0), ((px_rows - used_h) / 2.0).floor().max(0.0));

    // the colour of one map pixel (column, row in half-cells)
    let pixel = |cx: f64, py: f64| -> Option<(u8, u8, u8)> {
        let (mx, my) = (cx - ox, py - oy);
        if mx < 0.0 || my < 0.0 || mx >= used_w || my >= used_h {
            return None;
        }
        let (x0, y0) = ((mx * step) as usize, (my * step) as usize);
        let (x1, y1) = ((((mx + 1.0) * step).ceil() as usize).max(x0 + 1), (((my + 1.0) * step).ceil() as usize).max(y0 + 1));
        let mut best: Option<(u8, (u8, u8, u8))> = None;
        for ty in y0..y1.min(map.height) {
            for tx in x0..x1.min(map.width) {
                let c = tile_color(map.get(tx, ty), map.material(tx, ty));
                if best.is_none_or(|b| c.0 > b.0) {
                    best = Some(c);
                }
            }
        }
        best.map(|b| b.1)
    };
    // where something in the world lands on the map
    let spot = |x: f64, y: f64| ((x / step + ox).floor() as i64, (y / step + oy).floor() as i64);
    let p = &world.player;
    let you = spot(p.x, p.y);
    let ahead = spot(p.x + p.dir_x * step * 1.5, p.y + p.dir_y * step * 1.5);
    let survivors: Vec<(i64, i64)> = world.npcs.iter().map(|n| spot(n.x, n.y)).collect();
    let marker = |cx: i64, py: i64| {
        if (cx, py) == you {
            Some(YOU)
        } else if (cx, py) == ahead {
            Some(AHEAD)
        } else if survivors.contains(&(cx, py)) {
            Some(SURVIVOR)
        } else {
            None
        }
    };

    for row in 0..inner.height - legend_h {
        for col in 0..inner.width {
            let (cx, top, bot) = (col as f64, row as f64 * 2.0, row as f64 * 2.0 + 1.0);
            let t = marker(col as i64, top as i64).or_else(|| pixel(cx, top));
            let b = marker(col as i64, bot as i64).or_else(|| pixel(cx, bot));
            if t.is_none() && b.is_none() {
                continue;
            }
            if let Some(cell) = buf.cell_mut((inner.x + col, inner.y + row)) {
                let keep = cell.bg;
                let rgb = |c: Option<(u8, u8, u8)>, fallback: Color| c.map_or(fallback, |(r, g, b)| Color::Rgb(r, g, b));
                cell.set_char('▄').set_bg(rgb(t, keep)).set_fg(rgb(b, keep));
            }
        }
    }
    let legend = "■ you  ■ facing  ■ survivors  ■ doors";
    let ly = inner.bottom() - 1;
    let lx = inner.x + inner.width.saturating_sub(legend.chars().count() as u16) / 2;
    put(buf, lx, ly, legend, Style::new().fg(Color::Rgb(200, 200, 210)));
    for (offset, (r, g, b)) in [(0u16, YOU), (7, AHEAD), (17, SURVIVOR), (30, (230, 190, 70))] {
        put(buf, lx + offset, ly, "■", Style::new().fg(Color::Rgb(r, g, b)));
    }
}
