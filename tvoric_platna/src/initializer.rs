use prehladavac_db_jks::library_jks::SongManager;

/// Načíta `SongManager` zo JSON súboru na danom disku.
///
/// Súbor musí mať formát, ktorý očakáva
/// `SongManager::load_manager_from_json`.
pub fn init_song_manager(path: &str) -> SongManager {
    SongManager::load_manager_from_json(path)
}