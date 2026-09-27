use crossterm::{
    cursor, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use std::io::{self, Write};

pub const SIDE_PAD: u16 = 0;
pub const RENDER_START: u16 = 2;
pub const OVERLAY_ROW: u16 = 3;

pub fn pixel(frame: &[u8], w: usize, x: usize, y: usize) -> (u8, u8, u8) {
    let i = (y * w + x) * 4;
    (frame[i], frame[i + 1], frame[i + 2])
}

pub fn render_top_border<W: Write>(out: &mut W, view_w: usize, status: &str) -> io::Result<()> {
    queue!(out,
        cursor::MoveTo(0, 0),
        terminal::Clear(ClearType::CurrentLine),
        cursor::MoveTo(SIDE_PAD + 2, 0),
        SetForegroundColor(Color::Rgb { r: 235, g: 220, b: 140 }),
        Print(status),
        ResetColor,
        cursor::MoveTo(0, 1),
        terminal::Clear(ClearType::CurrentLine),
        cursor::MoveTo(SIDE_PAD, 1),
        SetForegroundColor(Color::Rgb { r: 120, g: 130, b: 150 }),
        Print("─".repeat(view_w)),
        ResetColor,
    )
}

pub fn blit_frame<W: Write>(
    out: &mut W,
    frame: &[u8],
    view_w: usize,
    render_rows: usize,
    start_row: u16,
) -> io::Result<()> {
    for ty in 0..render_rows {
        queue!(out,
            cursor::MoveTo(0, start_row + ty as u16),
            terminal::Clear(ClearType::CurrentLine),
            cursor::MoveTo(SIDE_PAD, start_row + ty as u16),
        )?;
        for tx in 0..view_w {
            let (tr, tg, tb) = pixel(frame, view_w, tx, ty * 2);
            let (br, bg, bb) = pixel(frame, view_w, tx, ty * 2 + 1);
            queue!(
                out,
                SetBackgroundColor(Color::Rgb { r: tr, g: tg, b: tb }),
                SetForegroundColor(Color::Rgb { r: br, g: bg, b: bb }),
                Print('▄'),
            )?;
        }
    }
    Ok(())
}
