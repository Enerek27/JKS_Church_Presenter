use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, List, ListItem, Paragraph, StatefulWidget, Widget, Wrap},
};
use tui_tree_widget::{Tree, TreeItem, TreeState};

use crate::{
    app::{App, FocusedWidget},
    dominikani_logo::ASCII_LOGO,
    song_lister::TreeId,
};

// ── Farby – normálny režim ────────────────────────────────────
const COLOR_BG: Color = Color::Rgb(18, 18, 24);
const COLOR_PANEL_BG: Color = Color::Rgb(24, 24, 32);
const COLOR_PANEL_BORDER: Color = Color::Rgb(80, 80, 110);
const COLOR_PANEL_BORDER_FOCUS: Color = Color::Rgb(120, 170, 255);
const COLOR_SELECTED_BG: Color = Color::Rgb(50, 90, 160);
const COLOR_SELECTED_FG: Color = Color::White;
const COLOR_HELP_BG: Color = Color::Rgb(30, 30, 40);
const COLOR_HELP_FG: Color = Color::Rgb(220, 220, 230);

// ── Farby – premietací režim ──────────────────────────────────
const COLOR_PRES_BG: Color = Color::Rgb(8, 8, 14);

// Aktuálna sloha
const COLOR_CURR_FG: Color = Color::Rgb(255, 255, 255);
const COLOR_CURR_BG: Color = Color::Rgb(28, 45, 90);
const COLOR_CURR_BORDER: Color = Color::Rgb(100, 150, 255);

// Vedľajšie slohy
const COLOR_SIDE_FG: Color = Color::Rgb(200, 200, 220);
const COLOR_SIDE_BG: Color = Color::Rgb(16, 16, 24);
const COLOR_SIDE_LABEL: Color = Color::Rgb(100, 130, 200);

// Pravý panel (zoznam)
const COLOR_LIST_BG: Color = Color::Rgb(12, 12, 20);
const COLOR_LIST_BORDER: Color = Color::Rgb(60, 70, 110);

// Blackscreen overlay
const COLOR_OVERLAY_BG: Color = Color::Rgb(12, 12, 20);
const COLOR_OVERLAY_FG: Color = Color::Rgb(40, 40, 55);
const COLOR_BANNER_FG: Color = Color::Rgb(220, 80, 80);
const COLOR_BANNER_BG: Color = Color::Rgb(40, 15, 15);

const SELECTED: Style = Style::new()
    .bg(COLOR_SELECTED_BG)
    .fg(COLOR_SELECTED_FG)
    .add_modifier(Modifier::BOLD);

// ── Helpers ───────────────────────────────────────────────────
fn fill_bg(area: Rect, buf: &mut Buffer, color: Color) {
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            let cell = &mut buf[(x, y)];
            cell.set_bg(color);
            cell.set_fg(Color::Reset); // ← toto pridaj
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

/// Kreslí ASCII logo NA POZADIE – ale len do buniek ktoré majú bg == COLOR_BG,
/// teda do buniek kde ešte nebol nakreslený žiadny panel.
/// Vďaka tomu logo nikdy neovplyvní fg buniek vo vnútri panelov.
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
    let start_x = (area.left() + area.width / 2).saturating_sub(logo_width / 2);
    let start_y = (area.top() + area.height / 2).saturating_sub(logo_height / 2);
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
            cell.set_fg(Color::Rgb(60, 60, 80));
            cell.set_bg(COLOR_BG);
            cell.set_symbol(&ch.to_string());
        }
    }
}

/// Prekryje plochu tmavým overlay a zobrazí banner ZATMAVENÉ v strede.
fn render_blackscreen_overlay(area: Rect, buf: &mut Buffer) {
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            let cell = &mut buf[(x, y)];
            cell.set_fg(COLOR_OVERLAY_FG);
            cell.set_bg(COLOR_OVERLAY_BG);
        }
    }
    let banner = "  ■  ZATMAVENÉ  ■  ";
    let banner_len = banner.chars().count() as u16;
    let banner_x = area.left() + area.width.saturating_sub(banner_len) / 2;
    let banner_y = area.top() + area.height / 2;
    for (i, ch) in banner.chars().enumerate() {
        let x = banner_x + i as u16;
        if x >= area.right() {
            break;
        }
        let cell = &mut buf[(x, banner_y)];
        cell.set_symbol(&ch.to_string());
        cell.set_fg(COLOR_BANNER_FG);
        cell.set_bg(COLOR_BANNER_BG);
        cell.modifier = Modifier::BOLD;
    }
}

impl App {
    pub fn render_loading_overlay(&self, area: Rect, buf: &mut Buffer) {
        // Tmavý overlay cez celú obrazovku
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                let cell = &mut buf[(x, y)];
                cell.set_bg(Color::Rgb(8, 8, 14));
                cell.set_fg(Color::Reset);
                cell.set_symbol(" ");
            }
        }

        let percent = (self.loading_progress * 100.0) as u8;
        let label = format!(" Odosielam piesne... {}% ", percent);

        // Bar je v strede obrazovky, výška 5 riadkov
        let bar_w = area.width.saturating_sub(8); // okraj 4 po každej strane
        let bar_x = area.left() + 4;
        let bar_y = area.top() + area.height / 2 - 1;

        // Nadpis
        let title = " Načítavam... ";
        let title_x = area.left() + area.width.saturating_sub(title.chars().count() as u16) / 2;
        for (i, ch) in title.chars().enumerate() {
            let x = title_x + i as u16;
            if x >= area.right() {
                break;
            }
            let cell = &mut buf[(x, bar_y - 2)];
            cell.set_symbol(&ch.to_string());
            cell.set_fg(Color::Rgb(180, 200, 255));
            cell.set_bg(Color::Rgb(8, 8, 14));
            cell.modifier = Modifier::BOLD;
        }

        // Pozadie baru (prázdne)
        for x in bar_x..bar_x + bar_w {
            let cell = &mut buf[(x, bar_y)];
            cell.set_symbol("░");
            cell.set_fg(Color::Rgb(50, 50, 70));
            cell.set_bg(Color::Rgb(8, 8, 14));
        }

        // Vyplnená časť
        let filled = ((self.loading_progress) * bar_w as f32) as u16;
        for x in bar_x..bar_x + filled.min(bar_w) {
            let cell = &mut buf[(x, bar_y)];
            cell.set_symbol("█");
            cell.set_fg(Color::Rgb(100, 160, 255));
            cell.set_bg(Color::Rgb(8, 8, 14));
        }

        // Percentá pod barom
        let label_x = area.left() + area.width.saturating_sub(label.chars().count() as u16) / 2;
        for (i, ch) in label.chars().enumerate() {
            let x = label_x + i as u16;
            if x >= area.right() {
                break;
            }
            let cell = &mut buf[(x, bar_y + 2)];
            cell.set_symbol(&ch.to_string());
            cell.set_fg(Color::Rgb(120, 170, 255));
            cell.set_bg(Color::Rgb(8, 8, 14));
            cell.modifier = Modifier::BOLD;
        }
    }
    // ── Normálny: ľavý panel ──────────────────────────────────
    pub fn render_left(&mut self, area: Rect, buf: &mut Buffer) {
        fill_bg(area, buf, COLOR_PANEL_BG);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

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
            .render(chunks[0], buf);

        let mut border = Block::bordered()
            .title("Pesničky v databáze")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded)
            .style(Style::new().bg(COLOR_PANEL_BG))
            .border_style(Style::new().fg(COLOR_PANEL_BORDER));
        if self.focusing_widget == FocusedWidget::Left {
            border = border.border_style(Style::new().fg(COLOR_PANEL_BORDER_FOCUS));
        }
        let hl = if self.focusing_widget == FocusedWidget::Left {
            SELECTED
        } else {
            Style::new().bg(COLOR_PANEL_BG).fg(Color::White)
        };
        render_left_tree(
            &mut self.song_lister.state,
            &items,
            chunks[1],
            buf,
            border,
            hl,
        );
    }

    // ── Normálny: pravý panel ─────────────────────────────────
    pub fn render_right(&mut self, area: Rect, buf: &mut Buffer) {
        fill_bg(area, buf, COLOR_PANEL_BG);
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
        let hl = if self.focusing_widget == FocusedWidget::Right {
            SELECTED
        } else {
            Style::new().bg(COLOR_PANEL_BG).fg(Color::White)
        };
        let list = List::new(items)
            .block(border)
            .highlight_style(hl)
            .highlight_symbol(">>")
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Always);
        StatefulWidget::render(list, area, buf, &mut self.selected_song_lister.state);
    }

    // ── Help bar ──────────────────────────────────────────────
    pub fn render_help_bar(&mut self, area: Rect, buf: &mut Buffer) {
        let text = if self.premieta() {
            "↑↓: sloha  |  ←→: pieseň  |  Medzerník: zatmavenie  |  End: ukončiť premietanie"
        } else {
            match self.focusing_widget {
                FocusedWidget::Left => {
                    "Tab: panel  |  Šípky: pohyb  |  Medzerník: pridať  |  Enter: upraviť  |  p: pridať  |  Delete: zmazať  |  q: koniec"
                }
                FocusedWidget::Right => {
                    "Tab: panel  |  Šípky: pohyb  |  Medzerník: odobrať  |  Home: štart  |  q: koniec"
                }
                FocusedWidget::Search => {
                    "Píš pre hľadanie  |  Backspace: zmaž  |  Tab: panel  |  q: koniec"
                }
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

    // ── Premietací: ľavý panel ────────────────────────────────
    pub fn render_presentation_left(&mut self, area: Rect, buf: &mut Buffer) {
        let blackscreen = self
            .komunikator
            .as_ref()
            .map(|k| k.stav.blackscreen)
            .unwrap_or(false);
        let sloha_idx = self
            .komunikator
            .as_ref()
            .map(|k| k.stav.cislo_slohy as usize)
            .unwrap_or(0);
        let pocet_sloh = self.get_pocet_sloh();
        let nazov = self.get_nazov_piesne();

        fill_bg(area, buf, COLOR_PRES_BG);

        let title = if nazov.is_empty() {
            format!(" sloha {}/{} ", sloha_idx, pocet_sloh)
        } else {
            format!(" {} ── {}/{} ", nazov, sloha_idx, pocet_sloh)
        };
        let outer_block = Block::bordered()
            .title(title)
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Double)
            .style(Style::new().bg(COLOR_PRES_BG))
            .border_style(Style::new().fg(COLOR_PANEL_BORDER_FOCUS));

        let inner = outer_block.inner(area);
        outer_block.render(area, buf);

        let (prev, curr, next) = self.get_slohy_pre_zobrazenie();
        let has_prev = !prev.is_empty();
        let has_next = !next.is_empty();

        let constraints = match (has_prev, has_next) {
            (true, true) => vec![
                Constraint::Percentage(22),
                Constraint::Min(6),
                Constraint::Percentage(22),
            ],
            (true, false) => vec![
                Constraint::Percentage(22),
                Constraint::Min(6),
                Constraint::Length(0),
            ],
            (false, true) => vec![
                Constraint::Length(0),
                Constraint::Min(6),
                Constraint::Percentage(22),
            ],
            (false, false) => vec![
                Constraint::Length(0),
                Constraint::Min(6),
                Constraint::Length(0),
            ],
        };

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(inner);

        if has_prev {
            fill_bg(rows[0], buf, COLOR_SIDE_BG);
            let label_area = Rect {
                height: 1,
                ..rows[0]
            };
            let text_area = Rect {
                y: rows[0].y + 1,
                height: rows[0].height.saturating_sub(1),
                ..rows[0]
            };
            Paragraph::new(Line::from(vec![
                Span::styled(
                    "▲ ",
                    Style::new()
                        .fg(COLOR_SIDE_LABEL)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("predchádzajúca", Style::new().fg(COLOR_SIDE_LABEL)),
            ]))
            .render(label_area, buf);
            Paragraph::new(prev)
                .style(Style::new().fg(COLOR_SIDE_FG).bg(COLOR_SIDE_BG))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true })
                .render(text_area, buf);
        }

        fill_bg(rows[1], buf, COLOR_CURR_BG);
        let curr_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .style(Style::new().bg(COLOR_CURR_BG))
            .border_style(
                Style::new()
                    .fg(COLOR_CURR_BORDER)
                    .add_modifier(Modifier::BOLD),
            );
        let curr_inner = curr_block.inner(rows[1]);
        curr_block.render(rows[1], buf);
        let text_height = curr.lines().count() as u16;
        let v_pad = curr_inner.height.saturating_sub(text_height) / 2;
        let centered = Rect {
            y: curr_inner.y + v_pad,
            height: curr_inner.height.saturating_sub(v_pad),
            ..curr_inner
        };
        Paragraph::new(curr)
            .style(
                Style::new()
                    .fg(COLOR_CURR_FG)
                    .bg(COLOR_CURR_BG)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .render(centered, buf);

        if has_next {
            fill_bg(rows[2], buf, COLOR_SIDE_BG);
            let label_area = Rect {
                height: 1,
                ..rows[2]
            };
            let text_area = Rect {
                y: rows[2].y + 1,
                height: rows[2].height.saturating_sub(1),
                ..rows[2]
            };
            Paragraph::new(Line::from(vec![
                Span::styled(
                    "▼ ",
                    Style::new()
                        .fg(COLOR_SIDE_LABEL)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("nasledujúca", Style::new().fg(COLOR_SIDE_LABEL)),
            ]))
            .render(label_area, buf);
            Paragraph::new(next)
                .style(Style::new().fg(COLOR_SIDE_FG).bg(COLOR_SIDE_BG))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true })
                .render(text_area, buf);
        }

        if blackscreen {
            render_blackscreen_overlay(inner, buf);
        }
    }

    // ── Premietací: pravý panel (zoznam) ──────────────────────
    pub fn render_presentation_right(&mut self, area: Rect, buf: &mut Buffer) {
        fill_bg(area, buf, COLOR_LIST_BG);

        let sloha_idx = self
            .komunikator
            .as_ref()
            .map(|k| k.stav.cislo_slohy as usize)
            .unwrap_or(0);
        let pocet_sloh = self.get_pocet_sloh();
        let blackscreen = self
            .komunikator
            .as_ref()
            .map(|k| k.stav.blackscreen)
            .unwrap_or(false);

        let info_text = if blackscreen {
            " ■ ZATMAVENÉ".to_string()
        } else {
            format!(" sloha  {}/{}", sloha_idx, pocet_sloh)
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        Paragraph::new(info_text)
            .style(
                Style::new()
                    .fg(if blackscreen {
                        Color::Rgb(220, 80, 80)
                    } else {
                        COLOR_CURR_BORDER
                    })
                    .bg(COLOR_LIST_BG)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Style::new().fg(if blackscreen {
                        Color::Rgb(120, 40, 40)
                    } else {
                        COLOR_LIST_BORDER
                    }))
                    .style(Style::new().bg(COLOR_LIST_BG)),
            )
            .render(chunks[0], buf);

        let items: Vec<ListItem> = self
            .selected_song_lister
            .song_manager
            .get_format_all()
            .into_iter()
            .map(|s| ListItem::new(s).style(Style::new().fg(COLOR_SIDE_FG)))
            .collect();

        let list = List::new(items)
            .block(
                Block::bordered()
                    .title(" Piesne ")
                    .title_alignment(Alignment::Center)
                    .border_type(BorderType::Rounded)
                    .style(Style::new().bg(COLOR_LIST_BG))
                    .border_style(Style::new().fg(COLOR_LIST_BORDER)),
            )
            .highlight_style(
                Style::new()
                    .bg(COLOR_SELECTED_BG)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ")
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Always);
        StatefulWidget::render(list, chunks[1], buf, &mut self.selected_song_lister.state);
    }
}

// ── Hlavný Widget render ──────────────────────────────────────
impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        fill_bg(area, buf, COLOR_BG);
        let premieta = self.premieta();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(area);

        if premieta {
            let split = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(72), Constraint::Percentage(28)])
                .split(chunks[0]);
            self.render_presentation_left(split[0], buf);
            self.render_presentation_right(split[1], buf);
        } else {
            // Logo sa kreslí AŽ PO fill_bg ale PRED panelmi –
            // panely ho potom prekreslia cez vlastný fill_bg,
            // takže logo neovplyvní fg textu v zozname.
            render_ascii_background(chunks[0], buf);

            let split = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(chunks[0]);
            self.render_left(split[0], buf);
            self.render_right(split[1], buf);
        }
        self.render_help_bar(chunks[1], buf);
        if self.loading {
            self.render_loading_overlay(area, buf);
        }
    }
}
