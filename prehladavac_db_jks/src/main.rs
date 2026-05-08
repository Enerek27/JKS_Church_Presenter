use std::{collections::BTreeMap, error::Error, fs::File};

use prehladavac_db_jks::{
    db::db_insert_song,
    library_jks::{SongJks, StrofaJKS},
};

// -> Result<(), Box<dyn Error>>
fn main() {
    /*  // 1. Otvor CSV súbor
    let file = File::open("jks.csv")?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false) // podľa tvojho príkladu
        .from_reader(file);

    // 2. Zoskupiť strofy podľa id piesne
    let mut mapa: BTreeMap<i32, Vec<StrofaJKS>> = BTreeMap::new();

    for result in rdr.records() {
        let record = result?;

        // Očakávame 3 stĺpce: id_piesne, cislo_strofy, text
        let id_piesne: i32 = record[0].parse()?;
        let cislo_strofy: i32 = record[1].parse()?;
        let raw_text = record[2].to_string();

        // Nahradenie " - " za nové riadky
        let text = raw_text.replace(" - ", "\n");

        let strofa = StrofaJKS {
            id: id_piesne,
            cislo_strofy,
            text,
        };

        mapa.entry(id_piesne).or_default().push(strofa);
    }

    // 3. Z každej skupiny spraviť SongJks a vložiť do DB
    for (id_piesne, mut strofy_piesne) in mapa {
        // zoradiť podľa cislo_strofy, aby bola 0,1,2,3,...
        strofy_piesne.sort_by_key(|s| s.cislo_strofy);

        // pocet_strof = bez názvu, ak ho máš v strofe 0
        let pocet_strof = (strofy_piesne.len() as i32) - 1;

        let song = SongJks::new(id_piesne, pocet_strof, strofy_piesne);
        db_insert_song(&song);
    }

    Ok(()) */
}
