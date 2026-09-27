use crate::game::world::{GameMode, World};
use crate::objects::FixtureKind;

pub fn build_action_label(world: &World) -> Option<String> {
    if world.mode != GameMode::Exploring {
        return Some("Levantarse".into());
    }
    if let Some(idx) = world.nearby_dropped_idx() {
        return Some(format!("Coger {}", world.dropped[idx].item.label()));
    }
    if let Some(idx) = world.nearby_equippable_idx() {
        return Some(format!("Coger {}", world.equippables[idx].label()));
    }
    if let Some(idx) = world.nearby_pickable_clothing_idx() {
        let verb = if world.player.has_backpack { "Guardar" } else { "Vestir" };
        return Some(format!("{} {}", verb, world.clothing[idx].label()));
    }
    if let Some(idx) = world.nearby_npc_idx() {
        return Some(format!("Dialogar {}", world.npcs[idx].name));
    }
    if world.nearby_sofa_idx().is_some() {
        return Some("Sentarse".into());
    }
    if world.nearby_bench_idx().is_some() {
        return Some("Sentarse".into());
    }
    if world.nearby_bed_idx().is_some() {
        return Some("Tumbarse  ·  Z: dormir".into());
    }
    if world.nearby_window_idx().is_some() {
        return Some("Mirar por la ventana".into());
    }
    if world.nearby_vending_idx().is_some() {
        return Some("Coger refresco  ·  Z: coger café".into());
    }
    if let Some(idx) = world.nearby_fixture_idx() {
        let f = &world.fixtures[idx];
        let label = match f.kind {
            FixtureKind::Sink => "Beber  ·  Z: asearse".to_string(),
            FixtureKind::Shower => "Ducharse".to_string(),
            FixtureKind::Toilet => {
                if f.paper_units > 0 || world.player_has_paper() {
                    "Usar water  ·  Z: coger papel".to_string()
                } else {
                    "Usar water".to_string()
                }
            }
            FixtureKind::Urinal => "Usar urinario".to_string(),
        };
        return Some(label);
    }
    if world.front_door_pos().is_some() {
        return Some("Puerta".into());
    }
    None
}
