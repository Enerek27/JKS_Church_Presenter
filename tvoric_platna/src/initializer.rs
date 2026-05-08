use prehladavac_db_jks::library_jks::SongManager;

pub fn init_song_manager(path: &str) -> SongManager {
    SongManager::load_manager_from_json(path)
}
