use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, List, ListItem, Paragraph, StatefulWidget, Widget},
};
use tui_tree_widget::{Tree, TreeItem, TreeState};

use crate::{
    app::{App, FocusedWidget},
    dominikani_logo::ASCII_LOGO,
    song_lister::TreeId,
};

// Tmavý theme.
const COLOR_BG: Color = Color::Rgb(18, 18, 24); // hlavné pozadie
const COLOR_PANEL_BG: Color = Color::Rgb(24, 24, 32); // panely
const COLOR_PANEL_BORDER: Color = Color::Rgb(80, 80, 110);
const COLOR_PANEL_BORDER_FOCUS: Color = Color::Rgb(120, 170, 255);
const COLOR_SEARCH_BG: Color = Color::Rgb(32, 32, 48);
const COLOR_SELECTED_BG: Color = Color::Rgb(50, 90, 160);
const COLOR_SELECTED_FG: Color = Color::White;
const COLOR_HELP_BG: Color = Color::Rgb(30, 30, 40);
const COLOR_HELP_FG: Color = Color::Rgb(220, 220, 230);

const SELECTED: Style = Style::new()
    .bg(COLOR_SELECTED_BG)
    .fg(COLOR_SELECTED_FG)
    .add_modifier(Modifier::BOLD);

fn render_ascii_background(area: Rect, buf: &mut Buffer) {
    let lines: Vec<&str> = ASCII_LOGO.lines().collect();
    if lines.is_empty() {
        return;
    }

    let logo_height = lines.len() as u16;
    let logo_width = lines
        .iter()
        .map(|l| l.chars().count() as u16)
        .max()
        .unwrap_or(0);

    // stred obrazovky
    let center_x = area.left() + area.width / 2;
    let center_y = area.top() + area.height / 2;

    // ľavý horný roh loga
    let start_x = center_x.saturating_sub(logo_width / 2);
    let start_y = center_y.saturating_sub(logo_height / 2);

    for (row, line) in lines.iter().enumerate() {
        let y = start_y + row as u16;
        if y >= area.bottom() {
            break;
        }

        for (col, ch) in line.chars().enumerate() {
            let x = start_x + col as u16;
            if x >= area.right() {
                break;
            }

            let cell = &mut buf[(x, y)];
            // Jemný, nenápadný text v pozadí
            cell.set_fg(Color::Rgb(60, 60, 80));
            cell.set_bg(COLOR_BG);
            cell.set_symbol(&ch.to_string());
        }
    }
}

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
        // vyplň pozadie panela
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                buf[(x, y)].set_bg(COLOR_PANEL_BG);
            }
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        let search_area = chunks[0];
        let list_area = chunks[1];

        let search_text = self.song_lister.search.clone();
        let items: Vec<TreeItem<'static, TreeId>> = self.song_lister.build_tree();

        let mut search_block = Block::bordered()
            .title("Hľadaj")
            .title_alignment(Alignment::Left)
            .border_type(BorderType::Rounded)
            .style(Style::new().bg(COLOR_PANEL_BG))
            .border_style(Style::new().fg(COLOR_PANEL_BORDER));

        if self.focusing_widget == FocusedWidget::Search {
            search_block = search_block.border_style(Style::new().fg(COLOR_PANEL_BORDER_FOCUS));
        }

        Paragraph::new(search_text)
            .block(search_block)
            .style(Style::new().fg(Color::White).bg(COLOR_PANEL_BG))
            .alignment(Alignment::Left)
            .render(search_area, buf);

        let mut border = Block::bordered()
            .title("Pesničky v databáze")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded)
            .style(Style::new().bg(COLOR_PANEL_BG))
            .border_style(Style::new().fg(COLOR_PANEL_BORDER));

        if self.focusing_widget == FocusedWidget::Left {
            border = border.border_style(Style::new().fg(COLOR_PANEL_BORDER_FOCUS));
        }

        let highlight_style = if self.focusing_widget == FocusedWidget::Left {
            SELECTED
        } else {
            Style::new().bg(COLOR_PANEL_BG).fg(Color::White)
        };

        render_left_tree(
            &mut self.song_lister.state,
            &items,
            list_area,
            buf,
            border,
            highlight_style,
        );
    }

    /// Renderuje pravý panel so zoznamom vybraných piesní na premietanie.
    pub fn render_right(&mut self, area: Rect, buf: &mut Buffer) {
        // vyplň pozadie panela
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                buf[(x, y)].set_bg(COLOR_PANEL_BG);
            }
        }

        let items: Vec<ListItem> = self
            .selected_song_lister
            .song_manager
            .get_format_all()
            .into_iter()
            .map(|s| ListItem::new(s).style(Style::new().fg(Color::White)))
            .collect();

        let mut border = Block::bordered()
            .title("Premietanie")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded)
            .style(Style::new().bg(COLOR_PANEL_BG))
            .border_style(Style::new().fg(COLOR_PANEL_BORDER));

        if self.focusing_widget == FocusedWidget::Right {
            border = border.border_style(Style::new().fg(COLOR_PANEL_BORDER_FOCUS));
        }

        if self.selected_song_lister.state.selected().is_none()
            && !self.selected_song_lister.song_manager.is_empty()
        {
            self.selected_song_lister.state.select(Some(0));
        }

        let highlight_style = if self.focusing_widget == FocusedWidget::Right {
            SELECTED
        } else {
            Style::new().bg(COLOR_PANEL_BG).fg(Color::White)
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

        Paragraph::new(text)
            .style(
                Style::default()
                    .fg(COLOR_HELP_FG)
                    .bg(COLOR_HELP_BG)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Left)
            .render(area, buf);
    }
}

impl Widget for &mut App {
    /// Hlavný vstup na vykreslenie celej aplikácie do terminálu.
    fn render(self, area: Rect, buf: &mut Buffer) {
        // globálne pozadie
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                buf[(x, y)].set_bg(COLOR_BG);
            }
        }

        render_ascii_background(area, buf);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(area);

        let content_area = chunks[0];
        let help_area = chunks[1];

        let main_split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(content_area);

        let left_area = main_split[0];
        let right_area = main_split[1];

        self.render_left(left_area, buf);
        self.render_right(right_area, buf);
        self.render_help_bar(help_area, buf);
    }
}
