//! Pixel-art textures, painted procedurally once at start-up.
//!
//! Every surface uses the same density ([`TEX_RES`] texels per metre), so the pixel grain
//! is consistent from walls to floors to furniture. Rows count up from the floor. Each
//! texture keeps two smaller copies (mip levels) for when it's seen far away, so distant
//! grout lines don't shimmer.

use std::sync::OnceLock;

use crate::engine::camera::TEX_RES;
use crate::engine::map::Material;

/// Texel flags, in the top byte of a texel.
pub const GLASS: u32 = 1 << 24;
pub const CLEAR: u32 = 2 << 24;
pub const FLAGS: u32 = 0xFF00_0000;

pub struct Tex {
    pub w: usize,
    pub h: usize,
    px: Vec<u32>,
}

impl Tex {
    fn paint(w: usize, h: usize, f: impl Fn(i32, i32) -> u32) -> Tex {
        let mut px = Vec::with_capacity(w * h);
        for y in 0..h {
            for x in 0..w {
                px.push(f(x as i32, y as i32));
            }
        }
        Tex { w, h, px }
    }

    /// Wraps horizontally (and vertically if `wrap_y`), clamps otherwise.
    #[inline]
    pub fn get(&self, x: i64, y: i64, wrap_y: bool) -> u32 {
        let xi = x.rem_euclid(self.w as i64) as usize;
        let yi = if wrap_y { y.rem_euclid(self.h as i64) as usize } else { y.clamp(0, self.h as i64 - 1) as usize };
        self.px[yi * self.w + xi]
    }

    fn half(&self) -> Tex {
        let (w, h) = ((self.w / 2).max(1), (self.h / 2).max(1));
        Tex::paint(w, h, |x, y| {
            let mut sum = [0u32; 3];
            let (mut n, mut glass, mut clear) = (0u32, 0, 0);
            for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                let t = self.get((x * 2 + dx) as i64, (y * 2 + dy) as i64, true);
                match t & FLAGS {
                    CLEAR => clear += 1,
                    GLASS => glass += 1,
                    _ => {}
                }
                if t & FLAGS != CLEAR {
                    sum[0] += (t >> 16) & 255;
                    sum[1] += (t >> 8) & 255;
                    sum[2] += t & 255;
                    n += 1;
                }
            }
            if clear >= 2 || n == 0 {
                return CLEAR;
            }
            let c = rgb((sum[0] / n) as u8, (sum[1] / n) as u8, (sum[2] / n) as u8);
            if glass >= 2 { c | GLASS } else { c }
        })
    }
}

/// A texture and its two smaller mip levels.
pub struct Mipped([Tex; 3]);

impl Mipped {
    fn new(t: Tex) -> Self {
        let m1 = t.half();
        let m2 = m1.half();
        Mipped([t, m1, m2])
    }

    pub fn base(&self) -> &Tex {
        &self.0[0]
    }

    /// Samples at metres (`s` across, `v` up), at mip `level` (0 = full detail).
    #[inline]
    pub fn at(&self, s: f64, v: f64, level: usize, wrap_y: bool) -> u32 {
        let t = &self.0[level.min(2)];
        let k = TEX_RES / (1 << level.min(2)) as f64;
        t.get((s * k).floor() as i64, (v * k).floor() as i64, wrap_y)
    }
}

/// Wall finishes. Which one a wall shows depends on the room on the viewer's side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallFinish {
    Corridor,
    Room,
    Tile,
    Lobby,
    Steel,
    Paper,
    Block,
    Facade,
    Hedge,
    Column,
}

impl WallFinish {
    pub fn for_floor(m: Material) -> WallFinish {
        match m {
            Material::Corridor => WallFinish::Corridor,
            Material::Room => WallFinish::Room,
            Material::Tile => WallFinish::Tile,
            Material::Terrazzo => WallFinish::Lobby,
            Material::Epoxy => WallFinish::Steel,
            Material::Carpet => WallFinish::Paper,
            Material::Concrete => WallFinish::Block,
            _ => WallFinish::Facade,
        }
    }
}

/// How a door leaf looks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoorStyle {
    /// Patient room: wood veneer, a narrow vision panel, a kick plate.
    Wood,
    /// Restroom: painted, push plate, no glass.
    Restroom,
    /// Surgery, pharmacy, archive: double steel doors with round portholes.
    Secure,
    /// Entrances: aluminium-framed glass sliding doors.
    Glass,
    /// Plant room, kitchen yard: painted steel with a push bar.
    Service,
}

pub struct Textures {
    walls: Vec<Mipped>,
    pub window_in: Mipped,
    pub window_out: Mipped,
    doors: Vec<Mipped>,
    floors: Vec<Mipped>,
    pub ceiling: Mipped,
}

impl Textures {
    pub fn wall(&self, f: WallFinish) -> &Mipped {
        &self.walls[f as usize]
    }

    pub fn door(&self, s: DoorStyle) -> &Mipped {
        &self.doors[s as usize]
    }

    pub fn floor(&self, m: Material) -> &Mipped {
        &self.floors[m as usize]
    }
}

pub fn textures() -> &'static Textures {
    static T: OnceLock<Textures> = OnceLock::new();
    T.get_or_init(|| Textures {
        walls: [
            WallFinish::Corridor, WallFinish::Room, WallFinish::Tile, WallFinish::Lobby, WallFinish::Steel,
            WallFinish::Paper, WallFinish::Block, WallFinish::Facade, WallFinish::Hedge, WallFinish::Column,
        ]
        .iter()
        .map(|&f| Mipped::new(wall_tex(f)))
        .collect(),
        window_in: Mipped::new(window_tex(true)),
        window_out: Mipped::new(window_tex(false)),
        doors: [DoorStyle::Wood, DoorStyle::Restroom, DoorStyle::Secure, DoorStyle::Glass, DoorStyle::Service]
            .iter()
            .map(|&s| Mipped::new(door_tex(s)))
            .collect(),
        floors: [
            Material::Corridor, Material::Room, Material::Tile, Material::Terrazzo, Material::Epoxy, Material::Carpet,
            Material::Concrete, Material::Grass, Material::Sidewalk, Material::Asphalt, Material::Parking, Material::Pavers,
        ]
        .iter()
        .map(|&m| Mipped::new(floor_tex(m)))
        .collect(),
        ceiling: Mipped::new(ceiling_tex()),
    })
}

// ---------------------------------------------------------------- colour helpers

#[inline]
pub const fn rgb(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | b as u32
}

#[inline]
pub fn channels(c: u32) -> [f32; 3] {
    [((c >> 16) & 255) as f32, ((c >> 8) & 255) as f32, (c & 255) as f32]
}

/// Adds `d` to every channel.
fn lift(c: u32, d: i32) -> u32 {
    let f = |v: u32| (v as i32 + d).clamp(0, 255) as u8;
    rgb(f((c >> 16) & 255), f((c >> 8) & 255), f(c & 255))
}

fn mul(c: u32, k: f32) -> u32 {
    let f = |v: u32| (v as f32 * k).clamp(0.0, 255.0) as u8;
    rgb(f((c >> 16) & 255), f((c >> 8) & 255), f(c & 255))
}

fn mix(a: u32, b: u32, t: f32) -> u32 {
    let (a, b) = (channels(a), channels(b));
    let f = |i: usize| (a[i] + (b[i] - a[i]) * t).clamp(0.0, 255.0) as u8;
    rgb(f(0), f(1), f(2))
}

/// A stable pseudo-random number in 0..1 for integer coordinates.
#[inline]
pub fn hash(x: i32, y: i32, seed: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x8DA6_B343) ^ (y as u32).wrapping_mul(0xD816_3841) ^ seed.wrapping_mul(0xCB1A_B31F);
    h ^= h >> 13;
    h = h.wrapping_mul(0x5BD1_E995);
    h ^= h >> 15;
    (h & 0xFFFF) as f32 / 65535.0
}

/// Smooth value noise, about one bump per `scale` texels.
fn noise(x: i32, y: i32, scale: f32, seed: u32) -> f32 {
    let (fx, fy) = (x as f32 / scale, y as f32 / scale);
    let (ix, iy) = (fx.floor() as i32, fy.floor() as i32);
    let (tx, ty) = (fx - ix as f32, fy - iy as f32);
    let (sx, sy) = (tx * tx * (3.0 - 2.0 * tx), ty * ty * (3.0 - 2.0 * ty));
    let a = hash(ix, iy, seed) + (hash(ix + 1, iy, seed) - hash(ix, iy, seed)) * sx;
    let b = hash(ix, iy + 1, seed) + (hash(ix + 1, iy + 1, seed) - hash(ix, iy + 1, seed)) * sx;
    a + (b - a) * sy
}

/// Paint with a little unevenness, so flat colours read as surfaces.
fn grain(c: u32, x: i32, y: i32, seed: u32, amount: f32) -> u32 {
    let n = (hash(x, y, seed) - 0.5) * amount + (noise(x, y, 6.0, seed + 1) - 0.5) * amount;
    lift(c, n as i32)
}

// ---------------------------------------------------------------- walls (40 × 56: 2 m wide, 2.8 m tall)

const WALL_W: usize = 40;
const WALL_TEX_H: usize = 56;

fn wall_tex(f: WallFinish) -> Tex {
    Tex::paint(WALL_W, WALL_TEX_H, |x, y| match f {
        WallFinish::Corridor => corridor_wall(x, y),
        WallFinish::Room => room_wall(x, y),
        WallFinish::Tile => tile_wall(x, y),
        WallFinish::Lobby => lobby_wall(x, y),
        WallFinish::Steel => steel_wall(x, y),
        WallFinish::Paper => paper_wall(x, y),
        WallFinish::Block => block_wall(x, y),
        WallFinish::Facade => facade(x, y),
        WallFinish::Hedge => hedge(x, y),
        WallFinish::Column => column(x, y),
    })
}

/// Paint high on a wall, a touch darker towards the ceiling.
fn upper_paint(c: u32, x: i32, y: i32, seed: u32) -> u32 {
    let shade = if y >= 53 { -10 } else if y >= 50 { -4 } else { 0 };
    lift(grain(c, x, y, seed, 6.0), shade)
}

fn cove_base(c: u32, y: i32) -> u32 {
    if y == 2 { lift(c, 18) } else { c }
}

fn corridor_wall(x: i32, y: i32) -> u32 {
    match y {
        0..=2 => cove_base(rgb(72, 70, 66), y),
        3..=16 => {
            // scuffed lower paint: carts and beds rub here
            let scuff = hash(x / 3, y / 2, 7) > 0.93 && y < 10;
            let c = grain(rgb(150, 170, 160), x, y, 3, 7.0);
            if scuff { lift(c, -22) } else { c }
        }
        17 => rgb(122, 96, 68),                       // underside of the handrail
        18 => grain(rgb(190, 150, 104), x * 3, y, 9, 10.0), // beech rail
        19 => rgb(222, 188, 140),                     // its rounded top
        20 if x % 20 == 4 || x % 20 == 5 => rgb(150, 150, 146), // bracket
        20 => rgb(176, 178, 170),                     // shadow under the rail
        _ => upper_paint(rgb(224, 224, 214), x, y, 5),
    }
}

fn room_wall(x: i32, y: i32) -> u32 {
    match y {
        0..=2 => cove_base(rgb(92, 96, 100), y),
        3..=16 => grain(rgb(190, 206, 210), x, y, 11, 6.0),
        17 => rgb(150, 160, 164),
        18 => rgb(236, 238, 236),
        19 => rgb(214, 216, 214),
        _ => upper_paint(rgb(228, 230, 222), x, y, 13),
    }
}

fn tile_wall(x: i32, y: i32) -> u32 {
    if y >= 41 {
        return upper_paint(rgb(226, 228, 222), x, y, 17);
    }
    if y == 40 {
        return rgb(204, 206, 200);
    }
    if x % 4 == 0 || y % 4 == 0 {
        return rgb(168, 172, 176); // grout
    }
    let (tx, ty) = (x / 4, y / 4);
    let base = if ty == 5 { rgb(86, 138, 178) } else { rgb(232, 236, 238) }; // a blue band at 1 m
    let c = lift(base, ((hash(tx, ty, 19) - 0.5) * 12.0) as i32);
    let cracked = hash(tx, ty, 23) > 0.96 && (x - y).rem_euclid(4) == 0;
    if cracked { lift(c, -40) } else if x % 4 == 1 && y % 4 == 3 { lift(c, 10) } else { c }
}

fn lobby_wall(x: i32, y: i32) -> u32 {
    match y {
        0..=2 => rgb(66, 46, 30),
        3..=23 => {
            if x % 20 == 0 {
                return rgb(88, 60, 38); // panel joint
            }
            let grain_line = noise(x * 6, y, 5.0, 29);
            mix(rgb(150, 104, 64), rgb(124, 84, 50), grain_line * 0.8)
        }
        24 => rgb(110, 76, 46),
        25 => rgb(176, 130, 84),
        _ => upper_paint(rgb(232, 224, 204), x, y, 31),
    }
}

fn steel_wall(x: i32, y: i32) -> u32 {
    if y <= 4 {
        return lift(rgb(176, 182, 186), ((hash(x * 7, y, 37) - 0.5) * 18.0) as i32);
    }
    if x % 20 == 0 || y == 28 || y == 5 {
        return rgb(146, 154, 158);
    }
    if x % 20 == 1 || y == 29 {
        return rgb(226, 232, 234);
    }
    grain(rgb(206, 214, 216), x, y, 41, 4.0)
}

fn paper_wall(x: i32, y: i32) -> u32 {
    match y {
        0..=2 => rgb(118, 88, 58),
        17 => rgb(118, 84, 54),
        18 => rgb(150, 110, 70),
        _ => {
            let stripe = if x % 4 == 0 { -8 } else { 0 };
            lift(upper_paint(rgb(214, 204, 180), x, y, 43), stripe)
        }
    }
}

fn block_wall(x: i32, y: i32) -> u32 {
    if y <= 3 {
        // a yellow and black kerb line
        return if (x + y) / 3 % 2 == 0 { rgb(200, 170, 40) } else { rgb(40, 40, 40) };
    }
    let row = y / 4;
    let off = if row % 2 == 0 { 0 } else { 4 };
    if y % 4 == 0 || (x + off) % 8 == 0 {
        return rgb(134, 136, 134);
    }
    let stain = noise(x, y, 9.0, 47) > 0.7;
    let c = grain(rgb(166, 170, 166), x, y, 53, 10.0);
    if stain { lift(c, -14) } else { c }
}

fn facade(x: i32, y: i32) -> u32 {
    match y {
        0..=17 => {
            // brick plinth
            let course = y / 2;
            let off = if course % 2 == 0 { 0 } else { 2 };
            if y % 2 == 0 || (x + off) % 4 == 0 {
                return rgb(168, 158, 146);
            }
            lift(rgb(146, 74, 54), ((hash((x + off) / 4, course, 59) - 0.5) * 28.0) as i32)
        }
        18 => rgb(200, 194, 182),
        19 => rgb(160, 154, 144),
        52 => rgb(70, 72, 78),
        53..=55 => rgb(112, 116, 122),
        _ => {
            if y == 36 || x % 40 == 0 {
                return rgb(150, 146, 138); // panel reveals
            }
            // rain streaks under the coping
            let streak = noise(x * 4, 0, 3.0, 61) * ((y - 20) as f32 / 32.0);
            let c = grain(rgb(198, 194, 184), x, y, 67, 8.0);
            lift(c, -(streak * 26.0) as i32)
        }
    }
}

fn hedge(x: i32, y: i32) -> u32 {
    // ragged top edge against the sky
    let top = 40 + (noise(x, 0, 2.5, 71) * 4.0) as i32;
    if y > top {
        return CLEAR;
    }
    let n = noise(x, y, 2.0, 73) * 0.7 + hash(x, y, 79) * 0.3;
    if n < 0.25 {
        rgb(26, 48, 26)
    } else if n < 0.6 {
        rgb(46, 84, 40)
    } else if n < 0.85 {
        rgb(64, 108, 50)
    } else {
        rgb(96, 136, 62)
    }
}

fn column(x: i32, y: i32) -> u32 {
    let u = x % 20;
    if y < 24 && (u <= 1 || u >= 18) {
        return if u == 0 || u == 19 { rgb(150, 156, 160) } else { rgb(214, 220, 224) }; // corner guards
    }
    if y <= 2 {
        return rgb(80, 80, 82);
    }
    upper_paint(rgb(222, 220, 212), x, y, 83)
}

// ---------------------------------------------------------------- windows (20 × 56, one per metre)

fn window_tex(inside: bool) -> Tex {
    Tex::paint(20, WALL_TEX_H, |x, y| {
        let frame = if inside { rgb(170, 176, 182) } else { rgb(62, 60, 58) };
        match y {
            0..=16 => {
                if inside {
                    room_wall(x, y.min(16))
                } else {
                    facade(x, y)
                }
            }
            17 => if inside { rgb(214, 214, 208) } else { rgb(150, 146, 138) },
            18 => if inside { rgb(242, 242, 236) } else { rgb(204, 198, 188) }, // sill
            19 | 20 | 46 | 47 => frame,
            21..=45 => {
                if x <= 1 || x >= 18 || x == 9 || x == 10 {
                    return frame;
                }
                // half-lowered blinds over the top of the glass
                if inside && y >= 36 {
                    return if y % 2 == 0 { rgb(232, 230, 216) } else { rgb(196, 194, 182) };
                }
                GLASS
            }
            _ => {
                if inside {
                    upper_paint(rgb(228, 230, 222), x, y, 13)
                } else {
                    facade(x, y)
                }
            }
        }
    })
}

// ---------------------------------------------------------------- doors (20 × 42: 1 m × 2.1 m)

fn door_tex(s: DoorStyle) -> Tex {
    Tex::paint(20, 42, |x, y| {
        let frame = rgb(118, 122, 128);
        if x == 0 || x == 19 || y >= 41 {
            return frame;
        }
        if x == 1 || x == 18 || y == 40 {
            return rgb(86, 88, 92); // the shadow line round the leaf
        }
        let kick = (2..=6).contains(&y);
        let brushed = |x: i32, y: i32| lift(rgb(172, 176, 180), ((hash(x, y, 89) - 0.5) * 16.0) as i32);
        match s {
            DoorStyle::Wood => {
                if (12..=15).contains(&x) && (22..=34).contains(&y) {
                    return if x == 12 || x == 15 || y == 22 || y == 34 { rgb(80, 80, 86) } else { GLASS };
                }
                if kick {
                    return brushed(x, y);
                }
                if (20..=21).contains(&y) && (3..=6).contains(&x) {
                    return if y == 21 { rgb(196, 198, 202) } else { rgb(70, 72, 76) }; // lever
                }
                mix(rgb(198, 160, 110), rgb(170, 132, 88), noise(x, y * 5, 4.0, 97))
            }
            DoorStyle::Restroom => {
                if kick {
                    return brushed(x, y);
                }
                if (4..=7).contains(&x) && (22..=28).contains(&y) {
                    return brushed(x, y); // push plate
                }
                grain(rgb(190, 202, 212), x, y, 101, 5.0)
            }
            DoorStyle::Secure => {
                if x == 9 || x == 10 {
                    return rgb(70, 72, 76); // meeting stiles
                }
                let cx = if x < 10 { 5.0 } else { 14.0 };
                let d = ((x as f32 - cx).powi(2) + (y as f32 - 29.0).powi(2)).sqrt();
                if d < 2.6 {
                    return GLASS;
                }
                if d < 3.5 {
                    return rgb(60, 62, 66);
                }
                if y == 18 || y == 19 {
                    return rgb(176, 44, 40); // red "authorised only" band
                }
                if kick {
                    return lift(brushed(x, y), -10);
                }
                brushed(x, y)
            }
            DoorStyle::Glass => {
                if x <= 2 || x >= 17 || x == 9 || x == 10 || y <= 3 || y == 18 || y == 19 {
                    return lift(rgb(150, 156, 162), if y == 19 { 20 } else { 0 });
                }
                if (30..=31).contains(&y) && x % 2 == 0 {
                    return rgb(220, 228, 232); // frosted safety dots
                }
                GLASS
            }
            DoorStyle::Service => {
                if (19..=20).contains(&y) && (3..=16).contains(&x) {
                    return if y == 20 { rgb(200, 202, 204) } else { rgb(60, 62, 64) }; // push bar
                }
                grain(rgb(92, 108, 102), x, y, 103, 6.0)
            }
        }
    })
}

// ---------------------------------------------------------------- floors (world space, wrap both ways)

fn floor_tex(m: Material) -> Tex {
    match m {
        Material::Corridor => Tex::paint(48, 48, |x, y| {
            if x % 6 == 0 || y % 6 == 0 {
                return rgb(160, 156, 146);
            }
            let (tx, ty) = (x / 6, y / 6);
            let base = if hash(tx, ty, 107) > 0.78 { rgb(180, 182, 178) } else { rgb(204, 198, 182) };
            let fleck = hash(x, y, 109);
            if fleck > 0.93 { lift(base, -34) } else if fleck < 0.05 { lift(base, 16) } else { lift(base, ((hash(tx, ty, 113) - 0.5) * 10.0) as i32) }
        }),
        Material::Room => Tex::paint(40, 40, |x, y| {
            let base = mix(rgb(178, 192, 198), rgb(160, 176, 184), noise(x, y, 8.0, 127));
            let fleck = hash(x, y, 131);
            if fleck > 0.92 { lift(base, -30) } else if fleck < 0.06 { lift(base, 18) } else { base }
        }),
        Material::Tile => Tex::paint(40, 40, |x, y| {
            if x % 4 == 0 || y % 4 == 0 {
                return rgb(140, 146, 150);
            }
            let c = lift(rgb(200, 206, 210), ((hash(x / 4, y / 4, 137) - 0.5) * 14.0) as i32);
            if hash(x / 4, y / 4, 139) > 0.95 { lift(c, -26) } else { c }
        }),
        Material::Terrazzo => Tex::paint(40, 40, |x, y| {
            if x == 0 || y == 0 {
                return rgb(176, 146, 82); // brass divider strip
            }
            let chip = hash(x, y, 149);
            let base = rgb(204, 194, 174);
            if chip > 0.9 { rgb(244, 240, 232) } else if chip > 0.83 { rgb(150, 140, 126) } else if chip < 0.05 { rgb(96, 84, 74) } else { lift(base, ((noise(x, y, 7.0, 151) - 0.5) * 10.0) as i32) }
        }),
        Material::Epoxy => Tex::paint(40, 40, |x, y| grain(rgb(150, 182, 168), x, y, 157, 6.0)),
        Material::Carpet => Tex::paint(20, 20, |x, y| {
            let c = lift(rgb(70, 78, 96), ((hash(x, y, 163) - 0.5) * 22.0) as i32);
            if x % 10 == 0 || y % 10 == 0 { lift(c, -6) } else { c }
        }),
        Material::Concrete => Tex::paint(40, 40, |x, y| {
            if x == 0 || y == 0 {
                return rgb(118, 116, 110);
            }
            let c = mix(rgb(154, 152, 146), rgb(132, 130, 124), noise(x, y, 10.0, 167));
            lift(c, ((hash(x, y, 173) - 0.5) * 12.0) as i32)
        }),
        Material::Grass => Tex::paint(40, 40, |x, y| {
            let n = noise(x, y, 7.0, 179) * 0.6 + hash(x, y, 181) * 0.4;
            if n < 0.22 { rgb(44, 74, 36) } else if n < 0.55 { rgb(64, 104, 48) } else if n < 0.84 { rgb(80, 122, 56) } else if n < 0.96 { rgb(104, 142, 66) } else { rgb(132, 140, 72) }
        }),
        Material::Sidewalk => Tex::paint(60, 60, |x, y| {
            if x % 30 == 0 || y % 30 == 0 {
                return rgb(124, 120, 114);
            }
            lift(grain(rgb(172, 168, 160), x, y, 191, 10.0), ((hash(x / 30, y / 30, 193) - 0.5) * 10.0) as i32)
        }),
        Material::Asphalt | Material::Parking => Tex::paint(40, 40, move |x, y| {
            let base = if m == Material::Parking { rgb(62, 62, 66) } else { rgb(50, 50, 54) };
            let a = hash(x, y, 197);
            if a > 0.94 { lift(base, 26) } else if a < 0.05 { lift(base, -16) } else { lift(base, ((noise(x, y, 9.0, 199) - 0.5) * 10.0) as i32) }
        }),
        Material::Pavers => Tex::paint(40, 40, |x, y| {
            let row = y / 10;
            let off = if row % 2 == 0 { 0 } else { 5 };
            if y % 10 == 0 || (x + off) % 10 == 0 {
                return rgb(132, 124, 116);
            }
            lift(grain(rgb(178, 170, 158), x, y, 211, 8.0), ((hash((x + off) / 10, row, 223) - 0.5) * 18.0) as i32)
        }),
    }
}

// ---------------------------------------------------------------- the suspended ceiling (0.6 m tiles)

fn ceiling_tex() -> Tex {
    Tex::paint(24, 24, |x, y| {
        if x % 12 == 0 || y % 12 == 0 {
            return rgb(238, 238, 234); // T-bar
        }
        if x % 12 == 1 || y % 12 == 1 {
            return rgb(200, 200, 194); // its shadow on the tile
        }
        let fissure = hash(x, y, 227) > 0.86;
        let c = rgb(224, 224, 216);
        if fissure { lift(c, -20) } else { c }
    })
}

/// What has happened to one ceiling tile (0.6 m cell) since the hospital emptied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CeilingTile {
    Fine,
    /// A brown ring where water came through.
    Stained,
    /// Fallen out: you see the dark void, a pipe, cables.
    Missing,
    /// Slightly out of its grid, darker.
    Sagging,
}

pub fn ceiling_tile(cx: i32, cy: i32) -> CeilingTile {
    let h = hash(cx, cy, 229);
    if h > 0.985 {
        CeilingTile::Missing
    } else if h > 0.955 {
        CeilingTile::Stained
    } else if h > 0.93 {
        CeilingTile::Sagging
    } else {
        CeilingTile::Fine
    }
}

/// The ceiling at a point, `tile` from [`ceiling_tile`], `lx`/`ly` 0..1 within the tile.
pub fn ceiling_texel(base: u32, tile: CeilingTile, lx: f64, ly: f64) -> u32 {
    match tile {
        CeilingTile::Fine => base,
        CeilingTile::Sagging => lift(base, -14),
        CeilingTile::Stained => {
            let d = ((lx - 0.55).powi(2) + (ly - 0.45).powi(2)).sqrt();
            if (0.22..0.29).contains(&d) { mix(base, rgb(150, 126, 96), 0.45) } else if d < 0.22 { mix(base, rgb(196, 182, 152), 0.3) } else { base }
        }
        CeilingTile::Missing => {
            if (0.42..0.5).contains(&ly) {
                rgb(96, 94, 90) // a duct or pipe in the void
            } else if (lx * 10.0) as i32 % 4 == 0 && ly > 0.6 {
                rgb(46, 40, 34) // hanging cable
            } else {
                rgb(28, 28, 30)
            }
        }
    }
}

/// Lighter or darker by `k`, for callers outside this module.
pub fn scale(c: u32, k: f32) -> u32 {
    mul(c, k)
}

/// A straight blend between two colours.
pub fn blend(a: u32, b: u32, t: f32) -> u32 {
    mix(a, b, t)
}
