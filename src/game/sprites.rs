use crate::engine::map::Tile;
use crate::engine::sprites::{DoorLeafAxis, SignShapeKind, SpriteInput, SpriteShape};
use crate::game::world::{Weather, World};
use crate::objects;

pub fn collect_sprites(world: &World) -> Vec<SpriteInput> {
    let mut sprites: Vec<SpriteInput> = Vec::new();
    sprites.extend(world.benches.iter().map(|b| SpriteInput {
        x: b.x, y: b.y,
        shape: SpriteShape::Bench,
        scale: 1.0,
        elevation: 0.0,
    }));
    sprites.extend(world.sofas.iter().map(|s| SpriteInput {
        x: s.x, y: s.y,
        shape: SpriteShape::Sofa,
        scale: 1.0,
        elevation: 0.0,
    }));
    sprites.extend(world.outlets.iter().map(|o| SpriteInput {
        x: o.x, y: o.y,
        shape: SpriteShape::Outlet,
        scale: 1.0,
        elevation: 0.35,
    }));
    sprites.extend(world.vending.iter().map(|v| {
        let shape = match v.kind {
            objects::VendingKind::Snacks => SpriteShape::VendingSnacks,
            objects::VendingKind::Drinks => SpriteShape::VendingDrinks,
        };
        SpriteInput { x: v.x, y: v.y, shape, scale: 1.0, elevation: 0.0 }
    }));
    sprites.extend(world.receptions.iter().map(|r| SpriteInput {
        x: r.x, y: r.y,
        shape: SpriteShape::ReceptionCounter,
        scale: 1.0,
        elevation: 0.0,
    }));
    let win_shape = match world.weather {
        Weather::Clear => SpriteShape::WindowClear,
        Weather::Cloudy => SpriteShape::WindowCloudy,
    };
    sprites.extend(world.windows.iter().map(|wnd| SpriteInput {
        x: wnd.x, y: wnd.y,
        shape: win_shape.clone(),
        scale: 1.0,
        elevation: 0.30,
    }));
    sprites.extend(world.fixtures.iter().map(|f| {
        let shape = match f.kind {
            objects::FixtureKind::Toilet => {
                if f.paper_units > 0 { SpriteShape::ToiletWithPaper } else { SpriteShape::Toilet }
            }
            objects::FixtureKind::Sink => SpriteShape::Sink,
            objects::FixtureKind::Shower => SpriteShape::Shower,
            objects::FixtureKind::Urinal => SpriteShape::Urinal,
        };
        let elevation = match f.kind {
            objects::FixtureKind::Urinal => 0.0,
            objects::FixtureKind::Sink => 0.15,
            _ => 0.0,
        };
        SpriteInput { x: f.x, y: f.y, shape, scale: 1.0, elevation }
    }));
    for e in world.equippables.iter().filter(|e| !e.taken()) {
        let (x, y) = e.position();
        let on_bench = world.benches.iter()
            .any(|b| (b.x - x).abs() < 0.3 && (b.y - y).abs() < 0.3);
        let on_sofa = world.sofas.iter()
            .any(|s| (s.x - x).abs() < 0.4 && (s.y - y).abs() < 0.3);
        let (elevation, scale) = if on_sofa {
            (0.28, 0.55)
        } else if on_bench {
            (0.28, 0.55)
        } else {
            (0.0, 1.0)
        };
        sprites.push(SpriteInput {
            x, y,
            shape: e.sprite_shape(),
            scale,
            elevation,
        });
    }
    for d in &world.dropped {
        sprites.push(SpriteInput {
            x: d.x, y: d.y,
            shape: SpriteShape::ClothingPile { color: d.item.dropped_sprite_color() },
            scale: d.item.dropped_scale(),
            elevation: 0.0,
        });
    }
    for c in world.clothing.iter().filter(|c| !c.taken) {
        let shape = if matches!(c.kind, objects::ClothingKind::Hoodie) {
            SpriteShape::HangingClothing { color: c.color() }
        } else {
            SpriteShape::ClothingPile { color: c.color() }
        };
        sprites.push(SpriteInput {
            x: c.x, y: c.y,
            shape,
            scale: 1.0,
            elevation: c.elevation,
        });
    }
    sprites.extend(world.beds.iter().map(|b| {
        let shape = match b.kind {
            objects::BedKind::Hospital => SpriteShape::HospitalBed,
            objects::BedKind::Operating => SpriteShape::OperatingTable,
        };
        SpriteInput { x: b.x, y: b.y, shape, scale: 1.0, elevation: 0.0 }
    }));
    sprites.extend(world.burras.iter().map(|b| SpriteInput {
        x: b.x, y: b.y,
        shape: SpriteShape::Burra,
        scale: 1.0,
        elevation: 0.0,
    }));
    sprites.extend(world.decors.iter().map(|d| {
        let shape = match d.kind {
            objects::DecorKind::AluminumTable => SpriteShape::AluminumTable,
            objects::DecorKind::SurgicalCabinet => SpriteShape::SurgicalCabinet,
            objects::DecorKind::WoodenTable => SpriteShape::WoodenTable,
            objects::DecorKind::TreeOak => SpriteShape::TreeOak,
            objects::DecorKind::TreePine => SpriteShape::TreePine,
            objects::DecorKind::Bush => SpriteShape::Bush,
            objects::DecorKind::Weed => SpriteShape::Weed,
            objects::DecorKind::Car { body } => SpriteShape::Car { body },
            objects::DecorKind::BusStop => SpriteShape::BusStop,
            objects::DecorKind::Mailbox => SpriteShape::Mailbox,
        };
        SpriteInput { x: d.x, y: d.y, shape, scale: 1.0, elevation: d.elevation }
    }));
    sprites.extend(world.coat_racks.iter().map(|c| SpriteInput {
        x: c.x, y: c.y,
        shape: SpriteShape::CoatRack,
        scale: 1.0,
        elevation: 0.0,
    }));
    sprites.extend(world.npcs.iter().map(|n| SpriteInput {
        x: n.x, y: n.y,
        shape: SpriteShape::Person { skin: n.skin, shirt: n.shirt, pants: n.pants },
        scale: 1.0,
        elevation: 0.0,
    }));
    for door in &world.doors {
        let tile = world.map.get(door.tx, door.ty);
        let (open, is_main, is_operating) = match tile {
            Tile::Door { open } => (open, false, false),
            Tile::MainDoor { open } => (open, true, false),
            Tile::BathroomDoor { open } => (open, false, false),
            Tile::OperatingDoor { open } => (open, false, true),
            _ => (false, false, false),
        };
        if !open { continue; }
        let wx_left = matches!(world.map.get(door.tx.saturating_sub(1), door.ty), Tile::Wall | Tile::BathroomWall | Tile::HospitalOuterWall);
        let wx_right = matches!(world.map.get(door.tx + 1, door.ty), Tile::Wall | Tile::BathroomWall | Tile::HospitalOuterWall);
        let wy_top = matches!(world.map.get(door.tx, door.ty.saturating_sub(1)), Tile::Wall | Tile::BathroomWall | Tile::HospitalOuterWall);
        let wy_bot = matches!(world.map.get(door.tx, door.ty + 1), Tile::Wall | Tile::BathroomWall | Tile::HospitalOuterWall);
        let horizontal_wall = wx_left && wx_right;
        let vertical_wall = wy_top && wy_bot;
        let color = if is_operating { (50, 110, 180) } else if is_main { (10, 10, 12) } else { (200, 205, 210) };
        let axis = DoorLeafAxis::Vertical;
        let (a, b) = if vertical_wall {
            ((door.tx as f64 + 0.5, door.ty as f64 + 0.06),
             (door.tx as f64 + 0.5, door.ty as f64 + 0.94))
        } else if horizontal_wall {
            ((door.tx as f64 + 0.06, door.ty as f64 + 0.5),
             (door.tx as f64 + 0.94, door.ty as f64 + 0.5))
        } else {
            ((door.tx as f64 + 0.5, door.ty as f64 + 0.06),
             (door.tx as f64 + 0.5, door.ty as f64 + 0.94))
        };
        let positions: &[(f64, f64)] = if is_main { &[a, b] } else { &[a] };
        for (sx, sy) in positions {
            sprites.push(SpriteInput {
                x: *sx, y: *sy,
                shape: SpriteShape::DoorLeaf { color, axis },
                scale: 1.0,
                elevation: 0.0,
            });
        }
    }
    sprites.extend(world.signs.iter().map(|s| {
        let shape_kind = match s.kind {
            objects::SignKind::RoomSmall => SignShapeKind::RoomSmall,
            objects::SignKind::MainSquare => SignShapeKind::MainSquare,
            objects::SignKind::HallwayLong => SignShapeKind::HallwayLong,
            objects::SignKind::Bathroom => SignShapeKind::Bathroom,
            objects::SignKind::HospitalPlaque => SignShapeKind::HospitalPlaque,
        };
        SpriteInput {
            x: s.x, y: s.y,
            shape: SpriteShape::Sign(shape_kind),
            scale: 1.0,
            elevation: 0.62,
        }
    }));
    sprites
}
