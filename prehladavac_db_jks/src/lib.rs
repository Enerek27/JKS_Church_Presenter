pub mod db;
pub mod model;
pub mod schema;

/// Typy a dátové štruktúry pre prácu s JKS pesničkami a ich správu.
pub mod library_jks {
    use std::{
        fmt::{Display, Formatter},
        fs::{File, remove_file},
        io::{BufReader, BufWriter},
    };

    use serde::{Deserialize, Serialize};





    /// Jedna strofa JKS pesničky uložená v pamäti.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct StrofaJKS {
        pub id: i32,
        pub cislo_strofy: i32,
        pub typ_piesne: Option<TypPiesne>,
        pub text: String,
    }

    /// Vysoká kategória typu pesničky (JKS podtyp, antifóna, Taizé atď.).
    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
    pub enum TypPiesne {
        JKS(JKSTypPiesne),
        AntifonaSurin,
        Antifona,
        Taize,
        Mladeznicka,
        Hymna,
        Zalm,
        Responz,
    }

    /// Podtypy JKS pesničiek podľa liturgickej kategórie.
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
        Ofertorium,
    }

    impl TypPiesne {
        /// Vytvorí typ pesničky z textu tak, ako je uložený v databáze.
        pub fn from_str_db(name: &str) -> Option<Self> {
            match name {
                "Žalm" => Some(TypPiesne::Zalm),
                "Antifona Šurin" => Some(TypPiesne::AntifonaSurin),
                "Mládežnícka" => Some(TypPiesne::Mladeznicka),
                "Hymna" => Some(TypPiesne::Hymna),
                "Antifona" => Some(TypPiesne::Antifona),
                "Taize" => Some(TypPiesne::Taize),
                "Responz" => Some(TypPiesne::Responz),

                "Ofertórium" => Some(TypPiesne::JKS(JKSTypPiesne::Ofertorium)),

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

        /// Vráti všetky typy pesničiek v poradí používanom v aplikácii.
        pub fn all() -> &'static [TypPiesne] {
            &[
                TypPiesne::AntifonaSurin,
                TypPiesne::Antifona,
                TypPiesne::Taize,
                TypPiesne::Mladeznicka,
                TypPiesne::Hymna,
                TypPiesne::Zalm,
                TypPiesne::Responz,
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
                TypPiesne::JKS(JKSTypPiesne::Ofertorium),
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
                TypPiesne::Zalm => "Žalm",
                TypPiesne::Responz => "Responz",
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
                    JKSTypPiesne::Ofertorium => "Ofertórium",
                },
            };
            write!(f, "{}", navrat)
        }
    }

    impl Display for JKSTypPiesne {
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
                JKSTypPiesne::Ofertorium => "Ofertórium",
            };
            write!(f, "{}", navrat)
        }
    }

    /// Celá JKS pesnička vrátane strof a typu.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SongJks {
        pub id: i32,
        pub pocet_strof: i32,
        pub strofy: Vec<StrofaJKS>,
        pub typ_pesnicky: TypPiesne,
    }

    impl SongJks {
        /// Vytvorí novú pesničku z id, počtu strof, zoznamu strof a typu pesničky.
        pub fn new(
            id: i32,
            pocet_strof: i32,
            strofy: Vec<StrofaJKS>,
            typ_piesne: TypPiesne,
        ) -> SongJks {
            SongJks {
                id,
                pocet_strof,
                strofy,
                typ_pesnicky: typ_piesne,
            }
        }

        /// Vráti text zadanej strofy podľa indexu, alebo chybové hlásenie.
        pub fn get_strofa_text(&self, number_of_strofa: i32) -> &str {
            if let Some(strofa) = self.strofy.get(number_of_strofa as usize) {
                &strofa.text
            } else {
                "Nenajdene ERROR"
            }
        }

        /// Naformátuje pesničku pre zobrazenie v zoznamoch (id a názov/strofa 0).
        pub fn format_song(&self) -> String {
            let strofa = self
                .strofy
                .get(0)
                .expect("Nenajdena strofa 0 (názov pesničky)");

            format!("{:>5}  {}", self.id, strofa.text)
        }
    }

    /// Kontajner na všetky pesničky s pomocnými metódami.
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
        /// Vytvorí prázdny správca pesničiek.
        pub fn new() -> SongManager {
            SongManager { piesne: vec![] }
        }

        /// Nájde pesničku podľa id.
        pub fn get_song_by_id(&self, id: i32, typ_piesne: TypPiesne) -> Option<&SongJks> {
            self.piesne.iter().find(|s| s.id == id && s.typ_pesnicky == typ_piesne)
        }

        /// Pridá pesničku a udržiava zoznam zoradený podľa id.
        pub fn add_song(&mut self, napridanie: SongJks) {
            self.piesne.push(napridanie);
            self.piesne.sort_by_key(|s| s.id);
        }

        /// Odstráni pesničku podľa id, ak existuje.
        pub fn remove_song_by_id(&mut self, id: i32, typ_piesne: TypPiesne) {
            if let Some(index) = self
                .piesne
                .iter()
                .position(|s| s.id == id && s.typ_pesnicky == typ_piesne)
            {
                self.piesne.remove(index);
            }
        }

        /// Vráti všetky pesničky ako zoznam referencií.
        pub fn get_all_songs(&self) -> Vec<&SongJks> {
            self.piesne.iter().collect()
        }

        /// Zistí, či správca neobsahuje žiadne pesničky.
        pub fn is_empty(&self) -> bool {
            self.piesne.is_empty()
        }

        /// Vráti všetky pesničky naformátované ako textové riadky.
        pub fn get_format_all(&self) -> Vec<String> {
            self.piesne.iter().map(|s| s.format_song()).collect()
        }

        /// Uloží manažéra pesničiek do JSON súboru.
        pub fn save_to_file_json(&self, path: &str) {
            let file = File::create(path).expect("Chyba otvorenia suboru pre uloženie JSON");
            let writer = BufWriter::new(file);

            serde_json::to_writer_pretty(writer, self).expect("Chyba serializácie manažéra");
        }

        /// Načíta manažéra pesničiek z JSON súboru a po načítaní súbor odstráni.
        pub fn load_manager_from_json(path: &str) -> SongManager {
            let file = File::open(path).expect("Chyba pri otváraní súboru pre načítanie manažéra");
            let reader = BufReader::new(file);

            let ret = serde_json::from_reader(reader).expect("Chyba pri deserializácii manažéra");
            remove_file(path).expect("Subor po manažérovi sa nepodarilo odstrániť");
            ret
        }
    }
}
