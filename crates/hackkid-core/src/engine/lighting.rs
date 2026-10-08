//! Light: a lightmap at half-metre resolution.
//!
//! What each light reaches is worked out once (walls block it, doors let some through);
//! each frame only the part that changes is redone: daylight for the hour, street lamps,
//! and the tubes that flicker. Dead tubes leave their rooms dark.

use crate::engine::map::Map;
use crate::engine::sky::Env;

/// Light cells per metre.
pub const LM_RES: f64 = 2.0;
/// The faint light that's always there indoors, so the dark isn't pure black.
const AMBIENT: f32 = 0.07;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LightKind {
    /// A ceiling tube; its index in the world's fluorescents. Flickering ones are redone
    /// each frame, steady ones baked.
    Tube { idx: usize, flickers: bool },
    /// Daylight through a window, strength from the hour.
    Window,
    /// A sodium street lamp, on at night.
    Lamp,
    /// A small steady glow: exit signs, vending machines.
    Glow,
}

#[derive(Debug, Clone, Copy)]
pub struct LightSource {
    pub x: f64,
    pub y: f64,
    pub kind: LightKind,
    pub color: [f32; 3],
    pub intensity: f32,
    pub radius: f32,
}

pub struct LightMap {
    pub w: usize,
    pub h: usize,
    sources: Vec<LightSource>,
    /// Per source: which cells it reaches, and how strongly.
    reach: Vec<Vec<(u32, f32)>>,
    window: Vec<f32>,
    lamp: Vec<[f32; 3]>,
    sky: Vec<f32>,
    ao: Vec<f32>,
    steady: Vec<[f32; 3]>,
}

/// The light everywhere, for one frame.
#[derive(Default)]
pub struct LightField {
    pub w: usize,
    pub h: usize,
    cells: Vec<[f32; 3]>,
}

impl LightField {
    /// Light at a world point, smoothly interpolated between cells.
    #[inline]
    pub fn sample(&self, x: f64, y: f64) -> [f32; 3] {
        if self.cells.is_empty() {
            return [1.0; 3];
        }
        let gx = (x * LM_RES - 0.5).clamp(0.0, (self.w - 1) as f64);
        let gy = (y * LM_RES - 0.5).clamp(0.0, (self.h - 1) as f64);
        let (ix, iy) = (gx as usize, gy as usize);
        let (fx, fy) = ((gx - ix as f64) as f32, (gy - iy as f64) as f32);
        let ix1 = (ix + 1).min(self.w - 1);
        let iy1 = (iy + 1).min(self.h - 1);
        let c = |x: usize, y: usize| self.cells[y * self.w + x];
        let (a, b, c0, d) = (c(ix, iy), c(ix1, iy), c(ix, iy1), c(ix1, iy1));
        let mut out = [0.0; 3];
        for k in 0..3 {
            let top = a[k] + (b[k] - a[k]) * fx;
            let bot = c0[k] + (d[k] - c0[k]) * fx;
            out[k] = top + (bot - top) * fy;
        }
        out
    }
}

impl LightMap {
    pub fn build(map: &Map, sources: Vec<LightSource>) -> Self {
        let w = (map.width as f64 * LM_RES) as usize;
        let h = (map.height as f64 * LM_RES) as usize;
        let n = w * h;
        let center = |i: usize| (((i % w) as f64 + 0.5) / LM_RES, ((i / w) as f64 + 0.5) / LM_RES);

        let mut sky = vec![0.0f32; n];
        let mut ao = vec![1.0f32; n];
        for (i, (s, a)) in sky.iter_mut().zip(ao.iter_mut()).enumerate() {
            let (x, y) = center(i);
            let tile = map.at(x, y);
            if tile.is_outdoor() || matches!(tile, crate::engine::map::Tile::Hedge) {
                *s = 1.0;
            }
            // corners and wall bases get less light
            let blocked = [(0.6, 0.0), (-0.6, 0.0), (0.0, 0.6), (0.0, -0.6), (0.45, 0.45), (-0.45, 0.45), (0.45, -0.45), (-0.45, -0.45)]
                .iter()
                .filter(|(dx, dy)| map.at(x + dx, y + dy).blocks_movement() && !map.at(x + dx, y + dy).is_any_door())
                .count();
            *a = 1.0 - 0.06 * blocked as f32;
        }

        let mut reach = Vec::with_capacity(sources.len());
        let mut window = vec![0.0f32; n];
        let mut lamp = vec![[0.0f32; 3]; n];
        let mut steady = vec![[0.0f32; 3]; n];
        for s in &sources {
            let cells = light_reach(map, s, w, h);
            match s.kind {
                LightKind::Window => {
                    for &(i, k) in &cells {
                        window[i as usize] += k;
                    }
                }
                LightKind::Lamp => {
                    for &(i, k) in &cells {
                        for (acc, col) in lamp[i as usize].iter_mut().zip(s.color) {
                            *acc += k * col;
                        }
                    }
                }
                LightKind::Glow | LightKind::Tube { flickers: false, .. } => {
                    for &(i, k) in &cells {
                        for (acc, col) in steady[i as usize].iter_mut().zip(s.color) {
                            *acc += k * col;
                        }
                    }
                }
                LightKind::Tube { flickers: true, .. } => {}
            }
            let keep = matches!(s.kind, LightKind::Tube { flickers: true, .. });
            reach.push(if keep { cells } else { Vec::new() });
        }
        Self { w, h, sources, reach, window, lamp, sky, ao, steady }
    }

    /// The light field for this moment. `tube` gives a flickering tube's current brightness.
    pub fn field(&self, out: &mut LightField, env: &Env, tube: impl Fn(usize) -> f32) {
        let n = self.w * self.h;
        out.w = self.w;
        out.h = self.h;
        out.cells.resize(n, [0.0; 3]);
        let day = env.daylight();
        let lamps = env.lamps_on() * 1.25;
        for i in 0..n {
            let (s, win, l, st, ao) = (self.sky[i], self.window[i], self.lamp[i], self.steady[i], self.ao[i]);
            let mut c = [0.0f32; 3];
            for k in 0..3 {
                c[k] = (st[k] + day[k] * (win + s) + l[k] * lamps + AMBIENT * (1.0 - s)) * ao;
            }
            out.cells[i] = c;
        }
        let flickering = self.sources.iter().zip(&self.reach).filter(|(s, _)| matches!(s.kind, LightKind::Tube { flickers: true, .. }));
        for (src, cells) in flickering {
            let LightKind::Tube { idx, .. } = src.kind else { continue };
            let b = tube(idx);
            if b <= 0.01 {
                continue;
            }
            for &(i, k) in cells {
                let i = i as usize;
                let ao = self.ao[i];
                for c in 0..3 {
                    out.cells[i][c] += k * src.color[c] * b * ao;
                }
            }
        }
        // roll off the highlights so overlapping lights don't burn out to white
        for c in out.cells.iter_mut() {
            for v in c.iter_mut() {
                *v = *v * 1.3 / (1.0 + 0.3 * *v);
            }
        }
    }
}

/// Which cells a light reaches and how strongly: a soft falloff, cut off by walls.
fn light_reach(map: &Map, s: &LightSource, w: usize, h: usize) -> Vec<(u32, f32)> {
    let r = s.radius as f64;
    let (x0, x1) = (((s.x - r) * LM_RES).max(0.0) as usize, (((s.x + r) * LM_RES) as usize).min(w - 1));
    let (y0, y1) = (((s.y - r) * LM_RES).max(0.0) as usize, (((s.y + r) * LM_RES) as usize).min(h - 1));
    let mut out = Vec::new();
    for gy in y0..=y1 {
        for gx in x0..=x1 {
            let (cx, cy) = ((gx as f64 + 0.5) / LM_RES, (gy as f64 + 0.5) / LM_RES);
            let d = ((cx - s.x).powi(2) + (cy - s.y).powi(2)).sqrt();
            if d >= r {
                continue;
            }
            // broad and soft, as from a ceiling fitting: rooms with a few tubes light evenly
            let t = 1.0 - d / r;
            let falloff = t.powf(1.5) / (1.0 + (d / 3.0).powi(2));
            let pass = transmission(map, s.x, s.y, cx, cy);
            if pass <= 0.0 {
                continue;
            }
            out.push(((gy * w + gx) as u32, (falloff as f32) * s.intensity * pass));
        }
    }
    out
}

/// How much light gets from one point to another: 0 behind a wall, less through a door.
fn transmission(map: &Map, ax: f64, ay: f64, bx: f64, by: f64) -> f32 {
    let (tx, ty) = (bx.floor() as i64, by.floor() as i64);
    let (mut x, mut y) = (ax.floor() as i64, ay.floor() as i64);
    let (dx, dy) = (bx - ax, by - ay);
    let step_x = if dx > 0.0 { 1 } else { -1 };
    let step_y = if dy > 0.0 { 1 } else { -1 };
    let t_dx = if dx == 0.0 { f64::INFINITY } else { 1.0 / dx.abs() };
    let t_dy = if dy == 0.0 { f64::INFINITY } else { 1.0 / dy.abs() };
    let mut t_x = if dx > 0.0 { (x as f64 + 1.0 - ax) * t_dx } else { (ax - x as f64) * t_dx };
    let mut t_y = if dy > 0.0 { (y as f64 + 1.0 - ay) * t_dy } else { (ay - y as f64) * t_dy };
    let mut pass = 1.0f32;
    for _ in 0..64 {
        if x == tx && y == ty {
            return pass;
        }
        if t_x < t_y {
            x += step_x;
            t_x += t_dx;
        } else {
            y += step_y;
            t_y += t_dy;
        }
        if x == tx && y == ty {
            return pass;
        }
        if x < 0 || y < 0 {
            return 0.0;
        }
        let tile = map.get(x as usize, y as usize);
        if tile.is_any_door() {
            pass *= 0.3;
        } else if tile.blocks_sight() {
            return 0.0;
        }
    }
    pass
}
