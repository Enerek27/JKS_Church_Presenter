pub mod db;
pub mod model;
pub mod schema;

use serde_json;
pub mod library_jks {
    use std::{
        fs::{File, remove_file},
        io::{BufReader, BufWriter},
    };

    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct StrofaJKS {
        pub id: i32,
        pub cislo_strofy: i32,
        pub text: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SongJks {
        pub id: i32,
        pub pocet_strof: i32,
        pub strofy: Vec<StrofaJKS>,
    }

    impl SongJks {
        pub fn new(id: i32, pocet_strof: i32, strofy: Vec<StrofaJKS>) -> SongJks {
            let ret = SongJks {
                id,
                pocet_strof,
                strofy,
            };
            ret
        }

        pub fn get_strofa_text(&self, number_of_strofa: i32) -> &str {
            if let Some(strofa) = self.strofy.get(number_of_strofa as usize) {
                return &strofa.text;
            };
            return "Nenajdene ERROR";
        }

        pub fn format_song(&self) -> String {
            let strofa = match self.strofy.get(0) {
                Some(s) => s,
                None => panic!("Nenajdena strofa padam. Nie je strofa 0 alias nazov"),
            };

            format!("{:>5}  {}", self.id, strofa.text)
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct SongManager {
        pub piesne: Vec<SongJks>,
    }

    impl Default for SongManager {
        fn default() -> Self {
            SongManager::new()
        }
    }

    impl SongManager {
        pub fn new() -> SongManager {
            SongManager { piesne: vec![] }
        }

        pub fn get_song_by_id(&self, id: i32) -> Option<&SongJks> {
            self.piesne.iter().find(|s| s.id == id)
        }

        pub fn add_song(&mut self, napridanie: SongJks) {
            self.piesne.push(napridanie);
        }

        pub fn remove_song_by_id(&mut self, id: i32) {
            if let Some(index) = self.piesne.iter().position(|s| s.id == id) {
                self.piesne.remove(index);
            }
        }

        pub fn get_all_songs(&self) -> Vec<&SongJks> {
            let mut ret = Vec::new();
            for song in &self.piesne {
                ret.push(song);
            }

            ret
        }

        pub fn is_empty(&self) -> bool {
            self.piesne.is_empty()
        }

        pub fn get_format_all(&self) -> Vec<String> {
            let ret = self.piesne.iter().map(|s| s.format_song()).collect();
            ret
        }

        pub fn save_to_file_json(&self, path: &str) {
            let file = File::create(path).expect("Chyba otvorenia suboru pre ulozenie json");
            let writer = BufWriter::new(file);

            serde_json::to_writer_pretty(writer, self).expect("Chyba serializácie manažéra");
        }

        pub fn load_manager_from_json(path: &str) -> SongManager {
            let file = File::open(path).expect("Chyba pri otváraní súboru pre načítanie manažéra");
            let reader = BufReader::new(file);

            let ret = serde_json::from_reader(reader).expect("Chyba pri deserializácii manažéra");
            remove_file(path).expect("Subor po manažérovy sa nepodarilo odstranit");
            ret
        }
    }
}
