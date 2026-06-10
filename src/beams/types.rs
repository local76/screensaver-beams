//! Auxiliary types and defaults for the beams screensaver.

use std::f32::consts::FRAC_PI_2;

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
            angle_center: FRAC_PI_2,
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
            angle_center: FRAC_PI_2,
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
            angle_center: FRAC_PI_2,
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
            angle_center: FRAC_PI_2,
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
            angle_center: FRAC_PI_2,
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
            angle_center: FRAC_PI_2,
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
