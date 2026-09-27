#[derive(Debug, Clone, Copy)]
pub enum SignShapeKind {
    RoomSmall,
    MainSquare,
    HallwayLong,
    Bathroom,
    HospitalPlaque,
}

#[derive(Debug, Clone, Copy)]
pub enum DoorLeafAxis {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone)]
pub enum SpriteShape {
    Backpack,
    ClothingPile { color: (u8, u8, u8) },
    HangingClothing { color: (u8, u8, u8) },
    HospitalBed,
    CoatRack,
    Person { skin: (u8, u8, u8), shirt: (u8, u8, u8), pants: (u8, u8, u8) },
    Bench,
    Sign(SignShapeKind),
    DoorLeaf { color: (u8, u8, u8), axis: DoorLeafAxis },
    Sofa,
    Outlet,
    Toilet,
    ToiletWithPaper,
    Sink,
    Shower,
    Urinal,
    VendingSnacks,
    VendingDrinks,
    ReceptionCounter,
    WindowClear,
    WindowCloudy,
    OperatingTable,
    Burra,
    AluminumTable,
    SurgicalCabinet,
    WoodenTable,
    TreeOak,
    TreePine,
    Bush,
    Weed,
    Car { body: (u8, u8, u8) },
    BusStop,
    Mailbox,
}

pub struct SpriteInput {
    pub x: f64,
    pub y: f64,
    pub shape: SpriteShape,
    pub scale: f64,
    pub elevation: f64,
}

impl SpriteShape {
    pub fn sample(&self, u: f64, v: f64) -> Option<(u8, u8, u8)> {
        match self {
            SpriteShape::Backpack => backpack_pixel(u, v),
            SpriteShape::ClothingPile { color } => clothing_pixel(u, v, *color),
            SpriteShape::HangingClothing { color } => hanging_pixel(u, v, *color),
            SpriteShape::HospitalBed => bed_pixel(u, v),
            SpriteShape::CoatRack => coat_rack_pixel(u, v),
            SpriteShape::Person { skin, shirt, pants } => person_pixel(u, v, *skin, *shirt, *pants),
            SpriteShape::Bench => bench_pixel(u, v),
            SpriteShape::Sign(k) => sign_pixel(u, v, *k),
            SpriteShape::DoorLeaf { color, .. } => door_leaf_pixel(u, v, *color),
            SpriteShape::Sofa => sofa_pixel(u, v),
            SpriteShape::Outlet => outlet_pixel(u, v),
            SpriteShape::Toilet => toilet_pixel(u, v, false),
            SpriteShape::ToiletWithPaper => toilet_pixel(u, v, true),
            SpriteShape::Sink => sink_pixel(u, v),
            SpriteShape::Shower => shower_pixel(u, v),
            SpriteShape::Urinal => urinal_pixel(u, v),
            SpriteShape::VendingSnacks => vending_pixel(u, v, false),
            SpriteShape::VendingDrinks => vending_pixel(u, v, true),
            SpriteShape::ReceptionCounter => reception_pixel(u, v),
            SpriteShape::WindowClear => window_pixel(u, v, false),
            SpriteShape::WindowCloudy => window_pixel(u, v, true),
            SpriteShape::OperatingTable => operating_table_pixel(u, v),
            SpriteShape::Burra => burra_pixel(u, v),
            SpriteShape::AluminumTable => aluminum_table_pixel(u, v),
            SpriteShape::SurgicalCabinet => surgical_cabinet_pixel(u, v),
            SpriteShape::WoodenTable => wooden_table_pixel(u, v),
            SpriteShape::TreeOak => tree_oak_pixel(u, v),
            SpriteShape::TreePine => tree_pine_pixel(u, v),
            SpriteShape::Bush => bush_pixel(u, v),
            SpriteShape::Weed => weed_pixel(u, v),
            SpriteShape::Car { body } => car_pixel(u, v, *body),
            SpriteShape::BusStop => bus_stop_pixel(u, v),
            SpriteShape::Mailbox => mailbox_pixel(u, v),
        }
    }

    pub fn aspect(&self) -> (f64, f64) {
        match self {
            SpriteShape::Backpack => (0.55, 0.62),
            SpriteShape::ClothingPile { .. } => (0.90, 0.28),
            SpriteShape::HangingClothing { .. } => (0.42, 0.62),
            SpriteShape::HospitalBed => (1.80, 0.55),
            SpriteShape::CoatRack => (0.28, 0.95),
            SpriteShape::Person { .. } => (0.42, 1.05),
            SpriteShape::Bench => (1.30, 0.55),
            SpriteShape::Sign(k) => match k {
                SignShapeKind::RoomSmall => (0.16, 0.16),
                SignShapeKind::MainSquare => (0.24, 0.24),
                SignShapeKind::HallwayLong => (0.60, 0.16),
                SignShapeKind::Bathroom => (0.30, 0.30),
                SignShapeKind::HospitalPlaque => (1.30, 0.55),
            },
            SpriteShape::DoorLeaf { axis, .. } => match axis {
                DoorLeafAxis::Vertical => (0.12, 1.05),
                DoorLeafAxis::Horizontal => (1.05, 0.12),
            },
            SpriteShape::Sofa => (1.55, 0.75),
            SpriteShape::Outlet => (0.10, 0.10),
            SpriteShape::Toilet => (0.60, 0.75),
            SpriteShape::ToiletWithPaper => (0.60, 0.75),
            SpriteShape::Sink => (0.55, 0.55),
            SpriteShape::Shower => (0.60, 1.35),
            SpriteShape::Urinal => (0.45, 0.60),
            SpriteShape::VendingSnacks => (0.85, 0.90),
            SpriteShape::VendingDrinks => (0.85, 0.90),
            SpriteShape::ReceptionCounter => (3.20, 0.48),
            SpriteShape::WindowClear => (1.40, 1.05),
            SpriteShape::WindowCloudy => (1.40, 1.05),
            SpriteShape::OperatingTable => (2.10, 0.70),
            SpriteShape::Burra => (1.15, 0.92),
            SpriteShape::AluminumTable => (0.55, 0.55),
            SpriteShape::SurgicalCabinet => (0.90, 0.85),
            SpriteShape::WoodenTable => (0.85, 0.45),
            SpriteShape::TreeOak => (1.30, 0.98),
            SpriteShape::TreePine => (0.95, 0.98),
            SpriteShape::Bush => (0.85, 0.55),
            SpriteShape::Weed => (0.30, 0.28),
            SpriteShape::Car { .. } => (1.55, 0.72),
            SpriteShape::BusStop => (1.50, 0.98),
            SpriteShape::Mailbox => (0.42, 0.85),
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
        let noise = ((u * 8.0).sin() * (v * 6.0).cos() * 0.5 + 0.5);
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

fn car_pixel(u: f64, v: f64, body: (u8, u8, u8)) -> Option<(u8, u8, u8)> {
    if u < 0.0 || u > 1.0 || v < 0.0 || v > 1.0 { return None; }
    let dark = (
        (body.0 as f64 * 0.45) as u8,
        (body.1 as f64 * 0.45) as u8,
        (body.2 as f64 * 0.45) as u8,
    );
    let seam = (
        (body.0 as f64 * 0.30) as u8,
        (body.1 as f64 * 0.30) as u8,
        (body.2 as f64 * 0.30) as u8,
    );
    let glass = (95, 130, 165);
    let glass_hi = (150, 185, 210);
    let tire = (18, 18, 22);
    let hub = (170, 175, 185);
    let hub_bolt = (60, 60, 68);
    let headlight = (245, 235, 190);
    let taillight = (205, 40, 40);
    let plate = (235, 230, 200);
    let shadow = (14, 14, 18);

    // Ground shadow
    if v > 0.92 {
        let dx = u - 0.5;
        let s = (dx / 0.48).powi(2) + ((v - 0.955) / 0.04).powi(2);
        if s < 1.0 { return Some(shadow); }
        return None;
    }

    // Wheels — front and rear
    let wheel_y = 0.82;
    let wheel_r = 0.09;
    let wheel_inner_r = 0.045;
    for wx in [0.22_f64, 0.78] {
        let du = u - wx;
        let dv = (v - wheel_y) * 1.05;
        let d2 = du * du + dv * dv;
        if d2 < wheel_r * wheel_r {
            if d2 < 0.010 * 0.010 { return Some(hub_bolt); }
            if d2 < wheel_inner_r * wheel_inner_r {
                // Spokes
                let ang = dv.atan2(du);
                let spoke = ((ang * 5.0 / std::f64::consts::PI) * 3.0).sin();
                if spoke.abs() < 0.35 { return Some(hub_bolt); }
                return Some(hub);
            }
            return Some(tire);
        }
    }

    // Above roof line — sky/nothing
    if v < 0.18 { return None; }

    // Roof + greenhouse (windshield-rear window)
    if v < 0.42 {
        // Trapezoidal cabin narrower on top
        let inset = 0.28 - (v - 0.18) / (0.42 - 0.18) * 0.10;
        if u < 0.06 + inset || u > 0.94 - inset { return None; }
        // Roof top strip
        if v < 0.22 { return Some(dark); }
        // B-pillar between windows
        if (u - 0.50).abs() < 0.020 { return Some(dark); }
        // Window frame border
        let frame_top = v < 0.24;
        let frame_bot = v > 0.40;
        if frame_top || frame_bot { return Some(seam); }
        // Highlight reflection band
        if v < 0.30 { return Some(glass_hi); }
        return Some(glass);
    }

    // Belt line / body highlight
    if v < 0.45 {
        if u < 0.06 || u > 0.94 { return None; }
        return Some(seam);
    }

    // Body — main door panels
    if v < 0.82 {
        if u < 0.05 || u > 0.95 { return None; }

        // Wheel arch cutouts
        for wx in [0.22_f64, 0.78] {
            let du = u - wx;
            let dv = (v - 0.78) * 1.4;
            if du * du + dv * dv < 0.11 * 0.11 && v > 0.68 {
                return None;
            }
        }

        // Head/tail light region on rockers
        if v > 0.55 && v < 0.66 {
            if u > 0.03 && u < 0.10 { return Some(headlight); }
            if u > 0.90 && u < 0.97 { return Some(taillight); }
        }

        // Front door edge (vertical seam near cabin front)
        if (u - 0.30).abs() < 0.010 && v > 0.46 && v < 0.80 {
            return Some(seam);
        }
        // Rear door edge (vertical seam near cabin rear)
        if (u - 0.70).abs() < 0.010 && v > 0.46 && v < 0.80 {
            return Some(seam);
        }
        // Central B-pillar seam between doors
        if (u - 0.50).abs() < 0.014 && v > 0.46 && v < 0.80 {
            return Some(seam);
        }
        // Door handles (chrome recessed pulls)
        let handle = (170, 175, 185);
        if v > 0.54 && v < 0.575 {
            if (u - 0.395).abs() < 0.055 || (u - 0.605).abs() < 0.055 {
                return Some(handle);
            }
        }
        // Bottom edge outline (skirt) — clearer door footprint
        if v > 0.735 && v < 0.755 && u > 0.30 && u < 0.70 {
            return Some(seam);
        }
        // Bottom rocker panel (darker)
        if v > 0.76 {
            return Some(dark);
        }
        // License plate hint at rear
        if v > 0.68 && v < 0.74 && u > 0.85 && u < 0.94 {
            return Some(plate);
        }
        // Body shading
        let side_shade = if u < 0.5 { 1.05 } else { 0.86 };
        let vert = 1.0 - (v - 0.45) * 0.20;
        let s = (side_shade * vert).clamp(0.6, 1.15);
        return Some((
            (body.0 as f64 * s).clamp(0.0, 255.0) as u8,
            (body.1 as f64 * s).clamp(0.0, 255.0) as u8,
            (body.2 as f64 * s).clamp(0.0, 255.0) as u8,
        ));
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
    if v > 0.10 && v < 0.85 {
        if (u < 0.06 && u > 0.02) || (u > 0.94 && u < 0.98) { return Some(post); }
    }
    // Back glass panel
    if v > 0.12 && v < 0.72 && u > 0.08 && u < 0.92 {
        // Glass frame
        if u < 0.10 || u > 0.90 || v < 0.14 || v > 0.68 { return Some(post); }
        // Sign atop right
        if u > 0.70 && u < 0.90 && v > 0.14 && v < 0.28 {
            if u > 0.72 && u < 0.88 && v > 0.16 && v < 0.26 {
                if ((u * 20.0) as i32) % 3 == 0 && v > 0.19 && v < 0.24 { return Some(sign_txt); }
                return Some(sign_bg);
            }
        }
        return Some(glass);
    }
    // Bench
    if v >= 0.72 && v < 0.85 {
        if u > 0.14 && u < 0.86 { return Some(seat); }
    }
    // Bench legs
    if v >= 0.85 && v < 0.95 {
        if (u > 0.18 && u < 0.24) || (u > 0.50 && u < 0.56) || (u > 0.76 && u < 0.82) {
            return Some(post);
        }
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

fn surgical_cabinet_pixel(u: f64, v: f64) -> Option<(u8, u8, u8)> {
    if u < 0.0 || u > 1.0 || v < 0.0 || v > 1.0 { return None; }
    let frame = (220, 225, 232);
    let dark = (95, 105, 120);
    let glass = (185, 205, 220);
    let scalpel = (155, 160, 170);
    let gauze = (240, 240, 245);
    let mask = (110, 175, 155);
    let glove = (60, 120, 180);
    let red = (200, 50, 60);
    // Outer frame
    if u < 0.05 || u > 0.95 || v < 0.05 || v > 0.95 { return Some(dark); }
    // Shelf lines
    if (v - 0.35).abs() < 0.03 || (v - 0.65).abs() < 0.03 { return Some(dark); }
    // Vertical middle divider
    if (u - 0.5).abs() < 0.02 { return Some(dark); }
    // Content per cell
    let col = if u < 0.5 { 0 } else { 1 };
    let row = if v < 0.35 { 0 } else if v < 0.65 { 1 } else { 2 };
    match (row, col) {
        (0, 0) => {
            // Gauzes (white rolls)
            let cu = (u - 0.05) / 0.40;
            let cv = (v - 0.05) / 0.30;
            if cu > 0.15 && cu < 0.85 && cv > 0.15 && cv < 0.85 {
                let stripe = (cu * 3.0) as i32 % 2 == 0;
                if stripe { Some(gauze) } else { Some(frame) }
            } else { Some(glass) }
        }
        (0, 1) => {
            // Masks (green)
            let cu = (u - 0.52) / 0.42;
            let cv = (v - 0.05) / 0.30;
            if cv > 0.20 && cv < 0.80 && cu > 0.10 && cu < 0.90 {
                Some(mask)
            } else { Some(glass) }
        }
        (1, 0) => {
            // Scalpels/scissors
            let cu = (u - 0.05) / 0.40;
            let cv = (v - 0.35) / 0.30;
            if cv > 0.25 && cv < 0.75 {
                if cu > 0.15 && cu < 0.28 { Some(scalpel) }
                else if cu > 0.35 && cu < 0.48 { Some(scalpel) }
                else if cu > 0.60 && cu < 0.85 && (cv - 0.5).abs() < 0.08 { Some(scalpel) }
                else { Some(glass) }
            } else { Some(glass) }
        }
        (1, 1) => {
            // Gloves (blue)
            let cu = (u - 0.52) / 0.42;
            if cu > 0.15 && cu < 0.85 && (cu * 3.0).sin() > -0.3 {
                Some(glove)
            } else { Some(glass) }
        }
        (2, 0) => {
            // Antiseptic bottle (red)
            let cu = (u - 0.05) / 0.40;
            let cv = (v - 0.65) / 0.30;
            if cu > 0.30 && cu < 0.70 && cv > 0.20 && cv < 0.85 {
                Some(red)
            } else { Some(glass) }
        }
        (2, 1) => {
            // Extra gauze packs
            let cv = (v - 0.65) / 0.30;
            if cv > 0.30 && cv < 0.70 { Some(gauze) } else { Some(glass) }
        }
        _ => Some(glass),
    }
}

fn wooden_table_pixel(u: f64, v: f64) -> Option<(u8, u8, u8)> {
    if u < 0.0 || u > 1.0 || v < 0.0 || v > 1.0 { return None; }
    let top = (150, 100, 60);
    let dark = (80, 55, 30);
    let leg = (95, 65, 40);
    // Legs
    if v > 0.55 {
        if (u > 0.10 && u < 0.16) || (u > 0.84 && u < 0.90) { return Some(leg); }
        return None;
    }
    // Table top
    if v > 0.45 { return Some(dark); }
    let grain = ((u * 8.0).sin() * 0.05 + 1.0) as f64;
    let shade = if u < 0.5 { 1.0 } else { 0.90 };
    Some((
        ((top.0 as f64) * shade * grain) as u8,
        ((top.1 as f64) * shade * grain) as u8,
        ((top.2 as f64) * shade * grain) as u8,
    ))
}

fn operating_table_pixel(u: f64, v: f64) -> Option<(u8, u8, u8)> {
    if u < 0.0 || u > 1.0 || v < 0.0 || v > 1.0 { return None; }
    let steel_light = (200, 210, 220);
    let steel_dark = (110, 120, 135);
    let pad = (30, 55, 90);
    let seam = (18, 32, 55);
    let shadow = (35, 40, 50);
    // Wheeled legs
    if v > 0.90 {
        if (u > 0.06 && u < 0.11) || (u > 0.24 && u < 0.29)
            || (u > 0.71 && u < 0.76) || (u > 0.89 && u < 0.94) {
            return Some(shadow);
        }
        return None;
    }
    // Support column
    if v > 0.62 {
        let col_a = u > 0.18 && u < 0.30;
        let col_b = u > 0.70 && u < 0.82;
        if col_a || col_b { return Some(steel_dark); }
        return None;
    }
    // Top edge frame (steel)
    if v < 0.15 || v > 0.55 {
        if v < 0.05 || v > 0.60 { return None; }
        return Some(steel_dark);
    }
    // Padding surface (blue seamed)
    let seg = (u * 3.0) as i32;
    let seg_u = u * 3.0 - seg as f64;
    if seg_u < 0.04 { return Some(seam); }
    let shade = if u < 0.5 { 1.0 } else { 0.9 };
    let r = (pad.0 as f64 * shade) as u8;
    let g = (pad.1 as f64 * shade) as u8;
    let b = (pad.2 as f64 * shade) as u8;
    // Bright specular strip near top of pad
    if v < 0.22 { return Some(steel_light); }
    Some((r, g, b))
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
    if v > 0.10 {
        if (u > 0.05 && u < 0.09) || (u > 0.91 && u < 0.95) {
            return Some(leg);
        }
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

fn window_pixel(u: f64, v: f64, cloudy: bool) -> Option<(u8, u8, u8)> {
    if u < 0.0 || u > 1.0 || v < 0.0 || v > 1.0 { return None; }
    let frame = (60, 45, 30);
    let mullion = (55, 40, 25);
    // Outer frame
    if u < 0.08 || u > 0.92 || v < 0.08 || v > 0.92 { return Some(frame); }
    // Cross mullion
    if (u - 0.5).abs() < 0.03 || (v - 0.5).abs() < 0.03 { return Some(mullion); }
    // Glass panels
    if cloudy {
        let base = (150.0 + 40.0 * v, 160.0 + 40.0 * v, 175.0 + 30.0 * v);
        let cloud = ((u * 6.0).sin() * 0.3 + (v * 4.0).cos() * 0.2 + 0.35).clamp(0.0, 0.85);
        let r = (base.0 * (1.0 - cloud) + 240.0 * cloud) as u8;
        let g = (base.1 * (1.0 - cloud) + 240.0 * cloud) as u8;
        let b = (base.2 * (1.0 - cloud) + 240.0 * cloud) as u8;
        Some((r, g, b))
    } else {
        let r = (75.0 + 90.0 * v) as u8;
        let g = (130.0 + 90.0 * v) as u8;
        let b = (200.0 + 40.0 * v) as u8;
        Some((r, g, b))
    }
}

fn reception_pixel(u: f64, v: f64) -> Option<(u8, u8, u8)> {
    if u < 0.0 || u > 1.0 || v < 0.0 || v > 1.0 { return None; }
    let wood_light = (170, 130, 90);
    let wood_dark = (110, 78, 50);
    let top = (220, 220, 225);
    let front = (150, 115, 78);
    let shadow = (60, 45, 30);
    let panel_line = (85, 60, 40);

    // Base foot shadow
    if v > 0.96 { return Some(shadow); }

    // Countertop (top ~12%)
    if v < 0.14 {
        // Slight edge overhang shading
        if u < 0.02 || u > 0.98 { return Some(wood_dark); }
        if v < 0.03 { return Some(wood_light); }
        // Front lip
        if v > 0.10 { return Some(wood_dark); }
        // Top surface glossy
        return Some(top);
    }

    // Vertical front panels
    if u < 0.02 || u > 0.98 { return Some(shadow); }

    // Panel divisions: 4 panels
    let panel_col = (u * 4.0) as i32;
    let panel_u = (u * 4.0) - panel_col as f64;
    if panel_u < 0.04 || panel_u > 0.96 {
        return Some(panel_line);
    }

    // Front face
    let shade = if u < 0.5 { 1.0 } else { 0.88 };
    let (r, g, b) = front;
    let r = (r as f64 * shade) as u8;
    let g = (g as f64 * shade) as u8;
    let b = (b as f64 * shade) as u8;

    // Small drawer handles on middle panels
    let panel_v = v;
    if panel_v > 0.30 && panel_v < 0.34 && panel_u > 0.30 && panel_u < 0.70 {
        return Some((210, 205, 195));
    }

    Some((r, g, b))
}

fn vending_pixel(u: f64, v: f64, drinks: bool) -> Option<(u8, u8, u8)> {
    if u < 0.0 || u > 1.0 || v < 0.0 || v > 1.0 { return None; }
    let body = if drinks { (30, 60, 130) } else { (170, 40, 40) };
    let dark = (18, 20, 28);
    let glass = (60, 80, 110);
    let panel = (200, 200, 210);
    let led = (240, 220, 90);
    // Base pedestal
    if v > 0.95 { return Some(dark); }
    // Outer frame
    if u < 0.05 || u > 0.95 { return Some(dark); }
    if v < 0.05 { return Some(dark); }
    // Product window (top 60%)
    if v < 0.55 && u > 0.10 && u < 0.90 {
        // Grid of product slots
        let col = ((u - 0.10) / 0.80 * 4.0) as i32;
        let row = ((v - 0.05) / 0.50 * 4.0) as i32;
        // Slot dividers
        let uc = (u - 0.10) / 0.80 * 4.0 - col as f64;
        let vr = (v - 0.05) / 0.50 * 4.0 - row as f64;
        if uc < 0.08 || vr < 0.12 { return Some(dark); }
        // Product color varies
        let hue = ((col * 3 + row * 5) % 7) as f64;
        let r = (120.0 + hue * 20.0) as u8;
        let g = (180.0 - hue * 12.0).max(60.0) as u8;
        let b = (60.0 + hue * 25.0) as u8;
        if drinks {
            return Some((r.saturating_sub(30), g.saturating_sub(20), b + 20));
        }
        return Some((r, g, b));
    }
    // Glass reflection band
    if v < 0.60 { return Some(glass); }
    // Body panel
    if v < 0.88 {
        // Coin slot / keypad on right
        if u > 0.70 && u < 0.88 && v > 0.65 && v < 0.85 {
            if v < 0.68 { return Some(panel); }
            if v < 0.72 { return Some(led); }
            // Keypad dots
            let ku = ((u - 0.72) / 0.16 * 3.0) as i32;
            let kv = ((v - 0.74) / 0.11 * 3.0) as i32;
            let uc = (u - 0.72) / 0.16 * 3.0 - ku as f64;
            let vr = (v - 0.74) / 0.11 * 3.0 - kv as f64;
            if uc > 0.20 && uc < 0.80 && vr > 0.20 && vr < 0.80 {
                return Some((230, 230, 230));
            }
            return Some(dark);
        }
        // Delivery slot on left-bottom
        if u > 0.15 && u < 0.60 && v > 0.78 && v < 0.86 {
            return Some(dark);
        }
        // Brand shine
        let shade = if u < 0.5 { 1.0 } else { 0.85 };
        return Some((
            (body.0 as f64 * shade) as u8,
            (body.1 as f64 * shade) as u8,
            (body.2 as f64 * shade) as u8,
        ));
    }
    // Bottom bezel
    Some(dark)
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

fn sofa_pixel(u: f64, v: f64) -> Option<(u8, u8, u8)> {
    if u < 0.0 || u > 1.0 || v < 0.0 || v > 1.0 { return None; }
    let base = (110, 60, 90);
    let shade = |c: (u8, u8, u8), s: f64| -> (u8, u8, u8) {
        (
            (c.0 as f64 * s).clamp(0.0, 255.0) as u8,
            (c.1 as f64 * s).clamp(0.0, 255.0) as u8,
            (c.2 as f64 * s).clamp(0.0, 255.0) as u8,
        )
    };

    if v > 0.90 {
        let leg = (u > 0.05 && u < 0.10)
            || (u > 0.22 && u < 0.27)
            || (u > 0.73 && u < 0.78)
            || (u > 0.90 && u < 0.95);
        if leg { return Some((45, 28, 20)); }
        return None;
    }

    let arm_left = u < 0.15;
    let arm_right = u > 0.85;

    if arm_left || arm_right {
        if v < 0.32 { return None; }
        let corner_r = 0.06;
        let cu = if arm_left { 0.15 - corner_r } else { 0.85 + corner_r };
        let cv = 0.32 + corner_r;
        let d = ((u - cu).powi(2) + (v - cv).powi(2)).sqrt();
        if ((arm_left && u < 0.15 - corner_r) || (arm_right && u > 0.85 + corner_r))
            && v < 0.32 + corner_r
            && d > corner_r
        {
            return None;
        }
        let horiz = if arm_left { 1.0 - u * 0.4 } else { 0.72 + u * 0.20 };
        let vert = 1.0 - (v - 0.32) * 0.10;
        return Some(shade(base, 0.80 * vert * (horiz / 1.0).clamp(0.7, 1.05)));
    }

    if v < 0.55 {
        if v < 0.08 { return None; }
        let corner_r = 0.10;
        let bl_cu = 0.15 + corner_r;
        let br_cu = 0.85 - corner_r;
        let corner_cv = 0.08 + corner_r;
        if u < bl_cu && v < corner_cv {
            let d = ((u - bl_cu).powi(2) + (v - corner_cv).powi(2)).sqrt();
            if d > corner_r { return None; }
        }
        if u > br_cu && v < corner_cv {
            let d = ((u - br_cu).powi(2) + (v - corner_cv).powi(2)).sqrt();
            if d > corner_r { return None; }
        }
        let cushion_split = ((u - 0.50).abs() < 0.008) && v > 0.15;
        if cushion_split { return Some(shade(base, 0.55)); }
        let vert_shade = 1.0 - (v - 0.08) * 0.35;
        let side_shade = if u < 0.50 { 1.0 } else { 0.90 };
        return Some(shade(base, vert_shade * side_shade));
    }

    if v < 0.88 {
        let cushion_line = (u - 0.50).abs() < 0.008;
        if cushion_line { return Some((70, 38, 60)); }
        let front_lip = v > 0.83;
        if front_lip {
            return Some(shade((135, 78, 108), 0.75));
        }
        let vert_shade = 1.0 - (v - 0.55) * 0.20;
        let side_shade = if u < 0.50 { 1.05 } else { 0.92 };
        return Some(shade((135, 78, 108), vert_shade * side_shade));
    }

    None
}

fn outlet_pixel(u: f64, v: f64) -> Option<(u8, u8, u8)> {
    if u < 0.0 || u > 1.0 || v < 0.0 || v > 1.0 { return None; }
    let plate = (220, 215, 205);
    let hole = (25, 25, 30);
    let cx = 0.5;
    let cy = 0.5;
    let dx = u - cx;
    let dy = v - cy;
    let d = (dx * dx + dy * dy).sqrt();
    if d > 0.48 { return None; }
    let hole1 = ((u - 0.38).powi(2) + (v - 0.5).powi(2)).sqrt() < 0.08;
    let hole2 = ((u - 0.62).powi(2) + (v - 0.5).powi(2)).sqrt() < 0.08;
    if hole1 || hole2 { return Some(hole); }
    Some(plate)
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

fn sign_pixel(u: f64, v: f64, kind: SignShapeKind) -> Option<(u8, u8, u8)> {
    if u < 0.0 || u > 1.0 || v < 0.0 || v > 1.0 { return None; }
    let border = match kind {
        SignShapeKind::Bathroom => 0.08,
        _ => 0.12,
    };
    let inside = u > border && u < 1.0 - border && v > border && v < 1.0 - border;
    let (fill, edge, text) = match kind {
        SignShapeKind::RoomSmall => ((220, 180, 130), (110, 80, 40), None),
        SignShapeKind::MainSquare => ((80, 145, 200), (25, 60, 100), None),
        SignShapeKind::HallwayLong => ((230, 200, 90), (110, 80, 20), None),
        SignShapeKind::Bathroom => ((235, 238, 242), (60, 70, 85), Some((20, 30, 45))),
        SignShapeKind::HospitalPlaque => ((235, 240, 245), (60, 70, 90), Some((190, 40, 40))),
    };
    if !inside { return Some(edge); }
    if matches!(kind, SignShapeKind::HospitalPlaque) {
        if hospital_glyph_hit(u, v) { return text.map(|c| c).or(Some(fill)).map(|c| c); }
    }
    if let Some(tc) = text {
        if matches!(kind, SignShapeKind::HospitalPlaque) { /* handled above */ } else if wc_glyph_hit(u, v) { return Some(tc); }
    }
    Some(fill)
}

fn hospital_glyph_hit(u: f64, v: f64) -> bool {
    // Simple "H" glyph in the middle
    if v < 0.20 || v > 0.80 { return false; }
    let cu = (u - 0.5) / 0.35;  // ~-1 to 1
    if cu.abs() > 1.0 { return false; }
    // Two vertical bars + horizontal cross
    let left_bar = cu > -0.85 && cu < -0.55;
    let right_bar = cu > 0.55 && cu < 0.85;
    let cross = v > 0.45 && v < 0.55 && cu > -0.85 && cu < 0.85;
    left_bar || right_bar || cross
}

fn wc_glyph_hit(u: f64, v: f64) -> bool {
    if v < 0.22 || v > 0.78 { return false; }
    let ty = ((v - 0.22) / 0.56 * 5.0) as usize;
    if ty >= 5 { return false; }
    let text_left = 0.14;
    let text_right = 0.86;
    if u < text_left || u > text_right { return false; }
    let tx = ((u - text_left) / (text_right - text_left) * 11.0) as usize;
    if tx >= 11 { return false; }
    const GLYPHS: [[u8; 11]; 5] = [
        [1,0,0,0,1, 0, 0,1,1,1,0],
        [1,0,0,0,1, 0, 1,0,0,0,0],
        [1,0,1,0,1, 0, 1,0,0,0,0],
        [1,1,0,1,1, 0, 1,0,0,0,0],
        [1,0,0,0,1, 0, 0,1,1,1,0],
    ];
    GLYPHS[ty][tx] == 1
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

fn bench_pixel(u: f64, v: f64) -> Option<(u8, u8, u8)> {
    let seat_top = 0.05;
    let seat_bot = 0.55;
    let legs_top = seat_bot;
    let legs_bot = 0.98;

    if v >= seat_top && v <= seat_bot {
        let slat_gap_1 = (v - 0.22).abs() < 0.014;
        let slat_gap_2 = (v - 0.38).abs() < 0.014;
        if slat_gap_1 || slat_gap_2 { return Some((45, 50, 58)); }

        let edge = (v - seat_top).abs() < 0.02
            || (v - seat_bot).abs() < 0.02
            || (u - 0.04).abs() < 0.015
            || (u - 0.96).abs() < 0.015;
        if edge { return Some((100, 105, 115)); }

        let brushed = ((u * 60.0).sin() * 0.04 + 1.0).max(0.90);
        let shade = brushed * (0.90 + (1.0 - v) * 0.18);
        let r = (150.0 * shade).clamp(0.0, 255.0) as u8;
        let g = (155.0 * shade).clamp(0.0, 255.0) as u8;
        let b = (165.0 * shade).clamp(0.0, 255.0) as u8;
        return Some((r, g, b));
    }

    if v >= legs_top && v <= legs_bot {
        let leg_l = (u - 0.14).abs() < 0.035;
        let leg_r = (u - 0.86).abs() < 0.035;
        if leg_l || leg_r {
            let highlight = if (u - 0.14).abs() < 0.010 || (u - 0.86).abs() < 0.010 { 1.15 } else { 1.0 };
            let shade = 0.80 + (1.0 - v) * 0.15;
            let base = 105.0 * shade * highlight;
            let r = (base * 0.95).clamp(0.0, 255.0) as u8;
            let g = (base).clamp(0.0, 255.0) as u8;
            let b = (base * 1.10).clamp(0.0, 255.0) as u8;
            return Some((r, g, b));
        }
    }

    None
}

fn bed_pixel(u: f64, v: f64) -> Option<(u8, u8, u8)> {
    let frame_top = 0.10;
    let frame_bot = 0.98;
    let frame_left = 0.05;
    let frame_right = 0.95;
    if v < frame_top || v > frame_bot { return None; }
    if u < frame_left || u > frame_right { return None; }

    let leg_h = v > 0.86;
    let leg_here = leg_h && ((u - 0.10).abs() < 0.04 || (u - 0.90).abs() < 0.04);
    if leg_here { return Some((70, 70, 75)); }
    if leg_h && !leg_here { return None; }

    let headboard = u < 0.14;
    if headboard {
        if v < 0.20 || v > 0.80 { return None; }
        return Some((190, 190, 200));
    }

    let footboard_top = 0.20;
    let mattress_body = v > footboard_top && v < 0.86;
    if !mattress_body { return None; }

    let pillow = u > 0.16 && u < 0.30 && v > 0.24 && v < 0.42;
    if pillow {
        return Some((250, 248, 240));
    }

    let sheet_stripe = v > 0.50 && v < 0.52;
    if sheet_stripe { return Some((180, 190, 200)); }

    let mattress_edge = (v - footboard_top).abs() < 0.02 || (v - 0.86).abs() < 0.02
        || (u - frame_left).abs() < 0.02 || (u - frame_right).abs() < 0.02;
    if mattress_edge { return Some((150, 150, 155)); }

    let shade = 0.90 + (1.0 - v) * 0.10;
    let base = 220.0 * shade;
    Some((base as u8, base as u8, (base * 0.98) as u8))
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
    if v > hook_y - 0.04 && v < hook_y + 0.06 {
        if (u - 0.20).abs() < 0.05 || (u - 0.80).abs() < 0.05 {
            return Some((160, 130, 80));
        }
    }
    let hook_y2 = 0.20;
    if v > hook_y2 - 0.03 && v < hook_y2 + 0.04 {
        if (u - 0.30).abs() < 0.03 || (u - 0.70).abs() < 0.03 {
            return Some((160, 130, 80));
        }
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

fn person_pixel(
    u: f64, v: f64,
    skin: (u8, u8, u8),
    shirt: (u8, u8, u8),
    pants: (u8, u8, u8),
) -> Option<(u8, u8, u8)> {
    let head_cy = 0.15;
    let head_r = 0.11;
    let du = u - 0.5;
    let dv = v - head_cy;
    if du * du + dv * dv < head_r * head_r {
        if v < 0.10 {
            return Some(darken(skin, 0.55));
        }
        return Some(skin);
    }

    let neck = v > 0.24 && v < 0.30 && (u - 0.5).abs() < 0.05;
    if neck { return Some(darken(skin, 0.85)); }

    let torso_top = 0.30;
    let torso_bot = 0.62;
    if v > torso_top && v < torso_bot {
        let half = 0.15 + (v - torso_top) / (torso_bot - torso_top) * 0.03;
        if (u - 0.5).abs() < half {
            let stripe = if (v - 0.42).abs() < 0.01 { 0.85 } else { 1.0 };
            return Some(mul(shirt, stripe));
        }
        let arm_l = (u - 0.30).abs() < 0.06 && v < 0.55;
        let arm_r = (u - 0.70).abs() < 0.06 && v < 0.55;
        if arm_l || arm_r { return Some(shirt); }
        let hand_l = (u - 0.28).abs() < 0.05 && v > 0.53 && v < 0.60;
        let hand_r = (u - 0.72).abs() < 0.05 && v > 0.53 && v < 0.60;
        if hand_l || hand_r { return Some(skin); }
    }

    if v >= torso_bot && v < 0.94 {
        let leg_l = (u - 0.40).abs() < 0.07;
        let leg_r = (u - 0.60).abs() < 0.07;
        if leg_l || leg_r { return Some(pants); }
    }

    if v >= 0.94 && v < 1.0 {
        let shoe_l = (u - 0.38).abs() < 0.08;
        let shoe_r = (u - 0.62).abs() < 0.08;
        if shoe_l || shoe_r { return Some((40, 30, 20)); }
    }

    None
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
