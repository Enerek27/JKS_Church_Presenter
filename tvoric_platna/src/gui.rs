use eframe::egui;
use egui::ViewportBuilder;

use monitor_lib::monitor_lib::MonitorGeometry;
use prehladavac_db_jks::library_jks::SongManager;

/// Stav medzi zobrazením textu a čiernou obrazovkou.
pub enum Transition {
    /// Zobrazuje sa text aktuálnej strofy.
    Viewing,
    /// Čierna obrazovka pri prechode medzi pesničkami.
    BlackScreen { direction: Direction },
}

/// Smer prechodu pri čiernej obrazovke.
pub enum Direction {
    /// Prechod na nasledujúcu pesničku.
    Next,
    /// Prechod na predchádzajúcu pesničku.
    Previous,
}

/// Aplikácia na zobrazenie piesní na projektore.
pub struct SongScreenApp {
    manager: SongManager,
    song_idx: usize,
    strofa_idx: usize,
    space_pressed: bool,
    transition: Transition,
}

impl SongScreenApp {
    /// Vytvorí novú aplikáciu so zadaným správcom piesní.
    pub fn new(manager: SongManager) -> Self {
        Self {
            manager,
            song_idx: 0,
            strofa_idx: 0,
            transition: Transition::Viewing,
            space_pressed: false,
        }
    }

    /// Vráti text aktuálnej strofy alebo prázdny reťazec pri blackscreene.
    fn current_text(&self) -> &str {
        match self.transition {
            Transition::BlackScreen { .. } => "",
            Transition::Viewing => self
                .manager
                .piesne
                .get(self.song_idx)
                .and_then(|s| s.strofy.get(self.strofa_idx))
                .map(|s| s.text.as_str())
                .unwrap_or(""),
        }
    }
}

impl eframe::App for SongScreenApp {
    /// Kreslí aktuálny stav na obrazovku (čierne pozadie + text strofy).
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let text = self.current_text();
        egui::Frame::default()
            .fill(egui::Color32::BLACK)
            .show(ui, |ui| {
                let rect = ui.max_rect();
                let available_size = rect.size();

                // Hľadanie najväčšieho použiteľného fontu (binárne vyhľadávanie).
                let mut min = 8.0;
                let mut max = 200.0;
                let mut best = 32.0;

                for _ in 0..12 {
                    let size = (min + max) * 0.5;

                    let galley = ui.fonts_mut(|fonts| {
                        fonts.layout(
                            text.to_string(),
                            egui::FontId::proportional(size),
                            egui::Color32::WHITE,
                            available_size.x,
                        )
                    });

                    let fits =
                        galley.size().x <= available_size.x && galley.size().y <= available_size.y;

                    if fits {
                        best = size;
                        min = size;
                    } else {
                        max = size;
                    }
                }

                ui.with_layout(
                    egui::Layout::centered_and_justified(egui::Direction::TopDown),
                    |ui| {
                        ui.add_sized(
                            available_size,
                            egui::Label::new(
                                egui::RichText::new(text)
                                    .strong()
                                    .size(best)
                                    .color(egui::Color32::WHITE),
                            )
                            .wrap(),
                        );
                    },
                );
            });
    }

    /// Spracováva vstupy z klávesnice a mení stav zobrazenia.
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Escape) {
                std::process::exit(0);
            }

            // Prepínanie manuálneho blackscreenu medzerníkom.
            if i.key_pressed(egui::Key::Space) {
                match self.transition {
                    Transition::BlackScreen { .. } => {
                        self.transition = Transition::Viewing;
                        self.space_pressed = false;
                    }
                    Transition::Viewing => {
                        self.transition = Transition::BlackScreen {
                            direction: Direction::Next,
                        };
                        self.space_pressed = true;
                    }
                };
                return;
            }

            // Režim zobrazenia textu.
            if let Transition::Viewing = self.transition {
                // Nasledujúca strofa / prechod na blackscreen.
                if i.key_pressed(egui::Key::ArrowRight) {
                    if let Some(song) = self.manager.piesne.get(self.song_idx) {
                        if self.strofa_idx + 1 < song.strofy.len() {
                            self.strofa_idx += 1;
                            return;
                        }
                    }

                    self.transition = Transition::BlackScreen {
                        direction: Direction::Next,
                    };
                    return;
                }

                // Predchádzajúca strofa / blackscreen späť.
                if i.key_pressed(egui::Key::ArrowLeft) {
                    if self.strofa_idx > 0 {
                        self.strofa_idx -= 1;
                        return;
                    }

                    self.transition = Transition::BlackScreen {
                        direction: Direction::Previous,
                    };
                    return;
                }

                // Ďalšia pesnička (prvá strofa).
                if i.key_pressed(egui::Key::ArrowUp) {
                    if self.song_idx + 1 < self.manager.piesne.len() {
                        self.song_idx += 1;
                        self.strofa_idx = 0;
                    }
                    return;
                }

                // Predchádzajúca pesnička (prvá strofa).
                if i.key_pressed(egui::Key::ArrowDown) {
                    if self.song_idx > 0 {
                        self.song_idx -= 1;
                        self.strofa_idx = 0;
                    }
                    return;
                }

                return;
            }

            // Režim čiernej obrazovky (automatický prechod medzi piesňami).
            if let Transition::BlackScreen { .. } = self.transition {
                let mut changed = false;

                if self.space_pressed {
                    return;
                }

                // Nasledujúca pesnička.
                if i.key_pressed(egui::Key::ArrowRight) {
                    if self.song_idx + 1 < self.manager.piesne.len() {
                        self.song_idx += 1;
                        self.strofa_idx = 0;
                        changed = true;
                    }
                }

                // Predchádzajúca pesnička (posledná strofa).
                if i.key_pressed(egui::Key::ArrowLeft) {
                    if self.song_idx > 0 {
                        self.song_idx -= 1;

                        let song = &self.manager.piesne[self.song_idx];
                        self.strofa_idx = song.pocet_strof.saturating_sub(1) as usize;

                        changed = true;
                    }
                }

                if changed {
                    self.transition = Transition::Viewing;
                }
            }
        });

        ctx.request_repaint();
    }
}

/// Spustí fullscreen GUI na danom monitore s piesňami zo `SongManager`.
pub fn run_gui(manager: SongManager, geom: MonitorGeometry) -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_decorations(false)
            .with_title("Song Screen")
            .with_inner_size([geom.width, geom.height])
            .with_position([geom.x, geom.y])
            .with_fullscreen(true),
        ..Default::default()
    };

    eframe::run_native(
        "Song Screen",
        options,
        Box::new(move |cc| {
            let mut fonts = eframe::egui::FontDefinitions::default();
            fonts.font_data.insert(
                "inter_bold".to_owned(),
                eframe::egui::FontData::from_static(include_bytes!("./assets/Roboto-Bold.ttf"))
                    .into(),
            );
            fonts
                .families
                .entry(eframe::egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "inter_bold".to_owned());
            cc.egui_ctx.set_fonts(fonts);

            Ok(Box::new(SongScreenApp::new(manager)))
        }),
    )
}
