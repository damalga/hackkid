use crate::engine::raycaster::{RaycastResult, Side};
use crate::engine::map::{Map, Tile};
use crate::engine::sprites::SpriteInput;
use crate::objects::{Door, Fluorescent};

/// A line on the floor plan, from one end to the other.
type Segment = ((f64, f64), (f64, f64));

/// Height of the walls and of the ceiling plane, in tiles. The eye is at 0.5.
pub const CEILING_H: f64 = 2.6;

pub struct Renderer {
    pub screen_width: usize,
    pub screen_height: usize,
}

impl Renderer {
    pub fn new(screen_width: usize, screen_height: usize) -> Self {
        Self { screen_width, screen_height }
    }

    /// Walls, doors, ceiling/sky and floor/ground for one frame, as RGBA.
    ///
    /// Sky or ceiling (and ground or indoor floor) is decided per pixel from the tile
    /// actually above or below that point, so looking out of a doorway shows the sky
    /// and looking in from the street shows the hospital floor.
    pub fn build_frame(
        &self,
        result: &RaycastResult,
        doors: &[Door],
        cloudy: bool,
        map: &Map,
        pos_x: f64,
        pos_y: f64,
    ) -> Vec<u8> {
        let w = self.screen_width;
        let h = self.screen_height;
        let mut pixels = vec![0u8; w * h * 4];
        let half = h as f64 / 2.0;
        // the ceiling plane and the floor, projected the same way as the walls (eye at 0.5)
        let ceil_z = h as f64 * (CEILING_H - 0.5);
        let floor_z = h as f64 * 0.5;

        for (x, ray) in result.columns.iter().enumerate() {
            let (draw_start, draw_end) = if ray.hit {
                let inv_d = 1.0 / ray.distance.max(0.001);
                let bottom_offset = ((h as f64) * inv_d * 0.5) as i32;
                let top_offset = ((h as f64) * inv_d * (ray.tile.render_height() - 0.5)) as i32;
                (
                    ((h as i32 / 2) - top_offset).max(0) as usize,
                    ((h as i32 / 2) + bottom_offset).min(h as i32) as usize,
                )
            } else {
                (h / 2, h / 2)
            };

            for y in 0..draw_start {
                let row_dist = ceil_z / (half - y as f64).max(0.5);
                let wx = pos_x + ray.ray_dir_x * row_dist;
                let wy = pos_y + ray.ray_dir_y * row_dist;
                let (r, g, b) = if open_sky(map, wx, wy) {
                    sky_color(h, y, cloudy, ray.ray_dir_x, ray.ray_dir_y)
                } else {
                    ceiling_color(h, y)
                };
                put(&mut pixels, w, x, y, r, g, b);
            }

            for y in draw_end..h {
                let row_dist = floor_z / (y as f64 - half).max(0.5);
                let wx = pos_x + ray.ray_dir_x * row_dist;
                let wy = pos_y + ray.ray_dir_y * row_dist;
                let (r, g, b) = if wx < 0.0 || wy < 0.0 || wx >= map.width as f64 || wy >= map.height as f64 || row_dist > 80.0 {
                    (20, 25, 30)
                } else if map.at(wx, wy).is_outdoor() {
                    let (gr, gg, gb) = ground_pixel(map, wx, wy);
                    let fog_g = (1.0 - (row_dist / 32.0).min(1.0)).max(0.20);
                    (shade(gr, fog_g), shade(gg, fog_g), shade(gb, fog_g))
                } else {
                    let fog_f = (y as f64 / h as f64) * 0.5;
                    ((18.0 + 15.0 * fog_f) as u8, (32.0 + 15.0 * fog_f) as u8, (65.0 + 20.0 * fog_f) as u8)
                };
                put(&mut pixels, w, x, y, r, g, b);
            }

            if !ray.hit {
                continue;
            }

            let door_here = ray.tile.is_any_door();
            let inv_d = 1.0 / ray.distance.max(0.001);
            let wall_top_offset = ((h as f64) * inv_d * (CEILING_H - 0.5)) as i32;
            let wall_draw_start = ((h as i32 / 2) - wall_top_offset).max(0) as usize;

            let side_k = if matches!(ray.side, Side::Horizontal) { 0.65 } else { 1.0 };
            let fog = (1.0 - (ray.distance / 16.0).min(1.0)).max(0.1);
            let (r, g, b) = tint(ray.tile.color(), side_k * fog);
            // the wall above a door (it is only 1.0 tall) fills the gap up to the full wall height
            let (wr, wg, wb) = tint(lintel_color(ray.tile), side_k * fog);

            let matching_door = if door_here {
                doors.iter().find(|d| ray.hit_tx == d.tx as i32 && ray.hit_ty == d.ty as i32)
            } else { None };

            if door_here {
                for y in wall_draw_start..draw_start {
                    put(&mut pixels, w, x, y, wr, wg, wb);
                }
            }
            // the wall above any open door the ray passed through before hitting this wall
            if let Some(od) = ray.open_door {
                let inv_od = 1.0 / od.distance.max(0.001);
                let od_top = ((h as f64) * inv_od * (CEILING_H - 0.5)) as i32;
                let od_door_top = ((h as f64) * inv_od * 0.5) as i32;
                let od_wall_start = ((h as i32 / 2) - od_top).max(0) as usize;
                let od_door_start = ((h as i32 / 2) - od_door_top).max(0) as usize;
                let od_side_k = if matches!(od.side, Side::Horizontal) { 0.65 } else { 1.0 };
                let od_fog = (1.0 - (od.distance / 16.0).min(1.0)).max(0.1);
                let (owr, owg, owb) = tint(lintel_color(od.tile), od_side_k * od_fog);
                for y in od_wall_start..od_door_start.max(od_wall_start) {
                    put(&mut pixels, w, x, y, owr, owg, owb);
                }
            }

            for y in draw_start..draw_end {
                let v = if draw_end > draw_start {
                    (y - draw_start) as f64 / (draw_end - draw_start) as f64
                } else { 0.0 };

                // black outline for doors (top, bottom, left, right edges of the leaf)
                if door_here {
                    let edge = !(0.02..=0.98).contains(&v) || !(0.03..=0.97).contains(&ray.wall_x);
                    if edge {
                        let bk = (5.0 * fog) as u8;
                        put(&mut pixels, w, x, y, bk, bk, bk);
                        continue;
                    }
                }
                if let Some(door) = matching_door {
                    if let Some((kr, kg, kb)) = knob_pixel(door, ray.wall_x, v, fog) {
                        put(&mut pixels, w, x, y, kr, kg, kb);
                        continue;
                    }
                    if door.has_secondary_knob()
                        && let Some((kr, kg, kb)) = secondary_knob_pixel(door, ray.wall_x, v, fog)
                    {
                        put(&mut pixels, w, x, y, kr, kg, kb);
                        continue;
                    }
                }
                put(&mut pixels, w, x, y, r, g, b);
            }
        }

        pixels
    }

    /// Lit fluorescent tubes on the ceiling, with a soft glow around them.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_ceiling_lights(
        &self,
        frame: &mut [u8],
        result: &RaycastResult,
        map: &Map,
        pos_x: f64,
        pos_y: f64,
        fluorescents: &[Fluorescent],
        elapsed_ms: u64,
    ) {
        let w = self.screen_width;
        let h = self.screen_height;
        let pos_z = h as f64 * (CEILING_H - 0.5);
        let half_h = h / 2;

        // only tubes that are on and within sight distance (14) of the player matter
        let lit: Vec<(Segment, f32, (u8, u8, u8))> = fluorescents
            .iter()
            .map(|f| (f.endpoints(), f.brightness(elapsed_ms), f.color))
            .filter(|&(((x1, y1), (x2, y2)), br, _)| br > 0.01 && dist_to_segment(pos_x, pos_y, x1, y1, x2, y2) < 14.5)
            .collect();
        if lit.is_empty() {
            return;
        }

        for x in 0..w {
            let ray = &result.columns[x];
            let draw_start = if ray.hit {
                let inv_d = 1.0 / ray.distance.max(0.001);
                let tile_h = ray.tile.render_height();
                let top_offset = ((h as f64) * inv_d * (tile_h - 0.5)) as i32;
                ((h as i32 / 2) - top_offset).max(0) as usize
            } else { 0 };
            let ceiling_end = draw_start.min(half_h);

            for y in 0..ceiling_end {
                let p = (h as f64 / 2.0 - y as f64).max(0.5);
                let row_dist = pos_z / p;
                if row_dist > 14.0 { continue; }
                let world_x = pos_x + ray.ray_dir_x * row_dist;
                let world_y = pos_y + ray.ray_dir_y * row_dist;
                if open_sky(map, world_x, world_y) {
                    continue;
                }

                let mut best_bright = 0.0f32;
                let mut best_color = (0u8, 0u8, 0u8);
                for &(((x1, y1), (x2, y2)), br, color) in &lit {
                    let d = dist_to_segment(world_x, world_y, x1, y1, x2, y2);
                    let contribution = if d < 0.10 {
                        br
                    } else if d < 0.40 {
                        br * (1.0 - (d - 0.10) as f32 / 0.30)
                    } else {
                        0.0
                    };
                    if contribution > best_bright {
                        best_bright = contribution;
                        best_color = color;
                    }
                }

                if best_bright > 0.0 {
                    let fog = (1.0 - (row_dist / 14.0)).max(0.0) as f32;
                    let intensity = (best_bright * fog).clamp(0.0, 1.0);
                    let idx = (y * w + x) * 4;
                    let base_r = frame[idx] as f32;
                    let base_g = frame[idx + 1] as f32;
                    let base_b = frame[idx + 2] as f32;
                    let (tr, tg, tb) = best_color;
                    let mix = |a: f32, b: u8| -> u8 {
                        (a * (1.0 - intensity) + b as f32 * intensity).clamp(0.0, 255.0) as u8
                    };
                    frame[idx] = mix(base_r, tr);
                    frame[idx + 1] = mix(base_g, tg);
                    frame[idx + 2] = mix(base_b, tb);
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_sprites(
        &self,
        frame: &mut [u8],
        result: &RaycastResult,
        pos_x: f64,
        pos_y: f64,
        dir_x: f64,
        dir_y: f64,
        plane_x: f64,
        plane_y: f64,
        sprites: &[SpriteInput],
    ) {
        let w = self.screen_width;
        let h = self.screen_height;
        let inv_det = 1.0 / (plane_x * dir_y - dir_x * plane_y);

        let mut sorted: Vec<(f64, &SpriteInput)> = sprites
            .iter()
            .map(|s| {
                let dx = s.x - pos_x;
                let dy = s.y - pos_y;
                (dx * dx + dy * dy, s)
            })
            .collect();
        sorted.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        for (_, sprite) in sorted {
            let dx = sprite.x - pos_x;
            let dy = sprite.y - pos_y;

            let transform_x = inv_det * (dir_y * dx - dir_x * dy);
            let transform_y = inv_det * (-plane_y * dx + plane_x * dy);
            if transform_y <= 0.05 { continue; }

            let (asp_w, asp_h) = sprite.shape.aspect();
            let base_size = (h as f64 * sprite.scale) / transform_y;
            let sprite_h = (base_size * asp_h) as i32;
            let sprite_w = (base_size * asp_w) as i32;

            let ground_off = 0.5 - sprite.elevation;
            let vertical_shift = (base_size * ground_off) as i32;
            let center_y = h as i32 / 2 + vertical_shift;

            let screen_x = ((w as f64 / 2.0) * (1.0 + transform_x / transform_y)) as i32;
            let start_x = (screen_x - sprite_w / 2).max(0);
            let end_x = (screen_x + sprite_w / 2).min(w as i32);
            let sprite_top = center_y - sprite_h;
            let start_y = sprite_top.max(0);
            let end_y = center_y.min(h as i32);

            for x in start_x..end_x {
                let xi = x as usize;
                if let Some(ray) = result.columns.get(xi)
                    && ray.hit && transform_y >= ray.distance { continue; }
                let u = (x - (screen_x - sprite_w / 2)) as f64 / sprite_w.max(1) as f64;
                for y in start_y..end_y {
                    let v = (y - sprite_top) as f64 / sprite_h.max(1) as f64;
                    if let Some((r, g, b)) = sprite.shape.sample(u, v) {
                        let fog = (1.0 - (transform_y / 12.0).min(1.0)).max(0.15);
                        let idx = (y as usize * w + xi) * 4;
                        frame[idx] = (r as f64 * fog) as u8;
                        frame[idx + 1] = (g as f64 * fog) as u8;
                        frame[idx + 2] = (b as f64 * fog) as u8;
                        frame[idx + 3] = 255;
                    }
                }
            }
        }
    }
}

fn shade(c: u8, k: f64) -> u8 {
    (c as f64 * k) as u8
}

fn tint((r, g, b): (u8, u8, u8), k: f64) -> (u8, u8, u8) {
    (shade(r, k), shade(g, k), shade(b, k))
}

/// The wall colour above a door leaf, matching the wall the door sits in.
fn lintel_color(tile: Tile) -> (u8, u8, u8) {
    match tile {
        Tile::BathroomDoor { .. } | Tile::OperatingDoor { .. } => Tile::BathroomWall.color(),
        Tile::MainDoor { .. } => Tile::HospitalOuterWall.color(),
        Tile::Door { .. } => Tile::Wall.color(),
        other => other.color(),
    }
}

/// True where nothing roofs the point over: outdoor tiles and anything off the map.
fn open_sky(map: &Map, x: f64, y: f64) -> bool {
    if x < 0.0 || y < 0.0 || x >= map.width as f64 || y >= map.height as f64 {
        return true;
    }
    map.at(x, y).is_outdoor()
}

fn put(pixels: &mut [u8], w: usize, x: usize, y: usize, r: u8, g: u8, b: u8) {
    let i = (y * w + x) * 4;
    pixels[i] = r;
    pixels[i + 1] = g;
    pixels[i + 2] = b;
    pixels[i + 3] = 255;
}

fn sky_color(h: usize, y: usize, cloudy: bool, ray_dir_x: f64, ray_dir_y: f64) -> (u8, u8, u8) {
    let half = h as f64 / 2.0;
    let t = (y as f64 / half).clamp(0.0, 1.0);

    // Base gradient (top -> horizon)
    let (mut r, mut g, mut b) = if cloudy {
        (
            95.0 + 90.0 * t,
            110.0 + 85.0 * t,
            140.0 + 60.0 * t,
        )
    } else {
        (
            60.0 + 130.0 * t,
            95.0 + 130.0 * t,
            165.0 + 65.0 * t,
        )
    };

    // Fixed sun direction (world XY, unit vector). Pointing NE-ish.
    let sun_dx = 0.60;
    let sun_dy = -0.80;
    // Angular alignment with the current ray. ray_dir is not strictly unit but ~1 near center.
    let ray_len = (ray_dir_x * ray_dir_x + ray_dir_y * ray_dir_y).sqrt().max(0.001);
    let horiz_align = (ray_dir_x * sun_dx + ray_dir_y * sun_dy) / ray_len;

    // Screen-space sun position: pitch fixed, azimuth from horiz_align.
    let sun_v = 0.28; // vertical anchor (0=top,1=horizon)
    let dv = t - sun_v;
    // Horizontal "distance" from sun center in normalized units.
    // horiz_align=1 → dead center. Convert to angular offset for a wider glow.
    let du = (1.0 - horiz_align).max(0.0).sqrt() * 1.6;

    let disc_r = 0.11;
    let d = (du * du + dv * dv).sqrt();
    if horiz_align > 0.0 {
        if d < disc_r {
            // Bright sun disc
            let core = if cloudy { 0.65 } else { 1.0 };
            let sr = 255.0 * core + (1.0 - core) * r;
            let sg = 245.0 * core + (1.0 - core) * g;
            let sb = 200.0 * core + (1.0 - core) * b;
            r = sr; g = sg; b = sb;
        } else if d < disc_r + 0.22 {
            // Soft halo
            let k = 1.0 - (d - disc_r) / 0.22;
            let halo = k.powi(2) * if cloudy { 0.35 } else { 0.70 };
            r = r * (1.0 - halo) + 255.0 * halo;
            g = g * (1.0 - halo) + 240.0 * halo;
            b = b * (1.0 - halo) + 190.0 * halo;
        }
    }

    // Clouds (both weathers, more coverage when cloudy).
    let cloud_intensity = if cloudy { 0.85 } else { 0.28 };
    // Cloud bands based on ray azimuth (angle of ray_dir_x/y) and screen height.
    let azimuth = ray_dir_y.atan2(ray_dir_x);
    let band = (azimuth * 2.7).sin() * 0.5
        + (azimuth * 1.3 + t * 5.0).sin() * 0.3
        + (azimuth * 5.1).sin() * 0.2;
    let vert_bias = (0.55 - t).max(0.0) * 1.4; // clouds mostly upper half
    let raw = (band * 0.5 + 0.35) * vert_bias;
    let cloud_factor = (raw * cloud_intensity).clamp(0.0, 0.92);
    if cloud_factor > 0.02 {
        let cr = if cloudy { 235.0 } else { 245.0 };
        let cg = if cloudy { 235.0 } else { 245.0 };
        let cb = if cloudy { 240.0 } else { 250.0 };
        r = r * (1.0 - cloud_factor) + cr * cloud_factor;
        g = g * (1.0 - cloud_factor) + cg * cloud_factor;
        b = b * (1.0 - cloud_factor) + cb * cloud_factor;
    }

    (
        r.clamp(0.0, 255.0) as u8,
        g.clamp(0.0, 255.0) as u8,
        b.clamp(0.0, 255.0) as u8,
    )
}

fn ceiling_color(h: usize, y: usize) -> (u8, u8, u8) {
    let half = h as f64 / 2.0;
    let t = (y as f64 / half).clamp(0.0, 1.0);
    let fog = (0.4 + t * 0.55).clamp(0.15, 1.0);
    let (r, g, b) = (225, 240, 252);
    (
        (r as f64 * fog) as u8,
        (g as f64 * fog) as u8,
        (b as f64 * fog) as u8,
    )
}

fn knob_pixel(door: &Door, u: f64, v: f64, fog: f64) -> Option<(u8, u8, u8)> {
    let (u1, u2) = door.knob_u_range();
    let (v1, v2) = door.knob_v_range();
    knob_ellipse(u, v, u1, u2, v1, v2, door.knob_color, fog)
}

fn secondary_knob_pixel(door: &Door, u: f64, v: f64, fog: f64) -> Option<(u8, u8, u8)> {
    let (u1, u2) = door.secondary_knob_u_range();
    let (v1, v2) = door.knob_v_range();
    knob_ellipse(u, v, u1, u2, v1, v2, door.knob_color, fog)
}

#[allow(clippy::too_many_arguments)]
fn knob_ellipse(
    u: f64, v: f64,
    u1: f64, u2: f64, v1: f64, v2: f64,
    color: (u8, u8, u8), fog: f64,
) -> Option<(u8, u8, u8)> {
    if u < u1 || u > u2 || v < v1 || v > v2 { return None; }
    let cu = (u1 + u2) / 2.0;
    let cv = (v1 + v2) / 2.0;
    let ru = (u2 - u1) / 2.0;
    let rv = (v2 - v1) / 2.0;
    let du = (u - cu) / ru;
    let dv = (v - cv) / rv;
    if du * du + dv * dv > 1.0 { return None; }
    let highlight = if du < -0.2 && dv < -0.2 { 1.25 } else { 1.0 };
    let (r, g, b) = color;
    let mix = |c: u8| ((c as f64 * fog * highlight).clamp(0.0, 255.0)) as u8;
    Some((mix(r), mix(g), mix(b)))
}

fn ground_pixel(map: &Map, wx: f64, wy: f64) -> (u8, u8, u8) {
    let tx = wx as usize;
    let ty = wy as usize;
    let u = wx - tx as f64;
    let v = wy - ty as f64;
    let tile = map.get(tx, ty);
    match tile {
        Tile::Garden | Tile::Outdoor => garden_tex(u, v, tx, ty),
        Tile::Sidewalk => sidewalk_tex(u, v, tx, ty),
        Tile::Road => road_tex(u, v, tx, ty, map),
        Tile::Parking => parking_tex(u, v, tx, ty, map),
        Tile::Floor => (25, 40, 75),
        _ => tile.color(),
    }
}

fn hash_noise(u: f64, v: f64, tx: usize, ty: usize, freq: f64) -> f64 {
    let seed_u = tx as f64 * 0.531 + u * freq;
    let seed_v = ty as f64 * 0.373 + v * freq;
    ((seed_u * 12.9898 + seed_v * 78.233).sin() * 43758.5453).fract().abs()
}

fn garden_tex(u: f64, v: f64, tx: usize, ty: usize) -> (u8, u8, u8) {
    let n = hash_noise(u, v, tx, ty, 24.0);
    let base = (58, 100, 52);
    let dark = (38, 72, 36);
    let mid = (72, 118, 58);
    let light = (95, 140, 68);
    let dry = (130, 138, 70);
    if n < 0.18 { dark }
    else if n < 0.55 { base }
    else if n < 0.82 { mid }
    else if n < 0.94 { light }
    else { dry }
}

fn sidewalk_tex(u: f64, v: f64, tx: usize, ty: usize) -> (u8, u8, u8) {
    let border = 0.05;
    if u < border || u > 1.0 - border || v < border || v > 1.0 - border {
        return (8, 8, 10);
    }
    let n = hash_noise(u, v, tx, ty, 18.0) * 14.0 - 7.0;
    let r = (155.0 + n).clamp(120.0, 190.0) as u8;
    let g = (152.0 + n).clamp(120.0, 190.0) as u8;
    let b = (144.0 + n).clamp(115.0, 185.0) as u8;
    (r, g, b)
}

fn road_tex(u: f64, v: f64, tx: usize, ty: usize, map: &Map) -> (u8, u8, u8) {
    let n = hash_noise(u, v, tx, ty, 40.0) * 8.0 - 4.0;
    let base_r = (34.0 + n).clamp(20.0, 55.0);
    let base_g = (34.0 + n).clamp(20.0, 55.0);
    let base_b = (38.0 + n).clamp(22.0, 60.0);
    let white = (232, 228, 210);

    let n_road = matches!(map.get(tx, ty.wrapping_sub(1)), Tile::Road);
    let s_road = matches!(map.get(tx, ty + 1), Tile::Road);
    let e_road = matches!(map.get(tx + 1, ty), Tile::Road);
    let w_road = matches!(map.get(tx.wrapping_sub(1), ty), Tile::Road);

    // Horizontal road strip (E/W neighbors are road)
    if e_road || w_road {
        // Solid white edge line at outer sidewalk boundary (top and bottom of strip)
        if !n_road && v < 0.06 { return white; }
        if !s_road && v > 0.94 { return white; }
        // Dashed centerline between two lanes
        let center_v = if s_road && !n_road { 1.0 }
                       else if n_road && !s_road { 0.0 }
                       else { 0.5 };
        if (v - center_v).abs() < 0.05 {
            let seg = (tx as f64 + u) * 1.6;
            let f = seg - seg.floor();
            if f < 0.55 { return white; }
        }
    }
    // Vertical road strip
    if n_road || s_road {
        if !e_road && u > 0.94 { return white; }
        if !w_road && u < 0.06 { return white; }
        let center_u = if e_road && !w_road { 1.0 }
                       else if w_road && !e_road { 0.0 }
                       else { 0.5 };
        if (u - center_u).abs() < 0.05 {
            let seg = (ty as f64 + v) * 1.6;
            let f = seg - seg.floor();
            if f < 0.55 { return white; }
        }
    }
    (base_r as u8, base_g as u8, base_b as u8)
}

fn parking_tex(u: f64, v: f64, tx: usize, ty: usize, map: &Map) -> (u8, u8, u8) {
    let n = hash_noise(u, v, tx, ty, 30.0) * 6.0 - 3.0;
    let base_r = (60.0 + n).clamp(45.0, 78.0);
    let base_g = (60.0 + n).clamp(45.0, 78.0);
    let base_b = (64.0 + n).clamp(48.0, 82.0);
    let white = (230, 225, 205);

    let n_park = matches!(map.get(tx, ty.wrapping_sub(1)), Tile::Parking);
    let s_park = matches!(map.get(tx, ty + 1), Tile::Parking);
    let e_park = matches!(map.get(tx + 1, ty), Tile::Parking);
    let w_park = matches!(map.get(tx.wrapping_sub(1), ty), Tile::Parking);

    // Perimeter curb line where parking meets non-parking
    if !n_park && v < 0.06 { return white; }
    if !s_park && v > 0.94 { return white; }
    if !e_park && u > 0.94 { return white; }
    if !w_park && u < 0.06 { return white; }

    // Stall dividers every 2 tiles (paired), if this row runs east-west
    if (e_park || w_park) && tx.is_multiple_of(2)
        && u < 0.04 { return white; }
    if (e_park || w_park) && tx % 2 == 1
        && u > 0.96 { return white; }
    // Stall dividers every 2 tiles vertical
    if (n_park || s_park) && !(e_park || w_park) && ty.is_multiple_of(2)
        && v < 0.04 { return white; }
    if (n_park || s_park) && !(e_park || w_park) && ty % 2 == 1
        && v > 0.96 { return white; }
    (base_r as u8, base_g as u8, base_b as u8)
}

fn dist_to_segment(px: f64, py: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let l2 = dx * dx + dy * dy;
    if l2 < 1e-6 {
        return ((px - x1).powi(2) + (py - y1).powi(2)).sqrt();
    }
    let t = (((px - x1) * dx + (py - y1) * dy) / l2).clamp(0.0, 1.0);
    let cx = x1 + t * dx;
    let cy = y1 + t * dy;
    ((px - cx).powi(2) + (py - cy).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::raycaster;
    use crate::game::world::World;

    const W: usize = 160;
    const H: usize = 90;

    /// For the centre column: how many pixels above the horizon are ceiling, and how many sky.
    fn centre_column(world: &World, x: f64, y: f64, dx: f64, dy: f64) -> (usize, usize) {
        let r = Renderer::new(W, H);
        let rays = raycaster::cast(&world.map, x, y, dx, dy, -dy * 0.66, dx * 0.66, W);
        let frame = r.build_frame(&rays, &world.doors, false, &world.map, x, y);
        let ray = &rays.columns[W / 2];
        let (mut ceiling, mut sky) = (0, 0);
        for py in 0..H / 2 {
            let i = (py * W + W / 2) * 4;
            let px = (frame[i], frame[i + 1], frame[i + 2]);
            if px == ceiling_color(H, py) {
                ceiling += 1;
            } else if px == sky_color(H, py, false, ray.ray_dir_x, ray.ray_dir_y) {
                sky += 1;
            }
        }
        (ceiling, sky)
    }

    #[test]
    fn indoors_there_is_a_ceiling() {
        let (ceiling, sky) = centre_column(&World::new(), 12.0, 16.0, 1.0, 0.0); // down the length of the hub
        assert!(ceiling > 0 && sky == 0);
    }

    #[test]
    fn outdoors_there_is_no_ceiling_above_the_hospital_roof() {
        let (ceiling, sky) = centre_column(&World::new(), 30.0, 70.0, 0.0, -1.0);
        assert!(ceiling == 0 && sky > 0, "ceiling {ceiling}, sky {sky}");
    }

    #[test]
    fn the_open_street_door_shows_the_sky() {
        let mut w = World::new();
        w.map.set(22, 59, Tile::MainDoor { open: true });
        let (_, sky) = centre_column(&w, 22.5, 57.0, 0.0, 1.0);
        assert!(sky > 0, "looking out of the door from the vestibule shows sky");
    }
}
