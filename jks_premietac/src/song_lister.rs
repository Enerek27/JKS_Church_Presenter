use std::{collections::BTreeMap, fmt::Debug};

use prehladavac_db_jks::{
    db::db_load_all,
    library_jks::{SongJks, SongManager, StrofaJKS, TypPiesne},
};
use ratatui::widgets::ListState;
use tui_tree_widget::{TreeItem, TreeState};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TreeId {
    Folder(TypPiesne),
    Song(i32),
}

#[derive(Debug)]
pub struct SongListerList {
    pub song_manager: SongManager,
    pub state: ListState,
}

impl Default for SongListerList {
    fn default() -> Self {
        Self {
            song_manager: SongManager::default(),
            state: ListState::default(),
        }
    }
}

impl SongListerList {
    pub fn new() -> Self {
        Self::default()
    }

    // posun dole
    pub fn select_next(&mut self) {
        let len = self.song_manager.piesne.len();
        if len == 0 {
            return;
        }
        let i = match self.state.selected() {
            Some(i) if i + 1 < len => i + 1,
            _ => 0,
        };
        self.state.select(Some(i));
    }

    // posun hore
    pub fn select_previous(&mut self) {
        let len = self.song_manager.piesne.len();
        if len == 0 {
            return;
        }
        let i = match self.state.selected() {
            Some(0) | None => len - 1,
            Some(i) => i - 1,
        };
        self.state.select(Some(i));
    }

    // ak nič nie je vybraté a niečo tam je, vyber prvý
    pub fn ensure_selected(&mut self) {
        if self.state.selected().is_none() && !self.song_manager.is_empty() {
            self.state.select(Some(0));
        }
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.state.selected()
    }

    pub fn get_selected_song(&self) -> Option<&SongJks> {
        let idx = self.state.selected()?;
        self.song_manager.piesne.get(idx)
    }

    pub fn remove_selected(&mut self) {
        if let Some(idx) = self.state.selected() {
            if idx < self.song_manager.piesne.len() {
                self.song_manager.piesne.remove(idx);
                // posun výberu, aby neukazoval mimo
                let len = self.song_manager.piesne.len();
                if len == 0 {
                    self.state.select(None);
                } else if idx >= len {
                    self.state.select(Some(len - 1));
                } else {
                    self.state.select(Some(idx));
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct SongLister {
    pub song_manager: SongManager,
    pub state: TreeState<TreeId>,
    pub search: String,
}

impl Default for SongLister {
    fn default() -> Self {
        Self {
            song_manager: SongManager::default(),
            state: TreeState::default(),
            search: String::new(),
        }
    }
}

impl SongLister {
    pub fn new() -> Self {
        SongLister {
            song_manager: db_load_all(),
            state: TreeState::default(),
            search: String::new(),
        }
    }

    pub fn build_tree(&self) -> Vec<TreeItem<'static, TreeId>> {
        let mut groups: BTreeMap<TypPiesne, Vec<&SongJks>> = BTreeMap::new();

        for song in self.search_get_formated() {
            for typ in &song.typ_pesnicky {
                groups.entry(*typ).or_default().push(song);
            }
        }

        groups
            .into_iter()
            .map(|(typ_enum, songs)| {
                let typ_name: String = typ_enum.to_string();

                let children: Vec<TreeItem<'static, TreeId>> = songs
                    .into_iter()
                    .map(|song| {
                        TreeItem::new_leaf(TreeId::Song(song.id), song.format_song().to_string())
                    })
                    .collect();

                TreeItem::new(TreeId::Folder(typ_enum), typ_name, children).unwrap()
            })
            .collect()
    }

    pub fn select_next(&mut self) {
        self.state.key_down();
    }

    pub fn select_previous(&mut self) {
        self.state.key_up();
    }

    pub fn expand(&mut self) {
        self.state.key_right();
    }

    pub fn collapse(&mut self) {
        self.state.key_left();
    }

    pub fn search_get_formated(&self) -> Vec<&SongJks> {
        let all = self.song_manager.get_all_songs();
        if self.search.is_empty() {
            return all;
        };

        let searched = self.search.to_lowercase();
        all.into_iter()
            .filter(|s| s.format_song().to_lowercase().contains(&searched))
            .collect()
    }
}
