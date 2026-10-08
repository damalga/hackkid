//! What the emergency terminal plays when the album isn't installed: a synthesized
//! transmission, different for each track. A soft minor chord under drifting radio
//! static, with the track number tapped out in Morse every few seconds.

use std::num::NonZero;
use std::time::Duration;

use rodio::{ChannelCount, SampleRate, Source};

const RATE: u32 = 44_100;
const SECONDS: u64 = 75;
const TAU: f64 = std::f64::consts::TAU;

pub struct Carrier {
    t: u64,
    len: u64,
    chord: [f64; 3],
    phases: [f64; 4],
    noise: u32,
    lp: f32,
    lp2: f32,
    /// Morse for the track number, one entry per 90 ms dit: on or off.
    morse: Vec<bool>,
}

impl Carrier {
    /// The transmission for track number `n` (1-based).
    pub fn new(n: u8) -> Self {
        // roots walk round a minor pentatonic scale, from about A2 up
        let steps = [0.0, 3.0, 5.0, 7.0, 10.0];
        let semis = steps[(n as usize - 1) % 5] + 12.0 * (((n as usize - 1) / 5) as f64 * 0.5).floor();
        let root = 110.0 * 2f64.powf(semis / 12.0);
        let mut morse = Vec::new();
        for digit in format!("{n:02}").chars() {
            let d = digit.to_digit(10).unwrap_or(0) as usize;
            // 1 → .----, 6 → -...., 0 → -----
            for i in 0..5 {
                let dot = if d <= 5 { i < d } else { i >= d - 5 };
                let len = if dot && d != 0 { 1 } else { 3 };
                morse.extend(std::iter::repeat_n(true, len));
                morse.push(false);
            }
            morse.extend([false; 3]);
        }
        morse.extend([false; 40]);
        Self {
            t: 0,
            len: SECONDS * RATE as u64,
            chord: [root, root * 1.189, root * 1.498],
            phases: [0.0; 4],
            noise: 0x9E37_79B9 ^ n as u32,
            lp: 0.0,
            lp2: 0.0,
            morse,
        }
    }

    fn white(&mut self) -> f32 {
        self.noise ^= self.noise << 13;
        self.noise ^= self.noise >> 17;
        self.noise ^= self.noise << 5;
        (self.noise as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

impl Iterator for Carrier {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.t >= self.len {
            return None;
        }
        let secs = self.t as f64 / RATE as f64;
        self.t += 1;

        // the chord breathes slowly
        let mut pad = 0.0;
        for (i, f) in self.chord.into_iter().enumerate() {
            self.phases[i] = (self.phases[i] + TAU * f / RATE as f64) % TAU;
            let swell = 0.6 + 0.4 * (secs * 0.11 + i as f64 * 2.1).sin();
            pad += self.phases[i].sin() * 0.05 * swell;
        }

        // band-limited static that fades in and out like a weak station, with crackle
        let n = self.white();
        self.lp += 0.25 * (n - self.lp);
        self.lp2 += 0.02 * (self.lp - self.lp2);
        let fading = 0.5 + 0.5 * (secs * 0.37).sin() * (secs * 0.13).cos();
        let mut hiss = (self.lp - self.lp2) * 0.09 * fading as f32;
        if self.white() > 0.9997 {
            hiss += self.white() * 0.25;
        }

        // the call sign: the track number in Morse, at 700 Hz
        let slot = ((secs / 0.09) as usize) % self.morse.len();
        self.phases[3] = (self.phases[3] + TAU * 700.0 / RATE as f64) % TAU;
        let beep = if self.morse[slot] { self.phases[3].sin() * 0.06 } else { 0.0 };

        let fade = (secs / 2.0).min(1.0).min((SECONDS as f64 - secs) / 3.0).max(0.0);
        Some(((pad + beep) * fade) as f32 + hiss * fade as f32)
    }
}

impl Source for Carrier {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> ChannelCount {
        NonZero::<u16>::MIN
    }

    fn sample_rate(&self) -> SampleRate {
        NonZero::new(RATE).unwrap_or(NonZero::<u32>::MIN)
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs(SECONDS))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_track_has_its_own_short_transmission() {
        for n in 1..=13u8 {
            let c = Carrier::new(n);
            assert!(c.morse.iter().any(|&on| on), "track {n} taps out its number");
            let samples: Vec<f32> = Carrier::new(n).take(RATE as usize * 3).collect();
            assert!(samples.iter().all(|s| s.abs() <= 1.0));
            assert!(samples.iter().any(|s| s.abs() > 0.01), "track {n} isn't silent");
        }
        assert_eq!(Carrier::new(7).count(), (SECONDS * RATE as u64) as usize, "it ends, so the next track starts");
        assert_ne!(Carrier::new(1).chord, Carrier::new(2).chord);
    }
}
