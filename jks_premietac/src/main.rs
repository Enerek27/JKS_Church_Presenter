use crate::{app::App, popups::ask_song_type};

pub mod app;
pub mod event;
pub mod file_opener;
pub mod popups;
pub mod song_lister;
pub mod ui;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal).await;
    ratatui::restore();
    result
}

/*
fn main() {
    ask_song_type();
}
    */
