use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

pub struct AudioState {
    pub mute: AtomicBool,
    pub hour_x100: AtomicU32,
    pub indoor: AtomicBool,
    pub body: AtomicU32,
    pub thirst: AtomicU32,
    pub sleep: AtomicU32,
    pub dialog: AtomicBool,
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
        }
    }
}

impl AudioState {
    pub fn write(&self, hour: f64, indoor: bool, body: f64, thirst: f64, sleep: f64, dialog: bool) {
        self.hour_x100.store((hour * 100.0).clamp(0.0, 2400.0) as u32, Ordering::Relaxed);
        self.indoor.store(indoor, Ordering::Relaxed);
        self.body.store(body.clamp(0.0, 100.0) as u32, Ordering::Relaxed);
        self.thirst.store(thirst.clamp(0.0, 100.0) as u32, Ordering::Relaxed);
        self.sleep.store(sleep.clamp(0.0, 100.0) as u32, Ordering::Relaxed);
        self.dialog.store(dialog, Ordering::Relaxed);
    }

    pub fn toggle_mute(&self) -> bool {
        let cur = self.mute.load(Ordering::Relaxed);
        self.mute.store(!cur, Ordering::Relaxed);
        !cur
    }

    pub fn is_muted(&self) -> bool {
        self.mute.load(Ordering::Relaxed)
    }
}
