use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, List, ListItem, Paragraph, StatefulWidget, Widget},
};
use tui_tree_widget::{Tree, TreeItem, TreeState};

use crate::{
    app::{App, FocusedWidget},
    song_lister::TreeId,
};

const SELECTED: Style = Style::new()
    .bg(Color::LightMagenta)
    .add_modifier(Modifier::BOLD);

fn render_left_tree(
    state: &mut TreeState<TreeId>,
    items: &[TreeItem<'static, TreeId>],
    area: Rect,
    buf: &mut Buffer,
    border: Block<'_>,
    highlight_style: Style,
) {
    let tree = Tree::new(items)
        .expect("Nie sú všetky identifikátory unikátne")
        .block(border)
        .highlight_style(highlight_style)
        .highlight_symbol(">>");

    StatefulWidget::render(tree, area, buf, state);
}

impl App {
    /// Renderuje ľavý panel so stromom piesní a vyhľadávacím riadkom.
    pub fn render_left(&mut self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(4), Constraint::Min(0)])
            .split(area);

        let search_text_value = self.song_lister.search.clone();
        let items: Vec<TreeItem<'static, TreeId>> = self.song_lister.build_tree();

        let highlight_search = if self.focusing_widget == FocusedWidget::Search {
            Style::new().bg(Color::LightCyan)
        } else {
            Style::new()
        };

        let search_bar = Block::bordered()
            .title("Hľadaj")
            .border_style(highlight_search)
            .border_type(BorderType::Rounded);

        let search_paragraph = Paragraph::new(search_text_value).block(search_bar);
        search_paragraph.render(chunks[0], buf);

        let mut border = Block::bordered()
            .title("Pesničky v databáze")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        if self.focusing_widget == FocusedWidget::Left {
            border = border.border_style(Style::new().bg(Color::LightCyan));
        }

        let highlight_style = if self.focusing_widget == FocusedWidget::Left {
            SELECTED
        } else {
            Style::new()
        };

        let state: &mut TreeState<TreeId> = &mut self.song_lister.state;
        render_left_tree(state, &items, chunks[1], buf, border, highlight_style);
    }

    /// Renderuje pravý panel so zoznamom vybraných piesní na premietanie.
    pub fn render_right(&mut self, area: Rect, buf: &mut Buffer) {
        let items: Vec<ListItem> = self
            .selected_song_lister
            .song_manager
            .get_format_all()
            .into_iter()
            .map(ListItem::new)
            .collect();

        let mut border = Block::bordered()
            .title("Premietanie")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        if self.focusing_widget == FocusedWidget::Right {
            border = border.border_style(Style::new().bg(Color::LightCyan));
        }

        if self.selected_song_lister.state.selected().is_none()
            && !self.selected_song_lister.song_manager.is_empty()
        {
            self.selected_song_lister.state.select(Some(0));
        }

        let highlight_style = if self.focusing_widget == FocusedWidget::Right {
            SELECTED
        } else {
            Style::new()
        };

        let song_list = List::new(items)
            .block(border)
            .highlight_style(highlight_style)
            .highlight_symbol(">>")
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Always);

        StatefulWidget::render(song_list, area, buf, &mut self.selected_song_lister.state);
    }

    /// Renderuje spodný pomocný panel s nápovedou podľa aktívneho panelu.
    pub fn render_help_bar(&mut self, area: Rect, buf: &mut Buffer) {
        let text = match self.focusing_widget {
            FocusedWidget::Left => {
                "Tab: prepni panel  |  Šípky: pohyb  |  Medzerník: pridať do premietania  |  Enter: upraviť  |  p: pridať pesničku  |  Delete: zmazať  |  q/Esc: ukončiť"
            }
            FocusedWidget::Right => {
                "Tab: prepni panel  |  Šípky: pohyb  |  Medzerník: odstrániť z premietania  |  Home: štart prezentácie  |  q/Esc: ukončiť"
            }
            FocusedWidget::Search => {
                "Píš pre hľadanie  |  Backspace: zmaž znak  |  Tab: prepni panel  |  q/Esc: ukončiť"
            }
        };

        let para = Paragraph::new(text)
            .style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Gray)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Left);

        para.render(area, buf);
    }
}

impl Widget for &mut App {
    /// Hlavný vstup na vykreslenie celej aplikácie do terminálu.
    fn render(self, area: Rect, buf: &mut Buffer) {
        let outer = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(area);

        let content_area = outer[0];
        let help_area = outer[1];

        let main_split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(content_area);

        self.render_left(main_split[0], buf);
        self.render_right(main_split[1], buf);
        self.render_help_bar(help_area, buf);
    }
}