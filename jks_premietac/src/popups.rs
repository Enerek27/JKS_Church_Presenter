use std::sync::{Arc, Mutex};

use eframe::egui::{self, Ui};
use inputbox::InputBox;
use native_dialog::DialogBuilder;
use prehladavac_db_jks::library_jks::TypPiesne;

pub struct GuiWrapper {
    pub inner: AppGui,
    pub result: Arc<Mutex<Option<Vec<TypPiesne>>>>,
}

impl eframe::App for GuiWrapper {
    fn ui(&mut self, ctx: &mut Ui, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ctx, |ui| self.inner.ui(ui));

        if self.inner.is_finished() {
            let res = self.inner.clone().result();
            if let Ok(mut lock) = self.result.lock() {
                *lock = res;
            }
            // zatvoriť okno v novom API
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

#[derive(Clone)]
pub struct AppGui {
    pub items: Vec<(TypPiesne, bool)>,
    pub selected_index: usize,
    pub done: bool,
    pub cancelled: bool,
}

impl AppGui {
    pub fn ui(&mut self, ui: &mut Ui) {
        ui.heading("Vyber typy pesničky");

        if !self.items.is_empty() && ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
            self.selected_index = (self.selected_index + 1).min(self.items.len() - 1);
        }

        if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
            self.selected_index = self.selected_index.saturating_sub(1);
        }

        if !self.items.is_empty() && ui.input(|i| i.key_pressed(egui::Key::Space)) {
            let item = &mut self.items[self.selected_index];
            item.1 = !item.1;
        }

        if ui.button("OK").clicked() {
            self.done = true;
        }

        if ui.button("Cancel").clicked() {
            self.cancelled = true;
        }

        for (i, (typ, selected)) in self.items.iter_mut().enumerate() {
            let label = if *selected {
                format!("✔ {}", typ)
            } else {
                format!("  {}", typ)
            };

            let response = ui.selectable_label(i == self.selected_index, label);

            if response.clicked() {
                *selected = !*selected;
                self.selected_index = i;
            }

            if response.hovered() {
                self.selected_index = i;
            }
        }

        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("OK").clicked() {
                self.done = true;
            }

            if ui.button("Cancel").clicked() {
                self.cancelled = true;
            }
        });
    }

    pub fn is_finished(&self) -> bool {
        self.done || self.cancelled
    }

    pub fn result(self) -> Option<Vec<TypPiesne>> {
        if self.cancelled {
            return None;
        }

        Some(
            self.items
                .into_iter()
                .filter(|(_, selected)| *selected)
                .map(|(t, _)| t)
                .collect(),
        )
    }
}

pub fn show_error(msg: &str) {
    DialogBuilder::message()
        .set_level(native_dialog::MessageLevel::Error)
        .set_title("Chyba")
        .set_text(msg)
        .confirm()
        .show()
        .unwrap();
}

pub fn open_editor_and_wait(path: &str) {
    let editor = "mousepad";
    use std::process::Command;

    Command::new(editor)
        .arg(path)
        .status()
        .expect("Nepodarilo sa otvoriť editor linux");
}

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

pub fn send_yes_no_messege(msg: &str) -> bool {
    DialogBuilder::message()
        .set_level(native_dialog::MessageLevel::Info)
        .set_title("Otazka")
        .set_text(msg)
        .confirm()
        .show()
        .unwrap_or(false)
}

pub fn ask_song_type() -> Option<Vec<TypPiesne>> {
    let options = TypPiesne::all(); // &'static [TypPiesne]

    let app = AppGui {
        items: options.iter().copied().map(|t| (t, false)).collect(),
        selected_index: 0,
        done: false,
        cancelled: false,
    };

    let result: Arc<Mutex<Option<Vec<TypPiesne>>>> = Arc::new(Mutex::new(None));
    let result_clone = Arc::clone(&result);

    let _ = eframe::run_native(
        "Vyber typy",
        eframe::NativeOptions::default(),
        Box::new(move |_cc| {
            Ok::<Box<dyn eframe::App>, Box<dyn std::error::Error + Send + Sync>>(Box::new(
                GuiWrapper {
                    inner: app,
                    result: result_clone,
                },
            ))
        }),
    );

    Arc::try_unwrap(result)
        .ok()
        .and_then(|m| m.into_inner().ok().and_then(|r| r))
}
