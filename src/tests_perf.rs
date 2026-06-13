use std::time::{Duration, Instant};
use crate::runner::core::TerminalCell;
use crate::runner::core::screensaver::Screensaver;
use crate::beams::Beams;

#[test]
fn test_performance_benchmark() {
    let mut beams = Beams::default();
    // Prevent the slow system info shell queries from triggering during the loop
    // by starting the sys_refresh_timer far in the negative.
    beams.sys_refresh_timer = -1000.0;

    let cols = 80;
    let rows = 24;
    beams.init(cols, rows);

    let mut grid = vec![TerminalCell::default(); cols * rows];
    let frame_dt = Duration::from_millis(16);

    let start = Instant::now();

    for _ in 0..100 {
        beams.update(frame_dt, cols, rows);
        beams.draw(&mut grid, cols, rows);
    }

    let elapsed = start.elapsed();
    println!("100 frames completed in: {:?}", elapsed);

    // Assert it completes within a budget (1500ms in release, 5000ms in debug)
    let budget = if cfg!(debug_assertions) {
        Duration::from_millis(5000)
    } else {
        Duration::from_millis(1500)
    };

    assert!(elapsed < budget, "Performance budget exceeded: {:?}", elapsed);
}
