use std::io::StdoutLock;

use async_trait::async_trait;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};
use tui_textarea::{Input, Key};

use crate::{
    api::Api,
    types::{Story, StorySubType},
};

use super::Page;

pub struct StoriesPage<'a> {
    loading: bool,
    last: Option<String>,
    api: &'a mut Api,
    state: TableState,
    items: Vec<Vec<String>>,
}

const LOAD_SIZE: u32 = 30;

impl<'a> StoriesPage<'a> {
    pub fn create_table_rows(data: Vec<Story>) -> Vec<Vec<String>> {
        data.into_iter()
            .filter_map(|story| match story.title.payload.sub_type {
                StorySubType::P2p => Some(vec![
                    story.amount,
                    story.title.receiver.unwrap().username,
                    story.title.sender.unwrap().username,
                    story.date,
                    story.note.content.unwrap_or_default(),
                ]),
                _ => None,
            })
            .collect::<Vec<_>>()
    }

    pub fn new(api: &mut Api) -> StoriesPage {
        StoriesPage {
            last: None,
            api,
            loading: true,
            state: TableState::default(),
            items: vec![vec!["Loading...".to_string()]],
        }
    }

    pub async fn load_more_items(&mut self) {
        // load more items
        let stories_data = self
            .api
            .get_recents(LOAD_SIZE, self.last.as_deref())
            .await
            .expect("failed to get story data");

        self.items.pop();

        self.items
            .append(&mut Self::create_table_rows(stories_data.stories));

        self.items.push(vec!["Load more :)".to_string()]);

        self.last = Some(stories_data.next_id);
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    i
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}

#[async_trait]
impl<'a> Page for StoriesPage<'a> {
    async fn on_input_event(&mut self, event: tui_textarea::Input) -> bool {
        match event {
            Input { key: Key::Down, .. } => self.next(),
            Input { key: Key::Up, .. } => self.previous(),
            Input {
                key: Key::Enter, ..
            } => {
                if let Some(i) = self.state.selected() {
                    let len = self.items.len();
                    if i >= len - 1 {
                        self.loading = true;
                        self.items[len - 1][0] = "Loading...".to_string();
                    }
                }
            }
            Input { key: Key::Left, .. } | Input { key: Key::Esc, .. } => {
                self.unselect();
                return true;
            }
            _ => {}
        }
        false
    }

    async fn make_progress(&mut self) -> bool {
        if self.loading {
            self.load_more_items().await;
            self.loading = false;
            return true;
        }
        false
    }

    fn render(&mut self, f: &mut Frame<CrosstermBackend<StdoutLock>>, area: Rect) {
        let selected_style = Style::default().add_modifier(Modifier::REVERSED);
        let normal_style = Style::default().bg(Color::Blue);
        let header_cells = ["Amount", "To", "From", "Date", "Note"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
        let header = Row::new(header_cells)
            .style(normal_style)
            .height(1)
            .bottom_margin(1);

        let rows = self
            .items
            .iter()
            .map(|item| {
                let height = item
                    .iter()
                    .map(|content| content.chars().filter(|c| *c == '\n').count())
                    .max()
                    .unwrap_or(0)
                    + 1;
                let cells = item.iter().enumerate().map(|(i, c)| {
                    let mut cell = Cell::from(c.as_str());
                    // green if +, red if -
                    if i == 0 {
                        if c.starts_with("+") {
                            cell = cell.style(Style::default().fg(Color::Green));
                        } else if c.starts_with("-") {
                            cell = cell.style(Style::default().fg(Color::Red));
                        }
                    }
                    cell
                });
                Row::new(cells).height(height as u16).bottom_margin(1)
            })
            .collect::<Vec<_>>();

        let t = Table::new(rows)
            .header(header)
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(selected_style)
            .highlight_symbol(">> ")
            .widths(&[
                Constraint::Percentage(10),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(45),
            ]);
        f.render_stateful_widget(t, area, &mut self.state);
    }
}
