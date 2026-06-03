use std::{
    env,
    process::{Command, Stdio},
    thread,
    time::Duration,
};

use crate::{
    event::{AppEvent, Event, EventHandler},
    file_opener::{fo_add_song, fo_delete_song, fo_open_to_edit_song},
    seial_comunikator::{Komunikator, Prikaz, najdi_esp32},
    song_lister::{SongLister, SongListerList, TreeId},
};

use crossterm::event::{KeyCode, KeyEvent};
use prehladavac_db_jks::library_jks::TypPiesne;
use ratatui::DefaultTerminal;
use tokio::sync::oneshot;

#[derive(Debug, PartialEq)]
pub enum FocusedWidget {
    Left,
    Right,
    Search,
}

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub events: EventHandler,
    pub focusing_widget: FocusedWidget,
    pub song_lister: SongLister,
    pub selected_song_lister: SongListerList,
    pub komunikator: Option<Komunikator>,
    pub loading: bool, // ← nové
    pub loading_progress: f32,
    pub pending_komunikator: Option<oneshot::Receiver<Komunikator>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            events: EventHandler::new(),
            focusing_widget: FocusedWidget::Left,
            song_lister: SongLister::new(),
            selected_song_lister: SongListerList::default(),
            komunikator: None,
            loading: false,
            loading_progress: 0.0,
            pending_komunikator: None,
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn premieta(&self) -> bool {
        self.komunikator
            .as_ref()
            .map(|k| k.stav.premietanie_bezi)
            .unwrap_or(false)
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            match self.events.next().await? {
                Event::Tick => self.tick(),
                Event::Crossterm(event) => match event {
                    crossterm::event::Event::Key(key_event)
                        if key_event.kind == crossterm::event::KeyEventKind::Press =>
                    {
                        self.handle_key_events(key_event)?
                    }
                    _ => {}
                },
                Event::App(app_event) => match app_event {
                    AppEvent::Quit => self.quit(),
                    AppEvent::IncrementWidget => self.increment_widget(),
                    AppEvent::DecrementWidget => self.decrement_widget(),
                    AppEvent::IncrementSong => self.increment_song(),
                    AppEvent::DecrementSong => self.decrement_song(),
                    AppEvent::SelectSong => self.select_song(),
                    AppEvent::RemoveSelectedSong => self.remove_selected_song(),
                    AppEvent::EditSong => self.edit_song(),
                    AppEvent::AddSong => self.add_song(),
                    AppEvent::DeleteSong => self.delete_song(),
                    AppEvent::PresentationStart => self.presentation_start(),
                    AppEvent::PresentationStop => self.presentation_stop(),
                    AppEvent::PresentationToggleDark => self.presentation_toggle_dark(),
                    AppEvent::NextSloha => self.next_sloha(),
                    AppEvent::PrevSloha => self.prev_sloha(),
                    AppEvent::NextPiesen => self.next_piesen(),
                    AppEvent::PrevPiesen => self.prev_piesen(),
                    AppEvent::SendCurrentSong => self.presentation_send_current_song(),
                    AppEvent::ConnectSerial => self.connect_serial(),
                    AppEvent::ExpandFolder => self.expand_folder(),
                    AppEvent::CollapseFolder => self.collapse_folder(),
                    AppEvent::LoadingProgress(p) => {
                        self.loading = true;
                        self.loading_progress = p;
                    }
                    AppEvent::LoadingDone => {
                        self.loading = false;
                        self.loading_progress = 1.0;
                    }
                },
            }
        }
        Ok(())
    }

    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        if self.premieta() {
            match key_event.code {
                KeyCode::Up => self.events.send(AppEvent::PrevSloha),
                KeyCode::Down => self.events.send(AppEvent::NextSloha),
                KeyCode::Left => self.events.send(AppEvent::PrevPiesen),
                KeyCode::Right => self.events.send(AppEvent::NextPiesen),
                KeyCode::Char(' ') => self.events.send(AppEvent::PresentationToggleDark),
                KeyCode::End => self.events.send(AppEvent::PresentationStop),
                _ => {}
            }
            return Ok(());
        }

        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Quit),
            KeyCode::Tab => self.events.send(AppEvent::IncrementWidget),
            KeyCode::BackTab => self.events.send(AppEvent::DecrementWidget),
            KeyCode::Up => self.events.send(AppEvent::DecrementSong),
            KeyCode::Down => self.events.send(AppEvent::IncrementSong),
            _ => {}
        }

        if self.focusing_widget == FocusedWidget::Left {
            match key_event.code {
                KeyCode::Char(' ') => self.events.send(AppEvent::SelectSong),
                KeyCode::Enter => self.events.send(AppEvent::EditSong),
                KeyCode::Char('p') => self.events.send(AppEvent::AddSong),
                KeyCode::Delete => self.events.send(AppEvent::DeleteSong),
                KeyCode::Left => self.events.send(AppEvent::CollapseFolder),
                KeyCode::Right => self.events.send(AppEvent::ExpandFolder),
                _ => {}
            }
        } else if self.focusing_widget == FocusedWidget::Right {
            match key_event.code {
                KeyCode::Char(' ') => self.events.send(AppEvent::RemoveSelectedSong),
                KeyCode::Home => self.events.send(AppEvent::PresentationStart),
                _ => {}
            }
        } else if self.focusing_widget == FocusedWidget::Search {
            match key_event.code {
                KeyCode::Char(c) => self.song_lister.search.push(c),
                KeyCode::Backspace => {
                    self.song_lister.search.pop();
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub fn tick(&mut self) {
        if let Some(rx) = self.pending_komunikator.as_mut() {
            if let Ok(kom) = rx.try_recv() {
                self.komunikator = Some(kom);
                self.selected_song_lister.state.select(Some(0));
                self.pending_komunikator = None;
            }
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn increment_widget(&mut self) {
        self.focusing_widget = match self.focusing_widget {
            FocusedWidget::Left => FocusedWidget::Right,
            FocusedWidget::Right => FocusedWidget::Search,
            FocusedWidget::Search => FocusedWidget::Left,
        }
    }

    pub fn decrement_widget(&mut self) {
        self.focusing_widget = match self.focusing_widget {
            FocusedWidget::Left => FocusedWidget::Search,
            FocusedWidget::Right => FocusedWidget::Left,
            FocusedWidget::Search => FocusedWidget::Right,
        }
    }

    pub fn increment_song(&mut self) {
        match self.focusing_widget {
            FocusedWidget::Left => self.song_lister.select_next(),
            FocusedWidget::Right => self.selected_song_lister.select_next(),
            _ => {}
        }
    }

    pub fn decrement_song(&mut self) {
        match self.focusing_widget {
            FocusedWidget::Left => self.song_lister.select_previous(),
            FocusedWidget::Right => self.selected_song_lister.select_previous(),
            _ => {}
        }
    }

    pub fn select_song(&mut self) {
        if self.focusing_widget != FocusedWidget::Left {
            return;
        }
        let (typ_pesnicky, song_id) = match self.get_selected_song_id() {
            Some(id) => id,
            None => return,
        };
        if let Some(song) = self
            .song_lister
            .song_manager
            .get_song_by_id(song_id, typ_pesnicky)
        {
            self.selected_song_lister
                .song_manager
                .add_song(song.clone(), false);
        }
    }

    pub fn get_selected_song_id(&self) -> Option<(TypPiesne, i32)> {
        let last = self.song_lister.state.selected().last()?;
        match last {
            TreeId::Song(typ, id) => Some((typ.clone(), *id)),
            _ => None,
        }
    }

    pub fn remove_selected_song(&mut self) {
        if self.focusing_widget != FocusedWidget::Right {
            return;
        }
        if self.selected_song_lister.song_manager.is_empty() {
            return;
        }
        let index = match self.selected_song_lister.state.selected() {
            Some(i) => i,
            None => return,
        };
        if index < self.selected_song_lister.song_manager.piesne.len() {
            self.selected_song_lister.song_manager.piesne.remove(index);
        }
    }

    pub fn edit_song(&mut self) {
        if self.focusing_widget != FocusedWidget::Left {
            return;
        }
        let (typ_pesnicky, song_id) = match self.get_selected_song_id() {
            Some(id) => id,
            None => return,
        };
        if let Some(song) = self
            .song_lister
            .song_manager
            .get_song_by_id(song_id, typ_pesnicky)
        {
            let copy = song.clone();
            fo_open_to_edit_song(&copy, &mut self.song_lister.song_manager);
        }
    }

    pub fn add_song(&mut self) {
        if self.focusing_widget == FocusedWidget::Left && !self.song_lister.song_manager.is_empty()
        {
            fo_add_song(&mut self.song_lister.song_manager);
        }
    }

    pub fn delete_song(&mut self) {
        if self.focusing_widget != FocusedWidget::Left {
            return;
        }
        let (typ_pesnicky, song_id) = match self.get_selected_song_id() {
            Some(id) => id,
            None => return,
        };
        let song = match self
            .song_lister
            .song_manager
            .get_song_by_id(song_id, typ_pesnicky)
        {
            Some(s) => s.clone(),
            None => return,
        };
        fo_delete_song(&mut self.song_lister.song_manager, &song);
    }

    pub fn collapse_folder(&mut self) {
        if self.focusing_widget == FocusedWidget::Left {
            self.song_lister.collapse();
        }
    }

    pub fn expand_folder(&mut self) {
        if self.focusing_widget == FocusedWidget::Left {
            self.song_lister.expand();
        }
    }

    // ── SERIAL ────────────────────────────────────────────────

    pub fn connect_serial(&mut self) {
        let port = match najdi_esp32() {
            Some(p) => p,
            None => {
                eprintln!("[Serial] ESP32 nenájdený!");
                return;
            }
        };
        match Komunikator::pripoj(&port, 115200) {
            Ok(kom) => {
                self.komunikator = Some(kom);
                #[cfg(debug_assertions)]
                eprintln!("[Serial] Pripojený na {}", port);
            }
            Err(e) => eprintln!("[Serial] Chyba pripojenia: {}", e),
        }
    }

    pub fn je_pripojeny(&self) -> bool {
        self.komunikator.is_some()
    }

    // ── PREMIETANIE ───────────────────────────────────────────

    pub fn presentation_start(&mut self) {
        if self.komunikator.is_none() {
            self.connect_serial();
        }
        let kom = match self.komunikator.take() {
            Some(k) => k,
            None => return,
        };

        let piesne: Vec<(i32, Vec<(i32, String)>)> = self
            .selected_song_lister
            .song_manager
            .piesne
            .iter()
            .enumerate()
            .map(|(idx, s)| {
                let strofy = s
                    .strofy
                    .iter()
                    .map(|st| (st.cislo_strofy, st.text.clone()))
                    .collect();
                (idx as i32, strofy)
            })
            .collect();

        let total_piesni = piesne.len();
        if total_piesni == 0 {
            self.komunikator = Some(kom);
            return;
        }

        // Oneshot channel pre vrátenie komunikátora
        let (kom_tx, kom_rx) = tokio::sync::oneshot::channel::<Komunikator>();

        let sender = self.events.sender();
        self.loading = true;
        self.loading_progress = 0.0;

        tokio::task::spawn_blocking(move || {
            let mut kom = kom;

            sender.send(Event::App(AppEvent::LoadingProgress(0.0))).ok();

            for (i, (idx, strofy)) in piesne.iter().enumerate() {
                let progress = i as f32 / total_piesni as f32;
                sender
                    .send(Event::App(AppEvent::LoadingProgress(progress)))
                    .ok();

                if let Err(e) = kom.odosli_prikaz(Prikaz::PosliPiesen(*idx)) {
                    eprintln!("[Serial] Chyba hlavička piesne idx={}: {}", idx, e);
                    continue;
                }
                thread::sleep(Duration::from_millis(100));

                for (cislo, text) in strofy {
                    let clean_text = text.replace('\n', "^");
                    let chars: Vec<char> = clean_text.chars().collect();
                    let chunks: Vec<String> = chars
                        .chunks(20)
                        .map(|c| c.iter().collect::<String>())
                        .collect();

                    if clean_text.is_empty() {
                        let data = format!("$${}$$%%%", cislo);
                        kom.odosli_retazec(&data).ok();
                        thread::sleep(Duration::from_millis(50));
                    } else {
                        let total_chunks = chunks.len();
                        for (ci, chunk) in chunks.iter().enumerate() {
                            let data = match (ci, total_chunks) {
                                (0, 1) => format!("$${}$${}%%%", cislo, chunk),
                                (0, _) => format!("$${}$${}", cislo, chunk),
                                (n, total) if n == total - 1 => format!("{}%%%", chunk),
                                _ => chunk.to_string(),
                            };
                            kom.odosli_retazec(&data).ok();
                            thread::sleep(Duration::from_millis(50));
                        }
                    }
                    thread::sleep(Duration::from_millis(100));
                }
                thread::sleep(Duration::from_millis(150));
            }

            sender.send(Event::App(AppEvent::LoadingProgress(1.0))).ok();

            if let Err(e) = kom.odosli_prikaz(Prikaz::SpustiPremietanie) {
                eprintln!("[Serial] Chyba pri štarte: {}", e);
            } else {
                thread::sleep(Duration::from_millis(100));
                let _ = kom.odosli_prikaz(Prikaz::PrepniNa {
                    piesen: 0,
                    sloha: 1,
                });
            }

            // Vráť komunikátor cez oneshot – nie cez AppEvent
            kom_tx.send(kom).ok();
            sender.send(Event::App(AppEvent::LoadingDone)).ok();
        });

        // Ulož rx na spracovanie v tick()
        self.pending_komunikator = Some(kom_rx);
    }
    pub fn presentation_stop(&mut self) {
        let kom = match self.komunikator.as_mut() {
            Some(k) => k,
            None => return,
        };
        if let Err(e) = kom.odosli_prikaz(Prikaz::VypniPremietanie) {
            eprintln!("[Serial] Chyba: {}", e);
        }
    }

    pub fn presentation_toggle_dark(&mut self) {
        let kom = match self.komunikator.as_mut() {
            Some(k) => k,
            None => return,
        };
        if kom.stav.blackscreen {
            let _ = kom.odosli_prikaz(Prikaz::Odtmav);
        } else {
            let _ = kom.odosli_prikaz(Prikaz::Zatmav);
        }
    }

    pub fn next_sloha(&mut self) {
        let pocet_sloh = self.get_pocet_sloh();
        // cislo_slohy je 1-based, max = pocet_sloh
        let aktualna = self
            .komunikator
            .as_ref()
            .map(|k| k.stav.cislo_slohy as usize)
            .unwrap_or(1);

        if self.komunikator.is_none() {
            return;
        }

        if aktualna >= pocet_sloh {
            self.next_piesen();
        } else {
            if let Some(k) = self.komunikator.as_mut() {
                if let Err(e) = k.dalsia_sloha() {
                    eprintln!("[Serial] {}", e);
                }
            }
        }
    }

    pub fn prev_sloha(&mut self) {
        // cislo_slohy je 1-based, min = 1
        let aktualna = self
            .komunikator
            .as_ref()
            .map(|k| k.stav.cislo_slohy)
            .unwrap_or(1);

        if self.komunikator.is_none() {
            return;
        }

        if aktualna <= 1 {
            self.prev_piesen_posledna_sloha();
        } else {
            if let Some(k) = self.komunikator.as_mut() {
                if let Err(e) = k.predchadzajuca_sloha() {
                    eprintln!("[Serial] {}", e);
                }
            }
        }
    }

    pub fn next_piesen(&mut self) {
        let aktualna_idx = self.selected_song_lister.state.selected().unwrap_or(0);
        let pocet_piesni = self.selected_song_lister.song_manager.piesne.len();

        if aktualna_idx + 1 >= pocet_piesni {
            return;
        }

        let dalsi_idx = aktualna_idx + 1;
        self.selected_song_lister.state.select(Some(dalsi_idx));

        if let Some(k) = self.komunikator.as_mut() {
            if let Err(e) = k.odosli_prikaz(Prikaz::PrepniNa {
                piesen: dalsi_idx as i32,
                sloha: 1, // vždy začni od prvej slohy
            }) {
                eprintln!("[Serial] {}", e);
            }
        }
    }

    pub fn prev_piesen(&mut self) {
        let aktualna_idx = self.selected_song_lister.state.selected().unwrap_or(0);
        if aktualna_idx == 0 {
            return;
        }

        let predch_idx = aktualna_idx - 1;
        self.selected_song_lister.state.select(Some(predch_idx));

        if let Some(k) = self.komunikator.as_mut() {
            if let Err(e) = k.odosli_prikaz(Prikaz::PrepniNa {
                piesen: predch_idx as i32,
                sloha: 1, // vždy začni od prvej slohy
            }) {
                eprintln!("[Serial] {}", e);
            }
        }
    }

    fn prev_piesen_posledna_sloha(&mut self) {
        let aktualna_idx = self.selected_song_lister.state.selected().unwrap_or(0);
        if aktualna_idx == 0 {
            return;
        }

        let predch_idx = aktualna_idx - 1;
        // strofy[0] = názov, strofy[1..] = slohy → posledná sloha = len - 1
        let posledna_sloha = self
            .selected_song_lister
            .song_manager
            .piesne
            .get(predch_idx)
            .map(|p| p.strofy.len().saturating_sub(1) as i32)
            .unwrap_or(1)
            .max(1);

        self.selected_song_lister.state.select(Some(predch_idx));

        if let Some(k) = self.komunikator.as_mut() {
            if let Err(e) = k.odosli_prikaz(Prikaz::PrepniNa {
                piesen: predch_idx as i32,
                sloha: posledna_sloha,
            }) {
                eprintln!("[Serial] {}", e);
            }
        }
    }

    pub fn presentation_send_current_song(&mut self) {
        let index = match self.selected_song_lister.state.selected() {
            Some(i) => i,
            None => return,
        };
        let song_id = match self.selected_song_lister.song_manager.piesne.get(index) {
            Some(s) => s.id,
            None => return,
        };
        let kom = match self.komunikator.as_mut() {
            Some(k) => k,
            None => {
                eprintln!("[Serial] Nie si pripojený.");
                return;
            }
        };
        if let Err(e) = kom.odosli_prikaz(Prikaz::PrepniNa {
            piesen: song_id,
            sloha: 1,
        }) {
            eprintln!("[Serial] Chyba: {}", e);
        }
    }

    // ── HELPER pre UI ─────────────────────────────────────────

    /// Strofy: index 0 = názov piesne, index 1..N = slohy textu
    /// cislo_slohy z komunikátora je 1-based (1 = prvá sloha textu = strofy[1])
    pub fn get_slohy_pre_zobrazenie(&self) -> (String, String, String) {
        let kom = match self.komunikator.as_ref() {
            Some(k) => k,
            None => return (String::new(), String::new(), String::new()),
        };

        let piesen_idx = kom.stav.cislo_pesnicky as usize;
        let sloha_idx = kom.stav.cislo_slohy as usize; // 1-based

        let piesen = match self
            .selected_song_lister
            .song_manager
            .piesne
            .get(piesen_idx)
        {
            Some(p) => p,
            None => return (String::new(), String::new(), String::new()),
        };

        // strofy[0] = názov, strofy[sloha_idx] = aktuálna sloha
        let get_text = |idx: usize| -> String {
            piesen
                .strofy
                .get(idx)
                .map(|s| s.text.clone())
                .unwrap_or_default()
        };

        let predchadzajuca = if sloha_idx > 1 {
            get_text(sloha_idx - 1)
        } else {
            String::new()
        };
        let aktualna = get_text(sloha_idx);
        let nasledujuca = get_text(sloha_idx + 1);

        (predchadzajuca, aktualna, nasledujuca)
    }

    pub fn get_nazov_piesne(&self) -> String {
        let kom = match self.komunikator.as_ref() {
            Some(k) => k,
            None => return String::new(),
        };
        let piesen_idx = kom.stav.cislo_pesnicky as usize;
        self.selected_song_lister
            .song_manager
            .piesne
            .get(piesen_idx)
            .and_then(|p| p.strofy.get(0))
            .map(|s| s.text.clone())
            .unwrap_or_default()
    }

    pub fn get_pocet_sloh(&self) -> usize {
        let kom = match self.komunikator.as_ref() {
            Some(k) => k,
            None => return 0,
        };
        let piesen_idx = kom.stav.cislo_pesnicky as usize;
        self.selected_song_lister
            .song_manager
            .piesne
            .get(piesen_idx)
            // strofy[0] = názov → skutočné slohy = len - 1
            .map(|p| p.strofy.len().saturating_sub(1))
            .unwrap_or(0)
    }
}
