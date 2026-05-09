use std::env;

use monitor_lib::monitor_lib::{load_monitor_geometry, MonitorGeometry};

use crate::{gui::run_gui, initializer::init_song_manager};

pub mod gui;
pub mod initializer;

/// Hlavný vstup aplikácie na premietanie piesní.
///
/// 1. Zistí cestu k spustiteľnému súboru.
/// 2. Podľa nej nájde `temp_song_manager.json` a načíta `SongManager`.
/// 3. Načíta konfiguráciu monitora z `monitor_config.json`
///    (alebo použije predvolenú geometriu).
/// 4. Spustí fullscreen GUI na danom monitore.
fn main() {
    let exe_path = match env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Chyba pri current_exe: {}", e);
            return;
        }
    };

    let base_dir = match exe_path.parent() {
        Some(dir) => dir,
        None => {
            eprintln!("Nepodarilo sa získať parent directory");
            return;
        }
    };

    let path = base_dir.join("temp_song_manager.json");
    let path_str = path.to_string_lossy();
    let monitor_path = base_dir.join("monitor_config.json");

    let song_manager = init_song_manager(&path_str);

    let geom = load_monitor_geometry(monitor_path).unwrap_or(MonitorGeometry {
        x: 0.0,
        y: 0.0,
        width: 1280.0,
        height: 720.0,
    });

    run_gui(song_manager, geom).expect("Chyba rendrovania aplikácie na zobrazenie");
}