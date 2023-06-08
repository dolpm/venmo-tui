use std::io::{self, StdoutLock};

use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};
use tui_textarea::{Input, Key};

use crate::{api::Api, types::LoginResponse};

use super::{me::MePage, stories::StoriesPage, Page, ASCII_TITLE};

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: {
                let mut s = ListState::default();
                s.select(Some(0));
                s
            },
            items,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };

        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

enum CurrentPage {
    Home,
    Transactions,
    Logout,
}

impl Default for CurrentPage {
    fn default() -> Self {
        CurrentPage::Home
    }
}

impl ToString for CurrentPage {
    fn to_string(&self) -> String {
        match self {
            CurrentPage::Home => "Home",
            CurrentPage::Transactions => "Transactions",
            CurrentPage::Logout => "Logout",
        }
        .to_string()
    }
}

struct SideBar<'a> {
    items: StatefulList<(&'a str, CurrentPage)>,
}

impl<'a> SideBar<'a> {
    fn new() -> SideBar<'a> {
        SideBar {
            items: StatefulList::with_items(vec![
                ("Home", CurrentPage::Home),
                ("Transactions", CurrentPage::Transactions),
                ("Logout", CurrentPage::Logout),
            ]),
        }
    }
}

enum FocusedArea {
    SideBar,
    MainWindow,
}

pub async fn draw_home_page(
    term: &mut Terminal<CrosstermBackend<StdoutLock<'_>>>,
    api: &'static mut Api,
) -> io::Result<Option<LoginResponse>> {
    let mut focused_area = FocusedArea::SideBar;

    let mut side_bar = SideBar::new();
    let (mut assoc_index, mut current_page): (usize, Option<Box<dyn Page>>) =
        (0, Some(Box::new(MePage::new(api))));

    let venmo_text_big = Paragraph::new(
        ASCII_TITLE
            .lines()
            .skip(1)
            .map(|l| Spans::from(Span::styled(l, Style::default())))
            .collect::<Vec<_>>(),
    )
    .style(Style::default().fg(Color::Blue))
    .block(Block::default())
    .alignment(Alignment::Left);

    let outer_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(8)].as_ref());

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(10), Constraint::Percentage(90)].as_ref());

    let items: Vec<ListItem> = side_bar
        .items
        .items
        .iter()
        .map(|i| ListItem::new(Spans::from(i.0)).style(Style::default().fg(Color::Black)))
        .collect();

    'outer: loop {
        if let Some(selected) = side_bar.items.state.selected() {
            if selected != assoc_index {
                drop(current_page);
                current_page = match side_bar.items.items[selected].1 {
                    CurrentPage::Home => Some(Box::new(MePage::new(api))),
                    CurrentPage::Transactions => Some(Box::new(StoriesPage::new(api))),
                    CurrentPage::Logout => None,
                };
                assoc_index = selected;
            }
        }

        term.draw(|f| {
            let outer_chunks = outer_layout.split(f.size());

            f.render_widget(venmo_text_big.clone(), outer_chunks[0]);

            let chunks = layout.split(outer_chunks[1]);

            {
                // Create a List from all list items and highlight the currently selected one
                let items = List::new(items.clone())
                    .block(Block::default().borders(Borders::ALL))
                    .highlight_style(
                        Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD),
                    );
                f.render_stateful_widget(items, chunks[0], &mut side_bar.items.state);
            }

            if let Some(_) = side_bar.items.state.selected() {
                if let Some(ref mut p) = &mut current_page {
                    p.render(f, chunks[1]);
                }
            }
        })?;

        if let Some(ref mut p) = &mut current_page {
            if p.make_progress().await {
                continue;
            }
        }

        match focused_area {
            FocusedArea::SideBar => match crossterm::event::read()?.into() {
                Input { key: Key::Esc, .. } => break 'outer,
                Input { key: Key::Down, .. } => side_bar.items.next(),
                Input { key: Key::Up, .. } => side_bar.items.previous(),
                Input {
                    key: Key::Right, ..
                } => {
                    focused_area = FocusedArea::MainWindow;
                }
                Input {
                    key: Key::Enter, ..
                } => {
                    if let Some(selected) = side_bar.items.state.selected() {
                        match side_bar.items.items[selected].1 {
                            CurrentPage::Logout => {
                                drop(current_page);
                                api.logout().await.expect("failed to logout ugh :(");
                                break 'outer;
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            },
            FocusedArea::MainWindow => {
                let event: Input = crossterm::event::read()?.into();

                if let Some(_selected) = side_bar.items.state.selected() {
                    if let Some(ref mut p) = &mut current_page {
                        if !p.on_input_event(event.clone()).await {
                            continue;
                        };
                    }
                }

                // if escape, exit
                // if left, go back to side bar
                match event {
                    Input { key: Key::Esc, .. } => break 'outer,
                    Input { key: Key::Left, .. } => {
                        focused_area = FocusedArea::SideBar;
                    }
                    _ => {}
                };
            }
        }
    }

    Ok(None)
}
