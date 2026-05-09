//! Malý nástroj na nastavenie monitora pre premietanie.
//!
//! Spustí grafický výber monitora a uloží jeho geometriu do `monitor_config.json`
//! vedľa spustiteľného súboru. Hlavná aplikácia si tento súbor neskôr načíta.

use std::env;

use monitor_lib::monitor_lib::setup_monitor;

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

    let path = base_dir.join("monitor_config.json");

    setup_monitor(path);
}
