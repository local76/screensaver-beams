//! Star rendering for the beams screensaver.

use crate::runner::core::TerminalCell;
use super::types::{Spotlight, Star};
use super::light::get_light_at;

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
