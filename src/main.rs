#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod runner;
mod beams;

fn main() {
    let effect = beams::Beams::new();
    runner::run_main(effect, "beams");
}