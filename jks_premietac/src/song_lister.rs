use prehladavac_db_jks::{
    db::db_load_all,
    library_jks::{SongManager, StrofaJKS},
};
use ratatui::widgets::ListState;

#[derive(Debug)]
pub struct SongLister {
    pub song_manager: SongManager,
    pub state: ListState,
    pub search: String,
}

impl Default for SongLister {
    fn default() -> Self {
        Self {
            song_manager: SongManager::default(),
            state: ListState::default(),
            search: String::new(),
        }
    }
}

impl SongLister {
    pub fn new() -> Self {
        SongLister {
            song_manager: db_load_all(),
            state: ListState::default(),
            search: String::new(),
        }
    }

    pub fn select_next(&mut self) {
        if self.song_manager.is_empty() {
            return;
        }

        let selected_now = match self.state.selected() {
            Some(i) => i,
            None => 0,
        };

        let lenght = self.song_manager.get_all_songs().len();
        let mut select_next = 0;
        if selected_now == lenght - 1 {
            select_next = 0;
        } else {
            select_next = selected_now + 1;
        }

        self.state.select(Some(select_next));
    }

    pub fn select_previous(&mut self) {
        if self.song_manager.is_empty() {
            return;
        }

        let selected_now = match self.state.selected() {
            Some(i) => i,
            None => 0,
        };

        let lenght = self.song_manager.get_all_songs().len();
        let mut select_next = 0;
        if selected_now == 0 {
            select_next = lenght - 1;
        } else {
            select_next = selected_now - 1;
        }

        self.state.select(Some(select_next));
    }

    pub fn search_get_formated(&self) -> Vec<String> {
        let all = self.song_manager.get_format_all();
        if self.search.is_empty() {
            return all;
        };

        let searched = self.search.to_lowercase();
        all.into_iter()
            .filter(|s| s.to_lowercase().contains(&searched))
            .collect()
    }
}
