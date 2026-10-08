//! What the player is next to, what X and Z do with it, and the prompt that says so.
//!
//! Everything goes through [`World::focus`], so the prompt on screen and what the keys
//! actually do can't drift apart.

use crate::game::world::World;
use crate::objects::{FixtureKind, VendingKind};

/// The one thing the player would interact with right now, nearest kinds first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Dropped(usize),
    Equippable(usize),
    Clothing(usize),
    Npc(usize),
    Sofa(usize),
    Bench(usize),
    Bed(usize),
    Fixture(usize),
    Vending(usize),
    Container(usize),
    Window(usize, usize),
    Door(usize, usize),
}

impl World {
    pub fn focus(&self) -> Option<Focus> {
        None.or_else(|| self.nearby_dropped_idx().map(Focus::Dropped))
            .or_else(|| self.nearby_equippable_idx().map(Focus::Equippable))
            .or_else(|| self.nearby_pickable_clothing_idx().map(Focus::Clothing))
            .or_else(|| self.nearby_npc_idx().map(Focus::Npc))
            .or_else(|| self.nearby_sofa_idx().map(Focus::Sofa))
            .or_else(|| self.nearby_bench_idx().map(Focus::Bench))
            .or_else(|| self.nearby_bed_idx().map(Focus::Bed))
            .or_else(|| self.nearby_fixture_idx().map(Focus::Fixture))
            .or_else(|| self.nearby_vending_idx().map(Focus::Vending))
            .or_else(|| self.nearby_container_idx().map(Focus::Container))
            .or_else(|| self.front_window().map(|(x, y)| Focus::Window(x, y)))
            .or_else(|| self.front_door_pos().map(|(x, y)| Focus::Door(x, y)))
    }

    /// E: the main action on whatever is in focus.
    pub fn interact(&mut self) {
        match self.focus() {
            Some(Focus::Dropped(i)) => self.pick_dropped(i),
            Some(Focus::Equippable(i)) => self.pick_equippable(i),
            Some(Focus::Clothing(i)) => self.pick_clothing(i),
            Some(Focus::Npc(i)) => self.talk_to(i),
            Some(Focus::Sofa(i)) => self.sit_on_sofa(i),
            Some(Focus::Bench(i)) => self.sit_on_bench(i),
            Some(Focus::Bed(i)) => self.lie_on_bed(i),
            Some(Focus::Fixture(i)) => self.use_fixture(i),
            Some(Focus::Vending(i)) => self.use_vending(i, false),
            Some(Focus::Container(i)) => self.search(i),
            Some(Focus::Window(..)) => self.observe_window(),
            Some(Focus::Door(x, y)) => self.toggle_door(x, y),
            None => {}
        }
    }

    /// F: the second action, where there is one.
    pub fn secondary(&mut self) {
        match self.focus() {
            Some(Focus::Bed(i)) => self.sleep_at_bed(i),
            Some(Focus::Vending(i)) => self.use_vending(i, true),
            Some(Focus::Fixture(i)) => match self.fixtures[i].kind {
                FixtureKind::Sink => self.sink_secondary(),
                FixtureKind::Toilet => self.take_paper(i),
                _ => {}
            },
            _ => {}
        }
    }

    /// The prompt for the focus, e.g. `E: Lie down  ·  F: Sleep`.
    pub fn action_label(&self) -> Option<String> {
        let label = match self.focus()? {
            Focus::Dropped(i) => format!("Pick up {}", self.dropped[i].item.label()),
            Focus::Equippable(i) => format!("Pick up {}", self.equippables[i].label()),
            Focus::Clothing(i) => {
                let verb = if self.player.has_backpack { "Store" } else { "Wear" };
                format!("{verb} {}", self.clothing[i].label())
            }
            Focus::Npc(i) => format!("Talk to {}", self.npcs[i].name),
            Focus::Sofa(_) | Focus::Bench(_) => "Sit down".into(),
            Focus::Bed(_) => "Lie down  ·  F: Sleep".into(),
            Focus::Window(..) => "Look out of the window".into(),
            Focus::Container(i) => {
                let c = &self.containers[i];
                if c.searched && c.items.is_empty() {
                    format!("Search {} (empty)", c.kind.label())
                } else {
                    format!("Search {}", c.kind.label())
                }
            }
            Focus::Vending(i) => match self.vending[i].kind {
                VendingKind::Drinks => "Take energy drink  ·  F: Take coffee".into(),
                VendingKind::Snacks => "Take energy bar".into(),
            },
            Focus::Fixture(i) => {
                let f = &self.fixtures[i];
                match f.kind {
                    FixtureKind::Sink if self.has_canteen_in_inventory() && self.player.canteen_fill < 100.0 => "Drink  ·  F: Fill flask".into(),
                    FixtureKind::Sink => "Drink  ·  F: Wash".into(),
                    FixtureKind::Shower => "Take a shower".into(),
                    FixtureKind::Toilet if f.paper_units > 0 => "Use toilet  ·  F: Take paper".into(),
                    FixtureKind::Toilet => "Use toilet".into(),
                    FixtureKind::Urinal => "Use urinal".into(),
                }
            }
            Focus::Door(x, y) => {
                let tile = self.map.get(x, y);
                match tile.door_open() {
                    Some(true) => "Close door".into(),
                    _ if self.door_lock(x, y).is_some_and(|k| !self.player.inventory.has(k)) => "Locked door".into(),
                    _ => "Open door".into(),
                }
            }
        };
        Some(format!("E: {label}"))
    }
}
