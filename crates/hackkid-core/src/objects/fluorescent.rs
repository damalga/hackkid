#[derive(Debug, Clone, Copy)]
pub enum FluorescentAxis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy)]
pub enum FluorescentState {
    Steady,
    Flicker { period_ms: u64, duty: f32 },
    Dead,
}

#[derive(Debug, Clone)]
pub struct Fluorescent {
    pub cx: f64,
    pub cy: f64,
    pub half_len: f64,
    pub axis: FluorescentAxis,
    pub state: FluorescentState,
    pub color: (u8, u8, u8),
}

impl Fluorescent {
    pub fn new(cx: f64, cy: f64, half_len: f64, axis: FluorescentAxis) -> Self {
        Self {
            cx, cy, half_len, axis,
            state: FluorescentState::Steady,
            color: (255, 250, 220),
        }
    }

    pub fn with_state(mut self, state: FluorescentState) -> Self {
        self.state = state;
        self
    }

    pub fn endpoints(&self) -> ((f64, f64), (f64, f64)) {
        match self.axis {
            FluorescentAxis::Horizontal => (
                (self.cx - self.half_len, self.cy),
                (self.cx + self.half_len, self.cy),
            ),
            FluorescentAxis::Vertical => (
                (self.cx, self.cy - self.half_len),
                (self.cx, self.cy + self.half_len),
            ),
        }
    }

    pub fn brightness(&self, elapsed_ms: u64) -> f32 {
        match self.state {
            FluorescentState::Steady => 1.0,
            FluorescentState::Dead => 0.0,
            FluorescentState::Flicker { period_ms, duty } => {
                let seed = ((self.cx * 1000.0) as u64).wrapping_add((self.cy * 100.0) as u64);
                let phase = (elapsed_ms.wrapping_add(seed) % period_ms) as f32 / period_ms as f32;
                if phase < duty {
                    let jitter = (phase * 40.0).sin() * 0.3 + 0.7;
                    jitter.clamp(0.2, 1.0)
                } else {
                    0.05
                }
            }
        }
    }
}
