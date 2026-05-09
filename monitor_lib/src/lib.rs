/// Pomocné typy a funkcie na výber monitora a uloženie jeho geometrie.
pub mod monitor_lib {
    use std::fs::{File, remove_file};
    use std::path::Path;

    use inputbox::InputBox;
    use native_dialog::DialogBuilder;
    use serde::{Deserialize, Serialize};
    use winit::{
        application::ApplicationHandler,
        event::WindowEvent,
        event_loop::{ActiveEventLoop, EventLoop},
        monitor::MonitorHandle,
    };

    /// Pozícia a veľkosť monitora v pixeloch.
    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    pub struct MonitorGeometry {
        pub x: f32,
        pub y: f32,
        pub width: f32,
        pub height: f32,
    }

    /// Konfiguračný súbor s uloženou geometriou monitora.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Config {
        pub monitor: Option<MonitorGeometry>,
    }

    /// Zobrazí chybové hlásenie v natívnom dialógovom okne.
    fn show_error(msg: &str) {
        DialogBuilder::message()
            .set_level(native_dialog::MessageLevel::Error)
            .set_title("Chyba")
            .set_text(msg)
            .confirm()
            .show()
            .unwrap();
    }

    /// Získa zoznam dostupných monitorov pomocou winitu v samostatnom event loope.
    fn get_monitors() -> Vec<MonitorHandle> {
        struct DummyApp {
            monitors: Vec<MonitorHandle>,
        }

        impl ApplicationHandler for DummyApp {
            fn resumed(&mut self, event_loop: &ActiveEventLoop) {
                self.monitors = event_loop.available_monitors().collect();
                event_loop.exit();
            }

            fn window_event(
                &mut self,
                event_loop: &ActiveEventLoop,
                _id: winit::window::WindowId,
                event: WindowEvent,
            ) {
                if let WindowEvent::CloseRequested = event {
                    event_loop.exit();
                }
            }
        }

        let event_loop = EventLoop::new().unwrap();
        let mut app = DummyApp {
            monitors: Vec::new(),
        };
        let _ = event_loop.run_app(&mut app);
        app.monitors
    }

    /// Zobrazí grafický výber monitora a uloží jeho geometriu do JSON konfigu.
    ///
    /// Ak používateľ výber zruší alebo nie sú dostupné žiadne monitory,
    /// funkcia nič nezapíše a ticho skončí.
    pub fn setup_monitor<P: AsRef<Path>>(config_path: P) {
        let monitors = get_monitors();
        if monitors.is_empty() {
            show_error("Nenašiel som žiadne monitory");
            return;
        }

        loop {
            let mut prompt = String::from("Vyber monitor (zadaj číslo):\n\n");
            for (i, m) in monitors.iter().enumerate() {
                let name = m.name().unwrap_or_else(|| "<bez názvu>".to_string());
                let pos = m.position();
                let size = m.size();
                let tag = if pos.x > 0 { " (externý)" } else { "" };

                prompt.push_str(&format!(
                    "{}: {}{}\n   rozlíšenie: {} × {}\n   pozícia: x={}, y={}\n\n",
                    i + 1,
                    name,
                    tag,
                    size.width,
                    size.height,
                    pos.x,
                    pos.y,
                ));
            }

            let result = InputBox::new()
                .title("Výber monitora pre premietanie")
                .prompt(&prompt)
                .show()
                .unwrap();

            let Some(text) = result else {
                return;
            };

            let trimmed = text.trim();
            if trimmed.is_empty() {
                continue;
            }

            match trimmed.parse::<usize>() {
                Ok(n) if n >= 1 && n <= monitors.len() => {
                    let m = &monitors[n - 1];
                    let pos = m.position();
                    let size = m.size();

                    let geom = MonitorGeometry {
                        x: pos.x as f32,
                        y: pos.y as f32,
                        width: size.width as f32,
                        height: size.height as f32,
                    };

                    let cfg = Config {
                        monitor: Some(geom),
                    };

                    let file = File::create(config_path.as_ref())
                        .expect("Neviem vytvoriť config súbor pre monitor");
                    serde_json::to_writer_pretty(file, &cfg)
                        .expect("Chyba pri zápise configu monitora");

                    return;
                }
                _ => {
                    show_error("Zlé číslo monitora");
                }
            }
        }
    }

    /// Načíta geometriu monitora z JSON konfigu a po načítaní súbor odstráni.
    ///
    /// Ak súbor neexistuje alebo nie je platný, vráti `None`.
    pub fn load_monitor_geometry<P: AsRef<Path>>(config_path: P) -> Option<MonitorGeometry> {
        let file = File::open(&config_path).ok()?;
        let cfg: Config = serde_json::from_reader(file).ok()?;
        remove_file(&config_path).expect("Subor na vymazanie neexistuje monitor_lib load");
        cfg.monitor
    }
}
