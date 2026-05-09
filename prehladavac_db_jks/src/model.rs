use diesel::{deserialize::Queryable, prelude::Insertable};

use crate::library_jks::{StrofaJKS, TypPiesne};

#[derive(Queryable, Insertable)]
#[diesel(table_name = crate::schema::jks)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct JksStrofaDB {
    pub row_id: Option<i32>,
    pub id: i32,
    pub cislo_stofy: i32,
    pub typ_piesne: Option<String>, // Nullable<Text> → Option<String>
    pub text: String,
}

impl From<&StrofaJKS> for JksStrofaDB {
    fn from(s: &StrofaJKS) -> Self {
        JksStrofaDB {
            row_id: None,
            id: s.id,
            cislo_stofy: s.cislo_strofy,
            typ_piesne: s.typ_piesne.as_ref().map(|t| t.to_string()),
            text: s.text.clone(),
        }
    }
}

impl From<&JksStrofaDB> for StrofaJKS {
    fn from(db: &JksStrofaDB) -> Self {
        let typ_piesne = db
            .typ_piesne
            .as_ref()
            .and_then(|s| TypPiesne::from_str_db(s));

        StrofaJKS {
            id: db.id,
            cislo_strofy: db.cislo_stofy,
            typ_piesne,
            text: db.text.clone(),
        }
    }
}
