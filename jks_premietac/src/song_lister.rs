use std::{collections::BTreeMap, fmt::Debug};

use prehladavac_db_jks::{
    db::db_load_all,
    library_jks::{JKSTypPiesne, SongJks, SongManager, TypPiesne},
};
use ratatui::widgets::ListState;
use tui_tree_widget::{TreeItem, TreeState};

/// Identifikátor uzlov v stromovom zobrazení piesní.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TreeId {
    /// Koreňový priečinok podľa hlavného typu piesne.
    FolderTyp(TypPiesne),
    /// Podpriečinok pre konkrétny JKS podtyp.
    FolderJks(JKSTypPiesne),
    /// Konkrétna pesnička podľa jej ID.
    Song(i32),
}

/// Jednoduchý lineárny zoznam piesní s výberom.
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
    /// Vytvorí nový prázdny zoznam piesní.
    pub fn new() -> Self {
        Self::default()
    }

    /// Posunie výber na ďalšiu pesničku (s cyklovaním na začiatok).
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

    /// Posunie výber na predchádzajúcu pesničku (s cyklovaním na koniec).
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

    /// Ak nič nie je vybraté a zoznam nie je prázdny, vyberie prvú pesničku.
    pub fn ensure_selected(&mut self) {
        if self.state.selected().is_none() && !self.song_manager.is_empty() {
            self.state.select(Some(0));
        }
    }

    /// Vráti index aktuálne vybranej pesničky.
    pub fn selected_index(&self) -> Option<usize> {
        self.state.selected()
    }

    /// Vráti referenciu na aktuálne vybranú pesničku.
    pub fn get_selected_song(&self) -> Option<&SongJks> {
        let idx = self.state.selected()?;
        self.song_manager.piesne.get(idx)
    }

    /// Odstráni aktuálne vybranú pesničku zo zoznamu a upraví výber.
    pub fn remove_selected(&mut self) {
        if let Some(idx) = self.state.selected() {
            if idx < self.song_manager.piesne.len() {
                self.song_manager.piesne.remove(idx);
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

/// Správca stromového zobrazenia piesní (podľa typu a JKS podtypu).
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
    /// Načíta všetky piesne z databázy a vytvorí nový stromový lister.
    pub fn new() -> Self {
        SongLister {
            song_manager: db_load_all(),
            state: TreeState::default(),
            search: String::new(),
        }
    }

    /// Vytvorí strom položiek podľa typu piesní a JKS podtypov.
    pub fn build_tree(&self) -> Vec<TreeItem<'static, TreeId>> {
        let mut groups_non_jks: BTreeMap<TypPiesne, Vec<&SongJks>> = BTreeMap::new();
        let mut groups_jks: BTreeMap<JKSTypPiesne, Vec<&SongJks>> = BTreeMap::new();

        for song in self.search_get_formated() {
            match song.typ_pesnicky {
                TypPiesne::JKS(jks_typ) => {
                    groups_jks.entry(jks_typ).or_default().push(song);
                }
                other => {
                    groups_non_jks.entry(other).or_default().push(song);
                }
            }
        }

        let mut roots: Vec<TreeItem<'static, TreeId>> = Vec::new();

        // Ne-JKS typy: FolderTyp -> Song
        for (typ_enum, songs) in groups_non_jks {
            let typ_name: String = typ_enum.to_string();

            let children: Vec<TreeItem<'static, TreeId>> = songs
                .into_iter()
                .map(|song| {
                    TreeItem::new_leaf(TreeId::Song(song.id), song.format_song().to_string())
                })
                .collect();

            roots.push(TreeItem::new(TreeId::FolderTyp(typ_enum), typ_name, children).unwrap());
        }

        // JKS typ: koreň "JKS" s podpriečinkami pre každý JKSTypPiesne
        if !groups_jks.is_empty() {
            let mut jks_children: Vec<TreeItem<'static, TreeId>> = Vec::new();

            for (jks_typ, songs) in groups_jks {
                let jks_name = jks_typ.to_string();

                let song_children: Vec<TreeItem<'static, TreeId>> = songs
                    .into_iter()
                    .map(|song| {
                        TreeItem::new_leaf(TreeId::Song(song.id), song.format_song().to_string())
                    })
                    .collect();

                jks_children.push(
                    TreeItem::new(TreeId::FolderJks(jks_typ), jks_name, song_children).unwrap(),
                );
            }

            roots.push(
                TreeItem::new(
                    TreeId::FolderTyp(TypPiesne::JKS(JKSTypPiesne::Advent)),
                    "JKS".to_string(),
                    jks_children,
                )
                .unwrap(),
            );
        }

        roots
    }

    /// Posunie výber v strome na ďalšiu položku.
    pub fn select_next(&mut self) {
        self.state.key_down();
    }

    /// Posunie výber v strome na predchádzajúcu položku.
    pub fn select_previous(&mut self) {
        self.state.key_up();
    }

    /// Rozbalí aktuálny uzol (alebo prejde do poduzla).
    pub fn expand(&mut self) {
        self.state.key_right();
    }

    /// Zbalí aktuálny uzol (alebo prejde na nadradený).
    pub fn collapse(&mut self) {
        self.state.key_left();
    }

    /// Vráti piesne prefiltrované podľa `search` reťazca.
    ///
    /// Filtrovanie prebieha nad textom z `format_song()` (ID + názov).
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
