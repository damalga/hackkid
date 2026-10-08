//! Furniture and vehicles as solid boxes: axis-aligned blocks in metres, each face with its
//! own pixel art. Unlike flat sprites they keep their perspective as you walk round them,
//! and you see the top of anything below eye height.

use serde::{Deserialize, Serialize};

use crate::engine::textures::{hash, rgb};

/// Which way the front of a piece of furniture looks (the floor plan has north up).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Facing {
    North,
    East,
    South,
    West,
}

impl Facing {
    pub fn vector(self) -> (f64, f64) {
        match self {
            Facing::North => (0.0, -1.0),
            Facing::East => (1.0, 0.0),
            Facing::South => (0.0, 1.0),
            Facing::West => (-1.0, 0.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoxKind {
    Bed,
    Headboard,
    OpTable,
    Gurney,
    KitchenCounter,
    Stove,
    ServingCounter,
    NurseCounter,
    Reception,
    Fridge,
    VendingSnacks,
    VendingDrinks,
    Locker,
    Shelf,
    StoreShelf,
    FileCabinet,
    GlassCabinet,
    Desk,
    BedsideTable,
    CrashCart,
    Generator,
    CafeTable,
    WoodTable,
    SofaSeat,
    SofaBack,
    BenchSeat,
    BenchBack,
    PewSeat,
    PewBack,
    Car(u32),
    Ambulance,
    Dumpster,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoxFace {
    Front,
    Back,
    Left,
    Right,
    Top,
}

impl BoxKind {
    /// Width across the front, depth front to back, height; metres.
    pub fn size(self) -> (f64, f64, f64) {
        match self {
            BoxKind::Bed => (0.95, 2.1, 0.62),
            BoxKind::Headboard => (0.95, 0.06, 1.0),
            BoxKind::OpTable => (0.6, 1.95, 0.95),
            BoxKind::Gurney => (0.65, 1.95, 0.85),
            BoxKind::KitchenCounter | BoxKind::ServingCounter => (2.0, 0.65, 0.92),
            BoxKind::Stove => (0.8, 0.7, 0.92),
            BoxKind::NurseCounter => (2.4, 0.7, 1.1),
            BoxKind::Reception => (3.2, 0.8, 1.1),
            BoxKind::Fridge => (0.75, 0.7, 1.8),
            BoxKind::VendingSnacks | BoxKind::VendingDrinks => (0.95, 0.85, 1.85),
            BoxKind::Locker => (0.9, 0.5, 1.85),
            BoxKind::Shelf => (1.2, 0.5, 1.9),
            BoxKind::StoreShelf => (1.6, 0.55, 1.6),
            BoxKind::FileCabinet => (0.47, 0.62, 1.32),
            BoxKind::GlassCabinet => (0.9, 0.45, 1.9),
            BoxKind::Desk => (1.4, 0.7, 0.75),
            BoxKind::BedsideTable => (0.45, 0.45, 0.78),
            BoxKind::CrashCart => (0.75, 0.55, 1.05),
            BoxKind::Generator => (2.4, 1.2, 1.7),
            BoxKind::CafeTable => (1.2, 0.8, 0.75),
            BoxKind::WoodTable => (1.6, 0.9, 0.75),
            BoxKind::SofaSeat => (1.8, 0.8, 0.44),
            BoxKind::SofaBack => (1.8, 0.2, 0.85),
            BoxKind::BenchSeat => (1.6, 0.45, 0.46),
            BoxKind::BenchBack => (1.6, 0.07, 0.88),
            BoxKind::PewSeat => (2.6, 0.45, 0.46),
            BoxKind::PewBack => (2.6, 0.07, 0.95),
            BoxKind::Car(_) => (1.8, 4.3, 1.45),
            BoxKind::Ambulance => (2.1, 5.6, 2.6),
            BoxKind::Dumpster => (1.9, 1.1, 1.3),
        }
    }

    /// The colour of a face at (`u`, `v`) metres: `u` left to right as seen from in front
    /// of that face, `v` up from the bottom (on the top: from the front edge back).
    /// `w`, `h` are the face's size. `None` is a gap you can see through.
    pub fn texel(self, face: BoxFace, u: f64, v: f64, w: f64, h: f64) -> Option<u32> {
        let (nu, nv) = (u / w, v / h);
        let (ix, iy) = ((u * 20.0) as i32, (v * 20.0) as i32);
        match self {
            BoxKind::Bed => bed(face, u, v, w, h, nu, nv),
            BoxKind::Headboard => Some(if face == BoxFace::Top { rgb(150, 156, 160) } else { panel(rgb(196, 186, 160), nu, nv, 0.06) }),
            BoxKind::OpTable => op_table(face, nu, v, h),
            BoxKind::Gurney => gurney(face, nu, v, h),
            BoxKind::KitchenCounter | BoxKind::ServingCounter => counter(self, face, u, v, w, h, nu),
            BoxKind::Stove => stove(face, u, v, w, h),
            BoxKind::NurseCounter | BoxKind::Reception => desk_counter(self, face, u, v, w, h),
            BoxKind::Fridge => fridge(face, nu, nv, v, h),
            BoxKind::VendingSnacks | BoxKind::VendingDrinks => vending(self == BoxKind::VendingDrinks, face, nu, nv, ix, iy),
            BoxKind::Locker => locker(face, nu, nv, u, v),
            BoxKind::Shelf | BoxKind::StoreShelf => shelf(self == BoxKind::StoreShelf, face, u, v, h, ix),
            BoxKind::FileCabinet => file_cabinet(face, nu, nv),
            BoxKind::GlassCabinet => glass_cabinet(face, nu, nv, ix, iy),
            BoxKind::Desk => desk(face, u, v, w, h),
            BoxKind::BedsideTable => bedside(face, nu, nv),
            BoxKind::CrashCart => crash_cart(face, nu, nv),
            BoxKind::Generator => generator(face, u, v, w, h, nu, nv),
            BoxKind::CafeTable | BoxKind::WoodTable => table(self == BoxKind::WoodTable, face, nu, v, h, ix, iy),
            BoxKind::SofaSeat | BoxKind::SofaBack => sofa(self == BoxKind::SofaBack, face, u, v, w, h),
            BoxKind::BenchSeat | BoxKind::BenchBack | BoxKind::PewSeat | BoxKind::PewBack => {
                let pew = matches!(self, BoxKind::PewSeat | BoxKind::PewBack);
                bench(pew, matches!(self, BoxKind::BenchBack | BoxKind::PewBack), face, u, v, w, h)
            }
            BoxKind::Car(body) => car(body, false, face, u, v, w, h),
            BoxKind::Ambulance => car(rgb(236, 236, 230), true, face, u, v, w, h),
            BoxKind::Dumpster => dumpster(face, nu, nv, iy),
        }
    }
}

// ---------------------------------------------------------------- helpers

fn lift(c: u32, d: i32) -> u32 {
    let f = |v: u32| (v as i32 + d).clamp(0, 255) as u8;
    rgb(f((c >> 16) & 255), f((c >> 8) & 255), f(c & 255))
}

/// A flat panel with a lighter top-left and darker bottom-right edge.
fn panel(c: u32, nu: f64, nv: f64, edge: f64) -> u32 {
    if nu < edge || nv > 1.0 - edge {
        lift(c, 18)
    } else if nu > 1.0 - edge || nv < edge {
        lift(c, -22)
    } else {
        c
    }
}

fn grain(c: u32, ix: i32, iy: i32, seed: u32, amount: f32) -> u32 {
    lift(c, ((hash(ix, iy, seed) - 0.5) * amount) as i32)
}

/// Legs at the corners of a face: true where `u` is on a leg.
fn leg(nu: f64, width: f64) -> bool {
    nu < width || nu > 1.0 - width
}

// ---------------------------------------------------------------- art

fn bed(face: BoxFace, u: f64, v: f64, w: f64, h: f64, nu: f64, nv: f64) -> Option<u32> {
    let frame = rgb(120, 136, 150);
    match face {
        BoxFace::Top => {
            // pillow at the head (the back), blanket over the foot half
            let back = h - v;
            if back < 0.45 && (0.1..0.9).contains(&nu) {
                return Some(if back < 0.06 || back > 0.4 { rgb(214, 216, 216) } else { rgb(244, 244, 240) });
            }
            if v < 0.95 {
                let fold = ((v * 20.0) as i32 % 6 == 0) as i32;
                return Some(lift(rgb(120, 160, 196), -12 * fold));
            }
            if nu < 0.05 || nu > 0.95 {
                return Some(rgb(170, 176, 180)); // side rails
            }
            Some(grain(rgb(238, 240, 238), (u * 20.0) as i32, (v * 20.0) as i32, 601, 8.0))
        }
        _ => {
            if v < 0.16 {
                // wheels at the corners, open underneath
                let near_edge = if matches!(face, BoxFace::Left | BoxFace::Right) { u < 0.18 || u > w - 0.18 } else { leg(nu, 0.16) };
                return near_edge.then(|| if v < 0.1 { rgb(30, 30, 32) } else { rgb(90, 94, 98) });
            }
            if v > h - 0.14 {
                return Some(if face == BoxFace::Front && nv > 0.9 { rgb(120, 160, 196) } else { rgb(236, 238, 236) }); // mattress
            }
            if face == BoxFace::Front && (0.3..0.7).contains(&nu) && v > 0.25 {
                return Some(panel(rgb(196, 186, 160), (nu - 0.3) / 0.4, (v - 0.25) / (h - 0.4), 0.08)); // footboard with the chart
            }
            if v > h - 0.2 {
                return Some(rgb(178, 184, 188)); // side rail
            }
            Some(panel(frame, nu, nv, 0.03))
        }
    }
}

fn op_table(face: BoxFace, nu: f64, v: f64, h: f64) -> Option<u32> {
    match face {
        BoxFace::Top => Some(if nu < 0.06 || nu > 0.94 { rgb(150, 158, 162) } else { rgb(40, 70, 72) }),
        _ => {
            if v > h - 0.1 {
                Some(rgb(36, 60, 62)) // the pad's edge
            } else if v > h - 0.16 {
                Some(rgb(170, 176, 180))
            } else if (0.38..0.62).contains(&nu) || v < 0.06 {
                Some(lift(rgb(160, 166, 170), if nu < 0.45 { 16 } else { -10 })) // the column and its foot
            } else {
                None
            }
        }
    }
}

fn gurney(face: BoxFace, nu: f64, v: f64, h: f64) -> Option<u32> {
    match face {
        BoxFace::Top => Some(if nu < 0.08 || nu > 0.92 { rgb(170, 176, 180) } else { rgb(232, 236, 236) }),
        _ => {
            if v > h - 0.1 {
                Some(rgb(232, 236, 236))
            } else if v > h - 0.16 {
                Some(rgb(150, 156, 160))
            } else if v < 0.12 && leg(nu, 0.12) {
                Some(rgb(30, 30, 32))
            } else if leg(nu, 0.06) {
                Some(rgb(170, 176, 180))
            } else {
                None
            }
        }
    }
}

fn counter(kind: BoxKind, face: BoxFace, u: f64, v: f64, w: f64, h: f64, nu: f64) -> Option<u32> {
    let steel = kind == BoxKind::ServingCounter;
    let body = if steel { rgb(176, 182, 186) } else { rgb(196, 186, 168) };
    match face {
        BoxFace::Top => {
            if steel {
                // trays along the serving line
                let tray = ((u * 2.0) as i32 % 2 == 0) && (0.15..0.5).contains(&(v / h));
                return Some(if tray { rgb(120, 130, 136) } else { grain(rgb(196, 202, 206), (u * 40.0) as i32, (v * 20.0) as i32, 611, 10.0) });
            }
            // a sink set into the counter, about a third along
            let (su, sv) = (nu - 0.3, v / h - 0.5);
            if su.abs() < 0.1 && sv.abs() < 0.28 {
                return Some(if su.abs() < 0.085 && sv.abs() < 0.24 { rgb(120, 126, 130) } else { rgb(200, 204, 208) });
            }
            Some(grain(rgb(170, 166, 154), (u * 20.0) as i32, (v * 20.0) as i32, 613, 14.0))
        }
        BoxFace::Front => {
            if v < 0.1 {
                return Some(rgb(50, 50, 52)); // kick
            }
            if v > h - 0.04 {
                return Some(rgb(150, 146, 136));
            }
            // cabinet doors, 0.5 m each, with handles
            let du = (u * 2.0).fract();
            if !(0.04..=0.96).contains(&du) || v > h - 0.2 && v < h - 0.17 {
                return Some(lift(body, -36));
            }
            if (0.82..0.9).contains(&du) && (h - 0.32..h - 0.24).contains(&v) {
                return Some(rgb(90, 92, 96));
            }
            let _ = w;
            Some(if steel { grain(body, (u * 40.0) as i32, (v * 20.0) as i32, 617, 10.0) } else { body })
        }
        _ => Some(lift(body, -14)),
    }
}

fn stove(face: BoxFace, u: f64, v: f64, w: f64, h: f64) -> Option<u32> {
    match face {
        BoxFace::Top => {
            let (cu, cv) = ((u / w * 2.0).fract() - 0.5, (v / h * 2.0).fract() - 0.5);
            let d = (cu * cu + cv * cv).sqrt();
            Some(if d < 0.3 && d > 0.18 { rgb(30, 30, 30) } else if d <= 0.18 { rgb(60, 60, 62) } else { rgb(186, 190, 194) })
        }
        BoxFace::Front => Some(if v < 0.1 {
            rgb(40, 40, 42)
        } else if v > h - 0.12 {
            if ((u * 10.0) as i32) % 2 == 0 { rgb(30, 30, 32) } else { rgb(186, 190, 194) } // knobs
        } else if (0.25..h - 0.25).contains(&v) && (0.1..w - 0.1).contains(&u) {
            rgb(40, 40, 44) // oven window
        } else {
            rgb(196, 200, 204)
        }),
        _ => Some(rgb(170, 174, 178)),
    }
}

fn desk_counter(kind: BoxKind, face: BoxFace, u: f64, v: f64, w: f64, h: f64) -> Option<u32> {
    let wood = if kind == BoxKind::Reception { rgb(150, 104, 62) } else { rgb(186, 178, 160) };
    match face {
        BoxFace::Top => {
            // a transaction ledge along the front, the lower work surface behind it
            if v < 0.25 {
                return Some(lift(wood, 20));
            }
            let paper = hash((u * 4.0) as i32, (v * 4.0) as i32, 621) > 0.75;
            Some(if paper { rgb(240, 240, 234) } else { rgb(200, 200, 194) })
        }
        BoxFace::Front => {
            if v < 0.08 {
                return Some(rgb(50, 46, 42));
            }
            if v > h - 0.06 {
                return Some(lift(wood, 26));
            }
            if kind == BoxKind::Reception {
                let text = "RECEPTION";
                let tw = text.len() as f64 * 0.16;
                let (x0, y0) = ((w - tw) / 2.0, h * 0.55);
                let (lx, ly) = (u - x0, v - y0);
                if (0.0..tw).contains(&lx) && (0.0..0.2).contains(&ly) {
                    let ci = (lx / 0.16) as usize;
                    let gx = ((lx % 0.16) / 0.04) as usize;
                    let gy = 4 - ((ly / 0.04) as usize).min(4);
                    if gx < 3 && glyph_row(text.as_bytes()[ci] as char, gy) & (4 >> gx) != 0 {
                        return Some(rgb(214, 180, 90)); // brass letters
                    }
                }
            }
            let strip = (h - 0.25..h - 0.2).contains(&v);
            Some(if strip { lift(wood, -30) } else { grain(wood, (u * 40.0) as i32, (v * 4.0) as i32, 623, 10.0) })
        }
        _ => Some(lift(wood, -18)),
    }
}

fn glyph_row(c: char, row: usize) -> u8 {
    let g: [u8; 5] = match c {
        'R' => [6, 5, 6, 5, 5], 'E' => [7, 4, 6, 4, 7], 'C' => [3, 4, 4, 4, 3], 'P' => [6, 5, 6, 4, 4],
        'T' => [7, 2, 2, 2, 2], 'I' => [7, 2, 2, 2, 7], 'O' => [2, 5, 5, 5, 2], 'N' => [6, 5, 5, 5, 5],
        'A' => [2, 5, 7, 5, 5], 'M' => [5, 7, 7, 5, 5], 'B' => [6, 5, 6, 5, 6], 'U' => [5, 5, 5, 5, 7],
        'L' => [4, 4, 4, 4, 7], 'S' => [3, 4, 2, 1, 6],
        _ => [0; 5],
    };
    g[row]
}

fn fridge(face: BoxFace, nu: f64, nv: f64, v: f64, h: f64) -> Option<u32> {
    let white = rgb(236, 238, 236);
    match face {
        BoxFace::Front => {
            if v < 0.06 {
                return Some(rgb(60, 60, 62));
            }
            if (h * 0.64..h * 0.655).contains(&v) {
                return Some(rgb(150, 152, 150)); // freezer split
            }
            if (0.82..0.88).contains(&nu) && ((0.4..0.6).contains(&nv) || (0.72..0.85).contains(&nv)) {
                return Some(rgb(160, 164, 166)); // handles
            }
            Some(panel(white, nu, nv, 0.03))
        }
        BoxFace::Top => Some(rgb(222, 224, 222)),
        _ => Some(lift(white, -16)),
    }
}

fn vending(drinks: bool, face: BoxFace, nu: f64, nv: f64, ix: i32, iy: i32) -> Option<u32> {
    let body = if drinks { rgb(176, 36, 40) } else { rgb(36, 70, 140) };
    match face {
        BoxFace::Front => {
            if nv < 0.06 {
                return Some(rgb(30, 30, 32));
            }
            if (0.72..0.95).contains(&nu) {
                // the keypad and coin slot column
                return Some(if (0.55..0.7).contains(&nv) && (0.78..0.9).contains(&nu) { rgb(40, 40, 44) } else { lift(body, -20) });
            }
            if nu < 0.06 || nu > 0.95 || nv > 0.94 || (0.1..0.2).contains(&nv) {
                return Some(lift(body, if nv > 0.94 { 20 } else { 0 }));
            }
            if nv < 0.2 {
                return Some(rgb(20, 20, 22)); // the pickup flap
            }
            // rows of products behind the glass, lit from inside
            let row = (nv * 7.0) as i32;
            if (nv * 7.0).fract() < 0.18 {
                return Some(rgb(150, 156, 162));
            }
            let item = hash(ix / 2, row, if drinks { 631 } else { 633 });
            let c = if drinks {
                [rgb(200, 30, 30), rgb(30, 90, 190), rgb(240, 240, 240), rgb(40, 160, 70)][(item * 4.0) as usize % 4]
            } else {
                [rgb(230, 180, 40), rgb(200, 80, 40), rgb(120, 60, 30), rgb(60, 140, 200), rgb(230, 230, 220)][(item * 5.0) as usize % 5]
            };
            Some(if (ix + iy) % 3 == 0 { lift(c, 30) } else { c })
        }
        BoxFace::Top => Some(rgb(40, 40, 44)),
        _ => Some(panel(body, nu, nv, 0.02)),
    }
}

fn locker(face: BoxFace, nu: f64, nv: f64, u: f64, v: f64) -> Option<u32> {
    let steel = rgb(110, 130, 150);
    match face {
        BoxFace::Front => {
            if nv < 0.04 {
                return Some(rgb(50, 54, 60));
            }
            let du = (nu * 2.0).fract();
            if !(0.04..=0.96).contains(&du) {
                return Some(rgb(60, 70, 80));
            }
            // vents near the top and bottom, a handle, a number plate
            let vent = ((0.82..0.92).contains(&nv) || (0.1..0.2).contains(&nv)) && (v * 40.0) as i32 % 2 == 0 && (0.25..0.75).contains(&du);
            if vent {
                return Some(rgb(50, 56, 64));
            }
            if (0.75..0.85).contains(&du) && (0.48..0.56).contains(&nv) {
                return Some(rgb(190, 194, 198));
            }
            if (0.3..0.7).contains(&du) && (0.7..0.74).contains(&nv) {
                return Some(rgb(230, 230, 220));
            }
            let _ = u;
            Some(steel)
        }
        BoxFace::Top => Some(lift(steel, -30)),
        _ => Some(lift(steel, -14)),
    }
}

fn shelf(store: bool, face: BoxFace, u: f64, v: f64, h: f64, ix: i32) -> Option<u32> {
    let frame = if store { rgb(220, 222, 220) } else { rgb(150, 156, 160) };
    let levels = if store { 4.0 } else { 5.0 };
    let lv = v / h * levels;
    let level = lv as i32;
    if lv.fract() < 0.06 || v < 0.08 {
        return Some(frame);
    }
    match face {
        BoxFace::Top => Some(lift(frame, -30)),
        BoxFace::Front | BoxFace::Back => {
            if u < 0.04 || (u * 20.0) as i32 % 24 == 23 {
                return Some(lift(frame, -20)); // uprights
            }
            // boxes and packs, different on every shelf
            let slot = ix / if store { 3 } else { 5 };
            let k = hash(slot, level, if store { 641 } else { 643 });
            if lv.fract() > 0.85 || k < 0.12 {
                return Some(rgb(40, 40, 42)); // gap: the back of the shelf
            }
            let c = if store {
                [rgb(230, 60, 50), rgb(250, 210, 60), rgb(60, 150, 220), rgb(240, 240, 236), rgb(80, 180, 90), rgb(230, 120, 180)][(k * 6.0) as usize % 6]
            } else {
                [rgb(180, 140, 90), rgb(236, 236, 230), rgb(90, 140, 200), rgb(200, 170, 120)][(k * 4.0) as usize % 4]
            };
            Some(if (ix % (if store { 3 } else { 5 })) == 0 { lift(c, -30) } else { c })
        }
        _ => Some(if (u * 20.0) as i32 % 10 == 0 { frame } else { rgb(40, 40, 42) }),
    }
}

fn file_cabinet(face: BoxFace, nu: f64, nv: f64) -> Option<u32> {
    let c = rgb(160, 164, 160);
    match face {
        BoxFace::Front => {
            let dv = (nv * 4.0).fract();
            if dv < 0.04 {
                return Some(rgb(90, 92, 90));
            }
            if (0.4..0.6).contains(&nu) && (0.6..0.7).contains(&dv) {
                return Some(rgb(70, 72, 70)); // handle
            }
            if (0.38..0.62).contains(&nu) && (0.78..0.88).contains(&dv) {
                return Some(rgb(230, 228, 214)); // label
            }
            Some(c)
        }
        BoxFace::Top => Some(lift(c, -20)),
        _ => Some(lift(c, -12)),
    }
}

fn glass_cabinet(face: BoxFace, nu: f64, nv: f64, ix: i32, iy: i32) -> Option<u32> {
    let white = rgb(232, 234, 232);
    match face {
        BoxFace::Front => {
            if nu < 0.05 || nu > 0.95 || (0.48..0.52).contains(&nu) || nv < 0.05 || nv > 0.96 || (0.44..0.47).contains(&nv) {
                return Some(white);
            }
            if nv < 0.44 {
                return Some(panel(lift(white, -10), nu, nv / 0.44, 0.05)); // solid lower doors
            }
            // bottles behind glass
            let row = ((nv - 0.47) * 8.0) as i32;
            if ((nv - 0.47) * 8.0).fract() < 0.1 {
                return Some(rgb(200, 210, 214));
            }
            let k = hash(ix / 2, row, 651);
            if k > 0.45 {
                let c = [rgb(150, 90, 40), rgb(230, 230, 230), rgb(60, 120, 180), rgb(200, 60, 60)][(k * 7.0) as usize % 4];
                return Some(if iy % 4 == 0 { lift(c, 30) } else { c });
            }
            Some(rgb(150, 176, 186))
        }
        BoxFace::Top => Some(white),
        _ => Some(lift(white, -14)),
    }
}

fn desk(face: BoxFace, u: f64, v: f64, w: f64, h: f64) -> Option<u32> {
    let wood = rgb(176, 140, 96);
    match face {
        BoxFace::Top => {
            let (nu, nv) = (u / w, v / h);
            if (0.3..0.65).contains(&nu) && (0.15..0.35).contains(&nv) {
                return Some(rgb(50, 50, 54)); // keyboard
            }
            if (0.05..0.25).contains(&nu) && (0.2..0.7).contains(&nv) {
                return Some(rgb(242, 242, 236)); // papers
            }
            Some(grain(wood, (u * 20.0) as i32, (v * 5.0) as i32, 661, 14.0))
        }
        BoxFace::Front => {
            if v > h - 0.04 {
                return Some(lift(wood, 20));
            }
            if u > w - 0.45 {
                // the drawer pedestal
                let dv = (v / (h - 0.04) * 3.0).fract();
                if dv < 0.05 {
                    return Some(lift(wood, -40));
                }
                if (0.55..0.7).contains(&dv) && ((u - (w - 0.45)) / 0.45 - 0.5).abs() < 0.15 {
                    return Some(rgb(70, 66, 60));
                }
                return Some(wood);
            }
            if v < 0.3 {
                return None; // knee space
            }
            Some(lift(wood, -24))
        }
        _ => Some(if v > h - 0.04 { lift(wood, 20) } else { lift(wood, -30) }),
    }
}

fn bedside(face: BoxFace, nu: f64, nv: f64) -> Option<u32> {
    let c = rgb(200, 196, 182);
    match face {
        BoxFace::Front => {
            if (0.7..0.73).contains(&nv) || nv < 0.06 {
                return Some(lift(c, -40));
            }
            if (0.4..0.6).contains(&nu) && (0.8..0.86).contains(&nv) {
                return Some(rgb(110, 110, 112));
            }
            Some(panel(c, nu, nv, 0.06))
        }
        BoxFace::Top => {
            let d = ((nu - 0.65).powi(2) + (nv - 0.4).powi(2)).sqrt();
            Some(if d < 0.14 { rgb(110, 170, 210) } else { lift(c, 10) }) // a water jug
        }
        _ => Some(lift(c, -14)),
    }
}

fn crash_cart(face: BoxFace, nu: f64, nv: f64) -> Option<u32> {
    let red = rgb(196, 40, 36);
    match face {
        BoxFace::Front => {
            if nv < 0.08 {
                return (nu < 0.12 || nu > 0.88).then_some(rgb(30, 30, 32));
            }
            let dv = ((nv - 0.08) / 0.9 * 5.0).fract();
            if dv < 0.06 {
                return Some(lift(red, -50));
            }
            if (0.4..0.6).contains(&nu) && (0.5..0.62).contains(&dv) {
                return Some(rgb(220, 220, 220));
            }
            Some(red)
        }
        BoxFace::Top => Some(if (0.15..0.85).contains(&nu) && (0.2..0.8).contains(&nv) { rgb(70, 72, 76) } else { rgb(230, 230, 230) }),
        _ => Some(if nv < 0.08 { rgb(30, 30, 32) } else { lift(red, -24) }),
    }
}

fn generator(face: BoxFace, u: f64, v: f64, w: f64, h: f64, nu: f64, nv: f64) -> Option<u32> {
    let yellow = rgb(210, 170, 40);
    match face {
        BoxFace::Top => {
            let d = ((nu - 0.75).powi(2) + (v / h - 0.5).powi(2)).sqrt();
            Some(if d < 0.08 { rgb(30, 30, 30) } else { lift(yellow, -20) })
        }
        _ => {
            if v < 0.12 {
                return Some(rgb(50, 52, 54)); // skid
            }
            if face == BoxFace::Front && (0.08..0.32).contains(&nu) && (0.45..0.85).contains(&nv) {
                // the control panel: gauges and a red stop button
                let (lu, lv) = ((nu - 0.08) / 0.24, (nv - 0.45) / 0.4);
                let gauge = ((lu * 3.0).fract() - 0.5).abs() < 0.3 && (0.55..0.85).contains(&lv);
                return Some(if gauge { rgb(236, 236, 228) } else if (0.4..0.6).contains(&lu) && lv < 0.3 { rgb(200, 30, 30) } else { rgb(70, 74, 78) });
            }
            if (0.12..0.2).contains(&v) {
                return Some(if ((u * 10.0) as i32) % 2 == 0 { rgb(30, 30, 30) } else { yellow }); // hazard stripe
            }
            // louvres
            if face != BoxFace::Front && (0.4..0.85).contains(&nv) && (v * 20.0) as i32 % 2 == 0 {
                return Some(lift(yellow, -40));
            }
            let _ = w;
            Some(panel(yellow, nu, nv, 0.02))
        }
    }
}

fn table(wood: bool, face: BoxFace, nu: f64, v: f64, h: f64, ix: i32, iy: i32) -> Option<u32> {
    let top = if wood { rgb(170, 128, 84) } else { rgb(214, 210, 200) };
    match face {
        BoxFace::Top => Some(if wood { grain(top, ix, iy / 4, 671, 16.0) } else { grain(top, ix, iy, 673, 8.0) }),
        _ => {
            if v > h - 0.04 {
                Some(lift(top, -30))
            } else if leg(nu, 0.05) {
                Some(rgb(70, 70, 72))
            } else {
                None
            }
        }
    }
}

fn sofa(back: bool, face: BoxFace, u: f64, v: f64, w: f64, h: f64) -> Option<u32> {
    let fabric = rgb(70, 120, 120);
    if face == BoxFace::Top {
        let cushion = (u / w * 3.0).fract();
        return Some(if back { lift(fabric, -16) } else if !(0.03..=0.97).contains(&cushion) { lift(fabric, -36) } else { lift(fabric, 12) });
    }
    if !back && v < 0.08 {
        let nu = u / w;
        return leg(nu, 0.04).then_some(rgb(40, 34, 30));
    }
    if face == BoxFace::Front || face == BoxFace::Back {
        let cushion = (u / w * 3.0).fract();
        if !(0.02..=0.98).contains(&cushion) {
            return Some(lift(fabric, -30));
        }
        if back && v > h - 0.08 {
            return Some(lift(fabric, 14));
        }
    }
    Some(if v / h > 0.85 { lift(fabric, 10) } else { fabric })
}

fn bench(pew: bool, back: bool, face: BoxFace, u: f64, v: f64, w: f64, h: f64) -> Option<u32> {
    let wood = if pew { rgb(120, 80, 46) } else { rgb(170, 130, 86) };
    let seat = if pew { wood } else { rgb(60, 90, 140) };
    if back {
        return Some(match face {
            BoxFace::Top => lift(wood, 16),
            _ => {
                if v < 0.46 {
                    return if leg(u / w, 0.04) || pew { Some(lift(wood, -24)) } else { None };
                }
                if pew { grain(wood, (u * 40.0) as i32, (v * 4.0) as i32, 681, 12.0) } else { seat }
            }
        });
    }
    match face {
        BoxFace::Top => Some(if pew { lift(wood, 10) } else { seat }),
        _ => {
            if v > h - 0.08 {
                Some(if pew { lift(wood, -10) } else { lift(seat, -20) })
            } else if leg(u / w, 0.04) || (pew && v < 0.06) {
                Some(rgb(60, 60, 62))
            } else {
                None
            }
        }
    }
}

fn car(body: u32, ambulance: bool, face: BoxFace, u: f64, v: f64, w: f64, h: f64) -> Option<u32> {
    let glass = rgb(40, 52, 62);
    let glass_hi = rgb(110, 130, 146);
    let belt = if ambulance { h * 0.62 } else { h * 0.58 };
    match face {
        BoxFace::Top => {
            // windscreen and rear window at the ends of the roof
            let nv = v / h;
            if ambulance {
                return Some(if nv < 0.12 { glass } else if (0.4..0.48).contains(&nv) && (u / w - 0.5).abs() < 0.2 { rgb(40, 80, 200) } else { body });
            }
            Some(if nv < 0.28 || nv > 0.82 { glass } else { lift(body, 8) })
        }
        BoxFace::Left | BoxFace::Right => {
            let nu = u / w;
            // wheels, and the gap under the body between them
            let (w1, w2) = (0.18 * w, 0.82 * w);
            for wx in [w1, w2] {
                let d = ((u - wx).powi(2) + (v - 0.32).powi(2)).sqrt();
                if d < 0.32 {
                    return Some(if d < 0.14 { rgb(170, 172, 176) } else { rgb(24, 24, 26) });
                }
            }
            if v < 0.18 {
                return None;
            }
            if ambulance {
                if (h * 0.4..h * 0.48).contains(&v) {
                    return Some(rgb(200, 30, 30)); // the stripe
                }
                if v > belt && nu < 0.2 {
                    return Some(if (u * 10.0) as i32 % 5 == 0 { glass_hi } else { glass });
                }
                return Some(if v > h - 0.12 { lift(body, -20) } else { body });
            }
            if v > belt {
                let cabin = (0.22..0.78).contains(&nu);
                if !cabin {
                    return None; // the bonnet and boot are lower than the roof
                }
                if v > h - 0.06 || !(0.25..=0.75).contains(&nu) || (0.49..0.51).contains(&nu) {
                    return Some(body);
                }
                return Some(if ((u + v) * 10.0) as i32 % 6 == 0 { glass_hi } else { glass });
            }
            if (belt - 0.04..belt).contains(&v) {
                return Some(lift(body, 30));
            }
            Some(if (0.5..0.51).contains(&nu) { lift(body, -40) } else { body })
        }
        BoxFace::Front | BoxFace::Back => {
            let nu = u / w;
            let front = face == BoxFace::Front;
            if v < 0.18 {
                return (nu < 0.15 || nu > 0.85).then_some(rgb(24, 24, 26));
            }
            if v > belt {
                if ambulance {
                    if v > h - 0.15 {
                        return Some(if (nu * 6.0) as i32 % 2 == 0 { rgb(220, 40, 40) } else { rgb(40, 80, 220) }); // light bar
                    }
                    return Some(if front && (0.08..0.92).contains(&nu) && v < h - 0.3 { glass } else { body });
                }
                // windscreen inset from the sides
                if !(0.1..=0.9).contains(&nu) {
                    return None;
                }
                if v > h - 0.06 {
                    return Some(body);
                }
                return Some(if ((u - v) * 10.0) as i32 % 7 == 0 { glass_hi } else { glass });
            }
            // lights at the corners, grille or plate in the middle
            let lamp = !(0.18..=0.82).contains(&nu) && (belt - 0.22..belt - 0.08).contains(&v);
            if lamp {
                return Some(if front { rgb(240, 240, 220) } else { rgb(200, 30, 30) });
            }
            if (0.38..0.62).contains(&nu) && (0.3..0.4).contains(&v) {
                return Some(if front { rgb(40, 40, 42) } else { rgb(236, 236, 220) });
            }
            Some(body)
        }
    }
}

fn dumpster(face: BoxFace, nu: f64, nv: f64, iy: i32) -> Option<u32> {
    let green = rgb(40, 96, 60);
    match face {
        BoxFace::Top => Some(if (0.49..0.51).contains(&nu) { rgb(20, 40, 26) } else { rgb(30, 70, 44) }),
        _ => {
            if nv < 0.1 {
                return (nu < 0.12 || nu > 0.88).then_some(rgb(24, 24, 26));
            }
            if iy % 5 == 0 {
                return Some(lift(green, -24)); // ribs
            }
            Some(panel(green, nu, nv, 0.03))
        }
    }
}

/// A piece of box furniture placed in the world.
#[derive(Debug, Clone, Copy)]
pub struct BoxInput {
    pub kind: BoxKind,
    pub facing: Facing,
    /// Centre in the floor plan.
    pub x: f64,
    pub y: f64,
    /// Bottom height (metres above the floor).
    pub z0: f64,
    pub w: f64,
    pub d: f64,
    pub h: f64,
}

impl BoxInput {
    pub fn new(kind: BoxKind, x: f64, y: f64, facing: Facing) -> Self {
        let (w, d, h) = kind.size();
        Self { kind, facing, x, y, z0: 0.0, w, d, h }
    }

    /// Floor-plan bounds: (min x, min y, max x, max y).
    pub fn bounds(&self) -> (f64, f64, f64, f64) {
        let (hx, hy) = match self.facing {
            Facing::North | Facing::South => (self.w / 2.0, self.d / 2.0),
            Facing::East | Facing::West => (self.d / 2.0, self.w / 2.0),
        };
        (self.x - hx, self.y - hy, self.x + hx, self.y + hy)
    }

    /// The box face whose outward normal is `n`, and the face's width.
    pub fn face_for(&self, n: (i32, i32)) -> (BoxFace, f64) {
        let f = self.facing.vector();
        let (fx, fy) = (f.0.round() as i32, f.1.round() as i32);
        if n == (fx, fy) {
            (BoxFace::Front, self.w)
        } else if n == (-fx, -fy) {
            (BoxFace::Back, self.w)
        } else if n == (fy, -fx) {
            // seen from in front, this side is on the viewer's right
            (BoxFace::Right, self.d)
        } else {
            (BoxFace::Left, self.d)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn faces_follow_the_facing() {
        let b = BoxInput::new(BoxKind::Desk, 5.0, 5.0, Facing::South);
        assert_eq!(b.face_for((0, 1)).0, BoxFace::Front);
        assert_eq!(b.face_for((0, -1)).0, BoxFace::Back);
        // facing south, the viewer looks north: their left is west
        assert_eq!(b.face_for((-1, 0)).0, BoxFace::Left);
        assert_eq!(b.face_for((1, 0)).0, BoxFace::Right);
        let (x0, y0, x1, y1) = b.bounds();
        assert!((x1 - x0 - 1.4).abs() < 1e-9 && (y1 - y0 - 0.7).abs() < 1e-9);
    }
}
