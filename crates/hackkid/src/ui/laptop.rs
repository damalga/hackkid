//! The emergency terminal: two screens. BROADCAST is the 13 transmissions of *Fanfare*;
//! NET is a command line onto the hospital's building network (see `hackkid_core::game::net`).

use hackkid_core::game::net::{Minigame, Screen};
use hackkid_core::game::tracks::TRACKS;
use hackkid_core::game::world::{MusicSource, World};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};

use super::text::{ellipsize, wrap};
use super::{Hits, Hover, fill, hovered, put};

const BG: Color = Color::Rgb(12, 18, 40);
const FG: Color = Color::Rgb(210, 220, 240);
const DIM: Color = Color::Rgb(150, 160, 190);
const HIGHLIGHT: Color = Color::Rgb(255, 220, 90);
const ACCENT: Color = Color::Rgb(80, 200, 220);
const NET_BG: Color = Color::Rgb(8, 12, 14);
const NET_FG: Color = Color::Rgb(150, 230, 160);
const NET_DIM: Color = Color::Rgb(80, 130, 90);
const NET_HOT: Color = Color::Rgb(255, 170, 60);
const NET_BAD: Color = Color::Rgb(240, 90, 80);

pub fn draw(buf: &mut Buffer, area: Rect, world: &World, hover: Hover, hits: &mut Hits, now_ms: u64) {
    let net = world.laptop.screen == Screen::Net;
    let bg = if net { NET_BG } else { BG };
    fill(buf, area, bg);
    header(buf, area, world, hover, hits, bg);
    let body = Rect::new(area.x, area.y + 3, area.width, area.height.saturating_sub(5));
    if net {
        draw_net(buf, body, world, now_ms);
    } else {
        draw_broadcast(buf, body, world, hover, hits);
    }
    let charging = if world.near_outlet() { " ⚡ charging" } else { "" };
    let batt = format!("Battery: {:>3.0}%{charging}", world.laptop.battery);
    let bx = area.right().saturating_sub(batt.chars().count() as u16 + 3);
    put(buf, bx, area.y + 1, &batt, Style::new().fg(if net { NET_FG } else { FG }).bg(bg));
    let hint = if net {
        "type a command, Enter runs it · ↑↓ recall · PgUp/PgDn or wheel: scroll · Tab: broadcast · Esc: leave a test / close"
    } else {
        "↑↓ or click: select · E or click again: play / stop · Tab: NET · Esc or right-click: close (keeps playing)"
    };
    let x = area.x + area.width.saturating_sub(hint.chars().count() as u16) / 2;
    put(buf, x, area.bottom().saturating_sub(2), &ellipsize(hint, area.width.saturating_sub(2) as usize), Style::new().fg(if net { NET_DIM } else { DIM }).bg(bg));
}

/// The title and the two tabs.
fn header(buf: &mut Buffer, area: Rect, world: &World, hover: Hover, hits: &mut Hits, bg: Color) {
    let net = world.laptop.screen == Screen::Net;
    let title = "FANFARE // EMERGENCY TERMINAL";
    put(buf, area.x + 3, area.y + 1, title, Style::new().fg(if net { NET_FG } else { ACCENT }).bg(bg));
    let mut x = area.x + 3 + title.chars().count() as u16 + 4;
    for (i, label) in ["BROADCAST 104.2 MHz", "NET"].into_iter().enumerate() {
        let text = format!(" {label} ");
        let rect = Rect::new(x, area.y + 1, text.chars().count() as u16, 1);
        if rect.right() > area.right() {
            break;
        }
        hits.tabs.push((rect, i));
        let active = (i == 1) == net;
        let style = if active {
            Style::new().fg(Color::Black).bg(if net { NET_FG } else { HIGHLIGHT })
        } else if hovered(hover, rect) {
            Style::new().fg(Color::White).bg(Color::Rgb(50, 60, 80))
        } else {
            Style::new().fg(DIM).bg(bg)
        };
        put(buf, x, area.y + 1, &text, style);
        x += rect.width + 1;
    }
    put(buf, area.x, area.y + 2, &"─".repeat(area.width as usize), Style::new().fg(if net { NET_DIM } else { Color::Rgb(60, 80, 120) }).bg(bg));
}

fn draw_broadcast(buf: &mut Buffer, area: Rect, world: &World, hover: Hover, hits: &mut Hits) {
    let st = |c: Color| Style::new().fg(c).bg(BG);
    // track list on the left, scrolled so the cursor stays visible
    let list_x = area.x + 4;
    let list_w = 34u16;
    let list_top = area.y + 1;
    let rows = area.height.saturating_sub(2) as usize;
    let cursor = world.laptop.cursor;
    let first = cursor.saturating_sub(rows.saturating_sub(1)).min(TRACKS.len().saturating_sub(rows));
    for (row, (i, t)) in TRACKS.iter().enumerate().skip(first).take(rows).enumerate() {
        let selected = i == cursor;
        let playing = world.laptop.playing == Some(i);
        let line = format!("{} {:02} {:<20}{}", if selected { "▶" } else { " " }, t.n, ellipsize(t.name, 20), if playing { " ♪" } else { "" });
        let rect = Rect::new(list_x, list_top + row as u16, list_w, 1);
        hits.tracks.push((rect, i));
        let color = if selected { HIGHLIGHT } else if playing { ACCENT } else { FG };
        let style = if hovered(hover, rect) && !selected { st(color).bg(Color::Rgb(28, 38, 70)) } else { st(color) };
        put(buf, list_x, list_top + row as u16, &line, style);
    }

    // the selected transmission on the right
    let t = &TRACKS[cursor.min(TRACKS.len() - 1)];
    let panel_x = list_x + list_w + 4;
    let panel_w = area.right().saturating_sub(panel_x + 3) as usize;
    // below where a message box would sit, so a message never hides the title
    let panel_y = area.y + 4;
    put(buf, panel_x, panel_y, &format!("TRANSMISSION {:02}: {}", t.n, t.name), st(ACCENT));
    let status = match (world.laptop.playing == Some(cursor), world.laptop.source) {
        (false, _) => "STATUS: [STANDING BY]",
        (true, Some(MusicSource::Carrier)) => "STATUS: [CARRIER TONE ONLY: NO AUDIO DATA]",
        (true, Some(MusicSource::Silent)) => "STATUS: [RECEIVING: NO SOUND DEVICE]",
        (true, _) => "STATUS: [PLAYING AUDIO CARRIER]",
    };
    put(buf, panel_x, panel_y + 2, status, st(HIGHLIGHT));
    let max_lines = area.bottom().saturating_sub(panel_y + 5) as usize;
    for (i, line) in wrap(t.log, panel_w).iter().take(max_lines).enumerate() {
        put(buf, panel_x, panel_y + 4 + i as u16, line, st(FG));
    }
}

/// The NET screen: scrollback and the input line, with the known hosts down the right
/// and whatever test is running in a panel above the prompt.
fn draw_net(buf: &mut Buffer, area: Rect, world: &World, now_ms: u64) {
    let st = |c: Color| Style::new().fg(c).bg(NET_BG);
    let shell = &world.laptop.shell;
    let side_w = if area.width >= 100 { 34 } else { 0 };
    let main_w = area.width.saturating_sub(side_w + 4) as usize;
    let x0 = area.x + 2;

    // the input line at the bottom, a test above it, the scrollback above that
    let mut bottom = area.bottom();
    let prompt = world.net_prompt();
    let line = format!("{prompt}{}", shell.input);
    let shown = if line.chars().count() > main_w { line.chars().skip(line.chars().count() - main_w).collect() } else { line.clone() };
    put(buf, x0, bottom - 1, &shown, st(NET_FG));
    let cx = x0 + shown.chars().count() as u16;
    if cx < area.right() && (now_ms / 500).is_multiple_of(2) {
        put(buf, cx, bottom - 1, "▏", st(NET_FG));
    }
    bottom -= 1;
    if let Some(game) = &shell.game {
        let h = 4u16.min(bottom.saturating_sub(area.y + 2));
        let panel = Rect::new(x0, bottom - h, main_w as u16, h);
        draw_game(buf, panel, game, now_ms);
        bottom -= h;
    }
    if let Some(job) = shell.job {
        let p = job.progress(now_ms);
        let w = (main_w.saturating_sub(24)).clamp(8, 40);
        let filled = (p * w as f64) as usize;
        let text = format!("{} {}{} {:>3.0}%", job.label(), "█".repeat(filled), "░".repeat(w - filled), p * 100.0);
        put(buf, x0, bottom - 1, &ellipsize(&text, main_w), st(NET_HOT));
        bottom -= 1;
    }
    // scrollback: wrapped, the newest at the bottom, scrolled back by `scroll` lines
    let rows = bottom.saturating_sub(area.y) as usize;
    let mut wrapped: Vec<(String, Color)> = Vec::new();
    for l in &shell.lines {
        let color = if l.starts_with("net> ") || l.contains("$ ") && l.starts_with(char::is_alphabetic) && !l.starts_with("  ") {
            NET_HOT
        } else if l.starts_with("---") || l.starts_with("  ") {
            NET_DIM
        } else if l.contains("denied") || l.contains("refused") || l.contains("miss") || l.contains("Wrong") {
            NET_BAD
        } else {
            NET_FG
        };
        // lines that fit keep their spacing (the tables line up); long ones wrap
        if l.chars().count() <= main_w {
            wrapped.push((l.clone(), color));
        } else {
            for w in wrap(l, main_w) {
                wrapped.push((w, color));
            }
        }
    }
    let end = wrapped.len().saturating_sub(shell.scroll.min(wrapped.len()));
    let start = end.saturating_sub(rows);
    for (i, (l, color)) in wrapped[start..end].iter().enumerate() {
        put(buf, x0, area.y + i as u16, l, st(*color));
    }
    if shell.scroll > 0 {
        put(buf, x0, area.y, &format!("── {} lines below ──", end.min(shell.scroll)), st(NET_HOT));
    }

    // the side panel: what the scan found, the session, the locks once on the controller
    if side_w == 0 {
        return;
    }
    let sx = area.right() - side_w;
    for y in area.y..area.bottom() {
        put(buf, sx - 1, y, "│", st(NET_DIM));
    }
    let mut y = area.y;
    put(buf, sx + 1, y, "HOSTS", st(NET_DIM));
    y += 1;
    let found: Vec<_> = world.net.hosts.iter().filter(|h| h.found).collect();
    if found.is_empty() {
        put(buf, sx + 1, y, "(scan)", st(NET_DIM));
        y += 1;
    }
    for h in found {
        if y >= area.bottom() {
            return;
        }
        let mark = if h.lockout_until_ms > now_ms {
            format!("✗ {}s", (h.lockout_until_ms - now_ms).div_ceil(1000))
        } else if h.owned {
            "✓".into()
        } else {
            String::new()
        };
        let text = format!("{:<11} {:<13} {mark}", h.addr(), h.name);
        let color = if shell.session.is_some_and(|s| world.net.hosts[s].ip == h.ip) { NET_HOT } else if h.owned { NET_FG } else { NET_DIM };
        put(buf, sx + 1, y, &ellipsize(&text, side_w as usize - 2), st(color));
        y += 1;
    }
    y += 1;
    if y + 1 < area.bottom() {
        put(buf, sx + 1, y, "SESSION", st(NET_DIM));
        let s = match shell.session {
            Some(i) => format!("{}@{}", world.net.hosts[i].account.as_ref().map_or("guest", |a| a.user), world.net.hosts[i].name),
            None => "none".into(),
        };
        put(buf, sx + 1, y + 1, &s, st(NET_FG));
        y += 3;
    }
    let locks: Vec<_> = world.doors.iter().filter_map(|d| d.net_id.map(|id| (id, d.lock.is_none()))).collect();
    if y + 1 < area.bottom() && world.net.hosts.get(4).is_some_and(|h| h.owned) {
        put(buf, sx + 1, y, "LOCKS", st(NET_DIM));
        y += 1;
        for (id, released) in locks {
            if y >= area.bottom() {
                break;
            }
            let (text, color) = if released { (format!("{id:<7} released"), NET_FG) } else { (format!("{id:<7} locked"), NET_BAD) };
            put(buf, sx + 1, y, &text, st(color));
            y += 1;
        }
    }
}

/// The test in progress: the sweeping cursor of the override, or the relay code.
fn draw_game(buf: &mut Buffer, panel: Rect, game: &Minigame, now_ms: u64) {
    let st = |c: Color| Style::new().fg(c).bg(NET_BG);
    match game {
        Minigame::Timing(t) => {
            let w = (panel.width as usize).saturating_sub(12).clamp(10, 60);
            let cursor = (t.cursor(now_ms) * (w - 1) as f64).round() as usize;
            let lo = ((t.target - t.width / 2.0) * (w - 1) as f64).round().max(0.0) as usize;
            let hi = (((t.target + t.width / 2.0) * (w - 1) as f64).round() as usize).min(w - 1);
            let mut bar = String::new();
            for i in 0..w {
                bar.push(if i == cursor { '█' } else if (lo..=hi).contains(&i) { '▓' } else { '·' });
            }
            let flash = t.last.filter(|(_, at)| now_ms.saturating_sub(*at) < 350).map(|(hit, _)| hit);
            let color = match flash {
                Some(true) => NET_FG,
                Some(false) => NET_BAD,
                None => NET_HOT,
            };
            put(buf, panel.x, panel.y, "OVERRIDE  press Enter with the cursor in the window", st(NET_DIM));
            put(buf, panel.x, panel.y + 1, &format!("[{bar}]"), st(color));
            let hits: String = (0..t.need).map(|i| if i < t.hits { '●' } else { '○' }).collect();
            let misses: String = (0..3).map(|i| if i < t.misses { '✗' } else { '·' }).collect();
            put(buf, panel.x, panel.y + 2, &format!("hits {hits}   misses {misses}"), st(NET_FG));
        }
        Minigame::Sequence(s) => {
            put(buf, panel.x, panel.y, &format!("RELAY PANEL  {}  code {} of {}", s.door_id, s.round, s.rounds), st(NET_DIM));
            if s.showing(now_ms) {
                let left = (s.shown_until_ms - now_ms) as f64 / 1000.0;
                let spaced: String = s.code.chars().flat_map(|c| [c, ' ']).collect();
                put(buf, panel.x, panel.y + 1, &format!("  {spaced}    (shown for {left:.1} s more)"), st(NET_HOT));
            } else {
                put(buf, panel.x, panel.y + 1, "  ▪ ▪ ▪ ▪    type the code and press Enter", st(NET_FG));
            }
            let misses: String = (0..2).map(|i| if i < s.misses { '✗' } else { '·' }).collect();
            put(buf, panel.x, panel.y + 2, &format!("misses {misses}"), st(NET_FG));
        }
    }
}
