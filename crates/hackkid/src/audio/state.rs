use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// What the drone needs to know about the game, shared with the audio thread.
pub struct AudioState {
    pub mute: AtomicBool,
    pub hour_x100: AtomicU32,
    pub indoor: AtomicBool,
    pub body: AtomicU32,
    pub thirst: AtomicU32,
    pub sleep: AtomicU32,
    pub dialog: AtomicBool,
    /// A transmission is playing: the drone steps back.
    pub music: AtomicBool,
}

impl Default for AudioState {
    fn default() -> Self {
        Self {
            mute: AtomicBool::new(false),
            hour_x100: AtomicU32::new(1200),
            indoor: AtomicBool::new(true),
            body: AtomicU32::new(80),
            thirst: AtomicU32::new(25),
            sleep: AtomicU32::new(20),
            dialog: AtomicBool::new(false),
            music: AtomicBool::new(false),
        }
    }
}

impl AudioState {
    pub fn toggle_mute(&self) -> bool {
        !self.mute.fetch_xor(true, Ordering::Relaxed)
    }
}
