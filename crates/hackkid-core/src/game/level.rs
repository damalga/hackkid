//! The hospital: its walls and doors, and where everything in it starts out.
//!
//! The geometry is stamped room by room onto a 60×82 tile map; every object list is
//! in tile coordinates (x east, y south). Moving content around happens here only.

use crate::engine::map::{Map, Tile};
use crate::equippables::{Backpack, Equippable};
use crate::game::items::{Item, ItemKind};
use crate::game::world::DroppedItem;
use crate::objects::{
    Bed, Bench, Burra, Clothing, ClothingKind, CoatRack, Decor, DecorKind, Door, Fixture, Fluorescent, FluorescentAxis,
    FluorescentState, Npc, Outlet, Reception, Sign, SignKind, Sofa, VendingKind, VendingMachine, Window, private_bathroom,
    public_bathroom,
};

/// Where the player wakes up.
pub const PLAYER_START: (f64, f64) = (13.5, 4.5);

pub struct Level {
    pub map: Map,
    pub doors: Vec<Door>,
    pub fluorescents: Vec<Fluorescent>,
    pub equippables: Vec<Equippable>,
    pub clothing: Vec<Clothing>,
    pub beds: Vec<Bed>,
    pub coat_racks: Vec<CoatRack>,
    pub benches: Vec<Bench>,
    pub sofas: Vec<Sofa>,
    pub outlets: Vec<Outlet>,
    pub fixtures: Vec<Fixture>,
    pub vending: Vec<VendingMachine>,
    pub receptions: Vec<Reception>,
    pub windows: Vec<Window>,
    pub burras: Vec<Burra>,
    pub decors: Vec<Decor>,
    pub npcs: Vec<Npc>,
    pub signs: Vec<Sign>,
    /// Loose items lying around at the start.
    pub items: Vec<DroppedItem>,
}

pub fn hospital() -> Level {
    let map = hospital_map();

    let doors = vec![
        Door::new(13, 6),
        Door::new(19, 6),
        Door::new(25, 6),
        Door::new(31, 6),
        Door::bathroom(14, 3),
        Door::bathroom(20, 3),
        Door::bathroom(26, 3),
        Door::bathroom(32, 3),
        Door::new(36, 16),
        Door::new(41, 16),
        Door::main(22, 12),
        Door::bathroom(10, 19),
        Door::bathroom(6, 19),
        Door::main(34, 19),
        Door::main(22, 23),
        Door::main(22, 44),
        Door::street(22, 59),
    ];

    let fluorescents = vec![
        Fluorescent::new(13.5, 4.0, 0.35, FluorescentAxis::Horizontal),
        Fluorescent::new(19.5, 4.0, 0.35, FluorescentAxis::Horizontal)
            .with_state(FluorescentState::Flicker { period_ms: 1400, duty: 0.72 }),
        Fluorescent::new(25.5, 4.0, 0.35, FluorescentAxis::Horizontal),
        Fluorescent::new(31.5, 4.0, 0.35, FluorescentAxis::Horizontal)
            .with_state(FluorescentState::Dead),
        Fluorescent::new(17.0, 9.0, 0.55, FluorescentAxis::Horizontal),
        Fluorescent::new(22.0, 9.0, 0.55, FluorescentAxis::Horizontal)
            .with_state(FluorescentState::Flicker { period_ms: 900, duty: 0.55 }),
        Fluorescent::new(27.0, 9.0, 0.55, FluorescentAxis::Horizontal),

        Fluorescent::new(14.0, 14.0, 0.60, FluorescentAxis::Horizontal),
        Fluorescent::new(22.0, 14.0, 0.60, FluorescentAxis::Horizontal),
        Fluorescent::new(30.0, 14.0, 0.60, FluorescentAxis::Horizontal)
            .with_state(FluorescentState::Flicker { period_ms: 700, duty: 0.60 }),
        Fluorescent::new(14.0, 18.0, 0.60, FluorescentAxis::Horizontal)
            .with_state(FluorescentState::Flicker { period_ms: 1600, duty: 0.80 }),
        Fluorescent::new(22.0, 18.0, 0.60, FluorescentAxis::Horizontal),
        Fluorescent::new(30.0, 18.0, 0.60, FluorescentAxis::Horizontal),
        Fluorescent::new(22.0, 21.5, 0.60, FluorescentAxis::Horizontal),
        Fluorescent::new(36.5, 14.0, 0.35, FluorescentAxis::Horizontal),
        Fluorescent::new(41.5, 14.0, 0.35, FluorescentAxis::Horizontal),
        Fluorescent::new(39.0, 19.5, 0.55, FluorescentAxis::Horizontal),
        Fluorescent::new(22.0, 30.0, 0.6, FluorescentAxis::Horizontal),
        Fluorescent::new(22.0, 40.0, 0.6, FluorescentAxis::Horizontal),
        Fluorescent::new(15.0, 46.5, 0.55, FluorescentAxis::Horizontal),
        Fluorescent::new(22.5, 46.5, 0.55, FluorescentAxis::Horizontal),
        Fluorescent::new(29.0, 46.5, 0.55, FluorescentAxis::Horizontal),
        Fluorescent::new(15.0, 56.5, 0.55, FluorescentAxis::Horizontal)
            .with_state(FluorescentState::Flicker { period_ms: 1300, duty: 0.70 }),
        Fluorescent::new(22.5, 56.5, 0.55, FluorescentAxis::Horizontal),
        Fluorescent::new(29.0, 56.5, 0.55, FluorescentAxis::Horizontal),
    ];

    let equippables = vec![
        Equippable::Backpack(Backpack::player_starter(14.5, 5.5)),
    ];

    let mut clothing = vec![
        Clothing::new(19.5, 4.7, ClothingKind::HospitalGown),
        Clothing::new(21.5, 10.5, ClothingKind::ScrubBundle),
        Clothing::new(22.5, 10.7, ClothingKind::HospitalGown),
        Clothing::new(14.0, 16.0, ClothingKind::HospitalGown),
        Clothing::new(19.5, 15.5, ClothingKind::ScrubBundle),
        Clothing::new(26.0, 17.5, ClothingKind::HospitalGown),
        Clothing::new(17.5, 19.5, ClothingKind::ScrubBundle),
        Clothing::new(28.5, 20.5, ClothingKind::HospitalGown),
        Clothing::new(30.5, 16.5, ClothingKind::HospitalGown),
        Clothing::new(15.5, 21.5, ClothingKind::ScrubBundle),
        Clothing::new(24.5, 21.8, ClothingKind::HospitalGown),
        Clothing::new(11.5, 3.55, ClothingKind::Hoodie).with_elevation(0.32),
        Clothing::new(11.2, 14.5, ClothingKind::Pants).with_elevation(0.32),
        // Outdoor ground clothing (elevation 0 = not pickable, just description)
        Clothing::new(12.0, 68.0, ClothingKind::HospitalGown),
        Clothing::new(20.5, 70.5, ClothingKind::ScrubBundle),
        Clothing::new(28.0, 66.5, ClothingKind::HospitalGown),
        Clothing::new(34.5, 74.5, ClothingKind::ScrubBundle),
        Clothing::new(48.0, 27.5, ClothingKind::HospitalGown),
        Clothing::new(51.0, 40.0, ClothingKind::ScrubBundle),
        Clothing::new(53.5, 62.5, ClothingKind::HospitalGown),
        Clothing::new(46.5, 71.5, ClothingKind::ScrubBundle),
    ];

    let mut beds = vec![
        Bed::new(11.5, 5.5),
        Bed::new(17.5, 5.5),
        Bed::new(23.5, 5.5),
        Bed::new(29.5, 5.5),
        Bed::operating(36.5, 14.0),
    ];

    let mut coat_racks = vec![
        CoatRack::new(11.5, 3.5),
        CoatRack::new(17.5, 3.5),
        CoatRack::new(23.5, 3.5),
        CoatRack::new(29.5, 3.5),
    ];

    let mut benches = vec![
        Bench::new(16.0, 13.4),
        Bench::new(11.15, 14.5),
        Bench::new(32.85, 14.5),
        Bench::new(28.0, 22.3),
    ];

    let mut sofas = vec![
        Sofa::new(14.5, 5.55),
        Sofa::new(20.5, 5.55),
        Sofa::new(26.5, 5.55),
        Sofa::new(32.5, 5.55),
        Sofa::new(43.15, 14.5),
    ];

    let mut fixtures: Vec<Fixture> = Vec::new();
    fixtures.extend(private_bathroom(13.5, 1.5));
    fixtures.extend(private_bathroom(19.5, 1.5));
    fixtures.extend(private_bathroom(25.5, 1.5));
    fixtures.extend(private_bathroom(31.5, 1.5));
    // the two public restrooms, x 3..6 and 7..10, along their north walls
    fixtures.extend(public_bathroom(3.35, 18.3));
    fixtures.extend(public_bathroom(7.35, 18.3));

    let mut outlets = vec![
        // Origin rooms: outlet next to bed on south wall
        Outlet::new(11.5, 5.98),
        Outlet::new(17.5, 5.98),
        Outlet::new(23.5, 5.98),
        Outlet::new(29.5, 5.98),
        // Origin rooms: outlet next to sofa on south wall
        Outlet::new(14.5, 5.98),
        Outlet::new(20.5, 5.98),
        Outlet::new(26.5, 5.98),
        Outlet::new(32.5, 5.98),
        // Operating room outlet on top wall next to bed head
        Outlet::new(36.5, 13.02),
        // Waiting room: outlet on east wall next to sofa
        Outlet::new(43.98, 14.5),
        // Hub benches
        Outlet::new(16.8, 13.02),
        Outlet::new(11.02, 14.5),
        Outlet::new(33.98, 14.5),
        Outlet::new(28.7, 22.98),
    ];

    let mut signs = vec![
        Sign::new(14.3, 7.02, SignKind::RoomSmall),
        Sign::new(20.3, 7.02, SignKind::RoomSmall),
        Sign::new(26.3, 7.02, SignKind::RoomSmall),
        Sign::new(32.3, 7.02, SignKind::RoomSmall),
        Sign::new(37.3, 17.02, SignKind::RoomSmall),
        Sign::new(42.3, 17.02, SignKind::RoomSmall),
        Sign::new(23.3, 13.02, SignKind::MainSquare),
        Sign::new(11.02, 20.3, SignKind::Bathroom),
        Sign::new(33.98, 20.3, SignKind::MainSquare),
        Sign::new(23.3, 22.98, SignKind::HallwayLong),
        // Big HOSPITAL plaque outside street door
        Sign::new(24.5, 59.15, SignKind::HospitalPlaque),
    ];

    let npcs = vec![
        Npc::julian(17.5, 17.5),
    ];

    let mut vending = vec![
        VendingMachine::new(15.5, 58.85, VendingKind::Snacks, 3),
        VendingMachine::new(17.2, 58.85, VendingKind::Drinks, 2),
    ];

    let mut receptions = vec![
        Reception::new(27.5, 56.6),
    ];

    let mut windows = vec![
        Window::new(11.02, 50.0),
        Window::new(33.98, 50.0),
    ];

    let mut burras = vec![
        // Operating room: burra against east wall (side)
        Burra::new(37.85, 15.4),
    ];

    let mut decors = vec![
        // Operating room aluminum tables against north wall
        Decor::new(35.5, 13.35, DecorKind::AluminumTable),
        Decor::new(37.5, 13.35, DecorKind::AluminumTable),
        // Surgical cabinet on west wall (elevated - wall-mounted)
        Decor::new(35.15, 14.5, DecorKind::SurgicalCabinet).with_elevation(0.35),
        // Waiting room wooden table
        Decor::new(40.7, 13.7, DecorKind::WoodenTable),

        // Bus stop and mailbox on the sidewalk at y=63.5
        Decor::new(18.5, 63.4, DecorKind::BusStop),
        Decor::new(25.0, 63.4, DecorKind::Mailbox),
        Decor::new(46.5, 40.0, DecorKind::Mailbox),

        // South frontage trees + bushes (garden between hospital wall y=59 and sidewalk y=60)
        Decor::new(6.0, 62.0, DecorKind::TreeOak),
        Decor::new(13.5, 68.0, DecorKind::TreePine),
        Decor::new(30.0, 68.0, DecorKind::TreeOak),
        Decor::new(38.5, 68.5, DecorKind::TreePine),
        Decor::new(43.5, 68.0, DecorKind::TreeOak),
        Decor::new(3.5, 68.5, DecorKind::TreeOak),
        Decor::new(3.5, 75.0, DecorKind::TreePine),

        // South garden bushes (below parking lot y=65..66)
        Decor::new(7.5, 72.0, DecorKind::Bush),
        Decor::new(11.5, 72.5, DecorKind::Bush),
        Decor::new(17.5, 73.0, DecorKind::Bush),
        Decor::new(21.5, 72.0, DecorKind::Bush),
        Decor::new(26.5, 74.0, DecorKind::TreeOak),
        Decor::new(32.5, 72.5, DecorKind::Bush),
        Decor::new(36.5, 74.0, DecorKind::Bush),
        Decor::new(41.5, 72.5, DecorKind::Bush),
        Decor::new(45.5, 74.0, DecorKind::Bush),
        Decor::new(49.5, 72.5, DecorKind::TreePine),
        Decor::new(53.5, 71.5, DecorKind::Bush),
        Decor::new(56.5, 73.5, DecorKind::TreeOak),

        // East side trees, bushes (garden between hospital wall x=45 and sidewalk x=46)
        Decor::new(4.5, 30.0, DecorKind::TreeOak),
        Decor::new(3.5, 42.0, DecorKind::TreePine),
        Decor::new(4.5, 55.0, DecorKind::TreeOak),
        Decor::new(50.0, 8.0, DecorKind::TreeOak),
        Decor::new(53.5, 5.5, DecorKind::TreePine),
        Decor::new(54.5, 33.0, DecorKind::TreePine),
        Decor::new(48.5, 45.0, DecorKind::Bush),
        Decor::new(54.5, 45.0, DecorKind::TreeOak),
        Decor::new(56.5, 55.0, DecorKind::TreeOak),
        Decor::new(54.5, 62.0, DecorKind::TreePine),
        Decor::new(58.0, 40.0, DecorKind::Bush),
        Decor::new(58.0, 25.0, DecorKind::Bush),
    ];

    // Scatter weeds across garden tiles
    let weed_positions: &[(f64, f64)] = &[
        (2.5, 3.5), (5.5, 6.5), (7.5, 10.5), (2.5, 15.5), (5.5, 23.5),
        (2.5, 30.5), (4.5, 36.5), (2.5, 44.5), (5.5, 50.5), (2.5, 58.5),
        (6.5, 61.5), (16.5, 61.5), (28.5, 61.5), (40.5, 61.5), (5.5, 68.5),
        (9.5, 68.5), (15.5, 71.5), (19.5, 74.5), (24.5, 68.5), (34.5, 71.5),
        (44.5, 68.5), (52.5, 40.5), (58.5, 15.5), (56.5, 3.5), (58.5, 55.5),
        (52.5, 72.5), (48.5, 74.5), (7.5, 76.5), (14.5, 78.5), (22.5, 78.5),
        (30.5, 78.5), (38.5, 78.5), (46.5, 78.5), (55.5, 78.5),
    ];
    for (wx, wy) in weed_positions {
        decors.push(Decor::new(*wx, *wy, DecorKind::Weed));
    }

    // Cars in south parking lot (rows y=65..66, stalls 2 tiles wide centered at x=8.5,10.5,...)
    let car_colors: &[(u8, u8, u8)] = &[
        (185, 40, 40),   // red
        (30, 90, 190),   // blue
        (24, 24, 28),    // black
        (225, 225, 228), // white
        (200, 180, 60),  // yellow
        (40, 100, 55),   // green
        (110, 60, 30),   // brown
        (140, 145, 155), // silver
    ];
    let south_stall_xs = [8.5_f64, 10.5, 12.5, 14.5, 16.5, 22.5, 26.5, 28.5, 32.5, 34.5, 40.5, 44.5];
    for (i, sx) in south_stall_xs.iter().enumerate() {
        let color = car_colors[i % car_colors.len()];
        decors.push(Decor::new(*sx, 65.6, DecorKind::Car { body: color }));
    }
    // East parking column: cars stacked vertically (stalls 2 tiles tall each)
    let east_stall_ys = [7.5_f64, 9.5, 12.5, 14.5, 17.5, 19.5, 22.5, 25.5, 28.5];
    for (i, sy) in east_stall_ys.iter().enumerate() {
        let color = car_colors[(i + 3) % car_colors.len()];
        decors.push(Decor::new(51.6, *sy, DecorKind::Car { body: color }));
    }
    // A couple of cars parked at the road shoulder
    decors.push(Decor::new(20.5, 62.4, DecorKind::Car { body: (30, 90, 190) }));
    decors.push(Decor::new(36.5, 62.4, DecorKind::Car { body: (24, 24, 28) }));

    let door_centers: Vec<(f64, f64)> = doors.iter()
        .map(|d| (d.tx as f64 + 0.5, d.ty as f64 + 0.5))
        .collect();
    for s in &mut signs { push_away_from_doors(&mut s.x, &mut s.y, &door_centers, 1.6); }
    for o in &mut outlets { push_away_from_doors(&mut o.x, &mut o.y, &door_centers, 1.3); }
    for c in &mut coat_racks { push_away_from_doors(&mut c.x, &mut c.y, &door_centers, 1.4); }
    for b in &mut benches { push_away_from_doors(&mut b.x, &mut b.y, &door_centers, 1.6); }
    for s in &mut sofas { push_away_from_doors(&mut s.x, &mut s.y, &door_centers, 1.2); }
    for b in &mut beds { push_away_from_doors(&mut b.x, &mut b.y, &door_centers, 1.8); }
    for f in &mut fixtures { push_away_from_doors(&mut f.x, &mut f.y, &door_centers, 1.3); }
    for c in &mut clothing { push_away_from_doors(&mut c.x, &mut c.y, &door_centers, 1.4); }
    for b in &mut burras { push_away_from_doors(&mut b.x, &mut b.y, &door_centers, 1.6); }
    for v in &mut vending { push_away_from_doors(&mut v.x, &mut v.y, &door_centers, 1.3); }
    for r in &mut receptions { push_away_from_doors(&mut r.x, &mut r.y, &door_centers, 1.5); }
    for w in &mut windows { push_away_from_doors(&mut w.x, &mut w.y, &door_centers, 1.2); }
    for d in &mut decors { push_away_from_doors(&mut d.x, &mut d.y, &door_centers, 1.6); }

    // the triage keycard that opens the sealed east ward, left at the reception desk
    let items = vec![DroppedItem { x: 27.5, y: 55.7, item: Item::new(ItemKind::TriageKeycard) }];

    Level {
        map, doors, fluorescents, equippables, clothing, beds, coat_racks, benches, sofas, outlets,
        fixtures, vending, receptions, windows, burras, decors, npcs, signs, items,
    }
}

/// Nudges furniture that ended up in front of a doorway out of the way.
fn push_away_from_doors(x: &mut f64, y: &mut f64, doors: &[(f64, f64)], min_d: f64) {
    for (dx, dy) in doors {
        let vx = *x - dx;
        let vy = *y - dy;
        let d2 = vx * vx + vy * vy;
        if d2 < 0.0001 { continue; }
        let d = d2.sqrt();
        if d < min_d {
            let push = (min_d - d) * 0.6;
            *x += vx / d * push;
            *y += vy / d * push;
        }
    }
}

fn hospital_map() -> Map {
    let w = 60;
    let h = 82;
    let mut map = Map::new(w, h);

    stamp_origin(&mut map, 10, 0);
    stamp_hub(&mut map, 10, 12);
    stamp_bathrooms(&mut map, 2, 17);
    stamp_right_ward(&mut map, 34, 12);
    stamp_front_ward(&mut map, 20, 23);
    stamp_vestibule(&mut map, 10, 44);
    stamp_around_hospital(&mut map);
    stamp_parking(&mut map);

    map.set(22, 12, Tile::Door { open: false });
    map.set(10, 19, Tile::BathroomDoor { open: false });
    // between the two public restrooms: the far one has no other way in
    map.set(6, 19, Tile::BathroomDoor { open: false });
    map.set(34, 19, Tile::OperatingDoor { open: false });
    map.set(22, 23, Tile::Door { open: false });
    map.set(22, 44, Tile::Door { open: false });
    map.set(22, 59, Tile::MainDoor { open: false });

    map
}

fn stamp_origin(map: &mut Map, x0: usize, y0: usize) {
    let w = 25;
    let h = 13;
    for x in 0..w {
        map.set(x0 + x, y0, Tile::Wall);
        map.set(x0 + x, y0 + h - 1, Tile::Wall);
    }
    for y in 0..h {
        map.set(x0, y0 + y, Tile::Wall);
        map.set(x0 + w - 1, y0 + y, Tile::Wall);
    }
    // Column walls between rooms
    for y in 1..6 {
        for x in [6usize, 12, 18] {
            map.set(x0 + x, y0 + y, Tile::Wall);
        }
    }
    // Cross wall separating rooms from hallway
    for x in 1..w - 1 {
        map.set(x0 + x, y0 + 6, Tile::Wall);
    }
    // Doors to hallway at the center of each room
    for x in [3usize, 9, 15, 21] {
        map.set(x0 + x, y0 + 6, Tile::Door { open: false });
    }
    // Bathroom sub-room in NE corner of each patient room (3 wide × 2 tall)
    // Bathroom interior: (bx1..=bx1+2, y0+1..=y0+2)
    // Walls (all BathroomWall, painted white):
    //   west (bx1-1, y0+1..=y0+2)
    //   south (bx1..=bx1+2, y0+3) with door at center
    //   north (bx1..=bx1+2, y0)  — part of the outer wall
    //   east (bx1+3, y0+1..=y0+2) — part of the wall between rooms
    let room_starts = [1usize, 7, 13, 19];
    for &start in &room_starts {
        let bx1 = x0 + start + 2;
        let west_wall_x = x0 + start + 1;
        let east_wall_x = x0 + start + 5;
        map.set(west_wall_x, y0 + 1, Tile::BathroomWall);
        map.set(west_wall_x, y0 + 2, Tile::BathroomWall);
        map.set(bx1, y0 + 3, Tile::BathroomWall);
        map.set(bx1 + 1, y0 + 3, Tile::BathroomDoor { open: false });
        map.set(bx1 + 2, y0 + 3, Tile::BathroomWall);
        map.set(bx1, y0, Tile::BathroomWall);
        map.set(bx1 + 1, y0, Tile::BathroomWall);
        map.set(bx1 + 2, y0, Tile::BathroomWall);
        map.set(east_wall_x, y0 + 1, Tile::BathroomWall);
        map.set(east_wall_x, y0 + 2, Tile::BathroomWall);
    }
}

fn stamp_hub(map: &mut Map, x0: usize, y0: usize) {
    let w = 25;
    let h = 12;
    for x in 0..w {
        map.set(x0 + x, y0, Tile::Wall);
        map.set(x0 + x, y0 + h - 1, Tile::Wall);
    }
    for y in 0..h {
        map.set(x0, y0 + y, Tile::Wall);
        map.set(x0 + w - 1, y0 + y, Tile::Wall);
    }
    // Two interior columns (tall)
    map.set(x0 + 8, y0 + 6, Tile::Column);
    map.set(x0 + 16, y0 + 6, Tile::Column);
}

fn stamp_bathrooms(map: &mut Map, x0: usize, y0: usize) {
    let w = 9;
    let h = 6;
    for x in 0..w {
        map.set(x0 + x, y0, Tile::BathroomWall);
        map.set(x0 + x, y0 + h - 1, Tile::BathroomWall);
    }
    for y in 0..h {
        map.set(x0, y0 + y, Tile::BathroomWall);
    }
    for y in 1..h - 1 {
        map.set(x0 + 4, y0 + y, Tile::BathroomWall);
    }
}

fn stamp_right_ward(map: &mut Map, x0: usize, y0: usize) {
    let w = 11;
    let h = 12;
    for x in 0..w {
        map.set(x0 + x, y0, Tile::BathroomWall);
        map.set(x0 + x, y0 + h - 1, Tile::BathroomWall);
    }
    for y in 0..h {
        map.set(x0 + w - 1, y0 + y, Tile::BathroomWall);
    }
    for y in 1..4 {
        map.set(x0 + 5, y0 + y, Tile::BathroomWall);
    }
    for x in 1..w - 1 {
        map.set(x0 + x, y0 + 4, Tile::BathroomWall);
    }
    // Operating room door (blue) on left, regular door (brown) on right for waiting room
    map.set(x0 + 2, y0 + 4, Tile::OperatingDoor { open: false });
    map.set(x0 + 7, y0 + 4, Tile::Door { open: false });
}

fn stamp_front_ward(map: &mut Map, x0: usize, y0: usize) {
    let w = 5;
    let h = 22;
    for x in 0..w {
        map.set(x0 + x, y0, Tile::Wall);
        map.set(x0 + x, y0 + h - 1, Tile::Wall);
    }
    for y in 0..h {
        map.set(x0, y0 + y, Tile::Wall);
        map.set(x0 + w - 1, y0 + y, Tile::Wall);
    }
}

fn stamp_around_hospital(map: &mut Map) {
    let w = map.width;
    let h = map.height;
    // Interior bounding boxes (x0, y0, x1, y1) exclusive on end
    let boxes: &[(usize, usize, usize, usize)] = &[
        (10, 0, 35, 12),   // origin rooms
        (10, 12, 35, 24),  // hub
        (2, 17, 11, 23),   // public bathrooms
        (34, 12, 45, 24),  // right ward
        (20, 23, 25, 45),  // front ward corridor
        (10, 44, 35, 60),  // vestibule
    ];
    let mut interior = vec![false; w * h];
    for &(x0, y0, x1, y1) in boxes {
        for y in y0..y1.min(h) {
            for x in x0..x1.min(w) {
                interior[y * w + x] = true;
            }
        }
    }
    // Default: non-interior Floor → Garden (background)
    for y in 0..h {
        for x in 0..w {
            if !interior[y * w + x] && map.get(x, y) == Tile::Floor {
                map.set(x, y, Tile::Garden);
            }
        }
    }
    // Sidewalk + Road ring wrapping hospital bounding
    // Bounding of exterior wall footprint: x=2..45, y=0..59 (approx)
    // South ring:
    for x in 0..w {
        if map.get(x, 60) == Tile::Garden { map.set(x, 60, Tile::Sidewalk); }
        if map.get(x, 61) == Tile::Garden { map.set(x, 61, Tile::Road); }
        if map.get(x, 62) == Tile::Garden { map.set(x, 62, Tile::Road); }
        if map.get(x, 63) == Tile::Garden { map.set(x, 63, Tile::Sidewalk); }
    }
    // East ring:
    for y in 0..h {
        if map.get(46, y) == Tile::Garden { map.set(46, y, Tile::Sidewalk); }
        if map.get(47, y) == Tile::Garden { map.set(47, y, Tile::Road); }
        if map.get(48, y) == Tile::Garden { map.set(48, y, Tile::Road); }
        if map.get(49, y) == Tile::Garden { map.set(49, y, Tile::Sidewalk); }
    }
    // Perimeter walls
    for x in 0..w {
        map.set(x, 0, Tile::Wall);
        map.set(x, h - 1, Tile::Wall);
    }
    for y in 0..h {
        map.set(0, y, Tile::Wall);
        map.set(w - 1, y, Tile::Wall);
    }
    // Convert hospital outer Wall tiles (with outdoor neighbor) to tall HospitalOuterWall
    let mut outer: Vec<(usize, usize)> = Vec::new();
    for y in 0..h {
        for x in 0..w {
            if !matches!(map.get(x, y), Tile::Wall) { continue; }
            // Skip perimeter of map (edge walls) — those are not hospital
            if x == 0 || y == 0 || x == w - 1 || y == h - 1 { continue; }
            let mut has_outdoor = false;
            for (dx, dy) in [(-1i32, 0), (1, 0), (0, -1), (0, 1)] {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx < 0 || ny < 0 || (nx as usize) >= w || (ny as usize) >= h { continue; }
                if map.get(nx as usize, ny as usize).is_outdoor() {
                    has_outdoor = true; break;
                }
            }
            if has_outdoor { outer.push((x, y)); }
        }
    }
    for (x, y) in outer {
        map.set(x, y, Tile::HospitalOuterWall);
    }
}

fn stamp_parking(map: &mut Map) {
    let w = map.width;
    let h = map.height;
    // South parking lot: 2 rows deep, spanning most of the frontage below the south sidewalk
    let south_y = 65;
    for dy in 0..2 {
        let y = south_y + dy;
        if y >= h { continue; }
        for x in 6..(w - 6) {
            if map.get(x, y) == Tile::Garden {
                map.set(x, y, Tile::Parking);
            }
        }
    }
    // East parking column next to the east sidewalk (x=49). Two tiles wide.
    let east_x = 51;
    for x in east_x..(east_x + 2).min(w) {
        for y in 6..30 {
            if map.get(x, y) == Tile::Garden {
                map.set(x, y, Tile::Parking);
            }
        }
    }
}

fn stamp_vestibule(map: &mut Map, x0: usize, y0: usize) {
    let w = 25;
    let h = 16;
    for x in 0..w {
        map.set(x0 + x, y0, Tile::Wall);
        map.set(x0 + x, y0 + h - 1, Tile::Wall);
    }
    for y in 0..h {
        map.set(x0, y0 + y, Tile::Wall);
        map.set(x0 + w - 1, y0 + y, Tile::Wall);
    }
    // Interior columns (tall)
    for &cx in &[4usize, 10, 14, 20] {
        for &cy in &[4usize, 11] {
            map.set(x0 + cx, y0 + cy, Tile::Column);
        }
    }
}
