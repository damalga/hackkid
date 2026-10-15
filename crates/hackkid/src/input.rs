//! Keys to actions: W and S (or Up/Down) walk forward and back, A and D (or Left/Right)
//! turn you on the spot (`,` and `.` are aliases), Z toggles the view between level and
//! 45° down and X toggles between level and 45° up (Space also levels it), Shift runs,
//! E uses, F is the second action, C crouches (press again to stand), L is the
//! flashlight, Tab or I the inventory, M the map, Esc the menu, F5/F9 save and load.
//!
//! A terminal only reports key presses, never releases, so holding a key is a stream of
//! repeats. Each press asks the game to keep moving for a short while; the repeats
//! keep extending that, and a tap gives one short step.

use hackkid_core::game::action::Action;
use hackkid_core::game::world::GameMode;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// How long a repeat keeps you moving: longer than the gap between repeats, but short
/// enough that a tap is a short nudge, not a lunge.
pub const REPEAT_HOLD_MS: u16 = 110;
/// How long the first press keeps you moving, until the terminal's repeat delay is known.
pub const FIRST_HOLD_MS: u16 = 240;
/// Running needs a longer hold so swapping direction keys with Shift held doesn't drop
/// back to walking in the gap between two keypresses. A fresh direction key with Shift
/// still runs; the previous intent stays running until the new one lands.
const RUN_HOLD_FLOOR_MS: u16 = 360;
/// The fixed look-up and look-down stop, in milliradians: 45°. Z and X snap to this,
/// Space levels back to zero. Clamped at the neck's limit (about 66°) in the player.
const FIXED_PITCH_MRAD: i32 = 785;

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
    let move_hold = if shift { hold_ms.max(RUN_HOLD_FLOOR_MS) } else { hold_ms };
    let walk = |forward: i8, side: i8| Some(Action::Move { forward, side, run: shift, hold_ms: move_hold });
    let turn_for = |dir: i8| Some(Action::TurnFor { dir, hold_ms });

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
        // the bag stays open while you walk: W/S move, A/D turn, Up/Down pick a slot
        GameMode::Inventory => match key.code {
            KeyCode::Tab | KeyCode::Char('i' | 'I') => Some(Action::CloseInventory),
            _ if back => Some(Action::CloseInventory),
            KeyCode::Up => Some(Action::InventoryUp),
            KeyCode::Down => Some(Action::InventoryDown),
            KeyCode::Char('w' | 'W') => walk(1, 0),
            KeyCode::Char('s' | 'S') => walk(-1, 0),
            KeyCode::Char('a' | 'A') | KeyCode::Left | KeyCode::Char(',' | '<') => turn_for(-1),
            KeyCode::Char('d' | 'D') | KeyCode::Right | KeyCode::Char('.' | '>') => turn_for(1),
            KeyCode::Char('z' | 'Z') => Some(Action::TogglePitch(-FIXED_PITCH_MRAD)),
            KeyCode::Char('x' | 'X') => Some(Action::TogglePitch(FIXED_PITCH_MRAD)),
            KeyCode::Char(' ') => Some(Action::CenterView),
            _ if use_key => Some(Action::InventoryUseSelected),
            KeyCode::Char('g' | 'G') => Some(Action::InventoryDropSelected),
            KeyCode::Char('b' | 'B') => Some(Action::ToggleBackpack),
            KeyCode::Char('l' | 'L') => Some(Action::ToggleFlashlight),
            _ => None,
        },
        GameMode::Sitting => match key.code {
            KeyCode::Char('e' | 'E') => Some(Action::StandUp),
            _ if back => Some(Action::StandUp),
            KeyCode::Char(',' | '<') | KeyCode::Left => Some(Action::TurnFor { dir: -1, hold_ms }),
            KeyCode::Char('.' | '>') | KeyCode::Right => Some(Action::TurnFor { dir: 1, hold_ms }),
            KeyCode::Char('z' | 'Z') => Some(Action::TogglePitch(-FIXED_PITCH_MRAD)),
            KeyCode::Char('x' | 'X') => Some(Action::TogglePitch(FIXED_PITCH_MRAD)),
            KeyCode::Char(' ') => Some(Action::CenterView),
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
            KeyCode::Char('a' | 'A') | KeyCode::Left | KeyCode::Char(',' | '<') => Some(Action::TurnFor { dir: -1, hold_ms }),
            KeyCode::Char('d' | 'D') | KeyCode::Right | KeyCode::Char('.' | '>') => Some(Action::TurnFor { dir: 1, hold_ms }),
            KeyCode::Char('z' | 'Z') => Some(Action::TogglePitch(-FIXED_PITCH_MRAD)),
            KeyCode::Char('x' | 'X') => Some(Action::TogglePitch(FIXED_PITCH_MRAD)),
            KeyCode::Char(' ') => Some(Action::CenterView),
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

/// Hold this long to walk until told otherwise.
pub const LOCK_HOLD_MS: u16 = u16::MAX;
/// Two taps of a movement key within this long lock that direction in.
const DOUBLE_TAP_MS: u64 = 400;

/// Learns how the terminal repeats keys, so a tap moves you one short step while a held
/// key keeps you moving without a stutter: the first press holds long enough to bridge
/// the terminal's repeat delay (once seen), and repeats hold just past their interval.
///
/// It also spots tap sequences: two or more presses close together that aren't followed
/// by repeats. Two of them locks the direction in; three triggers a bigger action (an
/// animated spin, for turn keys).
pub struct KeyRepeat {
    last_code: Option<KeyCode>,
    last_ms: u64,
    /// The terminal's delay before it starts repeating, once measured.
    first_hold_ms: u16,
    /// A run of fresh presses of the same key within [`DOUBLE_TAP_MS`] of each other,
    /// waiting to see whether another tap or a terminal repeat follows.
    seq: Option<TapSeq>,
}

#[derive(Clone, Copy)]
struct TapSeq {
    key: KeyEvent,
    count: u8,
    first_at: u64,
    last_at: u64,
}

impl Default for KeyRepeat {
    fn default() -> Self {
        Self { last_code: None, last_ms: 0, first_hold_ms: FIRST_HOLD_MS, seq: None }
    }
}

impl KeyRepeat {
    /// How long this press of `code` should hold its movement.
    pub fn hold_for(&mut self, key: KeyEvent, now_ms: u64) -> u16 {
        let code = key.code;
        let gap = now_ms.saturating_sub(self.last_ms);
        let same = self.last_code == Some(code);
        self.last_code = Some(code);
        self.last_ms = now_ms;
        if same && gap < 120 {
            // terminal auto-repeat: whatever tap sequence was pending was really the
            // terminal starting to repeat, learn its delay from them.
            if let Some(s) = self.seq.take()
                && s.key.code == code
                && s.count >= 2
            {
                self.first_hold_ms = (s.last_at.saturating_sub(s.first_at) as u16 + 80).clamp(140, 440);
            }
            return REPEAT_HOLD_MS;
        }
        let seq_recent = self.seq.as_ref().is_some_and(|s| s.key.code == code && now_ms.saturating_sub(s.last_at) < DOUBLE_TAP_MS);
        if same && gap < 900 && !seq_recent {
            // the first repeat after a lone press (nothing's pending): learn directly
            self.first_hold_ms = (gap as u16 + 80).clamp(140, 440);
            self.seq = None;
            return REPEAT_HOLD_MS;
        }
        // a fresh press: continue the pending sequence if it's for the same key within
        // the window, otherwise start a new one.
        if seq_recent {
            let s = self.seq.as_mut().unwrap();
            s.count = s.count.saturating_add(1);
            s.last_at = now_ms;
            s.key = key;
        } else {
            self.seq = Some(TapSeq { key, count: 1, first_at: now_ms, last_at: now_ms });
        }
        self.first_hold_ms
    }

    /// A tap sequence that no repeat and no further tap followed: the key and how many
    /// times it was pressed (two or more). Returns at most once per sequence.
    pub fn take_tap_sequence(&mut self, now_ms: u64) -> Option<(KeyEvent, u8)> {
        let s = self.seq?;
        if s.count < 2 || now_ms.saturating_sub(s.last_at) < DOUBLE_TAP_MS {
            return None;
        }
        self.seq = None;
        Some((s.key, s.count))
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
        let run = Some(Action::Move { forward: 1, side: 0, run: true, hold_ms: RUN_HOLD_FLOOR_MS });
        assert_eq!(key_to_action(key(KeyCode::Char('W')), ex, 100), run, "Shift runs with a longer hold so swapping direction keys stays running");
        assert_eq!(key_to_action(key(KeyCode::Char('a')), ex, 100), Some(Action::TurnFor { dir: -1, hold_ms: 100 }), "A turns left on the spot");
        assert_eq!(key_to_action(key(KeyCode::Left), ex, 100), Some(Action::TurnFor { dir: -1, hold_ms: 100 }), "so does the left arrow");
        assert_eq!(key_to_action(key(KeyCode::Char(',')), ex, 100), Some(Action::TurnFor { dir: -1, hold_ms: 100 }), "the comma is an alias for A");
        assert_eq!(key_to_action(key(KeyCode::Char(' ')), ex, 100), Some(Action::CenterView));
        assert_eq!(key_to_action(KeyEvent::new(KeyCode::Up, KeyModifiers::SHIFT), ex, 100), run);
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
        let w = key(KeyCode::Char('w'));
        assert_eq!(r.hold_for(w, 1_000), FIRST_HOLD_MS, "a first press holds the default");
        // the terminal starts repeating 500 ms later: now the first hold bridges that
        assert_eq!(r.hold_for(w, 1_500), REPEAT_HOLD_MS);
        assert_eq!(r.hold_for(w, 1_533), REPEAT_HOLD_MS, "repeats hold just past the next one");
        assert_eq!(r.hold_for(w, 5_000), 440, "a fresh press holds the learned delay plus a margin");
        assert_eq!(r.hold_for(key(KeyCode::Char('a')), 5_010), 440, "another key is its own first press");
        assert!(r.take_tap_sequence(5_200).is_none());
    }

    #[test]
    fn a_double_tap_locks_the_walk_but_a_repeat_start_doesnt() {
        let mut r = KeyRepeat::default();
        let w = key(KeyCode::Char('w'));
        // tap, tap: nothing follows, so it's a double tap. Settlement waits one tap
        // window to make sure a third tap isn't on its way.
        r.hold_for(w, 1_000);
        r.hold_for(w, 1_250);
        assert!(r.take_tap_sequence(1_300).is_none(), "not decided until no further tap could follow");
        assert!(r.take_tap_sequence(1_500).is_none(), "still within the tap window");
        let (reported, count) = r.take_tap_sequence(1_700).unwrap();
        assert_eq!(reported.code, KeyCode::Char('w'));
        assert_eq!(count, 2);
        assert!(r.take_tap_sequence(1_800).is_none(), "reported once");
        // triple tap: three fresh presses in a row, no repeat follows
        let mut r = KeyRepeat::default();
        r.hold_for(w, 3_000);
        r.hold_for(w, 3_250);
        r.hold_for(w, 3_500);
        let (_, count) = r.take_tap_sequence(4_000).unwrap();
        assert_eq!(count, 3, "three taps are a triple tap");
        // press, then the terminal's first repeat 250 ms later, then more: not a double tap
        let mut r = KeyRepeat::default();
        r.hold_for(w, 2_000);
        r.hold_for(w, 2_250);
        r.hold_for(w, 2_283);
        assert!(r.take_tap_sequence(2_500).is_none());
        assert_eq!(r.hold_for(w, 9_000), 330, "and the repeat delay was learned from it");
    }
}
