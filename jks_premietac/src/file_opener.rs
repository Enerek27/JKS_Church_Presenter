#[cfg(target_os = "windows")]
use std::process::Command;
use std::{
    fs::{File, remove_file},
    io::{BufRead, BufReader, BufWriter, Write},
};

use prehladavac_db_jks::{
    db::{db_delete_song, db_insert_song},
    library_jks::{JKSTypPiesne, SongJks, SongManager, StrofaJKS, TypPiesne},
};

use crate::popups::{ask_song_id, ask_song_type, send_yes_no_messege, show_error};

const FILE_PATH: &str = "./upravujem_text.txt";

fn song_id_existing(typ_piesne: TypPiesne, song_manager: &SongManager, song_id: i32) -> bool {
    if matches!(typ_piesne, TypPiesne::JKS(_)) {
        match song_manager
            .get_all_songs()
            .iter()
            .filter(|s| matches!(s.typ_pesnicky, TypPiesne::JKS(_)))
            .find(|s| s.id == song_id)
        {
            Some(_) => true,
            None => false,
        }
    } else {
        match song_manager
            .get_all_songs()
            .iter()
            .filter(|s| s.typ_pesnicky == typ_piesne)
            .find(|s| s.id == song_id)
        {
            Some(_) => true,
            None => false,
        }
    }
}

/// Odstráni pesničku z databázy aj z `SongManager` po potvrdení používateľom.
pub fn fo_delete_song(song_manager: &mut SongManager, song_for_delete: &SongJks) {
    let msg = format!(
        "{}: {}",
        "Naozaj chces odstranit pesnicky s id", song_for_delete.id
    );
    if !send_yes_no_messege(&msg) {
        return;
    };

    db_delete_song(song_for_delete.id, song_for_delete.typ_pesnicky);
    song_manager.remove_song_by_id(song_for_delete.id, song_for_delete.typ_pesnicky);
}

/// Vytvorí novú pesničku tak, že otvorí dočasný textový súbor v editore,
/// načíta z neho strofy a uloží pesničku do DB aj do `SongManager`.
pub fn fo_add_song(song_manager: &mut SongManager) {
    let subor = File::create(FILE_PATH).expect("Subor sa Nepodarilo vytvorit");
    let _writer = BufWriter::new(subor);

    open_and_wait(FILE_PATH);

    let typ = match ask_song_type() {
        Some(t) => t,
        None => {
            show_error("Musíš vybrať typ pesničky");
            return;
        }
    };

    let mut song_id;
    loop {
        song_id = ask_song_id();

        if song_id_existing(typ, song_manager, song_id) {
            show_error("Pesnicka s touto id exituje");
        } else {
            break;
        }
    }

    let songa = SongJks {
        id: song_id,
        pocet_strof: 0,
        strofy: vec![],
        typ_pesnicky: typ,
    };

    nacitaj_zo_suboru(song_manager, &songa);

    remove_file(FILE_PATH).expect("Nepodarilo sa vymazať súbor");
}

/// Otvorí existujúcu pesničku v textovom editore, umožní úpravu,
/// a po uložení prepíše pesničku v DB aj v `SongManager`.
pub fn fo_open_to_edit_song(songa: &SongJks, song_manager: &mut SongManager) {
    let subor = File::create(FILE_PATH).expect("Subor sa Nepodarilo vytvorit");

    let mut writer = BufWriter::new(subor);

    for strofa in &songa.strofy {
        let line = format!("{}", strofa.text);
        writer
            .write(line.as_bytes())
            .expect("Chyba zapisu do suboru");
        writer.write(b"\n\n").expect("Chyba zápisu noveho riadka");
    }
    writer.flush().expect("Chyba pri flushnuti");

    open_and_wait(FILE_PATH);

    nacitaj_zo_suboru(song_manager, songa);

    remove_file(FILE_PATH).expect("Nepodarilo sa vymazať súbor");
}

/// Načíta strofy z dočasného textového súboru a vytvorí z nich novú verziu pesničky.
fn nacitaj_zo_suboru(song_manager: &mut SongManager, songa_edit: &SongJks) {
    let subor = File::open(FILE_PATH).expect("Subor sa nepodarilo otvorit na citanie");
    let reader = BufReader::new(subor);
    let mut nove_strofy: Vec<StrofaJKS> = vec![];
    let mut aktualne_riadky: Vec<String> = vec![];

    for line_opt in reader.lines() {
        let line = match line_opt {
            Ok(l) => l,
            Err(_) => continue,
        };

        if line.trim().is_empty() {
            if !aktualne_riadky.is_empty() {
                let text = aktualne_riadky.join("\n");
                let cislo_strofy = nove_strofy.len() as i32;

                let strofa = StrofaJKS {
                    id: songa_edit.id,
                    cislo_strofy,
                    typ_piesne: Some(songa_edit.typ_pesnicky),
                    text,
                };
                nove_strofy.push(strofa);
                aktualne_riadky.clear();
            }
        } else {
            aktualne_riadky.push(line);
        }
    }

    if !aktualne_riadky.is_empty() {
        let text = aktualne_riadky.join("\n");
        let cislo_strofy = nove_strofy.len() as i32;

        let strofa = StrofaJKS {
            id: songa_edit.id,
            cislo_strofy,
            typ_piesne: Some(songa_edit.typ_pesnicky),
            text,
        };
        nove_strofy.push(strofa);
    }

    let pocet_strof = (nove_strofy.len() as i32) - 1;
    let nova_songa = SongJks::new(
        songa_edit.id,
        pocet_strof,
        nove_strofy,
        songa_edit.typ_pesnicky,
    );

    db_delete_song(songa_edit.id, songa_edit.typ_pesnicky);
    song_manager.remove_song_by_id(songa_edit.id, songa_edit.typ_pesnicky);
    db_insert_song(&nova_songa);
    song_manager.add_song(nova_songa);
}

/// Spustí externý editor a počká, kým používateľ súbor zavrie.
fn open_and_wait(path: &str) {
    #[cfg(target_os = "windows")]
    {
        Command::new("notepad")
            .arg(path)
            .status()
            .expect("Nepodarilo sa spustiť notepad");
    }

    #[cfg(target_os = "linux")]
    {
        use crate::popups::open_editor_and_wait;

        open_editor_and_wait(path);
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        Command::new("open")
            .args(["-W", path])
            .status()
            .expect("Nepodarilo sa spustiť editor");
    }
}
