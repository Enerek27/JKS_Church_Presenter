use diesel::{deserialize::Queryable, prelude::Insertable};

use crate::library_jks::{StrofaJKS, TypPiesne};

/// Jeden riadok z tabuľky `jks` v databáze.
///
/// `JksStrofaDB` je štruktúra určená priamo pre Diesel:
/// - `row_id` je primárny kľúč (autoincrement),
/// - `id` je číslo pesničky,
/// - `cislo_stofy` je poradie strofy,
/// - `typ_piesne` je textová reprezentácia typu (alebo `NULL`),
/// - `text` je obsah strofy.
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

/// Konverzia z doménovej strofy `StrofaJKS` na DB štruktúru `JksStrofaDB`.
///
/// Typ piesne sa uloží ako `String` (pomocou `Display` implementácie),
/// primárny kľúč `row_id` sa necháva `None`, aby ho doplnila databáza.
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

/// Konverzia z DB štruktúry `JksStrofaDB` späť na doménovú `StrofaJKS`.
///
/// Text `typ_piesne` sa premapuje späť na `TypPiesne` pomocou `from_str_db`.
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