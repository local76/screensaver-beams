//! Consolidated beams screensaver effect module.
//!
//! **Taxonomy Classification**: System Role (Purpose - Application Software).


use library::core::{LcgRng, TerminalCell};
use std::time::Duration;
use library::core::screensaver::Screensaver;

use library::platform::native::sys_info::get_system_info;
use library::toolkit::sys_info::query_current_palette;

use library::toolkit::rgb_controller::{RgbController, is_openrgb_enabled};

use library::toolkit::rgb_protocol::RgbColor;
use library::core::logo_block::render_logo_block;

#[derive(Clone)]
pub struct Spotlight {
    pub origin_x_ratio: f32,
    pub angle_center: f32,
    pub angle_amplitude: f32,
    pub speed: f32,
    pub phase_offset: f32,
    pub phase: f32,
    pub spread: f32,
    pub color_r: f32,
    pub color_g: f32,
    pub color_b: f32,
}

pub struct Star {
    pub x: f32,
    pub y: f32,
    pub phase: f32,
    pub ch: char,
    pub excitation: f32,
}

pub struct DustParticle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
}

pub fn default_spotlights() -> Vec<Spotlight> {
    vec![
        Spotlight {
            origin_x_ratio: 0.15,
            angle_center: std::f32::consts::FRAC_PI_2,
            angle_amplitude: 0.55,
            speed: 0.85,
            phase_offset: 0.0,
            phase: 0.0,
            spread: 0.15,
            color_r: 160.0,
            color_g: 20.0,
            color_b: 255.0,
        },
        Spotlight {
            origin_x_ratio: 0.50,
            angle_center: std::f32::consts::FRAC_PI_2,
            angle_amplitude: 0.70,
            speed: 0.55,
            phase_offset: 2.5,
            phase: 0.0,
            spread: 0.12,
            color_r: 0.0,
            color_g: 130.0,
            color_b: 255.0,
        },
        Spotlight {
            origin_x_ratio: 0.85,
            angle_center: std::f32::consts::FRAC_PI_2,
            angle_amplitude: 0.60,
            speed: 1.05,
            phase_offset: 4.5,
            phase: 0.0,
            spread: 0.14,
            color_r: 255.0,
            color_g: 0.0,
            color_b: 130.0,
        },
        Spotlight {
            origin_x_ratio: 0.30,
            angle_center: std::f32::consts::FRAC_PI_2,
            angle_amplitude: 0.70,
            speed: 0.70,
            phase_offset: 1.2,
            phase: 0.0,
            spread: 0.13,
            color_r: 0.0,
            color_g: 220.0,
            color_b: 180.0,
        },
        Spotlight {
            origin_x_ratio: 0.70,
            angle_center: std::f32::consts::FRAC_PI_2,
            angle_amplitude: 0.65,
            speed: 0.90,
            phase_offset: 3.1,
            phase: 0.0,
            spread: 0.14,
            color_r: 255.0,
            color_g: 170.0,
            color_b: 0.0,
        },
        Spotlight {
            origin_x_ratio: 0.40,
            angle_center: std::f32::consts::FRAC_PI_2,
            angle_amplitude: 0.60,
            speed: 0.60,
            phase_offset: 5.0,
            phase: 0.0,
            spread: 0.12,
            color_r: 220.0,
            color_g: 0.0,
            color_b: 220.0,
        },
    ]
}



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
    pub(super) rgb: Option<RgbController>,
    pub(super) rgb_timer: f32,
}

impl Default for Beams {
    fn default() -> Self {
        Self::new()
    }
}

impl Beams {
    pub fn new() -> Self {
        // Pre-4.1 registry read (BeamCount, TwinkleStars) collapsed to defaults
        // for the inline migration. Will be re-added in 4.2 once library has
        // a settings module. Defaults: 4 beams, twinkle stars on.
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
            rgb: if is_openrgb_enabled() { Some(RgbController::new()) } else { None },
            rgb_timer: 0.0,
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

        self.rgb_timer += delta;
        if self.rgb_timer >= 0.05 {
            self.rgb_timer = 0.0;
            if let Some(ref r) = self.rgb {
                let accent = query_current_palette().accent;
                let mut spots = self.spotlights.clone();
                if spots.len() >= 2 {
                    spots[1].color_r = accent.0 as f32;
                    spots[1].color_g = accent.1 as f32;
                    spots[1].color_b = accent.2 as f32;
                }

                let get_mixed_color_at_pos = |x_ratio: f32| -> RgbColor {
                    let mut r_sum = 0.0f32;
                    let mut g_sum = 0.0f32;
                    let mut b_sum = 0.0f32;
                    for spot in &spots {
                        let angle = spot.angle_center + spot.angle_amplitude * (spot.phase + spot.phase_offset).sin();
                        let hit_x = spot.origin_x_ratio + angle.cos() / angle.sin().max(0.01);
                        let dist = (x_ratio - hit_x).abs();
                        let intensity = (-(dist * dist) / (2.0 * spot.spread * spot.spread)).exp();
                        r_sum += spot.color_r * intensity;
                        g_sum += spot.color_g * intensity;
                        b_sum += spot.color_b * intensity;
                    }
                    RgbColor::new(
                        r_sum.clamp(0.0, 255.0) as u8,
                        g_sum.clamp(0.0, 255.0) as u8,
                        b_sum.clamp(0.0, 255.0) as u8,
                    )
                };

                r.set_device_color(5, get_mixed_color_at_pos(0.5));
                r.set_device_color(6, get_mixed_color_at_pos(0.8));
                r.set_device_color(12, get_mixed_color_at_pos(0.1));
                let c_internal = get_mixed_color_at_pos(0.6);
                r.set_device_color(0, c_internal);
                r.set_device_color(1, c_internal);
                r.set_device_color(2, c_internal);
            }
        }
    }

    fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        self.draw_impl(grid, cols, rows);
    }
}


impl Beams {
    pub fn draw_impl(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        let mut spotlights = self.spotlights.clone();
        let accent = query_current_palette().accent;
        if spotlights.len() >= 2 {
            spotlights[1].color_r = accent.0 as f32;
            spotlights[1].color_g = accent.1 as f32;
            spotlights[1].color_b = accent.2 as f32;
        }

        let y_origin = rows as f32;
        let max_dist = y_origin * 1.6;

        let mut current_angles = Vec::new();
        let mut spot_cots = Vec::new();
        for spot in &spotlights {
            let angle = spot.angle_center + spot.angle_amplitude * (spot.phase + spot.phase_offset).sin();
            current_angles.push(angle);

            let a_min = angle - spot.spread;
            let a_max = angle + spot.spread;

            let cot_min = if a_min > 1e-4 {
                let (sin, cos) = a_min.sin_cos();
                cos / sin
            } else {
                0.0
            };

            let cot_max = if a_max < std::f32::consts::PI - 1e-4 {
                let (sin, cos) = a_max.sin_cos();
                cos / sin
            } else {
                0.0
            };

            spot_cots.push((a_min, a_max, cot_min, cot_max));
        }

        let get_light_at = |cx: f32, cy: f32| -> (f32, f32, f32, f32) {
            let mut r = 0.0f32;
            let mut g = 0.0f32;
            let mut b = 0.0f32;
            let mut total_intensity = 0.0f32;

            for (i, spot) in spotlights.iter().enumerate() {
                let x_origin = spot.origin_x_ratio * cols as f32;
                let dx = (cx - x_origin) * 0.55;
                let dy = y_origin - cy;

                if dy > 0.0 {
                    let (a_min, a_max, cot_min, cot_max) = spot_cots[i];
                    let mut in_beam = true;
                    if a_min > 1e-4 && dx >= dy * cot_min {
                        in_beam = false;
                    }
                    if in_beam && a_max < std::f32::consts::PI - 1e-4 && dx <= dy * cot_max {
                        in_beam = false;
                    }

                    if in_beam {
                        let angle = dy.atan2(dx);
                        let dist = (dx*dx + dy*dy).sqrt();
                        let current_angle = current_angles[i];
                        let mut da = angle - current_angle;

                        da = (da + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;

                        if da.abs() < spot.spread {
                            let angular_intensity = 1.0 - (da.abs() / spot.spread);
                            let dist_intensity = (1.0 - dist / max_dist).max(0.0);
                            let wave = 0.88 + 0.12 * (dist * 0.28 - self.time_elapsed * 14.0).sin();
                            let intensity = angular_intensity * dist_intensity * wave;

                            r += intensity * spot.color_r;
                            g += intensity * spot.color_g;
                            b += intensity * spot.color_b;
                            total_intensity += intensity;
                        }
                    }
                }
            }
            (r.min(255.0), g.min(255.0), b.min(255.0), total_intensity.min(1.0))
        };

        // 1. Volumetric background beams
        for y in 0..rows {
            for x in 0..cols {
                let (r, g, b, _) = get_light_at(x as f32, y as f32);
                let bg_r = (r * 0.15) as u8;
                let bg_g = (g * 0.15) as u8;
                let bg_b = (b * 0.15) as u8;

                grid[y * cols + x] = TerminalCell {
                    ch: ' ',
                    fg: (0, 0, 0),
                    bg: (bg_r, bg_g, bg_b),
                    bold: false,
                };
            }
        }

        if self.twinkle_stars_opt == 1 {
            // Top candidates for lens flares (highly excited stars, max 4)
            let mut flare_candidates: Vec<(usize, f32)> = self.stars.iter()
                .enumerate()
                .filter(|(_, star)| star.excitation > 0.8)
                .map(|(idx, star)| (idx, star.excitation))
                .collect();
            flare_candidates.sort_by(|a, b| b.1.total_cmp(&a.1));
            let allowed_flares: Vec<usize> = flare_candidates.iter()
                .take(4)
                .map(|&(idx, _)| idx)
                .collect();

            for (i, star) in self.stars.iter().enumerate() {
                let sx = (star.x * cols as f32) as usize;
                let sy = (star.y * rows as f32) as usize;
                if sx < cols && sy < rows {
                    let (lr, lg, lb, intensity) = get_light_at(sx as f32, sy as f32);
                    let sparkle_base = ((self.time_elapsed * 2.2 + star.phase).sin() + 1.0) * 0.5;
                    let sparkle = (sparkle_base + star.excitation).min(2.0);
                    let final_brightness = sparkle * 0.4 + intensity * 0.6;

                    let mut star_r = (65.0 + final_brightness * 190.0) as u8;
                    let mut star_g = (65.0 + final_brightness * 190.0) as u8;
                    let mut star_b = (85.0 + final_brightness * 170.0) as u8;

                    if star.excitation > 0.05 || intensity > 0.1 {
                        let blend = (star.excitation * 0.6 + intensity * 0.4).min(1.0);
                        star_r = (star_r as f32 * (1.0 - blend) + lr * blend).min(255.0) as u8;
                        star_g = (star_g as f32 * (1.0 - blend) + lg * blend).min(255.0) as u8;
                        star_b = (star_b as f32 * (1.0 - blend) + lb * blend).min(255.0) as u8;
                    }

                    let cell = &mut grid[sy * cols + sx];
                    cell.ch = if final_brightness > 0.8 { '\u{2739}' } else if final_brightness > 0.5 { '\u{2726}' } else { star.ch };
                    cell.fg = (star_r, star_g, star_b);
                    cell.bold = final_brightness > 0.6 || star.excitation > 0.3;

                    let is_excited = allowed_flares.contains(&i);
                    if is_excited {
                        let flare_intensity = ((star.excitation - 0.8) / 0.7 + 0.5).min(1.5);

                        // horizontal flare
                        let h_len = 12;
                        for dx in 1..h_len {
                            let alpha = (120.0f32 * flare_intensity).max(30.0f32) as u8;
                            let fade = alpha.saturating_sub((dx * (110 / h_len)) as u8);
                            if fade > 10 {
                                if sx + dx < cols {
                                    let cell = &mut grid[sy * cols + (sx + dx)];
                                    if cell.ch == ' ' || cell.ch == '\u{2500}' {
                                        cell.ch = '\u{2500}';
                                        cell.fg = (
                                            (fade as f32 * 0.4 + lr * 0.6).min(255.0) as u8,
                                            (fade as f32 * 0.75 * 0.4 + lg * 0.6).min(255.0) as u8,
                                            (fade.saturating_add(45) as f32 * 0.4 + lb * 0.6).min(255.0) as u8,
                                        );
                                    }
                                }
                                if sx >= dx {
                                    let cell = &mut grid[sy * cols + (sx - dx)];
                                    if cell.ch == ' ' || cell.ch == '\u{2500}' {
                                        cell.ch = '\u{2500}';
                                        cell.fg = (
                                            (fade as f32 * 0.4 + lr * 0.6).min(255.0) as u8,
                                            (fade as f32 * 0.75 * 0.4 + lg * 0.6).min(255.0) as u8,
                                            (fade.saturating_add(45) as f32 * 0.4 + lb * 0.6).min(255.0) as u8,
                                        );
                                    }
                                }
                            }
                        }

                        // vertical flare
                        let v_len = 5;
                        for dy in 1..v_len {
                            let alpha = (90.0f32 * flare_intensity).max(20.0f32) as u8;
                            let fade = alpha.saturating_sub((dy * (80 / v_len)) as u8);
                            if fade > 10 {
                                if sy + dy < rows {
                                    let cell = &mut grid[(sy + dy) * cols + sx];
                                    if cell.ch == ' ' || cell.ch == '\u{2502}' {
                                        cell.ch = '\u{2502}';
                                        cell.fg = (
                                            (fade as f32 * 0.4 + lr * 0.6).min(255.0) as u8,
                                            (fade as f32 * 0.75 * 0.4 + lg * 0.6).min(255.0) as u8,
                                            (fade.saturating_add(30) as f32 * 0.4 + lb * 0.6).min(255.0) as u8,
                                        );
                                    }
                                }
                                if sy >= dy {
                                    let cell = &mut grid[(sy - dy) * cols + sx];
                                    if cell.ch == ' ' || cell.ch == '\u{2502}' {
                                        cell.ch = '\u{2502}';
                                        cell.fg = (
                                            (fade as f32 * 0.4 + lr * 0.6).min(255.0) as u8,
                                            (fade as f32 * 0.75 * 0.4 + lg * 0.6).min(255.0) as u8,
                                            (fade.saturating_add(30) as f32 * 0.4 + lb * 0.6).min(255.0) as u8,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 3. Dust particles
        for p in &self.particles {
            let px = (p.x * cols as f32) as usize;
            let py = (p.y * rows as f32) as usize;
            if px < cols && py < rows {
                let (lr, lg, lb, intensity) = get_light_at(px as f32, py as f32);
                if intensity > 0.05 {
                    let ch = if intensity > 0.6 { '*' } else if intensity > 0.3 { '+' } else { '.' };
                    let p_r = (140.0 * (1.0 - intensity) + lr * intensity).min(255.0) as u8;
                    let p_g = (100.0 * (1.0 - intensity) + lg * intensity).min(255.0) as u8;
                    let p_b = (255.0 * (1.0 - intensity) + lb * intensity).min(255.0) as u8;

                    let cell = &mut grid[py * cols + px];
                    let current_ch = cell.ch;
                    if current_ch == ' ' || current_ch == '\u{2500}' || current_ch == '\u{2502}' {
                        cell.ch = ch;
                        cell.fg = (p_r, p_g, p_b);
                    }
                }
            }
        }

        // 4. Centered system-logo overlay (was trance_core::logo_lines in pre-4.1)
        let sys = get_system_info();
        let logo_lines = render_logo_block(&sys.logo_text, None);
        if !logo_lines.is_empty() {
            let logo_h = logo_lines.len();
            let logo_w = logo_lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
            let logo_x = cols.saturating_sub(logo_w) / 2;
            let logo_y = rows.saturating_sub(logo_h) / 2;

            for (r_offset, line) in logo_lines.iter().enumerate().take(logo_h) {
                let gy = logo_y + r_offset;
                if gy >= rows {
                    continue;
                }
                for (c_offset, ch) in line.chars().enumerate() {
                    let gx = logo_x + c_offset;
                    if gx >= cols {
                        continue;
                    }
                    if ch != ' ' {
                        let (lr, lg, lb, intensity) = get_light_at(gx as f32, gy as f32);
                        let (fg_r, fg_g, fg_b) = if intensity > 0.05 {
                            let l_r = (90.0 * (1.0 - intensity) + lr * intensity).min(255.0) as u8;
                            let l_g = (20.0 * (1.0 - intensity) + lg * intensity).min(255.0) as u8;
                            let l_b = (120.0 * (1.0 - intensity) + lb * intensity).min(255.0) as u8;
                            (l_r, l_g, l_b)
                        } else {
                            (45, 20, 60)
                        };

                        let cell = &mut grid[gy * cols + gx];
                        cell.ch = ch;
                        cell.fg = (fg_r, fg_g, fg_b);
                        cell.bold = intensity > 0.05;
                    }
                }
            }
        }
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
        // Beams should draw spotlight effects/logo to the grid
        let drawn_count = grid.iter().filter(|c| c.ch != '\0').count();
        assert!(drawn_count > 0, "No beams/particles drawn in the grid");
    }
}
