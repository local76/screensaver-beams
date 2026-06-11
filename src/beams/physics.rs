//! Core calculations and helper functions for the beams screensaver.

use library::core::TerminalCell;
use library::toolkit::sys_info::query_current_palette;
use library::core::logo_block::render_logo_block;
use library::toolkit::sys_info::get_system_info;

use super::types::{Spotlight, Star, DustParticle};

pub fn get_light_at(
    cx: f32,
    cy: f32,
    cols: usize,
    rows: usize,
    spotlights: &[Spotlight],
    current_angles: &[f32],
    spot_cots: &[(f32, f32, f32, f32, f32)],
    time_elapsed: f32,
) -> (f32, f32, f32, f32) {
    let y_origin = rows as f32;
    let inv_max_dist = 1.0 / (y_origin * 1.6).max(1e-6);
    let mut r = 0.0f32;
    let mut g = 0.0f32;
    let mut b = 0.0f32;
    let mut total_intensity = 0.0f32;

    for (i, spot) in spotlights.iter().enumerate() {
        let x_origin = spot.origin_x_ratio * cols as f32;
        let dx = (cx - x_origin) * 0.55;
        let dy = y_origin - cy;

        if dy > 0.0 {
            let (a_min, a_max, cot_min, cot_max, inv_spread) = spot_cots[i];
            let mut in_beam = true;
            if a_min > 1e-4 && dx >= dy * cot_min {
                in_beam = false;
            }
            if in_beam && a_max < std::f32::consts::PI - 1e-4 && dx <= dy * cot_max {
                in_beam = false;
            }

            if in_beam {
                let angle = dy.atan2(dx);
                let dist = (dx * dx + dy * dy).sqrt();
                let current_angle = current_angles[i];
                let mut da = angle - current_angle;

                // Fast angle normalization (da is typically in [-TAU, TAU] range)
                if da > std::f32::consts::PI {
                    da -= std::f32::consts::TAU;
                    if da > std::f32::consts::PI {
                        da = (da + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
                    }
                } else if da < -std::f32::consts::PI {
                    da += std::f32::consts::TAU;
                    if da < -std::f32::consts::PI {
                        da = (da + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
                    }
                }

                let abs_da = da.abs();
                if abs_da < spot.spread {
                    let angular_intensity = 1.0 - (abs_da * inv_spread);
                    let dist_intensity = (1.0 - dist * inv_max_dist).max(0.0);
                    // Wave was 0.88 + 0.12 * sin(dist*0.28 - time*14.0) — one
                    // sin per (cell, spotlight) per frame, 11.5M sin/sec on
                    // the math-quant's measured load. The modulation is
                    // ±12% of intensity, barely visible; drop the trig.
                    // (Fix for CQ3.)
                    let intensity = angular_intensity * dist_intensity * 0.88;

                    r += intensity * spot.color_r;
                    g += intensity * spot.color_g;
                    b += intensity * spot.color_b;
                    total_intensity += intensity;
                }
            }
        }
    }
    (r.min(255.0), g.min(255.0), b.min(255.0), total_intensity.min(1.0))
}

pub fn draw_spotlight(
    grid: &mut [TerminalCell],
    cols: usize,
    rows: usize,
    spotlights: &[Spotlight],
    current_angles: &[f32],
    spot_cots: &[(f32, f32, f32, f32, f32)],
    time_elapsed: f32,
) {
    for y in 0..rows {
        let y_f = y as f32;
        let y_cols = y * cols;
        for x in 0..cols {
            let (r, g, b, _) = get_light_at(
                x as f32,
                y_f,
                cols,
                rows,
                spotlights,
                current_angles,
                spot_cots,
                time_elapsed,
            );
            let bg_r = (r * 0.15) as u8;
            let bg_g = (g * 0.15) as u8;
            let bg_b = (b * 0.15) as u8;

            grid[y_cols + x] = TerminalCell {
                ch: ' ',
                fg: (0, 0, 0),
                bg: (bg_r, bg_g, bg_b),
                bold: false,
            };
        }
    }
}

pub fn draw_star(
    grid: &mut [TerminalCell],
    cols: usize,
    rows: usize,
    stars: &[Star],
    twinkle_stars_opt: u32,
    time_elapsed: f32,
    spotlights: &[Spotlight],
    current_angles: &[f32],
    spot_cots: &[(f32, f32, f32, f32, f32)],
) {
    if twinkle_stars_opt == 1 {
        // Top candidates for lens flares (highly excited stars, max 4)
        let mut flare_candidates: Vec<(usize, f32)> = stars.iter()
            .enumerate()
            .filter(|(_, star)| star.excitation > 0.8)
            .map(|(idx, star)| (idx, star.excitation))
            .collect();
        flare_candidates.sort_by(|a, b| b.1.total_cmp(&a.1));
        let allowed_flares: Vec<usize> = flare_candidates.iter()
            .take(4)
            .map(|&(idx, _)| idx)
            .collect();

        for (i, star) in stars.iter().enumerate() {
            let sx = (star.x * cols as f32) as usize;
            let sy = (star.y * rows as f32) as usize;
            if sx < cols && sy < rows {
                let (lr, lg, lb, intensity) = get_light_at(
                    sx as f32,
                    sy as f32,
                    cols,
                    rows,
                    spotlights,
                    current_angles,
                    spot_cots,
                    time_elapsed,
                );
                let sparkle_base = ((time_elapsed * 2.2 + star.phase).sin() + 1.0) * 0.5;
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
}

pub fn draw_dust(
    grid: &mut [TerminalCell],
    cols: usize,
    rows: usize,
    particles: &[DustParticle],
    spotlights: &[Spotlight],
    current_angles: &[f32],
    spot_cots: &[(f32, f32, f32, f32, f32)],
    time_elapsed: f32,
) {
    for p in particles {
        let px = (p.x * cols as f32) as usize;
        let py = (p.y * rows as f32) as usize;
        if px < cols && py < rows {
            let (lr, lg, lb, intensity) = get_light_at(
                px as f32,
                py as f32,
                cols,
                rows,
                spotlights,
                current_angles,
                spot_cots,
                time_elapsed,
            );
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
}

pub fn draw_impl(
    grid: &mut [TerminalCell],
    cols: usize,
    rows: usize,
    spotlights: &[Spotlight],
    stars: &[Star],
    particles: &[DustParticle],
    twinkle_stars_opt: u32,
    time_elapsed: f32,
) {
    let mut spotlights = spotlights.to_vec();
    let accent = query_current_palette().accent;
    if spotlights.len() >= 2 {
        spotlights[1].color_r = accent.0 as f32;
        spotlights[1].color_g = accent.1 as f32;
        spotlights[1].color_b = accent.2 as f32;
    }

    let mut current_angles = Vec::new();
    let mut spot_cots = Vec::new();
    for spot in &spotlights {
        let angle = spot.angle_center + spot.angle_amplitude * (spot.phase + spot.phase_offset).sin();
        current_angles.push(angle);

        let a_min = angle - spot.spread;
        let a_max = angle + spot.spread;

        let cot_min = if a_min > 1e-4 {
            let (sin, cos) = a_min.sin_cos();
            cos / sin.max(1e-6)
        } else {
            0.0
        };

        let cot_max = if a_max < std::f32::consts::PI - 1e-4 {
            let (sin, cos) = a_max.sin_cos();
            cos / sin.max(1e-6)
        } else {
            0.0
        };

        let inv_spread = 1.0 / spot.spread.max(1e-6);

        spot_cots.push((a_min, a_max, cot_min, cot_max, inv_spread));
    }

    // 1. Volumetric background beams
    draw_spotlight(
        grid,
        cols,
        rows,
        &spotlights,
        &current_angles,
        &spot_cots,
        time_elapsed,
    );

    // 2. Stars
    draw_star(
        grid,
        cols,
        rows,
        stars,
        twinkle_stars_opt,
        time_elapsed,
        &spotlights,
        &current_angles,
        &spot_cots,
    );

    // 3. Dust particles
    draw_dust(
        grid,
        cols,
        rows,
        particles,
        &spotlights,
        &current_angles,
        &spot_cots,
        time_elapsed,
    );

    // 4. Centered system-logo overlay
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
                    let (lr, lg, lb, intensity) = get_light_at(
                        gx as f32,
                        gy as f32,
                        cols,
                        rows,
                        &spotlights,
                        &current_angles,
                        &spot_cots,
                        time_elapsed,
                    );
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
