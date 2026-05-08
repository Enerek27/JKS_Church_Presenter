use std::{
    env,
    process::{Command, Stdio},
};

use crate::{
    event::{AppEvent, Event, EventHandler},
    file_opener::{fo_add_song, fo_delete_song, fo_open_to_edit_song},
    song_lister::SongLister,
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::DefaultTerminal;





#[derive(Debug, PartialEq)]
pub enum FocusedWidget {
    Left,
    Right,
    Search,
}

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,

    /// Event handler.
    pub events: EventHandler,

    pub focusing_widget: FocusedWidget,

    pub song_lister: SongLister,

    pub selected_song_lister: SongLister,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,

            events: EventHandler::new(),
            focusing_widget: FocusedWidget::Left,
            song_lister: SongLister::new(),
            selected_song_lister: SongLister::default(),
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Run the application's main loop.
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
                },
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Quit),
            KeyCode::Tab => self.events.send(AppEvent::IncrementWidget),
            KeyCode::BackTab => self.events.send(AppEvent::DecrementWidget),
            KeyCode::Up => self.events.send(AppEvent::DecrementSong),
            KeyCode::Down => self.events.send(AppEvent::IncrementSong),
            // Other handlers you could add here.
            _ => {}
        }
        if self.focusing_widget == FocusedWidget::Left {
            match key_event.code {
                KeyCode::Char(' ') => self.events.send(AppEvent::SelectSong),
                KeyCode::Enter => self.events.send(AppEvent::EditSong),
                KeyCode::Char('p') => self.events.send(AppEvent::AddSong),
                KeyCode::Delete => self.events.send(AppEvent::DeleteSong),
                // Other handlers you could add here.
                _ => {}
            }
        } else if self.focusing_widget == FocusedWidget::Right {
            match key_event.code {
                KeyCode::Char(' ') => self.events.send(AppEvent::RemoveSelectedSong),
                KeyCode::Home => self.events.send(AppEvent::PresentationStart),
                // Other handlers you could add here.
                _ => {}
            }
        } else if self.focusing_widget == FocusedWidget::Search {
            match key_event.code {
                KeyCode::Char(c) => self.song_lister.search.push(c),
                KeyCode::Backspace => {
                    self.song_lister.search.pop();
                }
                // Other handlers you could add here.
                _ => {}
            }
        }
        Ok(())
    }

    /// Handles the tick event of the terminal.
    ///
    /// The tick event is where you can update the state of your application with any logic that
    /// needs to be updated at a fixed frame rate. E.g. polling a server, updating an animation.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
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
        if self.focusing_widget == FocusedWidget::Left {
            self.song_lister.select_next();
        } else if self.focusing_widget == FocusedWidget::Right {
            self.selected_song_lister.select_next();
        }
    }

    pub fn decrement_song(&mut self) {
        if self.focusing_widget == FocusedWidget::Left {
            self.song_lister.select_previous();
        } else if self.focusing_widget == FocusedWidget::Right {
            self.selected_song_lister.select_previous();
        }
    }

    pub fn select_song(&mut self) {
        if self.focusing_widget == FocusedWidget::Left || !self.song_lister.song_manager.is_empty()
        {
            let index = match self.song_lister.state.selected() {
                Some(i) => i,
                None => 0,
            };
            let string = match self.song_lister.search_get_formated().get(index) {
                Some(s) => s.to_lowercase(),
                None => String::new(),
            };

            let cislo_opt: Option<i32> = string
                .split_whitespace()
                .next()
                .and_then(|num_str| num_str.parse::<i32>().ok());

            if let Some(idecko) = cislo_opt {
                let pesnicka_opt = self.song_lister.song_manager.get_song_by_id(idecko);
                if let Some(pesnicka) = pesnicka_opt {
                    self.selected_song_lister
                        .song_manager
                        .add_song(pesnicka.clone());
                }
            }
        }
    }

    pub fn remove_selected_song(&mut self) {
        if self.focusing_widget == FocusedWidget::Right
            && !self.selected_song_lister.song_manager.is_empty()
        {
            let index = match self.selected_song_lister.state.selected() {
                Some(i) => i,
                None => 0,
            };
            self.selected_song_lister.song_manager.piesne.remove(index);
        }
    }

    pub fn edit_song(&mut self) {
        if self.focusing_widget == FocusedWidget::Left && !self.song_lister.song_manager.is_empty()
        {
            let index = match self.song_lister.state.selected() {
                Some(i) => i,
                None => 0,
            };

            let string = match self.song_lister.search_get_formated().get(index) {
                Some(s) => s.to_lowercase(),
                None => String::new(),
            };

            let cislo_opt: Option<i32> = string
                .split_whitespace()
                .next()
                .and_then(|num_str| num_str.parse::<i32>().ok());

            if let Some(idecko) = cislo_opt {
                if let Some(pesnicka) = self.song_lister.song_manager.get_song_by_id(idecko) {
                    let copy = pesnicka.clone();
                    fo_open_to_edit_song(&copy, &mut self.song_lister.song_manager);
                }
            }
        }
    }

    pub fn add_song(&mut self) {
        if self.focusing_widget == FocusedWidget::Left && !self.song_lister.song_manager.is_empty()
        {
            fo_add_song(&mut self.song_lister.song_manager);
        }
    }

    pub fn delete_song(&mut self) {
        if self.focusing_widget == FocusedWidget::Left && !self.song_lister.song_manager.is_empty()
        {
            let index = match self.song_lister.state.selected() {
                Some(i) => i,
                None => 0,
            };

            let string = match self.song_lister.search_get_formated().get(index) {
                Some(s) => s.to_lowercase(),
                None => String::new(),
            };

            let cislo_opt: Option<i32> = string
                .split_whitespace()
                .next()
                .and_then(|num_str| num_str.parse::<i32>().ok());

            if let Some(idecko) = cislo_opt {
                let pesnicka_opt = self.song_lister.song_manager.get_song_by_id(idecko);
                if let Some(pesnicka) = pesnicka_opt {
                    self.song_lister.song_manager = fo_delete_song(pesnicka);
                }
            }
        }
    }

    fn presentation_start(&self) {

        
        
        let exe_path = match env::current_exe() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Chyba pri current_exe: {}", e);
                return;
            }
        };
        
        let base_dir = match exe_path.parent() {
            Some(dir) => dir,
            None => {
                eprintln!("Nepodarilo sa získať parent directory");
                return;
            }
        };

        let song_manager_path = base_dir.join("temp_song_manager.json");

        self.selected_song_lister.song_manager.save_to_file_json(song_manager_path.to_str().expect("Neplatná cesta zlé znaky UTF-8"));

        let presenter_path = base_dir.join("tvoric_platna");

        let status = Command::new(&presenter_path)
            .arg("--fullscreen")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status();

        match status {
            Ok(exit_status) => {
                if exit_status.success() {
                    println!("Presenter skončil OK");
                } else {
                    println!("Presenter skončil s chybou: {:?}", exit_status);
                }
            }

            Err(e) => {
                eprintln!("Nepodarilo sa spustiť presenter: {}", e);
            }
        }
    }
}
