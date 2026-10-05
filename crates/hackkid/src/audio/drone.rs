use std::num::NonZero;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use rodio::{ChannelCount, SampleRate, Source};

use crate::audio::state::AudioState;

const TWO_PI: f64 = std::f64::consts::PI * 2.0;

pub struct DroneSource {
    state: Arc<AudioState>,
    sample_rate: u32,
    phase_fund: f64,
    phase_fifth: f64,
    phase_sub: f64,
    phase_lfo: f64,
    phase_pulse: f64,
    filter_y: f32,
    cutoff_smooth: f32,
    vol_smooth: f32,
    detune_smooth: f32,
    sample_idx: u64,
    cache_period: u64,
    cached_hour: f32,
    cached_indoor: bool,
    cached_body: f32,
    cached_thirst: f32,
    cached_sleep: f32,
    cached_dialog: bool,
    cached_mute: bool,
    cached_music: bool,
}

impl DroneSource {
    pub fn new(state: Arc<AudioState>, sample_rate: u32) -> Self {
        Self {
            state,
            sample_rate,
            phase_fund: 0.0,
            phase_fifth: 0.0,
            phase_sub: 0.0,
            phase_lfo: 0.0,
            phase_pulse: 0.0,
            filter_y: 0.0,
            cutoff_smooth: 1500.0,
            vol_smooth: 0.0,
            detune_smooth: 0.0,
            sample_idx: 0,
            cache_period: (sample_rate as u64) / 20, // refresh 20x per second
            cached_hour: 12.0,
            cached_indoor: true,
            cached_body: 80.0,
            cached_thirst: 25.0,
            cached_sleep: 20.0,
            cached_dialog: false,
            cached_mute: false,
            cached_music: false,
        }
    }

    fn refresh_cache(&mut self) {
        self.cached_hour = self.state.hour_x100.load(Ordering::Relaxed) as f32 / 100.0;
        self.cached_indoor = self.state.indoor.load(Ordering::Relaxed);
        self.cached_body = self.state.body.load(Ordering::Relaxed) as f32;
        self.cached_thirst = self.state.thirst.load(Ordering::Relaxed) as f32;
        self.cached_sleep = self.state.sleep.load(Ordering::Relaxed) as f32;
        self.cached_dialog = self.state.dialog.load(Ordering::Relaxed);
        self.cached_mute = self.state.mute.load(Ordering::Relaxed);
        self.cached_music = self.state.music.load(Ordering::Relaxed);
    }
}

impl Iterator for DroneSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.sample_idx.is_multiple_of(self.cache_period) {
            self.refresh_cache();
        }
        self.sample_idx = self.sample_idx.wrapping_add(1);

        let sr = self.sample_rate as f64;

        // Night detune: fundamental drifts down between 22h and 06h.
        let hour = self.cached_hour;
        let is_night = hour < 6.0 || hour >= 22.0;
        let target_detune: f32 = if is_night { -2.5 } else { 0.5 };
        self.detune_smooth += (target_detune - self.detune_smooth) * 0.0004;

        // Anxiety pulse when body low or thirst high.
        let anxiety = ((20.0 - self.cached_body).max(0.0) / 20.0
            + (self.cached_thirst - 80.0).max(0.0) / 20.0
            + (self.cached_sleep - 80.0).max(0.0) / 20.0)
            .min(1.5);

        // Filter cutoff: indoor darker, outdoor open.
        let target_cutoff: f32 = if self.cached_indoor { 900.0 } else { 3200.0 };
        self.cutoff_smooth += (target_cutoff - self.cutoff_smooth) * 0.0006;

        // Volume: dip under a transmission or dialog, silent when muted.
        let target_vol: f32 = if self.cached_mute {
            0.0
        } else if self.cached_music {
            0.03
        } else if self.cached_dialog {
            0.06
        } else {
            0.14
        };
        self.vol_smooth += (target_vol - self.vol_smooth) * 0.0009;

        // Oscillators
        let fund_hz = 55.0 + self.detune_smooth as f64;
        let fifth_hz = fund_hz * 1.5;
        let sub_hz = fund_hz * 0.5;

        let lfo = (self.phase_lfo).sin();
        let pulse = (self.phase_pulse).sin();

        // LFO slow amplitude on fifth
        let fifth_amp = 0.35 + 0.20 * lfo as f32;
        // Sub anxiety pulse: 1.2 Hz tremolo scaled by anxiety
        let sub_amp = 0.45 * (1.0 - 0.5 * anxiety)
            + 0.30 * pulse as f32 * anxiety;

        let s_fund = self.phase_fund.sin() as f32 * 0.55;
        let s_fifth = self.phase_fifth.sin() as f32 * fifth_amp;
        let s_sub = self.phase_sub.sin() as f32 * sub_amp;

        let raw = (s_fund + s_fifth + s_sub) * 0.5;

        // Advance phases
        self.phase_fund = (self.phase_fund + TWO_PI * fund_hz / sr) % TWO_PI;
        self.phase_fifth = (self.phase_fifth + TWO_PI * fifth_hz / sr) % TWO_PI;
        self.phase_sub = (self.phase_sub + TWO_PI * sub_hz / sr) % TWO_PI;
        self.phase_lfo = (self.phase_lfo + TWO_PI * 0.05 / sr) % TWO_PI;
        self.phase_pulse = (self.phase_pulse + TWO_PI * 1.2 / sr) % TWO_PI;

        // One-pole low-pass filter
        let cutoff = self.cutoff_smooth.max(60.0);
        let dt = 1.0 / self.sample_rate as f32;
        let rc = 1.0 / (TWO_PI as f32 * cutoff);
        let alpha = dt / (rc + dt);
        self.filter_y += alpha * (raw - self.filter_y);

        let out = self.filter_y * self.vol_smooth;
        Some(out.clamp(-0.98, 0.98))
    }
}

impl Source for DroneSource {
    fn current_span_len(&self) -> Option<usize> { None }
    fn channels(&self) -> ChannelCount { NonZero::<u16>::MIN }
    fn sample_rate(&self) -> SampleRate { NonZero::new(self.sample_rate).unwrap_or(NonZero::<u32>::MIN) }
    fn total_duration(&self) -> Option<Duration> { None }
}
