//! Light and spotlight calculations for the beams screensaver.

use crate::runner::core::TerminalCell;
use super::types::Spotlight;

pub fn get_light_at(
    cx: f32,
    cy: f32,
    cols: usize,
    rows: usize,
    spotlights: &[Spotlight],
    current_angles: &[f32],
    spot_cots: &[(f32, f32, f32, f32, f32)],
    _time_elapsed: f32,
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
