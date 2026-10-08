//! The hospital: its floor plan ([`hospital.txt`](hospital.txt), one character per metre)
//! and everything in it: furniture, fixtures, lights, signs, what's in the cupboards.
//!
//! Coordinates are metres in the floor plan, x east, y south. Furniture is placed by its
//! centre and the way its front faces.

use crate::engine::boxes::{BoxKind, Facing};
use crate::engine::decals::{self, Decals, Fixture as Wall, SignStyle};
use crate::engine::lighting::{LightKind, LightSource};
use crate::engine::map::{Map, Tile};
use crate::engine::textures::{DoorStyle, hash, rgb};
use crate::equippables::{Backpack, Equippable};
use crate::game::items::{Item, ItemKind};
use crate::game::world::DroppedItem;
use crate::objects::{
    Bed, Bench, Clothing, ClothingKind, Container, ContainerKind, Door, Fixture, FixtureKind, Fluorescent,
    FluorescentAxis, FluorescentState, Npc, Outlet, Prop, PropKind, Sofa, VendingKind, VendingMachine,
};

use Facing::{East as E, North as N, South as S, West as W};

const PLAN: &str = include_str!("hospital.txt");

/// Where you wake up (lying in bed in ICU 104) and where you stand when you get up.
pub const PLAYER_BED: (f64, f64) = (34.5, 8.3);
pub const PLAYER_START: (f64, f64) = (33.0, 10.3);

pub struct Level {
    pub map: Map,
    pub doors: Vec<Door>,
    pub fluorescents: Vec<Fluorescent>,
    pub equippables: Vec<Equippable>,
    pub clothing: Vec<Clothing>,
    pub beds: Vec<Bed>,
    pub benches: Vec<Bench>,
    pub sofas: Vec<Sofa>,
    pub outlets: Vec<Outlet>,
    pub fixtures: Vec<Fixture>,
    pub vending: Vec<VendingMachine>,
    pub npcs: Vec<Npc>,
    pub props: Vec<Prop>,
    pub containers: Vec<Container>,
    /// Items that are always in a given container (by index), whatever the dice say.
    pub guaranteed: Vec<(usize, Item)>,
    pub decals: Decals,
    /// Loose items lying around at the start.
    pub items: Vec<DroppedItem>,
    /// Lights besides the ceiling tubes: windows, street lamps, exit signs, vending glow.
    pub extra_lights: Vec<LightSource>,
}

struct Build {
    lvl: Level,
}

impl Build {
    fn prop(&mut self, x: f64, y: f64, kind: PropKind, f: Facing) {
        self.lvl.props.push(Prop::new(x, y, kind, f));
    }

    fn raised(&mut self, x: f64, y: f64, kind: PropKind, e: f64) {
        self.lvl.props.push(Prop::new(x, y, kind, S).raised(e));
    }

    /// Box furniture pushed against a wall: `wall_x`/`wall_y` is the wall's face, the box
    /// sits in front of it facing away.
    fn against(&self, kind: BoxKind, along: f64, wall: f64, f: Facing) -> (f64, f64) {
        let (_, d, _) = kind.size();
        match f {
            S => (along, wall + d / 2.0),
            N => (along, wall - d / 2.0),
            E => (wall + d / 2.0, along),
            W => (wall - d / 2.0, along),
        }
    }

    fn container(&mut self, x: f64, y: f64, kind: ContainerKind, f: Facing) -> usize {
        self.lvl.containers.push(Container::new(x, y, kind, f));
        self.lvl.containers.len() - 1
    }

    fn container_on_wall(&mut self, kind: ContainerKind, along: f64, wall: f64, f: Facing) -> usize {
        let (x, y) = self.against(kind.box_kind().expect("box container"), along, wall, f);
        self.container(x, y, kind, f)
    }

    fn sign(&mut self, x: f64, y: f64, text: &str, style: SignStyle, v0: f64) {
        let (img, w, h) = decals::sign(text, style);
        let id = self.lvl.decals.add_image(img);
        let placed = self.lvl.decals.place(&self.lvl.map, x, y, w, v0, h, id, false);
        debug_assert!(placed, "no wall for sign {text:?} at ({x}, {y})");
        if matches!(style, SignStyle::Exit) {
            self.lvl.extra_lights.push(LightSource { x, y, kind: LightKind::Glow, color: [0.3, 1.0, 0.45], intensity: 0.35, radius: 2.5 });
        }
    }

    fn fixture(&mut self, x: f64, y: f64, f: Wall, v0: f64) {
        let (img, w, h) = decals::fixture(f);
        let id = self.lvl.decals.add_image(img);
        let placed = self.lvl.decals.place(&self.lvl.map, x, y, w, v0, h, id, f == Wall::Clock);
        debug_assert!(placed, "no wall for {f:?} at ({x}, {y})");
    }

    fn outlet(&mut self, x: f64, y: f64) {
        self.fixture(x, y, Wall::Outlet, 0.35);
        self.lvl.outlets.push(Outlet::new(x, y));
    }

    fn clothes(&mut self, x: f64, y: f64, kind: ClothingKind) {
        self.lvl.clothing.push(Clothing::new(x, y, kind));
    }

    fn bed(&mut self, x: f64, y: f64, f: Facing, headboard: bool) {
        self.lvl.beds.push(Bed::new(x, y, f));
        if headboard {
            let (fx, fy) = f.vector();
            self.prop(x - fx * 1.08, y - fy * 1.08, PropKind::Headboard, f);
        }
    }

    fn bench(&mut self, along: f64, wall: f64, f: Facing, pew: bool) {
        let seat = if pew { BoxKind::PewSeat } else { BoxKind::BenchSeat };
        let (x, y) = self.against(seat, along, wall, f);
        let (fx, fy) = f.vector();
        // leave room for the backrest between seat and wall
        let (x, y) = (x + fx * 0.07, y + fy * 0.07);
        self.lvl.benches.push(if pew { Bench::pew(x, y, f) } else { Bench::new(x, y, f) });
    }
}

pub fn hospital() -> Level {
    let map = Map::parse(PLAN);
    let mut b = Build {
        lvl: Level {
            doors: doors(&map),
            fluorescents: ceiling_lights(&map),
            map,
            equippables: Vec::new(),
            clothing: Vec::new(),
            beds: Vec::new(),
            benches: Vec::new(),
            sofas: Vec::new(),
            outlets: Vec::new(),
            fixtures: Vec::new(),
            vending: Vec::new(),
            npcs: Vec::new(),
            props: Vec::new(),
            containers: Vec::new(),
            guaranteed: Vec::new(),
            decals: Decals::default(),
            items: Vec::new(),
            extra_lights: Vec::new(),
        },
    };
    window_light(&mut b);
    icu_ward(&mut b);
    nurse_station(&mut b);
    west_wing(&mut b);
    middle_block(&mut b);
    surgery(&mut b);
    south_wing(&mut b);
    lobby(&mut b);
    outside(&mut b);
    b.lvl
}

// ---------------------------------------------------------------- doors and lights

fn doors(map: &Map) -> Vec<Door> {
    let mut doors = Vec::new();
    for y in 0..map.height {
        for x in 0..map.width {
            let style = match map.get(x, y) {
                Tile::Door { .. } => DoorStyle::Wood,
                Tile::BathroomDoor { .. } => DoorStyle::Restroom,
                Tile::OperatingDoor { .. } => DoorStyle::Secure,
                Tile::MainDoor { .. } => DoorStyle::Glass,
                _ => continue,
            };
            let mut d = Door::new(x, y, style);
            match (x, y) {
                (42, 24) | (49, 32) | (17, 32) => d.lock = Some(ItemKind::TriageKeycard),
                (57, 48) => d.lock = Some(ItemKind::ArchiveClearance),
                (59, 15) | (6, 72) => d.style = DoorStyle::Service,
                _ => {}
            }
            if matches!(map.get(x, y), Tile::MainDoor { .. }) {
                d.auto_close_ms = Some(5_000);
            }
            doors.push(d);
        }
    }
    doors
}

/// Ceiling tubes on a grid in every room and corridor, about 3 m apart. A few flicker,
/// a few are dead: nobody has changed a tube since The Fanfare.
fn ceiling_lights(map: &Map) -> Vec<Fluorescent> {
    let (w, h) = (map.width, map.height);
    let mut region = vec![usize::MAX; w * h];
    let mut lights = Vec::new();
    let mut next = 0;
    for start in 0..w * h {
        if region[start] != usize::MAX || map.get(start % w, start / w) != Tile::Floor {
            continue;
        }
        // flood one room (doors and walls bound it)
        let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0, 0);
        let mut stack = vec![start];
        region[start] = next;
        while let Some(i) = stack.pop() {
            let (x, y) = (i % w, i / w);
            (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
            for (dx, dy) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                if nx < 0 || ny < 0 || nx as usize >= w || ny as usize >= h {
                    continue;
                }
                let j = ny as usize * w + nx as usize;
                if region[j] == usize::MAX && map.get(nx as usize, ny as usize) == Tile::Floor {
                    region[j] = next;
                    stack.push(j);
                }
            }
        }
        next += 1;
        let (rw, rh) = ((x1 - x0 + 1) as f64, (y1 - y0 + 1) as f64);
        let (nx, ny) = ((rw / 3.4).round().max(1.0), (rh / 3.4).round().max(1.0));
        let axis = if rw >= rh { FluorescentAxis::Horizontal } else { FluorescentAxis::Vertical };
        for j in 0..ny as usize {
            for i in 0..nx as usize {
                let lx = x0 as f64 + (i as f64 + 0.5) * rw / nx;
                let ly = y0 as f64 + (j as f64 + 0.5) * rh / ny;
                if map.at(lx, ly) != Tile::Floor || region[ly as usize * w + lx as usize] != next - 1 {
                    continue;
                }
                let hsh = hash(lx as i32, ly as i32, 907);
                // the lights in your own room (ICU 104) all work
                let mine = (31.0..38.0).contains(&lx) && (7.0..15.0).contains(&ly);
                let state = if mine || hsh < 0.79 {
                    FluorescentState::Steady
                } else if hsh < 0.9 {
                    FluorescentState::Flicker { period_ms: 600 + (hsh * 9_000.0) as u64 % 1_400, duty: 0.55 + hsh * 0.3 }
                } else {
                    FluorescentState::Dead
                };
                lights.push(Fluorescent::new(lx, ly, 0.6, axis).with_state(state));
            }
        }
    }
    lights
}

/// Daylight comes in through every window.
fn window_light(b: &mut Build) {
    let map = &b.lvl.map;
    for y in 0..map.height {
        for x in 0..map.width {
            if map.get(x, y) != Tile::WindowWall {
                continue;
            }
            for (dx, dy) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                if nx < 0 || ny < 0 || map.get(nx as usize, ny as usize) != Tile::Floor {
                    continue;
                }
                b.lvl.extra_lights.push(LightSource {
                    x: x as f64 + 0.5 + dx as f64 * 0.95,
                    y: y as f64 + 0.5 + dy as f64 * 0.95,
                    kind: LightKind::Window,
                    color: [1.0; 3],
                    intensity: 0.55,
                    radius: 6.0,
                });
            }
        }
    }
}

// ---------------------------------------------------------------- the ward: ICU 101–106

fn icu_ward(b: &mut Build) {
    for i in 0..6 {
        let x0 = 7.0 + 8.0 * i as f64;
        let n = 101 + i;
        let mine = n == 104;
        // bed with its head to the north wall, the window above it
        b.bed(x0 + 3.5, 8.3, S, true);
        b.prop(x0 + 2.55, 7.45, PropKind::IvStand, S);
        b.prop(x0 + 2.45, 8.45, PropKind::Monitor, S);
        b.container(x0 + 4.45, 7.3, ContainerKind::BedsideTable, S);
        // a visitor's sofa against the east wall
        let (sx, sy) = b.against(BoxKind::SofaSeat, 9.6, x0 + 7.0 - 0.2, W);
        b.lvl.sofas.push(Sofa::new(sx, sy, W));
        b.prop(x0 + 6.6, 13.6, PropKind::CoatRack, S);
        b.outlet(x0 + 2.2, 7.02);
        b.outlet(x0 + 6.98, 11.5);
        b.fixture(x0 + 6.98, 8.6, Wall::Whiteboard, 1.0);
        b.fixture(x0 + 0.02, 9.0, Wall::Clock, 2.0);
        b.fixture(x0 + 5.6, 14.98, Wall::Switch, 1.1);
        // the en-suite: toilet, sink with a mirror cabinet, shower
        b.lvl.fixtures.push(Fixture::new(x0 + 0.45, 12.45, FixtureKind::Toilet).with_paper(6 + i as u32));
        b.lvl.fixtures.push(Fixture::new(x0 + 1.6, 12.25, FixtureKind::Sink));
        b.lvl.fixtures.push(Fixture::new(x0 + 0.55, 14.45, FixtureKind::Shower));
        b.fixture(x0 + 1.6, 12.02, Wall::Mirror, 1.15);
        let cab = b.container(x0 + 2.45, 12.25, ContainerKind::MedCabinet, S);
        b.fixture(x0 + 2.45, 12.02, Wall::MedCabinet, 1.2);
        if mine {
            b.lvl.guaranteed.push((cab, Item::new_with_units(ItemKind::Ibuprofen, 6)));
        }
        // outside the door: the room's plaque and a sanitiser
        b.sign(x0 + 4.25, 16.02, &format!("ICU {n}"), SignStyle::Plaque, 1.45);
        b.fixture(x0 + 5.85, 16.02, Wall::Sanitizer, 1.05);
        // the other patients vanished: their gowns lie on the beds
        if !mine {
            b.lvl.clothing.push(Clothing::new(x0 + 3.5, 8.6, ClothingKind::HospitalGown).with_elevation(0.62));
        }
    }
    // your backpack, on the sofa in your room
    b.lvl.equippables.push(Equippable::Backpack(Backpack::player_starter(37.4, 9.6)));

    // the north corridor
    b.bench(25.5, 19.0, N, false);
    b.bench(46.0, 19.0, N, false);
    b.prop(43.0, 18.45, PropKind::Wheelchair, S);
    b.prop(56.2, 18.55, PropKind::Gurney, E);
    b.prop(30.4, 16.3, PropKind::TrashBin, S);
    b.prop(63.5, 16.4, PropKind::Plant, S);
    b.clothes(16.6, 17.2, ClothingKind::ScrubBundle);
    b.clothes(49.5, 16.6, ClothingKind::ScrubBundle);
    b.fixture(33.0, 16.02, Wall::Extinguisher, 0.75);
    b.fixture(52.5, 18.98, Wall::Aed, 1.1);
    b.sign(40.0, 16.02, "RECEPTION _", SignStyle::Wayfinding, 2.15);
    b.sign(23.0, 16.02, "< ICU 101-103", SignStyle::Wayfinding, 2.15);
    b.sign(56.0, 16.02, "ICU 104-106 >", SignStyle::Wayfinding, 2.15);
    b.fixture(7.02, 16.5, Wall::QuietPoster, 1.2);
    for x in [16.5, 26.5, 34.5, 50.5, 62.5] {
        b.outlet(x, 18.98);
    }

    // the plant room
    let (gx, gy) = (60.0, 10.0);
    b.prop(gx, gy, PropKind::Generator, S);
    b.container_on_wall(ContainerKind::Shelf, 8.6, 55.0, E);
    let (wx, wy) = b.against(BoxKind::KitchenCounter, 12.0, 65.0, W);
    b.prop(wx, wy, PropKind::KitchenCounter, W);
    b.sign(58.0, 16.02, "PLANT ROOM", SignStyle::Plaque, 1.45);
    b.sign(60.0, 7.02, "DANGER HIGH VOLTAGE", SignStyle::Warning, 1.6);
}

fn nurse_station(b: &mut Build) {
    // the counter faces the corridor through the open front, staff side behind
    b.prop(11.0, 20.55, PropKind::NurseCounter, N);
    b.prop(10.6, 21.6, PropKind::Chair(rgb(60, 70, 90)), N);
    b.lvl.npcs.push(Npc::julian(13.7, 21.4));
    let desk = b.container_on_wall(ContainerKind::Desk, 23.5, 7.0, E);
    b.lvl.guaranteed.push((desk, Item::new(ItemKind::ChocolateBar)));
    b.container_on_wall(ContainerKind::FileCabinet, 26.3, 7.0, E);
    b.container_on_wall(ContainerKind::CrashCart, 26.2, 17.0, W);
    b.prop(7.5, 20.5, PropKind::Plant, S);
    b.fixture(16.98, 23.2, Wall::Whiteboard, 1.0);
    b.fixture(16.98, 23.2, Wall::Clock, 2.1);
    b.fixture(16.98, 21.0, Wall::Bulletin, 1.1);
    b.sign(9.0, 16.02, "NURSE STATION _", SignStyle::Wayfinding, 2.15);
    b.outlet(7.02, 22.4);
    b.outlet(16.98, 25.5);
    b.clothes(9.5, 22.5, ClothingKind::ScrubBundle);
}

fn west_wing(b: &mut Build) {
    // medication room (locked): glass cabinets, shelves, a drugs fridge
    for x in [8.4, 9.4, 10.4] {
        b.container_on_wall(ContainerKind::GlassCabinet, x, 29.0, S);
    }
    b.container_on_wall(ContainerKind::Fridge, 15.5, 29.0, S);
    b.container_on_wall(ContainerKind::Shelf, 32.5, 7.0, E);
    b.container_on_wall(ContainerKind::Shelf, 33.8, 7.0, E);
    let (cx, cy) = b.against(BoxKind::KitchenCounter, 11.5, 36.0, N);
    b.prop(cx, cy, PropKind::KitchenCounter, N);
    b.sign(18.02, 30.8, "MEDICATION", SignStyle::Plaque, 1.45);
    b.sign(18.02, 33.3, "STAFF ONLY", SignStyle::Warning, 1.45);

    // doctor's office: the archive clearance is in the desk
    let desk = b.container(12.0, 41.5, ContainerKind::Desk, E);
    b.lvl.guaranteed.push((desk, Item::new(ItemKind::ArchiveClearance)));
    b.prop(11.3, 41.5, PropKind::Chair(rgb(40, 40, 44)), E);
    b.container_on_wall(ContainerKind::FileCabinet, 38.0, 7.0, E);
    b.container_on_wall(ContainerKind::FileCabinet, 38.6, 7.0, E);
    b.container_on_wall(ContainerKind::Shelf, 44.5, 7.0, E);
    let (sx, sy) = b.against(BoxKind::SofaSeat, 45.0, 17.0 - 0.2, W);
    b.lvl.sofas.push(Sofa::new(sx, sy, W));
    b.prop(16.5, 37.5, PropKind::Plant, S);
    b.fixture(16.98, 39.5, Wall::Painting, 1.3);
    b.fixture(7.02, 41.5, Wall::Bulletin, 1.1);
    b.sign(18.02, 40.8, "DR. OFFICE", SignStyle::Plaque, 1.45);
    b.outlet(16.98, 41.0);

    // the west corridor
    b.prop(18.3, 36.5, PropKind::TrashBin, S);
    b.clothes(19.2, 30.4, ClothingKind::ScrubBundle);
    b.fixture(18.02, 25.5, Wall::Extinguisher, 0.75);
}

fn middle_block(b: &mut Build) {
    // supply room: shelves along both long walls, spare equipment
    for x in [23.2, 24.6, 26.0, 27.4, 28.8] {
        b.container_on_wall(ContainerKind::Shelf, x, 20.0, S);
        b.container_on_wall(ContainerKind::Shelf, x, 27.0, N);
    }
    b.prop(33.5, 23.0, PropKind::Gurney, N);
    b.prop(35.6, 20.8, PropKind::Wheelchair, S);
    b.prop(36.4, 21.2, PropKind::Wheelchair, S);
    b.prop(31.2, 20.6, PropKind::IvStand, S);
    b.prop(31.6, 21.0, PropKind::IvStand, S);
    b.sign(20.98, 21.8, "SUPPLY", SignStyle::Plaque, 1.45);

    // staff break room: kitchenette, table, sofa, lockers, a coat rack with a jacket
    b.container_on_wall(ContainerKind::Fridge, 22.45, 28.0, S);
    b.container_on_wall(ContainerKind::KitchenCupboard, 23.9, 28.0, S);
    b.container_on_wall(ContainerKind::KitchenCupboard, 25.9, 28.0, S);
    b.prop(29.5, 32.5, PropKind::WoodTable, S);
    for (x, y) in [(28.7, 31.7), (30.3, 31.7), (28.7, 33.3), (30.3, 33.3)] {
        b.prop(x, y, PropKind::Chair(rgb(170, 70, 40)), S);
    }
    let (sx, sy) = b.against(BoxKind::SofaSeat, 35.0, 38.0 - 0.2, N);
    b.lvl.sofas.push(Sofa::new(sx, sy, N));
    b.container_on_wall(ContainerKind::Locker, 29.0, 38.0, W);
    b.container_on_wall(ContainerKind::Locker, 30.0, 38.0, W);
    b.prop(23.0, 36.6, PropKind::CoatRack, S);
    b.lvl.clothing.push(Clothing::new(23.0, 36.6, ClothingKind::Hoodie).with_elevation(0.8).pickable());
    b.prop(36.6, 34.5, PropKind::TrashBin, S);
    b.fixture(22.02, 35.0, Wall::Bulletin, 1.1);
    b.fixture(29.5, 37.98, Wall::Clock, 2.0);
    b.fixture(33.0, 28.02, Wall::QuietPoster, 1.3);
    b.sign(20.98, 30.8, "STAFF ROOM", SignStyle::Plaque, 1.45);
    b.outlet(27.5, 28.02);
    b.outlet(22.02, 36.0);

    // on-call room: two beds, lockers, the trousers left on the bench
    b.bed(26.0, 40.15, S, false);
    b.bed(30.0, 40.15, S, false);
    b.container_on_wall(ContainerKind::Locker, 41.0, 38.0, W);
    b.container_on_wall(ContainerKind::Locker, 42.0, 38.0, W);
    b.bench(33.5, 48.0, N, false);
    b.lvl.clothing.push(Clothing::new(33.5, 47.62, ClothingKind::Pants).with_elevation(0.46).pickable());
    b.container_on_wall(ContainerKind::Desk, 24.0, 48.0, N);
    b.prop(36.5, 39.5, PropKind::Plant, S);
    // Selenia's camp: three weeks of living here
    b.lvl.npcs.push(Npc::selenia(31.0, 44.2));
    b.prop(34.6, 45.0, PropKind::Chair(rgb(170, 70, 40)), S);
    b.prop(35.6, 46.6, PropKind::TrashBin, S);
    b.prop(28.0, 46.4, PropKind::CafeTable, S);
    b.sign(20.98, 41.8, "ON-CALL", SignStyle::Plaque, 1.45);
    b.outlet(22.02, 40.5);

    // the spine corridor down to the lobby
    b.bench(26.0, 39.0, E, false);
    b.bench(45.0, 39.0, E, false);
    b.prop(41.6, 30.5, PropKind::Wheelchair, S);
    b.clothes(40.3, 35.0, ClothingKind::ScrubBundle);
    b.sign(41.98, 22.6, "SURGERY", SignStyle::Plaque, 1.45);
    b.sign(41.98, 25.6, "AUTHORISED ONLY", SignStyle::Warning, 1.45);
    b.fixture(39.02, 38.0, Wall::Extinguisher, 0.75);
    b.fixture(39.02, 20.5, Wall::Aed, 1.1);
}

fn surgery(b: &mut Build) {
    // recovery: four beds behind curtains
    for (k, x) in [45.0, 48.6, 52.2, 55.8].into_iter().enumerate() {
        b.bed(x, 21.15, S, true);
        b.prop(x - 0.9, 20.5, PropKind::Monitor, S);
        b.prop(x + 0.85, 20.45, PropKind::IvStand, S);
        if k < 3 {
            b.raised(x + 1.8, 21.4, PropKind::Curtain, 0.3);
        }
        if k % 2 == 1 {
            b.lvl.clothing.push(Clothing::new(x, 21.4, ClothingKind::HospitalGown).with_elevation(0.62));
        }
        b.outlet(x - 0.6, 20.02);
    }
    b.container_on_wall(ContainerKind::CrashCart, 24.5, 65.0, W);
    b.container_on_wall(ContainerKind::GlassCabinet, 21.2, 65.0, W);
    b.sign(44.0, 27.98, "SCRUB", SignStyle::Plaque, 1.45);

    // scrub room
    for y in [30.4, 31.4, 33.6] {
        b.lvl.fixtures.push(Fixture::new(43.25, y, FixtureKind::Sink));
        b.fixture(43.02, y, Wall::Mirror, 1.15);
    }
    b.prop(47.2, 34.4, PropKind::GownRack, S);
    b.container_on_wall(ContainerKind::Shelf, 30.2, 49.0, W);
    b.sign(48.98, 30.6, "OR 1", SignStyle::Plaque, 1.45);
    b.sign(44.0, 35.98, "STERILE", SignStyle::Plaque, 1.45);

    // the operating room
    b.lvl.beds.push(Bed::operating(57.0, 34.5, S));
    b.raised(57.0, 34.5, PropKind::SurgicalLight, 1.9);
    b.prop(55.5, 33.6, PropKind::InstrumentTable, S);
    b.prop(58.5, 35.6, PropKind::InstrumentTable, S);
    b.prop(58.9, 33.3, PropKind::Monitor, S);
    b.prop(55.4, 36.0, PropKind::IvStand, S);
    for x in [51.6, 52.6, 62.5] {
        b.container_on_wall(ContainerKind::GlassCabinet, x, 29.0, S);
    }
    b.prop(62.5, 38.4, PropKind::Gurney, N);
    b.clothes(55.8, 34.6, ClothingKind::ScrubBundle);
    b.fixture(50.02, 31.0, Wall::Clock, 2.1);
    b.outlet(50.02, 35.0);

    // sterile storage
    for y in [38.2, 39.6, 41.0, 42.4, 43.8, 45.2] {
        b.container_on_wall(ContainerKind::Shelf, y, 43.0, E);
        b.container_on_wall(ContainerKind::Shelf, y, 49.0, W);
    }

    // the clinical archive (needs clearance): filing cabinets, and a tape in the desk
    for k in 0..8 {
        b.container_on_wall(ContainerKind::FileCabinet, 51.0 + k as f64 * 0.55, 42.0, S);
    }
    b.container_on_wall(ContainerKind::Shelf, 61.0, 42.0, S);
    b.container_on_wall(ContainerKind::Shelf, 62.4, 42.0, S);
    let desk = b.container(57.0, 45.4, ContainerKind::Desk, N);
    b.lvl.guaranteed.push((desk, Item::new(ItemKind::CassetteTape)));
    b.prop(57.0, 46.2, PropKind::Chair(rgb(40, 40, 44)), N);
    b.sign(55.8, 49.02, "CLINICAL ARCHIVE", SignStyle::Plaque, 1.45);
}

fn south_wing(b: &mut Build) {
    // the cross corridor
    b.prop(60.0, 51.4, PropKind::Wheelchair, S);
    b.clothes(25.0, 50.4, ClothingKind::ScrubBundle);
    b.clothes(50.0, 49.6, ClothingKind::HospitalGown);
    b.sign(64.98, 49.0, "EXIT >", SignStyle::Exit, 2.1);
    b.sign(21.0, 51.98, "CAFETERIA _", SignStyle::Wayfinding, 2.15);
    b.sign(10.6, 51.98, "WC", SignStyle::Plaque, 1.45);
    b.fixture(30.0, 49.02, Wall::Extinguisher, 0.75);
    b.fixture(14.0, 49.02, Wall::HandWashPoster, 1.2);
    for x in [12.0, 33.0, 47.0, 60.0] {
        b.outlet(x, 49.02);
    }

    // the public restroom
    b.lvl.fixtures.push(Fixture::new(12.7, 54.0, FixtureKind::Sink));
    b.lvl.fixtures.push(Fixture::new(12.7, 55.2, FixtureKind::Sink));
    b.fixture(12.98, 54.0, Wall::Mirror, 1.15);
    b.fixture(12.98, 55.2, Wall::Mirror, 1.15);
    b.lvl.fixtures.push(Fixture::new(7.45, 57.6, FixtureKind::Toilet).with_paper(4));
    b.lvl.fixtures.push(Fixture::new(9.6, 58.75, FixtureKind::Urinal));
    b.lvl.fixtures.push(Fixture::new(10.6, 58.75, FixtureKind::Urinal));
    b.fixture(12.98, 56.5, Wall::HandWashPoster, 1.2);

    // cafeteria: serving line, tables, chairs
    let (sx, sy) = b.against(BoxKind::ServingCounter, 56.0, 14.0, E);
    b.prop(sx, sy, PropKind::ServingCounter, E);
    b.prop(sx, sy + 2.0, PropKind::ServingCounter, E);
    for (j, ty) in [55.5, 58.5, 61.5, 64.5].into_iter().enumerate() {
        for (i, tx) in [18.5, 22.0, 25.5, 29.0].into_iter().enumerate() {
            b.prop(tx, ty, PropKind::CafeTable, S);
            for dy in [-0.7, 0.7] {
                b.prop(tx - 0.3, ty + dy, PropKind::Chair(rgb(220, 120, 40)), S);
                if (i + j) % 2 == 0 {
                    b.prop(tx + 0.3, ty + dy, PropKind::Chair(rgb(220, 120, 40)), S);
                }
            }
        }
    }
    b.clothes(23.5, 57.2, ClothingKind::HospitalGown);
    b.clothes(27.0, 62.9, ClothingKind::ScrubBundle);
    b.prop(31.5, 53.4, PropKind::TrashBin, S);
    b.prop(14.5, 66.5, PropKind::Plant, S);
    b.fixture(19.0, 53.02, Wall::Bulletin, 1.2);
    b.fixture(27.5, 53.02, Wall::Clock, 2.1);
    b.fixture(18.0, 66.98, Wall::HandWashPoster, 1.2);

    // kitchen
    b.container_on_wall(ContainerKind::KitchenCupboard, 61.0, 13.0, W);
    b.container_on_wall(ContainerKind::KitchenCupboard, 66.0, 13.0, W);
    let (kx, ky) = b.against(BoxKind::Stove, 68.4, 13.0, W);
    b.prop(kx, ky, PropKind::Stove, W);
    b.prop(kx, ky + 0.8, PropKind::Stove, W);
    for y in [60.8, 61.6, 62.4] {
        b.container_on_wall(ContainerKind::Fridge, y, 7.0, E);
    }
    b.container_on_wall(ContainerKind::Shelf, 66.0, 7.0, E);
    b.container_on_wall(ContainerKind::Shelf, 67.4, 7.0, E);
    b.prop(9.8, 64.5, PropKind::KitchenCounter, E);
    b.clothes(10.5, 70.5, ClothingKind::ScrubBundle);
    b.prop(11.6, 74.5, PropKind::TrashBin, S);
    b.sign(14.02, 61.8, "KITCHEN", SignStyle::Plaque, 1.45);
    b.sign(7.02, 70.8, "EXIT", SignStyle::Exit, 2.15);

    // chapel: an altar, a cross, pews facing it
    b.fixture(14.02, 71.5, Wall::Cross, 1.3);
    b.prop(14.9, 71.5, PropKind::WoodTable, E);
    for x in [19.0, 22.0, 25.0, 28.0] {
        for y in [70.0, 73.6] {
            b.lvl.benches.push(Bench::pew(x, y, W));
        }
    }
    b.prop(14.6, 68.6, PropKind::Plant, S);
    b.prop(14.6, 74.4, PropKind::Plant, S);
    b.fixture(23.0, 68.02, Wall::Painting, 1.3);
    b.sign(33.02, 69.8, "CHAPEL", SignStyle::Plaque, 1.45);
}

fn lobby(b: &mut Build) {
    // reception, with a desk behind it
    b.prop(44.5, 60.5, PropKind::Reception, S);
    b.container(44.5, 58.9, ContainerKind::Desk, S);
    b.prop(43.4, 59.7, PropKind::Chair(rgb(40, 40, 44)), S);
    // waiting rows facing reception
    for row in [64.8, 66.4] {
        for k in 0..8 {
            b.prop(41.4 + k as f64 * 0.6, row, PropKind::Chair(rgb(60, 110, 160)), N);
        }
    }
    // vending machines along the west wall
    let (vx, vy) = b.against(BoxKind::VendingSnacks, 61.6, 33.0, E);
    b.lvl.vending.push(VendingMachine::new(vx, vy, VendingKind::Snacks, 3, E));
    b.lvl.vending.push(VendingMachine::new(vx, vy + 1.05, VendingKind::Drinks, 2, E));
    for (x, y) in [(vx, vy), (vx, vy + 1.05)] {
        b.lvl.extra_lights.push(LightSource { x: x + 0.7, y, kind: LightKind::Glow, color: [0.7, 0.85, 1.0], intensity: 0.45, radius: 3.0 });
    }
    for (x, y) in [(37.4, 58.0), (52.6, 58.0), (37.4, 68.0), (52.6, 68.0)] {
        b.prop(x, y, PropKind::Plant, S);
    }
    b.prop(36.0, 74.8, PropKind::TrashBin, S);
    b.prop(52.0, 74.8, PropKind::TrashBin, S);
    b.prop(41.0, 74.5, PropKind::Wheelchair, S);
    b.clothes(36.0, 66.0, ClothingKind::HospitalGown);
    b.clothes(48.5, 70.0, ClothingKind::ScrubBundle);
    b.clothes(45.2, 73.2, ClothingKind::HospitalGown);
    // the elevators are out
    b.fixture(47.0, 53.02, Wall::Elevator, 0.0);
    b.fixture(49.0, 53.02, Wall::Elevator, 0.0);
    b.fixture(44.5, 53.02, Wall::Clock, 2.2);
    b.fixture(56.98, 64.0, Wall::Painting, 1.3);
    b.fixture(33.02, 66.0, Wall::Bulletin, 1.2);
    b.sign(43.5, 75.98, "EXIT", SignStyle::Exit, 2.25);
    b.sign(44.5, 75.98, "EXIT", SignStyle::Exit, 2.25);
    b.sign(56.98, 56.8, "GIFT SHOP", SignStyle::Plaque, 1.45);
    b.sign(56.98, 68.8, "TRIAGE", SignStyle::Plaque, 1.45);
    b.sign(44.5, 53.02, "RECEPTION", SignStyle::Wayfinding, 1.75);
    for (x, y) in [(33.02, 64.0), (56.98, 60.0), (40.0, 75.98)] {
        b.outlet(x, y);
    }

    // gift shop / pharmacy
    for y in [54.4, 56.2, 59.8, 61.6] {
        b.container_on_wall(ContainerKind::StoreShelf, y, 65.0, W);
    }
    b.container_on_wall(ContainerKind::Fridge, 58.45, 53.0, S);
    b.container(60.6, 57.5, ContainerKind::StoreShelf, W);
    let (tx, ty) = b.against(BoxKind::KitchenCounter, 63.0, 58.0, E);
    b.prop(tx, ty, PropKind::KitchenCounter, E);

    // triage: an exam bed, a curtain, the keycard in the desk drawer
    b.bed(63.85, 68.4, W, false);
    b.raised(62.0, 70.2, PropKind::Curtain, 0.3);
    let desk = b.container_on_wall(ContainerKind::Desk, 60.6, 76.0, N);
    b.lvl.guaranteed.push((desk, Item::new(ItemKind::TriageKeycard)));
    b.prop(60.6, 74.8, PropKind::Chair(rgb(40, 40, 44)), N);
    b.container_on_wall(ContainerKind::GlassCabinet, 72.6, 65.0, W);
    b.prop(64.3, 70.9, PropKind::IvStand, S);
    b.prop(58.6, 67.0, PropKind::Monitor, S);
    b.outlet(58.02, 72.0);
}

fn outside(b: &mut Build) {
    // HOSPITAL in red letters over the entrance, EMERGENCY over the ambulance doors
    for (text, x, y, h, color) in [("HOSPITAL", 44.0, 77.02, 0.45, rgb(196, 30, 30)), ("EMERGENCY", 66.02, 50.0, 0.32, rgb(196, 30, 30))] {
        let (img, aspect) = decals::letters(text, color);
        let id = b.lvl.decals.add_image(img);
        b.lvl.decals.place(&b.lvl.map, x, y, h * aspect, 2.2, h, id, false);
    }
    b.sign(5.98, 72.0, "DELIVERIES", SignStyle::Plaque, 2.25);

    // street lamps along the streets and over the car park
    let mut lamps = Vec::new();
    for x in [10.0, 22.0, 34.0, 50.0, 62.0] {
        lamps.push((x, 91.2));
    }
    for y in [8.0, 24.0, 40.0, 58.0, 74.0] {
        lamps.push((72.5, y));
    }
    lamps.extend([(34.0, 79.5), (54.0, 79.5), (20.0, 85.0), (58.0, 85.0)]);
    for (x, y) in lamps {
        b.prop(x, y, PropKind::StreetLamp, S);
        b.lvl.extra_lights.push(LightSource { x, y, kind: LightKind::Lamp, color: [1.0, 0.62, 0.28], intensity: 1.1, radius: 9.0 });
    }

    // the car park, full the morning everyone vanished
    let colors = [rgb(176, 36, 36), rgb(36, 80, 170), rgb(28, 28, 32), rgb(226, 226, 228), rgb(196, 176, 60), rgb(40, 100, 60), rgb(110, 60, 34), rgb(140, 146, 156)];
    let mut k = 0;
    for stall in 2..25 {
        let x = stall as f64 * 2.5 + 1.25;
        if (41.0..47.0).contains(&x) {
            continue; // the footpath
        }
        for (y, f) in [(83.0, N), (87.4, S)] {
            if hash(stall, y as i32, 919) < 0.62 {
                b.prop(x, y, PropKind::Car(colors[k % colors.len()]), f);
                k += 1;
            }
        }
    }
    // cars stopped dead in the street, drivers gone mid-journey
    b.prop(20.0, 93.0, PropKind::Car(colors[1]), E);
    b.prop(48.0, 94.7, PropKind::Car(colors[3]), W);
    b.prop(61.5, 93.0, PropKind::Car(colors[4]), E);
    b.prop(69.0, 30.0, PropKind::Car(colors[2]), N);
    b.prop(70.7, 64.0, PropKind::Car(colors[0]), S);
    b.prop(69.5, 50.0, PropKind::Ambulance, E);
    b.prop(3.6, 74.0, PropKind::Dumpster, E);

    b.prop(30.0, 96.6, PropKind::BusStop, N);
    b.prop(52.0, 91.4, PropKind::Mailbox, S);
    b.bench(36.0, 77.0, S, false);
    b.bench(52.0, 77.0, S, false);
    b.prop(39.0, 77.6, PropKind::TrashBin, S);
    b.prop(49.0, 77.6, PropKind::TrashBin, S);

    // trees, bushes, weeds
    let trees = [
        (10.0, 2.6), (19.0, 3.0), (28.0, 2.4), (37.0, 3.0), (46.0, 2.6), (55.0, 3.0), (63.0, 2.6),
        (2.8, 14.0), (3.0, 28.0), (2.6, 42.0), (3.0, 56.0), (75.5, 12.0), (76.0, 30.0), (75.5, 48.0), (76.0, 66.0), (75.6, 84.0),
        (10.0, 97.8), (40.0, 97.8), (60.0, 97.8),
    ];
    for (i, (x, y)) in trees.into_iter().enumerate() {
        b.prop(x, y, if i % 3 == 1 { PropKind::TreePine } else { PropKind::TreeOak }, S);
    }
    for x in [9.0, 13.0, 17.0, 21.0, 25.0, 60.0, 63.0] {
        b.prop(x, 78.2, PropKind::Bush, S);
    }
    for (x, y) in [(4.5, 9.0), (2.5, 35.0), (4.0, 48.0), (7.5, 79.5), (28.0, 79.6), (61.0, 79.4), (74.0, 20.0), (74.5, 40.0), (77.0, 58.0), (74.0, 76.0), (15.0, 4.5), (33.0, 4.0), (50.0, 4.4)] {
        b.prop(x, y, PropKind::Weed, S);
    }

    // people vanished out here too
    b.clothes(40.0, 79.5, ClothingKind::HospitalGown);
    b.clothes(12.0, 91.4, ClothingKind::ScrubBundle);
    b.clothes(69.8, 20.0, ClothingKind::ScrubBundle);
    b.clothes(33.0, 88.6, ClothingKind::HospitalGown);
    b.clothes(55.0, 96.4, ClothingKind::ScrubBundle);
}
