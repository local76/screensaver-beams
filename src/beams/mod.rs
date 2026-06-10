//! Consolidated beams screensaver effect module.

mod types;
mod physics;

pub use types::{Spotlight, Star, DustParticle, default_spotlights};

use library::core::{LcgRng, TerminalCell};
use std::time::Duration;
use library::core::screensaver::Screensaver;

use library::platform::native::sys_info::get_system_info;
use library::toolkit::sys_info::query_current_palette;


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
        let cpu_load = 0.4;

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
        }
    }
}

impl Screensaver for Beams {
    fn update(&mut self, dt: Duration, cols: usize, rows: usize) {
        let delta = dt.as_secs_f32();
        self.time_elapsed += delta;

        self.sys_refresh_timer += delta;
        if self.sys_refresh_timer >= 1.0 {
            let sys = get_system_info();
            self.mem_pressure = sys.mem_used_pct / 100.0;
            self.cpu_load = (self.mem_pressure * 0.6 + 0.3).min(1.0);
            self.sys_refresh_timer = 0.0;

            let biased_load = self.cpu_load + (self.host_bias - 0.5) * 0.15;
            for spot in &mut self.spotlights {
                let load_factor = 1.0 + biased_load * 0.7 + self.mem_pressure * 0.5;
                spot.speed = (spot.speed * 0.85 + (0.6 + load_factor * 0.5) * 0.15).clamp(0.3, 2.8);
                spot.spread = (0.11 + self.mem_pressure * 0.09 + biased_load * 0.04).clamp(0.08, 0.28);
            }
        }

        if cols != self.last_cols || rows != self.last_rows {
            self.stars.clear();
            self.particles.clear();

            let area = cols * rows;
            let target_stars = if self.twinkle_stars_opt == 1 { (area / 16).clamp(30, 200) } else { 0 };
            for i in 0..target_stars {
                self.stars.push(Star {
                    x: self.rng.next_f32(),
                    y: self.rng.next_f32(),
                    phase: self.rng.next_f32() * std::f32::consts::TAU,
                    ch: if i % 8 == 0 { '\u{2726}' } else if i % 3 == 0 { '\u{2022}' } else { '.' },
                    excitation: 0.0,
                });
            }

            let target_particles = (area / 12).clamp(30, 150);
            for _ in 0..target_particles {
                self.particles.push(DustParticle {
                    x: self.rng.next_f32(),
                    y: self.rng.next_f32(),
                    vx: self.rng.next_range(-0.04, 0.04),
                    vy: -self.rng.next_range(0.05, 0.12),
                });
            }

            self.last_cols = cols;
            self.last_rows = rows;
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
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beams_new() {
        let b = Beams::new();
        assert!(!b.spotlights.is_empty());
        assert_eq!(b.last_cols, 0);
        assert_eq!(b.last_rows, 0);
    }

    #[test]
    fn test_beams_init() {
        let mut b = Beams::new();
        b.update(Duration::from_millis(16), 80, 24);
        assert_eq!(b.last_cols, 80);
        assert_eq!(b.last_rows, 24);
        assert!(!b.stars.is_empty());
        assert!(!b.particles.is_empty());
    }

    #[test]
    fn test_beams_update_and_draw() {
        let mut b = Beams::new();
        b.init(80, 24);
        b.update(Duration::from_millis(16), 80, 24);
        let mut grid = vec![TerminalCell::default(); 80 * 24];
        b.draw(&mut grid, 80, 24);
        let drawn_count = grid.iter().filter(|c| c.ch != '\0').count();
        assert!(drawn_count > 0, "No beams/particles drawn in the grid");
    }
}
