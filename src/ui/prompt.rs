use crate::game::world::{GameMode, World};
use crate::objects::{FixtureKind, VendingKind};

pub fn build_action_label(world: &World) -> Option<String> {
    if world.mode != GameMode::Exploring {
        return Some("Stand Up".into());
    }
    if let Some(idx) = world.nearby_dropped_idx() {
        return Some(format!("Pick up {}", world.dropped[idx].item.label()));
    }
    if let Some(idx) = world.nearby_equippable_idx() {
        return Some(format!("Pick up {}", world.equippables[idx].label()));
    }
    if let Some(idx) = world.nearby_pickable_clothing_idx() {
        let verb = if world.player.has_backpack { "Store" } else { "Wear" };
        return Some(format!("{} {}", verb, world.clothing[idx].label()));
    }
    if let Some(idx) = world.nearby_npc_idx() {
        return Some(format!("Talk to {}", world.npcs[idx].name));
    }
    if world.nearby_sofa_idx().is_some() {
        return Some("Sit Down".into());
    }
    if world.nearby_bench_idx().is_some() {
        return Some("Sit Down".into());
    }
    if world.nearby_bed_idx().is_some() {
        return Some("Lie Down  ·  Z: Sleep".into());
    }
    if world.nearby_window_idx().is_some() {
        return Some("Look out Window".into());
    }
    if let Some(idx) = world.nearby_vending_idx() {
        let label = match world.vending[idx].kind {
            VendingKind::Drinks => "Take Energy Drink  ·  Z: Take Coffee",
            VendingKind::Snacks => "Take Energy Bar",
        };
        return Some(label.into());
    }
    if let Some(idx) = world.nearby_fixture_idx() {
        let f = &world.fixtures[idx];
        let label = match f.kind {
            FixtureKind::Sink => {
                if world.has_canteen_in_inventory() && world.player.canteen_fill < 100.0 {
                    "Drink  ·  Z: Fill Canteen".to_string()
                } else {
                    "Drink  ·  Z: Wash".to_string()
                }
            }
            FixtureKind::Shower => "Take Shower".to_string(),
            FixtureKind::Toilet => {
                if f.paper_units > 0 || world.player_has_paper() {
                    "Use Toilet  ·  Z: Take Paper".to_string()
                } else {
                    "Use Toilet".to_string()
                }
            }
            FixtureKind::Urinal => "Use Urinal".to_string(),
        };
        return Some(label);
    }
    if world.front_door_pos().is_some() {
        return Some("Door".into());
    }
    None
}
