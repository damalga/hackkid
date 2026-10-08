//! The emergency broadcast terminal: the 13 transmissions of *Fanfare*.

use hackkid_core::game::tracks::TRACKS;
use hackkid_core::game::world::{MusicSource, World};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};

use super::text::{ellipsize, wrap};
use super::{fill, put};

const BG: Color = Color::Rgb(12, 18, 40);
const FG: Color = Color::Rgb(210, 220, 240);
const DIM: Color = Color::Rgb(150, 160, 190);
const HIGHLIGHT: Color = Color::Rgb(255, 220, 90);
const ACCENT: Color = Color::Rgb(80, 200, 220);

pub fn draw(buf: &mut Buffer, area: Rect, world: &World) {
    fill(buf, area, BG);
    let st = |c: Color| Style::new().fg(c).bg(BG);
    let centre = |text: &str| area.x + area.width.saturating_sub(text.chars().count() as u16) / 2;

    let header = "FANFARE // EMERGENCY BROADCAST TERMINAL (104.2 MHz)";
    put(buf, centre(header), area.y + 1, header, st(ACCENT));

    // track list on the left, scrolled so the cursor stays visible
    let list_x = area.x + 4;
    let list_w = 34u16;
    let list_top = area.y + 3;
    let rows = area.height.saturating_sub(7) as usize;
    let cursor = world.laptop.cursor;
    let first = cursor.saturating_sub(rows.saturating_sub(1)).min(TRACKS.len().saturating_sub(rows));
    for (row, (i, t)) in TRACKS.iter().enumerate().skip(first).take(rows).enumerate() {
        let selected = i == cursor;
        let playing = world.laptop.playing == Some(i);
        let line = format!(
            "{} {:02} {:<20}{}",
            if selected { "▶" } else { " " },
            t.n,
            ellipsize(t.name, 20),
            if playing { " ♪" } else { "" }
        );
        let color = if selected { HIGHLIGHT } else if playing { ACCENT } else { FG };
        put(buf, list_x, list_top + row as u16, &line, st(color));
    }

    // the selected transmission on the right
    let t = &TRACKS[cursor.min(TRACKS.len() - 1)];
    let panel_x = list_x + list_w + 4;
    let panel_w = area.right().saturating_sub(panel_x + 3) as usize;
    let panel_y = area.y + 4;
    put(buf, panel_x, panel_y, &format!("TRANSMISSION {:02}: {}", t.n, t.name), st(ACCENT));
    let status = match (world.laptop.playing == Some(cursor), world.laptop.source) {
        (false, _) => "STATUS: [STANDING BY]",
        (true, Some(MusicSource::Carrier)) => "STATUS: [CARRIER TONE ONLY: NO AUDIO DATA]",
        (true, Some(MusicSource::Silent)) => "STATUS: [RECEIVING: NO SOUND DEVICE]",
        (true, _) => "STATUS: [PLAYING AUDIO CARRIER]",
    };
    put(buf, panel_x, panel_y + 2, status, st(HIGHLIGHT));
    let max_lines = area.bottom().saturating_sub(panel_y + 7) as usize;
    for (i, line) in wrap(t.log, panel_w).iter().take(max_lines).enumerate() {
        put(buf, panel_x, panel_y + 4 + i as u16, line, st(FG));
    }

    let charging = if world.near_outlet() { " ⚡ charging" } else { "" };
    let batt = format!("Battery: {:>3.0}%{charging}", world.laptop.battery);
    let bx = area.right().saturating_sub(batt.chars().count() as u16 + 3);
    put(buf, bx, area.bottom().saturating_sub(3), &batt, st(FG));

    let hint = "↑↓ select · E play / stop · Esc close (keeps playing)";
    put(buf, centre(hint), area.bottom().saturating_sub(2), hint, st(DIM));
}
