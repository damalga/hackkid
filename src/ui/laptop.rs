use chrono::{NaiveDate, Utc};
use crossterm::{
    cursor, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use std::io::{self, Write};

use crate::game::world::World;
use crate::ui::frame::{RENDER_START, SIDE_PAD};

const TRACKS: &[(&str, &str, &str)] = &[
    ("01", "The Silence Before", "All regional oscillators ceased at 04:12 UTC. Automated emergency repeat active on 104.2 MHz."),
    ("02", "Mid-Stride", "Subjects vanished mid-step. Footwear and synthetic fibers unperturbed. No biological mass remaining."),
    ("03", "ICU Ward 104", "ICU Ward 104 containment intact due to deep sedative delta waves. Monitoring vital signs..."),
    ("04", "Acoustic Horizon", "Acoustic wavefront propagated at Mach 1.2 across metropolitan grid. Harmonic frequency matches human neural resonance."),
    ("05", "Empty Scrubs", "Clothing piles registered at every intersection. City census: 0% active population."),
    ("06", "Beeping Monitors", "Infusion pumps and cardiac monitors ticking endlessly to empty beds."),
    ("07", "Sublevel Generator", "Sublevel backup generator fuel at 48%. Emergency relays routing to rooftop array."),
    ("08", "Carrier Wave", "Carrier wave established on 104.2 MHz. Seeking secondary receiver nodes in sector 4."),
    ("09", "Resonance", "Resonance amplification sustained. Acoustic barrier thinning around hospital perimeter."),
    ("10", "Rooftop Array", "Rooftop broadcast array operational. Powering up directional horn."),
    ("11", "Static Echoes", "Static echoes detected from outer districts. Automated responses repeating."),
    ("12", "The Fanfare (Part I)", "THE FANFARE (Part I): The tone that unwove humanity. A chord of pure silence."),
    ("13", "The Fanfare (Part II)", "THE FANFARE (Part II): Transmission complete. Listen for the return frequency."),
];

pub fn render_laptop_view<W: Write>(
    out: &mut W,
    view_w: usize,
    render_rows: usize,
    world: &World,
) -> io::Result<()> {
    let bg = SetBackgroundColor(Color::Rgb { r: 12, g: 18, b: 40 });
    let fg = SetForegroundColor(Color::Rgb { r: 210, g: 220, b: 240 });
    let dim = SetForegroundColor(Color::Rgb { r: 150, g: 160, b: 190 });
    let highlight = SetForegroundColor(Color::Rgb { r: 255, g: 220, b: 90 });
    let accent = SetForegroundColor(Color::Rgb { r: 80, g: 200, b: 220 });

    for row_off in 0..render_rows as u16 {
        let row = RENDER_START + row_off;
        let line: String = " ".repeat(view_w);
        queue!(out, cursor::MoveTo(SIDE_PAD, row), bg, Print(&line), ResetColor)?;
    }

    // Header
    let header = "FANFARE // EMERGENCY BROADCAST TERMINAL (104.2 MHz)";
    let hx = SIDE_PAD + ((view_w.saturating_sub(header.chars().count())) as u16 / 2);
    queue!(out, cursor::MoveTo(hx, RENDER_START + 1), bg, accent, Print(header), ResetColor)?;

    // Calculate current week release gate
    let start_date = NaiveDate::from_ymd_opt(2026, 4, 14).unwrap();
    let today = Utc::now().date_naive();
    let days = (today - start_date).num_days();
    let current_week = if days < 0 { 1 } else { ((days / 7) + 1).min(12) };

    // Draw track list on the left
    let list_x = SIDE_PAD + 4;
    let list_y = RENDER_START + 3;
    for (i, &(num, title, _)) in TRACKS.iter().enumerate() {
        let sel = world.laptop_cursor == i;
        let unlocked = (i + 1 <= current_week as usize) || (i >= 11 && current_week >= 12);
        let status_str = if unlocked {
            if world.laptop_playing_track == Some(i) { "[PLAYING]" } else { "" }
        } else {
            "[LOCKED]"
        };
        let line = format!("{} {} {:<22} {}", if sel { "▶" } else { " " }, num, title, status_str);
        let row = list_y + i as u16;
        if row < RENDER_START + render_rows as u16 - 3 {
            queue!(out, cursor::MoveTo(list_x, row), bg)?;
            if sel {
                queue!(out, highlight, Print(&line), ResetColor)?;
            } else if unlocked {
                queue!(out, fg, Print(&line), ResetColor)?;
            } else {
                queue!(out, dim, Print(&line), ResetColor)?;
            }
        }
    }

    // Right panel: transcript / lyrics for selected track
    let sel_idx = world.laptop_cursor.min(TRACKS.len() - 1);
    let (num, title, transcript) = TRACKS[sel_idx];
    let unlocked = (sel_idx + 1 <= current_week as usize) || (sel_idx >= 11 && current_week >= 12);

    let panel_x = SIDE_PAD + 42;
    let panel_y = RENDER_START + 4;
    let panel_w = view_w.saturating_sub(46);

    queue!(out, cursor::MoveTo(panel_x, panel_y), bg, accent, Print(format!("TRACK {} : {}", num, title)), ResetColor)?;
    
    if unlocked {
        let playing_str = if world.laptop_playing_track == Some(sel_idx) { "STATUS: [PLAYING AUDIO CARRIER]" } else { "STATUS: [STOPPED]" };
        queue!(out, cursor::MoveTo(panel_x, panel_y + 2), bg, highlight, Print(playing_str), ResetColor)?;

        // Word wrap transcript
        let mut words = transcript.split_whitespace();
        let mut line_buf = String::new();
        let mut ry = panel_y + 4;
        while let Some(word) = words.next() {
            if line_buf.chars().count() + word.chars().count() + 1 > panel_w {
                queue!(out, cursor::MoveTo(panel_x, ry), bg, fg, Print(&line_buf), ResetColor)?;
                line_buf.clear();
                ry += 1;
            }
            if !line_buf.is_empty() {
                line_buf.push(' ');
            }
            line_buf.push_str(word);
        }
        if !line_buf.is_empty() && ry < RENDER_START + render_rows as u16 - 3 {
            queue!(out, cursor::MoveTo(panel_x, ry), bg, fg, Print(&line_buf), ResetColor)?;
        }
    } else {
        let locked_msg = "[LOCKED - SIGNAL CARRIER NOT DETECTED]";
        queue!(out, cursor::MoveTo(panel_x, panel_y + 3), bg, SetForegroundColor(Color::Red), Print(locked_msg), ResetColor)?;
    }

    // Battery
    let near_outlet = world.outlets.iter().any(|o| {
        let dx = o.x - world.player.x;
        let dy = o.y - world.player.y;
        (dx * dx + dy * dy).sqrt() < 1.5
    });
    let charge_ind = if near_outlet { " ⚡ charging" } else { "" };
    let batt = format!("Battery: {:>3.0}%{}", world.laptop_battery, charge_ind);
    let batt_len = batt.chars().count();
    let bx = SIDE_PAD + view_w as u16 - batt_len as u16 - 2;
    let by = RENDER_START + render_rows as u16 - 2;
    queue!(out, cursor::MoveTo(bx, by), bg, fg, Print(&batt), ResetColor)?;

    let hint = "Esc: Power Off   ↑↓: Select Track   Enter: Toggle Playback";
    let hint_len = hint.chars().count();
    let hx = SIDE_PAD + ((view_w.saturating_sub(hint_len)) as u16 / 2);
    let hy = RENDER_START + render_rows as u16 - 1;
    queue!(out, cursor::MoveTo(hx, hy), bg, dim, Print(hint), ResetColor)?;
    Ok(())
}
