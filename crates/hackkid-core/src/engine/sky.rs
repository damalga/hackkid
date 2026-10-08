//! Time of day: the sky, the sun and the light it gives.

use crate::engine::textures::hash;

/// Light and weather for one frame.
#[derive(Debug, Clone, Copy)]
pub struct Env {
    pub hour: f64,
    pub cloudy: bool,
    /// Drives the drifting clouds.
    pub t_ms: u64,
}

fn smooth(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

impl Env {
    /// Height of the sun, -1 (midnight) .. 1 (noon). Sunrise at 6:00, sunset at 20:00.
    pub fn sun_altitude(&self) -> f64 {
        let h = self.hour.rem_euclid(24.0);
        ((h - 6.0) / 14.0 * std::f64::consts::PI).sin().clamp(-1.0, 1.0) * if (6.0..20.0).contains(&h) { 1.0 } else { 0.6 }
    }

    /// 0 at night, 1 in full day.
    pub fn day(&self) -> f32 {
        smooth(-0.12, 0.3, self.sun_altitude()) as f32
    }

    /// How golden the light is: high around sunrise and sunset.
    fn golden(&self) -> f32 {
        let a = self.sun_altitude();
        (smooth(-0.15, 0.05, a) * (1.0 - smooth(0.12, 0.42, a))) as f32
    }

    /// Light falling on open ground (and through windows), as an RGB multiplier.
    pub fn daylight(&self) -> [f32; 3] {
        let night = [0.15, 0.17, 0.27];
        let noon = [1.18, 1.16, 1.10];
        let mut c = lerp3(night, noon, self.day());
        c = lerp3(c, [1.15, 0.78, 0.52], self.golden() * 0.7);
        if self.cloudy {
            let l = (c[0] + c[1] + c[2]) / 3.0 * 0.78;
            c = lerp3(c, [l, l * 1.02, l * 1.06], 0.75);
        }
        c
    }

    /// Street lamps come on as the light goes.
    pub fn lamps_on(&self) -> f32 {
        1.0 - smooth(0.05, 0.35, self.sun_altitude()) as f32
    }

    /// Direction of the sun in the floor plane (x east, y south).
    fn sun_dir(&self) -> (f64, f64) {
        let h = self.hour.rem_euclid(24.0);
        let a = (h - 6.0) / 14.0 * std::f64::consts::PI; // east at dawn, south at noon, west at dusk
        (a.cos(), a.sin())
    }

    /// The sky colour looking towards `dir` (unit, floor plane), `elev` radians above the horizon.
    pub fn sky(&self, dir: (f64, f64), elev: f64) -> [f32; 3] {
        let up = (elev / 1.2).clamp(0.0, 1.0) as f32;
        let day = self.day();
        let golden = self.golden();
        let zenith = lerp3([6.0, 10.0, 26.0], [62.0, 112.0, 186.0], day);
        let horizon = lerp3(lerp3([18.0, 24.0, 44.0], [176.0, 204.0, 228.0], day), [236.0, 150.0, 96.0], golden);
        let mut c = lerp3(horizon, zenith, up.powf(0.6));

        // the sun (or the moon at night) and its glow
        let (sx, sy) = self.sun_dir();
        let alt = self.sun_altitude();
        let (bx, by, b_alt) = if alt > -0.1 { (sx, sy, alt * 1.1) } else { (-sx, -sy, 0.5) };
        let along = dir.0 * bx + dir.1 * by;
        if along > 0.0 {
            let dh = (1.0 - along).max(0.0).sqrt() * 1.4;
            let dv = elev - b_alt;
            let d = (dh * dh + dv * dv).sqrt();
            let is_sun = alt > -0.1;
            let disc = if is_sun { 0.07 } else { 0.045 };
            if d < disc && !self.cloudy {
                return if is_sun { [255.0, 246.0, 214.0] } else { [214.0, 220.0, 228.0] };
            }
            if is_sun {
                let glow = ((1.0 - d / 0.7).max(0.0) as f32).powi(2) * if self.cloudy { 0.25 } else { 0.55 };
                c = lerp3(c, [255.0, 226.0, 170.0], glow);
            }
        }

        // stars, when it's dark and clear
        if day < 0.3 && !self.cloudy {
            let az = dir.1.atan2(dir.0);
            let (ix, iy) = ((az * 60.0) as i32, (elev * 60.0) as i32);
            if hash(ix, iy, 401) > 0.985 {
                let tw = 0.6 + 0.4 * hash(ix, iy + (self.t_ms / 700) as i32, 409);
                return lerp3(c, [230.0, 232.0, 240.0], (1.0 - day / 0.3) * tw);
            }
        }

        // clouds drifting slowly from the west
        let az = dir.1.atan2(dir.0);
        let drift = self.t_ms as f64 / 90_000.0;
        let cx = az * 3.0 + drift;
        let cy = (1.0 / (elev + 0.12)) * 1.6;
        let n = cloud_noise(cx, cy) * 0.65 + cloud_noise(cx * 2.3 + 7.0, cy * 2.3) * 0.35;
        let cover = if self.cloudy { 0.18 } else { 0.58 };
        let k = (((n - cover) / 0.25).clamp(0.0, 1.0) as f32) * (1.0 - up * 0.3);
        let lit = lerp3([40.0, 44.0, 56.0], [236.0, 236.0, 232.0], day);
        let cloud = lerp3(lit, [244.0, 170.0, 130.0], golden * 0.6);
        c = lerp3(c, cloud, k * if self.cloudy { 0.95 } else { 0.85 });
        if self.cloudy {
            let grey = lerp3([20.0, 22.0, 28.0], [150.0, 156.0, 164.0], day);
            c = lerp3(c, grey, 0.55);
        }
        c
    }
}

fn cloud_noise(x: f64, y: f64) -> f64 {
    let (ix, iy) = (x.floor() as i32, y.floor() as i32);
    let (tx, ty) = (x - ix as f64, y - iy as f64);
    let (sx, sy) = (tx * tx * (3.0 - 2.0 * tx), ty * ty * (3.0 - 2.0 * ty));
    let h = |a: i32, b: i32| hash(a, b, 419) as f64;
    let a = h(ix, iy) + (h(ix + 1, iy) - h(ix, iy)) * sx;
    let b = h(ix, iy + 1) + (h(ix + 1, iy + 1) - h(ix, iy + 1)) * sx;
    a + (b - a) * sy
}
