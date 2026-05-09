use inputbox::InputBox;
use native_dialog::DialogBuilder;
use prehladavac_db_jks::library_jks::TypPiesne;

/// Zobrazí chybové hlásenie v natívnom dialógovom okne.
pub fn show_error(msg: &str) {
    DialogBuilder::message()
        .set_level(native_dialog::MessageLevel::Error)
        .set_title("Chyba")
        .set_text(msg)
        .confirm()
        .show()
        .unwrap();
}

/// Otvorí externý editor a počká, kým používateľ súbor zatvorí.
pub fn open_editor_and_wait(path: &str) {
    let editor = "mousepad";
    use std::process::Command;

    Command::new(editor)
        .arg(path)
        .status()
        .expect("Nepodarilo sa otvoriť editor linux");
}

/// Opakovane sa pýta na ID pesničky, kým používateľ nezadá platné celé číslo.
pub fn ask_song_id() -> i32 {
    loop {
        let result = InputBox::new()
            .title("Zadaj pravdivo")
            .prompt("Zadaj id novej pesničky (číslo)")
            .show()
            .unwrap();

        let Some(text) = result else {
            show_error("Nebolo nic zadane");
            continue;
        };

        match text.trim().parse() {
            Ok(c) => return c,
            Err(_) => {}
        };

        show_error("Zle zadané číslo");
    }
}

/// Zobrazí otázku typu „áno/nie“ a vráti `true` pri potvrdení.
pub fn send_yes_no_messege(msg: &str) -> bool {
    DialogBuilder::message()
        .set_level(native_dialog::MessageLevel::Info)
        .set_title("Otazka")
        .set_text(msg)
        .confirm()
        .show()
        .unwrap_or(false)
}

/// Nechá používateľa vybrať typ piesne zo zoznamu a vráti zvolený `TypPiesne`.
///
/// Ak používateľ výber zruší alebo nie sú dostupné žiadne typy, vráti `None`.
pub fn ask_song_type() -> Option<TypPiesne> {
    let all = TypPiesne::all();

    if all.is_empty() {
        return None;
    }

    loop {
        let mut prompt = String::from("Vyber typ piesne (zadaj číslo):\n\n");
        for (i, t) in all.iter().enumerate() {
            prompt.push_str(&format!("{}: {}\n", i + 1, t.to_string()));
        }

        let result = InputBox::new()
            .title("Typ piesne")
            .prompt(&prompt)
            .show()
            .unwrap();

        let Some(text) = result else {
            return None;
        };

        let trimmed = text.trim();
        if trimmed.is_empty() {
            continue;
        }

        match trimmed.parse::<usize>() {
            Ok(n) if n >= 1 && n <= all.len() => {
                return Some(all[n - 1]);
            }
            _ => {
                show_error("Zlé číslo typu piesne");
            }
        }
    }
}