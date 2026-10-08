use crate::game::tracks::TRACKS;
use crate::game::world::{GameMode, MENU_ITEMS, Request, TITLE_ITEMS, World};

/// Everything a frontend can ask of the world. Which keys produce which action is up
/// to the frontend; what an action does depends on the current [`GameMode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    // Movement: forward and sideways take a speed (2 = running)
    MoveForward(u8),
    MoveBackward,
    StrafeLeft(u8),
    StrafeRight(u8),
    RotateLeft,
    RotateRight,
    /// Turn by this many milliradians (mouse look); negative is left.
    Turn(i32),

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
    Wake,
    StandUp,
    Sleep,
    DrinkCanteen,
    ToggleBackpack,

    // Utility
    ToggleHidden,
    Save,
    Load,
    Restart,
}

const ROTATE_STEP: f64 = 0.22;

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
        _ => {}
    }

    match world.mode {
        GameMode::Startup => match action {
            Action::MenuUp => world.menu_cursor = world.menu_cursor.saturating_sub(1),
            Action::MenuDown => world.menu_cursor = (world.menu_cursor + 1).min(TITLE_ITEMS.len() - 1),
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
            Action::LaptopTogglePlay => world.toggle_track(),
            _ => {}
        },
        GameMode::Exploring => match action {
            Action::MoveForward(n) => world.player_move_forward(n),
            Action::MoveBackward => world.player_move_backward(),
            Action::StrafeLeft(n) => world.player_strafe(-1.0, n),
            Action::StrafeRight(n) => world.player_strafe(1.0, n),
            Action::RotateLeft => world.player.rotate(-ROTATE_STEP),
            Action::RotateRight => world.player.rotate(ROTATE_STEP),
            Action::Turn(mrad) => world.player.rotate(mrad as f64 / 1000.0),
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
