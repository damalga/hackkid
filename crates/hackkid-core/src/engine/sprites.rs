//! Flat sprites that always turn to face the camera: people, plants, and small or spindly
//! things a box can't show. Sizes are real, in metres; art is sampled at `u`, `v` in 0..1
//! from the top-left, returning `None` where the sprite is see-through.

pub type Rgb = (u8, u8, u8);

#[derive(Debug, Clone)]
pub enum SpriteShape {
    Backpack,
    ClothingPile { color: Rgb },
    HangingClothing { color: Rgb },
    CoatRack,
    Person(PersonLook),
    DoorLeaf { color: Rgb },
    Toilet,
    ToiletWithPaper,
    Sink,
    Shower,
    Urinal,
    GownRack,
    InstrumentTable,
    TreeOak,
    TreePine,
    Bush,
    Weed,
    BusStop,
    Mailbox,
    IvStand,
    Monitor,
    Wheelchair,
    Chair { color: Rgb },
    TrashBin,
    StreetLamp,
    Plant,
    Curtain,
    SurgicalLight,
}

/// A sprite placed in the world: its foot at (`x`, `y`), `elevation` metres up.
pub struct SpriteInput {
    pub x: f64,
    pub y: f64,
    pub shape: SpriteShape,
    pub w: f64,
    pub h: f64,
    pub elevation: f64,
}

impl SpriteInput {
    pub fn new(x: f64, y: f64, shape: SpriteShape) -> Self {
        let (w, h) = shape.size();
        Self { x, y, shape, w, h, elevation: 0.0 }
    }

    pub fn raised(mut self, elevation: f64) -> Self {
        self.elevation = elevation;
        self
    }

    pub fn scaled(mut self, k: f64) -> Self {
        self.w *= k;
        self.h *= k;
        self
    }
}

impl SpriteShape {
    pub fn sample(&self, u: f64, v: f64) -> Option<Rgb> {
        match self {
            SpriteShape::Backpack => backpack_pixel(u, v),
            SpriteShape::ClothingPile { color } => clothing_pixel(u, v, *color),
            SpriteShape::HangingClothing { color } => hanging_pixel(u, v, *color),
            SpriteShape::CoatRack => coat_rack_pixel(u, v),
            SpriteShape::Person(look) => person_pixel(u, v, look),
            SpriteShape::DoorLeaf { color } => door_leaf_pixel(u, v, *color),
            SpriteShape::Toilet => toilet_pixel(u, v, false),
            SpriteShape::ToiletWithPaper => toilet_pixel(u, v, true),
            SpriteShape::Sink => sink_pixel(u, v),
            SpriteShape::Shower => shower_pixel(u, v),
            SpriteShape::Urinal => urinal_pixel(u, v),
            SpriteShape::GownRack => burra_pixel(u, v),
            SpriteShape::InstrumentTable => aluminum_table_pixel(u, v),
            SpriteShape::TreeOak => tree_oak_pixel(u, v),
            SpriteShape::TreePine => tree_pine_pixel(u, v),
            SpriteShape::Bush => bush_pixel(u, v),
            SpriteShape::Weed => weed_pixel(u, v),
            SpriteShape::BusStop => bus_stop_pixel(u, v),
            SpriteShape::Mailbox => mailbox_pixel(u, v),
            SpriteShape::IvStand => iv_stand_pixel(u, v),
            SpriteShape::Monitor => monitor_pixel(u, v),
            SpriteShape::Wheelchair => wheelchair_pixel(u, v),
            SpriteShape::Chair { color } => chair_pixel(u, v, *color),
            SpriteShape::TrashBin => bin_pixel(u, v),
            SpriteShape::StreetLamp => lamp_pixel(u, v),
            SpriteShape::Plant => plant_pixel(u, v),
            SpriteShape::Curtain => curtain_pixel(u, v),
            SpriteShape::SurgicalLight => surgical_light_pixel(u, v),
        }
    }

    /// Parts that give their own light: screens, lamps.
    pub fn glows(&self, u: f64, v: f64) -> bool {
        match self {
            SpriteShape::Monitor => (0.12..0.88).contains(&u) && (0.04..0.26).contains(&v),
            SpriteShape::StreetLamp => (0.55..0.95).contains(&u) && (0.09..0.12).contains(&v),
            SpriteShape::SurgicalLight => v > 0.62 && v < 0.75,
            _ => false,
        }
    }

    /// Texture density relative to everything else: people are drawn finer, so faces
    /// and clothes read up close.
    pub fn detail(&self) -> f64 {
        match self {
            SpriteShape::Person(_) => 2.75,
            _ => 1.0,
        }
    }

    /// Width and height in metres.
    pub fn size(&self) -> (f64, f64) {
        match self {
            SpriteShape::Backpack => (0.42, 0.5),
            SpriteShape::ClothingPile { .. } => (0.75, 0.22),
            SpriteShape::HangingClothing { .. } => (0.5, 0.85),
            SpriteShape::CoatRack => (0.45, 1.75),
            SpriteShape::Person(look) => (0.58 * look.build, look.height),
            SpriteShape::DoorLeaf { .. } => (0.07, 2.05),
            SpriteShape::Toilet | SpriteShape::ToiletWithPaper => (0.45, 0.8),
            SpriteShape::Sink => (0.55, 0.4),
            SpriteShape::Shower => (0.85, 2.05),
            SpriteShape::Urinal => (0.4, 0.65),
            SpriteShape::GownRack => (1.3, 1.6),
            SpriteShape::InstrumentTable => (0.6, 0.9),
            SpriteShape::TreeOak => (5.0, 6.5),
            SpriteShape::TreePine => (3.2, 7.5),
            SpriteShape::Bush => (1.4, 1.0),
            SpriteShape::Weed => (0.4, 0.35),
            SpriteShape::BusStop => (2.6, 2.4),
            SpriteShape::Mailbox => (0.5, 1.15),
            SpriteShape::IvStand => (0.45, 1.95),
            SpriteShape::Monitor => (0.5, 1.5),
            SpriteShape::Wheelchair => (0.75, 0.95),
            SpriteShape::Chair { .. } => (0.5, 0.85),
            SpriteShape::TrashBin => (0.38, 0.6),
            SpriteShape::StreetLamp => (0.9, 4.6),
            SpriteShape::Plant => (0.6, 1.3),
            SpriteShape::Curtain => (1.9, 1.9),
            SpriteShape::SurgicalLight => (0.9, 0.9),
        }
    }
}

fn tree_oak_pixel(u: f64, v: f64) -> Option<(u8, u8, u8)> {
    if u < 0.0 || u > 1.0 || v < 0.0 || v > 1.0 { return None; }
    let trunk = (95, 65, 40);
    let dark_trunk = (55, 40, 25);
    let leaf1 = (60, 130, 55);
    let leaf2 = (85, 155, 65);
    let leaf3 = (40, 100, 40);
    // Trunk
    if v > 0.72 {
        if (u - 0.5).abs() < 0.06 { return Some(trunk); }
        if (u - 0.5).abs() < 0.09 { return Some(dark_trunk); }
        return None;
    }
    // Foliage: three overlapping circles
    let du = u - 0.5;
    let dv = v - 0.35;
    let r = (du * du + dv * dv * 1.8).sqrt();
    if r < 0.42 {
        let noise = (u * 8.0).sin() * (v * 6.0).cos() * 0.5 + 0.5;
        if noise < 0.33 { return Some(leaf3); }
        if noise < 0.66 { return Some(leaf1); }
        return Some(leaf2);
    }
    None
}

fn tree_pine_pixel(u: f64, v: f64) -> Option<(u8, u8, u8)> {
    if u < 0.0 || u > 1.0 || v < 0.0 || v > 1.0 { return None; }
    let trunk = (85, 55, 30);
    let pine1 = (40, 90, 45);
    let pine2 = (55, 110, 55);
    // Trunk
    if v > 0.78 {
        if (u - 0.5).abs() < 0.05 { return Some(trunk); }
        return None;
    }
    // Triangular tiers
    let du = (u - 0.5).abs();
    let stack = (v * 3.0) as i32;
    let tier_v = v * 3.0 - stack as f64;
    let width = 0.20 + (stack as f64) * 0.14 + tier_v * 0.10;
    if du < width && v < 0.78 {
        if tier_v < 0.15 || du > width * 0.85 { return Some(pine1); }
        return Some(pine2);
    }
    None
}

fn weed_pixel(u: f64, v: f64) -> Option<(u8, u8, u8)> {
    if u < 0.0 || u > 1.0 || v < 0.0 || v > 1.0 { return None; }
    if v < 0.15 { return None; }
    let blades: &[(f64, f64, f64)] = &[
        (0.20, 0.55, 0.020),
        (0.32, 0.30, 0.018),
        (0.42, 0.45, 0.020),
        (0.50, 0.20, 0.022),
        (0.58, 0.42, 0.019),
        (0.68, 0.32, 0.020),
        (0.80, 0.55, 0.018),
    ];
    let dark_green = (45, 85, 35);
    let mid_green = (75, 130, 55);
    let bright_green = (110, 170, 70);
    let dry = (150, 155, 70);
    for (bx, top, half_w) in blades {
        let bend = (v - *top) * 0.10 * ((*bx - 0.5) * 2.0);
        let du = u - (*bx + bend);
        if du.abs() < *half_w && v > *top {
            let along = (v - *top) / (1.0 - *top).max(0.001);
            if *bx > 0.55 && along < 0.4 { return Some(bright_green); }
            if along > 0.85 { return Some(mid_green); }
            if (bx * 10.0) as i32 % 3 == 0 { return Some(dry); }
            if along < 0.3 { return Some(bright_green); }
            return Some(mid_green);
        }
    }
    // Base tuft dark patch
    let du = u - 0.5;
    let dv = v - 0.90;
    if (du / 0.32).powi(2) + (dv / 0.08).powi(2) < 1.0 {
        return Some(dark_green);
    }
    None
}


fn bush_pixel(u: f64, v: f64) -> Option<(u8, u8, u8)> {
    if u < 0.0 || u > 1.0 || v < 0.0 || v > 1.0 { return None; }
    let green = (70, 130, 60);
    let green_dark = (45, 95, 40);
    let leaf_hi = (110, 170, 80);
    if v < 0.10 { return None; }
    let du = u - 0.5;
    let dv = v - 0.60;
    let r = (du * du * 1.3 + dv * dv * 0.9).sqrt();
    if r > 0.45 { return None; }
    let noise = ((u * 12.0).sin() * (v * 10.0).cos() + 1.0) * 0.5;
    if noise < 0.33 { return Some(green_dark); }
    if noise < 0.66 { return Some(green); }
    Some(leaf_hi)
}

fn bus_stop_pixel(u: f64, v: f64) -> Option<(u8, u8, u8)> {
    if u < 0.0 || u > 1.0 || v < 0.0 || v > 1.0 { return None; }
    let post = (120, 130, 140);
    let roof = (60, 70, 85);
    let glass = (170, 200, 220);
    let seat = (90, 60, 40);
    let sign_bg = (30, 80, 160);
    let sign_txt = (240, 220, 120);
    // Roof
    if v < 0.10 { return Some(roof); }
    // Support posts
    if v > 0.10 && v < 0.85
        && ((u < 0.06 && u > 0.02) || (u > 0.94 && u < 0.98)) { return Some(post); }
    // Back glass panel
    if v > 0.12 && v < 0.72 && u > 0.08 && u < 0.92 {
        // Glass frame
        if u < 0.10 || u > 0.90 || v < 0.14 || v > 0.68 { return Some(post); }
        // Sign atop right
        if u > 0.70 && u < 0.90 && v > 0.14 && v < 0.28
            && u > 0.72 && u < 0.88 && v > 0.16 && v < 0.26 {
                if ((u * 20.0) as i32) % 3 == 0 && v > 0.19 && v < 0.24 { return Some(sign_txt); }
                return Some(sign_bg);
            }
        return Some(glass);
    }
    // Bench
    if (0.72..0.85).contains(&v)
        && u > 0.14 && u < 0.86 { return Some(seat); }
    // Bench legs
    if (0.85..0.95).contains(&v)
        && ((u > 0.18 && u < 0.24) || (u > 0.50 && u < 0.56) || (u > 0.76 && u < 0.82)) {
            return Some(post);
        }
    None
}

fn mailbox_pixel(u: f64, v: f64) -> Option<(u8, u8, u8)> {
    if u < 0.0 || u > 1.0 || v < 0.0 || v > 1.0 { return None; }
    let red = (200, 40, 40);
    let dark_red = (130, 20, 20);
    let post = (60, 55, 50);
    let slot = (25, 25, 30);
    let logo = (240, 210, 60);
    // Post
    if v > 0.60 {
        if (u - 0.5).abs() < 0.06 { return Some(post); }
        return None;
    }
    // Box body
    if v < 0.60 {
        if u < 0.08 || u > 0.92 || v < 0.04 { return None; }
        // Slot
        if v > 0.14 && v < 0.20 && u > 0.20 && u < 0.80 { return Some(slot); }
        // Logo
        if v > 0.32 && v < 0.45 && u > 0.30 && u < 0.70 { return Some(logo); }
        // Shading
        let shade = if u < 0.5 { 1.0 } else { 0.85 };
        let (r, g, b) = red;
        let r = (r as f64 * shade) as u8;
        let g = (g as f64 * shade) as u8;
        let b = (b as f64 * shade) as u8;
        if v > 0.55 { return Some(dark_red); }
        return Some((r, g, b));
    }
    None
}

fn aluminum_table_pixel(u: f64, v: f64) -> Option<(u8, u8, u8)> {
    if u < 0.0 || u > 1.0 || v < 0.0 || v > 1.0 { return None; }
    let steel = (200, 208, 218);
    let dark = (110, 120, 135);
    let shadow = (55, 60, 75);
    // Wheels
    if v > 0.92 {
        if (u > 0.05 && u < 0.15) || (u > 0.85 && u < 0.95) { return Some(shadow); }
        return None;
    }
    // Legs
    if v > 0.55 {
        if (u > 0.10 && u < 0.16) || (u > 0.84 && u < 0.90) { return Some(dark); }
        return None;
    }
    // Table top (with tools scattered)
    if v < 0.12 {
        // Tools on top
        if v > 0.03 {
            if u > 0.20 && u < 0.35 { return Some((180, 40, 40)); }  // gauze red
            if u > 0.45 && u < 0.50 { return Some((150, 150, 160)); } // scalpel
            if u > 0.60 && u < 0.75 { return Some((240, 240, 245)); } // mask
        }
        return Some(steel);
    }
    // Body
    let shade = if u < 0.5 { 1.0 } else { 0.88 };
    Some(((steel.0 as f64 * shade) as u8, (steel.1 as f64 * shade) as u8, (steel.2 as f64 * shade) as u8))
}




fn burra_pixel(u: f64, v: f64) -> Option<(u8, u8, u8)> {
    if u < 0.0 || u > 1.0 || v < 0.0 || v > 1.0 { return None; }
    let rail = (185, 190, 200);
    let leg = (130, 135, 145);
    let hanger = (110, 115, 130);
    let green = (95, 165, 130);
    let white = (235, 240, 240);
    let dark = (60, 70, 85);
    // Vertical legs
    if v > 0.10
        && ((u > 0.05 && u < 0.09) || (u > 0.91 && u < 0.95)) {
            return Some(leg);
        }
    // Feet
    if v > 0.93 {
        if u > 0.02 && u < 0.13 { return Some(dark); }
        if u > 0.87 && u < 0.98 { return Some(dark); }
        return None;
    }
    // Top rail
    if v < 0.12 {
        if u > 0.05 && u < 0.95 { return Some(rail); }
        return None;
    }
    // Hanging gowns pattern (4 gowns)
    if v > 0.18 && v < 0.85 {
        let n = 4;
        let g_idx = (u * n as f64) as i32;
        let g_u = u * n as f64 - g_idx as f64;
        // Hanger
        if v < 0.22 && g_u > 0.42 && g_u < 0.58 { return Some(hanger); }
        // Gown body
        if g_u < 0.10 || g_u > 0.90 { return None; }
        let alt = g_idx % 2 == 0;
        let (br, bg, bb) = if alt { green } else { white };
        // Shadow on right side
        let shade = if u < 0.5 { 1.0 } else { 0.9 };
        // Collar detail
        if v < 0.26 && g_u > 0.30 && g_u < 0.70 {
            return Some(dark);
        }
        return Some((
            (br as f64 * shade) as u8,
            (bg as f64 * shade) as u8,
            (bb as f64 * shade) as u8,
        ));
    }
    None
}




fn toilet_pixel(u: f64, v: f64, paper: bool) -> Option<(u8, u8, u8)> {
    if u < 0.0 || u > 1.0 || v < 0.0 || v > 1.0 { return None; }
    let white = (238, 238, 242);
    let mid = (205, 205, 215);
    let dark = (155, 155, 170);
    let shadow = (95, 100, 115);
    let chrome = (185, 190, 200);
    let paper_col = (240, 232, 218);
    let paper_core = (170, 140, 100);
    let shade = |c: (u8, u8, u8), s: f64| (
        (c.0 as f64 * s).clamp(0.0, 255.0) as u8,
        (c.1 as f64 * s).clamp(0.0, 255.0) as u8,
        (c.2 as f64 * s).clamp(0.0, 255.0) as u8,
    );

    let du = u - 0.5;

    // Paper roll on top of cistern (only if paper flag)
    if paper {
        // Roll sits on tank top around v=0.00..0.06
        let roll_cy = 0.02;
        let roll_dv = v - roll_cy;
        let roll_du = u - 0.5;
        if roll_dv.abs() < 0.045 && roll_du.abs() < 0.13 {
            if roll_du.abs() < 0.03 { return Some(paper_core); }
            return Some(paper_col);
        }
    }

    // Tank (upper rectangle)
    if v < 0.34 {
        let tank = du.abs() < 0.24 && v > 0.05;
        if !tank { return None; }
        // Flush lever on top-right
        if v > 0.06 && v < 0.10 && u > 0.66 && u < 0.72 {
            return Some(chrome);
        }
        // Top edge shadow
        if v < 0.08 { return Some(shade(white, 0.90)); }
        // Bottom edge of tank
        if v > 0.30 { return Some(mid); }
        let s = if u < 0.5 { 1.0 } else { 0.90 };
        return Some(shade(white, s));
    }

    // Transition band (tank to bowl)
    if v < 0.42 {
        if du.abs() < 0.10 { return Some(mid); }
        return None;
    }

    // Seat + bowl area
    let dv = v - 0.62;
    let outer = (du / 0.40).powi(2) + (dv / 0.32).powi(2);
    if outer > 1.0 {
        // Pedestal base at bottom
        if v > 0.90 && du.abs() < 0.30 { return Some(shade(dark, 0.85)); }
        return None;
    }
    // Outer seat ring
    if outer > 0.78 {
        let s = if u < 0.5 { 1.0 } else { 0.92 };
        return Some(shade(white, s));
    }
    // Inner rim
    if outer > 0.62 { return Some(mid); }
    // Water inside bowl
    if outer > 0.45 { return Some(shadow); }
    Some((60, 80, 120))
}

fn sink_pixel(u: f64, v: f64) -> Option<(u8, u8, u8)> {
    if u < 0.0 || u > 1.0 || v < 0.0 || v > 1.0 { return None; }
    let white = (235, 235, 240);
    let shadow = (170, 170, 180);
    if v < 0.35 {
        let faucet = (u - 0.50).abs() < 0.05 && v > 0.05 && v < 0.30;
        let spout = (u - 0.50).abs() < 0.10 && v > 0.24 && v < 0.32;
        if faucet || spout { return Some((190, 190, 200)); }
        return None;
    }
    let du = u - 0.5;
    let dv = v - 0.65;
    let outer = (du / 0.48).powi(2) + (dv / 0.30).powi(2);
    if outer > 1.0 { return None; }
    let inner = (du / 0.38).powi(2) + (dv / 0.22).powi(2);
    if inner > 1.0 { return Some(shadow); }
    let shade = if u < 0.5 { 1.0 } else { 0.90 };
    Some(((white.0 as f64 * shade) as u8, (white.1 as f64 * shade) as u8, (white.2 as f64 * shade) as u8))
}

fn shower_pixel(u: f64, v: f64) -> Option<(u8, u8, u8)> {
    if u < 0.0 || u > 1.0 || v < 0.0 || v > 1.0 { return None; }
    let metal = (170, 180, 190);
    let tile = (200, 220, 230);
    let dark_tile = (100, 130, 150);
    if v > 0.92 {
        return Some((90, 95, 110));
    }
    if v < 0.10 {
        let head = (u - 0.50).abs() < 0.28 && v > 0.02;
        if head { return Some(metal); }
        return None;
    }
    if v < 0.14 && (u - 0.5).abs() < 0.05 { return Some(metal); }
    let pipe = (u - 0.5).abs() < 0.04 && v < 0.30;
    if pipe { return Some(metal); }
    let x_tile = ((u * 4.0) as i32 + (v * 8.0) as i32) % 2 == 0;
    let base = if x_tile { tile } else { dark_tile };
    let panel = (u - 0.5).abs() < 0.40;
    if !panel { return None; }
    let shade = if u < 0.5 { 1.0 } else { 0.90 };
    Some(((base.0 as f64 * shade) as u8, (base.1 as f64 * shade) as u8, (base.2 as f64 * shade) as u8))
}

fn urinal_pixel(u: f64, v: f64) -> Option<(u8, u8, u8)> {
    if u < 0.0 || u > 1.0 || v < 0.0 || v > 1.0 { return None; }
    let white = (235, 235, 240);
    let shadow = (170, 170, 180);
    if v < 0.10 {
        let flush = (u - 0.50).abs() < 0.08 && v > 0.02;
        if flush { return Some((190, 190, 200)); }
        return None;
    }
    let du = u - 0.5;
    let outer_h = if v < 0.50 { 0.35 } else { 0.42 };
    if du.abs() > outer_h { return None; }
    if v > 0.88 { return Some((80, 85, 100)); }
    let inner_h = outer_h - 0.08;
    if du.abs() > inner_h { return Some(shadow); }
    let shade = if u < 0.5 { 1.0 } else { 0.90 };
    Some(((white.0 as f64 * shade) as u8, (white.1 as f64 * shade) as u8, (white.2 as f64 * shade) as u8))
}



fn door_leaf_pixel(u: f64, v: f64, color: (u8, u8, u8)) -> Option<(u8, u8, u8)> {
    if u < 0.0 || u > 1.0 || v < 0.0 || v > 1.0 { return None; }
    let (r, g, b) = color;
    let shade = if u < 0.5 { 1.0 } else { 0.78 };
    Some((
        ((r as f64 * shade).clamp(0.0, 255.0)) as u8,
        ((g as f64 * shade).clamp(0.0, 255.0)) as u8,
        ((b as f64 * shade).clamp(0.0, 255.0)) as u8,
    ))
}




fn backpack_pixel(u: f64, v: f64) -> Option<(u8, u8, u8)> {
    let handle_top = 0.02;
    let body_top = 0.14;
    let body_bot = 0.98;
    let body_left = 0.12;
    let body_right = 0.88;

    if v < handle_top { return None; }

    if v < body_top {
        let hu = u - 0.5;
        let hv = v - 0.09;
        let outer = (hu / 0.10).powi(2) + (hv / 0.06).powi(2);
        let inner = (hu / 0.06).powi(2) + (hv / 0.03).powi(2);
        if outer <= 1.0 && inner > 1.0 {
            return Some((70, 48, 22));
        }
        return None;
    }

    if v > body_bot { return None; }
    if u < body_left || u > body_right { return None; }

    let corner = 0.07;
    let top_left = (u - (body_left + corner)).powi(2) + (v - (body_top + corner)).powi(2);
    if u < body_left + corner && v < body_top + corner && top_left > corner * corner { return None; }
    let top_right = (u - (body_right - corner)).powi(2) + (v - (body_top + corner)).powi(2);
    if u > body_right - corner && v < body_top + corner && top_right > corner * corner { return None; }
    let bot_left = (u - (body_left + corner)).powi(2) + (v - (body_bot - corner)).powi(2);
    if u < body_left + corner && v > body_bot - corner && bot_left > corner * corner { return None; }
    let bot_right = (u - (body_right - corner)).powi(2) + (v - (body_bot - corner)).powi(2);
    if u > body_right - corner && v > body_bot - corner && bot_right > corner * corner { return None; }

    let base = (100.0, 105.0, 118.0);
    let side_shade = if u < 0.5 { 1.14 } else { 0.82 };
    let vert_shade = 1.0 + (0.5 - v) * 0.14;
    let shade = side_shade * vert_shade;
    let r = (base.0 * shade).clamp(30.0, 255.0) as u8;
    let g = (base.1 * shade).clamp(30.0, 255.0) as u8;
    let b = (base.2 * shade).clamp(35.0, 255.0) as u8;

    let strap_l = (u - 0.22).abs() < 0.035;
    let strap_r = (u - 0.78).abs() < 0.035;
    if (strap_l || strap_r) && v > body_top && v < 0.78 {
        return Some((45, 48, 55));
    }

    let buckle_v = (v - 0.55).abs() < 0.022;
    if buckle_v && ((u - 0.22).abs() < 0.045 || (u - 0.78).abs() < 0.045) {
        return Some((215, 220, 230));
    }

    let top_seam = (v - 0.22).abs() < 0.012 && u > body_left + 0.05 && u < body_right - 0.05;
    if top_seam { return Some((50, 55, 65)); }

    let pocket_l = 0.28;
    let pocket_r = 0.72;
    let pocket_t = 0.44;
    let pocket_b = 0.88;
    if u >= pocket_l && u <= pocket_r && v >= pocket_t && v <= pocket_b {
        let pocket_edge = (u - pocket_l).abs() < 0.014
            || (u - pocket_r).abs() < 0.014
            || (v - pocket_t).abs() < 0.014
            || (v - pocket_b).abs() < 0.014;
        if pocket_edge { return Some((50, 55, 65)); }
        if (v - 0.60).abs() < 0.014 && u > pocket_l + 0.03 && u < pocket_r - 0.03 {
            return Some((190, 195, 205));
        }
        if (u - 0.5).abs() < 0.03 && (v - 0.65).abs() < 0.03 {
            return Some((60, 200, 140));
        }
        let pocket_r_c = (r as f64 * 1.10).clamp(0.0, 255.0) as u8;
        let pocket_g_c = (g as f64 * 1.10).clamp(0.0, 255.0) as u8;
        let pocket_b_c = (b as f64 * 1.10).clamp(0.0, 255.0) as u8;
        return Some((pocket_r_c, pocket_g_c, pocket_b_c));
    }

    Some((r, g, b))
}



fn coat_rack_pixel(u: f64, v: f64) -> Option<(u8, u8, u8)> {
    let dist = (u - 0.5).abs();

    if v > 0.93 {
        if v > 0.97 && dist < 0.42 {
            return Some((60, 45, 28));
        }
        if v > 0.93 && dist < 0.38 {
            let shade = if u < 0.5 { 1.0 } else { 0.78 };
            return Some(((95.0 * shade) as u8, (72.0 * shade) as u8, (48.0 * shade) as u8));
        }
    }

    if dist < 0.05 && v > 0.08 && v <= 0.93 {
        let shade = if u < 0.5 { 1.0 } else { 0.72 };
        return Some(((120.0 * shade) as u8, (95.0 * shade) as u8, (62.0 * shade) as u8));
    }

    let hook_y = 0.12;
    if v > hook_y - 0.04 && v < hook_y + 0.06
        && ((u - 0.20).abs() < 0.05 || (u - 0.80).abs() < 0.05) {
            return Some((160, 130, 80));
        }
    let hook_y2 = 0.20;
    if v > hook_y2 - 0.03 && v < hook_y2 + 0.04
        && ((u - 0.30).abs() < 0.03 || (u - 0.70).abs() < 0.03) {
            return Some((160, 130, 80));
        }

    None
}

fn hanging_pixel(u: f64, v: f64, color: (u8, u8, u8)) -> Option<(u8, u8, u8)> {
    let center = (u - 0.5).abs();

    if v < 0.06 {
        if center < 0.03 { return Some((90, 70, 45)); }
        return None;
    }

    let shoulder_end = 0.22;
    let width_at_v = if v < shoulder_end {
        0.06 + (v / shoulder_end) * 0.34
    } else {
        0.40 - (v - shoulder_end) / (1.0 - shoulder_end) * 0.06
    };
    if v > 0.98 { return None; }
    if center > width_at_v { return None; }

    if v > 0.16 && v < 0.24 {
        let neck_w = 0.10;
        if center < neck_w { return None; }
    }

    let stripe = ((v * 22.0).sin() * 0.04 + 0.94).clamp(0.82, 1.0);
    let side = if u < 0.5 { 1.0 } else { 0.88 };
    let (r, g, b) = color;
    Some((
        (r as f64 * stripe * side).clamp(0.0, 255.0) as u8,
        (g as f64 * stripe * side).clamp(0.0, 255.0) as u8,
        (b as f64 * stripe * side).clamp(0.0, 255.0) as u8,
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hair {
    /// Short and unbrushed, sticking up.
    Messy,
    /// Long, pulled back into a bun.
    Bun,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Top {
    Hoodie,
    /// A long open cardigan over a blouse.
    Cardigan,
}

/// What a person looks like.
#[derive(Debug, Clone, Copy)]
pub struct PersonLook {
    /// Metres.
    pub height: f64,
    /// Shoulder width relative to average.
    pub build: f64,
    pub skin: Rgb,
    pub hair: Rgb,
    pub hair_style: Hair,
    pub top: Rgb,
    pub top_style: Top,
    /// What shows underneath: drawstrings, a blouse.
    pub under: Rgb,
    pub legs: Rgb,
    pub shoes: Rgb,
    pub glasses: bool,
    pub stubble: bool,
    /// Lines round the eyes and mouth.
    pub lined: bool,
    pub bag: Option<Rgb>,
    /// The white hospital wristband.
    pub wristband: bool,
}

/// A standing figure, drawn on a 32 × 96 grid (about 55 pixels a metre, so a face is ten
/// pixels wide) and lit from the left. The silhouette gets a darker outline, as pixel art does.
fn person_pixel(u: f64, v: f64, look: &PersonLook) -> Option<Rgb> {
    const W: f64 = 32.0;
    const H: f64 = 96.0;
    let (x, y) = ((u * W).floor() as i32, (v * H).floor() as i32);
    let c = figure(x, y, look)?;
    let side = if x <= 12 { 1.07 } else if x >= 20 { 0.84 } else { 1.0 };
    let edge = figure(x - 1, y, look).is_none() || figure(x + 1, y, look).is_none() || figure(x, y - 1, look).is_none();
    Some(mul(c, if edge { side * 0.7 } else { side }))
}

fn figure(x: i32, y: i32, l: &PersonLook) -> Option<Rgb> {
    if !(0..32).contains(&x) || !(0..96).contains(&y) {
        return None;
    }
    if y <= 14 {
        return head(x, y, l);
    }
    if y <= 16 {
        // neck, and the hood bunched round it
        return match l.top_style {
            Top::Hoodie if !(14..=17).contains(&x) && (10..=21).contains(&x) && y == 16 => Some(darken(l.top, 0.8)),
            _ => (14..=17).contains(&x).then_some(if x == 17 { darken(l.skin, 0.84) } else { darken(l.skin, 0.92) }),
        };
    }
    // shoulders slope in a little; a wider build pushes them out
    let half = 9.0 + (l.build - 1.0) * 10.0;
    let (t0, t1) = ((15.5 - half).round() as i32, (15.5 + half).round() as i32);
    let rough = |a: i32, b: i32| ((a * 7 + b * 13) ^ (a * b)) & 3 == 0;

    // the tote bag, hanging from the right shoulder over the arm
    if let Some(bag) = l.bag {
        if (42..=57).contains(&y) && (t1 - 1..=t1 + 4).contains(&x) {
            return Some(if y == 42 || x == t1 + 4 || y == 57 { darken(bag, 0.75) } else if rough(x, y) { darken(bag, 0.93) } else { bag });
        }
        if (17..=42).contains(&y) && x == t1 - 1 - (y - 17) / 12 {
            return Some(darken(bag, 0.62)); // strap
        }
    }

    // arms, sleeves down to the wrists, hands below
    let left_arm = (t0 - 3..t0).contains(&x);
    let right_arm = (t1 + 1..=t1 + 3).contains(&x);
    if (left_arm || right_arm) && (18..=49).contains(&y) {
        if y >= 44 {
            if right_arm && l.wristband && y == 44 {
                return Some((242, 242, 238));
            }
            let finger = y >= 47 && (x - t0).rem_euclid(2) == 0;
            return Some(if finger { darken(l.skin, 0.86) } else { l.skin });
        }
        let cuff = l.top_style == Top::Hoodie && y >= 41;
        let fold = (y == 30 || y == 31) && (x == t0 - 2 || x == t1 + 2);
        return Some(if cuff || fold { darken(l.top, 0.82) } else { l.top });
    }

    // the torso: a hoodie to the hips, or a long cardigan over a blouse
    let hem = if l.top_style == Top::Cardigan { 56 } else { 46 };
    if (17..=hem).contains(&y) && (t0..=t1).contains(&x) {
        let dx = x as f64 - 15.5;
        match l.top_style {
            Top::Hoodie => {
                if y >= 44 {
                    return Some(darken(l.top, if x % 2 == 0 { 0.78 } else { 0.84 })); // ribbed hem
                }
                if (17..=25).contains(&y) && (x == 13 || x == 18) {
                    return Some(if y == 25 { darken(l.under, 0.8) } else { l.under }); // drawstrings
                }
                if (34..=41).contains(&y) && (10..=21).contains(&x) {
                    let rim = y == 34 || x == 10 || x == 21 || (y == 35 && (x == 11 || x == 20));
                    return Some(if rim { darken(l.top, 0.76) } else { l.top }); // the front pocket
                }
                if y == 17 && dx.abs() < 4.0 {
                    return Some(darken(l.top, 0.8));
                }
            }
            Top::Cardigan => {
                let open = (2.6 - (y as f64 - 17.0) * 0.04).max(1.2);
                if dx.abs() <= open && y <= 46 {
                    // the blouse, with a little collar at the top
                    let collar = y <= 19 && dx.abs() > open - 1.0;
                    return Some(if collar { (240, 236, 222) } else if y % 6 == 0 && dx.abs() < 0.6 { darken(l.under, 0.85) } else { l.under });
                }
                if dx.abs() <= open + 1.0 && y % 5 == 2 && y <= 44 {
                    return Some((214, 198, 160)); // buttons
                }
                if y > 46 && dx.abs() <= 1.0 {
                    return None; // open below the waist
                }
                if (46..=52).contains(&y) && (dx.abs() - 5.0).abs() <= 2.0 {
                    return Some(if y == 46 { darken(l.top, 0.75) } else { darken(l.top, 0.9) }); // pockets
                }
                if rough(x, y) && y > 20 {
                    return Some(darken(l.top, 0.93)); // knit
                }
            }
        }
        return Some(l.top);
    }

    // legs, a gap between them from the crotch down
    if (hem.min(47)..=89).contains(&y) {
        let (l0, l1) = (10, 21);
        if !(l0..=l1).contains(&x) || (y >= 56 && (15..=16).contains(&x)) {
            return None;
        }
        let knee = (69..=70).contains(&y) && (x == 12 || x == 19);
        let crease = (x == 12 || x == 19) && y > 58;
        let seam = (x == l0 || x == l1) && y > 50;
        return Some(if knee || crease { darken(l.legs, 0.86) } else if seam { darken(l.legs, 0.92) } else { l.legs });
    }

    // shoes
    if (90..=95).contains(&y) {
        let on = (9..=14).contains(&x) || (17..=22).contains(&x);
        return on.then(|| if y >= 94 { darken(l.shoes, 0.5) } else if y == 90 { darken(l.shoes, 0.88) } else { l.shoes });
    }
    None
}

/// The head, rows 0–14: hair, a ten-pixel face, ears.
fn head(x: i32, y: i32, l: &PersonLook) -> Option<Rgb> {
    let skin = l.skin;
    let hair = l.hair;
    let rough = |a: i32, b: i32| ((a * 7 + b * 13) ^ (a * b)) & 3 == 0;
    let frame = (36, 36, 42);

    // hair first: it sits over the top of the head
    match l.hair_style {
        Hair::Messy => {
            if y <= 1 && (12..=19).contains(&x) && (x * 3 + y) % 4 != 0 {
                return Some(darken(hair, if y == 0 { 1.25 } else { 1.0 })); // tufts
            }
            if (2..=4).contains(&y) && (10..=21).contains(&x) {
                return Some(if rough(x, y) { darken(hair, 1.3) } else { hair });
            }
            if y == 5 && (11..=16).contains(&x) && x % 2 == 1 {
                return Some(hair); // fringe falling forward
            }
            if (5..=8).contains(&y) && (x == 10 || x == 21) {
                return Some(darken(hair, 0.9));
            }
        }
        Hair::Bun => {
            if y <= 2 && (13..=18).contains(&x) && !(y == 0 && (x == 13 || x == 18)) {
                return Some(if (x + y) % 3 == 0 { darken(hair, 1.12) } else { hair }); // the bun
            }
            if (3..=5).contains(&y) && (10..=21).contains(&x) {
                return Some(if (x + 2 * y) % 4 == 0 { darken(hair, 1.12) } else if (x + y) % 5 == 0 { darken(hair, 0.86) } else { hair });
            }
            if (6..=10).contains(&y) && (x == 10 || x == 21) {
                return Some(hair);
            }
        }
    }
    // ears
    if (x == 10 || x == 21) && (7..=10).contains(&y) {
        return Some(darken(skin, 0.82));
    }
    // the face narrows to the chin
    let (f0, f1) = match y {
        13 => (12, 19),
        14 => (13, 18),
        _ => (11, 20),
    };
    if y < 4 || !(f0..=f1).contains(&x) {
        return None;
    }
    // glasses: two framed lenses and a bridge
    if l.glasses && (6..=9).contains(&y) {
        let left = (12..=15).contains(&x);
        let right = (16..=19).contains(&x);
        if left || right {
            let rim = y == 6 || y == 9 || x == 12 || x == 15 || x == 16 || x == 19;
            if rim && !(y == 9 && (x == 15 || x == 16)) {
                return Some(frame);
            }
        }
    }
    if y == 6 && ((12..=14).contains(&x) || (17..=19).contains(&x)) && !l.glasses {
        return Some(darken(hair, if l.hair_style == Hair::Bun { 0.7 } else { 1.0 })); // brows
    }
    if y == 7 {
        match x {
            13 | 18 => return Some((236, 232, 226)), // whites
            14 | 17 => return Some((46, 34, 28)),    // irises
            _ => {}
        }
    }
    if l.lined && ((y == 8 && (x == 12 || x == 19)) || ((10..=12).contains(&y) && (x == 13 || x == 18)) || (y == 5 && (13..=18).contains(&x) && x % 2 == 0)) {
        return Some(darken(skin, 0.8)); // crow's feet, laugh lines, a line across the brow
    }
    if (8..=10).contains(&y) && x == 16 {
        return Some(darken(skin, 0.82)); // shadow of the nose
    }
    if y == 11 && (15..=16).contains(&x) {
        return Some(darken(skin, 0.78)); // nostrils
    }
    if y == 12 && (14..=17).contains(&x) {
        return Some(if x == 14 || x == 17 { darken(skin, 0.78) } else { (156, 96, 88) }); // mouth
    }
    if l.stubble && y >= 11 && (rough(x, y) || y == 14 || (y == 11 && (14..=17).contains(&x))) {
        return Some(darken(skin, 0.72));
    }
    if l.glasses && (7..=8).contains(&y) && ((12..=15).contains(&x) || (16..=19).contains(&x)) {
        return Some(mix_rgb(skin, (200, 220, 236), 0.25)); // the lens over the cheek
    }
    Some(if x >= 19 { darken(skin, 0.88) } else if x <= 12 { darken(skin, 1.04) } else { skin })
}

fn mix_rgb(a: Rgb, b: Rgb, t: f64) -> Rgb {
    let f = |p: u8, q: u8| (p as f64 + (q as f64 - p as f64) * t) as u8;
    (f(a.0, b.0), f(a.1, b.1), f(a.2, b.2))
}

fn darken(c: (u8, u8, u8), f: f64) -> (u8, u8, u8) {
    ((c.0 as f64 * f) as u8, (c.1 as f64 * f) as u8, (c.2 as f64 * f) as u8)
}

fn mul(c: (u8, u8, u8), f: f64) -> (u8, u8, u8) {
    ((c.0 as f64 * f) as u8, (c.1 as f64 * f) as u8, (c.2 as f64 * f) as u8)
}

fn clothing_pixel(u: f64, v: f64, base: (u8, u8, u8)) -> Option<(u8, u8, u8)> {
    if v < 0.30 { return None; }

    let cx = 0.5;
    let cy = 0.70;
    let du = u - cx;
    let dv = (v - cy) * 1.6;

    let ripple = (u * 18.0).sin() * 0.04 + (v * 22.0).sin() * 0.03;
    let r2 = du * du + dv * dv;
    let base_r = 0.34 + ripple;
    if r2 > base_r * base_r { return None; }

    let (br, bg, bb) = base;
    let d = r2.sqrt();
    let shade = (1.0 - d / base_r * 0.35).clamp(0.55, 1.0);
    let fold = ((u * 12.0 + v * 8.0).sin() * 0.08 + 1.0).clamp(0.85, 1.1);
    let r = (br as f64 * shade * fold).clamp(0.0, 255.0) as u8;
    let g = (bg as f64 * shade * fold).clamp(0.0, 255.0) as u8;
    let b = (bb as f64 * shade * fold).clamp(0.0, 255.0) as u8;
    Some((r, g, b))
}

fn iv_stand_pixel(u: f64, v: f64) -> Option<Rgb> {
    let steel = (176, 180, 186);
    if v > 0.95 {
        // five-wheeled base
        return ((u - 0.5).abs() < 0.42 && v < 0.98).then_some((70, 72, 76)).or(((u * 5.0).fract() < 0.3 && (u - 0.5).abs() < 0.45).then_some((30, 30, 32)));
    }
    if (0.04..0.06).contains(&v) && (0.25..0.75).contains(&u) {
        return Some(steel); // hook bar
    }
    if (0.3..0.46).contains(&u) && (0.07..0.27).contains(&v) {
        return Some(if (0.3..0.32).contains(&u) || v < 0.09 { (200, 206, 210) } else { (236, 232, 200) }); // the bag
    }
    if (0.37..0.39).contains(&u) && (0.27..0.6).contains(&v) {
        return Some((220, 224, 226)); // drip line
    }
    if (0.52..0.78).contains(&u) && (0.42..0.54).contains(&v) {
        return Some(if (0.56..0.7).contains(&u) && (0.45..0.49).contains(&v) { (60, 200, 120) } else { (196, 200, 204) }); // pump
    }
    ((0.48..0.52).contains(&u) && v > 0.05).then_some(steel)
}

fn monitor_pixel(u: f64, v: f64) -> Option<Rgb> {
    if v < 0.3 {
        if !(0.06..=0.94).contains(&u) {
            return None;
        }
        if !(0.12..=0.88).contains(&u) || !(0.04..=0.26).contains(&v) {
            return Some((70, 74, 80)); // casing
        }
        // waveforms on black: ECG in green, SpO2 in cyan, blood pressure in red
        let x = (u - 0.12) / 0.76;
        let beat = (x * 3.0).fract();
        let ecg = 0.09 - if (0.4..0.45).contains(&beat) { 0.05 } else if (0.45..0.5).contains(&beat) { -0.03 } else { 0.0 };
        if (v - ecg).abs() < 0.012 {
            return Some((60, 230, 90));
        }
        if (v - (0.16 - 0.02 * (x * 18.0).sin())).abs() < 0.01 {
            return Some((70, 210, 230));
        }
        if (v - (0.22 - 0.015 * (x * 12.0).cos())).abs() < 0.01 {
            return Some((230, 70, 70));
        }
        return Some((8, 12, 10));
    }
    if v > 0.95 {
        return ((u * 4.0).fract() < 0.35).then_some((30, 30, 32));
    }
    if v > 0.92 {
        return ((u - 0.5).abs() < 0.4).then_some((80, 82, 86));
    }
    ((0.46..0.54).contains(&u)).then_some((150, 154, 160))
}

fn wheelchair_pixel(u: f64, v: f64) -> Option<Rgb> {
    let frame = (170, 174, 180);
    let d = ((u - 0.42).powi(2) + ((v - 0.62) * 0.79).powi(2)).sqrt();
    if (0.24..0.29).contains(&d) {
        return Some((40, 40, 44)); // the big wheel's tyre
    }
    if (0.2..0.24).contains(&d) || (d < 0.2 && ((u - 0.42).abs() < 0.012 || (v - 0.62).abs() < 0.012)) {
        return Some(frame); // rim and spokes
    }
    if ((u - 0.84).powi(2) + (v - 0.92).powi(2)).sqrt() < 0.06 {
        return Some((40, 40, 44)); // caster
    }
    if (0.46..0.54).contains(&v) && (0.25..0.85).contains(&u) {
        return Some((30, 32, 36)); // seat
    }
    if (0.24..0.32).contains(&u) && (0.12..0.5).contains(&v) {
        return Some((30, 32, 36)); // backrest
    }
    if (0.12..0.32).contains(&u) && (0.08..0.12).contains(&v) {
        return Some((40, 40, 44)); // push handle
    }
    if (0.78..0.82).contains(&u) && (0.54..0.88).contains(&v) {
        return Some(frame); // footrest hanger
    }
    None
}

fn chair_pixel(u: f64, v: f64, c: Rgb) -> Option<Rgb> {
    if (0.08..0.92).contains(&u) && (0.02..0.42).contains(&v) {
        return Some(if v < 0.06 { darken(c, 1.15) } else { c }); // backrest shell
    }
    if (0.05..0.95).contains(&u) && (0.48..0.56).contains(&v) {
        return Some(darken(c, 0.8)); // seat
    }
    if v > 0.56 && ((0.1..0.15).contains(&u) || (0.85..0.9).contains(&u)) {
        return Some((60, 62, 66));
    }
    ((0.46..0.54).contains(&u) && (0.42..0.48).contains(&v)).then_some((60, 62, 66))
}

fn bin_pixel(u: f64, v: f64) -> Option<Rgb> {
    if !(0.08..=0.92).contains(&u) {
        return None;
    }
    if v < 0.1 {
        return Some(if v < 0.04 { (60, 62, 66) } else { (150, 154, 160) }); // lid
    }
    if v > 0.94 {
        return (0.3..0.7).contains(&u).then_some((40, 40, 42)); // pedal
    }
    let shade = if u < 0.3 { 1.15 } else if u > 0.75 { 0.75 } else { 1.0 };
    Some(darken((140, 144, 150), shade))
}

fn lamp_pixel(u: f64, v: f64) -> Option<Rgb> {
    let pole = (60, 64, 70);
    if (0.5..0.96).contains(&u) && (0.04..0.09).contains(&v) {
        return Some(pole); // housing
    }
    if (0.55..0.95).contains(&u) && (0.09..0.12).contains(&v) {
        return Some((255, 196, 110)); // sodium lens
    }
    if (0.2..0.55).contains(&u) && (0.035..0.05).contains(&v) {
        return Some(pole); // arm
    }
    if v > 0.97 {
        return ((u - 0.22).abs() < 0.08).then_some((50, 52, 56));
    }
    ((0.19..0.25).contains(&u) && v > 0.035).then_some(pole)
}

fn plant_pixel(u: f64, v: f64) -> Option<Rgb> {
    if v > 0.72 {
        let half = 0.22 - (v - 0.72) * 0.25;
        return ((u - 0.5).abs() < half).then_some(if v < 0.76 { (226, 224, 216) } else { (204, 200, 190) });
    }
    // leaves fanning out from the pot
    let dx = u - 0.5;
    let spread = 0.15 + (0.72 - v) * 0.5;
    if dx.abs() > spread {
        return None;
    }
    let n = ((u * 23.0).sin() * (v * 17.0).cos() + (u * 7.0 + v * 11.0).sin()) * 0.5;
    if n < -0.35 {
        return None;
    }
    Some(if n > 0.3 { (90, 150, 70) } else if n > -0.05 { (60, 120, 54) } else { (40, 86, 40) })
}

fn curtain_pixel(u: f64, v: f64) -> Option<Rgb> {
    if v < 0.02 {
        return Some((170, 174, 180)); // the ceiling track
    }
    if v < 0.06 {
        return ((u * 20.0).fract() < 0.3).then_some((200, 200, 204)); // hooks
    }
    // pleats, and a mesh band at the top for air
    let fold = (u * 40.0).sin() * 0.5 + 0.5;
    let base = if v < 0.16 { (200, 214, 216) } else if ((u * 12.0).floor() as i32 + (v * 16.0).floor() as i32) % 2 == 0 { (150, 190, 200) } else { (136, 176, 190) };
    Some(darken(base, 0.82 + 0.18 * fold))
}

fn surgical_light_pixel(u: f64, v: f64) -> Option<Rgb> {
    if v < 0.45 {
        return ((0.46..0.54).contains(&u)).then_some((190, 194, 198)); // arm down from the ceiling
    }
    let dx = (u - 0.5) / 0.48;
    let dy = (v - 0.62) / 0.16;
    if dx * dx + dy * dy > 1.0 {
        return None;
    }
    Some(if v > 0.62 { (250, 250, 240) } else { (210, 214, 218) })
}
