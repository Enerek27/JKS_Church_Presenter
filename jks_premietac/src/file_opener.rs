#[cfg(target_os = "windows")]
use std::process::Command;
use std::{
    fs::{File, remove_file},
    io::{BufRead, BufReader, BufWriter, Write},
};

use prehladavac_db_jks::{
    db::{db_delete_song, db_insert_song},
    library_jks::{SongJks, SongManager, StrofaJKS},
};

use crate::popups::{ask_song_id, ask_song_type, send_yes_no_messege, show_error};

const FILE_PATH: &str = "./upravujem_text.txt";

pub fn fo_delete_song(song_manager: &mut SongManager, song_for_delete: &SongJks) {
    let msg = format!(
        "{}: {}",
        "Naozaj chces odstranit pesnicky s id", song_for_delete.id
    );
    if !send_yes_no_messege(&msg) {
        return;
    };

    db_delete_song(song_for_delete.id);
    song_manager.remove_song_by_id(song_for_delete.id);
}

pub fn fo_add_song(song_manager: &mut SongManager) {
    let subor = File::create(FILE_PATH).expect("Subor sa Nepodarilo vytvorit");
    let _writer = BufWriter::new(subor); // len vytvoríš prázdny súbor

    open_and_wait(FILE_PATH);

    let mut song_id;
    loop {
        song_id = ask_song_id();
        match song_manager
            .get_all_songs()
            .iter()
            .find(|s| s.id == song_id)
        {
            Some(_) => show_error("Pesnicka s touto id exituje"),
            None => break,
        };
    }

    let typec = ask_song_type();
    let vysledok_typ = match typec {
        Some(r) => r,
        None => vec![],
    };

    let songa = SongJks {
        id: song_id,
        pocet_strof: 0,
        strofy: vec![],
        typ_pesnicky: vysledok_typ,
    };

    nacitaj_zo_suboru(song_manager, &songa);

    remove_file(FILE_PATH).expect("Nepodarilo sa vymazať súbor");
}

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

        // Prázdny riadok = koniec strofy
        if line.trim().is_empty() {
            if !aktualne_riadky.is_empty() {
                let text = aktualne_riadky.join("\n");
                let cislo_strofy = nove_strofy.len() as i32; // 0,1,2,...

                let strofa = StrofaJKS {
                    id: songa_edit.id,
                    cislo_strofy,
                    text,
                };
                nove_strofy.push(strofa);
                aktualne_riadky.clear();
            }
        } else {
            // stále sme v tej istej strofe
            aktualne_riadky.push(line);
        }
    }

    // ak súbor nekončí prázdnym riadkom, ešte poslednú strofu uložiť
    if !aktualne_riadky.is_empty() {
        let text = aktualne_riadky.join("\n");
        let cislo_strofy = nove_strofy.len() as i32;

        let strofa = StrofaJKS {
            id: songa_edit.id,
            cislo_strofy,
            text,
        };
        nove_strofy.push(strofa);
    }

    // poskladáme novú SongJks
    let pocet_strof = (nove_strofy.len() as i32) - 1; // ako u teba
    let nova_songa = SongJks::new(
        songa_edit.id,
        pocet_strof,
        nove_strofy,
        songa_edit.typ_pesnicky.clone(),
    );

    db_delete_song(songa_edit.id);
    song_manager.remove_song_by_id(songa_edit.id);
    db_insert_song(&nova_songa);
    song_manager.add_song(nova_songa);
}

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
