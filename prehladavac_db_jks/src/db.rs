//! Funkcie na prácu s databázou pesničiek (uloženie, načítanie, mazanie).

use std::collections::BTreeMap;
use std::env;
use std::path::PathBuf;

use diesel::{Connection, ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection};
use dotenvy::dotenv;

use crate::library_jks::TypPiesne;
use crate::schema;
use crate::schema::jks::dsl::jks;
use crate::schema::jks::id;
use crate::{
    library_jks::{SongJks, SongManager, StrofaJKS},
    model::JksStrofaDB,
};

/// Vytvorí pripojenie k SQLite databáze podľa premennej prostredia `DATABASE_URL`.
pub fn establish_connection() -> SqliteConnection {
    

    dotenv().ok();

    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        // Fallback: ./jks.sqlite vedľa aktuálneho exe
        let exe_path =
            std::env::current_exe().expect("Failed to get path to current executable");
        let base_dir = exe_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        let default_path: PathBuf = base_dir.join("jks.db");
        default_path.to_string_lossy().to_string()
    });

    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

/// Vloží jednu strofu do tabuľky `jks`.
pub fn db_insert_strofa(strofa: &StrofaJKS, conn: &mut SqliteConnection) {
    let insert_jks: JksStrofaDB = strofa.into();

    diesel::insert_into(schema::jks::table)
        .values(insert_jks)
        .execute(conn)
        .expect("Error inserting into db");
}

/// Vloží celú pesničku do databázy.
///
/// Typ pesničky sa uloží len pri prvej strofe (cislo_strofy = 0),
/// ostatné strofy majú v stĺpci typu hodnotu `NULL`.
pub fn db_insert_song(song: &SongJks) {
    let conn = &mut establish_connection();

    for (idx, strofa) in song.strofy.iter().enumerate() {
        let mut s = strofa.clone();

        if idx == 0 {
            s.typ_piesne = Some(song.typ_pesnicky);
        } else {
            s.typ_piesne = None;
        }

        db_insert_strofa(&s, conn);
    }
}

/// Zmaže všetky strofy pesničky s daným číslom z databázy.
pub fn db_delete_song(cislo_piesne: i32) {
    let conn = &mut establish_connection();

    let _ = diesel::delete(jks.filter(id.eq(cislo_piesne))).execute(conn);
}

/// Načíta všetky pesničky z databázy a vráti ich v `SongManager`.
///
/// Riadky sú zoradené podľa `id` a `cislo_strofy`, zoskupené podľa `id`
/// a pre každú skupinu sa vytvorí inštancia `SongJks`.
pub fn db_load_all() -> SongManager {
    use crate::schema::jks::dsl as jks_dsl;

    let conn = &mut establish_connection();

    let rows: Vec<JksStrofaDB> = jks_dsl::jks
        .order((jks_dsl::id.asc(), jks_dsl::cislo_stofy.asc()))
        .load(conn)
        .expect("Chyba pri načítaní z DB");

    let strofy: Vec<StrofaJKS> = rows.iter().map(|w| w.into()).collect();

    let mut mapa: BTreeMap<i32, Vec<StrofaJKS>> = BTreeMap::new();
    for strofa in strofy {
        mapa.entry(strofa.id).or_default().push(strofa);
    }

    let mut manager = SongManager::new();

    for (id_piesne, strofy_piesne) in mapa {
        let pocet_strof = (strofy_piesne.len() - 1) as i32;

        let typ_pesnicky: TypPiesne = strofy_piesne
            .iter()
            .find(|s| s.cislo_strofy == 0)
            .and_then(|s| s.typ_piesne)
            .unwrap_or(TypPiesne::Hymna);

        manager.add_song(SongJks::new(
            id_piesne,
            pocet_strof,
            strofy_piesne,
            typ_pesnicky,
        ));
    }

    manager
}