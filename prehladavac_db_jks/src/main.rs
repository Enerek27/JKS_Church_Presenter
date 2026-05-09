use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead, BufReader},
};

use prehladavac_db_jks::{
    db::db_insert_song,
    library_jks::{SongJks, StrofaJKS, TypPiesne},
};

/// Načíta piesne z `jks.csv` a importuje ich do databázy.
///
/// CSV stĺpce:
/// `id,cislo_strofy,typ,text`
/// - `id`: číslo piesne
/// - `cislo_strofy`: poradové číslo strofy (0 = názov)
/// - `typ`: `"none"` alebo textový názov typu (napr. „Advent“)
/// - `text`: text strofy, `\n` sa nahrádza za reálny nový riadok
fn init_db() {
    let path = "jks.csv";

    let file = File::open(path).expect("Neviem otvoriť CSV súbor");
    let reader = BufReader::new(file);

    // mapa: id_piesne -> (zoznam strof, zistený typ piesne)
    let mut songs: BTreeMap<i32, (Vec<StrofaJKS>, Option<TypPiesne>)> = BTreeMap::new();

    for (line_no, line_res) in reader.lines().enumerate() {
        let line = match line_res {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Riadok {}: chyba čítania: {}", line_no + 1, e);
                continue;
            }
        };

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Rozparsuj: id,cislo_strofy,typ,text (s jednoduchou podporou úvodzoviek).
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;

        for c in line.chars() {
            match c {
                '"' => {
                    in_quotes = !in_quotes;
                    current.push(c);
                }
                ',' if !in_quotes => {
                    parts.push(current.trim().to_string());
                    current.clear();
                }
                _ => current.push(c),
            }
        }
        if !current.is_empty() {
            parts.push(current.trim().to_string());
        }

        if parts.len() < 4 {
            eprintln!(
                "Riadok {}: očakávam 4 stĺpce (id,cislo_strofy,typ,text), mám: {:?}",
                line_no + 1,
                parts
            );
            continue;
        }

        // id piesne
        let id: i32 = match parts[0].parse() {
            Ok(v) => v,
            Err(e) => {
                eprintln!(
                    "Riadok {}: neviem parsovať id: {} ({:?})",
                    line_no + 1,
                    e,
                    parts[0]
                );
                continue;
            }
        };

        // číslo strofy
        let cislo_strofy: i32 = match parts[1].parse() {
            Ok(v) => v,
            Err(e) => {
                eprintln!(
                    "Riadok {}: neviem parsovať cislo_strofy: {} ({:?})",
                    line_no + 1,
                    e,
                    parts[1]
                );
                continue;
            }
        };

        // typ piesne: "none" alebo názov
        let typ_raw = parts[2].trim().trim_matches('"');

        let typ_opt: Option<TypPiesne> = if typ_raw.eq_ignore_ascii_case("none") {
            None
        } else {
            match TypPiesne::from_str_db(typ_raw) {
                Some(t) => Some(t),
                None => {
                    eprintln!(
                        "Riadok {}: neznámy typ piesne {:?}, nastavujem None",
                        line_no + 1,
                        typ_raw
                    );
                    None
                }
            }
        };

        // text strofy – prelož \n na skutočný nový riadok
        let mut text_raw = parts[3].trim().trim_matches('"').to_string();
        text_raw = text_raw.replace("\\n", "\n");

        let strofa = StrofaJKS {
            id,
            cislo_strofy,
            typ_piesne: typ_opt,
            text: text_raw,
        };

        let entry = songs.entry(id).or_insert_with(|| (Vec::new(), None));
        entry.0.push(strofa);

        // ak tento riadok má typ a ešte nemáme typ pesničky, ulož ho
        if let Some(t) = typ_opt {
            if entry.1.is_none() {
                entry.1 = Some(t);
            }
        }
    }

    // Teraz máme všetky pesničky v mape, tak ich vložíme do DB.
    for (id, (mut strofy, typ_opt)) in songs {
        // zoradíme strofy podľa čísla
        strofy.sort_by_key(|s| s.cislo_strofy);

        let pocet_strof = (strofy.len() as i32) - 1;
        let typ_pesnicky = typ_opt.unwrap_or(TypPiesne::Hymna);

        let song = SongJks::new(id, pocet_strof, strofy, typ_pesnicky);

        db_insert_song(&song);
        println!("Importovaná pesnička id {}", id);
    }
}

fn main() {
    // Jednorazový import – nechávame zakomentované, aby sa to
    // nespúšťalo pri každom behu.
    // init_db();
}
