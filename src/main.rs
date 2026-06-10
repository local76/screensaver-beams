#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod beams;

fn main() {
    let effect = beams::Beams::new();
    library::screensaver_runner::run_main(effect, "beams");
}
