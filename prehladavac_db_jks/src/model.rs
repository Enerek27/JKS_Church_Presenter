use diesel::prelude::*;

use crate::{
    library_jks::{SongJks, StrofaJKS},
    schema::song_types,
};

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::jks)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct JksStrofaDB {
    pub row_id: Option<i32>,
    pub id: i32,
    pub cislo_stofy: i32,
    pub text: String,
}

impl From<&StrofaJKS> for JksStrofaDB {
    fn from(value: &StrofaJKS) -> Self {
        JksStrofaDB {
            row_id: None,
            id: value.id,
            cislo_stofy: value.cislo_strofy,
            text: value.text.clone(),
        }
    }
}

impl From<&JksStrofaDB> for StrofaJKS {
    fn from(value: &JksStrofaDB) -> Self {
        StrofaJKS {
            id: value.id,
            cislo_strofy: value.cislo_stofy,
            text: value.text.clone(),
        }
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::song_types)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct SongTypeDB {
    pub id: Option<i32>, // Nullable<Integer>
    pub name: String,    // Text
}

#[derive(Queryable)]
#[diesel(table_name = crate::schema::jks_types)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct JksTypeDB {
    pub song_id: i32,
    pub type_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = song_types)]
pub struct NewSongTypeDB {
    pub name: String,
}
