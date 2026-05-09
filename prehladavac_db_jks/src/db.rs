use std::collections::{BTreeMap, HashMap};
use std::env;

use diesel::dsl::{delete, insert_into, sql};
use diesel::sql_types::Integer;
use diesel::{
    Connection, ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper, SqliteConnection,
};
use dotenvy::dotenv;

use crate::library_jks::TypPiesne;

use crate::schema;
use crate::schema::jks::dsl::jks;
use crate::schema::jks::id;

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

pub fn db_insert_strofa(strofa: &StrofaJKS, conn: &mut SqliteConnection) {
    let insert_jks: JksStrofaDB = strofa.into();

    diesel::insert_into(schema::jks::table)
        .values(insert_jks)
        .execute(conn)
        .expect("Error inserting into db");
}

pub fn db_insert_song(song: &SongJks) {
    let conn = &mut establish_connection();

    for (idx, strofa) in song.strofy.iter().enumerate() {
        // spravíme kópiu strofy, aby sme jej mohli nastaviť typ
        let mut s = strofa.clone();

        if idx == 0 {
            // prvá strofa dostane typ pesničky
            s.typ_piesne = Some(song.typ_pesnicky);
        } else {
            // ostatné strofy bez typu (NULL v DB)
            s.typ_piesne = None;
        }

        db_insert_strofa(&s, conn);
    }
}

pub fn db_delete_song(cislo_piesne: i32) {
    let conn = &mut establish_connection();

    let _ = diesel::delete(jks.filter(id.eq(cislo_piesne))).execute(conn);
}

pub fn db_load_all() -> SongManager {
    use crate::schema::jks::dsl as jks_dsl; // TOTO je dôležitý riadok

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
