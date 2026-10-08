//! Things painted or fixed flat onto walls: signs, outlets, clocks, extinguishers,
//! posters, mirrors, elevator doors, the lettering on the facade.

use std::collections::HashMap;

use crate::engine::map::Map;
use crate::engine::textures::{CLEAR, FLAGS, hash, rgb};

/// Texels marked as giving their own light (exit signs).
pub const EMIT: u32 = 3 << 24;

/// Which side of a wall tile: the side facing north, east, south or west.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Face {
    North,
    East,
    South,
    West,
}

impl Face {
    /// The direction the face looks towards.
    pub fn normal(self) -> (i32, i32) {
        match self {
            Face::North => (0, -1),
            Face::East => (1, 0),
            Face::South => (0, 1),
            Face::West => (-1, 0),
        }
    }

    /// Whether, seen from in front, left-to-right runs with the world axis along the face.
    pub fn runs_forward(self) -> bool {
        matches!(self, Face::South | Face::West)
    }
}

pub struct Image {
    pub w: usize,
    pub h: usize,
    /// Rows from the top.
    px: Vec<u32>,
}

impl Image {
    fn new(w: usize, h: usize, f: impl Fn(i32, i32) -> u32) -> Image {
        let mut px = Vec::with_capacity(w * h);
        for y in 0..h {
            for x in 0..w {
                px.push(f(x as i32, y as i32));
            }
        }
        Image { w, h, px }
    }

    /// `u`, `v` 0..1 from the top-left.
    #[inline]
    pub fn at(&self, u: f64, v: f64) -> u32 {
        let x = ((u * self.w as f64) as usize).min(self.w - 1);
        let y = ((v * self.h as f64) as usize).min(self.h - 1);
        self.px[y * self.w + x]
    }
}

/// One piece of a decal on one wall tile face.
#[derive(Debug, Clone, Copy)]
pub struct Decal {
    /// World coordinate range along the face that this piece covers.
    pub s0: f64,
    pub s1: f64,
    /// The whole decal's range, to map onto the image.
    pub full0: f64,
    pub full1: f64,
    pub flip: bool,
    /// Height range above the floor, metres.
    pub v0: f64,
    pub v1: f64,
    pub image: usize,
    /// Draw clock hands showing the time.
    pub clock: bool,
}

impl Decal {
    /// Image coordinates for world position `s` along the face and height `v`.
    #[inline]
    pub fn uv(&self, s: f64, v: f64) -> (f64, f64) {
        let mut u = (s - self.full0) / (self.full1 - self.full0);
        if self.flip {
            u = 1.0 - u;
        }
        (u, (self.v1 - v) / (self.v1 - self.v0))
    }
}

#[derive(Default)]
pub struct Decals {
    pub images: Vec<Image>,
    by_face: HashMap<(u16, u16, Face), Vec<Decal>>,
}

impl Decals {
    pub fn on(&self, tx: usize, ty: usize, face: Face) -> &[Decal] {
        self.by_face.get(&(tx as u16, ty as u16, face)).map_or(&[], |v| v.as_slice())
    }

    pub fn add_image(&mut self, img: Image) -> usize {
        self.images.push(img);
        self.images.len() - 1
    }

    /// Fixes an image to the wall nearest to floor point (`x`, `y`), centred there,
    /// `width` metres wide, from `v0` to `v0 + height` above the floor. Returns false if
    /// no wall is within 0.4 m.
    #[allow(clippy::too_many_arguments)]
    pub fn place(&mut self, map: &Map, x: f64, y: f64, width: f64, v0: f64, height: f64, image: usize, clock: bool) -> bool {
        let Some((face, line)) = nearest_face(map, x, y) else { return false };
        let along = match face {
            Face::West | Face::East => y,
            _ => x,
        };
        let (full0, full1) = (along - width / 2.0, along + width / 2.0);
        let flip = !face.runs_forward();
        let mut s = full0.floor();
        while s < full1 {
            let (s0, s1) = (s.max(full0), (s + 1.0).min(full1));
            let cell = s as i64;
            let (tx, ty) = match face {
                Face::West | Face::East => (line, cell),
                _ => (cell, line),
            };
            if tx >= 0 && ty >= 0 && map.get(tx as usize, ty as usize).blocks_sight() {
                self.by_face.entry((tx as u16, ty as u16, face)).or_default().push(Decal {
                    s0, s1, full0, full1, flip, v0, v1: v0 + height, image, clock,
                });
            }
            s += 1.0;
        }
        true
    }
}

/// The wall face closest to a floor point: which face, and the wall tiles' row or column.
fn nearest_face(map: &Map, x: f64, y: f64) -> Option<(Face, i64)> {
    let candidates = [
        (Face::West, x.floor() + 1.0 - x, (x + 0.45).floor() as i64, 1.0, 0.0),
        (Face::East, x - x.floor(), (x - 0.45).floor() as i64, -1.0, 0.0),
        (Face::North, y.floor() + 1.0 - y, (y + 0.45).floor() as i64, 0.0, 1.0),
        (Face::South, y - y.floor(), (y - 0.45).floor() as i64, 0.0, -1.0),
    ];
    candidates
        .iter()
        .filter(|(_, d, _, dx, dy)| *d < 0.4 && map.at(x + dx * (d + 0.05), y + dy * (d + 0.05)).blocks_sight())
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|&(f, _, line, _, _)| (f, line))
}

// ---------------------------------------------------------------- a 3 × 5 pixel font

fn glyph(c: char) -> [u8; 5] {
    match c.to_ascii_uppercase() {
        'A' => [2, 5, 7, 5, 5], 'B' => [6, 5, 6, 5, 6], 'C' => [3, 4, 4, 4, 3], 'D' => [6, 5, 5, 5, 6],
        'E' => [7, 4, 6, 4, 7], 'F' => [7, 4, 6, 4, 4], 'G' => [3, 4, 5, 5, 3], 'H' => [5, 5, 7, 5, 5],
        'I' => [7, 2, 2, 2, 7], 'J' => [1, 1, 1, 5, 2], 'K' => [5, 5, 6, 5, 5], 'L' => [4, 4, 4, 4, 7],
        'M' => [5, 7, 7, 5, 5], 'N' => [6, 5, 5, 5, 5], 'O' => [2, 5, 5, 5, 2], 'P' => [6, 5, 6, 4, 4],
        'Q' => [2, 5, 5, 6, 3], 'R' => [6, 5, 6, 5, 5], 'S' => [3, 4, 2, 1, 6], 'T' => [7, 2, 2, 2, 2],
        'U' => [5, 5, 5, 5, 7], 'V' => [5, 5, 5, 5, 2], 'W' => [5, 5, 7, 7, 5], 'X' => [5, 5, 2, 5, 5],
        'Y' => [5, 5, 2, 2, 2], 'Z' => [7, 1, 2, 4, 7],
        '0' => [7, 5, 5, 5, 7], '1' => [2, 6, 2, 2, 7], '2' => [6, 1, 2, 4, 7], '3' => [6, 1, 2, 1, 6],
        '4' => [5, 5, 7, 1, 1], '5' => [7, 4, 6, 1, 6], '6' => [3, 4, 7, 5, 7], '7' => [7, 1, 1, 2, 2],
        '8' => [7, 5, 7, 5, 7], '9' => [7, 5, 7, 1, 6],
        '-' => [0, 0, 7, 0, 0], '.' => [0, 0, 0, 0, 2], '<' => [1, 2, 4, 2, 1], '>' => [4, 2, 1, 2, 4],
        '^' => [2, 7, 2, 2, 2], '!' => [2, 2, 2, 0, 2], '/' => [1, 1, 2, 4, 4], '+' => [0, 2, 7, 2, 0],
        '&' => [2, 5, 2, 5, 3], '_' => [2, 2, 2, 7, 2],
        _ => [0; 5],
    }
}

/// True where `text`, drawn at `scale` texels per font pixel from (x0, y0), has ink.
fn ink(text: &str, x: i32, y: i32, x0: i32, y0: i32, scale: i32) -> bool {
    let (lx, ly) = (x - x0, y - y0);
    if lx < 0 || ly < 0 || ly >= 5 * scale {
        return false;
    }
    let (ci, gx) = (lx / (4 * scale), (lx % (4 * scale)) / scale);
    if gx >= 3 {
        return false;
    }
    let Some(c) = text.chars().nth(ci as usize) else { return false };
    glyph(c)[(ly / scale) as usize] & (4 >> gx) != 0
}

fn text_width(text: &str, scale: i32) -> i32 {
    (text.chars().count() as i32 * 4 - 1) * scale
}

// ---------------------------------------------------------------- the images

#[derive(Debug, Clone, Copy)]
pub enum SignStyle {
    /// Blue plaque by a door: room names and numbers.
    Plaque,
    /// Dark green band over a corridor: directions.
    Wayfinding,
    /// Red and white: keep out.
    Warning,
    /// Green, lit: the way out.
    Exit,
}

/// A sign with one line of text. Returns the image and its size in metres.
pub fn sign(text: &str, style: SignStyle) -> (Image, f64, f64) {
    let scale = 1;
    let (bg, fg, edge) = match style {
        SignStyle::Plaque => (rgb(36, 74, 128), rgb(236, 240, 244), rgb(200, 206, 214)),
        SignStyle::Wayfinding => (rgb(24, 70, 60), rgb(240, 240, 228), rgb(16, 46, 40)),
        SignStyle::Warning => (rgb(236, 236, 230), rgb(176, 30, 30), rgb(176, 30, 30)),
        SignStyle::Exit => (rgb(30, 150, 70) | EMIT, rgb(240, 255, 240) | EMIT, rgb(20, 100, 50) | EMIT),
    };
    let (w, h) = (text_width(text, scale) + 6, 5 * scale + 6);
    let img = Image::new(w as usize, h as usize, |x, y| {
        if x == 0 || y == 0 || x == w - 1 || y == h - 1 {
            edge
        } else if ink(text, x, y, 3, 3, scale) {
            fg
        } else {
            bg
        }
    });
    // decal text is 40 texels per metre: legible up close, as real signage is
    (img, w as f64 / 40.0, h as f64 / 40.0)
}

/// Big raised letters for the facade, `height` metres tall.
pub fn letters(text: &str, color: u32) -> (Image, f64) {
    let scale = 2;
    let (w, h) = (text_width(text, scale), 5 * scale);
    let img = Image::new(w as usize, h as usize, |x, y| {
        if ink(text, x, y, 0, 0, scale) {
            if (y % scale) == 0 { color } else { super_dark(color) }
        } else {
            CLEAR
        }
    });
    (img, w as f64 / h as f64)
}

fn super_dark(c: u32) -> u32 {
    let f = |v: u32| ((v & 255) * 3 / 4) as u8;
    rgb(f(c >> 16), f(c >> 8), f(c))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fixture {
    Outlet,
    Switch,
    Extinguisher,
    Clock,
    HandWashPoster,
    QuietPoster,
    Whiteboard,
    Aed,
    Elevator,
    Mirror,
    MedCabinet,
    Bulletin,
    Sanitizer,
    Painting,
    Cross,
}

/// The image for a wall fixture, and its size in metres.
pub fn fixture(f: Fixture) -> (Image, f64, f64) {
    let (w, h, wm, hm) = match f {
        Fixture::Outlet => (4, 5, 0.08, 0.11),
        Fixture::Switch => (4, 5, 0.08, 0.11),
        Fixture::Extinguisher => (6, 14, 0.18, 0.55),
        Fixture::Clock => (12, 12, 0.32, 0.32),
        Fixture::HandWashPoster | Fixture::QuietPoster => (14, 20, 0.45, 0.62),
        Fixture::Whiteboard => (40, 24, 1.4, 0.85),
        Fixture::Aed => (12, 13, 0.36, 0.42),
        Fixture::Elevator => (24, 44, 1.2, 2.2),
        Fixture::Mirror => (14, 18, 0.5, 0.65),
        Fixture::MedCabinet => (14, 16, 0.5, 0.6),
        Fixture::Bulletin => (30, 20, 1.0, 0.7),
        Fixture::Sanitizer => (4, 8, 0.1, 0.22),
        Fixture::Painting => (24, 16, 0.9, 0.6),
        Fixture::Cross => (12, 16, 0.5, 0.7),
    };
    let img = Image::new(w, h, |x, y| fixture_px(f, x, y, w as i32, h as i32));
    (img, wm, hm)
}

fn fixture_px(f: Fixture, x: i32, y: i32, w: i32, h: i32) -> u32 {
    let edge = x == 0 || y == 0 || x == w - 1 || y == h - 1;
    match f {
        Fixture::Outlet => {
            if (x == 1 || x == 2) && (y == 1 || y == 3) { rgb(40, 40, 40) } else { rgb(236, 234, 226) }
        }
        Fixture::Switch => {
            if (1..=2).contains(&x) && (1..=3).contains(&y) { rgb(210, 208, 200) } else { rgb(236, 234, 226) }
        }
        Fixture::Extinguisher => {
            if y <= 1 {
                return if (2..=3).contains(&x) { rgb(40, 40, 40) } else { CLEAR }; // valve
            }
            if x == 0 {
                return if y > 3 && y < 12 { rgb(30, 30, 30) } else { CLEAR }; // hose
            }
            if (6..=8).contains(&y) && x > 1 { return rgb(232, 230, 220); } // label
            let shade = if x == 1 { 30 } else if x == w - 1 { -30 } else { 0 };
            let c = rgb(196, 30, 28);
            lift(c, shade)
        }
        Fixture::Clock => {
            let d = ((x as f32 - 5.5).powi(2) + (y as f32 - 5.5).powi(2)).sqrt();
            if d > 6.0 { CLEAR } else if d > 5.0 { rgb(40, 40, 44) } else if (x == 5 || x == 6) && (y == 1 || y == 10) || (y == 5 || y == 6) && (x == 1 || x == 10) { rgb(60, 60, 60) } else { rgb(242, 242, 236) }
        }
        Fixture::HandWashPoster => {
            if edge { return rgb(200, 200, 200); }
            if y <= 4 { return if (2..12).contains(&x) && y == 2 { rgb(240, 240, 240) } else { rgb(30, 110, 180) }; }
            let cell = (x - 1) / 6 + (y - 5) / 7 * 2;
            if (x - 1) % 6 == 0 || (y - 5) % 7 == 0 { rgb(250, 250, 250) } else { [rgb(150, 200, 230), rgb(230, 190, 150), rgb(160, 220, 180), rgb(240, 220, 150)][(cell as usize) % 4] }
        }
        Fixture::QuietPoster => {
            if edge { return rgb(60, 60, 70); }
            if ink("SHH", x, y, 2, 4, 1) { rgb(250, 250, 250) } else if y > 12 && y < 15 && x > 2 && x < w - 3 { rgb(200, 200, 210) } else { rgb(90, 60, 130) }
        }
        Fixture::Whiteboard => {
            if edge || y == h - 2 { return rgb(170, 174, 178); }
            let scribble = hash(x / 2, y, 501) > 0.8 && y % 4 == 2 && x % 13 < 9;
            if scribble { if x < w / 2 { rgb(40, 70, 170) } else { rgb(180, 40, 40) } } else { rgb(244, 246, 246) }
        }
        Fixture::Aed => {
            if edge { return rgb(20, 120, 70); }
            let heart = ((x - 6) as f32).abs() + ((y - 6) as f32 * 1.2) < 4.0 && y > 3;
            if heart { rgb(220, 40, 40) } else if y >= 10 { rgb(20, 140, 80) } else { rgb(240, 244, 240) }
        }
        Fixture::Elevator => {
            if y <= 4 {
                return if (1..=3).contains(&y) && (10..=13).contains(&x) { rgb(230, 120, 30) | EMIT } else { rgb(60, 62, 66) }; // floor indicator
            }
            if x <= 1 || x >= w - 2 || y == 5 { return rgb(110, 114, 118); }
            if x == w / 2 || x == w / 2 - 1 { return rgb(70, 72, 76); }
            lift(rgb(176, 180, 184), ((hash(x, y * 3, 503) - 0.5) * 18.0) as i32)
        }
        Fixture::Mirror => {
            if edge { return rgb(170, 174, 178); }
            let streak = (x + y) % 7 == 0 || (x + y) % 7 == 1;
            if streak { rgb(200, 214, 224) } else { rgb(150, 168, 182) }
        }
        Fixture::MedCabinet => {
            if edge { return rgb(170, 172, 176); }
            if x == w / 2 { return rgb(190, 192, 196); }
            let cross = ((x - 3).abs() <= 1 && (3..=7).contains(&y)) || ((y - 5).abs() <= 1 && (1..=5).contains(&x));
            if cross { rgb(200, 40, 40) } else { rgb(236, 238, 238) }
        }
        Fixture::Bulletin => {
            if edge { return rgb(110, 76, 46); }
            let paper = hash(x / 6, y / 5, 509);
            if x % 6 != 0 && y % 5 != 0 && paper > 0.4 {
                return if paper > 0.85 { rgb(250, 230, 120) } else if paper > 0.7 { rgb(200, 230, 250) } else { rgb(246, 246, 240) };
            }
            lift(rgb(176, 136, 90), ((hash(x, y, 511) - 0.5) * 20.0) as i32)
        }
        Fixture::Sanitizer => {
            if y >= h - 2 { return if x == 1 || x == 2 { rgb(60, 60, 64) } else { CLEAR }; }
            if (2..=4).contains(&y) { rgb(60, 130, 200) } else { rgb(236, 236, 236) }
        }
        Fixture::Painting => {
            if edge || x == 1 || y == 1 || x == w - 2 || y == h - 2 { return rgb(120, 86, 40); }
            let horizon = 9 + (((x as f32) * 0.5).sin() * 1.5) as i32;
            if y < horizon - 4 { rgb(150, 190, 220) } else if y < horizon { rgb(110, 140, 100) } else { rgb(80, 110, 70) }
        }
        Fixture::Cross => {
            let wood = rgb(130, 90, 50);
            if (5..=6).contains(&x) || ((4..=5).contains(&y) && (1..=10).contains(&x)) { wood } else { CLEAR }
        }
    }
}

fn lift(c: u32, d: i32) -> u32 {
    let f = |v: u32| (v as i32 + d).clamp(0, 255) as u8;
    rgb(f((c >> 16) & 255), f((c >> 8) & 255), f(c & 255)) | (c & FLAGS)
}
