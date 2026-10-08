//! The flat sprites in view: people, clothes, fixtures, plants and other props.
//! Furniture is drawn as boxes instead ([`World::boxes`]).

use crate::engine::boxes::BoxKind;
use crate::engine::sprites::{SpriteInput, SpriteShape};
use crate::game::world::World;
use crate::objects::{self, Visual};
use crate::engine::map::Tile;

pub fn collect_sprites(world: &World) -> Vec<SpriteInput> {
    let mut out: Vec<SpriteInput> = Vec::new();
    for f in &world.fixtures {
        let (shape, elev) = match f.kind {
            objects::FixtureKind::Toilet if f.paper_units > 0 => (SpriteShape::ToiletWithPaper, 0.0),
            objects::FixtureKind::Toilet => (SpriteShape::Toilet, 0.0),
            objects::FixtureKind::Sink => (SpriteShape::Sink, 0.62),
            objects::FixtureKind::Shower => (SpriteShape::Shower, 0.0),
            objects::FixtureKind::Urinal => (SpriteShape::Urinal, 0.35),
        };
        out.push(SpriteInput::new(f.x, f.y, shape).raised(elev));
    }
    for p in &world.props {
        if let Visual::Sprite(shape) = p.kind.visual() {
            out.push(SpriteInput::new(p.x, p.y, shape).raised(p.elevation));
        }
    }
    // a backpack left on a seat sits on it
    let seats: Vec<(f64, f64)> = world.sofas.iter().map(|s| (s.x, s.y)).chain(world.benches.iter().map(|b| (b.x, b.y))).collect();
    for e in &world.equippables {
        let (x, y) = e.position();
        let on_seat = seats.iter().any(|&(sx, sy)| (sx - x).abs() < 1.0 && (sy - y).abs() < 1.0);
        out.push(SpriteInput::new(x, y, e.sprite_shape()).raised(if on_seat { BoxKind::SofaSeat.size().2 } else { 0.0 }));
    }
    for d in &world.dropped {
        out.push(SpriteInput::new(d.x, d.y, SpriteShape::ClothingPile { color: d.item.dropped_sprite_color() }).scaled(d.item.dropped_scale()));
    }
    for c in world.clothing.iter().filter(|c| !c.taken) {
        let shape = if matches!(c.kind, objects::ClothingKind::Hoodie) {
            SpriteShape::HangingClothing { color: c.color() }
        } else {
            SpriteShape::ClothingPile { color: c.color() }
        };
        out.push(SpriteInput::new(c.x, c.y, shape).raised(c.elevation));
    }
    for n in &world.npcs {
        out.push(SpriteInput::new(n.x, n.y, SpriteShape::Person(n.look())));
    }
    // open doors: the leaf swung back against the jamb
    for door in &world.doors {
        if world.map.get(door.tx, door.ty).door_open() != Some(true) {
            continue;
        }
        let solid = |x: usize, y: usize| world.map.get(x, y).blocks_sight() && !world.map.get(x, y).is_any_door();
        let in_ns_wall = solid(door.tx.saturating_sub(1), door.ty) || solid(door.tx + 1, door.ty);
        let (x, y) = if in_ns_wall {
            (door.tx as f64 + 0.06, door.ty as f64 + 0.5)
        } else {
            (door.tx as f64 + 0.5, door.ty as f64 + 0.06)
        };
        let color = match world.map.get(door.tx, door.ty) {
            Tile::MainDoor { .. } => (150, 156, 162),
            Tile::OperatingDoor { .. } => (180, 186, 190),
            Tile::BathroomDoor { .. } => (190, 202, 212),
            _ => (196, 158, 110),
        };
        out.push(SpriteInput::new(x, y, SpriteShape::DoorLeaf { color }));
    }
    out
}
