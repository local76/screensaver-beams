//! Consolidated beams screensaver effect module.

mod types;
mod physics;
mod light;
mod physics_star;

pub use types::{Spotlight, Star, DustParticle, default_spotlights};

use crate::runner::core::{LcgRng, TerminalCell};
use std::time::Duration;
use crate::runner::core::screensaver::Screensaver;

use crate::runner::toolkit::sys_info::get_system_info;

pub struct Beams {
    pub(super) rng: LcgRng,
    pub(super) stars: Vec<Star>,
    pub(super) particles: Vec<DustParticle>,
    pub(super) spotlights: Vec<Spotlight>,
    pub(super) time_elapsed: f32,
    pub(super) last_cols: usize,
    pub(super) last_rows: usize,
    pub(super) twinkle_stars_opt: u32,

    // Live system dynamics
    pub(super) sys_refresh_timer: f32,
    pub(super) mem_pressure: f32,
    pub(super) cpu_load: f32,
    pub(super) host_bias: f32,
    pub(super) on_battery: bool,
    pub(super) frame_time_ema: f32,
    pub(super) quality_scale: f32,
    pub(super) target_frame_time: f32,
    pub(super) logo_text: String,
}

impl Default for Beams {
    fn default() -> Self {
        Self::new()
    }
}

impl Beams {
    pub fn new() -> Self {
        let beam_count: u32 = 4;
        let twinkle_stars_opt: u32 = 1;

        let sys = get_system_info();
        let host_bias = sys.hostname.chars().map(|c| c as u32).sum::<u32>() as f32 / 1000.0 % 1.0;
        let mem_pressure = sys.mem_used_pct / 100.0;
        let cpu_load = (sys.cpu_usage_pct / 100.0).clamp(0.0, 1.0);
        let on_battery = sys.power_status.contains("Battery");
        let logo_text = sys.logo_text.clone();

        let all_spots = default_spotlights();

        let mut spotlights = Vec::new();
        for i in 0..(beam_count as usize) {
            if i < all_spots.len() {
                spotlights.push(all_spots[i].clone());
            }
        }

        Self {
            rng: LcgRng::new(5678),
            stars: Vec::new(),
            particles: Vec::new(),
            spotlights,
            time_elapsed: 0.0,
            last_cols: 0,
            last_rows: 0,
            twinkle_stars_opt,
            sys_refresh_timer: 0.0,
            mem_pressure,
            cpu_load,
            host_bias,
            on_battery,
            frame_time_ema: 0.01666667,
            quality_scale: 1.0,
            target_frame_time: 0.01666667,
            logo_text,
        }
    }
}

impl Screensaver for Beams {
    fn update(&mut self, dt: Duration, cols: usize, rows: usize) {
        let dt_secs = dt.as_secs_f32();

        // Auto-detect high refresh rates during the startup phase
        if self.time_elapsed < 2.0 && dt_secs > 0.001 {
            if dt_secs < self.target_frame_time - 0.001 {
                self.target_frame_time = dt_secs;
            }
        }

        // Exponential moving average for frame time (alpha = 0.1)
        self.frame_time_ema = self.frame_time_ema * 0.9 + dt_secs.min(0.2) * 0.1;

        let speed_mult = if self.on_battery { 0.65 } else { 1.0 };
        let delta = dt_secs * speed_mult;
        self.time_elapsed += delta;

        // Adjust quality_scale based on frame time performance vs target
        if self.time_elapsed > 1.5 {
            if self.frame_time_ema > self.target_frame_time * 1.15 {
                self.quality_scale = (self.quality_scale - 0.15 * delta).max(0.20);
            } else if self.frame_time_ema < self.target_frame_time * 1.05 {
                self.quality_scale = (self.quality_scale + 0.04 * delta).min(1.0);
            }
        }

        self.sys_refresh_timer += delta;
        if self.sys_refresh_timer >= 1.0 {
            let sys = get_system_info();
            self.mem_pressure = sys.mem_used_pct / 100.0;
            self.cpu_load = (sys.cpu_usage_pct / 100.0).clamp(0.0, 1.0);
            self.on_battery = sys.power_status.contains("Battery");
            self.logo_text = sys.logo_text.clone();
            self.sys_refresh_timer = 0.0;

            let biased_load = self.cpu_load + (self.host_bias - 0.5) * 0.15;
            for spot in &mut self.spotlights {
                let load_factor = 1.0 + biased_load * 0.7 + self.mem_pressure * 0.5;
                spot.speed = (spot.speed * 0.85 + (0.6 + load_factor * 0.5) * 0.15).clamp(0.3, 2.8);
                spot.spread = (0.11 + self.mem_pressure * 0.09 + biased_load * 0.04).clamp(0.08, 0.28);
            }
        }

        // Handle resize or dynamic target count adjustments based on quality scale & battery status
        let max_stars = ((if self.twinkle_stars_opt == 1 { (cols * rows / 16).clamp(30, 200) } else { 0 }) as f32 * self.quality_scale * (if self.on_battery { 0.55 } else { 1.0 })) as usize;
        let max_particles = (((cols * rows / 12).clamp(30, 150)) as f32 * self.quality_scale * (if self.on_battery { 0.55 } else { 1.0 })) as usize;

        if cols != self.last_cols || rows != self.last_rows {
            self.stars.clear();
            self.particles.clear();
            self.last_cols = cols;
            self.last_rows = rows;
        }

        // Dynamically adjust star population to match target capacity
        if self.stars.len() > max_stars {
            self.stars.truncate(max_stars);
        } else if self.stars.len() < max_stars && max_stars > 0 {
            while self.stars.len() < max_stars {
                self.stars.push(Star {
                    x: self.rng.next_f32(),
                    y: self.rng.next_f32(),
                    phase: self.rng.next_f32() * std::f32::consts::TAU,
                    ch: if self.stars.len() % 8 == 0 { '\u{2726}' } else if self.stars.len() % 3 == 0 { '\u{2022}' } else { '.' },
                    excitation: 0.0,
                });
            }
        }

        // Dynamically adjust particle population to match target capacity
        if self.particles.len() > max_particles {
            self.particles.truncate(max_particles);
        } else if self.particles.len() < max_particles && max_particles > 0 {
            while self.particles.len() < max_particles {
                self.particles.push(DustParticle {
                    x: self.rng.next_f32(),
                    y: self.rng.next_f32(),
                    vx: self.rng.next_range(-0.04, 0.04),
                    vy: -self.rng.next_range(0.05, 0.12),
                });
            }
        }

        for star in &mut self.stars {
            if star.excitation > 0.0 {
                star.excitation -= delta * 2.0;
                if star.excitation < 0.0 {
                    star.excitation = 0.0;
                }
            }
        }

        let cols_f = cols as f32;
        let rows_f = rows as f32;
        for p in &mut self.particles {
            p.x += p.vx * delta;
            p.y += p.vy * delta;

            if p.y < 0.0 {
                p.y = 1.0;
                p.x = self.rng.next_f32();
            }
            if p.x < 0.0 {
                p.x = 1.0;
            }
            if p.x > 1.0 {
                p.x = 0.0;
            }

            for star in &mut self.stars {
                let dx = (p.x - star.x) * cols_f;
                let dy = (p.y - star.y) * rows_f * 2.0;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq < 6.25 {
                    let dist = dist_sq.sqrt();
                    let force = (1.0 - dist / 2.5) * 1.5;
                    star.excitation = star.excitation.max(force);
                }
            }
        }

        for spot in &mut self.spotlights {
            spot.phase += spot.speed * delta;
        }
    }

    fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        physics::draw_impl(
            grid,
            cols,
            rows,
            &self.spotlights,
            &self.stars,
            &self.particles,
            self.twinkle_stars_opt,
            self.time_elapsed,
            &self.logo_text,
        );
    }
}

#[cfg(test)]
#[path = "beams_tests.rs"]
mod tests;
