//! Keys to actions, the way most PC games lay them out: WASD to move and strafe, the
//! arrows (or a mouse drag) to turn, Shift to run, E to use, F for the second action,
//! C to crouch, Tab or I for the inventory, Esc for the menu, F5/F9 to save and load.

use hackkid_core::game::action::Action;
use hackkid_core::game::world::GameMode;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// Maps a key press to an action for the current mode. Key releases (which Windows
/// reports too) are ignored, so every key acts once.
pub fn key_to_action(key: KeyEvent, mode: GameMode) -> Option<Action> {
    if key.kind == KeyEventKind::Release {
        return None;
    }
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    match key.code {
        KeyCode::F(5) => return Some(Action::Save),
        KeyCode::F(9) => return Some(Action::Load),
        KeyCode::Char('s') if ctrl => return Some(Action::Save),
        KeyCode::Char('l') if ctrl => return Some(Action::Load),
        _ if ctrl => return None,
        _ => {}
    }
    // terminals send Shift+W as a capital W; arrows carry the modifier
    let shift = key.modifiers.contains(KeyModifiers::SHIFT) || matches!(key.code, KeyCode::Char(c) if c.is_ascii_uppercase());
    let pace = if shift { 2 } else { 1 };
    let up = matches!(key.code, KeyCode::Up | KeyCode::Char('w' | 'W'));
    let down = matches!(key.code, KeyCode::Down | KeyCode::Char('s' | 'S'));
    let use_key = matches!(key.code, KeyCode::Char('e' | 'E') | KeyCode::Enter);
    let back = matches!(key.code, KeyCode::Esc);

    match mode {
        GameMode::Startup | GameMode::Menu => match key.code {
            _ if up => Some(Action::MenuUp),
            _ if down => Some(Action::MenuDown),
            _ if use_key => Some(Action::MenuSelect),
            _ if back && mode == GameMode::Menu => Some(Action::CloseMenu),
            _ => None,
        },
        GameMode::InitialWake => Some(Action::Wake),
        GameMode::Sleeping => None,
        GameMode::Dead => (use_key || key.code == KeyCode::Char(' ')).then_some(Action::Restart),
        GameMode::Inventory => match key.code {
            KeyCode::Tab | KeyCode::Char('i' | 'I') => Some(Action::CloseInventory),
            _ if back => Some(Action::CloseInventory),
            _ if up => Some(Action::InventoryUp),
            _ if down => Some(Action::InventoryDown),
            _ if use_key => Some(Action::InventoryUseSelected),
            KeyCode::Char('g' | 'G') => Some(Action::InventoryDropSelected),
            KeyCode::Char('b' | 'B') => Some(Action::ToggleBackpack),
            _ => None,
        },
        GameMode::Sitting => (matches!(key.code, KeyCode::Char('e' | 'E')) || back).then_some(Action::StandUp),
        GameMode::Lying => match key.code {
            KeyCode::Char('e' | 'E') => Some(Action::StandUp),
            _ if back => Some(Action::StandUp),
            KeyCode::Char('f' | 'F') => Some(Action::Sleep),
            _ => None,
        },
        GameMode::Laptop => match key.code {
            KeyCode::Tab => Some(Action::LaptopClose),
            _ if back => Some(Action::LaptopClose),
            _ if up => Some(Action::LaptopUp),
            _ if down => Some(Action::LaptopDown),
            _ if use_key => Some(Action::LaptopTogglePlay),
            _ => None,
        },
        GameMode::Exploring => match key.code {
            _ if up => Some(Action::MoveForward(pace)),
            _ if down => Some(Action::MoveBackward),
            KeyCode::Char('a' | 'A') => Some(Action::StrafeLeft(pace)),
            KeyCode::Char('d' | 'D') => Some(Action::StrafeRight(pace)),
            KeyCode::Left => Some(Action::RotateLeft),
            KeyCode::Right => Some(Action::RotateRight),
            KeyCode::Char('e' | 'E') => Some(Action::Interact),
            KeyCode::Char('f' | 'F') => Some(Action::Secondary),
            KeyCode::Char('c' | 'C') => Some(Action::ToggleHidden),
            KeyCode::Char('q' | 'Q') => Some(Action::DrinkCanteen),
            KeyCode::Char('b' | 'B') => Some(Action::ToggleBackpack),
            KeyCode::Tab | KeyCode::Char('i' | 'I') => Some(Action::OpenInventory),
            _ if back => Some(Action::OpenMenu),
            _ => None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn the_standard_layout() {
        let ex = GameMode::Exploring;
        assert_eq!(key_to_action(key(KeyCode::Char('e')), ex), Some(Action::Interact));
        assert_eq!(key_to_action(key(KeyCode::Char('f')), ex), Some(Action::Secondary));
        assert_eq!(key_to_action(key(KeyCode::Char('w')), ex), Some(Action::MoveForward(1)));
        assert_eq!(key_to_action(key(KeyCode::Char('W')), ex), Some(Action::MoveForward(2)), "Shift runs");
        assert_eq!(key_to_action(key(KeyCode::Char('a')), ex), Some(Action::StrafeLeft(1)));
        assert_eq!(key_to_action(key(KeyCode::Left), ex), Some(Action::RotateLeft));
        assert_eq!(key_to_action(KeyEvent::new(KeyCode::Up, KeyModifiers::SHIFT), ex), Some(Action::MoveForward(2)));
        assert_eq!(key_to_action(key(KeyCode::Char('c')), ex), Some(Action::ToggleHidden));
        assert_eq!(key_to_action(key(KeyCode::Tab), ex), Some(Action::OpenInventory));
        assert_eq!(key_to_action(key(KeyCode::Tab), GameMode::Inventory), Some(Action::CloseInventory));
        assert_eq!(key_to_action(key(KeyCode::Esc), ex), Some(Action::OpenMenu));
        assert_eq!(key_to_action(key(KeyCode::F(5)), ex), Some(Action::Save));
        assert_eq!(key_to_action(key(KeyCode::Char('g')), GameMode::Inventory), Some(Action::InventoryDropSelected));
        assert_eq!(key_to_action(key(KeyCode::Char('f')), GameMode::Lying), Some(Action::Sleep));
        let release = KeyEvent { kind: KeyEventKind::Release, ..key(KeyCode::Char('e')) };
        assert_eq!(key_to_action(release, ex), None, "Windows key releases don't act twice");
    }
}
