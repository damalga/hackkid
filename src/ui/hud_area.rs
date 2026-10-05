use crossterm::{
    cursor, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use std::io::{self, Write};

use crate::game::world::{GameMode, StaminaState, World};
use crate::ui::frame::SIDE_PAD;
use crate::ui::hud::{is_alarm, render_hud_groups};
use crate::ui::widgets::draw_box;

pub fn render_hud_area<W: Write>(
    out: &mut W,
    world: &World,
    view_w: usize,
    hud_start: u16,
    elapsed_ms: u64,
) -> io::Result<()> {
    let vw = view_w as u16;
    let gap: u16 = 1;
    let box_w: u16 = (vw.saturating_sub(3 * gap)) / 4;
    let salud_bar_w = (box_w as usize).saturating_sub(15).max(4);

    let hud_groups = render_hud_groups(
        &world.player.stats,
        salud_bar_w,
    );

    let clear_row = |out: &mut W, row: u16| -> io::Result<()> {
        queue!(out,
            cursor::MoveTo(0, row),
            ResetColor,
            terminal::Clear(ClearType::CurrentLine),
        )
    };

    let hud_region_rows: u16 = 14;
    if hud_start > 0 {
        clear_row(out, hud_start - 1)?;
    }
    for row in hud_start..(hud_start + hud_region_rows) {
        clear_row(out, row)?;
    }

    let white = SetForegroundColor(Color::White);
    let dim = SetForegroundColor(Color::Rgb { r: 180, g: 180, b: 200 });

    let salud_x = SIDE_PAD;
    let mapa_x = SIDE_PAD + box_w + gap;
    let third_x = SIDE_PAD + 2 * (box_w + gap);
    let empty_x = SIDE_PAD + 3 * (box_w + gap);

    draw_box(out, salud_x, hud_start, box_w, hud_region_rows, "Vitals")?;
    let cx = salud_x + 2;
    let inner_w = (box_w as usize).saturating_sub(3);
    let stats_start = hud_start + 1;
    let blink_off = ((elapsed_ms / 350) % 2) == 1;
    let alarm_col = SetForegroundColor(Color::Rgb { r: 255, g: 110, b: 90 });
    let recover_col = SetForegroundColor(Color::Rgb { r: 90, g: 170, b: 255 });
    let optimal_col = SetForegroundColor(Color::Rgb { r: 90, g: 220, b: 120 });
    let header_col = SetForegroundColor(Color::Rgb { r: 140, g: 170, b: 210 });

    let mut ry = stats_start;
    let end_ry = hud_start + hud_region_rows - 1;
    for group in &hud_groups {
        if ry >= end_ry { break; }
        // Sub-section header: "─ Title ────"
        let title_txt = format!(" {} ", group.title);
        let title_len = title_txt.chars().count();
        let dashes = inner_w.saturating_sub(title_len + 1);
        let header_line = format!("─{}{}", title_txt, "─".repeat(dashes));
        queue!(out, cursor::MoveTo(cx, ry), header_col, Print(&header_line), ResetColor)?;
        ry += 1;
        for row in &group.rows {
            if ry >= end_ry { break; }
            if row.is_stamina {
                match world.stamina_state {
                    StaminaState::Optimal => {
                        queue!(out, cursor::MoveTo(cx, ry), optimal_col, Print(&row.line), ResetColor)?;
                    }
                    StaminaState::Recovering => {
                        queue!(out, cursor::MoveTo(cx, ry), recover_col, Print(&row.line), ResetColor)?;
                    }
                    _ => {
                        queue!(out, cursor::MoveTo(cx, ry), white, Print(&row.line), ResetColor)?;
                    }
                }
                ry += 1;
                continue;
            }
            let alarm = is_alarm(row.direction, row.value);
            if alarm && blink_off {
                queue!(out, cursor::MoveTo(cx, ry), Print(" ".repeat(row.line.chars().count())), ResetColor)?;
            } else if alarm {
                queue!(out, cursor::MoveTo(cx, ry), alarm_col, Print(&row.line), ResetColor)?;
            } else {
                queue!(out, cursor::MoveTo(cx, ry), white, Print(&row.line), ResetColor)?;
            }
            ry += 1;
        }
    }

    draw_box(out, mapa_x, hud_start, box_w, hud_region_rows, "Map")?;
    if world.player.has_map {
        use crate::engine::map::Tile;
        let mw = (box_w - 2) as i32;
        let mh = (hud_region_rows - 2) as i32;
        let px = world.player.x as i32;
        let py = world.player.y as i32;
        let tiles_w = world.map.width as i32;
        let tiles_h = world.map.height as i32;
        let has_city = world.player.has_city_map;
        let wall_col = SetForegroundColor(Color::White);
        let door_col = SetForegroundColor(Color::Rgb { r: 255, g: 220, b: 40 });
        let road_col = SetForegroundColor(Color::Rgb { r: 90, g: 90, b: 100 });
        let side_col = SetForegroundColor(Color::Rgb { r: 170, g: 170, b: 175 });
        let garden_col = SetForegroundColor(Color::Rgb { r: 90, g: 150, b: 80 });

        if has_city {
            // Fit whole city into the map box (downscale sampling)
            let sx = tiles_w as f64 / mw as f64;
            let sy = tiles_h as f64 / mh as f64;
            let step = sx.max(sy).max(1.0);
            let used_w = ((tiles_w as f64 / step).ceil() as i32).min(mw);
            let used_h = ((tiles_h as f64 / step).ceil() as i32).min(mh);
            let off_x = (mw - used_w) / 2;
            let off_y = (mh - used_h) / 2;
            for r in 0..mh {
                queue!(out, cursor::MoveTo(mapa_x + 1, hud_start + 1 + r as u16))?;
                for c in 0..mw {
                    let cell_c = c - off_x;
                    let cell_r = r - off_y;
                    if cell_c < 0 || cell_r < 0 || cell_c >= used_w || cell_r >= used_h {
                        queue!(out, Print(" "))?;
                        continue;
                    }
                    let tx0 = (cell_c as f64 * step) as i32;
                    let ty0 = (cell_r as f64 * step) as i32;
                    let tx1 = (((cell_c + 1) as f64) * step).ceil() as i32;
                    let ty1 = (((cell_r + 1) as f64) * step).ceil() as i32;
                    let mut has_door = false;
                    let mut has_wall = false;
                    let mut has_road = false;
                    let mut has_side = false;
                    let mut has_garden = false;
                    for ty in ty0..ty1.min(tiles_h) {
                        for tx in tx0..tx1.min(tiles_w) {
                            let tile = world.map.get(tx as usize, ty as usize);
                            if tile.is_any_door() { has_door = true; }
                            else if tile.blocks_movement() { has_wall = true; }
                            else {
                                match tile {
                                    Tile::Road => has_road = true,
                                    Tile::Sidewalk => has_side = true,
                                    Tile::Garden | Tile::Outdoor | Tile::Parking => has_garden = true,
                                    _ => {}
                                }
                            }
                        }
                    }
                    if has_door { queue!(out, door_col, Print("+"))?; }
                    else if has_wall { queue!(out, wall_col, Print("·"))?; }
                    else if has_road { queue!(out, road_col, Print(":"))?; }
                    else if has_side { queue!(out, side_col, Print("."))?; }
                    else if has_garden { queue!(out, garden_col, Print("*"))?; }
                    else { queue!(out, Print(" "))?; }
                }
                queue!(out, ResetColor)?;
            }
            let pc = off_x + (px as f64 / step) as i32;
            let pr = off_y + (py as f64 / step) as i32;
            if pc >= 0 && pc < mw && pr >= 0 && pr < mh {
                queue!(out,
                    cursor::MoveTo(mapa_x + 1 + pc as u16, hud_start + 1 + pr as u16),
                    SetForegroundColor(Color::Rgb { r: 255, g: 220, b: 40 }),
                    Print("●"),
                    ResetColor,
                )?;
            }
        } else {
            for r in 0..mh {
                let ty = py + r - mh / 2;
                queue!(out, cursor::MoveTo(mapa_x + 1, hud_start + 1 + r as u16))?;
                for c in 0..mw {
                    let tx = px + c - mw / 2;
                    if tx < 0 || ty < 0 || tx >= tiles_w || ty >= tiles_h {
                        queue!(out, Print(" "))?;
                        continue;
                    }
                    let tile = world.map.get(tx as usize, ty as usize);
                    if tile.is_any_door() {
                        queue!(out, door_col, Print("+"))?;
                    } else if tile.blocks_movement() {
                        queue!(out, wall_col, Print("·"))?;
                    } else {
                        queue!(out, Print(" "))?;
                    }
                }
                queue!(out, ResetColor)?;
            }
            let center_x = mapa_x + 1 + (mw / 2) as u16;
            let center_y = hud_start + 1 + (mh / 2) as u16;
            queue!(out,
                cursor::MoveTo(center_x, center_y),
                SetForegroundColor(Color::Rgb { r: 255, g: 220, b: 40 }),
                Print("●"),
                ResetColor,
            )?;
        }
    } else {
        let msg1 = "Sector map";
        let msg2 = "not acquired";
        let inner_w = (box_w - 2) as usize;
        let x1 = mapa_x + 1 + ((inner_w.saturating_sub(msg1.chars().count())) as u16 / 2);
        let x2 = mapa_x + 1 + ((inner_w.saturating_sub(msg2.chars().count())) as u16 / 2);
        let cy = hud_start + hud_region_rows / 2;
        queue!(out, cursor::MoveTo(x1, cy), dim, Print(msg1), ResetColor)?;
        queue!(out, cursor::MoveTo(x2, cy + 1), dim, Print(msg2), ResetColor)?;
    }

    draw_box(out, third_x, hud_start, box_w, hud_region_rows, "Inventory (I: open)")?;
    let inv_cx = third_x + 2;
    if world.player.has_backpack {
        let slots = &world.player.inventory.slots;
        let cap = world.player.inventory.capacity;
        let slot_w = (box_w as usize).saturating_sub(6);
        let visible = cap.min(slots.len()).min(hud_region_rows as usize - 2);
        for i in 0..visible {
            let sel = world.mode == GameMode::Inventory && world.player.inventory_cursor == i;
            let full: String = slots.get(i).and_then(|s| s.as_ref())
                .map(|it| {
                    if it.units > 0 {
                        format!("{} ({})", it.label(), it.units)
                    } else {
                        it.label().to_string()
                    }
                }).unwrap_or_else(|| "—".to_string());
            let mut short: String = full.chars().take(slot_w).collect();
            if full.chars().count() > slot_w { short.pop(); short.push('…'); }
            let is_pocket = i >= 6;
            let prefix = if is_pocket { "* " } else if sel { "▶ " } else { "  " };
            let line = format!("{}{:<width$}", prefix, short, width = slot_w);
            let ry = hud_start + 1 + i as u16;
            if sel {
                queue!(out,
                    cursor::MoveTo(inv_cx, ry),
                    SetBackgroundColor(Color::Rgb { r: 90, g: 90, b: 130 }),
                    white,
                    Print(&line),
                    ResetColor,
                )?;
            } else {
                queue!(out,
                    cursor::MoveTo(inv_cx, ry),
                    white,
                    Print(&line),
                    ResetColor,
                )?;
            }
        }
    } else {
        queue!(out, cursor::MoveTo(inv_cx, hud_start + 1), dim, Print("(no backpack)"), ResetColor)?;
    }

    draw_box(out, empty_x, hud_start, box_w, hud_region_rows, "Wardrobe")?;
    let vest_cx = empty_x + 2;
    let rows = world.player.wardrobe.display_rows();
    let inner_w = (box_w as usize).saturating_sub(3);
    for (i, (tag, item)) in rows.iter().enumerate() {
        let ry = hud_start + 1 + i as u16;
        if ry >= hud_start + hud_region_rows - 1 { break; }
        let raw = format!("{:<8} {}", tag, item);
        let short: String = raw.chars().take(inner_w).collect();
        queue!(out, cursor::MoveTo(vest_cx, ry), white, Print(short), ResetColor)?;
    }
    // Backpack row (bottom of Wardrobe)
    let bag_row = hud_start + 1 + rows.len() as u16 + 1;
    if bag_row < hud_start + hud_region_rows - 1 {
        let bag_state = if world.player.has_backpack { "Equipped (B: drop)" } else { "None" };
        let raw = format!("{:<8} {}", "Backpack:", bag_state);
        let short: String = raw.chars().take(inner_w).collect();
        queue!(out, cursor::MoveTo(vest_cx, bag_row), white, Print(short), ResetColor)?;
    }

    Ok(())
}
