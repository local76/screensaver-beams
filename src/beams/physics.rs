//! Core calculations and helper functions for the beams screensaver.

use crate::runner::core::TerminalCell;
use crate::runner::toolkit::sys_info::query_current_palette;
use crate::runner::core::logo_block::render_logo_block;

use super::types::{Spotlight, Star, DustParticle};
use super::light::{get_light_at, draw_spotlight};
use super::physics_star::draw_star;

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
    logo_text: &str,
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
    let logo_lines = render_logo_block(logo_text, None);
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
