use crossterm::{
    cursor, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use std::io::{self, Write};

use crate::game::world::World;
use crate::ui::frame::{RENDER_START, SIDE_PAD};

pub fn render_laptop_view<W: Write>(
    out: &mut W,
    view_w: usize,
    render_rows: usize,
    world: &World,
) -> io::Result<()> {
    let bg = SetBackgroundColor(Color::Rgb { r: 12, g: 18, b: 40 });
    let fg = SetForegroundColor(Color::Rgb { r: 210, g: 220, b: 240 });
    for row_off in 0..render_rows as u16 {
        let row = RENDER_START + row_off;
        let line: String = " ".repeat(view_w);
        queue!(out, cursor::MoveTo(SIDE_PAD, row), bg, Print(&line), ResetColor)?;
    }

    let icons = ["Navegador"];
    for (i, name) in icons.iter().enumerate() {
        let ix = SIDE_PAD + 3 + (i as u16) * 14;
        let iy = RENDER_START + 2;
        let sel = world.laptop_cursor == i;
        let inner = format!(" {:<7} ", name);
        let inner_w = inner.chars().count();
        let top = format!("┌{}┐", "─".repeat(inner_w));
        let mid_border = "│";
        let bot = format!("└{}┘", "─".repeat(inner_w));
        let border = if sel {
            SetForegroundColor(Color::Rgb { r: 255, g: 220, b: 90 })
        } else {
            SetForegroundColor(Color::Rgb { r: 200, g: 210, b: 230 })
        };
        queue!(out, cursor::MoveTo(ix, iy), bg, border, Print(&top))?;
        queue!(out, cursor::MoveTo(ix, iy + 1), bg, border, Print(mid_border), bg, fg, Print(&inner), bg, border, Print(mid_border))?;
        queue!(out, cursor::MoveTo(ix, iy + 2), bg, border, Print(&bot), ResetColor)?;
    }

    let near_outlet = world.outlets.iter().any(|o| {
        let dx = o.x - world.player.x;
        let dy = o.y - world.player.y;
        (dx * dx + dy * dy).sqrt() < 1.5
    });
    let charge_ind = if near_outlet { " ⚡ cargando" } else { "" };
    let batt = format!("Batería: {:>3.0}%{}", world.laptop_battery, charge_ind);
    let batt_len = batt.chars().count();
    let bx = SIDE_PAD + view_w as u16 - batt_len as u16 - 2;
    let by = RENDER_START + render_rows as u16 - 2;
    queue!(out, cursor::MoveTo(bx, by), bg, fg, Print(&batt), ResetColor)?;

    let hint = "Esc apagar   Enter abrir   ↑↓←→ mover";
    let hint_len = hint.chars().count();
    let hx = SIDE_PAD + ((view_w.saturating_sub(hint_len)) as u16 / 2);
    let hy = RENDER_START + render_rows as u16 - 1;
    let dim = SetForegroundColor(Color::Rgb { r: 150, g: 160, b: 190 });
    queue!(out, cursor::MoveTo(hx, hy), bg, dim, Print(hint), ResetColor)?;
    Ok(())
}
