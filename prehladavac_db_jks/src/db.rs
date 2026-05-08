use std::collections::BTreeMap;
use std::env;

use diesel::{Connection, ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection};
use dotenvy::dotenv;

use crate::schema;
use crate::schema::jks::dsl::{id, jks};
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

pub fn db_delete_song(cislo_piesne: i32) {
    let conn = &mut establish_connection();

    match diesel::delete(jks.filter(id.eq(cislo_piesne))).execute(conn) {
        Ok(_) => {}
        Err(_) => {}
    }
}

pub fn db_load_all() -> SongManager {
    let conn = &mut establish_connection();

    let rows: Vec<JksStrofaDB> = jks
        .order(cislo_stofy.asc())
        .load(conn)
        .expect("Chyba pri načítaní z DB");

    let mut manager = SongManager::new();
    let strofy: Vec<StrofaJKS> = rows.iter().map(|w| w.into()).collect();
    let mut mapa: BTreeMap<i32, Vec<StrofaJKS>> = BTreeMap::new();

    for strofa in strofy {
        mapa.entry(strofa.id).or_default().push(strofa);
    }

    for (id_piesne, strofy_piesne) in mapa {
        manager.add_song(SongJks::new(
            id_piesne,
            (strofy_piesne.len() - 1) as i32,
            strofy_piesne,
        ));
    }

    manager
}
