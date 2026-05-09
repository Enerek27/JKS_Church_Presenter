pub mod db;
pub mod model;
pub mod schema;

pub mod library_jks {
    use std::{
        fmt::{Display, Formatter},
        fs::{File, remove_file},
        io::{BufReader, BufWriter},
    };

    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct StrofaJKS {
        pub id: i32,
        pub cislo_strofy: i32,
        pub typ_piesne: Option<TypPiesne>,
        pub text: String,
    }

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
    pub enum TypPiesne {
        JKS(JKSTypPiesne),
        AntifonaSurin,
        Antifona,
        Taize,
        Mladeznicka,
        Hymna,
    }
    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
    pub enum JKSTypPiesne {
        Advent,
        Vianoce,
        KNajsvMenuJezisovmu,
        Post,
        VelkaNoc,
        NaneboVstupeniePana,
        NaDuchaSvateho,
        NaNajsvatejsiuTrojicu,
        NaBozieTelo,
        KSrdcuJezisovmu,
        KSvatejOmsi,
        KNajsvaSviatostiOltarnej,
        Litanie,
        AntifonaKPM,
        KPanneMarii,
        KSvatym,
        ZaZomrelych,
        Kajuce,
        PiesnePrilezitostne,
        PredPozehnanim,
    }

    impl TypPiesne {
        pub fn from_str_db(name: &str) -> Option<Self> {
            match name {
                "Antifona Šurin" => Some(TypPiesne::AntifonaSurin),
                "Mládežnícka" => Some(TypPiesne::Mladeznicka),
                "Hymna" => Some(TypPiesne::Hymna),
                "Antifona" => Some(TypPiesne::Antifona),
                "Taize" => Some(TypPiesne::Taize),

                "Advent" => Some(TypPiesne::JKS(JKSTypPiesne::Advent)),
                "Vianoce" => Some(TypPiesne::JKS(JKSTypPiesne::Vianoce)),
                "k Najsv. Menu Ježišovmu" => {
                    Some(TypPiesne::JKS(JKSTypPiesne::KNajsvMenuJezisovmu))
                }
                "Pôst" => Some(TypPiesne::JKS(JKSTypPiesne::Post)),
                "Veľká Noc" => Some(TypPiesne::JKS(JKSTypPiesne::VelkaNoc)),
                "Nanebovstúpenie Pána" => Some(TypPiesne::JKS(JKSTypPiesne::NaneboVstupeniePana)),
                "Na Ducha Svätého" => Some(TypPiesne::JKS(JKSTypPiesne::NaDuchaSvateho)),
                "Na Najsvätejšiu Trojicu" => {
                    Some(TypPiesne::JKS(JKSTypPiesne::NaNajsvatejsiuTrojicu))
                }
                "Na Božie Telo" => Some(TypPiesne::JKS(JKSTypPiesne::NaBozieTelo)),
                "k Srdcu Ježišovmu" => Some(TypPiesne::JKS(JKSTypPiesne::KSrdcuJezisovmu)),
                "k Svätej Omši" => Some(TypPiesne::JKS(JKSTypPiesne::KSvatejOmsi)),
                "k Najsv. Sviatosti Oltárnej" => {
                    Some(TypPiesne::JKS(JKSTypPiesne::KNajsvaSviatostiOltarnej))
                }
                "Litánie" => Some(TypPiesne::JKS(JKSTypPiesne::Litanie)),
                "Antifona k Panne Márii" => Some(TypPiesne::JKS(JKSTypPiesne::AntifonaKPM)),
                "k Panne Márii" => Some(TypPiesne::JKS(JKSTypPiesne::KPanneMarii)),
                "k Svätým" => Some(TypPiesne::JKS(JKSTypPiesne::KSvatym)),
                "Za zomrelých" => Some(TypPiesne::JKS(JKSTypPiesne::ZaZomrelych)),
                "Kajúce" => Some(TypPiesne::JKS(JKSTypPiesne::Kajuce)),
                "Príležitostné piesne" => {
                    Some(TypPiesne::JKS(JKSTypPiesne::PiesnePrilezitostne))
                }
                "Pred požehnaním" => Some(TypPiesne::JKS(JKSTypPiesne::PredPozehnanim)),

                _ => None,
            }
        }

        pub fn all() -> &'static [TypPiesne] {
            &[
                TypPiesne::AntifonaSurin,
                TypPiesne::Antifona,
                TypPiesne::Taize,
                TypPiesne::Mladeznicka,
                TypPiesne::Hymna,
                TypPiesne::JKS(JKSTypPiesne::Advent),
                TypPiesne::JKS(JKSTypPiesne::Vianoce),
                TypPiesne::JKS(JKSTypPiesne::KNajsvMenuJezisovmu),
                TypPiesne::JKS(JKSTypPiesne::Post),
                TypPiesne::JKS(JKSTypPiesne::VelkaNoc),
                TypPiesne::JKS(JKSTypPiesne::NaneboVstupeniePana),
                TypPiesne::JKS(JKSTypPiesne::NaDuchaSvateho),
                TypPiesne::JKS(JKSTypPiesne::NaNajsvatejsiuTrojicu),
                TypPiesne::JKS(JKSTypPiesne::NaBozieTelo),
                TypPiesne::JKS(JKSTypPiesne::KSrdcuJezisovmu),
                TypPiesne::JKS(JKSTypPiesne::KSvatejOmsi),
                TypPiesne::JKS(JKSTypPiesne::KNajsvaSviatostiOltarnej),
                TypPiesne::JKS(JKSTypPiesne::Litanie),
                TypPiesne::JKS(JKSTypPiesne::AntifonaKPM),
                TypPiesne::JKS(JKSTypPiesne::KPanneMarii),
                TypPiesne::JKS(JKSTypPiesne::KSvatym),
                TypPiesne::JKS(JKSTypPiesne::ZaZomrelych),
                TypPiesne::JKS(JKSTypPiesne::Kajuce),
                TypPiesne::JKS(JKSTypPiesne::PiesnePrilezitostne),
                TypPiesne::JKS(JKSTypPiesne::PredPozehnanim),
            ]
        }
    }

    impl Display for TypPiesne {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            let navrat = match self {
                TypPiesne::AntifonaSurin => "Antifona Šurin",
                TypPiesne::Mladeznicka => "Mládežnícka",
                TypPiesne::Hymna => "Hymna",
                TypPiesne::Antifona => "Antifona",
                TypPiesne::Taize => "Taize",
                TypPiesne::JKS(jkstyp_piesne) => match jkstyp_piesne {
                    JKSTypPiesne::Advent => "Advent",
                    JKSTypPiesne::Vianoce => "Vianoce",
                    JKSTypPiesne::KNajsvMenuJezisovmu => "k Najsv. Menu Ježišovmu",
                    JKSTypPiesne::Post => "Pôst",
                    JKSTypPiesne::VelkaNoc => "Veľká Noc",
                    JKSTypPiesne::NaneboVstupeniePana => "Nanebovstúpenie Pána",
                    JKSTypPiesne::NaDuchaSvateho => "Na Ducha Svätého",
                    JKSTypPiesne::NaNajsvatejsiuTrojicu => "Na Najsvätejšiu Trojicu",
                    JKSTypPiesne::NaBozieTelo => "Na Božie Telo",
                    JKSTypPiesne::KSrdcuJezisovmu => "k Srdcu Ježišovmu",
                    JKSTypPiesne::KSvatejOmsi => "k Svätej Omši",
                    JKSTypPiesne::KNajsvaSviatostiOltarnej => "k Najsv. Sviatosti Oltárnej",
                    JKSTypPiesne::Litanie => "Litánie",
                    JKSTypPiesne::AntifonaKPM => "Antifona k Panne Márii",
                    JKSTypPiesne::KPanneMarii => "k Panne Márii",
                    JKSTypPiesne::KSvatym => "k Svätým",
                    JKSTypPiesne::ZaZomrelych => "Za zomrelých",
                    JKSTypPiesne::Kajuce => "Kajúce",
                    JKSTypPiesne::PiesnePrilezitostne => "Príležitostné piesne",
                    JKSTypPiesne::PredPozehnanim => "Pred požehnaním",
                },
            };
            write!(f, "{}", navrat)
        }
    }
    impl std::fmt::Display for JKSTypPiesne {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            let navrat = match self {
                JKSTypPiesne::Advent => "Advent",
                JKSTypPiesne::Vianoce => "Vianoce",
                JKSTypPiesne::KNajsvMenuJezisovmu => "k Najsv. Menu Ježišovmu",
                JKSTypPiesne::Post => "Pôst",
                JKSTypPiesne::VelkaNoc => "Veľká Noc",
                JKSTypPiesne::NaneboVstupeniePana => "Nanebovstúpenie Pána",
                JKSTypPiesne::NaDuchaSvateho => "Na Ducha Svätého",
                JKSTypPiesne::NaNajsvatejsiuTrojicu => "Na Najsvätejšiu Trojicu",
                JKSTypPiesne::NaBozieTelo => "Na Božie Telo",
                JKSTypPiesne::KSrdcuJezisovmu => "k Srdcu Ježišovmu",
                JKSTypPiesne::KSvatejOmsi => "k Svätej Omši",
                JKSTypPiesne::KNajsvaSviatostiOltarnej => "k Najsv. Sviatosti Oltárnej",
                JKSTypPiesne::Litanie => "Litánie",
                JKSTypPiesne::AntifonaKPM => "Antifona k Panne Márii",
                JKSTypPiesne::KPanneMarii => "k Panne Márii",
                JKSTypPiesne::KSvatym => "k Svätým",
                JKSTypPiesne::ZaZomrelych => "Za zomrelých",
                JKSTypPiesne::Kajuce => "Kajúce",
                JKSTypPiesne::PiesnePrilezitostne => "Príležitostné piesne",
                JKSTypPiesne::PredPozehnanim => "Pred požehnaním",
            };
            write!(f, "{}", navrat)
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SongJks {
        pub id: i32,
        pub pocet_strof: i32,
        pub strofy: Vec<StrofaJKS>,
        pub typ_pesnicky: TypPiesne,
    }

    impl SongJks {
        pub fn new(
            id: i32,
            pocet_strof: i32,
            strofy: Vec<StrofaJKS>,
            typ_piesne: TypPiesne,
        ) -> SongJks {
            let ret = SongJks {
                id,
                pocet_strof,
                strofy,
                typ_pesnicky: typ_piesne,
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
            self.piesne.sort_by_key(|s| s.id);
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
