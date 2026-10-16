//! The two control schemes, switched from the menu's Controls page.
//!
//! **Mouse look** (the default): the pointer and two fingers turn and tilt the view,
//! A and D strafe. **Keys**: A and D turn the view in steps of 22.5°, so two taps are
//! 45°, Z and X look down and up, and the pointer leaves the view alone. The key list
//! the menu shows for each scheme is here too, so every frontend shows the same one.

use serde::{Deserialize, Serialize};

/// One tap of A, D or an arrow in the Keys scheme turns this much: 22.5°, two taps 45°.
pub const TURN_STEP: f64 = std::f64::consts::FRAC_PI_8;
/// The Controls page has two rows: the scheme, and Back.
pub const CONTROLS_ITEMS: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ControlScheme {
    #[default]
    Mouse,
    Keys,
}

impl ControlScheme {
    pub fn toggled(self) -> Self {
        match self {
            ControlScheme::Mouse => ControlScheme::Keys,
            ControlScheme::Keys => ControlScheme::Mouse,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            ControlScheme::Mouse => "Mouse look",
            ControlScheme::Keys => "Keys",
        }
    }

    /// One line on what the scheme does with A and D.
    pub fn summary(self) -> &'static str {
        match self {
            ControlScheme::Mouse => "the pointer turns and tilts the view, A and D strafe",
            ControlScheme::Keys => "A and D turn the view 22.5° a tap, Z and X look down and up",
        }
    }

    /// The keys, for the menu's Controls page.
    pub fn bindings(self) -> &'static [(&'static str, &'static str)] {
        const SHARED: [(&str, &str); 6] = [
            ("Shift + move", "Run"),
            ("E / F", "Use / second action"),
            ("C / L / Q", "Crouch / flashlight / drink from the flask"),
            ("Tab, T, M", "Bag, emergency terminal, map"),
            ("Esc", "This menu"),
            ("F5 / F9", "Quick save / quick load"),
        ];
        const MOUSE: [(&str, &str); 11] = [
            ("W / S, ↑ / ↓", "Walk forward / back (double-tap to keep walking)"),
            ("A / D, ← / →", "Strafe left / right"),
            ("Pointer", "Look around; park it at an edge to keep turning"),
            ("Two fingers, wheel", "Sideways turns, up / down tilts"),
            (", / . · Z / X · Space", "Turn · look down / up 45° · level the view"),
            SHARED[0],
            SHARED[1],
            SHARED[2],
            SHARED[3],
            SHARED[4],
            SHARED[5],
        ];
        const KEYS: [(&str, &str); 10] = [
            ("W / S, ↑ / ↓", "Walk forward / back (double-tap to keep walking)"),
            ("A / D, ← / →", "Turn 22.5° a tap, two taps 45°; hold to keep turning"),
            ("Z / X · Space", "Look down / up 45° · level the view"),
            ("Two fingers, wheel", "Sideways turns; up / down steps forward / back"),
            SHARED[0],
            SHARED[1],
            SHARED[2],
            SHARED[3],
            SHARED[4],
            SHARED[5],
        ];
        match self {
            ControlScheme::Mouse => &MOUSE,
            ControlScheme::Keys => &KEYS,
        }
    }
}

/// Which page the menu (title or pause) is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MenuPage {
    #[default]
    Main,
    Controls,
}
