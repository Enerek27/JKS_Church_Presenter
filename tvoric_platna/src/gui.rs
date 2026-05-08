use eframe::egui;
use egui::{Align, Color32, FontFamily, Frame, Layout, RichText, ViewportBuilder};

use prehladavac_db_jks::library_jks::SongManager;
use winit::event_loop::EventLoop;

pub enum Transition {
    Viewing,
    BlackScreen { direction: Direction },
}

pub enum Direction {
    Next,
    Previous,
}

pub struct SongScreenApp {
    manager: SongManager,
    song_idx: usize,
    strofa_idx: usize,
    //blackscreen: bool,
    transition: Transition,
    
    
}

impl SongScreenApp {
    pub fn new(manager: SongManager) -> Self {
        Self {
            manager,
            song_idx: 0,
            strofa_idx: 0,
            transition: Transition::Viewing,
            //blackscreen: false,
        }
    }

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

    // TU JE TVOJA ui() METÓDA
}

impl eframe::App for SongScreenApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let text = self.current_text();
        egui::Frame::default()
            .fill(egui::Color32::BLACK)
            .show(ui, |ui| {
                let rect = ui.max_rect();

                let available_size = rect.size();

                // Hľadanie najväčšieho možného fontu
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

    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.input(|i| {

            // ❌ ESC = exit app
            if i.key_pressed(egui::Key::Escape) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return;
            }

            // =========================
            // SPACE = MANUAL BLACKSCREEN TOGGLE
            // =========================
            if i.key_pressed(egui::Key::Space) {
                self.transition = match self.transition {
                    Transition::BlackScreen { .. } => Transition::Viewing,
                    Transition::Viewing => Transition::BlackScreen {
                        direction: Direction::Next, // default (nevadí, nepoužije sa pri manuálnom)
                    },
                };
                /*
                if self.blackscreen {
                    self.blackscreen = !self.blackscreen;
                }
                */
                return;
            }

            // =========================
            // VIEWING MODE
            // =========================
            if let Transition::Viewing = self.transition {
                // ➡ NEXT / STROFA
                if i.key_pressed(egui::Key::ArrowRight) {
                    if let Some(song) = self.manager.piesne.get(self.song_idx) {
                        if self.strofa_idx + 1 < song.strofy.len() {
                            self.strofa_idx += 1;
                            return;
                        }
                    }

                    // koniec piesne → transition
                    self.transition = Transition::BlackScreen {
                        direction: Direction::Next,
                    };
                    return;
                }

                // ⬅ PREVIOUS / STROFA
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

                return;
            }

            // =========================
            // BLACKSCREEN MODE
            // =========================
            if let Transition::BlackScreen { .. } = self.transition {
                let mut changed = false;
                /*
                if self.blackscreen {
                    return;
                }
                */
                // ➡ NEXT SONG
                if i.key_pressed(egui::Key::ArrowRight) {
                    if self.song_idx + 1 < self.manager.piesne.len() {
                        self.song_idx += 1;
                        self.strofa_idx = 0;
                        changed = true;
                    }
                }

                // ⬅ PREVIOUS SONG
                if i.key_pressed(egui::Key::ArrowLeft) {
                    if self.song_idx > 0 {
                        self.song_idx -= 1;

                        let song = &self.manager.piesne[self.song_idx];
                        self.strofa_idx = song.pocet_strof.saturating_sub(1) as usize;

                        changed = true;
                    }
                }

                // návrat do normálu iba ak sa niečo zmenilo
                if changed {
                    self.transition = Transition::Viewing;
                }
            }
        });

        ctx.request_repaint();
    }
}

pub fn run_gui(manager: SongManager) -> eframe::Result<()> {



    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_fullscreen(true)
            .with_decorations(false)
            .with_title("Song Screen"),
        ..Default::default()
    };

    eframe::run_native(
        "Song Screen",
        options,
        Box::new(|cc| {
            // ===== FONT SETUP =====
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
