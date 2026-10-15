use crate::game::tracks::TRACKS;
use crate::game::world::{GameMode, MENU_ITEMS, Request, TITLE_ITEMS, World};

/// Everything a frontend can ask of the world. Which keys produce which action is up
/// to the frontend; what an action does depends on the current [`GameMode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Keep walking this way for `hold_ms`: `forward` and `side` are -1, 0 or 1,
    /// `run` is Shift. A key held down keeps sending these; a tap sends one.
    Move { forward: i8, side: i8, run: bool, hold_ms: u16 },
    /// Keep turning (-1 left, 1 right) for `hold_ms`: the arrow keys.
    TurnFor { dir: i8, hold_ms: u16 },
    /// Turn by this many milliradians at once (mouse look); negative is left.
    Turn(i32),
    /// Look up (positive) or down by this many milliradians.
    Pitch(i32),
    /// Level the view.
    CenterView,
    /// Stop walking and turning now.
    Halt,

    // The two context actions (E and F)
    Interact,
    Secondary,

    // Inventory (GameMode::Inventory)
    InventoryUp,
    InventoryDown,
    InventoryUseSelected,
    InventoryDropSelected,

    // Emergency terminal (GameMode::Laptop)
    LaptopUp,
    LaptopDown,
    LaptopClose,
    LaptopTogglePlay,

    // Mode changes
    OpenInventory,
    CloseInventory,
    OpenMenu,
    CloseMenu,
    MenuUp,
    MenuDown,
    MenuSelect,
    /// Put the cursor on an item (a click or hover).
    MenuPick(usize),
    InventoryPick(usize),
    LaptopPick(usize),
    Wake,
    StandUp,
    Sleep,
    DrinkCanteen,
    ToggleBackpack,

    // Utility
    ToggleHidden,
    ToggleFlashlight,
    Save,
    Load,
    Restart,
}

pub fn dispatch(world: &mut World, action: Action) {
    match action {
        Action::Save => {
            if world.can_save() {
                world.request(Request::Save);
            } else {
                world.say("You can't save right now.");
            }
            return;
        }
        Action::Load => {
            world.request(Request::Load);
            return;
        }
        Action::Halt => {
            world.halt();
            return;
        }
        _ => {}
    }
    // you can walk about with the bag open
    if matches!(world.mode, GameMode::Exploring | GameMode::Inventory) {
        match action {
            Action::Move { forward, side, run, hold_ms } => {
                // the longest hold means "until told otherwise": a locked walk
                let until = if hold_ms == u16::MAX { u64::MAX } else { world.now_ms + hold_ms as u64 };
                return world.set_motion(forward as f64, side as f64, run, until);
            }
            Action::TurnFor { dir, hold_ms } => return world.set_turning(dir as f64, world.now_ms + hold_ms as u64),
            Action::Turn(mrad) => return world.player.rotate(mrad as f64 / 1000.0),
            Action::Pitch(mrad) => return world.player.pitch_by(mrad as f64 / 1000.0),
            Action::CenterView => return world.player.pitch = 0.0,
            Action::ToggleFlashlight => return world.toggle_flashlight(),
            _ => {}
        }
    }

    match world.mode {
        GameMode::Startup => match action {
            Action::MenuUp => world.menu_cursor = world.menu_cursor.saturating_sub(1),
            Action::MenuDown => world.menu_cursor = (world.menu_cursor + 1).min(TITLE_ITEMS.len() - 1),
            Action::MenuPick(i) => world.menu_cursor = i.min(TITLE_ITEMS.len() - 1),
            Action::MenuSelect => match world.menu_cursor {
                0 => {
                    world.mode = GameMode::InitialWake;
                    world.menu_cursor = 0;
                }
                1 => world.request(Request::Load),
                _ => world.request(Request::Quit),
            },
            _ => {}
        },
        GameMode::InitialWake => {
            if matches!(action, Action::Wake | Action::Interact) {
                world.get_up();
            }
        }
        GameMode::Menu => match action {
            Action::CloseMenu => world.mode = GameMode::Exploring,
            Action::MenuUp => world.menu_cursor = world.menu_cursor.saturating_sub(1),
            Action::MenuDown => world.menu_cursor = (world.menu_cursor + 1).min(MENU_ITEMS.len() - 1),
            Action::MenuPick(i) => world.menu_cursor = i.min(MENU_ITEMS.len() - 1),
            Action::MenuSelect => match world.menu_cursor {
                0 => world.mode = GameMode::Exploring,
                1 => {
                    world.request(Request::Save);
                    world.mode = GameMode::Exploring;
                }
                2 => {
                    world.request(Request::Load);
                    world.mode = GameMode::Exploring;
                }
                3 => world.request(Request::ToggleSound),
                4 => world.reset(),
                _ => world.request(Request::Quit),
            },
            _ => {}
        },
        GameMode::Inventory => match action {
            Action::CloseInventory => world.mode = GameMode::Exploring,
            Action::InventoryUp => world.player.inventory_cursor = world.player.inventory_cursor.saturating_sub(1),
            Action::InventoryDown => {
                let last = world.player.inventory.capacity.saturating_sub(1);
                world.player.inventory_cursor = (world.player.inventory_cursor + 1).min(last);
            }
            Action::InventoryPick(i) => world.player.inventory_cursor = i.min(world.player.inventory.capacity.saturating_sub(1)),
            Action::InventoryUseSelected => {
                let c = world.player.inventory_cursor;
                world.use_inventory_slot(c);
            }
            Action::InventoryDropSelected => {
                let c = world.player.inventory_cursor;
                world.drop_inventory_slot(c);
            }
            Action::ToggleBackpack => world.toggle_backpack(),
            _ => {}
        },
        GameMode::Sitting | GameMode::Lying => match action {
            Action::StandUp | Action::Interact => world.get_up(),
            Action::Sleep => world.sleep(),
            Action::ToggleFlashlight => world.toggle_flashlight(),
            Action::Turn(mrad) if world.mode == GameMode::Sitting => world.player.rotate(mrad as f64 / 1000.0),
            Action::Pitch(mrad) if world.mode == GameMode::Sitting => world.player.pitch_by(mrad as f64 / 1000.0),
            Action::CenterView => world.player.pitch = 0.0,
            Action::TurnFor { dir, hold_ms } if world.mode == GameMode::Sitting => world.set_turning(dir as f64, world.now_ms + hold_ms as u64),
            _ => {}
        },
        GameMode::Sleeping => {}
        GameMode::Dead => {
            if matches!(action, Action::Restart | Action::MenuSelect | Action::Interact) {
                world.reset();
            }
        }
        GameMode::Laptop => match action {
            Action::LaptopClose => world.close_laptop(),
            Action::LaptopUp => world.laptop.cursor = world.laptop.cursor.saturating_sub(1),
            Action::LaptopDown => world.laptop.cursor = (world.laptop.cursor + 1).min(TRACKS.len() - 1),
            Action::LaptopPick(i) => world.laptop.cursor = i.min(TRACKS.len() - 1),
            Action::LaptopTogglePlay => world.toggle_track(),
            _ => {}
        },
        GameMode::Exploring => match action {
            Action::ToggleHidden => world.toggle_crouch(),
            Action::OpenMenu => {
                world.mode = GameMode::Menu;
                world.menu_cursor = 0;
            }
            Action::OpenInventory => {
                if world.player.has_backpack {
                    world.mode = GameMode::Inventory;
                } else {
                    world.say("You're not carrying a backpack.");
                }
            }
            Action::Interact => world.interact(),
            Action::Secondary => world.secondary(),
            Action::ToggleBackpack => world.toggle_backpack(),
            Action::DrinkCanteen => {
                world.drink_from_canteen();
            }
            _ => {}
        },
    }
}
