//! Keys to actions, the way most PC games lay them out: WASD to move and strafe, the
//! arrows (or the mouse) to turn, Shift to run, E to use, F for the second action,
//! C to crouch, L for the flashlight, Tab or I for the inventory, M for the map, Esc for
//! the menu, F5/F9 to save and load.
//!
//! A terminal only reports key presses, never releases, so holding a key is a stream of
//! repeats. Each press asks the game to keep moving for a short while; the repeats
//! keep extending that, and a tap gives one short step.

use hackkid_core::game::action::Action;
use hackkid_core::game::world::GameMode;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// How long a repeat keeps you moving: longer than the gap between repeats.
pub const REPEAT_HOLD_MS: u16 = 180;
/// How long the first press keeps you moving, until the terminal's repeat delay is known.
pub const FIRST_HOLD_MS: u16 = 420;

/// Maps a key press to an action for the current mode. Key releases (which Windows
/// reports too) are ignored, so every key acts once. `hold_ms` is how long a movement
/// key should keep its effect.
pub fn key_to_action(key: KeyEvent, mode: GameMode, hold_ms: u16) -> Option<Action> {
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
    let up = matches!(key.code, KeyCode::Up | KeyCode::Char('w' | 'W'));
    let down = matches!(key.code, KeyCode::Down | KeyCode::Char('s' | 'S'));
    let use_key = matches!(key.code, KeyCode::Char('e' | 'E') | KeyCode::Enter);
    let back = matches!(key.code, KeyCode::Esc);
    let walk = |forward: i8, side: i8| Some(Action::Move { forward, side, run: shift, hold_ms });

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
        // the bag stays open while you walk: WASD moves, the arrows pick a slot
        GameMode::Inventory => match key.code {
            KeyCode::Tab | KeyCode::Char('i' | 'I') => Some(Action::CloseInventory),
            _ if back => Some(Action::CloseInventory),
            KeyCode::Up => Some(Action::InventoryUp),
            KeyCode::Down => Some(Action::InventoryDown),
            KeyCode::Char('w' | 'W') => walk(1, 0),
            KeyCode::Char('s' | 'S') => walk(-1, 0),
            KeyCode::Char('a' | 'A') => walk(0, -1),
            KeyCode::Char('d' | 'D') => walk(0, 1),
            KeyCode::Left => Some(Action::TurnFor { dir: -1, hold_ms }),
            KeyCode::Right => Some(Action::TurnFor { dir: 1, hold_ms }),
            _ if use_key => Some(Action::InventoryUseSelected),
            KeyCode::Char('g' | 'G') => Some(Action::InventoryDropSelected),
            KeyCode::Char('b' | 'B') => Some(Action::ToggleBackpack),
            KeyCode::Char('l' | 'L') => Some(Action::ToggleFlashlight),
            _ => None,
        },
        GameMode::Sitting => match key.code {
            KeyCode::Char('e' | 'E') => Some(Action::StandUp),
            _ if back => Some(Action::StandUp),
            KeyCode::Left => Some(Action::TurnFor { dir: -1, hold_ms }),
            KeyCode::Right => Some(Action::TurnFor { dir: 1, hold_ms }),
            KeyCode::Char('l' | 'L') => Some(Action::ToggleFlashlight),
            _ => None,
        },
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
            _ if up => walk(1, 0),
            _ if down => walk(-1, 0),
            KeyCode::Char('a' | 'A') => walk(0, -1),
            KeyCode::Char('d' | 'D') => walk(0, 1),
            KeyCode::Left => Some(Action::TurnFor { dir: -1, hold_ms }),
            KeyCode::Right => Some(Action::TurnFor { dir: 1, hold_ms }),
            KeyCode::Char('e' | 'E') => Some(Action::Interact),
            KeyCode::Char('f' | 'F') => Some(Action::Secondary),
            KeyCode::Char('c' | 'C') => Some(Action::ToggleHidden),
            KeyCode::Char('l' | 'L') => Some(Action::ToggleFlashlight),
            KeyCode::Char('q' | 'Q') => Some(Action::DrinkCanteen),
            KeyCode::Char('b' | 'B') => Some(Action::ToggleBackpack),
            KeyCode::Tab | KeyCode::Char('i' | 'I') => Some(Action::OpenInventory),
            _ if back => Some(Action::OpenMenu),
            _ => None,
        },
    }
}

/// Learns how the terminal repeats keys, so a tap moves you one short step while a held
/// key keeps you moving without a stutter: the first press holds long enough to bridge
/// the terminal's repeat delay (once seen), and repeats hold just past their interval.
pub struct KeyRepeat {
    last_code: Option<KeyCode>,
    last_ms: u64,
    /// The terminal's delay before it starts repeating, once measured.
    first_hold_ms: u16,
}

impl Default for KeyRepeat {
    fn default() -> Self {
        Self { last_code: None, last_ms: 0, first_hold_ms: FIRST_HOLD_MS }
    }
}

impl KeyRepeat {
    /// How long this press of `code` should hold its movement.
    pub fn hold_for(&mut self, code: KeyCode, now_ms: u64) -> u16 {
        let gap = now_ms.saturating_sub(self.last_ms);
        let same = self.last_code == Some(code);
        self.last_code = Some(code);
        self.last_ms = now_ms;
        if same && gap < 120 {
            // repeating: keep going a little past the next repeat
            REPEAT_HOLD_MS
        } else if same && gap < 900 {
            // the first repeat after a press: that's the terminal's delay
            self.first_hold_ms = (gap as u16 + 80).clamp(200, 800);
            REPEAT_HOLD_MS
        } else {
            self.first_hold_ms
        }
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
        let walk = |f: i8, s: i8, run: bool| Some(Action::Move { forward: f, side: s, run, hold_ms: 100 });
        assert_eq!(key_to_action(key(KeyCode::Char('e')), ex, 100), Some(Action::Interact));
        assert_eq!(key_to_action(key(KeyCode::Char('f')), ex, 100), Some(Action::Secondary));
        assert_eq!(key_to_action(key(KeyCode::Char('w')), ex, 100), walk(1, 0, false));
        assert_eq!(key_to_action(key(KeyCode::Char('W')), ex, 100), walk(1, 0, true), "Shift runs");
        assert_eq!(key_to_action(key(KeyCode::Char('a')), ex, 100), walk(0, -1, false));
        assert_eq!(key_to_action(key(KeyCode::Left), ex, 100), Some(Action::TurnFor { dir: -1, hold_ms: 100 }));
        assert_eq!(key_to_action(KeyEvent::new(KeyCode::Up, KeyModifiers::SHIFT), ex, 100), walk(1, 0, true));
        assert_eq!(key_to_action(key(KeyCode::Char('c')), ex, 100), Some(Action::ToggleHidden));
        assert_eq!(key_to_action(key(KeyCode::Char('l')), ex, 100), Some(Action::ToggleFlashlight));
        assert_eq!(key_to_action(key(KeyCode::Tab), ex, 100), Some(Action::OpenInventory));
        assert_eq!(key_to_action(key(KeyCode::Tab), GameMode::Inventory, 100), Some(Action::CloseInventory));
        assert_eq!(key_to_action(key(KeyCode::Char('w')), GameMode::Inventory, 100), walk(1, 0, false), "you walk with the bag open");
        assert_eq!(key_to_action(key(KeyCode::Up), GameMode::Inventory, 100), Some(Action::InventoryUp));
        assert_eq!(key_to_action(key(KeyCode::Esc), ex, 100), Some(Action::OpenMenu));
        assert_eq!(key_to_action(key(KeyCode::F(5)), ex, 100), Some(Action::Save));
        assert_eq!(key_to_action(key(KeyCode::Char('g')), GameMode::Inventory, 100), Some(Action::InventoryDropSelected));
        assert_eq!(key_to_action(key(KeyCode::Char('f')), GameMode::Lying, 100), Some(Action::Sleep));
        let release = KeyEvent { kind: KeyEventKind::Release, ..key(KeyCode::Char('e')) };
        assert_eq!(key_to_action(release, ex, 100), None, "Windows key releases don't act twice");
    }

    #[test]
    fn repeats_are_learned() {
        let mut r = KeyRepeat::default();
        let w = KeyCode::Char('w');
        assert_eq!(r.hold_for(w, 1_000), FIRST_HOLD_MS, "a first press holds the default");
        // the terminal starts repeating 500 ms later: now the first hold bridges that
        assert_eq!(r.hold_for(w, 1_500), REPEAT_HOLD_MS);
        assert_eq!(r.hold_for(w, 1_533), REPEAT_HOLD_MS, "repeats hold just past the next one");
        assert_eq!(r.hold_for(w, 5_000), 580, "a fresh press holds the learned delay plus a margin");
        assert_eq!(r.hold_for(KeyCode::Char('a'), 5_010), 580, "another key is its own first press");
    }
}
