use super::*;
use crate::runner::core::TerminalCell;
use crate::runner::core::screensaver::Screensaver;
use std::time::Duration;

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

#[test]
fn test_math_light_falloff() {
    // Test that intensity falls off as distance increases from the source.
    let spot = Spotlight {
        origin_x_ratio: 0.5,
        color_r: 255.0,
        color_g: 255.0,
        color_b: 255.0,
        angle_center: std::f32::consts::FRAC_PI_2,
        angle_amplitude: 0.0,
        phase: 0.0,
        phase_offset: 0.0,
        speed: 1.0,
        spread: 0.2,
    };
    let spotlights = vec![spot];
    let current_angles = vec![std::f32::consts::FRAC_PI_2];
    let spot_cots = vec![(
        std::f32::consts::FRAC_PI_2 - 0.2,
        std::f32::consts::FRAC_PI_2 + 0.2,
        (std::f32::consts::FRAC_PI_2 - 0.2).cos() / (std::f32::consts::FRAC_PI_2 - 0.2).sin(),
        (std::f32::consts::FRAC_PI_2 + 0.2).cos() / (std::f32::consts::FRAC_PI_2 + 0.2).sin(),
        1.0 / 0.2
    )];

    let (_, _, _, i1) = super::light::get_light_at(40.0, 20.0, 80, 24, &spotlights, &current_angles, &spot_cots, 0.0);
    let (_, _, _, i2) = super::light::get_light_at(40.0, 10.0, 80, 24, &spotlights, &current_angles, &spot_cots, 0.0);

    assert!(i1 > i2, "Intensity closer to the source ({}) should be greater than further away ({})", i1, i2);
}

#[test]
fn test_math_light_angle_boundary() {
    // Test that points outside the spread get 0 intensity.
    let spot = Spotlight {
        origin_x_ratio: 0.5,
        color_r: 255.0,
        color_g: 255.0,
        color_b: 255.0,
        angle_center: std::f32::consts::FRAC_PI_2,
        angle_amplitude: 0.0,
        phase: 0.0,
        phase_offset: 0.0,
        speed: 1.0,
        spread: 0.1,
    };
    let spotlights = vec![spot];
    let current_angles = vec![std::f32::consts::FRAC_PI_2];
    let spot_cots = vec![(
        std::f32::consts::FRAC_PI_2 - 0.1,
        std::f32::consts::FRAC_PI_2 + 0.1,
        (std::f32::consts::FRAC_PI_2 - 0.1).cos() / (std::f32::consts::FRAC_PI_2 - 0.1).sin(),
        (std::f32::consts::FRAC_PI_2 + 0.1).cos() / (std::f32::consts::FRAC_PI_2 + 0.1).sin(),
        1.0 / 0.1
    )];

    let (_, _, _, i_center) = super::light::get_light_at(40.0, 20.0, 80, 24, &spotlights, &current_angles, &spot_cots, 0.0);
    let (_, _, _, i_offside) = super::light::get_light_at(20.0, 20.0, 80, 24, &spotlights, &current_angles, &spot_cots, 0.0);

    assert!(i_center > 0.0);
    assert_eq!(i_offside, 0.0);
}
