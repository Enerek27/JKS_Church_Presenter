use std::collections::{BTreeMap, HashMap};
use std::env;

use diesel::dsl::{delete, insert_into};
use diesel::{
    Connection, ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper, SqliteConnection,
};
use dotenvy::dotenv;

use crate::library_jks::TypPiesne;
use crate::model::{JksTypeDB, NewSongTypeDB, SongTypeDB};
use crate::schema::jks::dsl::{id, jks};
use crate::schema::{self, jks_types};

use crate::schema::song_types::dsl::song_types;

use crate::{
    library_jks::{SongJks, SongManager, StrofaJKS},
    model::JksStrofaDB,
    schema::jks::cislo_stofy,
};

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn db_insert_strofa(strofa: &StrofaJKS) {
    let conn = &mut establish_connection();
    let insert_jks: JksStrofaDB = strofa.into();

    diesel::insert_into(schema::jks::table)
        .values(insert_jks)
        .execute(conn)
        .expect("Error inserting into db");
}

pub fn db_insert_song(song: &SongJks) {
    for strofa in &song.strofy {
        db_insert_strofa(strofa);
    }
}

pub fn reset_song_types() {
    use crate::library_jks::TypPiesne;

    let conn = &mut establish_connection();

    // 1. Zmazať všetky staré riadky v song_types
    delete(song_types)
        .execute(conn)
        .expect("Chyba pri mazani song_types");

    // 2. Pripraviť nové typy podľa enumu
    let typy = TypPiesne::all();

    let nove_typy: Vec<NewSongTypeDB> = typy
        .iter()
        .map(|t| NewSongTypeDB {
            name: t.to_string(),
        })
        .collect();

    // 3. Vložiť ich do song_types
    insert_into(song_types)
        .values(&nove_typy)
        .execute(conn)
        .expect("Chyba pri vkladani do song_types");
}

pub fn db_delete_song(cislo_piesne: i32) {
    let conn = &mut establish_connection();

    use crate::schema::jks::dsl::id;

    match diesel::delete(jks.filter(id.eq(cislo_piesne))).execute(conn) {
        Ok(_) => {}
        Err(_) => {}
    }
}

pub fn db_load_all() -> SongManager {
    let conn = &mut establish_connection();

    // 1. Načítať všetky strofy z jks
    let rows: Vec<JksStrofaDB> = jks
        .order(cislo_stofy.asc())
        .load(conn)
        .expect("Chyba pri načítaní z DB");

    // mapa row_id -> id (cislo piesne z JKS)
    let mut rowid_to_songid: HashMap<i32, i32> = HashMap::new();
    for r in &rows {
        if let Some(row_id) = r.row_id {
            rowid_to_songid.insert(row_id, r.id);
        }
    }

    // 2. Načítať všetky typy a väzby jks_types

    let all_song_types: Vec<SongTypeDB> = song_types
        .select(SongTypeDB::as_select())
        .load(conn)
        .expect("Chyba pri načítaní song_types");

    let all_links: Vec<JksTypeDB> = jks_types::table
        .load(conn)
        .expect("Chyba pri načítaní jks_types");

    // mapa type_id -> TypPiesne
    let mut typeid_to_enum: HashMap<i32, TypPiesne> = HashMap::new();
    for st in all_song_types {
        if let Some(t) = TypPiesne::from_str_db(&st.name) {
            if let Some(type_id_valid) = st.id {
                typeid_to_enum.insert(type_id_valid, t);
            }
        }
    }

    // mapa id_piesne_z_JKS -> Vec<TypPiesne>
    let mut songid_to_types: HashMap<i32, Vec<TypPiesne>> = HashMap::new();
    for link in all_links {
        if let Some(song_id_jks) = rowid_to_songid.get(&link.song_id) {
            if let Some(typ_enum) = typeid_to_enum.get(&link.type_id) {
                songid_to_types
                    .entry(*song_id_jks)
                    .or_default()
                    .push(typ_enum.clone());
            }
        }
    }

    // 3. Previesť DB model na doménové strofy
    let strofy: Vec<StrofaJKS> = rows.iter().map(|w| w.into()).collect();

    // 4. Zoskupiť strofy podľa id piesne (cislo z JKS)
    let mut mapa: BTreeMap<i32, Vec<StrofaJKS>> = BTreeMap::new();
    for strofa in strofy {
        mapa.entry(strofa.id).or_default().push(strofa);
    }

    // 5. Poskladať SongManager
    let mut manager = SongManager::new();

    for (id_piesne, strofy_piesne) in mapa {
        let pocet_strof = (strofy_piesne.len() - 1) as i32;
        let typy = songid_to_types.remove(&id_piesne).unwrap_or_else(Vec::new);

        manager.add_song(SongJks::new(id_piesne, pocet_strof, strofy_piesne, typy));
    }

    manager
}
