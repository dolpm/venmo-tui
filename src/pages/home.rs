use std::io::{self, StdoutLock};

use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Spans,
    widgets::{Block, Borders, List, ListItem, ListState},
    Terminal,
};
use tui_textarea::{Input, Key};

use crate::{api::Api, types::LoginResponse};

use super::{me::MePage, stories::StoriesPage, Page};

enum Selected {
    SideBar,
    MainWindow,
}

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
    Me,
    Transactions,
    PayAndRequest,
    Logout,
}

impl Default for CurrentPage {
    fn default() -> Self {
        CurrentPage::Me
    }
}

impl ToString for CurrentPage {
    fn to_string(&self) -> String {
        match self {
            CurrentPage::Me => "Me",
            CurrentPage::Transactions => "Transactions",
            CurrentPage::PayAndRequest => "PayAndRequest",
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
                ("Me", CurrentPage::Me),
                ("Transactions", CurrentPage::Transactions),
                ("Pay/Request", CurrentPage::PayAndRequest),
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
                    CurrentPage::Me => Some(Box::new(MePage::new(api))),
                    CurrentPage::Transactions => Some(Box::new(StoriesPage::new(api))),
                    CurrentPage::PayAndRequest => None,
                    CurrentPage::Logout => None,
                };
                assoc_index = selected;
            }
        }

        term.draw(|f| {
            let chunks = layout.split(f.size());

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
                let event = crossterm::event::read()?.into();

                // if escape, exit
                // if left, go back to side bar
                match event {
                    Input { key: Key::Esc, .. } => break 'outer,
                    Input { key: Key::Left, .. } => {
                        focused_area = FocusedArea::SideBar;
                    }
                    _ => {}
                };

                if let Some(_selected) = side_bar.items.state.selected() {
                    if let Some(ref mut p) = &mut current_page {
                        if p.on_input_event(event).await {
                            break 'outer;
                        };
                    }
                }
            }
        }
    }

    Ok(None)
}
