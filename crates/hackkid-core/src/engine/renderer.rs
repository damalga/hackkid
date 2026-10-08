//! The first-person view, drawn column by column into an RGBA frame.
//!
//! Walls come from the raycaster, floor and ceiling from casting each row onto their
//! planes, furniture from intersecting each column's ray with boxes, then flat sprites.
//! A depth buffer sorts it all out. Everything is lit from the light map, then given a
//! touch of distance haze and an ordered dither, which keeps the pixel-art grain.

use crate::engine::boxes::{BoxFace, BoxInput};
use crate::engine::camera::{DOOR_H, EYE_H, TEX_RES, View, WALL_H};
use crate::engine::decals::{Decals, EMIT, Face};
use crate::engine::lighting::LightField;
use crate::engine::map::{Map, Material, Tile};
use crate::engine::raycaster::{self, Side};
use crate::engine::sky::Env;
use crate::engine::sprites::SpriteInput;
use crate::engine::textures::{self, CLEAR, CeilingTile, DoorStyle, FLAGS, GLASS, WallFinish, channels, rgb, textures};
use crate::objects::{Door, Fluorescent};

/// Where the eye is and where it looks.
#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub x: f64,
    pub y: f64,
    pub dir_x: f64,
    pub dir_y: f64,
    pub plane_x: f64,
    pub plane_y: f64,
}

/// A ceiling light fitting this frame: a 0.6 × 1.2 m panel.
#[derive(Debug, Clone, Copy)]
pub struct Panel {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
    /// 0 for a dead tube, up to 1.
    pub glow: f32,
}

/// The light fittings, indexed by map tile for quick lookup.
#[derive(Default)]
pub struct Panels {
    w: usize,
    grid: Vec<u16>,
    list: Vec<Panel>,
}

impl Panels {
    pub fn rebuild(&mut self, map_w: usize, map_h: usize, tubes: &[Fluorescent], now_ms: u64) {
        self.w = map_w;
        self.grid.clear();
        self.grid.resize(map_w * map_h, 0);
        self.list.clear();
        for f in tubes {
            let ((ax, ay), (bx, by)) = f.endpoints();
            let (hx, hy) = if ay == by { (0.0, 0.3) } else { (0.3, 0.0) };
            let p = Panel { x0: ax.min(bx) - hx, y0: ay.min(by) - hy, x1: ax.max(bx) + hx, y1: ay.max(by) + hy, glow: f.brightness(now_ms) };
            self.list.push(p);
            let idx = self.list.len() as u16;
            for ty in p.y0.floor() as usize..=p.y1.floor() as usize {
                for tx in p.x0.floor() as usize..=p.x1.floor() as usize {
                    if tx < map_w && ty < map_h {
                        self.grid[ty * map_w + tx] = idx;
                    }
                }
            }
        }
    }

    #[inline]
    pub fn at(&self, x: f64, y: f64) -> Option<&Panel> {
        if x < 0.0 || y < 0.0 || self.w == 0 {
            return None;
        }
        let i = *self.grid.get(y as usize * self.w + x as usize)?;
        let p = self.list.get((i as usize).checked_sub(1)?)?;
        (x >= p.x0 && x <= p.x1 && y >= p.y0 && y <= p.y1).then_some(p)
    }
}

/// Everything a frame shows.
pub struct Scene<'a> {
    pub map: &'a Map,
    pub doors: &'a [Door],
    pub light: &'a LightField,
    pub panels: &'a Panels,
    pub decals: &'a Decals,
    pub env: Env,
    pub boxes: &'a [BoxInput],
    pub sprites: &'a [SpriteInput],
}

/// A frame and the scratch space to draw it, kept between frames.
#[derive(Default)]
pub struct Viewport {
    pub view: Option<View>,
    pub rgba: Vec<u8>,
    depth: Vec<f32>,
    pub light: LightField,
    pub panels: Panels,
}

impl Viewport {
    pub fn new(w: usize, h: usize) -> Self {
        let mut v = Viewport::default();
        v.resize(w, h);
        v
    }

    pub fn resize(&mut self, w: usize, h: usize) {
        if self.view.is_some_and(|v| v.w == w && v.h == h) {
            return;
        }
        self.view = Some(View::new(w.max(1), h.max(1)));
        self.rgba = vec![0; w.max(1) * h.max(1) * 4];
        self.depth = vec![f32::INFINITY; w.max(1) * h.max(1)];
    }

    pub fn size(&self) -> (usize, usize) {
        self.view.map_or((0, 0), |v| (v.w, v.h))
    }
}

// ---------------------------------------------------------------- shading helpers

type Rgbf = [f32; 3];

#[inline]
fn lit(albedo: u32, light: Rgbf, k: f32) -> Rgbf {
    let c = channels(albedo);
    [c[0] * light[0] * k, c[1] * light[1] * k, c[2] * light[2] * k]
}

#[inline]
fn mix(a: Rgbf, b: Rgbf, t: f32) -> Rgbf {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

/// Air thickens with distance: towards a dusty dark indoors, the horizon outdoors.
#[inline]
fn haze(c: Rgbf, d: f32, outdoor: bool, horizon: Rgbf) -> Rgbf {
    if outdoor {
        let t = ((d - 18.0) / 60.0).clamp(0.0, 0.55);
        mix(c, horizon, t)
    } else {
        let t = ((d - 11.0) / 30.0).clamp(0.0, 0.6);
        mix(c, [12.0, 12.0, 14.0], t)
    }
}

#[inline]
fn put(rgba: &mut [u8], i: usize, c: Rgbf) {
    rgba[i * 4] = c[0].clamp(0.0, 255.0) as u8;
    rgba[i * 4 + 1] = c[1].clamp(0.0, 255.0) as u8;
    rgba[i * 4 + 2] = c[2].clamp(0.0, 255.0) as u8;
    rgba[i * 4 + 3] = 255;
}

/// Coarser texture copies for surfaces seen from far away (texels per screen pixel).
#[inline]
fn mip(texels_per_px: f64) -> usize {
    if texels_per_px < 1.3 { 0 } else if texels_per_px < 2.6 { 1 } else { 2 }
}

/// Snap to the centre of a texel, so furniture and sprites keep the same pixel grain.
#[inline]
fn snap(m: f64) -> f64 {
    ((m * TEX_RES).floor() + 0.5) / TEX_RES
}

fn open_sky(map: &Map, x: f64, y: f64) -> bool {
    if x < 0.0 || y < 0.0 || x >= map.width as f64 || y >= map.height as f64 {
        return true;
    }
    let t = map.at(x, y);
    t.is_outdoor() || t == Tile::Hedge
}

/// The face a ray coming along (`rdx`, `rdy`) hits on a wall, given which kind of grid
/// line it crossed.
fn face_of(side: Side, rdx: f64, rdy: f64) -> Face {
    match side {
        Side::Vertical => if rdx > 0.0 { Face::West } else { Face::East },
        Side::Horizontal => if rdy > 0.0 { Face::North } else { Face::South },
    }
}

/// Light on vertical surfaces facing different ways, so corners read.
fn face_shade(n: (i32, i32)) -> f32 {
    match n {
        (0, -1) => 0.86,
        (0, 1) => 1.0,
        (1, 0) => 0.93,
        _ => 0.8,
    }
}

/// What you'd see through glass: the outside from inside, an interior from outside.
fn glass_color(env: &Env, v: f64, looking_out: bool, horizon: Rgbf) -> Rgbf {
    let day = env.daylight();
    if looking_out {
        let sky = mix(horizon, [day[0] * 200.0, day[1] * 210.0, day[2] * 230.0], 0.4);
        if v > 1.35 { sky } else { mix([46.0 * day[0], 74.0 * day[1], 44.0 * day[2]], sky, ((v - 0.9) / 0.5).clamp(0.0, 1.0) as f32) }
    } else {
        // dark glass with the sky reflected in it; after dark, the lights still on inside
        let night = 1.0 - env.day();
        let glass = mix([24.0, 30.0, 36.0], horizon, 0.3);
        mix(glass, [196.0, 192.0, 160.0], night * 0.55)
    }
}

/// White and yellow paint on roads and car parks.
fn road_marking(map: &Map, wx: f64, wy: f64, mat: Material) -> Option<u32> {
    let (tx, ty) = (wx as usize, wy as usize);
    let (u, v) = (wx.fract(), wy.fract());
    let white = rgb(226, 222, 206);
    let same = |dx: i32, dy: i32| map.material((tx as i32 + dx).max(0) as usize, (ty as i32 + dy).max(0) as usize) == mat;
    let (n, s, e, w) = (same(0, -1), same(0, 1), same(1, 0), same(-1, 0));
    match mat {
        Material::Asphalt => {
            if (e || w) && !(n && s) && (!n && v < 0.05 || !s && v > 0.95) {
                return Some(white);
            }
            if (n || s) && !(e && w) && (!e && u > 0.95 || !w && u < 0.05) {
                return Some(white);
            }
            // dashed centre line where two lanes meet
            if e && w && !n && s && v < 0.04 && (wx * 0.8).fract() < 0.55 {
                return Some(rgb(220, 180, 60));
            }
            if n && s && !e && w && u < 0.04 && (wy * 0.8).fract() < 0.55 {
                return Some(rgb(220, 180, 60));
            }
            None
        }
        Material::Parking => {
            // stall lines every 2.5 m along the rows, and the row ends
            if (wx / 2.5).fract() < 0.04 && (wy - 85.0).abs() >= 0.08 {
                return Some(white);
            }
            if (!n && v < 0.05) || (!s && v > 0.95) || (wy - 85.0).abs() < 0.06 {
                return Some(white);
            }
            None
        }
        _ => None,
    }
}

const BAYER: [f32; 16] = [0.0, 8.0, 2.0, 10.0, 12.0, 4.0, 14.0, 6.0, 3.0, 11.0, 1.0, 9.0, 15.0, 7.0, 13.0, 5.0];
/// Size of a colour step: 32 levels per channel, dithered between.
const QUANT: f32 = 8.0;

// ---------------------------------------------------------------- the frame

pub fn render(vp: &mut Viewport, cam: &Camera, sc: &Scene) {
    let Some(view) = vp.view else { return };
    let (w, h) = (view.w, view.h);
    let tex = textures();
    let map = sc.map;
    let hz = view.horizon;
    let env = &sc.env;
    let horizon_c = env.sky((cam.dir_x, cam.dir_y), 0.02);
    vp.depth.iter_mut().for_each(|d| *d = f32::INFINITY);
    let rays = raycaster::cast(map, cam.x, cam.y, cam.dir_x, cam.dir_y, cam.plane_x, cam.plane_y, w);

    // per-row distances to the ceiling plane and the floor (pixel centres)
    let ceil_dist: Vec<f64> = (0..h).map(|y| view.f_v * (WALL_H - EYE_H) / (hz - (y as f64 + 0.5)).max(0.01)).collect();
    let floor_dist: Vec<f64> = (0..h).map(|y| view.f_v * EYE_H / ((y as f64 + 0.5) - hz).max(0.01)).collect();
    let elev: Vec<f64> = (0..h).map(|y| ((hz - (y as f64 + 0.5)) / view.f_v).atan()).collect();

    for (x, ray) in rays.columns.iter().enumerate() {
        let (rdx, rdy) = (ray.ray_dir_x, ray.ray_dir_y);
        let len = (rdx * rdx + rdy * rdy).sqrt();
        let dir_n = (rdx / len, rdy / len);
        let (top, bottom) = if ray.hit {
            let d = ray.distance.max(0.02);
            (view.row_of(ray.tile.render_height(), d).max(0.0) as usize, (view.row_of(0.0, d).ceil().min(h as f64)) as usize)
        } else {
            (h / 2, h / 2)
        };

        // ---- sky or ceiling above the wall
        for y in 0..top.min(h) {
            let i = y * w + x;
            let dc = ceil_dist[y];
            let (wx, wy) = (cam.x + rdx * dc, cam.y + rdy * dc);
            if (y as f64) >= hz || open_sky(map, wx, wy) {
                put(&mut vp.rgba, i, env.sky(dir_n, elev[y].max(0.0)));
                continue;
            }
            let c = ceiling_pixel(sc, tex, wx, wy, dc);
            vp.depth[i] = dc as f32;
            put(&mut vp.rgba, i, haze(c, dc as f32, false, horizon_c));
        }

        // ---- floor or ground below it
        for (y, &df) in floor_dist.iter().enumerate().skip(bottom.min(h)) {
            let i = y * w + x;
            let (wx, wy) = (cam.x + rdx * df, cam.y + rdy * df);
            if wx < 0.0 || wy < 0.0 || wx >= map.width as f64 || wy >= map.height as f64 {
                put(&mut vp.rgba, i, [18.0, 22.0, 18.0]);
                continue;
            }
            let mat = map.material_at(wx, wy);
            let outdoor = map.at(wx, wy).is_outdoor();
            let level = if df < 5.0 { 0 } else if df < 11.0 { 1 } else { 2 };
            let mut albedo = tex.floor(mat).at(wx, wy, level, true);
            if matches!(mat, Material::Asphalt | Material::Parking)
                && let Some(m) = road_marking(map, wx, wy, mat)
            {
                albedo = m;
            }
            let light = sc.light.sample(wx, wy);
            let mut c = lit(albedo, light, 1.0);
            // polished floors mirror the lights overhead
            let gloss = mat.gloss();
            if gloss > 0.0 && !outdoor {
                let k = df * (1.0 + (WALL_H) / EYE_H);
                if let Some(p) = sc.panels.at(cam.x + rdx * k, cam.y + rdy * k)
                    && p.glow > 0.0
                {
                    let fres = 0.3 + 0.7 * ((df - 1.0) / 8.0).clamp(0.0, 1.0) as f32;
                    let r = gloss * fres * p.glow * 150.0;
                    c = [c[0] + r, c[1] + r, c[2] + r * 0.95];
                }
            }
            vp.depth[i] = df as f32;
            put(&mut vp.rgba, i, haze(c, df as f32, outdoor, horizon_c));
        }

        if !ray.hit {
            continue;
        }

        // ---- the wall
        let d = ray.distance.max(0.02);
        let face = face_of(ray.side, rdx, rdy);
        let n = face.normal();
        let (hx, hy) = (cam.x + rdx * d, cam.y + rdy * d);
        let (vx, vy) = (ray.hit_tx + n.0, ray.hit_ty + n.1);
        let viewer_mat = map.material(vx.max(0) as usize, vy.max(0) as usize);
        let viewer_out = map.get(vx.max(0) as usize, vy.max(0) as usize).is_outdoor();
        let along = if matches!(face, Face::West | Face::East) { hy } else { hx };
        let s_tex = if face.runs_forward() { along } else { -along };
        let finish = match ray.tile {
            Tile::Hedge => WallFinish::Hedge,
            Tile::Column => WallFinish::Column,
            _ if viewer_out => WallFinish::Facade,
            _ => WallFinish::for_floor(viewer_mat),
        };
        let level = mip(TEX_RES * d / view.f_v);
        let light = sc.light.sample(hx + n.0 as f64 * 0.25, hy + n.1 as f64 * 0.25);
        let shade = face_shade(n);
        let decals = sc.decals.on(ray.hit_tx as usize, ray.hit_ty as usize, face);
        let door_style = ray.tile.is_any_door().then(|| door_style(sc.doors, ray.hit_tx, ray.hit_ty));
        for y in top..bottom.min(h) {
            let i = y * w + x;
            let v = EYE_H + (hz - (y as f64 + 0.5)) * d / view.f_v;
            let mut emissive = false;
            let mut texel = match (ray.tile, door_style) {
                (_, Some(style)) if v < DOOR_H => tex.door(style).at(s_tex.rem_euclid(1.0), v, level, false),
                (Tile::WindowWall, _) => if viewer_out { &tex.window_out } else { &tex.window_in }.at(s_tex, v, level, false),
                _ => tex.wall(finish).at(s_tex, v, level, false),
            };
            for dcl in decals {
                if along >= dcl.s0 && along < dcl.s1 && v >= dcl.v0 && v < dcl.v1 {
                    let (u, vv) = dcl.uv(along, v);
                    let px = sc.decals.images[dcl.image].at(u, vv);
                    if px & FLAGS != CLEAR {
                        texel = if dcl.clock && clock_hand(u, vv, env.hour) { rgb(20, 20, 20) } else { px };
                        emissive = px & FLAGS == EMIT;
                    }
                }
            }
            let c = match texel & FLAGS {
                CLEAR => continue,
                GLASS => glass_color(env, v, !viewer_out, horizon_c),
                _ if emissive => channels(texel),
                _ => {
                    let ao = if v < 0.25 { 0.72 + v as f32 * 1.1 } else if v > 2.6 { 0.9 } else { 1.0 };
                    let up = 0.9 + 0.1 * (v / WALL_H) as f32;
                    lit(texel, light, shade * ao * up)
                }
            };
            vp.depth[i] = d as f32;
            put(&mut vp.rgba, i, haze(c, d as f32, viewer_out, horizon_c));
        }

        // ---- the wall above an open door the ray went through on the way
        if let Some(od) = ray.open_door {
            let dd = od.distance.max(0.02);
            let face = face_of(od.side, rdx, rdy);
            let n = face.normal();
            let (ox, oy) = (cam.x + rdx * dd, cam.y + rdy * dd);
            let door_cell = (ox - n.0 as f64 * 0.01, oy - n.1 as f64 * 0.01);
            let (vx, vy) = (door_cell.0.floor() as i32 + n.0, door_cell.1.floor() as i32 + n.1);
            let vm = map.material(vx.max(0) as usize, vy.max(0) as usize);
            let out = map.get(vx.max(0) as usize, vy.max(0) as usize).is_outdoor();
            let finish = if out { WallFinish::Facade } else { WallFinish::for_floor(vm) };
            let along = if matches!(face, Face::West | Face::East) { oy } else { ox };
            let s_tex = if face.runs_forward() { along } else { -along };
            let light = sc.light.sample(ox + n.0 as f64 * 0.25, oy + n.1 as f64 * 0.25);
            let (y0, y1) = (view.row_of(WALL_H, dd).max(0.0) as usize, view.row_of(DOOR_H, dd).max(0.0) as usize);
            let level = mip(TEX_RES * dd / view.f_v);
            for y in y0..y1.min(h) {
                let v = EYE_H + (hz - (y as f64 + 0.5)) * dd / view.f_v;
                let c = lit(tex.wall(finish).at(s_tex, v, level, false), light, face_shade(n));
                let i = y * w + x;
                vp.depth[i] = dd as f32;
                put(&mut vp.rgba, i, haze(c, dd as f32, out, horizon_c));
            }
        }
    }

    draw_boxes(vp, &view, cam, sc, horizon_c);
    draw_sprites(vp, &view, cam, sc, horizon_c);

    // ordered dither down to the pixel-art palette
    for y in 0..h {
        for x in 0..w {
            let t = (BAYER[(y & 3) * 4 + (x & 3)] / 16.0 - 0.47) * QUANT;
            let i = (y * w + x) * 4;
            for k in 0..3 {
                let c = vp.rgba[i + k] as f32 + t;
                vp.rgba[i + k] = ((c / QUANT).round() * QUANT).clamp(0.0, 255.0) as u8;
            }
        }
    }
}

fn door_style(doors: &[Door], tx: i32, ty: i32) -> DoorStyle {
    doors.iter().find(|d| d.tx as i32 == tx && d.ty as i32 == ty).map_or(DoorStyle::Wood, |d| d.style)
}

fn clock_hand(u: f64, v: f64, hour: f64) -> bool {
    let (px, py) = (u - 0.5, v - 0.5);
    let hand = |angle: f64, len: f64| {
        let (dx, dy) = (angle.sin(), -angle.cos());
        let t = (px * dx + py * dy).clamp(0.0, len);
        ((px - dx * t).powi(2) + (py - dy * t).powi(2)).sqrt() < 0.05
    };
    let tau = std::f64::consts::TAU;
    hand((hour % 12.0) / 12.0 * tau, 0.24) || hand(hour.fract() * tau, 0.36)
}

/// The suspended ceiling at a point: tiles, their damage, and the light fittings.
fn ceiling_pixel(sc: &Scene, tex: &textures::Textures, wx: f64, wy: f64, dc: f64) -> Rgbf {
    if let Some(p) = sc.panels.at(wx, wy) {
        let (lu, lv) = (((wx - p.x0) / (p.x1 - p.x0)), ((wy - p.y0) / (p.y1 - p.y0)));
        let edge = !(0.06..=0.94).contains(&lu) || !(0.06..=0.94).contains(&lv);
        if edge {
            return [180.0, 182.0, 180.0];
        }
        if p.glow <= 0.01 {
            // a dead tube: grey lens, dark tubes behind it
            let tube = ((if p.x1 - p.x0 > p.y1 - p.y0 { lv } else { lu }) * 3.0).fract() < 0.25;
            return if tube { [70.0, 70.0, 72.0] } else { [128.0, 130.0, 130.0] };
        }
        // prismatic lens over lit tubes
        let prism = ((wx * 40.0) as i32 + (wy * 40.0) as i32) % 2 == 0;
        let g = p.glow * if prism { 255.0 } else { 236.0 };
        return [g, g, g * 0.95];
    }
    let level = if dc < 5.0 { 0 } else if dc < 11.0 { 1 } else { 2 };
    let base = tex.ceiling.at(wx, wy, level, true);
    let (cx, cy) = ((wx / 0.6).floor() as i32, (wy / 0.6).floor() as i32);
    let tile = textures::ceiling_tile(cx, cy);
    let albedo = if tile == CeilingTile::Fine { base } else { textures::ceiling_texel(base, tile, (wx / 0.6).fract(), (wy / 0.6).fract()) };
    let light = sc.light.sample(wx, wy);
    lit(albedo, light, if tile == CeilingTile::Missing { 0.5 } else { 0.84 })
}

fn draw_boxes(vp: &mut Viewport, view: &View, cam: &Camera, sc: &Scene, horizon: Rgbf) {
    let (w, h) = (view.w, view.h);
    let hz = view.horizon;
    for b in sc.boxes {
        let (x0, y0, x1, y1) = b.bounds();
        // which columns the box can cover
        let inv_det = 1.0 / (cam.plane_x * cam.dir_y - cam.dir_x * cam.plane_y);
        let mut cmin = w as f64;
        let mut cmax = -1.0f64;
        let mut behind = 0;
        for (px, py) in [(x0, y0), (x1, y0), (x0, y1), (x1, y1)] {
            let (dx, dy) = (px - cam.x, py - cam.y);
            let tx = inv_det * (cam.dir_y * dx - cam.dir_x * dy);
            let ty = inv_det * (-cam.plane_y * dx + cam.plane_x * dy);
            if ty <= 0.05 {
                behind += 1;
                continue;
            }
            let sx = (w as f64 / 2.0) * (1.0 + tx / ty);
            cmin = cmin.min(sx);
            cmax = cmax.max(sx);
        }
        if behind == 4 {
            continue;
        }
        if behind > 0 {
            cmin = 0.0;
            cmax = w as f64;
        }
        let c0 = (cmin.floor().max(0.0)) as usize;
        let c1 = (cmax.ceil().min(w as f64)) as usize;
        let outdoor = sc.map.at(b.x, b.y).is_outdoor();
        let (z0, z1) = (b.z0, b.z0 + b.h);
        for x in c0..c1 {
            let cx = 2.0 * (x as f64 + 0.5) / w as f64 - 1.0;
            let (rdx, rdy) = (cam.dir_x + cam.plane_x * cx, cam.dir_y + cam.plane_y * cx);
            // slab test in the floor plan
            let (mut tn, mut tf) = (f64::NEG_INFINITY, f64::INFINITY);
            let (mut n_in, mut n_out) = ((0, 0), (0, 0));
            for (o, d, lo, hi, axis) in [(cam.x, rdx, x0, x1, 0), (cam.y, rdy, y0, y1, 1)] {
                if d.abs() < 1e-12 {
                    if o < lo || o > hi {
                        tn = f64::INFINITY;
                    }
                    continue;
                }
                let (mut ta, mut tb) = ((lo - o) / d, (hi - o) / d);
                let s = if d > 0.0 { -1 } else { 1 };
                if ta > tb {
                    std::mem::swap(&mut ta, &mut tb);
                }
                let (nin, nout) = if axis == 0 { ((s, 0), (-s, 0)) } else { ((0, s), (0, -s)) };
                if ta > tn {
                    tn = ta;
                    n_in = nin;
                }
                if tb < tf {
                    tf = tb;
                    n_out = nout;
                }
            }
            if tn >= tf || tn < 0.05 || tf <= 0.0 {
                continue; // missed, or the camera is inside the box (sitting on it)
            }
            let ya = view.row_of(z1, tn).min(view.row_of(z1, tf)).floor().max(0.0) as usize;
            let yb = (view.row_of(z0, tn).ceil().min(h as f64)) as usize;
            let (face_in, fw_in) = b.face_for(n_in);
            let (face_out, fw_out) = b.face_for(n_out);
            let hit_in = (cam.x + rdx * tn, cam.y + rdy * tn);
            let hit_out = (cam.x + rdx * tf, cam.y + rdy * tf);
            let u_in = face_u(b, n_in, hit_in, fw_in);
            let u_out = fw_out - face_u(b, n_out, hit_out, fw_out);
            for y in ya..yb {
                let i = y * w + x;
                let dy = y as f64 + 0.5 - hz;
                let z_in = EYE_H - dy * tn / view.f_v;
                let mut hit: Option<(u32, f64, f32, (f64, f64))> = None;
                if z_in >= z0 && z_in <= z1 {
                    if let Some(c) = b.kind.texel(face_in, snap(u_in), snap(z_in - z0), fw_in, b.h) {
                        hit = Some((c, tn, face_shade(n_in), hit_in));
                    } else {
                        let z_out = EYE_H - dy * tf / view.f_v;
                        if z_out >= z0 && z_out <= z1
                            && let Some(c) = b.kind.texel(face_out, snap(u_out), snap(z_out - z0), fw_out, b.h)
                        {
                            hit = Some((c, tf, face_shade(n_out) * 0.7, hit_out));
                        }
                    }
                } else if z_in > z1 && dy > 0.0 {
                    let t_top = (EYE_H - z1) * view.f_v / dy;
                    if t_top >= tn && t_top <= tf {
                        let p = (cam.x + rdx * t_top, cam.y + rdy * t_top);
                        let f = b.facing.vector();
                        let r = (f.1, -f.0);
                        let (rx, ry) = (p.0 - b.x, p.1 - b.y);
                        let u = rx * r.0 + ry * r.1 + b.w / 2.0;
                        let v = b.d / 2.0 - (rx * f.0 + ry * f.1);
                        if let Some(c) = b.kind.texel(BoxFace::Top, snap(u), snap(v), b.w, b.d) {
                            hit = Some((c, t_top, 1.04, p));
                        }
                    }
                }
                let Some((c, t, shade, p)) = hit else { continue };
                if (t as f32) >= vp.depth[i] {
                    continue;
                }
                let light = sc.light.sample(p.0, p.1);
                let col = lit(c, light, shade);
                vp.depth[i] = t as f32;
                put(&mut vp.rgba, i, haze(col, t as f32, outdoor, horizon));
            }
        }
    }
}

/// Distance along a box face from its left end (seen from in front of it).
fn face_u(b: &BoxInput, n: (i32, i32), p: (f64, f64), width: f64) -> f64 {
    let right = (n.1 as f64, -n.0 as f64);
    (p.0 - b.x) * right.0 + (p.1 - b.y) * right.1 + width / 2.0
}

fn draw_sprites(vp: &mut Viewport, view: &View, cam: &Camera, sc: &Scene, horizon: Rgbf) {
    let (w, h) = (view.w as i64, view.h as i64);
    let inv_det = 1.0 / (cam.plane_x * cam.dir_y - cam.dir_x * cam.plane_y);
    for s in sc.sprites {
        let (dx, dy) = (s.x - cam.x, s.y - cam.y);
        let tx = inv_det * (cam.dir_y * dx - cam.dir_x * dy);
        let ty = inv_det * (-cam.plane_y * dx + cam.plane_x * dy);
        if ty <= 0.1 {
            continue;
        }
        let sx = (view.w as f64 / 2.0) * (1.0 + tx / ty);
        let half = s.w * view.f_h / ty / 2.0;
        let top = view.row_of(s.elevation + s.h, ty);
        let bot = view.row_of(s.elevation, ty);
        let (x0, x1) = (((sx - half).floor() as i64).max(0), ((sx + half).ceil() as i64).min(w));
        let (y0, y1) = ((top.floor() as i64).max(0), (bot.ceil() as i64).min(h));
        if x0 >= x1 || y0 >= y1 {
            continue;
        }
        let light = sc.light.sample(s.x, s.y);
        let outdoor = sc.map.at(s.x, s.y).is_outdoor();
        let (tw, th) = (s.w * TEX_RES, s.h * TEX_RES);
        for x in x0..x1 {
            let u = (((x as f64 + 0.5 - (sx - half)) / (2.0 * half) * tw).floor() + 0.5) / tw;
            if !(0.0..1.0).contains(&u) {
                continue;
            }
            for y in y0..y1 {
                let i = (y * w + x) as usize;
                if ty as f32 >= vp.depth[i] {
                    continue;
                }
                let v = (((y as f64 + 0.5 - top) / (bot - top) * th).floor() + 0.5) / th;
                let Some((r, g, b)) = s.shape.sample(u, v) else { continue };
                let c = if s.shape.glows(u, v) { [r as f32, g as f32, b as f32] } else { lit(rgb(r, g, b), light, 0.95) };
                vp.depth[i] = ty as f32;
                put(&mut vp.rgba, i, haze(c, ty as f32, outdoor, horizon));
            }
        }
    }
}

/// Lying on your back: the ceiling straight overhead, about 2 m away.
pub fn render_ceiling(vp: &mut Viewport, cam: &Camera, sc: &Scene) {
    let Some(view) = vp.view else { return };
    let tex = textures();
    let (w, h) = (view.w, view.h);
    let dist = WALL_H - 0.75;
    // a wide look: you take in the whole ceiling over the bed
    let f = view.f_h.max(view.f_v) * 0.55;
    let (fx, fy) = (cam.dir_x, cam.dir_y);
    let (rx, ry) = (-fy, fx);
    for y in 0..h {
        for x in 0..w {
            let ox = (x as f64 + 0.5 - w as f64 / 2.0) / f * dist;
            let oy = (h as f64 / 2.0 - (y as f64 + 0.5)) / f * dist;
            let (wx, wy) = (cam.x + rx * ox + fx * oy, cam.y + ry * ox + fy * oy);
            let c = if sc.map.at(wx, wy).blocks_sight() {
                let l = sc.light.sample(cam.x, cam.y);
                lit(rgb(150, 152, 148), l, 0.6)
            } else {
                ceiling_pixel(sc, tex, wx, wy, dist)
            };
            put(&mut vp.rgba, y * w + x, c);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Viewport;
    use crate::engine::map::Tile;
    use crate::game::world::World;

    /// How many pixels above the horizon in the centre column are open sky (no depth).
    fn sky_above(world: &mut World, x: f64, y: f64, dx: f64, dy: f64) -> usize {
        world.player.x = x;
        world.player.y = y;
        world.player.face(dx, dy);
        let mut vp = Viewport::new(160, 90);
        world.render_view(&mut vp, 0);
        (0..45).filter(|&row| vp.depth[row * 160 + 80].is_infinite()).count()
    }

    #[test]
    fn indoors_there_is_a_ceiling() {
        assert_eq!(sky_above(&mut World::new(), 9.0, 17.5, 1.0, 0.0), 0, "looking down the north corridor");
    }

    #[test]
    fn outdoors_the_sky_is_over_the_roof() {
        assert!(sky_above(&mut World::new(), 44.0, 84.0, 0.0, -1.0) > 10, "from the car park, above the facade");
    }

    #[test]
    fn the_open_entrance_shows_the_sky() {
        let mut w = World::new();
        let shut = sky_above(&mut w, 44.0, 70.0, 0.0, 1.0);
        w.map.set(43, 76, Tile::MainDoor { open: true });
        w.map.set(44, 76, Tile::MainDoor { open: true });
        let open = sky_above(&mut w, 44.0, 70.0, 0.0, 1.0);
        assert!(open > shut, "the sky shows through the doors ({shut} → {open})");
    }

    #[test]
    fn frames_render_quickly() {
        let w = World::new();
        let mut vp = Viewport::new(320, 180);
        w.render_view(&mut vp, 0); // builds the light map
        let t = std::time::Instant::now();
        for i in 0..10 {
            w.render_view(&mut vp, i * 16);
        }
        let ms = t.elapsed().as_secs_f64() * 100.0;
        // generous, so it holds in debug builds too; release is ~2–3 ms
        assert!(ms < 400.0, "{ms:.1} ms per 320×180 frame");
    }
}
