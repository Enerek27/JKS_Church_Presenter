use std::env;

use prehladavac_db_jks::library_jks::SongManager;

use crate::{gui::run_gui, initializer::init_song_manager};

pub mod gui;
pub mod initializer;




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

    let song_manager = init_song_manager(&path_str);
    //let song_manager = SongManager::new();
    run_gui(song_manager).expect("Chyba rendrovania aplikácie na zobrazenie");
}
